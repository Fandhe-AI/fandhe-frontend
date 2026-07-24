//! `fandhe-frontend-pre-styled-ui::charts::{legend,tooltip}` の統合テスト
//! （イシュー #847）。
//!
//! `tests/charts_foundation.rs`（#846 基盤）と同型: クレート公開 API を外部
//! から呼び出し、[`fandhe_frontend_pre_styled_ui::charts::data::ChartData`]
//! との合成・golden レンダリング・fail-closed エラー・XSS 回帰（REQ-1）を
//! 固定する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::legend::{self, LegendProps};
use fandhe_frontend_pre_styled_ui::charts::tooltip;
use fandhe_frontend_pre_styled_ui::charts::ChartError;

fn sample_data() -> ChartData {
    ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string()],
        vec![
            Series::new("visits", vec![10.0, 20.0]),
            Series::new("signups", vec![1.0, 2.0]),
        ],
    )
    .unwrap()
}

/// golden レンダリングテスト: 2 系列の [`ChartData`] から組み立てた凡例が
/// 期待する HTML と全文一致することを固定する。
#[test]
fn legend_composed_from_chart_data_matches_golden_html() {
    let node = legend::legend(
        &sample_data(),
        &LegendProps {
            title: Some("Series".to_string()),
        },
    );
    let html = render(&node);
    assert_eq!(
        html,
        concat!(
            r#"<ul data-scope="chart-legend" data-part="root" role="list">"#,
            r#"<li data-scope="chart-legend" data-part="title">Series</li>"#,
            r#"<li data-scope="chart-legend" data-part="item">"#,
            r#"<span data-scope="chart-legend" data-part="marker" style="background: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart-legend" data-part="label">visits</span>"#,
            r#"</li>"#,
            r#"<li data-scope="chart-legend" data-part="item">"#,
            r#"<span data-scope="chart-legend" data-part="marker" style="background: var(--fandhe-color-chart-2)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart-legend" data-part="label">signups</span>"#,
            r#"</li>"#,
            r#"</ul>"#,
        )
    );
}

/// golden レンダリングテスト: [`tooltip::datum`] が `<title>`/`aria-label`
/// を伴う `<circle>` を期待通りに組み立てることを固定する。
#[test]
fn tooltip_datum_with_datum_label_matches_golden_html() {
    let label = tooltip::datum_label("Jan", "visits", 10.0);
    let node = tooltip::datum(
        1.0,
        2.0,
        4.0,
        &label,
        vec![("fill", "var(--fandhe-color-chart-1)")],
    );
    let html = render(&node);
    assert_eq!(
        html,
        concat!(
            r#"<circle data-scope="chart" data-part="datum" cx="1" cy="2" r="4" "#,
            r#"aria-label="Jan · visits: 10" fill="var(--fandhe-color-chart-1)">"#,
            r#"<title>Jan · visits: 10</title>"#,
            r#"</circle>"#,
        )
    );
}

/// [`legend::legend`] のマーカー色が [`fandhe_frontend_pre_styled_ui::charts::series_color_var`]
/// と同じ 6 色循環を系列インデックス順に割り当てることを固定する。
#[test]
fn legend_marker_colors_cycle_through_six_slots_like_series_color_var() {
    use fandhe_frontend_pre_styled_ui::charts::series_color_var;

    let categories = vec!["a".to_string()];
    let series: Vec<Series> = (0..8)
        .map(|i| Series::new(format!("s{i}"), vec![1.0]))
        .collect();
    let data = ChartData::new(categories, series).unwrap();
    let html = render(&legend::legend(&data, &LegendProps::default()));

    for i in 0..8 {
        assert!(html.contains(&series_color_var(i)));
    }
    // 7 番目（index 6）は index 0 と同じ色（chart-1）に循環する。
    assert_eq!(series_color_var(0), series_color_var(6));
}

/// fail-closed 検証: [`ChartData::new`] の空データ拒否がクレート公開 API
/// 経由でも機能することを固定する（凡例・ツールチップの入力データ経路）。
#[test]
fn public_api_construction_errors_are_fail_closed() {
    assert_eq!(
        ChartData::new(vec![], vec![]).unwrap_err(),
        ChartError::EmptyData
    );
}

/// XSS 回帰: 凡例タイトル・系列名に攻撃ペイロードを与えても、既定エスケープ
/// されること（REQ-1）を固定する。
#[test]
fn xss_regression_legend_title_and_series_name_are_escaped() {
    let payload = "</ul><script>alert(1)</script>";
    let data =
        ChartData::new(vec!["a".to_string()], vec![Series::new(payload, vec![1.0])]).unwrap();
    let props = LegendProps {
        title: Some(payload.to_string()),
    };
    let html = render(&legend::legend(&data, &props));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

/// XSS 回帰: [`tooltip::datum_label`] の入力（カテゴリ・系列名）が
/// `<title>`/`aria-label` の両方で既定エスケープされること（REQ-1）を固定する。
#[test]
fn xss_regression_tooltip_label_is_escaped_in_title_and_aria_label() {
    let payload = "</title><script>alert(1)</script>";
    let label = tooltip::datum_label(payload, "visits", 1.0);
    let html = render(&tooltip::datum(0.0, 0.0, 1.0, &label, vec![]));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

/// 呼び出し側 `attrs` に攻撃ペイロードを渡しても、[`tooltip::datum`] が
/// [`fandhe_frontend_core::render`] のエスケープ経路を通ること（REQ-1）を
/// 固定する（`fill` 等の見た目属性経路）。
#[test]
fn xss_regression_datum_attrs_are_escaped() {
    let payload = "\"><script>alert(1)</script>";
    let html = render(&tooltip::datum(
        0.0,
        0.0,
        1.0,
        "safe",
        vec![("data-testid", payload)],
    ));
    assert!(!html.contains("<script>"));
}
