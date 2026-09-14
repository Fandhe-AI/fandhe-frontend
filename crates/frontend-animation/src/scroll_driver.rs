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
    compute_progress_for_range(rect_top, rect_height, viewport_height, ProgressRange::Entry)
}

/// [`compute_progress`]/[`compute_progress_for_range`] が計算する進捗の
/// 区間（イシュー #2534）。
///
/// CSS Scroll-driven Animations の `view-timeline-range`（`entry`/`cover`/
/// `contain`）の命名意図に対応する（本モジュールは `entry`/`exit` を
/// 単純化して 1 本の進捗値として扱うため、`exit`/`entry-crossing`/
/// `exit-crossing` は持たない）。
///
/// `crates/wasm-full/src/scroll_driver.rs` の
/// `data-fandhe-scroll-progress` 属性値（`""`/`"entry"`/`"cover"`/
/// `"contain"`）が本 enum へ厳格一致で変換される（未知値は `Entry`
/// へ fail-closed、REQ-1/A03 の「動的文字列をセレクタ・プロパティ名へ
/// 混ぜない」不変条件をこの変換層で満たす）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressRange {
    /// 要素が侵入し始め（上端がビューポート下端に到達）～自身の高さぶん
    /// 侵入完了（下端がビューポート下端に到達）で 0→1（既存 `compute_
    /// progress` の定義、`fandhe-frontend-pre-styled-ui` の
    /// `SlotRecipe::scroll_reveal` が対応するネイティブ範囲
    /// `animation-range: entry 0% entry 100%`）。
    #[default]
    Entry,
    /// 要素がビューポートへ入り始めてから完全に出終わるまでの全期間で
    /// 0→1（`SlotRecipe::parallax` が対応するネイティブ範囲
    /// `animation-range: cover 0% cover 100%`）。
    Cover,
    /// 要素がビューポートに完全に収まっている期間で 0→1
    /// （`SlotRecipe::sticky_progress` が対応するネイティブ範囲
    /// `animation-range: contain 0% contain 100%`）。
    Contain,
}

/// [`compute_progress`] の範囲拡張版（DOM 非依存、native `cargo test` で
/// 検証可能、イシュー #2534）。`compute_progress` はこの関数の
/// `ProgressRange::Entry` 固定ラッパである。
///
/// 座標系は `compute_progress` と同じ: `rect_top` はビューポート上端
/// からの相対位置（下方向が正）、`rect_height`/`viewport_height` は
/// 非負を想定する要素・ビューポートの高さ。
///
/// # `Cover`
///
/// 0% は要素上端がビューポート下端に到達した瞬間（`Entry` 0% と同一）、
/// 100% は要素下端がビューポート上端に到達した瞬間（要素が完全に
/// 通過し終えた瞬間）。`progress = (viewport_height - rect_top) /
/// (viewport_height + rect_height)`。分母が 0 以下（両方の高さが 0）の
/// 場合は「既に通過済み」とみなし `1.0` を返す（`compute_progress` の
/// `rect_height <= 0.0` ガードと同型の安全側フォールバック）。
///
/// # `Contain`
///
/// 要素がビューポートより低い（`rect_height <= viewport_height`）場合に
/// 定義される区間: 0% は要素が初めて完全にビューポート内へ収まった瞬間
/// （`Entry` 100% と同一）、100% は要素上端がビューポート上端に到達し
/// 完全収容が終わる瞬間。`progress = (viewport_height - rect_height -
/// rect_top) / (viewport_height - rect_height)`。
///
/// 要素がビューポート以上に高い（`rect_height >= viewport_height`）場合、
/// CSS 仕様上 `contain` 区間は退化する（要素が一度も完全収容されない）。
/// 本関数は `Cover` と同じ計算へフォールバックする既知の単純化を採る
/// （進捗が常に一定値に張り付くより、スクロールに連動し続ける方が
/// 「進捗表示」としての実用性が高いという判断。`compute_progress` 自身の
/// 「既知の単純化」doc と同じ性質の割り切り）。
#[must_use]
pub fn compute_progress_for_range(
    rect_top: f64,
    rect_height: f64,
    viewport_height: f64,
    range: ProgressRange,
) -> f64 {
    match range {
        ProgressRange::Entry => {
            if rect_height <= 0.0 {
                return 1.0;
            }
            ((viewport_height - rect_top) / rect_height).clamp(0.0, 1.0)
        }
        ProgressRange::Cover => {
            let denom = viewport_height + rect_height;
            if denom <= 0.0 {
                return 1.0;
            }
            ((viewport_height - rect_top) / denom).clamp(0.0, 1.0)
        }
        ProgressRange::Contain => {
            let denom = viewport_height - rect_height;
            if denom <= 0.0 {
                // 要素がビューポート以上に高い: `contain` 区間が退化する
                // ため `Cover` の計算へフォールバックする（doc 参照）。
                return compute_progress_for_range(
                    rect_top,
                    rect_height,
                    viewport_height,
                    ProgressRange::Cover,
                );
            }
            ((denom - rect_top) / denom).clamp(0.0, 1.0)
        }
    }
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

/// `element` の祖先を遡り、最も近いスクロールコンテナ（`overflow-y` の
/// 計算値が `visible`/`clip` 以外の要素）を返す（PR #2557 codex-review P1
/// 是正）。
///
/// コンテナか否かの判定は `overflow-y` の計算値のみで行い、実際に
/// スクロール可能か（`scrollHeight > clientHeight`）は問わない（後述の
/// [`is_scroll_container`] 参照）。
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

/// `el` が CSS の意味での「スクロールコンテナ」かどうかを判定する
/// （[`find_scroll_container`] の走査述語）。
///
/// `overflow-y` の計算値が `visible`/`clip`（スクロールポートを生成
/// しない値）以外であればコンテナとみなす（CSS Overflow Module Level 3
/// のスクロールコンテナ定義
/// <https://www.w3.org/TR/css-overflow-3/#scroll-container>）。
///
/// **意図的に `scrollHeight > clientHeight`（実際に溢れているか）は
/// 問わない**（PR #2557 codex-review P1 是正）。`animation-timeline:
/// view()` の仕様（CSS Scroll-driven Animations の view-notation
/// <https://drafts.csswg.org/scroll-animations-1/#view-notation>）が
/// 参照する「近傍スクロールコンテナ」は overflow 特性のみで決まり、
/// 現時点でスクロール範囲を持つかどうかには依存しない。実際に溢れて
/// いるかを条件へ含めると、内容が収まっている（オーバーフローして
/// いない）`overflow: hidden`/`auto`/`scroll` の祖先を「コンテナでは
/// ない」として読み飛ばし、さらに外側のコンテナや `window` を基準に
/// 進捗を計算してしまう（例: 高さ 300px の `overflow: hidden` 要素内に
/// 高さ 100px の対象要素が収まっている場合、ページ全体のスクロールで
/// 進捗が 0→1 へ変化してしまい、ネイティブ経路〔対象要素と当該コンテナ
/// の相対位置は変わらないため進捗は一定〕と乖離する）。
///
/// スクロール範囲が無いコンテナが基準に選ばれた場合、[`compute_progress`]
/// は対象要素とコンテナの相対位置のみから進捗を計算するため、両者が
/// ページスクロールに対し常に同じ相対位置を保つ限り進捗は一定値
/// （典型的には対象要素がコンテナ内に収まっている＝進入済みとみなせる
/// `1.0` 付近）に留まる。これは「スクロール範囲が無いタイムラインは
/// 進行しない」というネイティブ挙動の近似として妥当な結果であり、
/// 本関数側で追加の分岐（範囲が無ければ除外する等）は行わない。
#[cfg(target_arch = "wasm32")]
fn is_scroll_container(window: &web_sys::Window, el: &web_sys::Element) -> bool {
    let Ok(Some(style)) = window.get_computed_style(el) else {
        return false;
    };
    let overflow_y = style.get_property_value("overflow-y").unwrap_or_default();
    !matches!(overflow_y.as_str(), "visible" | "clip" | "")
}

/// [`update_element_progress_for_range`] が計測直前に呼ぶ、フィードバック
/// ループ除去のための計測ヘルパ（codex-review P1 是正、PR #2563）。
///
/// # 背景（フィードバックループ）
///
/// `SlotRecipe::parallax`/`SlotRecipe::sticky_progress` のフォールバック
/// CSS（`@supports not (animation-timeline: view())`）は、本モジュールが
/// [`SCROLL_PROGRESS_PROPERTY`] へ書き込んだ進捗を読んで**同じ要素**へ
/// `translate`（parallax）/`scale`（sticky_progress）を適用する。計測
/// （`getBoundingClientRect()`）にその効果適用後の座標をそのまま使うと、
/// 次フレームの計測値が前フレームの効果適用結果に依存してしまい、同じ
/// スクロール位置でも進捗値がフレームを追うごとにずれ続ける
/// （[`compute_progress_for_range`] が前提とする「スクロール位置に対する
/// 線形補間」契約に反する）。
///
/// `position: sticky`（`sticky_progress` の典型的な利用形、呼び出し側
/// マークアップの責務）も同種の問題を起こす: ピン留め中は
/// `getBoundingClientRect().top` が一定値に張り付くため、スクロールを
/// 続けても `contain` 進捗が進まない。
///
/// # 是正方法
///
/// 計測直前に `translate`/`scale` を `none` へ、算出済みスタイルが
/// `position: sticky` の場合に限り `position` を `static` へ一時上書き
/// してから `getBoundingClientRect()` を呼び、直後に（同期的に・
/// 再描画を挟まず）元の値へ戻す。これにより計測結果は常に「本モジュール
/// 自身の効果適用前・ピン留め前」の、スクロール位置と連続的に対応する
/// 位置を表す。`position: fixed`/`absolute` 等 `sticky` 以外の値は変更
/// しない（無関係な計測結果を変えないため）。
///
/// 一時上書きは [`CssStyleDeclaration::set_property_with_priority`] で
/// 優先度 `"important"` を明示して書く（codex-review P1 是正、
/// PR #2563）。インラインスタイルは通常優先度であれば常にスタイルシートの
/// セレクタ規則に勝つが、スタイルシート側が `!important`（例:
/// `position: sticky !important`）を宣言している場合は通常優先度の
/// インライン上書きでは効かず、変形適用後・ピン留め後の座標をそのまま
/// 計測してしまう（`position: sticky !important` 環境下では
/// `unpinned_rect.top() == top_offset` が常に成立し、`contain` 進捗が
/// 常に 0 に固定される不具合を招く）。一時上書き自体を `!important` で
/// 書くことでスタイルシート側の `!important` 宣言よりも常に勝つように
/// する（下記「優先度の保存・復元」の通り、元の宣言の優先度は変えない）。
///
/// `transition`（例: `transition: translate 200ms`）が `translate`/
/// `scale`/`position` と併用されている場合、値としての `none`/`static`
/// への一時上書きは即時反映されても、遷移アニメーションが有効なままだと
/// 実際の描画・`getBoundingClientRect()` の戻り値は遷移の途中値（直前
/// フレームの変形が残った値）になり得るため、上記 3 プロパティを上書き
/// する**前**に `transition-property`（**shorthand ではなく longhand**、
/// codex-review P1 是正、PR #2563）を `!important` で `none` へ一時上書き
/// して遷移を同期的に無効化してから計測する。`transition-property: none`
/// は CSS Transitions の仕様上それだけで全プロパティの遷移を無効化できる
/// ため、`transition-duration`/`transition-timing-function`/
/// `transition-delay` の他 longhand には一切触れない。**shorthand
/// `transition` の `get_property_value`/`remove_property` に頼ると、要素が
/// `style="transition-duration: 200ms"` のような個別 longhand 宣言のみを
/// 持つ場合に `get_property_value("transition")` が空文字列を返し
/// （shorthand 宣言自体が存在しないため）、後続の shorthand 上書き
/// （`set_property_with_priority("transition", "none", "important")`）が
/// 元の longhand 宣言を実質的に破棄し、復元時の
/// `remove_property("transition")` が `transition-duration` を含む全
/// longhand を恒久的に消してしまう（計測のための一時上書きが利用者の
/// アニメーション設定を永続的に変更してしまう回帰）。`transition-property`
/// 単体の longhand として保存・上書き・復元することで、他の longhand
/// （`-duration`/`-timing-function`/`-delay`）を一切書き換えず、shorthand
/// 経由の暗黙の正規化・破棄を避ける。復元順序は `translate`/`scale`/
/// `position` を先に元へ戻し、`transition-property` は最後に戻す
/// （復元中も `transition-property: none` のままにすることで、復元そのもの
/// が新たな遷移の開始点にならないようにする）。
///
/// インラインスタイルの上書き・復元は同一の同期実行内で完結するため、
/// ブラウザが中間状態を描画することはない（強制リフローを伴う計測
/// ヘルパの一般的な手法。`getComputedStyle` 呼び出しコストは
/// [`is_scroll_container`] と同種の既知のトレードオフ）。
///
/// `element` が `HtmlElement`（インラインスタイル設定可能）でない場合
/// （SVG 等）は素の `getBoundingClientRect()` へフォールバックする
/// （fail-open: 計測結果が変形の影響を受け得るが、少なくとも panic
/// しない）。
///
/// 戻り値の `bool` は計測直前の算出済みスタイルが `position: sticky`
/// だったかどうか（[`update_element_progress_for_range`] が
/// [`ProgressRange::Contain`] のピン留め区間考慮計算へ分岐するために
/// 使う、codex-review P1 是正、PR #2563）。
///
/// # インラインスタイルの優先度（`!important`）保存・復元（codex-review
/// P1 是正、PR #2563）
///
/// 一時上書き前に [`CssStyleDeclaration::get_property_priority`] で
/// 各プロパティの優先度（`""` または `"important"`）も保存し、復元時は
/// [`CssStyleDeclaration::set_property_with_priority`] で渡す。単純な
/// `set_property`（優先度は常に空文字列扱い）で復元すると、元のインライン
/// 宣言が `!important` を持っていた場合に計測後の復元値から `!important`
/// が失われ、競合するスタイルシート側の重要宣言がある環境で表示・配置が
/// 恒久的に変化してしまう（計測は同期的な一時上書きのはずが副作用を
/// 残すバグ）。
#[cfg(target_arch = "wasm32")]
fn measure_untransformed_rect(element: &web_sys::Element) -> (web_sys::DomRect, bool) {
    let Some(html) = element.dyn_ref::<web_sys::HtmlElement>() else {
        return (element.get_bounding_client_rect(), false);
    };
    let style = html.style();

    // `transition-property` を最初に `!important` で `none` へ無効化する
    // （codex-review P1 是正、PR #2563）。以降で上書きする `translate`/
    // `scale`/`position` へ `transition: translate 200ms` 等が併用されて
    // いると、`none`/`static` への一時上書きは値としては即時反映される
    // が実際の描画・`getBoundingClientRect()` の戻り値は遷移アニメーション
    // の現在値（直前フレームの変形が残った途中値）になり得るため、遷移
    // 自体を同期的に止めてから計測する。**shorthand `transition` ではなく
    // longhand `transition-property` のみを保存・上書き・復元する**（要素
    // が `style="transition-duration: 200ms"` のような longhand 宣言のみ
    // を持つ場合、shorthand `transition` は宣言として存在せず
    // `get_property_value("transition")` は空文字列を返すため、shorthand
    // 経由の save/restore では longhand 宣言が復元時に丸ごと消え、利用者
    // のアニメーション設定を恒久的に変更してしまう回帰がある）。
    // `transition-property: none` は CSS Transitions の仕様上それだけで
    // 全プロパティの遷移を無効化できるため、`transition-duration`/
    // `transition-timing-function`/`transition-delay` には一切触れない。
    // スタイルシート側が `transition-property: ... !important` を宣言
    // していても本上書きが必ず勝つよう `!important` で書く（下記
    // `translate`/`scale`/`position` と同じ理由）。
    let saved_transition_property = style.get_property_value("transition-property").ok();
    let saved_transition_property_priority = style.get_property_priority("transition-property");
    let _ = style.set_property_with_priority("transition-property", "none", "important");

    // 一時上書きは `!important` で行う（codex-review P1 是正、PR #2563）。
    // スタイルシート側に `translate`/`scale`/`position` の `!important`
    // 宣言があると、通常優先度の上書きでは効かず（`!important` は
    // インラインスタイルの高い詳細度よりも優先される CSS の仕様）、
    // 変形適用後・ピン留め後の座標をそのまま計測してしまう
    // （`position: sticky !important` 環境で `contain` 進捗が常に 0 に
    // 固定される不具合の原因）。復元側は保存済みの元の優先度
    // （`saved_*_priority`）で書き戻すため、ここで `!important` を使っても
    // 元の宣言の優先度は変えない。
    let saved_translate = style.get_property_value("translate").ok();
    let saved_translate_priority = style.get_property_priority("translate");
    let _ = style.set_property_with_priority("translate", "none", "important");
    let saved_scale = style.get_property_value("scale").ok();
    let saved_scale_priority = style.get_property_priority("scale");
    let _ = style.set_property_with_priority("scale", "none", "important");

    let is_sticky = web_sys::window()
        .and_then(|window| window.get_computed_style(element).ok().flatten())
        .and_then(|computed| computed.get_property_value("position").ok())
        .map(|value| value == "sticky")
        .unwrap_or(false);
    let saved_position = if is_sticky {
        let prev = style.get_property_value("position").ok();
        let prev_priority = style.get_property_priority("position");
        let _ = style.set_property_with_priority("position", "static", "important");
        Some((prev, prev_priority))
    } else {
        None
    };

    let rect = element.get_bounding_client_rect();

    // 復元順序: `transition-property` は最後に戻す。`translate`/`scale`/
    // `position` を元の値へ戻す間は `transition-property: none` のままに
    // しておくことで、復元そのものが新たな遷移アニメーションの開始点に
    // ならないようにする（復元直後に `transition-property` を戻すと、
    // 以降のスクロール起因の再計測ではない通常のスタイル変化は正しく
    // 遷移する）。`transition-duration`/`transition-timing-function`/
    // `transition-delay` は本関数の一時上書きの対象外であり、常に元の
    // 値のまま変更されない。
    restore_inline_property(
        &style,
        "translate",
        saved_translate.as_deref(),
        &saved_translate_priority,
    );
    restore_inline_property(
        &style,
        "scale",
        saved_scale.as_deref(),
        &saved_scale_priority,
    );
    if let Some((prev, prev_priority)) = saved_position {
        restore_inline_property(&style, "position", prev.as_deref(), &prev_priority);
    }
    restore_inline_property(
        &style,
        "transition-property",
        saved_transition_property.as_deref(),
        &saved_transition_property_priority,
    );

    (rect, is_sticky)
}

/// [`measure_untransformed_rect`] の一時上書き復元を 1 プロパティぶん
/// 行う共通ヘルパ（`translate`/`scale`/`position` の 3 箇所で同型の
/// 分岐を重複させないため、codex-review P1 是正、PR #2563）。
///
/// `value` が計測前に値を持っていれば `priority` 付きで復元し（元の
/// `!important` を保つ）、値が無かった／空文字列だった場合は
/// `remove_property` で宣言ごと取り除く（計測前に存在しなかった
/// プロパティを空文字列で新規に生やさない）。
#[cfg(target_arch = "wasm32")]
fn restore_inline_property(
    style: &web_sys::CssStyleDeclaration,
    name: &str,
    value: Option<&str>,
    priority: &str,
) {
    match value {
        Some(v) if !v.is_empty() => {
            let _ = style.set_property_with_priority(name, v, priority);
        }
        _ => {
            let _ = style.remove_property(name);
        }
    }
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
/// `element` 自身の計測は [`measure_untransformed_rect`] 経由で行い、
/// 本関数が過去に書き込んだ進捗（`SCROLL_PROGRESS_PROPERTY`）を読んで
/// 同じ要素へ `translate`/`scale` を適用するフォールバック CSS
/// （`SlotRecipe::parallax`/`SlotRecipe::sticky_progress`）や
/// `position: sticky` のピン留めが計測結果へ混入しないようにする
/// （codex-review P1 是正、PR #2563。詳細は同関数 doc 参照）。
///
/// 計測に失敗した場合（`window` 不在等）は書き込みを行わず `None` を
/// 返す（fail-closed、panic しない）。
///
/// wasm32 以外のターゲットでは JS 呼び出し自体を伴わない no-op として
/// `None` を返す（`RafDriver::new`/`AnimationLoop::start` と同じ native
/// no-panic 方針）。
pub fn update_element_progress(element: &web_sys::Element, target: &mut DomTarget) -> Option<f64> {
    update_element_progress_for_range(element, target, ProgressRange::Entry)
}

/// [`update_element_progress`] の範囲拡張版（イシュー #2534）。
/// `update_element_progress` はこの関数の [`ProgressRange::Entry`] 固定
/// ラッパである。
///
/// `crates/wasm-full/src/scroll_driver.rs` が `data-fandhe-scroll-progress`
/// 属性値から解決した [`ProgressRange`] を渡す（属性値 → `ProgressRange`
/// の厳格一致変換自体は wasm-full 側の責務、本関数は既に解決済みの
/// `range` を受け取るのみ）。
pub fn update_element_progress_for_range(
    element: &web_sys::Element,
    target: &mut DomTarget,
    range: ProgressRange,
) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let (rect, is_sticky) = measure_untransformed_rect(element);
        let scroll_container = find_scroll_container(element);
        let (reference_top, reference_height) = match &scroll_container {
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
        // `position: sticky` 要素の `Contain` 進捗はピン留め区間そのもの
        // （codex-review P1 是正、PR #2563。詳細は [`sticky_contain_pin_progress`]
        // 参照）。ネストしたスクロールコンテナ配下の sticky 要素はこの
        // 単純化の対象外とし（`scroll_container.is_none()` 限定）、通常の
        // `compute_progress_for_range` 計算へフォールバックする（既知の
        // 単純化、`docs/design/motion-reference-adoption-policy.md` §6 と
        // 同種の割り切り。ページ全体スクロール前提の本イシューのテスト
        // 範囲を超えるため）。
        let sticky_progress =
            if is_sticky && range == ProgressRange::Contain && scroll_container.is_none() {
                sticky_contain_pin_progress(element, &rect)
            } else {
                None
            };
        let progress = match sticky_progress {
            Some(value) => value,
            None => compute_progress_for_range(
                rect.top() - reference_top,
                rect.height(),
                reference_height,
                range,
            ),
        };
        target.write(progress);
        Some(progress)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (element, target, range);
        None
    }
}

/// `position: sticky` 要素の [`ProgressRange::Contain`] を「ピン留め区間」
/// として計算する（codex-review P1 是正、PR #2563）。
///
/// # 是正前の不具合
///
/// [`measure_untransformed_rect`] は sticky 要素を計測直前に一時的に
/// `position: static` へ戻し「ピン留めされていない場合の自然な位置」を
/// 得る。この非ピン留め位置はスクロールにつれて連続的に動き続けるため、
/// `Contain` の式（`(viewport_height - rect_height - rect_top) /
/// (viewport_height - rect_height)`）へそのまま渡すと、ピン留め開始
/// 直後には非ピン留め位置の `rect_top` が急速に負の大きな値へ進んでしまい
/// 分子が分母を超えて即座に `1.0` へクランプされる（ピン留め中ずっと
/// 進捗が変化しない契約違反）。
///
/// # 是正方法
///
/// `Contain` が意図する「要素がビューポートに完全収容されている期間」は
/// sticky 要素にとってまさに「ピン留めされている期間」そのものである。
/// そこで非ピン留め位置ではなく、CSS Position スペックの sticky 配置式
/// （<https://www.w3.org/TR/css-position-3/#sticky-pos>）に基づき、
/// ピン留めの開始・終了をスクロールオフセットの区間として直接計算する:
///
/// - ピン留め開始（`pin_start`）: 要素の非ピン留め・文書相対な上端位置
///   （[`measure_untransformed_rect`] が返す `rect`）が `top` オフセット
///   と一致するスクロール位置
/// - ピン留め終了（`pin_end`）: 包含ブロック（[`Element::parent_element`]
///   で近似する既知の単純化、下記参照）の下端から要素高さを引いた位置が
///   `top` オフセットと一致するスクロール位置
///
/// `progress = clamp01((scroll_y - pin_start) / (pin_end - pin_start))`。
/// ピン留め開始前は `scroll_y < pin_start` のため `0.0` に、ピン留め終了後
/// （要素が包含ブロック下端に押し出され再び通常フローへ戻る）は `1.0` に
/// クランプされる。
///
/// # 既知の単純化
///
/// - 包含ブロックは `element.parent_element()` で近似する（sticky の
///   厳密な包含ブロックは「スクロール可能な祖先の padding box と直近の
///   ブロックコンテナ祖先」の交差だが、`crates/pre-styled-ui` の
///   `sticky_progress` 利用形は素の親要素配下へ直接ピン留め要素を
///   置く構成のみを想定するため、単純化として妥当）。
/// - `window.scroll_y()`（ページ全体スクロール）のみを扱う。呼び出し元
///   ([`update_element_progress_for_range`]) がネストしたスクロール
///   コンテナ配下ではこの関数を呼ばず通常計算へフォールバックする。
/// - `top` の単位は `px` のみ対応（`%`/`calc()` 等は `0.0` 扱いへ
///   フォールバック、fail-closed に「常にどこかへ収まる」進捗を返す）。
///
/// `window`/`document` 取得失敗・包含ブロック不在・ピン留め区間が退化
/// （`pin_end <= pin_start`、包含ブロックが要素の非ピン留め位置より
/// 低い異常構成）の場合は `None` を返し、呼び出し元が通常計算へ
/// フォールバックする。
#[cfg(target_arch = "wasm32")]
fn sticky_contain_pin_progress(
    element: &web_sys::Element,
    unpinned_rect: &web_sys::DomRect,
) -> Option<f64> {
    let window = web_sys::window()?;
    let scroll_y = window.scroll_y().ok()?;

    let top_offset = window
        .get_computed_style(element)
        .ok()
        .flatten()
        .and_then(|computed| computed.get_property_value("top").ok())
        .and_then(|value| parse_px_value(&value))
        .unwrap_or(0.0);

    let parent = element.parent_element()?;
    let parent_bottom_doc = parent.get_bounding_client_rect().bottom() + scroll_y;

    let rect_height = unpinned_rect.height();
    let static_top_doc = unpinned_rect.top() + scroll_y;

    let pin_start = static_top_doc - top_offset;
    let pin_end = parent_bottom_doc - rect_height - top_offset;
    let pin_total = pin_end - pin_start;

    if pin_total <= 0.0 {
        return None;
    }

    Some(((scroll_y - pin_start) / pin_total).clamp(0.0, 1.0))
}

/// `"12px"` のような CSS `<length>` の px 表現を数値へ変換する
/// （[`sticky_contain_pin_progress`] が `top` の算出値を読むために使う）。
/// `%`/`calc()`/`auto` 等 px 以外の表現は `None` を返す
/// （呼び出し元が既定値 `0.0` へフォールバックする）。
#[cfg(target_arch = "wasm32")]
fn parse_px_value(value: &str) -> Option<f64> {
    value.strip_suffix("px")?.trim().parse::<f64>().ok()
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

/// [`ProgressRange::Cover`]/[`ProgressRange::Contain`] の端点・中間値・
/// 退化ケースを固定する（イシュー #2534）。`compute_progress_tests` の
/// 既存 7 件（`Entry` 経路）は無改変のまま green を保つ回帰ガードとして
/// 別モジュールに分離する。
#[cfg(test)]
mod compute_progress_for_range_tests {
    use super::{compute_progress, compute_progress_for_range, ProgressRange};

    #[test]
    fn entry_range_matches_compute_progress_wrapper() {
        // `compute_progress` は `ProgressRange::Entry` の薄いラッパである
        // ことを直接確認する（リファクタの回帰ガード）。
        for (rect_top, rect_height, viewport_height) in [
            (2000.0, 100.0, 800.0),
            (700.0, 100.0, 800.0),
            (500.0, 0.0, 800.0),
        ] {
            assert_eq!(
                compute_progress(rect_top, rect_height, viewport_height),
                compute_progress_for_range(
                    rect_top,
                    rect_height,
                    viewport_height,
                    ProgressRange::Entry
                )
            );
        }
    }

    #[test]
    fn cover_starts_at_zero_when_top_edge_reaches_viewport_bottom() {
        // cover 0% は entry 0% と同一（要素上端がビューポート下端に到達）。
        assert_eq!(
            compute_progress_for_range(800.0, 100.0, 800.0, ProgressRange::Cover),
            0.0
        );
    }

    #[test]
    fn cover_ends_at_one_when_bottom_edge_reaches_viewport_top() {
        // cover 100% は要素下端がビューポート上端に到達した瞬間:
        // rect_top == -rect_height。
        assert_eq!(
            compute_progress_for_range(-100.0, 100.0, 800.0, ProgressRange::Cover),
            1.0
        );
    }

    #[test]
    fn cover_halfway_through_full_traverse() {
        // 対称性: rect_top == (viewport_height - rect_height) / 2 で 0.5。
        assert_eq!(
            compute_progress_for_range(350.0, 100.0, 800.0, ProgressRange::Cover),
            0.5
        );
    }

    #[test]
    fn cover_clamps_past_full_traverse_to_one() {
        assert_eq!(
            compute_progress_for_range(-500.0, 100.0, 800.0, ProgressRange::Cover),
            1.0
        );
    }

    #[test]
    fn cover_zero_size_both_dimensions_is_one_and_does_not_panic() {
        assert_eq!(
            compute_progress_for_range(0.0, 0.0, 0.0, ProgressRange::Cover),
            1.0
        );
    }

    #[test]
    fn contain_starts_at_zero_when_fully_entered() {
        // contain 0% は entry 100% と同一。
        assert_eq!(
            compute_progress_for_range(700.0, 100.0, 800.0, ProgressRange::Contain),
            0.0
        );
    }

    #[test]
    fn contain_ends_at_one_when_top_edge_reaches_viewport_top() {
        assert_eq!(
            compute_progress_for_range(0.0, 100.0, 800.0, ProgressRange::Contain),
            1.0
        );
    }

    #[test]
    fn contain_halfway_through_containment() {
        assert_eq!(
            compute_progress_for_range(350.0, 100.0, 800.0, ProgressRange::Contain),
            0.5
        );
    }

    #[test]
    fn contain_falls_back_to_cover_when_element_taller_than_viewport() {
        // rect_height >= viewport_height: contain 区間が退化するため
        // Cover と同じ値になる（doc の既知の単純化）。
        let rect_top = 100.0;
        let rect_height = 900.0;
        let viewport_height = 800.0;
        assert_eq!(
            compute_progress_for_range(
                rect_top,
                rect_height,
                viewport_height,
                ProgressRange::Contain
            ),
            compute_progress_for_range(
                rect_top,
                rect_height,
                viewport_height,
                ProgressRange::Cover
            )
        );
    }

    #[test]
    fn contain_equal_heights_falls_back_to_cover_without_panicking() {
        // rect_height == viewport_height（denom == 0）も退化ケースとして
        // panic せず Cover へフォールバックする。
        let value = compute_progress_for_range(100.0, 800.0, 800.0, ProgressRange::Contain);
        assert!((0.0..=1.0).contains(&value));
    }

    #[test]
    fn progress_range_default_is_entry() {
        assert_eq!(ProgressRange::default(), ProgressRange::Entry);
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
