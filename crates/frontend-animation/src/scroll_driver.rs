//! scroll ドライバ（`animation-timeline: view()` の機能検出 + 非対応時の
//! `scroll`/`resize` + rAF による progress 計算フォールバック、イシュー
//! #2521、親 `#2499`）。
//!
//! # 背景
//!
//! `#2499` は CSS `animation-timeline: view()` による scroll-driven reveal
//! （`fandhe-frontend-pre-styled-ui` の `SlotRecipe::scroll_reveal`）を
//! 実装済みだが、非対応ブラウザ向けの JS フォールバックは明示的に本
//! モジュールのスコープとして残された。本モジュールは「進入率（0.0〜1.0）
//! の計算」「DOM 計測（`getBoundingClientRect`/`innerHeight`）」「rAF ループ
//! 補助」のみを担う。`animation-timeline` の機能検出結果に応じてどの要素へ
//! 配線するか・イベントリスナーをどう登録するかは
//! `fandhe-frontend-wasm-full` の責務であり（`crates/wasm-full/src/
//! scroll_driver.rs`）、本モジュールはその配線から呼ばれる計算・計測
//! プリミティブを提供するのみである（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）。
//!
//! # 進捗 custom property の契約
//!
//! [`SCROLL_PROGRESS_PROPERTY`] へ書き込む値は要素のビューポート進入率
//! （0.0〜1.0）である。`crates/pre-styled-ui/` 側でこの変数を読む
//! `@supports not (animation-timeline: view())` フォールバック CSS の追加は
//! 本 issue のスコープ外（別クレート・別 publish チェーンを要するため）。

use std::cell::{Cell, RefCell};
use std::rc::Rc;

// wasm32 以外では `update_element_progress` が `Target::write` を呼ばない
// no-op 分岐のみコンパイルされるため（`raf_driver.rs` の
// `#[cfg(target_arch = "wasm32")] use wasm_bindgen::JsCast;` と同型）、
// この import も同じ cfg でガードする（未使用 import の警告を避ける）。
#[cfg(target_arch = "wasm32")]
use fandhe_animation::target::Target;
// `find_scroll_container` の `dyn_into::<web_sys::HtmlElement>()` に必要
// （同上の理由で wasm32 限定 import）。
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::dom_target::DomTarget;
use crate::raf_driver::AnimationLoop;

/// scroll-driven reveal の進捗を書き込む CSS カスタムプロパティ名。
///
/// `crates/pre-styled-ui/src/recipe.rs` の `SlotRecipe::scroll_reveal` が
/// 参照する `--fandhe-motion-scroll-reveal-distance`（距離のみ）とは別の
/// 変数であり、本モジュールが新規に定義する進捗専用の契約である。
pub const SCROLL_PROGRESS_PROPERTY: &str = "--fandhe-motion-scroll-progress";

/// `compute_progress` の純粋な進捗計算（DOM 非依存、native `cargo test` で
/// 検証可能）。
///
/// `animation-range: entry 0% entry 100%` を単純化した定義:
/// `progress = clamp01((viewport_height - rect_top) / rect_height)`。
///
/// - `rect_top == viewport_height`（要素の上端がビューポート下端に到達した
///   瞬間）で `progress == 0.0`
/// - `rect_top == viewport_height - rect_height`（要素の下端がビューポート
///   下端に到達した瞬間、要素高さぶん進入した状態）で `progress == 1.0`
/// - `rect_height <= 0.0`（高さ 0 の要素、またはゼロ割り回避）は既に
///   「入り終わっている」とみなし `1.0` を返す（パニックしない）
///
/// 要素がビューポートより高い場合の CSS `animation-range` 仕様準拠の
/// 厳密な entry/exit 重複域計算は扱わない既知の単純化である。
#[must_use]
pub fn compute_progress(rect_top: f64, rect_height: f64, viewport_height: f64) -> f64 {
    if rect_height <= 0.0 {
        return 1.0;
    }
    let raw = (viewport_height - rect_top) / rect_height;
    raw.clamp(0.0, 1.0)
}

/// 実行環境の判定結果（機能検出・`prefers-reduced-motion`）。
///
/// `wire_scroll_driver_with_env`（`crates/wasm-full/src/scroll_driver.rs`）が
/// テストからこの値を直接注入できるよう、[`Env::detect`] とは別に
/// [`Env::new`]（素のコンストラクタ）を公開する。headless Chrome は
/// `animation-timeline: view()` を既にネイティブ対応するため、フォール
/// バック経路の検証にはこの注入口が必須である。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Env {
    /// `CSS.supports("animation-timeline", "view()")` の結果（ネイティブ
    /// CSS 駆動に委譲できるかどうか）。
    pub scroll_timeline_supported: bool,
    /// `prefers-reduced-motion: reduce` の判定結果。
    pub reduced_motion: bool,
}

impl Env {
    /// テスト・明示指定用の素のコンストラクタ。
    #[must_use]
    pub fn new(scroll_timeline_supported: bool, reduced_motion: bool) -> Self {
        Self {
            scroll_timeline_supported,
            reduced_motion,
        }
    }

    /// 実ブラウザ環境から [`Env`] を検出する。
    ///
    /// `CSS.supports`/`matchMedia` の呼び出し自体が失敗する（API 不在・
    /// 例外）場合は、いずれも安全側（JS フォールバックを起動する・
    /// アニメーションを動かさない）へ fail-closed に倒す
    /// （security.md A05「不適切なセキュリティ設定」対応）。
    ///
    /// wasm32 以外のターゲット（native/SSR）では JS ホストが存在せず
    /// `web_sys::window()` 等の呼び出し自体が panic するため、`RafDriver::new`
    /// と同じ方針で JS 呼び出しを `cfg(target_arch = "wasm32")` でガードし、
    /// native では常に安全側デフォルト（非対応・reduced-motion）を返す。
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let scroll_timeline_supported =
                web_sys::css::supports_with_value("animation-timeline", "view()").unwrap_or(false);
            let reduced_motion = web_sys::window()
                .and_then(|window| {
                    window
                        .match_media("(prefers-reduced-motion: reduce)")
                        .ok()
                        .flatten()
                })
                .map(|list| list.matches())
                .unwrap_or(true);
            Self {
                scroll_timeline_supported,
                reduced_motion,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                scroll_timeline_supported: false,
                reduced_motion: true,
            }
        }
    }
}

/// `element` の祖先を遡り、最も近いスクロールコンテナ（`overflow-y` が
/// `visible`/`clip` 以外、かつ `scrollHeight > clientHeight` で実際に
/// スクロール可能な要素）を返す（PR #2557 codex-review P1 是正）。
///
/// ネイティブ `animation-timeline: view()` は要素の「近傍スクロール
/// ポート」（nearest scrollable ancestor）を基準に進捗を計算する。
/// JS フォールバック（[`update_element_progress`]）が常に
/// `window.innerHeight` を基準にしていると、要素がクリップされた
/// ネストしたスクロールコンテナ内にある場合に進捗を誤計算し、ネイティブ
/// 経路との契約（`crates/wasm-full/src/scroll_driver.rs` の doc
/// コメント「ネストしたスクロールコンテナの双方を捕捉できる」）と
/// 齟齬が生じるため、本関数で近傍コンテナを解決する。
///
/// `<html>`（`document.document_element()`）に到達した場合はページ全体
/// スクロールとみなし `None` を返す（`html` へ明示的に `overflow-y:
/// scroll` 等が設定されていても、実際の表示領域は viewport と一致する
/// ため、これを「ネストしたコンテナ」と誤認して
/// `documentElement.getBoundingClientRect()`〔文書全体の高さ〕を基準に
/// 使うと進捗が常に 1.0 付近へ張り付く誤計算になる）。
///
/// `<body>` は CSS Overflow Module Level 3 の overflow-propagation
/// （<https://www.w3.org/TR/css-overflow-3/#overflow-propagation>）に従い
/// **条件付きで**除外する（codex-review P1 是正、PR #2557）。`<html>` 自身
/// の `overflow-y` 計算値が `visible`（既定、明示指定なし）のときに限り
/// `<body>` の overflow は viewport（document のスクロール）へ伝播し、
/// `body.getBoundingClientRect()` はページスクロールに追従して動く固定
/// されない領域になるため除外が正しい。一方 `html { overflow: hidden }`
/// 等で `<html>` 自身に `visible` 以外の `overflow-y` が明示されている
/// 構成では伝播が起こらず（同仕様: 伝播は root の**指定値**が `visible`
/// の場合のみ）、`<body>` 自身が独立したスクロールコンテナになり得る
/// （例: `html { overflow: hidden } body { margin: 0; height: 300px;
/// overflow-y: auto }`）。この場合 `<body>` を無条件除外すると実際の
/// スクロール領域を見逃し `window.innerHeight` 基準の誤った進捗を書いて
/// しまうため、`<body>` も通常のスクロールコンテナ候補として
/// [`is_scroll_container`] の判定に乗せる。
///
/// `getComputedStyle`/`window`/`document` の取得に失敗した場合は
/// fail-closed に「伝播している」とみなし（安全側 = 従来どおり `<body>`
/// を除外し、誤ってネストコンテナと誤認しない）。
#[cfg(target_arch = "wasm32")]
fn find_scroll_container(element: &web_sys::Element) -> Option<web_sys::Element> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let root = document.document_element();
    let body: Option<web_sys::Element> = document.body().map(JsCast::unchecked_into);

    // overflow-propagation: `<html>` の `overflow-y` 計算値が `visible`
    // （既定）でなければ、`<body>` から viewport への伝播は起こらない。
    let root_overflow_propagates = root
        .as_ref()
        .and_then(|root_el| window.get_computed_style(root_el).ok().flatten())
        .map(|style| {
            let overflow_y = style.get_property_value("overflow-y").unwrap_or_default();
            matches!(overflow_y.as_str(), "visible" | "")
        })
        .unwrap_or(true);

    let mut current = element.parent_element();
    while let Some(candidate) = current {
        if root.as_ref() == Some(&candidate) {
            return None;
        }
        let is_body = body.as_ref() == Some(&candidate);
        if is_body && root_overflow_propagates {
            return None;
        }
        if is_scroll_container(&window, &candidate) {
            return Some(candidate);
        }
        current = candidate.parent_element();
    }
    None
}

/// `el` が実際にスクロール可能なコンテナかどうかを判定する
/// （[`find_scroll_container`] の走査述語）。
///
/// `overflow-y` の計算値が `visible`/`clip`（スクロールポートを生成
/// しない値）以外、かつ `scrollHeight > clientHeight`（実際に溢れて
/// いる）の両方を満たす場合のみコンテナとみなす。後者を課さないと、
/// `overflow-y: auto` だが中身が収まっている（スクロール不要な）要素も
/// 誤ってコンテナ扱いされ、`getBoundingClientRect()` の高さが
/// `window.innerHeight` とほぼ同義なだけの要素を無意味に基準へ使って
/// しまう。
#[cfg(target_arch = "wasm32")]
fn is_scroll_container(window: &web_sys::Window, el: &web_sys::Element) -> bool {
    let Ok(Some(style)) = window.get_computed_style(el) else {
        return false;
    };
    let overflow_y = style.get_property_value("overflow-y").unwrap_or_default();
    if matches!(overflow_y.as_str(), "visible" | "clip" | "") {
        return false;
    }
    let Ok(html_element) = el.clone().dyn_into::<web_sys::HtmlElement>() else {
        return false;
    };
    html_element.scroll_height() > html_element.client_height()
}

/// `element` の現在位置を計測し、[`compute_progress`] の結果を `target` へ
/// 書き込む（`fandhe-frontend-wasm-full` の scroll/resize リスナー・rAF
/// ループから毎フレーム呼ばれる想定）。
///
/// 基準とする表示領域は [`find_scroll_container`] が解決する最も近い
/// スクロールコンテナ（存在すればその `getBoundingClientRect()`）、
/// 無ければ `window.innerHeight`（ページ全体スクロール、従来どおり）
/// である（PR #2557 codex-review P1 是正）。祖先探索は呼び出しのたび
/// 毎フレーム行うため、動的なコンテナ構成変更（`overflow` の動的切替）
/// には追随するが、`getComputedStyle` 呼び出しコストが祖先段数ぶん
/// かかる既知のトレードオフである（`in_view.rs` の `MutationObserver`
/// 非対応と同種の割り切り）。
///
/// 計測に失敗した場合（`window` 不在等）は書き込みを行わず `None` を
/// 返す（fail-closed、panic しない）。
///
/// wasm32 以外のターゲットでは JS 呼び出し自体を伴わない no-op として
/// `None` を返す（`RafDriver::new`/`AnimationLoop::start` と同じ native
/// no-panic 方針）。
pub fn update_element_progress(element: &web_sys::Element, target: &mut DomTarget) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let rect = element.get_bounding_client_rect();
        let (reference_top, reference_height) = match find_scroll_container(element) {
            Some(container) => {
                // `getBoundingClientRect()` の高さは border box（border・
                // 横スクロールバー領域を含む）であり、実際に中身が見える
                // 表示領域（padding box、CSSOM View の `clientHeight`）より
                // 大きくなり得るため、進捗が実態より早く進み完了も早まる
                // （codex-review P1 指摘、PR #2557）。`clientTop`（border-top
                // 幅）・`clientHeight` を基準にする。変形（`transform`）が
                // 適用されたコンテナでは `clientHeight` 自体は変形の影響を
                // 受けないため厳密ではないが、`getBoundingClientRect()`
                // （視覚的位置、変形を反映）を起点に `clientTop` を加算する
                // ことで対象要素と同じ座標系を保ったまま表示領域を近似する
                // （既知の単純化、`compute_progress` の doc と同種の割り切り）。
                let container_rect = container.get_bounding_client_rect();
                let (client_top, client_height) = container
                    .clone()
                    .dyn_into::<web_sys::HtmlElement>()
                    .map(|html| {
                        (
                            f64::from(html.client_top()),
                            f64::from(html.client_height()),
                        )
                    })
                    .unwrap_or((0.0, container_rect.height()));
                (container_rect.top() + client_top, client_height)
            }
            None => {
                let viewport_height = web_sys::window()?.inner_height().ok()?.as_f64()?;
                (0.0, viewport_height)
            }
        };
        let progress =
            compute_progress(rect.top() - reference_top, rect.height(), reference_height);
        target.write(progress);
        Some(progress)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (element, target);
        None
    }
}

/// dirty-flag 方式で「scroll/resize イベント発火時のみ再計算する」rAF
/// ループを提供する（[`AnimationLoop`] を内部で再利用し、新規の rAF
/// プリミティブは増やさない）。
///
/// `mark_dirty()` を呼び出し側（`scroll`/`resize` イベントリスナー）から
/// 都度呼ぶだけで、実際の `recompute` 呼び出しは 1 フレームにつき最大 1 回
/// に抑えられる（大量のイベント発火に対する DoS 耐性、無制限コールバック
/// 蓄積の回避）。
///
/// native（非 wasm32）では [`AnimationLoop::start`] が no-op を返すため
/// `ScrollDriver` 自体も自然に no-op になる（追加の `#[cfg]` 分岐は不要）。
pub struct ScrollDriver {
    state: Rc<ScrollDriverState>,
}

/// [`ScrollDriver`] の共有可変状態（`mark_dirty`/rAF ステップ双方から
/// `Rc` 経由で参照される）。
struct ScrollDriverState {
    recompute: RefCell<Box<dyn FnMut()>>,
    dirty: Cell<bool>,
    /// rAF ループが現在フレーム要求中かどうか（Bugbot 指摘、PR #2557
    /// codex-review コメント: 常時 `true` を返すと `dirty == false` でも
    /// rAF ループが恒久稼働してしまう）。`false` になった後は
    /// `mark_dirty` が新しい [`AnimationLoop`] を起動して再開する。
    running: Cell<bool>,
    loop_: RefCell<Option<AnimationLoop>>,
}

impl ScrollDriver {
    /// `recompute` を dirty なフレームのみ呼ぶループを開始する。
    ///
    /// 開始直後の 1 フレーム目は無条件に `recompute` を呼ぶ（初期スクロール
    /// 位置に対応する進捗を、最初の `scroll`/`resize` イベントを待たずに
    /// 反映するため）。
    ///
    /// 1 フレーム連続で dirty が立っていなければループを停止し（アイドル
    /// 化）、以後は `mark_dirty()` が新しい rAF ループを起動して再開する
    /// （scroll/resize が起きていない間は無限に `requestAnimationFrame`
    /// を消費しない）。
    pub fn start(recompute: impl FnMut() + 'static) -> Self {
        let state = Rc::new(ScrollDriverState {
            recompute: RefCell::new(Box::new(recompute)),
            dirty: Cell::new(true),
            running: Cell::new(true),
            loop_: RefCell::new(None),
        });
        let loop_ = Self::spawn_loop(&state);
        *state.loop_.borrow_mut() = Some(loop_);
        Self { state }
    }

    /// `state` を基準に新しい [`AnimationLoop`] を 1 本起動する
    /// （`start`・アイドルからの再開〔`mark_dirty`〕の双方から呼ばれる
    /// 共通経路）。
    fn spawn_loop(state: &Rc<ScrollDriverState>) -> AnimationLoop {
        let state_for_step = state.clone();
        AnimationLoop::start(move || {
            if state_for_step.dirty.replace(false) {
                (state_for_step.recompute.borrow_mut())();
                true
            } else {
                // 2 フレーム連続で dirty が立たなかった（今フレームで
                // 何もせず false を返すのみ）: このクロージャ自身が
                // 属する `AnimationLoop`（`state.loop_` の中身）を
                // ここで drop しない（`raf_driver.rs::AnimationLoop::stop`
                // の doc が警告する「実行中の Closure を call_mut 実行中
                // に drop すると use-after-free」を避けるため）。単に
                // `false` を返して次フレーム予約をやめるだけに留め、
                // `state.loop_` 自体のクリア・再生成は `mark_dirty`
                // （scroll/resize イベントリスナー由来の別コールスタック）
                // 側に委ねる。
                state_for_step.running.set(false);
                false
            }
        })
    }

    /// 次フレームで `recompute` が呼ばれるようフラグを立てる（`scroll`/
    /// `resize` イベントリスナーから呼ばれる）。
    ///
    /// ループが既にアイドル化している場合（`running == false`）は
    /// 新しい [`AnimationLoop`] を起動して再開する。ここで古い（既に
    /// 停止済みの） `AnimationLoop` を新しいものへ差し替える `drop` が
    /// 発生するが、この呼び出しはイベントリスナーのコールスタックであり
    /// rAF コールバック自身の `call_mut` 実行中ではないため、`drop`
    /// （`AnimationLoop::stop` 経由の `Closure` 解放）は安全である。
    pub fn mark_dirty(&self) {
        self.state.dirty.set(true);
        if !self.state.running.replace(true) {
            let loop_ = Self::spawn_loop(&self.state);
            *self.state.loop_.borrow_mut() = Some(loop_);
        }
    }

    /// ループを明示停止する（[`ScrollDriver`] の `Drop` 実装からも呼ばれるが、
    /// 要素の動的除去等に伴う早期停止用に公開する）。
    ///
    /// `state.loop_` から [`AnimationLoop`] を `take` して局所変数へ移し
    /// てから明示的に `stop()` を呼ぶ（`take` 自体で `RefCell` 内は
    /// `None` になる）。これにより rAF コールバックが保持する
    /// `state_for_step: Rc<ScrollDriverState>`（`spawn_loop` 参照）を含む
    /// `Closure` が解放され、`state → loop_ → AnimationLoop → Closure →
    /// state` の強参照循環が断ち切られる（codex-review P1 指摘、PR
    /// #2557）。循環が残ったままだと [`ScrollDriver`] 自体を drop しても
    /// `state` の参照カウントが 0 にならず `AnimationLoop::drop` に到達
    /// せず、`recompute` に捕捉された DOM・状態はもちろん rAF ループ
    /// 自体も動き続ける（Bugbot 指摘、PR #2557、`raf_driver::AnimationLoop`
    /// の `Drop` 実装参照）。
    pub fn stop(&self) {
        self.state.running.set(false);
        if let Some(loop_) = self.state.loop_.borrow_mut().take() {
            loop_.stop();
        }
    }
}

impl Drop for ScrollDriver {
    /// [`ScrollDriver`] が破棄されたタイミングで rAF ループを停止し、
    /// `state` の強参照循環を断ち切る（`stop()` の doc 参照）。利用者が
    /// 明示的に `stop()` を呼ばずに [`ScrollDriver`] を単に drop した
    /// 場合（例: 要素の動的除去に伴うハンドル破棄）でも「Drop でも
    /// 停止する」契約（モジュール doc・`stop()` doc）を満たす（codex-review
    /// P1 / Bugbot 指摘、PR #2557）。
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod compute_progress_tests {
    use super::compute_progress;

    #[test]
    fn below_viewport_is_zero() {
        // 要素の上端がビューポート下端よりさらに下（未進入）。
        assert_eq!(compute_progress(2000.0, 100.0, 800.0), 0.0);
    }

    #[test]
    fn top_at_viewport_bottom_is_zero() {
        assert_eq!(compute_progress(800.0, 100.0, 800.0), 0.0);
    }

    #[test]
    fn fully_entered_is_one() {
        // rect_top == viewport_height - rect_height で要素高さぶん進入完了。
        assert_eq!(compute_progress(700.0, 100.0, 800.0), 1.0);
    }

    #[test]
    fn past_full_entry_clamped_to_one() {
        assert_eq!(compute_progress(0.0, 100.0, 800.0), 1.0);
    }

    #[test]
    fn halfway_is_half() {
        assert_eq!(compute_progress(750.0, 100.0, 800.0), 0.5);
    }

    #[test]
    fn zero_height_is_one() {
        assert_eq!(compute_progress(500.0, 0.0, 800.0), 1.0);
    }

    #[test]
    fn negative_height_is_one_and_does_not_panic() {
        assert_eq!(compute_progress(500.0, -10.0, 800.0), 1.0);
    }
}

/// native（非 wasm32）実行時に `Env::detect`/`update_element_progress`/
/// `ScrollDriver::start` が panic せず安全側の値・no-op を返すことを固定
/// する回帰テスト（`raf_driver::native_no_panic_tests` と同型）。
#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_no_panic_tests {
    use super::{Env, ScrollDriver};

    #[test]
    fn env_detect_returns_safe_defaults_on_native() {
        let env = Env::detect();
        assert!(!env.scroll_timeline_supported);
        assert!(env.reduced_motion);
    }

    #[test]
    fn scroll_driver_start_is_noop_on_native() {
        let driver = ScrollDriver::start(|| {});
        driver.mark_dirty();
        driver.stop();
    }
}
