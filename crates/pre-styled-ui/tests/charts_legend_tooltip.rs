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

use fandhe_frontend_core::{render, text};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::legend::{self, LegendProps};
use fandhe_frontend_pre_styled_ui::charts::tooltip;
use fandhe_frontend_pre_styled_ui::charts::{ChartError, SeriesColor};

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
///
/// イシュー #2086: root に align/marker の既定 variant class（`class="…"`）
/// が付き、marker span 自身にも marker 軸の class が付く（意図した差分。
/// root の直接の子・`data-*`・テキスト内容は不変）。
#[test]
fn legend_composed_from_chart_data_matches_golden_html() {
    let node = legend::legend(
        &sample_data(),
        &LegendProps {
            title: Some("Series".to_string()),
            ..Default::default()
        },
    );
    let html = render(&node);
    assert_eq!(
        html,
        concat!(
            r#"<ul data-scope="chart-legend" data-part="root" role="list" class="fd-chart-legend--align-start fd-chart-legend--marker-circle">"#,
            r#"<li data-scope="chart-legend" data-part="title">Series</li>"#,
            r#"<li data-scope="chart-legend" data-part="item">"#,
            r#"<button type="button" data-scope="chart-legend" data-part="trigger" data-series="visits" aria-pressed="true">"#,
            r#"<span data-scope="chart-legend" data-part="marker" class="fd-chart-legend--marker-circle" style="background: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart-legend" data-part="label">visits</span>"#,
            r#"</button>"#,
            r#"</li>"#,
            r#"<li data-scope="chart-legend" data-part="item">"#,
            r#"<button type="button" data-scope="chart-legend" data-part="trigger" data-series="signups" aria-pressed="true">"#,
            r#"<span data-scope="chart-legend" data-part="marker" class="fd-chart-legend--marker-circle" style="background: var(--fandhe-color-chart-2)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart-legend" data-part="label">signups</span>"#,
            r#"</button>"#,
            r#"</li>"#,
            r#"</ul>"#,
        )
    );
}

/// イシュー #2133: [`LegendProps::hidden_series`] が一致した系列の trigger を
/// `aria-pressed="false"` にし、[`LegendProps::controls`] が全 trigger へ
/// `aria-controls` を付与することを合成 HTML で固定する。
#[test]
fn legend_hidden_series_and_controls_reflected_in_composed_html() {
    let node = legend::legend(
        &sample_data(),
        &LegendProps {
            hidden_series: vec!["signups".to_string()],
            controls: Some("line-1".to_string()),
            ..Default::default()
        },
    );
    let html = render(&node);
    assert!(html.contains(r#"data-series="visits" aria-pressed="true" aria-controls="line-1""#));
    assert!(html.contains(r#"data-series="signups" aria-pressed="false" aria-controls="line-1""#));
}

/// [`LegendProps::hide_marker`]（イシュー #2086）が `true` のとき、marker/
/// icon slot を一切描画しないことを固定する（shadcn/ui `hideIcon` 相当）。
#[test]
fn legend_hide_marker_true_omits_marker_slot_from_composed_html() {
    let props = LegendProps {
        hide_marker: true,
        ..Default::default()
    };
    let html = render(&legend::legend(&sample_data(), &props));
    assert!(!html.contains(r#"data-part="marker""#));
    assert!(!html.contains(r#"data-part="icon""#));
    assert!(html.contains(">visits<"));
    assert!(html.contains(">signups<"));
}

/// [`LegendProps::align`]/[`LegendProps::marker`]（イシュー #2086）の
/// opt-in variant が root/marker span に反映されることを固定する。
#[test]
fn legend_align_center_and_marker_square_reflected_in_composed_html() {
    let props = LegendProps {
        align: legend::LegendAlign::Center,
        marker: legend::LegendMarker::Square,
        ..Default::default()
    };
    let html = render(&legend::legend(&sample_data(), &props));
    assert!(html.contains("fd-chart-legend--align-center"));
    assert!(html.contains(r#"data-part="marker" class="fd-chart-legend--marker-square""#));
}

/// golden レンダリングテスト: [`tooltip::datum_label_lines`]（イシュー
/// #2086）を使ったデータ点が、見出し・複数系列行・footer を `\n` 区切りで
/// `<title>`/`aria-label` の両方に埋め込むことを固定する。
#[test]
fn tooltip_datum_with_datum_label_lines_matches_golden_html() {
    let label = tooltip::datum_label_lines(
        Some("Jan"),
        &[("Visits", "120"), ("Signups", "20")],
        Some("Total: 140"),
    );
    let node = tooltip::datum(1.0, 2.0, 4.0, &label, vec![]);
    let html = render(&node);
    assert_eq!(
        html,
        concat!(
            "<circle data-scope=\"chart\" data-part=\"datum\" cx=\"1\" cy=\"2\" r=\"4\" ",
            "aria-label=\"Jan\nVisits: 120\nSignups: 20\nTotal: 140\">",
            "<title>Jan\nVisits: 120\nSignups: 20\nTotal: 140</title>",
            "</circle>",
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
        ..Default::default()
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

/// 系列に `label`（イシュー #2077）を設定すると凡例が `name` ではなく
/// `display_label()` を表示することを固定する。
#[test]
fn legend_uses_series_display_label_when_present() {
    let data = ChartData::new(
        vec!["Jan".to_string()],
        vec![Series::new("visits", vec![1.0]).with_label("Monthly Visits")],
    )
    .unwrap();
    let html = render(&legend::legend(&data, &LegendProps::default()));
    assert!(html.contains(">Monthly Visits<"));
    assert!(!html.contains(">visits<"));
}

/// 系列の [`SeriesColor`] 上書き（イシュー #2077）が凡例マーカー色に反映
/// されることを固定する（[`fandhe_frontend_pre_styled_ui::charts::ChartData::series_color_var`]
/// を経由する一元性の確認）。
#[test]
fn legend_marker_reflects_series_color_override() {
    let data = ChartData::new(
        vec!["Jan".to_string()],
        vec![Series::new("visits", vec![1.0]).with_color(SeriesColor::token("danger").unwrap())],
    )
    .unwrap();
    let html = render(&legend::legend(&data, &LegendProps::default()));
    assert!(html.contains("background: var(--fandhe-color-danger)"));
}

/// `icon`（イシュー #2077）を設定した系列は `marker` の代わりに
/// `data-part="icon"` を描画する（shadcn/ui `ChartConfig.icon` と同じ
/// 「icon 指定時はマーカーを置換」意味論）。
#[test]
fn legend_renders_icon_slot_instead_of_marker_when_icon_present() {
    let data = ChartData::new(
        vec!["Jan".to_string()],
        vec![
            Series::new("visits", vec![1.0])
                .with_color(SeriesColor::token("chart-2").unwrap())
                .with_icon(text("★")),
            Series::new("signups", vec![1.0]),
        ],
    )
    .unwrap();
    let html = render(&legend::legend(&data, &LegendProps::default()));
    assert!(html.contains(r#"data-part="icon""#));
    assert!(html.contains("color: var(--fandhe-color-chart-2)"));
    assert!(html.contains(">★<"));
    // 2 系列目（icon 未指定）は従来どおり marker を描画する。
    assert!(html.contains(r#"data-part="marker""#));
}

/// icon（[`fandhe_frontend_core::Node`]）内のテキストが既定エスケープを
/// 経由することを固定する（REQ-1、`.claude/rules/coding-rust.md`）。
#[test]
fn xss_regression_legend_icon_content_is_escaped() {
    let payload = "</span><script>alert(1)</script>";
    let data = ChartData::new(
        vec!["a".to_string()],
        vec![Series::new("s", vec![1.0]).with_icon(text(payload))],
    )
    .unwrap();
    let html = render(&legend::legend(&data, &LegendProps::default()));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

/// [`tooltip::layer`]（イシュー #2129、親 #2128）の統合 golden。
/// `entries_from_chart_data` → `layer` の結線を、クレート公開 API 経由の
/// `ChartData` から end-to-end で固定する（各チャート個別の golden とは
/// 独立に、`charts::tooltip` モジュール自体の契約を確認する）。
#[test]
fn tooltip_layer_end_to_end_golden_html() {
    let data = sample_data();
    let html = render(&tooltip::layer(&data, None));
    assert_eq!(
        html,
        concat!(
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="0" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Jan</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">10</span>"#,
            r#"</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="signups">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-2)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">signups</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">1</span>"#,
            r#"</div>"#,
            r#"</div>"#,
            r#"<div data-scope="chart" data-part="tooltip" data-index="1" hidden="">"#,
            r#"<div data-scope="chart" data-part="tooltip-label">Feb</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="visits">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">visits</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">20</span>"#,
            r#"</div>"#,
            r#"<div data-scope="chart" data-part="tooltip-item" data-series="signups">"#,
            r#"<span data-scope="chart" data-part="tooltip-indicator" style="--fandhe-chart-tooltip-color: var(--fandhe-color-chart-2)" aria-hidden="true"></span>"#,
            r#"<span data-scope="chart" data-part="tooltip-name">signups</span>"#,
            r#"<span data-scope="chart" data-part="tooltip-value">2</span>"#,
            r#"</div>"#,
            r#"</div>"#,
            r#"</div>"#,
        )
    );
}

/// `active` 指定時、該当カテゴリのみ `hidden` が省かれることをクレート
/// 公開 API 経由で固定する（`charts::tooltip` の単体テストと同じ契約を
/// 統合レベルでも確認する）。
#[test]
fn tooltip_layer_active_omits_hidden_for_matching_index_only() {
    let data = sample_data();
    let html = render(&tooltip::layer(&data, Some(1)));
    assert_eq!(html.matches(r#"hidden="""#).count(), 1);
    let idx1_tooltip_start = html.find(r#"data-index="1""#).unwrap();
    let idx1_tooltip = &html[idx1_tooltip_start..idx1_tooltip_start + 40];
    assert!(!idx1_tooltip.contains("hidden"));
}

/// XSS 回帰: カテゴリ名・系列表示名が `tooltip-label`/`tooltip-name` の
/// テキストノード、`data-series` 属性値のいずれでも既定エスケープを経由
/// することを固定する（REQ-1、`.claude/rules/coding-rust.md`）。
#[test]
fn xss_regression_tooltip_layer_escapes_category_and_series_names() {
    let payload = "</title><script>alert(1)</script>";
    let data = ChartData::new(
        vec![payload.to_string()],
        vec![Series::new(payload, vec![1.0])],
    )
    .unwrap();
    let html = render(&tooltip::layer(&data, None));
    assert!(!html.contains("<script>"));
    assert_eq!(html.matches("&lt;script&gt;").count(), 3);
}
