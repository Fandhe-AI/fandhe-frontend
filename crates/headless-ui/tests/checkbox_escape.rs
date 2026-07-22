//! Checkbox コンポーネントの XSS 回帰テスト（イシュー #535、親トラッキング #534
//! 「カテゴリ横断 XSS 回帰」の Checkbox 分を担う）。
//!
//! `crates/headless-ui/src/checkbox.rs` の各パーツは
//! [`fandhe_frontend_core::el`]（内部で [`crate::anatomy::Anatomy::part`] 経由）
//! への薄い委譲であり、値のエスケープは [`fandhe_frontend_core::render`]
//! （`escape_html_into`）に一元化されている。本ファイルはその契約が
//! `name`/`value`・呼び出し側 attrs のいずれの経路でも崩れていないことを、
//! 攻撃者が制御しうる値（引用符区切りを壊しイベントハンドラを注入しようと
//! するペイロード・`<script>` タグ）で固定する。
//! `.claude/rules/coding-rust.md` の「XSS 回帰テストは削除・弱体化しない」に
//! 従い、以後も維持する（`tests/helpers_escape.rs` の規約踏襲）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::checkbox::{hidden_input, root, CheckboxProps};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn hidden_input_name_value_are_escaped_on_render() {
    let props = CheckboxProps::default();
    let node = hidden_input(&props, ATTR_BREAK_PAYLOAD, ATTR_BREAK_PAYLOAD, vec![]);
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "name/value がエスケープされずイベントハンドラとして成立している: {html}"
    );
    assert!(
        html.contains("&quot;"),
        "属性値の \" が &quot; に置換されていない: {html}"
    );
}

#[test]
fn hidden_input_caller_attrs_are_escaped_on_render() {
    let props = CheckboxProps::default();
    let node = hidden_input(
        &props,
        "terms",
        "on",
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
    );
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "呼び出し側 attrs の値がエスケープされずイベントハンドラとして成立している: {html}"
    );
}

#[test]
fn root_children_text_is_escaped_on_render() {
    let props = CheckboxProps::default();
    let node = root(
        &props,
        vec![],
        vec![fandhe_frontend_core::text(SCRIPT_PAYLOAD)],
    );
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "子ノードのテキストが生の <script> のまま出力されている: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn reserved_attr_spoofing_cannot_override_type_checked_name_value() {
    // hidden_input の予約属性フィルタが、type/checked/name/value の偽装値を
    // 落としフレームワーク値を優先すること（フォーム送信値・チェック状態の
    // 偽装防止）を、攻撃ペイロードを混ぜた呼び出しで固定する。
    let props = CheckboxProps {
        checked: fandhe_frontend_headless_ui::checkbox::CheckedState::Checked,
        ..CheckboxProps::default()
    };
    let node = hidden_input(
        &props,
        "terms",
        "on",
        vec![
            ("type", "text"),
            ("checked", "false"),
            ("name", ATTR_BREAK_PAYLOAD),
            ("value", ATTR_BREAK_PAYLOAD),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"name="terms""#));
    assert!(html.contains(r#"value="on""#));
    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "予約属性フィルタを迂回してイベントハンドラが注入されている: {html}"
    );
}
