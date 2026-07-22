//! Menu（イシュー #540）の統合テスト。
//!
//! `crates/headless-ui/src/menu.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > trigger + positioner >
//! content > item_group > item_group_label + item ×2 + separator + item」の
//! 組み立て全体の data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する
//! （`tests/popover.rs` の構成に準拠）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::menu::{self, Menu, MenuCheckboxItem, MenuRadioItemGroup};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_labelledby_and_item_group() {
    let group_label = menu::item_group_label(Some("group-1-label"), vec![], vec![]);
    let item_a = menu::item("a", false, false, vec![], vec![]);
    let item_b = menu::item("b", false, true, vec![], vec![]);
    let group = menu::item_group(
        Some("group-1-label"),
        vec![],
        vec![group_label, item_a, item_b],
    );
    let separator = menu::separator(vec![], vec![]);
    let item_c = menu::item("c", true, false, vec![], vec![]);
    let content = menu::content(
        OpenState::Open,
        Some("menu-1"),
        Some("trigger-1"),
        vec![],
        vec![group, separator, item_c],
    );
    let positioner = menu::positioner(OpenState::Open, vec![], vec![content]);
    let trigger = menu::trigger(
        OpenState::Open,
        false,
        Some("menu-1"),
        vec![("id", "trigger-1")],
        vec![],
    );
    let root = menu::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="menu""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="item-group""#));
    assert!(html.contains(r#"data-part="item-group-label""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(html.contains(r#"data-part="separator""#));
    assert!(html.contains(r#"role="menu""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"role="menuitem""#));
    assert!(html.contains(r#"role="separator""#));
    // trigger <-> content の aria-controls/id 配線。
    assert!(html.contains(r#"aria-controls="menu-1""#));
    assert!(html.contains(r#"id="menu-1""#));
    // content の aria-labelledby <-> trigger の id 配線。
    assert!(html.contains(r#"aria-labelledby="trigger-1""#));
    assert!(html.contains(r#"id="trigger-1""#));
    // item_group の aria-labelledby <-> item_group_label の id 配線。
    assert!(html.contains(r#"aria-labelledby="group-1-label""#));
    assert!(html.contains(r#"id="group-1-label""#));
    // item の data-value / disabled / highlighted。
    assert!(html.contains(r#"data-value="a""#));
    assert!(html.contains(r#"data-value="b""#));
    assert!(html.contains(r#"data-value="c""#));
    assert!(html.contains(r#"data-highlighted="""#));
    assert!(html.contains(r#"aria-disabled="true""#));
}

#[test]
fn arrow_and_indicator_wire_into_assembly() {
    let arrow_tip = menu::arrow_tip(vec![], vec![]);
    let arrow = menu::arrow(vec![], vec![arrow_tip]);
    let indicator = menu::indicator(OpenState::Open, vec![], vec![]);
    let content = menu::content(OpenState::Open, None, None, vec![], vec![indicator]);
    let positioner = menu::positioner(OpenState::Open, vec![], vec![arrow, content]);

    let html = render(&positioner);
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"data-part="indicator""#));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut m = Menu::default();
    assert_eq!(m.state(), OpenState::Closed);
    assert!(render(&m.content(None, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut m, "toggle", ""));
    assert!(render(&m.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&m.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&m.positioner(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&m.content(None, None, vec![], vec![])).contains("hidden"));
    assert!(render(&m.indicator(vec![], vec![])).contains(r#"data-state="open""#));

    assert!(dispatch(&mut m, "open", ""));
    assert_eq!(m.state(), OpenState::Open);

    assert!(dispatch(&mut m, "close", ""));
    assert_eq!(m.state(), OpenState::Closed);

    assert!(!dispatch(&mut m, "no_such_action", ""));
    assert_eq!(m.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let m = Menu::default();
    let html = render(&m.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let m = Menu::new(OpenState::Open);
    let html = render(&render_for_hydration(&m));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = Menu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
    assert_eq!(restored, m);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = Menu::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_id_labelledby_value_payloads_are_escaped_end_to_end() {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![menu::item(ATTR_BREAK_PAYLOAD, false, false, vec![], vec![])],
    );
    let html = render(&menu::root(
        OpenState::Closed,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&menu::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

// --- TriggerItem/ContextTrigger（サブメニュー・右クリック、イシュー #598） ---

/// 親 Menu（open）の content 内に子 Menu（closed）由来の trigger_item +
/// positioner + content を入れ子で組み立て、haspopup 連鎖・role="menu" の
/// 二重出現・aria-controls/id 配線・親 open/子 closed の hidden 差異を固定する。
#[test]
fn nested_menu_assembly_wires_haspopup_chain_and_hidden_state() {
    let parent = Menu::new(OpenState::Open);
    let sub = Menu::new(OpenState::Closed);

    let sub_content = sub.content(
        Some("sub-1"),
        None,
        vec![],
        vec![menu::item("x", false, false, vec![], vec![])],
    );
    let sub_positioner = sub.positioner(vec![], vec![sub_content]);

    let trigger_item = sub.trigger_item(false, false, Some("sub-1"), vec![], vec![]);
    let regular_item = menu::item("a", false, false, vec![], vec![]);

    let parent_content = parent.content(
        None,
        None,
        vec![],
        vec![regular_item, trigger_item, sub_positioner],
    );
    let parent_trigger = parent.trigger(false, None, vec![], vec![]);
    let parent_positioner = parent.positioner(vec![], vec![parent_content]);
    let root = parent.root(vec![], vec![parent_trigger, parent_positioner]);

    let html = render(&root);

    // (a) aria-haspopup="menu" が親 trigger と子 trigger-item の 2 箇所に出る。
    assert_eq!(html.matches(r#"aria-haspopup="menu""#).count(), 2);
    // (b) role="menu" が親 content と子 content の 2 箇所に出る。
    assert_eq!(html.matches(r#"role="menu""#).count(), 2);
    // (c) aria-controls="sub-1" <-> id="sub-1" の配線。
    assert!(html.contains(r#"aria-controls="sub-1""#));
    assert!(html.contains(r#"id="sub-1""#));
    // (d) 親 open なので親 content には hidden が付かず、子 closed なので
    //     子 positioner には hidden が付く。
    let sub_positioner_html = render(&sub.positioner(vec![], vec![]));
    assert!(sub_positioner_html.contains(r#"hidden="""#));
    let parent_content_only = render(&parent.content(None, None, vec![], vec![]));
    assert!(!parent_content_only.contains("hidden"));
}

#[test]
fn nested_menu_dispatch_is_independent_between_parent_and_child() {
    let parent = Menu::new(OpenState::Open);
    let mut sub = Menu::default();
    assert_eq!(sub.state(), OpenState::Closed);

    assert!(dispatch(&mut sub, "open", ""));

    // 子だけ open になり、親は不変（open のまま）。
    assert!(
        render(&sub.trigger_item(false, false, None, vec![], vec![]))
            .contains(r#"aria-expanded="true""#)
    );
    assert_eq!(parent.state(), OpenState::Open);
}

#[test]
fn context_trigger_assembly_outputs_data_hooks_without_aria() {
    let m = Menu::new(OpenState::Closed);
    let content = m.content(
        None,
        None,
        vec![],
        vec![menu::item("a", false, false, vec![], vec![])],
    );
    let positioner = m.positioner(vec![], vec![content]);
    let ctx_trigger = m.context_trigger(vec![], vec![]);
    let root = m.root(vec![], vec![ctx_trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-part="context-trigger""#));
    assert!(html.contains(r#"<button"#));
    assert!(html.contains(r#"type="button""#));
    assert!(!html.contains("aria-haspopup"));
    assert!(!html.contains("aria-expanded"));
}

#[test]
fn nested_menu_assembly_xss_payload_is_escaped_end_to_end() {
    let sub = Menu::new(OpenState::Closed);
    let sub_content = sub.content(Some(ATTR_BREAK_PAYLOAD), None, vec![], vec![]);
    let sub_positioner = sub.positioner(vec![], vec![sub_content]);
    let trigger_item = sub.trigger_item(
        false,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    );

    let parent = Menu::new(OpenState::Open);
    let parent_content = parent.content(None, None, vec![], vec![trigger_item, sub_positioner]);
    let parent_trigger = parent.trigger(false, None, vec![], vec![]);
    let root = parent.root(vec![], vec![parent_trigger, parent_content]);

    let html = render(&root);
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

// --- checkbox_item / radio_item_group / radio_item 組み立て統合（イシュー #597） ---

#[test]
fn full_assembly_with_checkbox_item_and_radio_item_group_wires_roles_and_checked_state() {
    let checkbox = menu::checkbox_item(true, "notifications", false, false, vec![], vec![]);

    let group_label = menu::item_group_label(Some("radio-group-1-label"), vec![], vec![]);
    let radio_a = menu::radio_item(true, "a", false, false, vec![], vec![]);
    let radio_b = menu::radio_item(false, "b", false, false, vec![], vec![]);
    let radio_group = menu::radio_item_group(
        Some("radio-group-1-label"),
        vec![],
        vec![group_label, radio_a, radio_b],
    );

    let content = menu::content(
        OpenState::Open,
        None,
        None,
        vec![],
        vec![checkbox, radio_group],
    );
    let positioner = menu::positioner(OpenState::Open, vec![], vec![content]);
    let trigger = menu::trigger(OpenState::Open, false, None, vec![], vec![]);
    let root = menu::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-part="checkbox-item""#));
    assert!(html.contains(r#"role="menuitemcheckbox""#));
    assert!(html.contains(r#"data-part="radio-item-group""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"data-part="radio-item""#));
    assert!(html.contains(r#"role="menuitemradio""#));
    assert!(html.contains(r#"aria-labelledby="radio-group-1-label""#));
    assert!(html.contains(r#"id="radio-group-1-label""#));
    // checkbox は checked=true、radio a は checked、radio b は unchecked が
    // 同時に成立する（互いの状態機械が独立していることの固定）。
    assert!(html.matches(r#"aria-checked="true""#).count() == 2);
    assert!(html.contains(r#"aria-checked="false""#));
}

#[test]
fn menu_checkbox_item_toggle_dispatch_reflects_in_rendering() {
    let mut c = MenuCheckboxItem::default();
    assert!(
        render(&c.checkbox_item("notifications", false, false, vec![], vec![]))
            .contains(r#"data-state="unchecked""#)
    );

    assert!(dispatch(&mut c, "toggle", ""));
    let html = render(&c.checkbox_item("notifications", false, false, vec![], vec![]));
    assert!(html.contains(r#"data-state="checked""#));
    assert!(html.contains(r#"aria-checked="true""#));
}

#[test]
fn menu_radio_item_group_select_dispatch_reflects_in_rendering() {
    let mut g = MenuRadioItemGroup::default();
    assert!(dispatch(&mut g, "select", "a"));

    let radio_a = g.radio_item("a", false, false, vec![], vec![]);
    let radio_b = g.radio_item("b", false, false, vec![], vec![]);
    assert!(render(&radio_a).contains(r#"data-state="checked""#));
    assert!(render(&radio_b).contains(r#"data-state="unchecked""#));

    assert!(dispatch(&mut g, "select", "b"));
    let radio_a = g.radio_item("a", false, false, vec![], vec![]);
    let radio_b = g.radio_item("b", false, false, vec![], vec![]);
    assert!(render(&radio_a).contains(r#"data-state="unchecked""#));
    assert!(render(&radio_b).contains(r#"data-state="checked""#));
}

#[test]
fn menu_checkbox_item_hydration_round_trip_via_public_api() {
    let c = MenuCheckboxItem::new(true);
    let rendered = render(&render_for_hydration(&c));
    assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

    let restored = MenuCheckboxItem::from_hydration_attrs(&c.hydration_attrs()).unwrap();
    assert_eq!(restored, c);
}

#[test]
fn menu_checkbox_item_hydration_rejects_tampered_value() {
    let attrs = vec![(
        "data-hydrate-checked".to_string(),
        "<script>alert(1)</script>".to_string(),
    )];
    let err = MenuCheckboxItem::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn menu_radio_item_group_hydration_round_trip_via_public_api() {
    let mut g = MenuRadioItemGroup::default();
    assert!(dispatch(&mut g, "select", "a"));
    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("data-hydrate-selected="));

    let restored = MenuRadioItemGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn menu_radio_item_group_hydration_rejects_missing_attr() {
    let err = MenuRadioItemGroup::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-selected".to_string())
    );
}

#[test]
fn checkbox_item_and_radio_item_group_assembly_xss_payload_is_escaped_end_to_end() {
    let checkbox = menu::checkbox_item(
        false,
        ATTR_BREAK_PAYLOAD,
        false,
        false,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    );
    let radio_group = menu::radio_item_group(
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![menu::radio_item(
            false,
            ATTR_BREAK_PAYLOAD,
            false,
            false,
            vec![],
            vec![],
        )],
    );
    let content = menu::content(
        OpenState::Open,
        None,
        None,
        vec![],
        vec![checkbox, radio_group],
    );
    let root = menu::root(OpenState::Open, vec![], vec![content]);

    let html = render(&root);
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}
