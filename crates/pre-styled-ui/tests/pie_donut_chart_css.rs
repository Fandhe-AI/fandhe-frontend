//! styled PieChart / DonutChart（イシュー #850）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/marquee_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。`size`
//! variant ごとの規則の出力順が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::{donut_chart, pie_chart};

const PIE_CHART_GOLDEN_CSS: &str = r#"[data-scope="pie-chart"][data-part="root"] {
  display: inline-flex;
  --fandhe-pie-chart-size: 16rem;
  position: relative;
}

[data-scope="pie-chart"][data-part="chart"] {
  width: var(--fandhe-pie-chart-size);
  height: var(--fandhe-pie-chart-size);
}

[data-scope="pie-chart"][data-part="segment"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="pie-chart"][data-part="label"] {
  fill: var(--fandhe-color-fg);
  font-size: 6px;
  text-anchor: middle;
  dominant-baseline: central;
  paint-order: stroke;
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="pie-chart"][data-part="label-line"] {
  stroke: var(--fandhe-color-fg-muted);
  stroke-width: 0.5;
  fill: none;
}

[data-scope="pie-chart"][data-part="outside-label"] {
  fill: var(--fandhe-color-fg);
  font-size: 5px;
  text-anchor: start;
  dominant-baseline: central;
}

[data-scope="pie-chart"][data-part="root"].fd-pie-chart--size-xs {
  --fandhe-pie-chart-size: 4rem;
}

[data-scope="pie-chart"][data-part="root"].fd-pie-chart--size-sm {
  --fandhe-pie-chart-size: 10rem;
}

[data-scope="pie-chart"][data-part="root"].fd-pie-chart--size-md {
  --fandhe-pie-chart-size: 16rem;
}

[data-scope="pie-chart"][data-part="root"].fd-pie-chart--size-lg {
  --fandhe-pie-chart-size: 22rem;
}

[data-scope="pie-chart"][data-part="root"].fd-pie-chart--size-xl {
  --fandhe-pie-chart-size: 28rem;
}

[data-scope="pie-chart"][data-part="segment"].fd-pie-chart--separator-none {
  stroke: none;
}

[data-scope="pie-chart"][data-part="outside-label"][data-align="end"] {
  text-anchor: end;
}

[data-scope="pie-chart"][data-part="segment"][data-hidden] {
  display: none;
}

[data-scope="pie-chart"][data-part="label"][data-hidden] {
  display: none;
}

[data-scope="pie-chart"][data-part="outside-label"][data-hidden] {
  display: none;
}

[data-scope="pie-chart"][data-part="label-line"][data-hidden] {
  display: none;
}

[data-scope="pie-chart"][data-part="root"][data-has-active] {
  --fandhe-chart-inactive-opacity: 0.4;
  --fandhe-chart-active-scale: 1.05;
}

[data-scope="pie-chart"][data-part="segment"][data-index] {
  opacity: var(--fandhe-chart-inactive-opacity, 1);
  transition-property: opacity, transform;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pie-chart"][data-part="segment"][data-active] {
  opacity: 1;
  transform-box: view-box;
  transform-origin: 50% 50%;
  transform: scale(var(--fandhe-chart-active-scale, 1));
}
"#;

const DONUT_CHART_GOLDEN_CSS: &str = r#"[data-scope="donut-chart"][data-part="root"] {
  display: inline-flex;
  --fandhe-donut-chart-size: 16rem;
  position: relative;
}

[data-scope="donut-chart"][data-part="chart"] {
  width: var(--fandhe-donut-chart-size);
  height: var(--fandhe-donut-chart-size);
}

[data-scope="donut-chart"][data-part="segment"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="donut-chart"][data-part="label"] {
  fill: var(--fandhe-color-fg);
  font-size: 6px;
  text-anchor: middle;
  dominant-baseline: central;
  paint-order: stroke;
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
  stroke-linejoin: round;
}

[data-scope="donut-chart"][data-part="label-line"] {
  stroke: var(--fandhe-color-fg-muted);
  stroke-width: 0.5;
  fill: none;
}

[data-scope="donut-chart"][data-part="outside-label"] {
  fill: var(--fandhe-color-fg);
  font-size: 5px;
  text-anchor: start;
  dominant-baseline: central;
}

[data-scope="donut-chart"][data-part="center-value"] {
  fill: var(--fandhe-color-fg);
  font-size: 12px;
  font-weight: var(--fandhe-font-font-weight-bold);
  text-anchor: middle;
  dominant-baseline: central;
}

[data-scope="donut-chart"][data-part="center-label"] {
  fill: var(--fandhe-color-fg-muted);
  font-size: 4px;
  text-anchor: middle;
  dominant-baseline: central;
}

[data-scope="donut-chart"][data-part="root"].fd-donut-chart--size-xs {
  --fandhe-donut-chart-size: 4rem;
}

[data-scope="donut-chart"][data-part="root"].fd-donut-chart--size-sm {
  --fandhe-donut-chart-size: 10rem;
}

[data-scope="donut-chart"][data-part="root"].fd-donut-chart--size-md {
  --fandhe-donut-chart-size: 16rem;
}

[data-scope="donut-chart"][data-part="root"].fd-donut-chart--size-lg {
  --fandhe-donut-chart-size: 22rem;
}

[data-scope="donut-chart"][data-part="root"].fd-donut-chart--size-xl {
  --fandhe-donut-chart-size: 28rem;
}

[data-scope="donut-chart"][data-part="segment"].fd-donut-chart--separator-none {
  stroke: none;
}

[data-scope="donut-chart"][data-part="outside-label"][data-align="end"] {
  text-anchor: end;
}

[data-scope="donut-chart"][data-part="segment"][data-hidden] {
  display: none;
}

[data-scope="donut-chart"][data-part="label"][data-hidden] {
  display: none;
}

[data-scope="donut-chart"][data-part="outside-label"][data-hidden] {
  display: none;
}

[data-scope="donut-chart"][data-part="label-line"][data-hidden] {
  display: none;
}

[data-scope="donut-chart"][data-part="root"][data-has-active] {
  --fandhe-chart-inactive-opacity: 0.4;
  --fandhe-chart-active-scale: 1.05;
}

[data-scope="donut-chart"][data-part="segment"][data-index] {
  opacity: var(--fandhe-chart-inactive-opacity, 1);
  transition-property: opacity, transform;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="donut-chart"][data-part="segment"][data-active] {
  opacity: 1;
  transform-box: view-box;
  transform-origin: 50% 50%;
  transform: scale(var(--fandhe-chart-active-scale, 1));
}
"#;

#[test]
fn pie_chart_css_matches_golden_fixture() {
    assert_eq!(pie_chart::css(), PIE_CHART_GOLDEN_CSS);
}

#[test]
fn donut_chart_css_matches_golden_fixture() {
    assert_eq!(donut_chart::css(), DONUT_CHART_GOLDEN_CSS);
}

#[test]
fn pie_and_donut_chart_css_never_contain_style_breakout_sequences() {
    for css in [pie_chart::css(), donut_chart::css()] {
        assert!(!css.contains('<'));
        assert!(!css.contains("</style"));
    }
}

#[test]
fn pie_and_donut_chart_css_have_no_color_palette_variant() {
    // モジュール doc「size variant」節参照: セグメント配色はチャート共通
    // パレットの循環で決まるため、color-palette variant を意図的に
    // 提供しない。
    for css in [pie_chart::css(), donut_chart::css()] {
        assert!(!css.contains("color-palette"));
    }
}

/// #2084 以前（イシュー #1596/#1594 是正後）の CSS ブロック群が、本イシュー
/// の追加後も verbatim（1 バイトも変わらず）に含まれていることを固定する
/// （golden 純追加原則、`docs/design/shadcn-reference-adoption-policy.md`
/// §8）。`assert_eq!` の全文比較は `label-line`/`outside-label` 等の中間
/// 挿入により崩れるため、`starts_with` ではなく `contains` でブロック単位
/// に固定する。
#[test]
fn pie_chart_2084_pre_existing_blocks_remain_verbatim() {
    let css = pie_chart::css();
    for block in [
        "[data-scope=\"pie-chart\"][data-part=\"root\"] {\n  display: inline-flex;\n  --fandhe-pie-chart-size: 16rem;\n  position: relative;\n}\n",
        "[data-scope=\"pie-chart\"][data-part=\"chart\"] {\n  width: var(--fandhe-pie-chart-size);\n  height: var(--fandhe-pie-chart-size);\n}\n",
        "[data-scope=\"pie-chart\"][data-part=\"segment\"] {\n  stroke: var(--fandhe-color-bg);\n  stroke-width: 1;\n  stroke-linejoin: round;\n}\n",
        "[data-scope=\"pie-chart\"][data-part=\"label\"] {\n  fill: var(--fandhe-color-fg);\n  font-size: 6px;\n  text-anchor: middle;\n  dominant-baseline: central;\n  paint-order: stroke;\n  stroke: var(--fandhe-color-bg);\n  stroke-width: 1;\n  stroke-linejoin: round;\n}\n",
        "[data-scope=\"pie-chart\"][data-part=\"root\"].fd-pie-chart--size-xs {\n  --fandhe-pie-chart-size: 4rem;\n}\n",
        "[data-scope=\"pie-chart\"][data-part=\"root\"].fd-pie-chart--size-xl {\n  --fandhe-pie-chart-size: 28rem;\n}\n",
    ] {
        assert!(css.contains(block), "missing block: {block}");
    }
}

/// [`pie_chart_2084_pre_existing_blocks_remain_verbatim`] の donut 版。
#[test]
fn donut_chart_2084_pre_existing_blocks_remain_verbatim() {
    let css = donut_chart::css();
    for block in [
        "[data-scope=\"donut-chart\"][data-part=\"root\"] {\n  display: inline-flex;\n  --fandhe-donut-chart-size: 16rem;\n  position: relative;\n}\n",
        "[data-scope=\"donut-chart\"][data-part=\"segment\"] {\n  stroke: var(--fandhe-color-bg);\n  stroke-width: 1;\n  stroke-linejoin: round;\n}\n",
        "[data-scope=\"donut-chart\"][data-part=\"label\"] {\n  fill: var(--fandhe-color-fg);\n  font-size: 6px;\n  text-anchor: middle;\n  dominant-baseline: central;\n  paint-order: stroke;\n  stroke: var(--fandhe-color-bg);\n  stroke-width: 1;\n  stroke-linejoin: round;\n}\n",
        "[data-scope=\"donut-chart\"][data-part=\"root\"].fd-donut-chart--size-xl {\n  --fandhe-donut-chart-size: 28rem;\n}\n",
    ] {
        assert!(css.contains(block), "missing block: {block}");
    }
}
