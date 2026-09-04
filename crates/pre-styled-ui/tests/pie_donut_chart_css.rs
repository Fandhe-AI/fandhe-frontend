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
}

[data-scope="pie-chart"][data-part="chart"] {
  width: var(--fandhe-pie-chart-size);
  height: var(--fandhe-pie-chart-size);
}

[data-scope="pie-chart"][data-part="segment"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
}

[data-scope="pie-chart"][data-part="label"] {
  fill: var(--fandhe-color-fg);
  font-size: 6px;
  text-anchor: middle;
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
"#;

const DONUT_CHART_GOLDEN_CSS: &str = r#"[data-scope="donut-chart"][data-part="root"] {
  display: inline-flex;
  --fandhe-donut-chart-size: 16rem;
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
