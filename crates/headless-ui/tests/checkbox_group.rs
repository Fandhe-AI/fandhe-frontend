//! CheckboxGroup（`fandhe_frontend_headless_ui::checkbox_group`）の統合テスト
//! （イシュー #997、参照突合はイシュー #1603）。
//!
//! フル anatomy の `render()` 出力固定（[`fandhe_frontend_headless_ui::checkbox::hidden_input`]
//! の入れ子再利用を含む）・data-*/ARIA 属性の検証・dispatch/hydration 統合
//! （[`fandhe_frontend_headless_ui::CheckboxGroup`]）・XSS 回帰
//! （`tests/helpers_escape.rs` と同型の攻撃ペイロード）・`Anatomy::part` の
//! fail-closed 挙動（呼び出し側 `data-scope`/`data-part` 偽装除去）が
//! CheckboxGroup パーツ経由でも維持されることを固定する。
//!
//! イシュー #1603 で `checkbox_group::root`/`item`/`item_control`/
//! `item_indicator`/`item_text` の `disabled: bool` 引数が
//! `&CheckboxGroupProps` へ署名変更されたため、本ファイルの既存呼び出しは
//! すべて追随済み。加えて ark-ui `Checkbox.Group` / Radix Themes
//! `CheckboxGroup` との参照突合契約（anatomy パート網羅・`data-state` 語彙・
//! props フラグの一律出力・pointer/focus 系 `data-*` の非出力・
//! `aria-orientation` 非付与・item_hidden_input への invalid/readonly 伝播）
//! を追加する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::checkbox::{hidden_input, CheckboxProps, CheckedState};
use fandhe_frontend_headless_ui::checkbox_group::CheckboxGroupProps;
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
    let props = CheckboxGroupProps::default();
    let node = checkbox_group::root(
        &props,
        Some(Orientation::Vertical),
        Some("colors-label"),
        vec![],
        vec![
            checkbox_group::label(Some("colors-label"), vec![], vec![text("Colors")]),
            checkbox_group::item(
                true,
                &props,
                "red",
                vec![],
                vec![
                    hidden_input(&checkbox_props(true), "colors", "red", vec![]),
                    checkbox_group::item_control(
                        true,
                        &props,
                        vec![],
                        vec![checkbox_group::item_indicator(true, &props, vec![], vec![])],
                    ),
                    checkbox_group::item_text(true, &props, vec![], vec![text("Red")]),
                ],
            ),
            checkbox_group::item(
                false,
                &props,
                "blue",
                vec![],
                vec![
                    hidden_input(&checkbox_props(false), "colors", "blue", vec![]),
                    checkbox_group::item_control(
                        false,
                        &props,
                        vec![],
                        vec![checkbox_group::item_indicator(
                            false,
                            &props,
                            vec![],
                            vec![],
                        )],
                    ),
                    checkbox_group::item_text(false, &props, vec![], vec![text("Blue")]),
                ],
            ),
        ],
    );

    let html = render(&node);
    assert!(html.contains(r#"data-scope="checkbox-group" data-part="root""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-labelledby="colors-label""#));
    assert!(html.contains(r#"data-orientation="vertical""#));
    // イシュー #1603 D2: role="group" は WAI-ARIA 1.2 の aria-orientation
    // Used in Roles に含まれないため、data-orientation はあっても
    // aria-orientation は出ない。
    assert!(!html.contains("aria-orientation"));
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
        &CheckboxGroupProps::default(),
        "red",
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="checkbox-group""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(!html.contains("attacker"));
}

// --- イシュー #1603: 参考サイト（ark-ui Checkbox.Group / Radix Themes CheckboxGroup）突合契約 ---

#[test]
fn reference_anatomy_part_names_match_reference() {
    // 6 パーツ全てが data-scope="checkbox-group" と期待 data-part を持つ。
    let props = CheckboxGroupProps::default();
    let cases: Vec<(&str, String)> = vec![
        (
            "root",
            render(&checkbox_group::root(&props, None, None, vec![], vec![])),
        ),
        (
            "label",
            render(&checkbox_group::label(None, vec![], vec![])),
        ),
        (
            "item",
            render(&checkbox_group::item(false, &props, "red", vec![], vec![])),
        ),
        (
            "item-control",
            render(&checkbox_group::item_control(false, &props, vec![], vec![])),
        ),
        (
            "item-indicator",
            render(&checkbox_group::item_indicator(
                false,
                &props,
                vec![],
                vec![],
            )),
        ),
        (
            "item-text",
            render(&checkbox_group::item_text(false, &props, vec![], vec![])),
        ),
    ];
    for (part, html) in cases {
        assert!(
            html.contains(r#"data-scope="checkbox-group""#),
            "part={part}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "part={part}"
        );
    }
}

#[test]
fn data_state_vocabulary_is_two_valued_on_every_item_part() {
    let props = CheckboxGroupProps::default();
    for html in [
        render(&checkbox_group::item(true, &props, "red", vec![], vec![])),
        render(&checkbox_group::item_control(true, &props, vec![], vec![])),
        render(&checkbox_group::item_indicator(
            true,
            &props,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_text(true, &props, vec![], vec![])),
    ] {
        assert!(html.contains(r#"data-state="checked""#));
        assert!(!html.contains("\"open\""));
        assert!(!html.contains("\"closed\""));
    }
}

#[test]
fn props_flags_are_data_attrs_on_root_and_every_item_part() {
    let all_true = CheckboxGroupProps {
        disabled: true,
        readonly: true,
        invalid: true,
    };
    let all_false = CheckboxGroupProps::default();

    for html in [
        render(&checkbox_group::root(&all_true, None, None, vec![], vec![])),
        render(&checkbox_group::item(
            true,
            &all_true,
            "red",
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_control(
            true,
            &all_true,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_indicator(
            true,
            &all_true,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_text(true, &all_true, vec![], vec![])),
    ] {
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    for html in [
        render(&checkbox_group::root(
            &all_false,
            None,
            None,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item(
            true,
            &all_false,
            "red",
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_control(
            true,
            &all_false,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_indicator(
            true,
            &all_false,
            vec![],
            vec![],
        )),
        render(&checkbox_group::item_text(true, &all_false, vec![], vec![])),
    ] {
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
        assert!(!html.contains("data-invalid"));
    }
}

#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    // ark-ui の data-hover/data-active/data-focus は DOM ローカルな
    // pointer/focus 状態であり SSR 静的出力の関心外
    // （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
    let props = CheckboxGroupProps {
        disabled: true,
        readonly: true,
        invalid: true,
    };
    let html = render(&checkbox_group::root(
        &props,
        Some(Orientation::Horizontal),
        Some("group-label"),
        vec![],
        vec![
            checkbox_group::label(Some("group-label"), vec![], vec![text("Colors")]),
            checkbox_group::item(
                true,
                &props,
                "red",
                vec![],
                vec![
                    checkbox_group::item_control(
                        true,
                        &props,
                        vec![],
                        vec![checkbox_group::item_indicator(true, &props, vec![], vec![])],
                    ),
                    checkbox_group::item_text(true, &props, vec![], vec![text("Red")]),
                ],
            ),
        ],
    ));
    assert!(!html.contains("data-hover"));
    assert!(!html.contains("data-active"));
    assert!(!html.contains("data-focus="));
    assert!(!html.contains("data-motion"));
}

#[test]
fn root_has_group_role_without_aria_orientation() {
    let html = render(&checkbox_group::root(
        &CheckboxGroupProps::default(),
        Some(Orientation::Horizontal),
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"data-orientation="horizontal""#));
    assert!(!html.contains("aria-orientation"));
}

#[test]
fn item_control_has_no_checkbox_role_or_aria_checked() {
    let html = render(&checkbox_group::item_control(
        true,
        &CheckboxGroupProps::default(),
        vec![],
        vec![],
    ));
    assert!(!html.contains(r#"role="checkbox""#));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn item_indicator_hidden_only_when_unchecked() {
    let props = CheckboxGroupProps::default();
    assert!(render(&checkbox_group::item_indicator(
        false,
        &props,
        vec![],
        vec![]
    ))
    .contains(r#"hidden="""#));
    assert!(!render(&checkbox_group::item_indicator(
        true,
        &props,
        vec![],
        vec![]
    ))
    .contains(r#"hidden="""#));
}

#[test]
fn item_hidden_input_propagates_group_invalid_and_readonly() {
    // イシュー #1603 D3: CheckboxGroup::item_hidden_input は root の
    // invalid/readonly をネイティブ <input> へ OR 伝播する。
    let g = CheckboxGroup::default().with_props(CheckboxGroupProps {
        disabled: false,
        readonly: true,
        invalid: true,
    });
    let html = render(&g.item_hidden_input("red", checkbox_props(false), "colors", vec![]));
    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(html.contains("data-invalid"));
    assert!(html.contains("data-readonly"));
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
fn checkbox_group_hydration_round_trip_preserves_invalid_and_readonly() {
    // イシュー #1603: disabled と同型の hydration 往復保証を invalid/
    // readonly も持つ。
    let g = CheckboxGroup::default().with_props(CheckboxGroupProps {
        disabled: false,
        readonly: true,
        invalid: true,
    });
    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains(r#"data-hydrate-invalid="true""#));
    assert!(rendered.contains(r#"data-hydrate-readonly="true""#));

    let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn checkbox_group_hydration_missing_invalid_readonly_defaults_to_false() {
    use fandhe_frontend_interactive::codec;
    let attrs = vec![(
        "data-hydrate-selected".to_string(),
        codec::encode_list(&Vec::<String>::new()),
    )];
    let restored = CheckboxGroup::from_hydration_attrs(&attrs).unwrap();
    assert!(!restored.props().invalid);
    assert!(!restored.props().readonly);
}

#[test]
fn checkbox_group_from_hydration_attrs_invalid_disabled_value_rejected_fail_closed() {
    use fandhe_frontend_interactive::codec;
    let attrs = vec![
        (
            "data-hydrate-selected".to_string(),
            codec::encode_list(&Vec::<String>::new()),
        ),
        ("data-hydrate-disabled".to_string(), "yes".to_string()),
    ];
    let err = CheckboxGroup::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
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
        &CheckboxGroupProps::default(),
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
        &CheckboxGroupProps::default(),
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
        &CheckboxGroupProps::default(),
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
