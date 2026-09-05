//! Popover（イシュー #532）の統合テスト。
//!
//! `crates/headless-ui/src/popover.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > trigger + positioner >
//! content（+ title/description/close_trigger）」の組み立て全体の
//! data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・XSS 回帰をクレート
//! 外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::popover::{self, Popover};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_labelledby_describedby() {
    let trigger = popover::trigger(OpenState::Open, false, Some("popover-1"), vec![], vec![]);
    let title = popover::title(Some("title-1"), vec![], vec![]);
    let description = popover::description(Some("desc-1"), vec![], vec![]);
    let content = popover::content(
        OpenState::Open,
        Some("popover-1"),
        Some("title-1"),
        Some("desc-1"),
        vec![],
        vec![title, description],
    );
    let positioner = popover::positioner(OpenState::Open, vec![], vec![content]);
    let root = popover::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="popover""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="title""#));
    assert!(html.contains(r#"data-part="description""#));
    assert!(html.contains(r#"role="dialog""#));
    assert!(html.contains(r#"aria-controls="popover-1""#));
    assert!(html.contains(r#"id="popover-1""#));
    assert!(html.contains(r#"aria-labelledby="title-1""#));
    assert!(html.contains(r#"id="title-1""#));
    assert!(html.contains(r#"aria-describedby="desc-1""#));
    assert!(html.contains(r#"id="desc-1""#));
    assert!(!html.contains("hidden"));
    // content は tabindex="-1" を固定で持つ（イシュー #1642）。
    assert!(html.contains(r#"tabindex="-1""#));
}

#[test]
fn arrow_and_close_trigger_and_indicator_wire_into_assembly() {
    let arrow_tip = popover::arrow_tip(vec![], vec![]);
    let arrow = popover::arrow(vec![], vec![arrow_tip]);
    let close_trigger = popover::close_trigger(vec![("aria-label", "Close")], vec![]);
    let indicator = popover::indicator(OpenState::Open, vec![], vec![]);
    let content = popover::content(
        OpenState::Open,
        None,
        None,
        None,
        vec![],
        vec![close_trigger, indicator],
    );
    let positioner = popover::positioner(OpenState::Open, vec![], vec![arrow, content]);

    let html = render(&positioner);
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"data-part="close-trigger""#));
    assert!(html.contains(r#"aria-label="Close""#));
    assert!(html.contains(r#"data-part="indicator""#));
}

#[test]
fn anchor_is_independent_of_positioning_state() {
    let html = render(&popover::anchor(vec![], vec![]));
    assert!(html.contains(r#"data-scope="popover""#));
    assert!(html.contains(r#"data-part="anchor""#));
    assert!(!html.contains("data-state"));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut p = Popover::default();
    assert_eq!(p.state(), OpenState::Closed);
    assert!(render(&p.content(None, None, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut p, "toggle", ""));
    assert!(render(&p.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&p.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&p.positioner(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&p.content(None, None, None, vec![], vec![])).contains("hidden"));
    assert!(render(&p.indicator(vec![], vec![])).contains(r#"data-state="open""#));

    assert!(dispatch(&mut p, "open", ""));
    assert_eq!(p.state(), OpenState::Open);

    assert!(dispatch(&mut p, "close", ""));
    assert_eq!(p.state(), OpenState::Closed);

    assert!(!dispatch(&mut p, "no_such_action", ""));
    assert_eq!(p.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let p = Popover::default();
    let html = render(&p.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let p = Popover::new(OpenState::Open);
    let html = render(&render_for_hydration(&p));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = Popover::from_hydration_attrs(&p.hydration_attrs()).unwrap();
    assert_eq!(restored, p);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = Popover::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_id_labelledby_describedby_payloads_are_escaped_end_to_end() {
    let trigger = popover::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = popover::content(
        OpenState::Closed,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let html = render(&popover::root(
        OpenState::Closed,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&popover::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
