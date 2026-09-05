//! Collapsible（イシュー #529）の統合テスト。
//!
//! `crates/headless-ui/src/collapsible.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > trigger + content」の
//! 組み立て全体の data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::collapsible::{self, Collapsible};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_to_content_id() {
    let trigger = collapsible::trigger(OpenState::Open, false, Some("panel-1"), vec![], vec![]);
    let content = collapsible::content(OpenState::Open, false, Some("panel-1"), vec![], vec![]);
    let root = collapsible::root(OpenState::Open, false, vec![], vec![trigger, content]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="collapsible""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"aria-controls="panel-1""#));
    assert!(html.contains(r#"id="panel-1""#));
    assert!(!html.contains("hidden"));
}

/// root/trigger/indicator/content すべてに `disabled=true` を渡すと、4
/// パートすべてに `data-disabled` が出力される（イシュー #1637。ark-ui/Radix
/// の Data Attributes 表準拠）。
#[test]
fn full_assembly_disabled_true_adds_data_disabled_to_all_four_parts() {
    let trigger = collapsible::trigger(OpenState::Closed, true, Some("panel-1"), vec![], vec![]);
    let indicator = collapsible::indicator(OpenState::Closed, true, vec![], vec![]);
    let content = collapsible::content(OpenState::Closed, true, Some("panel-1"), vec![], vec![]);
    let root = collapsible::root(
        OpenState::Closed,
        true,
        vec![],
        vec![trigger, indicator, content],
    );

    let html = render(&root);
    assert_eq!(html.matches(r#"data-disabled="""#).count(), 4);
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut c = Collapsible::default();
    assert_eq!(c.state(), OpenState::Closed);
    assert!(render(&c.content(false, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut c, "toggle", ""));
    assert!(render(&c.root(false, vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&c.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    assert!(!render(&c.content(false, None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut c, "open", ""));
    assert_eq!(c.state(), OpenState::Open);

    assert!(dispatch(&mut c, "close", ""));
    assert_eq!(c.state(), OpenState::Closed);

    assert!(!dispatch(&mut c, "no_such_action", ""));
    assert_eq!(c.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let c = Collapsible::default();
    let html = render(&c.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let c = Collapsible::new(OpenState::Open);
    let html = render(&render_for_hydration(&c));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = Collapsible::from_hydration_attrs(&c.hydration_attrs()).unwrap();
    assert_eq!(restored, c);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = Collapsible::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_and_id_payloads_are_escaped_end_to_end() {
    let trigger = collapsible::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = collapsible::content(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let html = render(&collapsible::root(
        OpenState::Closed,
        false,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&collapsible::root(
        OpenState::Closed,
        false,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

/// 呼び出し側 `attrs` による `data-state`/`data-disabled` なりすましは
/// root > trigger + content の end-to-end 組み立てでも除外される
/// （イシュー #1637、`collapsible.rs` inline tests の end-to-end 版）。
#[test]
fn caller_attrs_spoofing_is_dropped_end_to_end() {
    let trigger = collapsible::trigger(
        OpenState::Open,
        false,
        Some("panel-1"),
        vec![("data-state", "closed"), ("data-disabled", "spoofed")],
        vec![],
    );
    let content = collapsible::content(
        OpenState::Open,
        false,
        Some("panel-1"),
        vec![("data-state", "closed"), ("data-disabled", "spoofed")],
        vec![],
    );
    let html = render(&collapsible::root(
        OpenState::Open,
        false,
        vec![("data-state", "closed"), ("data-disabled", "spoofed")],
        vec![trigger, content],
    ));

    assert_eq!(html.matches(r#"data-state="open""#).count(), 3);
    assert!(!html.contains("spoofed"));
    assert!(!html.contains("data-disabled"));
}
