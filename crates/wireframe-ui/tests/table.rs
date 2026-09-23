//! `table` 部品の契約テスト（イシュー #2662）。
//!
//! `crates/wireframe-ui/tests/calendar.rs`（非対話制約・XSS 回帰・CSS 配線
//! の観点）・`grid.rs`（資源有界化 clamp のテスト先例）と同型の観点に
//! 加え、ヘッダー有無・列数導出・短い行の空セル埋め・行の飽和を
//! wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{table, wireframe_css, Size, PARTS};

const HEADERS: [&str; 2] = ["Name", "Age"];
const ROWS: [[&str; 2]; 2] = [["Alice", "30"], ["Bob", "25"]];

#[test]
fn renders_root_class_for_every_size() {
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();
    for size in Size::ALL {
        let node = table(&HEADERS, &rows, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-table {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn root_open_tag_has_only_class() {
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();
    let html = render(&table(&HEADERS, &rows, Size::Md));
    let root_open_tag_end = html.find('>').expect("root open tag");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with(r#"<div class="fw-wire-table"#));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn header_row_is_omitted_when_headers_is_empty() {
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();
    let with_header = render(&table(&HEADERS, &rows, Size::Md));
    assert_eq!(
        with_header
            .matches(r#"class="fw-wire-table-header""#)
            .count(),
        1
    );

    let without_header = render(&table(&[], &rows, Size::Md));
    assert!(!without_header.contains("fw-wire-table-header"));
}

#[test]
fn columns_are_derived_from_header_and_row_lengths() {
    let rows: &[&[&str]] = &[&["a", "b", "c"]];
    let html = render(&table(&["x"], rows, Size::Md));
    // headers.len() == 1, row.len() == 3 -> columns == 3
    assert!(html.contains("fw-wire-table-cols-3"));
}

#[test]
fn columns_are_clamped_to_max_table_columns() {
    let wide_row: [&str; 20] = ["x"; 20];
    let rows: &[&[&str]] = &[&wide_row];
    let html = render(&table(&[], rows, Size::Md));
    assert!(html.contains("fw-wire-table-cols-12"));
    assert!(!html.contains("fw-wire-table-cols-13"));
}

#[test]
fn short_rows_are_padded_with_empty_cells() {
    let headers = ["A", "B", "C"];
    let rows: &[&[&str]] = &[&["1"]];
    let html = render(&table(&headers, rows, Size::Md));
    // ヘッダー行（3 セル、不足なし）+ データ行（1 セル + 空セル 2 個）。
    assert_eq!(html.matches("fw-wire-table-cell-empty").count(), 2);
}

#[test]
fn rows_longer_than_clamped_column_count_are_truncated() {
    // 15 セルの行は列数の由来だが MAX_TABLE_COLUMNS（12）へ飽和するため、
    // 実際に出力されるセルは先頭 12 個のみで 13 番目以降は出力されない。
    let wide_row: Vec<&str> = (1..=15)
        .map(|n| match n {
            1 => "c1",
            2 => "c2",
            _ => "cN",
        })
        .collect();
    let rows: &[&[&str]] = &[&wide_row];
    let html = render(&table(&[], rows, Size::Md));
    assert!(html.contains("fw-wire-table-cols-12"));
    assert_eq!(html.matches(r#"class="fw-wire-table-cell""#).count(), 12);
}

#[test]
fn empty_headers_and_rows_do_not_panic() {
    let html = render(&table(&[], &[], Size::Md));
    assert!(html.contains(r#"class="fw-wire-table fw-wire-size-md""#));
    assert!(!html.contains("fw-wire-table-header"));
    assert!(html.contains(r#"class="fw-wire-table-body""#));
}

#[test]
fn rows_are_clamped_to_max_table_rows() {
    let many_rows: Vec<&[&str]> = (0..25).map(|_| ["x"].as_slice()).collect();
    let html = render(&table(&[], &many_rows, Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-table-row"#).count(), 20);
}

#[test]
fn xss_regression_headers_and_cells_are_escaped() {
    let payload = "<script>alert(1)</script>";
    let rows: &[&[&str]] = &[&[payload]];
    let html = render(&table(&[payload], rows, Size::Md));
    assert!(!html.contains(payload));
    assert_eq!(html.matches("&lt;script&gt;").count(), 2);
}

#[test]
fn xss_regression_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let rows: &[&[&str]] = &[&[payload]];
    let html = render(&table(&[], rows, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();
    let html = render(&table(&HEADERS, &rows, Size::Md));
    for forbidden in [
        "<table",
        "<th",
        "<button",
        "<input",
        "<select",
        "<a ",
        "href=",
        "javascript:",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        " onclick=\"",
        "aria-expanded",
        "aria-haspopup",
        " aria-",
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn table_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::table::TABLE_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::table::TABLE_CSS));
}

#[test]
fn table_css_removes_border_top_from_first_body_row_when_header_is_absent() {
    // ヘッダーが空のとき `.fw-wire-table-body` がコンテナの先頭子要素になり、
    // その中の先頭行がコンテナ自身の外枠 border（`--fw-wire-line`）と
    // 隣接する。区切り線（`--fw-wire-line-subtle`）を残すと二重線に見える
    // ため、`.fw-wire-table-body:first-child .fw-wire-table-row:first-child`
    // で先頭行の `border-top` を外していることを固定する
    // （`crate::accordion::ACCORDION_CSS` の `:last-child` と同型の判断）。
    let css = fandhe_frontend_wireframe_ui::table::TABLE_CSS;
    assert!(
        css.contains(
            ".fw-wire-table-body:first-child .fw-wire-table-row:first-child {\n  border-top: none;\n}"
        ),
        "missing no-header first-row border-top removal rule: {css:?}"
    );
}

#[test]
fn table_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::table::TABLE_CSS;
    for selector in [
        ".fw-wire-table {",
        ".fw-wire-table-header {",
        ".fw-wire-table-body {",
        ".fw-wire-table-row {",
        ".fw-wire-table-cols-1 {",
        ".fw-wire-table-cols-12 {",
        ".fw-wire-table-cell {",
        ".fw-wire-table-cell-empty {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
    assert!(!css.contains('<'));
}
