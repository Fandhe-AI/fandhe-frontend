//! `fandhe-frontend-pre-styled-ui::charts` 基盤の統合テスト（イシュー #846）。
//!
//! 単体テスト（`crates/pre-styled-ui/src/charts/{data,scale,svg}.rs` 内の
//! `#[cfg(test)]`）が各モジュール内部の契約を検証するのに対し、本ファイルは
//! クレート公開 API（`fandhe_frontend_pre_styled_ui::charts::*`）を外部から
//! 呼び出す形で、モジュール間の連携（`ChartData` → `LinearScale` →
//! `svg` ノード木）と XSS 回帰（REQ-1）を固定する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する（既存
//! `tests/xss_escape_styled.rs` と同じ方針）。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series, SortDirection};
use fandhe_frontend_pre_styled_ui::charts::scale::LinearScale;
use fandhe_frontend_pre_styled_ui::charts::svg::{
    circle, fmt_coord, group, line, rect, svg_root, svg_text, PathBuilder, ViewBox,
};
use fandhe_frontend_pre_styled_ui::charts::{series_color_var, ChartError};

/// `ChartData` → `LinearScale` → SVG ノード木という典型的な連携経路を通し、
/// 各段の出力が次段の入力契約（有限値・domain/range）を満たすことを固定する。
#[test]
fn chart_data_scale_and_svg_compose_end_to_end() {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .unwrap();

    let (min, max) = data.domain();
    let scale = LinearScale::new((min, max), (0.0, 100.0)).unwrap().nice();

    let view_box = ViewBox::new(0.0, 0.0, 100.0, 100.0).unwrap();
    let bars = data.series()[0]
        .values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f64 * 10.0;
            let height = scale.scale(v);
            rect(
                x,
                100.0 - height,
                8.0,
                height,
                vec![("fill", &series_color_var(0))],
            )
        })
        .collect();
    let node = svg_root(&view_box, vec![], bars);
    let html = render(&node);

    assert!(html.contains(r#"<svg viewBox="0 0 100 100" role="img">"#));
    assert!(html.contains("<rect"));
    assert!(html.contains(r#"fill="var(--fandhe-color-chart-1)""#));
}

/// tick 値をそのまま [`fmt_coord`] へ渡す経路（軸ラベル描画、#847 が行う想定）
/// が決定的であることを固定する。
#[test]
fn ticks_feed_directly_into_fmt_coord_deterministically() {
    let scale = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
    let ticks = scale.ticks(5).unwrap();
    let formatted: Vec<String> = ticks.iter().copied().map(fmt_coord).collect();
    assert_eq!(formatted, vec!["0", "20", "40", "60", "80", "100"]);

    // 決定性: 同一入力を再実行しても同一出力。
    let formatted_again: Vec<String> = scale
        .ticks(5)
        .unwrap()
        .iter()
        .copied()
        .map(fmt_coord)
        .collect();
    assert_eq!(formatted, formatted_again);
}

/// `sort_by_series` で並び替えたカテゴリ順が SVG ノード木の描画順に反映
/// されることを固定する（軸ラベル・棒の対応関係の一貫性）。
#[test]
fn sorted_chart_data_reorders_svg_output() {
    let data = ChartData::new(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec![Series::new("v", vec![3.0, 1.0, 2.0])],
    )
    .unwrap();
    let sorted = data.sort_by_series("v", SortDirection::Ascending).unwrap();
    assert_eq!(sorted.categories(), &["b", "c", "a"]);

    let labels: Vec<Node> = sorted
        .categories()
        .iter()
        .enumerate()
        .map(|(i, name)| svg_text(i as f64 * 10.0, 0.0, vec![], vec![text(name)]))
        .collect();
    let html = render(&group(vec![], labels));
    let b_pos = html.find(">b<").unwrap();
    let c_pos = html.find(">c<").unwrap();
    let a_pos = html.find(">a<").unwrap();
    assert!(b_pos < c_pos);
    assert!(c_pos < a_pos);
}

/// XSS 回帰: 系列名・カテゴリ名に攻撃ペイロードを与えても、SVG テキスト
/// ノードとして既定エスケープされること（REQ-1）を固定する。
#[test]
fn xss_regression_series_and_category_names_are_escaped_in_svg_text() {
    let payload = "</svg><script>alert(1)</script>";
    let data = ChartData::new(
        vec![payload.to_string()],
        vec![Series::new(payload, vec![1.0])],
    )
    .unwrap();

    let category_label = svg_text(0.0, 0.0, vec![], vec![text(&data.categories()[0])]);
    let series_label = svg_text(0.0, 0.0, vec![], vec![text(&data.series()[0].name)]);

    for node in [category_label, series_label] {
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("</svg><script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}

/// XSS 回帰: 呼び出し側 `attrs` に攻撃ペイロードを渡しても、
/// `fandhe_frontend_core::render` の属性値エスケープを経由すること（REQ-1）
/// を固定する（`fill`/`data-*` 経路）。
#[test]
fn xss_regression_attrs_are_escaped_across_svg_helpers() {
    let payload = "\"><script>alert(1)</script>";
    let nodes = vec![
        rect(0.0, 0.0, 1.0, 1.0, vec![("data-testid", payload)]),
        circle(0.0, 0.0, 1.0, vec![("data-testid", payload)]),
        line(0.0, 0.0, 1.0, 1.0, vec![("data-testid", payload)]),
    ];
    for node in nodes {
        let html = render(&node);
        assert!(!html.contains("<script>"));
    }
}

/// `PathBuilder` の `d` 属性値が閉じた文字集合（`[0-9.\-, MLZ]`）のみで
/// 構成されることを固定する（`crates/headless-ui/src/qr_code.rs` の
/// `pattern_d_attribute_is_closed_character_set` と同型の契約テスト）。
#[test]
fn path_builder_d_attribute_is_closed_character_set() {
    let d = PathBuilder::new()
        .move_to(-12.345, 0.0)
        .line_to(99.999, -0.001)
        .close()
        .build();
    assert!(d
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | ' ' | ',' | 'M' | 'L' | 'Z')));
}

/// [`ChartData::new`]/[`LinearScale::new`] の fail-closed 検証がクレート
/// 公開 API 経由でも機能することを固定する。
#[test]
fn public_api_construction_errors_are_fail_closed() {
    assert_eq!(
        ChartData::new(vec![], vec![]).unwrap_err(),
        ChartError::EmptyData
    );
    assert_eq!(
        LinearScale::new((1.0, 1.0), (0.0, 100.0)).unwrap_err(),
        ChartError::DegenerateDomain
    );
}
