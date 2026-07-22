//! Menu（イシュー #540）の統合テスト。
//!
//! `crates/headless-ui/src/menu.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > trigger + positioner >
//! content > item_group > item_group_label + item ×2 + separator + item」の
//! 組み立て全体の data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する
//! （`tests/popover.rs` の構成に準拠）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::menu::{self, Menu};
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
