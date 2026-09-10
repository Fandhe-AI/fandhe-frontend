//! LineChart / AreaChart / Sparkline（イシュー #848、親 Phase #845）の統合
//! テスト。`charts` 基盤（#846）の最初の消費者として、同一 `ChartData` /
//! `values` から常に同一 SVG マークアップが生成されることを golden HTML で
//! 固定し、a11y 属性（`role="img"`/`aria-label`）・エッジケース（単一
//! カテゴリ・負値・フラットデータ・空データ・非有限値）・XSS 回帰を横断的に
//! 検証する。
//!
//! 各コンポーネント単体の詳細（クラス付与・variant・stylesheet 等）は
//! `crates/pre-styled-ui/src/{line_chart,area_chart,sparkline}.rs` の
//! `#[cfg(test)]` を参照。本ファイルは 3 部品を横断する golden HTML と、
//! XSS 回帰テスト（`.claude/rules/coding-rust.md`
//! 「XSS 回帰テストは削除・弱体化しない」）を担う。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::area_chart::{area_chart, AreaChartProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::ChartError;
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps, LineDots, LineLabel};
use fandhe_frontend_pre_styled_ui::sparkline::{sparkline, SparklineProps};

fn normal_data() -> ChartData {
    ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .unwrap()
}

fn single_data() -> ChartData {
    ChartData::new(
        vec!["only".to_string()],
        vec![Series::new("visits", vec![7.0])],
    )
    .unwrap()
}

fn negative_data() -> ChartData {
    ChartData::new(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec![Series::new("delta", vec![-10.0, 5.0, -3.0])],
    )
    .unwrap()
}

fn flat_data() -> ChartData {
    ChartData::new(
        vec!["a".to_string(), "b".to_string()],
        vec![Series::new("flat", vec![5.0, 5.0])],
    )
    .unwrap()
}

// ---------------------------------------------------------------------
// LineChart golden HTML
// ---------------------------------------------------------------------

#[test]
fn line_chart_normal_data_matches_golden_html() {
    let data = normal_data();
    let node = line_chart(&LineChartProps::new(&data, "monthly visits"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="monthly visits">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,150 L150,0 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="visits"></path><rect x="0" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="Jan · visits: 10">"#,
            r#"</rect><rect x="75" y="0" width="150" height="150" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="Feb · visits: 30">"#,
            r#"</rect><rect x="225" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="Mar · visits: 20">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Jan</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">10</span></div></div><div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Feb</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">30</span></div></div><div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Mar</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">20</span></div></div></div></div>"#,
        )
    );
}

#[test]
fn line_chart_single_category_matches_golden_html() {
    let data = single_data();
    let node = line_chart(&LineChartProps::new(&data, "single point"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="single point">"#,
            r#"<circle data-scope="line-chart" data-part="point" cx="150" cy="75" r="2.5" fill="var(--fandhe-color-chart-1)" data-series="visits"></circle><rect x="0" y="0" width="300" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="only · visits: 7">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">only</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">7</span></div></div></div></div>"#,
        )
    );
}

#[test]
fn line_chart_negative_values_matches_golden_html() {
    let data = negative_data();
    let node = line_chart(&LineChartProps::new(&data, "negative"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="negative">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,150 L150,0 L300,80" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="delta"></path><rect x="0" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="a · delta: -10">"#,
            r#"</rect><rect x="75" y="0" width="150" height="150" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="b · delta: 5">"#,
            r#"</rect><rect x="225" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="c · delta: -3">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">a</div><div data-scope="chart" data-part="tooltip-item" data-series="delta"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">delta</span><span data-scope="chart" data-part="tooltip-value">-10</span></div></div><div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">b</div><div data-scope="chart" data-part="tooltip-item" data-series="delta"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">delta</span><span data-scope="chart" data-part="tooltip-value">5</span></div></div><div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">c</div><div data-scope="chart" data-part="tooltip-item" data-series="delta"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">delta</span><span data-scope="chart" data-part="tooltip-value">-3</span></div></div></div></div>"#,
        )
    );
}

#[test]
fn line_chart_flat_data_matches_golden_html_center_line() {
    let data = flat_data();
    let node = line_chart(&LineChartProps::new(&data, "flat"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="flat">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,75 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="flat"></path><rect x="0" y="0" width="150" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="a · flat: 5">"#,
            r#"</rect><rect x="150" y="0" width="150" height="150" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="b · flat: 5">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">a</div><div data-scope="chart" data-part="tooltip-item" data-series="flat"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">flat</span><span data-scope="chart" data-part="tooltip-value">5</span></div></div><div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">b</div><div data-scope="chart" data-part="tooltip-item" data-series="flat"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">flat</span><span data-scope="chart" data-part="tooltip-value">5</span></div></div></div></div>"#,
        )
    );
}

// イシュー #2083（shadcn/ui Charts（line）突合）: `dots`/`label` の合成
// golden。`n == 3` 3 系列点それぞれに `point`（`data-part="point"`）+
// `value-label` を出す（余白 `top` は `LABEL_TOP_MARGIN` 20px のみ確保）。
#[test]
fn line_chart_dots_and_value_label_matches_golden_html() {
    let data = normal_data();
    let mut props = LineChartProps::new(&data, "dots and label");
    props.dots = LineDots::Filled;
    props.label = LineLabel::Value;
    let node = line_chart(&props, vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="dots and label">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,150 L150,20 L300,85" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="visits"></path><circle data-scope="line-chart" data-part="point" cx="0" cy="150" r="2.5" fill="var(--fandhe-color-chart-1)" data-series="visits">"#,
            r#"</circle><circle data-scope="line-chart" data-part="point" cx="150" cy="20" r="2.5" fill="var(--fandhe-color-chart-1)" data-series="visits"></circle><circle data-scope="line-chart" data-part="point" cx="300" cy="85" r="2.5" fill="var(--fandhe-color-chart-1)" data-series="visits">"#,
            r#"</circle><text x="0" y="138" data-scope="line-chart" data-part="value-label" text-anchor="middle" data-series="visits">10</text><text x="150" y="8" data-scope="line-chart" data-part="value-label" text-anchor="middle" data-series="visits">"#,
            r#"30</text><text x="300" y="73" data-scope="line-chart" data-part="value-label" text-anchor="middle" data-series="visits">20</text><rect x="0" y="20" width="75" height="130" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="Jan · visits: 10">"#,
            r#"</rect><rect x="75" y="20" width="150" height="130" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="Feb · visits: 30">"#,
            r#"</rect><rect x="225" y="20" width="75" height="130" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="Mar · visits: 20">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Jan</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">10</span></div></div><div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Feb</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">30</span></div></div><div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Mar</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">20</span></div></div></div></div>"#,
        )
    );
}

// イシュー #2083: 軸/グリッド合成（`show_x_axis`/`show_y_axis`/
// `show_grid` 全有効）の golden。area-chart の余白規則
// （`AXIS_LEFT_MARGIN`/`AXIS_BOTTOM_MARGIN`）を共有することを固定する。
#[test]
fn line_chart_axes_matches_golden_html() {
    let data = normal_data();
    let mut props = LineChartProps::new(&data, "axes");
    props.show_x_axis = true;
    props.show_y_axis = true;
    props.show_grid = true;
    let node = line_chart(&props, vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md"><svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="axes">"#,
            r#"<g data-scope="chart" data-part="grid"><line x1="40" y1="126" x2="300" y2="126" data-scope="chart" data-part="grid-line" class="fd-chart--lines-solid">"#,
            r#"</line><line x1="40" y1="94.5" x2="300" y2="94.5" data-scope="chart" data-part="grid-line" class="fd-chart--lines-solid"></line><line x1="40" y1="63" x2="300" y2="63" data-scope="chart" data-part="grid-line" class="fd-chart--lines-solid">"#,
            r#"</line><line x1="40" y1="31.5" x2="300" y2="31.5" data-scope="chart" data-part="grid-line" class="fd-chart--lines-solid"></line><line x1="40" y1="0" x2="300" y2="0" data-scope="chart" data-part="grid-line" class="fd-chart--lines-solid">"#,
            r#"</line></g><path data-scope="line-chart" data-part="series-line" d="M40,126 L170,0 L300,63" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="visits"></path><g data-scope="chart" data-part="y-axis">"#,
            r#"<text x="30" y="126" data-scope="chart" data-part="tick-label" text-anchor="end" dominant-baseline="middle">10</text><text x="30" y="94.5" data-scope="chart" data-part="tick-label" text-anchor="end" dominant-baseline="middle">"#,
            r#"15</text><text x="30" y="63" data-scope="chart" data-part="tick-label" text-anchor="end" dominant-baseline="middle">20</text><text x="30" y="31.5" data-scope="chart" data-part="tick-label" text-anchor="end" dominant-baseline="middle">"#,
            r#"25</text><text x="30" y="0" data-scope="chart" data-part="tick-label" text-anchor="end" dominant-baseline="middle">30</text></g><text data-scope="chart" data-part="tick-label" x="40" y="142" text-anchor="middle">"#,
            r#"Jan</text><text data-scope="chart" data-part="tick-label" x="170" y="142" text-anchor="middle">Feb</text><text data-scope="chart" data-part="tick-label" x="300" y="142" text-anchor="middle">"#,
            r#"Mar</text><g data-scope="chart" data-part="x-axis"><line x1="40" y1="126" x2="300" y2="126" data-scope="chart" data-part="axis-line"></line></g><rect x="40" y="0" width="65" height="126" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="Jan · visits: 10">"#,
            r#"</rect><rect x="105" y="0" width="130" height="126" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="Feb · visits: 30">"#,
            r#"</rect><rect x="235" y="0" width="65" height="126" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="Mar · visits: 20">"#,
            r#"</rect></svg><div data-scope="chart" data-part="tooltip-layer" aria-hidden="true"><div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Jan</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">10</span></div></div><div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Feb</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">30</span></div></div><div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Mar</div><div data-scope="chart" data-part="tooltip-item" data-series="visits"><span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true">"#,
            r#"</span><span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">20</span></div></div></div></div>"#,
        )
    );
}

#[test]
fn line_chart_show_tooltip_false_matches_pre_2129_golden_html() {
    // イシュー #2129: `show_tooltip: false` は本イシュー以前の出力と
    // バイト一致する（progressive enhancement の機械的保証、opt-out 経路）。
    let data = single_data();
    let mut props = LineChartProps::new(&data, "single point");
    props.show_tooltip = false;
    let node = line_chart(&props, vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="single point">"#,
            r#"<circle data-scope="line-chart" data-part="point" cx="150" cy="75" r="2.5" fill="var(--fandhe-color-chart-1)"></circle>"#,
            r#"</svg></div>"#,
        )
    );
}

// ---------------------------------------------------------------------
// AreaChart golden HTML
// ---------------------------------------------------------------------

#[test]
fn area_chart_normal_data_matches_golden_html() {
    let data = normal_data();
    let node = area_chart(&AreaChartProps::new(&data, "monthly visits"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="area-chart" data-part="root" class="fd-area-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="area-chart" data-part="plot" aria-label="monthly visits">"#,
            r#"<path data-scope="area-chart" data-part="series-area" d="M0,150 L150,0 L300,75 L300,150 L0,150 Z" fill="var(--fandhe-color-chart-1)" data-series="visits"></path>"#,
            r#"<path data-scope="area-chart" data-part="series-line" d="M0,150 L150,0 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none" data-series="visits"></path>"#,
            r#"<rect x="0" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="Jan · visits: 10"></rect>"#,
            r#"<rect x="75" y="0" width="150" height="150" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="Feb · visits: 30"></rect>"#,
            r#"<rect x="225" y="0" width="75" height="150" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="Mar · visits: 20"></rect>"#,
            r#"</svg>"#,
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Jan</div><div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">10</span>"#,
            r#"</div></div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Feb</div><div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">30</span>"#,
            r#"</div></div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Mar</div><div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">20</span>"#,
            r#"</div></div>"#,
            r#"</div></div>"#,
        )
    );
}

#[test]
fn area_chart_single_category_matches_golden_html() {
    let data = single_data();
    let node = area_chart(&AreaChartProps::new(&data, "single point"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="area-chart" data-part="root" class="fd-area-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="area-chart" data-part="plot" aria-label="single point">"#,
            r#"<circle data-scope="area-chart" data-part="point" cx="150" cy="75" r="2.5" fill="var(--fandhe-color-chart-1)" data-series="visits"></circle>"#,
            r#"<rect x="0" y="0" width="300" height="150" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="only · visits: 7"></rect>"#,
            r#"</svg>"#,
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">only</div><div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span><span data-scope="chart" data-part="tooltip-value">7</span>"#,
            r#"</div></div>"#,
            r#"</div></div>"#,
        )
    );
}

#[test]
fn area_chart_show_tooltip_false_matches_pre_2129_golden_html() {
    // イシュー #2129: `show_tooltip: false` は本イシュー以前の出力と
    // バイト一致する（progressive enhancement の機械的保証、opt-out 経路）。
    let data = single_data();
    let mut props = AreaChartProps::new(&data, "single point");
    props.show_tooltip = false;
    let node = area_chart(&props, vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="area-chart" data-part="root" class="fd-area-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="area-chart" data-part="plot" aria-label="single point">"#,
            r#"<circle data-scope="area-chart" data-part="point" cx="150" cy="75" r="2.5" fill="var(--fandhe-color-chart-1)"></circle>"#,
            r#"</svg></div>"#,
        )
    );
}

// ---------------------------------------------------------------------
// Sparkline golden HTML
// ---------------------------------------------------------------------

#[test]
fn sparkline_normal_values_matches_golden_html() {
    let values = [10.0, 30.0, 20.0, 40.0];
    let node = sparkline(&SparklineProps::new(&values, "weekly trend"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="sparkline" data-part="root" class="fd-sparkline--size-md">"#,
            r#"<svg viewBox="0 0 112 48" role="img" data-scope="sparkline" data-part="plot" aria-label="weekly trend">"#,
            r#"<path data-scope="sparkline" data-part="series-area" d="M0,48 L37.33,16 L74.67,32 L112,0 L112,48 L0,48 Z" fill="var(--fandhe-color-chart-1)"></path>"#,
            r#"<path data-scope="sparkline" data-part="series-line" d="M0,48 L37.33,16 L74.67,32 L112,0" stroke="var(--fandhe-color-chart-1)" fill="none"></path>"#,
            r#"<rect x="0" y="0" width="18.67" height="48" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="0 · value: 10"></rect>"#,
            r#"<rect x="18.67" y="0" width="37.33" height="48" data-scope="chart" data-part="hit-area" data-index="1" fill="none" pointer-events="none" tabindex="-1" aria-label="1 · value: 30"></rect>"#,
            r#"<rect x="56" y="0" width="37.33" height="48" data-scope="chart" data-part="hit-area" data-index="2" fill="none" pointer-events="none" tabindex="-1" aria-label="2 · value: 20"></rect>"#,
            r#"<rect x="93.33" y="0" width="18.67" height="48" data-scope="chart" data-part="hit-area" data-index="3" fill="none" pointer-events="none" tabindex="-1" aria-label="3 · value: 40"></rect>"#,
            r#"</svg>"#,
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">0</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="value">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">value</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">10</span>"#,
            r#"</div></div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">1</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="value">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">value</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">30</span>"#,
            r#"</div></div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="2" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">2</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="value">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">value</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">20</span>"#,
            r#"</div></div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="3" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">3</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="value">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">value</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">40</span>"#,
            r#"</div></div>"#,
            r#"</div></div>"#,
        )
    );
}

#[test]
fn sparkline_single_value_matches_golden_html() {
    let values = [7.0];
    let node = sparkline(&SparklineProps::new(&values, "single"), vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="sparkline" data-part="root" class="fd-sparkline--size-md">"#,
            r#"<svg viewBox="0 0 112 48" role="img" data-scope="sparkline" data-part="plot" aria-label="single">"#,
            r#"<circle data-scope="sparkline" data-part="point" cx="56" cy="24" r="2.5" fill="var(--fandhe-color-chart-1)"></circle>"#,
            r#"<rect x="0" y="0" width="112" height="48" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="0 · value: 7"></rect>"#,
            r#"</svg>"#,
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">0</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="value">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">value</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">7</span>"#,
            r#"</div></div>"#,
            r#"</div></div>"#,
        )
    );
}

#[test]
fn sparkline_show_tooltip_false_matches_pre_2129_golden_html() {
    // イシュー #2129: `show_tooltip: false` は本イシュー以前の出力と
    // バイト一致する（progressive enhancement の機械的保証、opt-out 経路）。
    let values = [7.0];
    let mut props = SparklineProps::new(&values, "single");
    props.show_tooltip = false;
    let node = sparkline(&props, vec![]).unwrap();
    assert_eq!(
        render(&node),
        concat!(
            r#"<div data-scope="sparkline" data-part="root" class="fd-sparkline--size-md">"#,
            r#"<svg viewBox="0 0 112 48" role="img" data-scope="sparkline" data-part="plot" aria-label="single">"#,
            r#"<circle data-scope="sparkline" data-part="point" cx="56" cy="24" r="2.5" fill="var(--fandhe-color-chart-1)"></circle>"#,
            r#"</svg></div>"#,
        )
    );
}

// ---------------------------------------------------------------------
// 決定性（同一入力を 2 回描画しても同一出力）
// ---------------------------------------------------------------------

#[test]
fn all_three_components_are_deterministic_across_repeated_renders() {
    let data = normal_data();
    let a1 = render(&line_chart(&LineChartProps::new(&data, "det"), vec![]).unwrap());
    let a2 = render(&line_chart(&LineChartProps::new(&data, "det"), vec![]).unwrap());
    assert_eq!(a1, a2);

    let b1 = render(&area_chart(&AreaChartProps::new(&data, "det"), vec![]).unwrap());
    let b2 = render(&area_chart(&AreaChartProps::new(&data, "det"), vec![]).unwrap());
    assert_eq!(b1, b2);

    let values = [1.0, 4.0, 2.0];
    let c1 = render(&sparkline(&SparklineProps::new(&values, "det"), vec![]).unwrap());
    let c2 = render(&sparkline(&SparklineProps::new(&values, "det"), vec![]).unwrap());
    assert_eq!(c1, c2);
}

// ---------------------------------------------------------------------
// a11y: role="img" と aria-label が全部品で固定して出力される
// ---------------------------------------------------------------------

#[test]
fn all_three_components_render_role_img_and_aria_label() {
    let data = normal_data();
    let values = [1.0, 2.0];

    let line_html = render(&line_chart(&LineChartProps::new(&data, "line a11y"), vec![]).unwrap());
    assert!(line_html.contains(r#"role="img""#));
    assert!(line_html.contains(r#"aria-label="line a11y""#));

    let area_html = render(&area_chart(&AreaChartProps::new(&data, "area a11y"), vec![]).unwrap());
    assert!(area_html.contains(r#"role="img""#));
    assert!(area_html.contains(r#"aria-label="area a11y""#));

    let spark_html =
        render(&sparkline(&SparklineProps::new(&values, "sparkline a11y"), vec![]).unwrap());
    assert!(spark_html.contains(r#"role="img""#));
    assert!(spark_html.contains(r#"aria-label="sparkline a11y""#));
}

// ---------------------------------------------------------------------
// fail-closed: 空データ・非有限値は panic せず Err を返す
// ---------------------------------------------------------------------

#[test]
fn chart_data_construction_rejects_empty_and_non_finite_before_reaching_components() {
    // ChartData::new が空・非有限値を拒否する契約（`charts` 基盤 #846）を、
    // 本イシューの 3 部品が実際にその契約の上でのみ動作していることの
    // 回帰として再確認する（本モジュール自体は追加の検証を行わない）。
    assert_eq!(
        ChartData::new(vec![], vec![Series::new("s", vec![])]).unwrap_err(),
        ChartError::EmptyData
    );
    assert_eq!(
        ChartData::new(
            vec!["a".to_string()],
            vec![Series::new("s", vec![f64::NAN])]
        )
        .unwrap_err(),
        ChartError::NonFiniteValue
    );
}

#[test]
fn sparkline_rejects_empty_values_without_panicking() {
    let values: [f64; 0] = [];
    assert_eq!(
        sparkline(&SparklineProps::new(&values, "empty"), vec![]).unwrap_err(),
        ChartError::EmptyData
    );
}

#[test]
fn sparkline_rejects_non_finite_values_without_panicking() {
    let values = [1.0, f64::INFINITY];
    assert_eq!(
        sparkline(&SparklineProps::new(&values, "inf"), vec![]).unwrap_err(),
        ChartError::NonFiniteValue
    );
}

// ---------------------------------------------------------------------
// XSS 回帰（`.claude/rules/coding-rust.md`: 削除・弱体化しない）
// ---------------------------------------------------------------------

const XSS_PAYLOADS: &[&str] = &[
    "\"><script>alert(1)</script>",
    "\"><img src=x onerror=alert(1)>",
    "javascript:alert(1)",
    "'-alert(1)-'",
    "</style><script>alert(1)</script>",
];

#[test]
fn line_chart_aria_label_is_escaped_for_all_xss_payloads() {
    let data = normal_data();
    for payload in XSS_PAYLOADS {
        let html = render(&line_chart(&LineChartProps::new(&data, payload), vec![]).unwrap());
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("</style>"));
    }
}

#[test]
fn area_chart_aria_label_is_escaped_for_all_xss_payloads() {
    let data = normal_data();
    for payload in XSS_PAYLOADS {
        let html = render(&area_chart(&AreaChartProps::new(&data, payload), vec![]).unwrap());
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("</style>"));
    }
}

#[test]
fn sparkline_aria_label_is_escaped_for_all_xss_payloads() {
    let values = [1.0, 2.0];
    for payload in XSS_PAYLOADS {
        let html = render(&sparkline(&SparklineProps::new(&values, payload), vec![]).unwrap());
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("</style>"));
    }
}

#[test]
fn all_three_components_escape_caller_supplied_attrs_for_all_xss_payloads() {
    let data = normal_data();
    let values = [1.0, 2.0];
    for payload in XSS_PAYLOADS {
        let line_html = render(
            &line_chart(
                &LineChartProps::new(&data, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert!(!line_html.contains("<script>"));

        let area_html = render(
            &area_chart(
                &AreaChartProps::new(&data, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert!(!area_html.contains("<script>"));

        let spark_html = render(
            &sparkline(
                &SparklineProps::new(&values, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert!(!spark_html.contains("<script>"));
    }
}

#[test]
fn all_three_components_drop_caller_class_attr_instead_of_merging_raw() {
    let data = normal_data();
    let values = [1.0, 2.0];
    let payload = "attacker-controlled\"><script>alert(1)</script>";

    let line_html = render(
        &line_chart(
            &LineChartProps::new(&data, "class"),
            vec![("class", payload)],
        )
        .unwrap(),
    );
    assert!(!line_html.contains("attacker-controlled"));
    assert_eq!(line_html.matches("class=\"").count(), 1);

    let area_html = render(
        &area_chart(
            &AreaChartProps::new(&data, "class"),
            vec![("class", payload)],
        )
        .unwrap(),
    );
    assert!(!area_html.contains("attacker-controlled"));
    assert_eq!(area_html.matches("class=\"").count(), 1);

    let spark_html = render(
        &sparkline(
            &SparklineProps::new(&values, "class"),
            vec![("class", payload)],
        )
        .unwrap(),
    );
    assert!(!spark_html.contains("attacker-controlled"));
    assert_eq!(spark_html.matches("class=\"").count(), 1);
}
