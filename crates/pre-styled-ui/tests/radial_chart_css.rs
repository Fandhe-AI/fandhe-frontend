//! styled RadialChart（イシュー #2079）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/pie_donut_chart_css.rs` の golden fixture
//! テストの前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。
//! `size` variant ごとの規則の出力順が崩れた場合や意図しない宣言の追加・
//! 欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::radial_chart;

const RADIAL_CHART_GOLDEN_CSS: &str = r#"[data-scope="radial-chart"][data-part="root"] {
  display: inline-flex;
  --fandhe-radial-chart-size: 16rem;
  position: relative;
}

[data-scope="radial-chart"][data-part="chart"] {
  width: var(--fandhe-radial-chart-size);
  height: var(--fandhe-radial-chart-size);
}

[data-scope="radial-chart"][data-part="track"] {
  fill: var(--fandhe-color-bg-muted);
}

[data-scope="radial-chart"][data-part="bar"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="radial-chart"][data-part="label"] {
  fill: var(--fandhe-color-fg);
  font-size: 4px;
  text-anchor: start;
  dominant-baseline: central;
  paint-order: stroke;
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="radial-chart"][data-part="grid-circle"] {
  fill: none;
  stroke: var(--fandhe-color-border);
  stroke-width: 0.5;
}

[data-scope="radial-chart"][data-part="grid-spoke"] {
  stroke: var(--fandhe-color-border);
  stroke-width: 0.5;
}

[data-scope="radial-chart"][data-part="center-value"] {
  fill: var(--fandhe-color-fg);
  font-size: 12px;
  font-weight: var(--fandhe-font-font-weight-bold);
  text-anchor: middle;
  dominant-baseline: central;
}

[data-scope="radial-chart"][data-part="center-label"] {
  fill: var(--fandhe-color-fg-muted);
  font-size: 4px;
  text-anchor: middle;
  dominant-baseline: central;
}

[data-scope="radial-chart"][data-part="root"].fd-radial-chart--size-xs {
  --fandhe-radial-chart-size: 4rem;
}

[data-scope="radial-chart"][data-part="root"].fd-radial-chart--size-sm {
  --fandhe-radial-chart-size: 10rem;
}

[data-scope="radial-chart"][data-part="root"].fd-radial-chart--size-md {
  --fandhe-radial-chart-size: 16rem;
}

[data-scope="radial-chart"][data-part="root"].fd-radial-chart--size-lg {
  --fandhe-radial-chart-size: 22rem;
}

[data-scope="radial-chart"][data-part="root"].fd-radial-chart--size-xl {
  --fandhe-radial-chart-size: 28rem;
}

[data-scope="radial-chart"][data-part="bar"][data-hidden] {
  display: none;
}

[data-scope="radial-chart"][data-part="label"][data-hidden] {
  display: none;
}

[data-scope="radial-chart"][data-part="track"][data-hidden] {
  display: none;
}

[data-scope="radial-chart"][data-part="root"][data-has-active] {
  --fandhe-chart-inactive-opacity: 0.4;
  --fandhe-chart-active-scale: 1.05;
}

[data-scope="radial-chart"][data-part="bar"][data-index] {
  opacity: var(--fandhe-chart-inactive-opacity, 1);
  transition-property: opacity, transform;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="radial-chart"][data-part="bar"][data-active] {
  opacity: 1;
  transform-box: view-box;
  transform-origin: 50% 50%;
  transform: scale(var(--fandhe-chart-active-scale, 1));
}
"#;

#[test]
fn radial_chart_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(radial_chart::css(), RADIAL_CHART_GOLDEN_CSS);
}

#[test]
fn radial_chart_css_is_deterministic() {
    assert_eq!(radial_chart::css(), radial_chart::css());
}

#[test]
fn radial_chart_css_does_not_contain_style_breakout_sequences() {
    let css = radial_chart::css();
    assert!(!css.contains('<'));
    assert!(!css.contains("</style"));
}

#[test]
fn radial_chart_css_does_not_declare_a_color_palette_axis() {
    // pie_chart/donut_chart と同じ判断で系列色はトークン循環のみとし、
    // `color-palette` variant 軸は提供しない（モジュール doc 参照）。
    assert!(!radial_chart::css().contains("color-palette"));
}
