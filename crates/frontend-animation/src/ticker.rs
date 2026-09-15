//! marquee の JS 駆動拡張（ticker、イシュー #2540）: 実測に基づく複製数
//! 決定・rAF による offset 前進・hover/scroll 速度連動の計算・DOM 適用。
//!
//! # 責務境界
//!
//! `fandhe-frontend-pre-styled-ui::marquee_motion` が供給する opt-in 属性
//! （`data-fandhe-ticker*`）から実際に DOM を駆動する層。属性の探索・
//! イベント委譲登録・`prefers-reduced-motion` 判定は
//! `fandhe-frontend-wasm-full::ticker` の責務であり、本モジュールは
//! 「要素 1 個分の駆動状態（[`Ticker`]）」と、その内部で使う純粋な計算
//! （speed/offset/複製数、native テストで検証可能）のみを持つ
//! （`docs/design/motion-reference-adoption-policy.md` §6 の 3 層構成）。
//!
//! # ponytail 割り切り
//!
//! - 複製数は `2..=16` にクランプする（極端に短いコンテンツでの要素数
//!   爆発を防ぐ、A04 対策）。上限を超える継ぎ目は許容する既知の制約。
//! - 速度は `0..=2000` px/s にクランプする。
//! - scroll 速度は毎フレーム 0.9 倍の素朴な指数減衰のみ（物理的なばね等は
//!   実装しない）。
//! - 動的に追加された ticker 要素は wire 時の `querySelectorAll` の対象外
//!   （`svg_path`/`in_view` と同じ制約）。

/// [`Ticker`] が DOM の CSS カスタムプロパティへ毎フレーム書き込む変数名。
/// `fandhe-frontend-pre-styled-ui::marquee_motion::TICKER_OFFSET_VAR` /
/// `fandhe-frontend-wasm-full::ticker::TICKER_OFFSET_VAR` と同値のリテラル
/// （drift テストで固定、`crates/pre-styled-ui/tests/marquee_motion_attr_drift.rs`）。
pub const TICKER_OFFSET_VAR: &str = "--fandhe-marquee-ticker-offset";

/// 速度の既定値（px/s）・上限。
pub const DEFAULT_SPEED_PX_S: f64 = 80.0;
const MAX_SPEED_PX_S: f64 = 2000.0;

/// 複製数のクランプ範囲。
const MIN_COPIES: usize = 2;
const MAX_COPIES: usize = 16;

/// dt（フレーム間隔）の上限（ミリ秒）。タブが非アクティブ化から復帰した
/// 直後等の巨大な dt で offset が飛ばないようにする。
const MAX_DT_MS: f64 = 100.0;

/// scroll 速度の毎フレーム減衰係数（素朴な指数減衰、モジュール doc
/// 「ponytail 割り切り」節参照）。
const SCROLL_DECAY_PER_FRAME: f64 = 0.9;

/// スクロール軸（`data-axis` 属性の値と対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// [`Ticker`] 1 個分の設定。
#[derive(Debug, Clone, Copy)]
pub struct TickerConfig {
    /// 基準速度（px/s、正の有限数）。
    pub speed_px_s: f64,
    /// スクロール方向の符号（`1.0` = 通常、`-1.0` = 逆方向。
    /// `marquee::MarqueeDirection` の `Start`/`End` に対応）。
    pub direction_sign: f64,
    pub axis: Axis,
    /// hover 中の速度係数（`0.0` = 停止・`1.0` = 変化なし、既定 `0.0`）。
    pub hover_factor: f64,
    /// scroll 速度連動の係数（`0.0` = 連動なし、既定 `0.0`）。
    pub scroll_factor: f64,
}

/// `data-fandhe-ticker-speed` 等の属性文字列から数値を読む。非有限・負・
/// パース失敗は `default` へ fail-safe し、`max` 超過は `max` へ clamp する
/// （security.md A05、利用者由来の文字列を検証なしに使わない）。
#[must_use]
pub fn parse_speed(raw: &str) -> f64 {
    parse_clamped(raw, DEFAULT_SPEED_PX_S, 0.0, MAX_SPEED_PX_S)
}

/// `data-fandhe-ticker-hover-factor`/`-scroll-factor` 等、`0.0..=max` の
/// 係数属性をパースする。非有限・負・パース失敗は `default` へ fail-safe
/// する。
#[must_use]
pub fn parse_factor(raw: &str, default: f64, max: f64) -> f64 {
    parse_clamped(raw, default, 0.0, max)
}

fn parse_clamped(raw: &str, default: f64, min: f64, max: f64) -> f64 {
    match raw.trim().parse::<f64>() {
        Ok(value) if value.is_finite() && value >= min => value.min(max),
        _ => default,
    }
}

/// hover・scroll 連動を反映した実効速度（px/s、常に 0 以上の有限値）を
/// 計算する。
///
/// - `hovered` なら `base * hover_factor` を基準にする（既定
///   `hover_factor = 0.0` は完全停止、既存 CSS 版の hover 一時停止契約と
///   一致させる）。
/// - `scroll_velocity`（px/s、符号付き）の絶対値に `scroll_factor` を
///   掛けた分を加算する。
#[must_use]
pub fn effective_speed(
    base_speed: f64,
    hovered: bool,
    hover_factor: f64,
    scroll_velocity: f64,
    scroll_factor: f64,
) -> f64 {
    if !base_speed.is_finite() || base_speed < 0.0 {
        return 0.0;
    }
    let base = if hovered {
        base_speed * hover_factor.clamp(0.0, 1.0)
    } else {
        base_speed
    };
    let scroll_boost = if scroll_velocity.is_finite() && scroll_factor.is_finite() {
        scroll_velocity.abs() * scroll_factor.max(0.0)
    } else {
        0.0
    };
    (base + scroll_boost).clamp(0.0, MAX_SPEED_PX_S)
}

/// `offset`（px、`(-cycle_len, 0]` に正規化）を `speed * direction_sign` で
/// `dt_ms` 分だけ前進させる。`cycle_len <= 0` や非有限入力は変化なし
/// （`offset` をそのまま返す）で fail-safe する。
#[must_use]
pub fn advance_offset(
    offset: f64,
    speed: f64,
    direction_sign: f64,
    dt_ms: f64,
    cycle_len: f64,
) -> f64 {
    if !offset.is_finite() || !speed.is_finite() || !cycle_len.is_finite() || cycle_len <= 0.0 {
        return 0.0;
    }
    let dt_ms = dt_ms.clamp(0.0, MAX_DT_MS);
    let direction_sign = if direction_sign < 0.0 { -1.0 } else { 1.0 };
    let delta = speed * direction_sign * (dt_ms / 1000.0);
    // シームレスループの折り返しは常に `[-cycle_len, 0]`（負方向へ進む
    // translate 値）へ正規化する: `direction_sign` が符号を反転させても
    // 折り返し先の範囲自体は変えず、複製列の先頭が root の左（上）端に
    // 常に揃う既存 CSS 版の `@keyframes` と同じ見た目を維持する。
    let next = offset - delta;
    (next.rem_euclid(cycle_len)) - cycle_len
}

/// ビューポート長・コンテンツ 1 個分の長さから、継ぎ目なく循環させるため
/// 必要な複製数（元のコンテンツ含む）を計算する。`content_len <= 0` は
/// [`MIN_COPIES`] を返す。
#[must_use]
pub fn required_copies(viewport_len: f64, content_len: f64) -> usize {
    if !content_len.is_finite() || content_len <= 0.0 || !viewport_len.is_finite() {
        return MIN_COPIES;
    }
    let needed = (viewport_len / content_len).ceil() as i64 + 1;
    needed.clamp(MIN_COPIES as i64, MAX_COPIES as i64) as usize
}

/// scroll 速度（px/s）を毎フレーム指数減衰させる（モジュール doc
/// 「ponytail 割り切り」節）。
#[must_use]
pub fn decay_velocity(velocity: f64) -> f64 {
    if !velocity.is_finite() {
        return 0.0;
    }
    let next = velocity * SCROLL_DECAY_PER_FRAME;
    if next.abs() < 0.01 {
        0.0
    } else {
        next
    }
}

#[cfg(target_arch = "wasm32")]
mod dom {
    use super::{Axis, TickerConfig, TICKER_OFFSET_VAR};
    use crate::raf_driver::AnimationLoop;
    use std::cell::Cell;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, Node};

    /// `element` の内容領域の長さ（`Axis::Horizontal` なら幅・`Vertical`
    /// なら高さ）を `getBoundingClientRect()` から読む。
    #[must_use]
    pub fn measure_len(element: &Element, axis: Axis) -> f64 {
        let rect = element.get_bounding_client_rect();
        match axis {
            Axis::Horizontal => rect.width(),
            Axis::Vertical => rect.height(),
        }
    }

    /// `element` の `gap`（px）を `getComputedStyle` から読む。取得・
    /// パース失敗は `0.0` へ fail-safe する。
    #[must_use]
    pub fn read_gap_px(element: &HtmlElement) -> f64 {
        let Some(window) = web_sys::window() else {
            return 0.0;
        };
        let Ok(Some(style)) = window.get_computed_style(element) else {
            return 0.0;
        };
        let Ok(gap) = style.get_property_value("gap") else {
            return 0.0;
        };
        gap.trim()
            .strip_suffix("px")
            .and_then(|n| n.parse::<f64>().ok())
            .filter(|n| n.is_finite())
            .unwrap_or(0.0)
    }

    /// `content`（既存 SSR 複製 1 個を含む）を `needed` 個になるまで
    /// `cloneNode(true)` で追加する（既存 2 個を超えて増やす場合のみ）。
    /// 追加した複製には `aria-hidden="true"`・`inert` を付与し、
    /// スクリーンリーダー二重読み上げ・タブ順序混入を防ぐ
    /// （`marquee` 既存複製と同じ契約）。`root` から現在の子要素数を数え、
    /// 冪等（再呼び出しでは増えない）。
    pub fn ensure_copies(root: &Element, content_template: &Element, needed: usize) {
        let needed = needed.clamp(super::MIN_COPIES, super::MAX_COPIES);
        let mut current = root.child_element_count() as usize;
        while current < needed {
            let Ok(clone) = content_template.clone_node_with_deep(true) else {
                break;
            };
            let Ok(clone_element) = clone.dyn_into::<Element>() else {
                break;
            };
            let _ = clone_element.set_attribute("aria-hidden", "true");
            let _ = clone_element.set_attribute("inert", "");
            if root.append_child(&clone_element as &Node).is_err() {
                break;
            }
            current += 1;
        }
    }

    /// `element` の style へ `offset`（px）を [`TICKER_OFFSET_VAR`] として
    /// 書き込む。`CSSStyleDeclaration.setProperty` の 2 引数 API のみを
    /// 使う（A03 対策、固定プロパティ名・`f64` 演算結果のみ）。
    pub fn write_offset(element: &HtmlElement, offset_px: f64) {
        let _ = element
            .style()
            .set_property(TICKER_OFFSET_VAR, &format!("{offset_px}px"));
    }

    /// 1 ticker 要素分の駆動状態。`root`/`content` を握り、`AnimationLoop`
    /// が毎フレーム offset を前進・DOM へ書き込む。`Drop` でループを停止
    /// する。
    pub struct Ticker {
        _loop: AnimationLoop,
        hovered: Rc<Cell<bool>>,
        scroll_velocity: Rc<Cell<f64>>,
        resize_pending: Rc<Cell<bool>>,
    }

    impl Ticker {
        /// `root`（`data-fandhe-ticker` 要素）・`content`（最初の
        /// `[data-part="content"]`）を駆動開始する。開始時点で
        /// [`ensure_copies`] を 1 回実行して必要な複製数を満たす。
        #[must_use]
        pub fn start(root: Element, content: Element, config: TickerConfig) -> Self {
            let Some(content_html) = content.dyn_ref::<HtmlElement>().cloned() else {
                // `HtmlElement` へダウンキャストできない場合は駆動しない
                // no-op（`RafDriver::new` と同じ fail-safe 方針）。
                return Self {
                    _loop: AnimationLoop::start(|| false),
                    hovered: Rc::new(Cell::new(false)),
                    scroll_velocity: Rc::new(Cell::new(0.0)),
                    resize_pending: Rc::new(Cell::new(false)),
                };
            };

            let viewport_len = measure_len(&root, config.axis);
            let content_len = measure_len(&content, config.axis) + read_gap_px(&content_html);
            ensure_copies(
                &root,
                &content,
                super::required_copies(viewport_len, content_len),
            );

            let hovered = Rc::new(Cell::new(false));
            let scroll_velocity = Rc::new(Cell::new(0.0));
            let resize_pending = Rc::new(Cell::new(false));
            let offset = Rc::new(Cell::new(0.0_f64));
            let last_ms: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));

            let step_root = root.clone();
            let step_content = content;
            let step_content_html = content_html;
            let step_hovered = Rc::clone(&hovered);
            let step_scroll_velocity = Rc::clone(&scroll_velocity);
            let step_resize_pending = Rc::clone(&resize_pending);
            let step_offset = Rc::clone(&offset);
            let step_last_ms = Rc::clone(&last_ms);

            let animation_loop = AnimationLoop::start(move || {
                let now_ms = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now());
                let Some(now_ms) = now_ms else {
                    return true;
                };
                let dt_ms = step_last_ms.get().map(|last| now_ms - last).unwrap_or(0.0);
                step_last_ms.set(Some(now_ms));

                if step_resize_pending.take() {
                    let viewport_len = measure_len(&step_root, config.axis);
                    let content_len =
                        measure_len(&step_content, config.axis) + read_gap_px(&step_content_html);
                    ensure_copies(
                        &step_root,
                        &step_content,
                        super::required_copies(viewport_len, content_len),
                    );
                }

                let scroll_velocity_now = super::decay_velocity(step_scroll_velocity.get());
                step_scroll_velocity.set(scroll_velocity_now);

                let content_len =
                    measure_len(&step_content, config.axis) + read_gap_px(&step_content_html);
                let speed = super::effective_speed(
                    config.speed_px_s,
                    step_hovered.get(),
                    config.hover_factor,
                    scroll_velocity_now,
                    config.scroll_factor,
                );
                let next_offset = super::advance_offset(
                    step_offset.get(),
                    speed,
                    config.direction_sign,
                    dt_ms,
                    content_len.max(1.0),
                );
                step_offset.set(next_offset);
                write_offset(&step_content_html, next_offset);
                true
            });

            Self {
                _loop: animation_loop,
                hovered,
                scroll_velocity,
                resize_pending,
            }
        }

        pub fn set_hovered(&self, hovered: bool) {
            self.hovered.set(hovered);
        }

        /// `scroll` イベントのデルタ（px）と経過時間（ms）から瞬間速度を
        /// 計算し、内部の scroll 速度へ加算する。
        pub fn push_scroll_delta(&self, delta_px: f64, dt_ms: f64) {
            if !delta_px.is_finite() || dt_ms <= 0.0 {
                return;
            }
            let instantaneous = delta_px / (dt_ms / 1000.0);
            if instantaneous.is_finite() {
                self.scroll_velocity
                    .set(self.scroll_velocity.get() + instantaneous);
            }
        }

        /// 次フレームでの再計測・複製数再調整を要求する。
        pub fn mark_resize(&self) {
            self.resize_pending.set(true);
        }

        pub fn stop(&self) {
            self._loop.stop();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{ensure_copies, measure_len, read_gap_px, write_offset, Ticker};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_speed_uses_default_for_invalid_input() {
        assert_eq!(parse_speed("abc"), DEFAULT_SPEED_PX_S);
        assert_eq!(parse_speed("-5"), DEFAULT_SPEED_PX_S);
        assert_eq!(parse_speed(""), DEFAULT_SPEED_PX_S);
    }

    #[test]
    fn parse_speed_clamps_to_max() {
        assert_eq!(parse_speed("1e9"), MAX_SPEED_PX_S);
    }

    #[test]
    fn parse_speed_accepts_valid_value() {
        assert_eq!(parse_speed("120"), 120.0);
    }

    #[test]
    fn parse_factor_uses_default_for_invalid_input() {
        assert_eq!(parse_factor("nope", 0.0, 1.0), 0.0);
        assert_eq!(parse_factor("-1", 0.5, 1.0), 0.5);
    }

    #[test]
    fn parse_factor_clamps_to_max() {
        assert_eq!(parse_factor("5", 0.0, 1.0), 1.0);
    }

    #[test]
    fn effective_speed_stops_on_hover_with_default_factor() {
        assert_eq!(effective_speed(80.0, true, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn effective_speed_halves_on_hover_with_half_factor() {
        assert_eq!(effective_speed(80.0, true, 0.5, 0.0, 0.0), 40.0);
    }

    #[test]
    fn effective_speed_adds_scroll_boost() {
        assert_eq!(effective_speed(80.0, false, 0.0, 100.0, 0.2), 100.0);
    }

    #[test]
    fn effective_speed_is_zero_for_invalid_base() {
        assert_eq!(effective_speed(f64::NAN, false, 0.0, 0.0, 0.0), 0.0);
        assert_eq!(effective_speed(-1.0, false, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn advance_offset_moves_negative_and_wraps() {
        // speed=100px/s, dt=50ms（MAX_DT_MS(100ms) 未満）→ 5px 前進。cycle_len=200。
        let offset = advance_offset(0.0, 100.0, 1.0, 50.0, 200.0);
        assert!((offset - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn advance_offset_wraps_within_cycle() {
        // 近い offset からさらに進めて -cycle_len を跨いでも [-cycle, 0] に収まる。
        let offset = advance_offset(-190.0, 100.0, 1.0, 500.0, 200.0);
        assert!((-200.0..=0.0).contains(&offset));
    }

    #[test]
    fn advance_offset_reverses_direction_sign() {
        let forward = advance_offset(-50.0, 100.0, 1.0, 100.0, 200.0);
        let backward = advance_offset(-50.0, 100.0, -1.0, 100.0, 200.0);
        assert!(forward < -50.0);
        assert!(backward > -50.0);
    }

    #[test]
    fn advance_offset_is_zero_for_non_positive_cycle_len() {
        assert_eq!(advance_offset(0.0, 100.0, 1.0, 100.0, 0.0), 0.0);
        assert_eq!(advance_offset(0.0, 100.0, 1.0, 100.0, -1.0), 0.0);
    }

    #[test]
    fn advance_offset_clamps_huge_dt() {
        // dt が MAX_DT_MS(100ms) にクランプされるため、1 フレームで進む距離は
        // speed * 100ms/1000 を超えない。
        let offset = advance_offset(0.0, 100.0, 1.0, 100_000.0, 1_000_000.0);
        assert!((offset - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn required_copies_covers_viewport_with_margin() {
        // viewport=500, content=120 → ceil(500/120)+1 = 6
        assert_eq!(required_copies(500.0, 120.0), 6);
    }

    #[test]
    fn required_copies_clamps_to_min() {
        assert_eq!(required_copies(10.0, 1000.0), MIN_COPIES);
    }

    #[test]
    fn required_copies_clamps_to_max() {
        assert_eq!(required_copies(100_000.0, 1.0), MAX_COPIES);
    }

    #[test]
    fn required_copies_is_min_for_non_positive_content_len() {
        assert_eq!(required_copies(500.0, 0.0), MIN_COPIES);
        assert_eq!(required_copies(500.0, -10.0), MIN_COPIES);
    }

    #[test]
    fn decay_velocity_shrinks_and_settles_to_zero() {
        let v = decay_velocity(100.0);
        assert!((v - 90.0).abs() < 1e-9);
        let mut velocity = 0.02;
        for _ in 0..50 {
            velocity = decay_velocity(velocity);
        }
        assert_eq!(velocity, 0.0);
    }

    #[test]
    fn decay_velocity_is_zero_for_non_finite() {
        assert_eq!(decay_velocity(f64::NAN), 0.0);
        assert_eq!(decay_velocity(f64::INFINITY), 0.0);
    }

    #[test]
    fn offset_var_is_stable_literal() {
        assert_eq!(TICKER_OFFSET_VAR, "--fandhe-marquee-ticker-offset");
    }
}
