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
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};
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
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="monthly visits">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,150 L150,0 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none"></path>"#,
            r#"</svg></div>"#,
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
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="single point">"#,
            r#"<circle data-scope="line-chart" data-part="point" cx="150" cy="75" r="2.5" fill="var(--fandhe-color-chart-1)"></circle>"#,
            r#"</svg></div>"#,
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
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="negative">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,150 L150,0 L300,80" stroke="var(--fandhe-color-chart-1)" fill="none"></path>"#,
            r#"</svg></div>"#,
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
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md">"#,
            r#"<svg viewBox="0 0 300 150" role="img" data-scope="line-chart" data-part="plot" aria-label="flat">"#,
            r#"<path data-scope="line-chart" data-part="series-line" d="M0,75 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none"></path>"#,
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
            r#"<path data-scope="area-chart" data-part="series-area" d="M0,150 L150,0 L300,75 L300,150 L0,150 Z" fill="var(--fandhe-color-chart-1)"></path>"#,
            r#"<path data-scope="area-chart" data-part="series-line" d="M0,150 L150,0 L300,75" stroke="var(--fandhe-color-chart-1)" fill="none"></path>"#,
            r#"</svg></div>"#,
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
            r#"</svg></div>"#,
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
