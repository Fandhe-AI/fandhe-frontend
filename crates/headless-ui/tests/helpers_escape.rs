//! anatomy / data-* / ARIA ヘルパの XSS 回帰テスト（イシュー #523）。
//!
//! `data_attrs` / `aria` の各ヘルパは属性名を `&'static str` リテラルに固定し
//! 値のみを動的に受け取る薄い委譲層であり、値のエスケープは
//! `fandhe_frontend_core::render`（`escape_html_into`）に一元化されている。
//! 本ファイルはその契約が崩れていないことを、攻撃者が制御しうる属性値
//! （引用符区切りを壊しイベントハンドラを注入しようとするペイロード）を
//! 用いて固定する。`.claude/rules/coding-rust.md` の
//! 「XSS 回帰テストは削除・弱体化しない」に従い、以後も維持する。

use fandhe_frontend_core::{el, render};
use fandhe_frontend_headless_ui::{anatomy, aria_controls, aria_label, data_state};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn data_state_value_is_escaped_on_render() {
    let node = el("div", vec![data_state(ATTR_BREAK_PAYLOAD)], vec![]);
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "data_state の値がエスケープされずイベントハンドラとして成立している: {html}"
    );
    assert!(
        html.contains("&quot;"),
        "属性値の \" が &quot; に置換されていない: {html}"
    );
}

#[test]
fn aria_label_value_is_escaped_on_render() {
    let payload = "<script>alert(1)</script>";
    let node = el("button", vec![aria_label(payload)], vec![]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "aria_label の値が生の <script> のまま出力されている: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn aria_controls_value_is_escaped_on_render() {
    let node = el("button", vec![aria_controls(ATTR_BREAK_PAYLOAD)], vec![]);
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "aria_controls の値がエスケープされずイベントハンドラとして成立している: {html}"
    );
}

#[test]
fn anatomy_part_escapes_caller_supplied_attr_values_too() {
    // Anatomy::part は data-scope/data-part を付与するだけの薄い委譲であり、
    // 呼び出し側 attrs（ここでは onclick を模した値）も el() の既定エスケープを
    // 素通りせず経由することを固定する。
    let a = anatomy("dialog");
    let node = a.part(
        "content",
        "div",
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    );
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "Anatomy::part 経由の値がエスケープされていない: {html}"
    );
    assert!(html.contains(r#"data-scope="dialog""#));
    assert!(html.contains(r#"data-part="content""#));
}
