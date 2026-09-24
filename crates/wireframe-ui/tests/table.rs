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
fn headerless_table_body_first_row_removes_double_top_border() {
    // Cursor Bugbot（Low）/ codex-review P2 の指摘: headers が空だと
    // `.fw-wire-table-row` の上罫線がルート外枠と二重になっていた。
    // `.fw-wire-table-body:first-child .fw-wire-table-row:first-child`
    // で headerless 時のみ body の先頭行の上罫線を除去し、ヘッダーありの
    // 区切り線（`.fw-wire-table-header .fw-wire-table-row` の解除規則）は
    // 維持されていることを CSS 定義で固定する。
    let css = fandhe_frontend_wireframe_ui::table::TABLE_CSS;
    assert!(css.contains(
        ".fw-wire-table-body:first-child .fw-wire-table-row:first-child {\n  border-top: none;\n}"
    ));
    // ヘッダーありの区切り線を消す既存規則は維持する。
    assert!(css.contains(".fw-wire-table-header .fw-wire-table-row {\n  border-top: none;\n}"));
    // 通常行の上罫線規則自体は消さない（ヘッダーありテーブルの 2 行目
    // 以降の区切り線として使われ続ける）。
    assert!(css.contains(
        ".fw-wire-table-row {\n  display: grid;\n  border-top: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);\n}"
    ));
}

#[test]
fn headerless_and_headered_table_body_structure_differs() {
    // headers が空のときは body div がルートの最初の（かつ唯一の）子に
    // なり、ヘッダーありのときは header div の後に続く 2 番目の子になる
    // （上記 CSS の `:first-child` セレクタが対象を正しく捉えるための
    // 構造契約）。
    let rows: Vec<&[&str]> = ROWS.iter().map(|row| row.as_slice()).collect();

    let headerless_html = render(&table(&[], &rows, Size::Md));
    let root_open_end = headerless_html.find('>').expect("root open tag end");
    // header div を挟まず、ルートタグ直後が body div であること
    // （`.fw-wire-table-body:first-child` が捉える対象と一致する）。
    assert!(headerless_html[root_open_end + 1..].starts_with(r#"<div class="fw-wire-table-body""#));
    assert!(!headerless_html.contains("fw-wire-table-header"));

    let headered_html = render(&table(&HEADERS, &rows, Size::Md));
    let header_pos = headered_html
        .find(r#"class="fw-wire-table-header""#)
        .expect("header class present");
    let headered_body_pos = headered_html
        .find(r#"class="fw-wire-table-body""#)
        .expect("body class present");
    assert!(
        header_pos < headered_body_pos,
        "header div should precede body div"
    );
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
