//! styled LineChart / AreaChart / Sparkline（イシュー #848）の決定的 CSS
//! 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/stat_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::{area_chart, line_chart, sparkline};

const LINE_CHART_GOLDEN_CSS: &str = r#"[data-scope="line-chart"][data-part="root"] {
  display: block;
  --fandhe-line-chart-height: 150px;
}

[data-scope="line-chart"][data-part="plot"] {
  display: block;
  width: 100%;
  height: var(--fandhe-line-chart-height, auto);
}

[data-scope="line-chart"][data-part="series-line"] {
  fill: none;
  stroke-width: 2;
}

[data-scope="line-chart"][data-part="root"].fd-line-chart--size-sm {
  --fandhe-line-chart-height: 96px;
}

[data-scope="line-chart"][data-part="root"].fd-line-chart--size-md {
  --fandhe-line-chart-height: 150px;
}

[data-scope="line-chart"][data-part="root"].fd-line-chart--size-lg {
  --fandhe-line-chart-height: 220px;
}
"#;

const AREA_CHART_GOLDEN_CSS: &str = r#"[data-scope="area-chart"][data-part="root"] {
  display: block;
  --fandhe-area-chart-height: 150px;
}

[data-scope="area-chart"][data-part="plot"] {
  display: block;
  width: 100%;
  height: var(--fandhe-area-chart-height, auto);
}

[data-scope="area-chart"][data-part="series-area"] {
  fill-opacity: 0.2;
  stroke: none;
}

[data-scope="area-chart"][data-part="series-line"] {
  fill: none;
  stroke-width: 2;
}

[data-scope="area-chart"][data-part="root"].fd-area-chart--size-sm {
  --fandhe-area-chart-height: 96px;
}

[data-scope="area-chart"][data-part="root"].fd-area-chart--size-md {
  --fandhe-area-chart-height: 150px;
}

[data-scope="area-chart"][data-part="root"].fd-area-chart--size-lg {
  --fandhe-area-chart-height: 220px;
}
"#;

const SPARKLINE_GOLDEN_CSS: &str = r#"[data-scope="sparkline"][data-part="root"] {
  display: inline-block;
  --fandhe-sparkline-height: 48px;
}

[data-scope="sparkline"][data-part="plot"] {
  display: block;
  width: auto;
  height: var(--fandhe-sparkline-height, auto);
}

[data-scope="sparkline"][data-part="series-area"] {
  fill-opacity: 0.2;
  stroke: none;
}

[data-scope="sparkline"][data-part="series-line"] {
  fill: none;
  stroke-width: 1.5;
}

[data-scope="sparkline"][data-part="root"].fd-sparkline--size-sm {
  --fandhe-sparkline-height: 32px;
}

[data-scope="sparkline"][data-part="root"].fd-sparkline--size-md {
  --fandhe-sparkline-height: 48px;
}

[data-scope="sparkline"][data-part="root"].fd-sparkline--size-lg {
  --fandhe-sparkline-height: 64px;
}
"#;

#[test]
fn line_chart_stylesheet_matches_golden_css() {
    assert_eq!(line_chart::stylesheet(), LINE_CHART_GOLDEN_CSS);
}

#[test]
fn area_chart_stylesheet_matches_golden_css() {
    assert_eq!(area_chart::stylesheet(), AREA_CHART_GOLDEN_CSS);
}

#[test]
fn sparkline_stylesheet_matches_golden_css() {
    assert_eq!(sparkline::stylesheet(), SPARKLINE_GOLDEN_CSS);
}
