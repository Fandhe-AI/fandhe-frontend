//! marquee の Ticker 相当拡張（opt-in、イシュー #2540、親 #2528）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9（参照は可・転写は
//! 禁止）に従い、Motion+ の `examples/ticker` から着想した機能軸（縦方向・
//! 速度/方向の JS 駆動・hover 一時停止・scroll 速度連動）のみを Rust/CSS で
//! 独自に再実装する opt-in 拡張。
//!
//! # `marquee.rs` 本体を変更しない理由
//!
//! [`crate::motion`]/[`crate::button_motion`] と同じ契約: 既存
//! [`crate::marquee`] の recipe・`css()`・golden テスト（`tests/marquee_css.rs`）
//! は一切変更しない。[`ticker`] は [`crate::marquee::marquee`] へ opt-in 属性
//! を前置して委譲する薄いラッパであり、[`Theme::to_css_with_marquee_motion`]
//! は [`Theme::to_css`] の出力へ [`MARQUEE_MOTION_CSS`] を追記するだけの
//! pure append（`Theme::to_css` 本体・走査ループは不変）。
//!
//! # 3 層構成（責務境界）
//!
//! 純計算（offset 前進・実効速度・必要複製数）と DOM 計測・rAF ループは
//! `fandhe-frontend-animation::ticker` の責務、opt-in 要素の走査・イベント
//! 委譲・`prefers-reduced-motion` 判定は `fandhe-frontend-wasm-full::ticker`
//! の責務であり、本モジュールはマークアップ・CSS のみを供給する
//! （`docs/design/motion-reference-adoption-policy.md` §6）。
//!
//! # `data-*` 属性・CSS カスタムプロパティの命名（他クレートとの契約）
//!
//! [`TICKER_ATTR`]/[`TICKER_SPEED_ATTR`]/[`TICKER_HOVER_FACTOR_ATTR`]/
//! [`TICKER_SCROLL_FACTOR_ATTR`]/[`TICKER_AXIS_ATTR`]/[`TICKER_ACTIVE_ATTR`]
//! のリテラル値は `fandhe-frontend-wasm-full::ticker` の同名定数と一致する
//! ことが前提（本クレートは `wasm-full` に依存しないため型共有はできず、
//! リテラルの写しとして保持する）。ドリフトは
//! `crates/pre-styled-ui/tests/marquee_motion_attr_drift.rs`
//! （`crates/button_motion_attr_drift.rs` と同型）が wasm-full 側ソースを
//! 読んで fail-closed に検知する。[`TICKER_OFFSET_VAR`] は
//! `fandhe-frontend-animation::ticker::TICKER_OFFSET_VAR` が唯一の正であり、
//! 本クレート・wasm-full の同名定数はそのリテラルの写しである。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! JS 駆動が有効化されるのは wasm-full が [`TICKER_ACTIVE_ATTR`] を付与した
//! 場合のみで、`prefers-reduced-motion: reduce` 環境では wasm-full 側が
//! wire 自体を抑制するため付与されない。本モジュールの
//! [`MARQUEE_MOTION_CSS`] は `[data-fandhe-ticker-active]` 配下でのみ
//! `animation: none` + `transform` 駆動へ切り替えるため、抑制時は既存
//! `marquee.rs` の `@media (prefers-reduced-motion: reduce)` 縮退がそのまま
//! 効く。縦方向の `@keyframes` は `marquee.rs` の一括ゼロ化を通らないため、
//! 本モジュールが個別に `@media` ブロックを追記する。

use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

use crate::marquee::{marquee, MarqueeProps};

/// opt-in（著者が SSR 出力に静的に付与）: JS 駆動（ticker）を有効化する
/// マーカー（値なし存在属性）。`fandhe_frontend_wasm_full::ticker::
/// TICKER_ATTR` と値が一致する必要がある（モジュール冒頭「他クレートとの
/// 契約」節）。
pub const TICKER_ATTR: &str = "data-fandhe-ticker";
/// opt-in（任意）: 基準速度（px/s）を著者が上書きする属性。
pub const TICKER_SPEED_ATTR: &str = "data-fandhe-ticker-speed";
/// opt-in（任意）: hover 中の速度係数（`0.0`〜`1.0`）を著者が上書きする
/// 属性。既定 `0.0`（完全停止、既存 CSS 版の hover 一時停止契約と一致）。
pub const TICKER_HOVER_FACTOR_ATTR: &str = "data-fandhe-ticker-hover-factor";
/// opt-in（任意）: scroll 速度連動の係数を著者が上書きする属性。既定
/// `0.0`（連動なし）。
pub const TICKER_SCROLL_FACTOR_ATTR: &str = "data-fandhe-ticker-scroll-factor";
/// スクロール軸（`horizontal`/`vertical`）を指定する属性。CSS 側の
/// セレクタ・JS 側の計測軸の両方が読む共有属性。
pub const TICKER_AXIS_ATTR: &str = "data-axis";
/// wasm-full が JS 駆動開始時に root へ付与するマーカー（[`MARQUEE_MOTION_CSS`]
/// が `animation: none` + `transform` 駆動へ切り替える対象）。
pub const TICKER_ACTIVE_ATTR: &str = "data-fandhe-ticker-active";
/// JS が毎フレーム書き込む offset の CSS カスタムプロパティ名。
/// `fandhe_frontend_animation::ticker::TICKER_OFFSET_VAR` が唯一の正。
pub const TICKER_OFFSET_VAR: &str = "--fandhe-marquee-ticker-offset";

/// Marquee のスクロール軸。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TickerAxis {
    /// 水平スクロール（既定）。
    #[default]
    Horizontal,
    /// 垂直スクロール。
    Vertical,
}

impl TickerAxis {
    fn attr_value(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// [`ticker`] の設定。数値系は `Option`（`None` なら対応属性を出力せず、
/// JS 側の既定値——`fandhe_frontend_animation::ticker` の定数——へ委ねる）。
#[derive(Debug, Clone, Copy, Default)]
pub struct TickerProps<'a> {
    /// 委譲先 [`crate::marquee::marquee`] の設定（`direction`/`decorative`/
    /// `label`）。
    pub marquee: MarqueeProps<'a>,
    /// スクロール軸（既定 `Horizontal`）。
    pub axis: TickerAxis,
    /// 基準速度（px/s）。`None` なら [`TICKER_SPEED_ATTR`] を出力しない。
    pub speed_px_s: Option<u32>,
    /// hover 中の速度係数（`0.0..=1.0`）。`None` なら
    /// [`TICKER_HOVER_FACTOR_ATTR`] を出力しない。
    pub hover_factor: Option<f32>,
    /// scroll 速度連動の係数。`None` なら [`TICKER_SCROLL_FACTOR_ATTR`] を
    /// 出力しない。
    pub scroll_factor: Option<f32>,
}

/// [`crate::marquee::marquee`] へ ticker の opt-in 属性を前置して委譲する。
/// 既存の 2 重 content・`aria-hidden`/`inert`・`decorative`/`label` 契約は
/// 完全に維持する（[`crate::marquee::marquee`] rustdoc 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::marquee::item;
/// use fandhe_frontend_pre_styled_ui::marquee_motion::{ticker, TickerProps};
///
/// let node = ticker(
///     &TickerProps {
///         speed_px_s: Some(60),
///         ..TickerProps::default()
///     },
///     vec![],
///     vec![item(vec![], vec![text("Breaking news")])],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"data-fandhe-ticker="""#));
/// assert!(html.contains(r#"data-fandhe-ticker-speed="60""#));
/// ```
#[must_use]
pub fn ticker<'a>(
    props: &TickerProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    // 型注釈を `'a` 固定にしない（速度等の `String` は本関数内で生成する
    // 局所値のため `'a` を生きられない）: `merged` の要素型は無注釈のまま
    // 推論に委ね、`marquee()` 呼び出し時にコンパイラが `attrs`〔共変〕を
    // 含めて短い借用期間へ独立に単一化する。
    let mut merged = vec![
        (TICKER_ATTR, ""),
        (TICKER_AXIS_ATTR, props.axis.attr_value()),
    ];
    let speed_string = props.speed_px_s.map(|v| v.to_string());
    if let Some(ref s) = speed_string {
        merged.push((TICKER_SPEED_ATTR, s.as_str()));
    }
    let hover_string = props.hover_factor.map(|v| v.to_string());
    if let Some(ref s) = hover_string {
        merged.push((TICKER_HOVER_FACTOR_ATTR, s.as_str()));
    }
    let scroll_string = props.scroll_factor.map(|v| v.to_string());
    if let Some(ref s) = scroll_string {
        merged.push((TICKER_SCROLL_FACTOR_ATTR, s.as_str()));
    }
    merged.extend(attrs);
    marquee(&props.marquee, merged, children)
}

/// `marquee` の縦方向・ticker 拡張 CSS（opt-in、`motion` feature 配下）。
///
/// 1. `[data-axis="vertical"]`: root/content を `flex-direction: column`
///    へ切り替え、独自 `@keyframes fd-marquee-scroll-y`（`translateY`）を
///    使う。両端フェードも `to bottom` 方向へ切り替える。
/// 2. `[data-fandhe-ticker-active]` 配下の content: `animation: none` +
///    `transform: translate(var(--fandhe-marquee-ticker-offset, 0px))`
///    （JS が毎フレーム書き込む値で駆動する。縦方向は `translateY`）。
/// 3. `@media (prefers-reduced-motion: reduce)`: 縦方向 content の
///    `@keyframes` を停止する（`marquee.rs` の水平版縮退は既存のまま、
///    モジュール doc「reduced-motion」節参照）。
pub const MARQUEE_MOTION_CSS: &str = concat!(
    "\n[data-scope=\"marquee\"][data-part=\"root\"][data-axis=\"vertical\"] {\n",
    "  flex-direction: column;\n",
    "  mask-image: linear-gradient(to bottom, transparent, black var(--fandhe-marquee-fade, 0px), black calc(100% - var(--fandhe-marquee-fade, 0px)), transparent);\n",
    "}\n",
    "[data-scope=\"marquee\"][data-part=\"content\"][data-axis=\"vertical\"] {\n",
    "  flex-direction: column;\n",
    "  min-width: auto;\n",
    "  min-height: max-content;\n",
    "  animation-name: fd-marquee-scroll-y;\n",
    "}\n",
    "@keyframes fd-marquee-scroll-y {\n",
    "  from {\n",
    "    transform: translateY(0);\n",
    "  }\n",
    "  to {\n",
    "    transform: translateY(calc(-100% - var(--fandhe-marquee-gap, var(--fandhe-space-4))));\n",
    "  }\n",
    "}\n",
    "\n[data-scope=\"marquee\"][data-part=\"root\"][data-fandhe-ticker-active] [data-part=\"content\"] {\n",
    "  animation: none;\n",
    "  transform: translateX(var(--fandhe-marquee-ticker-offset, 0px));\n",
    "}\n",
    "[data-scope=\"marquee\"][data-part=\"root\"][data-fandhe-ticker-active][data-axis=\"vertical\"] [data-part=\"content\"] {\n",
    "  transform: translateY(var(--fandhe-marquee-ticker-offset, 0px));\n",
    "}\n",
    "\n@media (prefers-reduced-motion: reduce) {\n",
    "  [data-scope=\"marquee\"][data-part=\"content\"][data-axis=\"vertical\"] {\n",
    "    animation: none;\n",
    "    min-height: 0;\n",
    "  }\n",
    "}\n",
);

/// opt-in API。[`crate::theme::Theme::to_css`] の出力へ
/// [`MARQUEE_MOTION_CSS`] を追記して返す（pure append、
/// [`crate::button_motion::Theme::to_css_with_button_motion`] と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に marquee ticker 拡張 CSS
    /// （[`MARQUEE_MOTION_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_marquee_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(MARQUEE_MOTION_CSS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marquee::item;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn ticker_renders_opt_in_marker_and_axis() {
        let node = ticker(&TickerProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-fandhe-ticker="""#));
        assert!(html.contains(r#"data-axis="horizontal""#));
    }

    #[test]
    fn ticker_renders_vertical_axis() {
        let node = ticker(
            &TickerProps {
                axis: TickerAxis::Vertical,
                ..TickerProps::default()
            },
            vec![],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-axis="vertical""#));
    }

    #[test]
    fn ticker_renders_optional_numeric_attrs_when_present() {
        let node = ticker(
            &TickerProps {
                speed_px_s: Some(120),
                hover_factor: Some(0.5),
                scroll_factor: Some(0.2),
                ..TickerProps::default()
            },
            vec![],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-fandhe-ticker-speed="120""#));
        assert!(html.contains(r#"data-fandhe-ticker-hover-factor="0.5""#));
        assert!(html.contains(r#"data-fandhe-ticker-scroll-factor="0.2""#));
    }

    #[test]
    fn ticker_omits_optional_numeric_attrs_when_absent() {
        let node = ticker(&TickerProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(!html.contains("data-fandhe-ticker-speed"));
        assert!(!html.contains("data-fandhe-ticker-hover-factor"));
        assert!(!html.contains("data-fandhe-ticker-scroll-factor"));
    }

    #[test]
    fn ticker_preserves_marquee_duplicated_content_contract() {
        let node = ticker(
            &TickerProps::default(),
            vec![],
            vec![item(vec![], vec![text("news")])],
        );
        let html = render(&node);
        assert_eq!(html.matches("news").count(), 2);
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains("inert"));
    }

    #[test]
    fn ticker_escapes_children_by_default() {
        let node = ticker(
            &TickerProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        );
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn to_css_with_marquee_motion_is_pure_append() {
        use crate::theme::Theme;
        let theme = Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_marquee_motion();
        assert!(extended.starts_with(&base));
        assert_eq!(&extended[base.len()..], MARQUEE_MOTION_CSS);
    }

    #[test]
    fn attr_constants_are_stable_literals() {
        assert_eq!(TICKER_ATTR, "data-fandhe-ticker");
        assert_eq!(TICKER_SPEED_ATTR, "data-fandhe-ticker-speed");
        assert_eq!(TICKER_HOVER_FACTOR_ATTR, "data-fandhe-ticker-hover-factor");
        assert_eq!(
            TICKER_SCROLL_FACTOR_ATTR,
            "data-fandhe-ticker-scroll-factor"
        );
        assert_eq!(TICKER_AXIS_ATTR, "data-axis");
        assert_eq!(TICKER_ACTIVE_ATTR, "data-fandhe-ticker-active");
        assert_eq!(TICKER_OFFSET_VAR, "--fandhe-marquee-ticker-offset");
    }
}
