//! CheckboxGroup（`fandhe_frontend_headless_ui::checkbox_group`）の統合テスト
//! （イシュー #997）。
//!
//! フル anatomy の `render()` 出力固定（[`fandhe_frontend_headless_ui::checkbox::hidden_input`]
//! の入れ子再利用を含む）・data-*/ARIA 属性の検証・dispatch/hydration 統合
//! （[`fandhe_frontend_headless_ui::CheckboxGroup`]）・XSS 回帰
//! （`tests/helpers_escape.rs` と同型の攻撃ペイロード）・`Anatomy::part` の
//! fail-closed 挙動（呼び出し側 `data-scope`/`data-part` 偽装除去）が
//! CheckboxGroup パーツ経由でも維持されることを固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::checkbox::{hidden_input, CheckboxProps, CheckedState};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{checkbox_group, CheckboxGroup};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

fn checkbox_props(checked: bool) -> CheckboxProps {
    CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
    }
}

#[test]
fn full_anatomy_renders_expected_html_reusing_checkbox_hidden_input() {
    let node = checkbox_group::root(
        false,
        Some(Orientation::Vertical),
        Some("colors-label"),
        vec![],
        vec![
            checkbox_group::label(Some("colors-label"), vec![], vec![text("Colors")]),
            checkbox_group::item(
                true,
                false,
                "red",
                vec![],
                vec![
                    hidden_input(&checkbox_props(true), "colors", "red", vec![]),
                    checkbox_group::item_control(
                        true,
                        false,
                        vec![],
                        vec![checkbox_group::item_indicator(true, false, vec![], vec![])],
                    ),
                    checkbox_group::item_text(true, false, vec![], vec![text("Red")]),
                ],
            ),
            checkbox_group::item(
                false,
                false,
                "blue",
                vec![],
                vec![
                    hidden_input(&checkbox_props(false), "colors", "blue", vec![]),
                    checkbox_group::item_control(
                        false,
                        false,
                        vec![],
                        vec![checkbox_group::item_indicator(false, false, vec![], vec![])],
                    ),
                    checkbox_group::item_text(false, false, vec![], vec![text("Blue")]),
                ],
            ),
        ],
    );

    let html = render(&node);
    assert!(html.contains(r#"data-scope="checkbox-group" data-part="root""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-labelledby="colors-label""#));
    assert!(html.contains(r#"data-orientation="vertical""#));
    assert!(html.contains(r#"data-scope="checkbox-group" data-part="label" id="colors-label""#));

    // 選択済み item の一式
    assert!(html.contains(
        r#"data-scope="checkbox-group" data-part="item" data-state="checked" data-value="red""#
    ));
    // ネイティブ input は checkbox anatomy をそのまま再利用する（別 scope）
    assert!(html.contains(r#"data-scope="checkbox" data-part="hidden-input""#));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"name="colors""#));
    assert!(html.contains(r#"value="red""#));
    assert!(html.contains(r#"checked="""#));
    assert!(html
        .contains(r#"data-scope="checkbox-group" data-part="item-control" data-state="checked""#));
    assert!(html.contains(
        r#"data-scope="checkbox-group" data-part="item-indicator" data-state="checked""#
    ));
    assert!(html.contains(
        r#"data-scope="checkbox-group" data-part="item-text" data-state="checked">Red<"#
    ));

    // 未選択 item は indicator が hidden、hidden_input に checked がない
    assert!(html.contains(
        r#"data-scope="checkbox-group" data-part="item" data-state="unchecked" data-value="blue""#
    ));
    assert!(html.contains(
        r#"data-scope="checkbox-group" data-part="item-indicator" data-state="unchecked" hidden="""#
    ));
}

// --- Anatomy::part fail-closed 回帰 ---

#[test]
fn caller_attrs_cannot_override_anatomy_scope_and_part() {
    let html = render(&checkbox_group::item(
        true,
        false,
        "red",
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="checkbox-group""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(!html.contains("attacker"));
}

// --- CheckboxGroup 状態機械: dispatch 統合 ---

#[test]
fn checkbox_group_dispatch_select_deselect_toggle_all_supported() {
    let mut g = CheckboxGroup::default();
    assert_eq!(g.selected(), &[] as &[String]);

    assert!(dispatch(&mut g, "select", "red"));
    assert!(dispatch(&mut g, "select", "blue"));
    assert!(g.is_checked("red"));
    assert!(g.is_checked("blue"));

    assert!(dispatch(&mut g, "deselect", "red"));
    assert!(!g.is_checked("red"));
    assert!(g.is_checked("blue"));

    assert!(dispatch(&mut g, "toggle", "blue"));
    assert!(!g.is_checked("blue"));

    // radio_group とは異なり "select"/"deselect"/"toggle" の 3 語彙すべてを
    // 受理する（モジュール doc「セキュリティ不変条件」節参照）。
    assert!(!dispatch(&mut g, "no_such_action", "red"));
}

#[test]
fn checkbox_group_view_root_is_element_and_uses_group_role() {
    let node = CheckboxGroup::default().view();
    assert!(matches!(node, Node::Element { .. }));

    let rendered = render(&node);
    assert!(rendered.contains(r#"role="group""#));
    // MultiSelect::view の素の data-state="open"/"closed" ではないこと。
    assert!(!rendered.contains("data-state"));
}

#[test]
fn checkbox_group_hydration_round_trip() {
    let mut g = CheckboxGroup::default();
    dispatch(&mut g, "select", "red");
    dispatch(&mut g, "select", "blue");

    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("data-hydrate-selected="));

    let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn checkbox_group_from_hydration_attrs_missing_attr_does_not_panic() {
    let err = CheckboxGroup::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-selected".to_string())
    );
}

// --- XSS 回帰: value/id/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration にペイロードを渡してもエスケープされる ---

#[test]
fn root_labelled_by_payload_is_escaped_on_render() {
    let html = render(&checkbox_group::root(
        false,
        None,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn item_value_payload_is_escaped_on_render() {
    let html = render(&checkbox_group::item(
        false,
        false,
        ATTR_BREAK_PAYLOAD,
        vec![],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn children_text_is_escaped_on_render() {
    let html = render(&checkbox_group::item_text(
        false,
        false,
        vec![],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn dispatch_select_payload_is_escaped_on_render() {
    let mut g = CheckboxGroup::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut g, "select", payload));

    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
}

#[test]
fn xss_payload_in_hydration_selected_is_rejected_not_rendered() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&[
        "<script>alert(1)</script>".to_string(),
        "<script>alert(1)</script>".to_string(),
    ]);
    let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
    let err = CheckboxGroup::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}
