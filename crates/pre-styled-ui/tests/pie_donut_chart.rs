//! styled PieChart / DonutChart（イシュー #850）の render 統合ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/src/pie_chart.rs`/`src/donut_chart.rs` の
//! `#[cfg(test)]` 内単体テストは属性の有無・件数を検証するにとどまるため、
//! 本ファイルは既知データ（4 セグメント 400/300/300/200、
//! `crates/pre-styled-ui/src/charts/pie.rs` の golden 表と同じ入力）に対する
//! `d` 属性の値そのものをバイト単位で固定し、[`crate::charts::pie`] の
//! 角度計算 → [`crate::charts::svg::fmt_coord`] の文字列化までの結線が
//! 崩れた場合に即座に検知する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::charts::pie::PieChartError;
use fandhe_frontend_pre_styled_ui::charts::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};

fn quarterly_data() -> ChartData {
    ChartData::new(
        vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![Series::new("revenue", vec![400.0, 300.0, 300.0, 200.0])],
    )
    .unwrap()
}

#[test]
fn pie_chart_renders_expected_d_attributes_for_known_data() {
    let html = render(&pie_chart(&PieChartProps::default(), &quarterly_data(), vec![]).unwrap());
    assert_eq!(
        html,
        r#"<div data-scope="pie-chart" data-part="root" class="fd-pie-chart--size-md"><svg viewBox="0 0 100 100" role="img" data-scope="pie-chart" data-part="chart" aria-label="pie chart"><path data-scope="pie-chart" data-part="segment" d="M50,50 L50,5 A45,45,0,0,1,88.97,72.5 Z" fill="var(--fandhe-color-chart-1)"></path><path data-scope="pie-chart" data-part="segment" d="M50,50 L88.97,72.5 A45,45,0,0,1,27.5,88.97 Z" fill="var(--fandhe-color-chart-2)"></path><path data-scope="pie-chart" data-part="segment" d="M50,50 L27.5,88.97 A45,45,0,0,1,11.03,27.5 Z" fill="var(--fandhe-color-chart-3)"></path><path data-scope="pie-chart" data-part="segment" d="M50,50 L11.03,27.5 A45,45,0,0,1,50,5 Z" fill="var(--fandhe-color-chart-4)"></path></svg></div>"#
    );
}

#[test]
fn donut_chart_renders_expected_d_attributes_for_known_data() {
    let html =
        render(&donut_chart(&DonutChartProps::default(), &quarterly_data(), vec![]).unwrap());
    assert_eq!(
        html,
        r#"<div data-scope="donut-chart" data-part="root" class="fd-donut-chart--size-md"><svg viewBox="0 0 100 100" role="img" data-scope="donut-chart" data-part="chart" aria-label="donut chart"><path data-scope="donut-chart" data-part="segment" d="M50,5 A45,45,0,0,1,88.97,72.5 L73.38,63.5 A27,27,0,0,0,50,23 Z" fill="var(--fandhe-color-chart-1)"></path><path data-scope="donut-chart" data-part="segment" d="M88.97,72.5 A45,45,0,0,1,27.5,88.97 L36.5,73.38 A27,27,0,0,0,73.38,63.5 Z" fill="var(--fandhe-color-chart-2)"></path><path data-scope="donut-chart" data-part="segment" d="M27.5,88.97 A45,45,0,0,1,11.03,27.5 L26.62,36.5 A27,27,0,0,0,36.5,73.38 Z" fill="var(--fandhe-color-chart-3)"></path><path data-scope="donut-chart" data-part="segment" d="M11.03,27.5 A45,45,0,0,1,50,5 L50,23 A27,27,0,0,0,26.62,36.5 Z" fill="var(--fandhe-color-chart-4)"></path></svg></div>"#
    );
}

#[test]
fn pie_and_donut_chart_render_is_deterministic_across_repeated_calls() {
    let data = quarterly_data();
    let a = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
    let b = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
    assert_eq!(a, b);

    let a = render(&donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap());
    let b = render(&donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap());
    assert_eq!(a, b);
}

#[test]
fn pie_and_donut_chart_a11y_output_has_single_role_and_aria_label() {
    let data = quarterly_data();
    for html in [
        render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap()),
        render(&donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap()),
    ] {
        assert_eq!(html.matches(r#"role="img""#).count(), 1);
        assert_eq!(html.matches("aria-label=").count(), 1);
    }
}

#[test]
fn pie_and_donut_chart_reject_multi_series() {
    let data = ChartData::new(
        vec!["A".to_string(), "B".to_string()],
        vec![
            Series::new("s1", vec![1.0, 2.0]),
            Series::new("s2", vec![3.0, 4.0]),
        ],
    )
    .unwrap();
    assert_eq!(
        pie_chart(&PieChartProps::default(), &data, vec![]).unwrap_err(),
        PieChartError::MultiSeries
    );
    assert_eq!(
        donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap_err(),
        PieChartError::MultiSeries
    );
}
