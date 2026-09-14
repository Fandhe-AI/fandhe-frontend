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

use std::cell::Cell;
use std::rc::Rc;

// wasm32 以外では `update_element_progress` が `Target::write` を呼ばない
// no-op 分岐のみコンパイルされるため（`raf_driver.rs` の
// `#[cfg(target_arch = "wasm32")] use wasm_bindgen::JsCast;` と同型）、
// この import も同じ cfg でガードする（未使用 import の警告を避ける）。
#[cfg(target_arch = "wasm32")]
use fandhe_animation::target::Target;

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

/// `element` の現在位置を計測し、[`compute_progress`] の結果を `target` へ
/// 書き込む（`fandhe-frontend-wasm-full` の scroll/resize リスナー・rAF
/// ループから毎フレーム呼ばれる想定）。
///
/// `element.get_bounding_client_rect()` と `window.inner_height()` の
/// 計測に失敗した場合（`window` 不在等）は書き込みを行わず `None` を返す
/// （fail-closed、panic しない）。
///
/// wasm32 以外のターゲットでは JS 呼び出し自体を伴わない no-op として
/// `None` を返す（`RafDriver::new`/`AnimationLoop::start` と同じ native
/// no-panic 方針）。
pub fn update_element_progress(element: &web_sys::Element, target: &mut DomTarget) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let rect = element.get_bounding_client_rect();
        let viewport_height = web_sys::window()?.inner_height().ok()?.as_f64()?;
        let progress = compute_progress(rect.top(), rect.height(), viewport_height);
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
    loop_: AnimationLoop,
    dirty: Rc<Cell<bool>>,
}

impl ScrollDriver {
    /// `recompute` を dirty なフレームのみ呼ぶループを開始する。
    ///
    /// 開始直後の 1 フレーム目は無条件に `recompute` を呼ぶ（初期スクロール
    /// 位置に対応する進捗を、最初の `scroll`/`resize` イベントを待たずに
    /// 反映するため）。
    pub fn start(mut recompute: impl FnMut() + 'static) -> Self {
        let dirty = Rc::new(Cell::new(true));
        let dirty_for_loop = dirty.clone();
        let loop_ = AnimationLoop::start(move || {
            if dirty_for_loop.replace(false) {
                recompute();
            }
            true
        });
        Self { loop_, dirty }
    }

    /// 次フレームで `recompute` が呼ばれるようフラグを立てる（`scroll`/
    /// `resize` イベントリスナーから呼ばれる）。
    pub fn mark_dirty(&self) {
        self.dirty.set(true);
    }

    /// ループを明示停止する（`Drop` でも `AnimationLoop::drop` 経由で
    /// 停止するが、要素の動的除去等に伴う早期停止用に公開する）。
    pub fn stop(&self) {
        self.loop_.stop();
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
