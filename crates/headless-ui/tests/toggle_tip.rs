//! ToggleTip（イシュー #761）の統合テスト。
//!
//! `crates/headless-ui/src/toggle_tip.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! 「root > trigger + positioner > (content + arrow)」の組み立て全体の
//! data-*/ARIA 対応（controls↔id）・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::toggle_tip::{self, ToggleTip};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_to_content_id() {
    let trigger = toggle_tip::trigger(OpenState::Open, false, Some("tip-1"), vec![], vec![]);
    let arrow = toggle_tip::arrow(vec![], vec![toggle_tip::arrow_tip(vec![], vec![])]);
    let content = toggle_tip::content(OpenState::Open, Some("tip-1"), vec![], vec![arrow]);
    let positioner = toggle_tip::positioner(OpenState::Open, vec![], vec![content]);
    let root = toggle_tip::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="toggle-tip""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"aria-expanded="true""#));
    assert!(html.contains(r#"aria-controls="tip-1""#));
    assert!(html.contains(r#"id="tip-1""#));
    // ToggleTip の 3 者境界（モジュール doc）: Popover と異なり
    // aria-haspopup は付与せず、Tooltip と異なり role="tooltip" も
    // aria-describedby も使わない。
    assert!(!html.contains("aria-haspopup"));
    assert!(!html.contains("role=\"tooltip\""));
    assert!(!html.contains("aria-describedby"));
    // open 状態のため content に hidden 存在属性は付かない。arrow の
    // aria-hidden="true"（装飾目的、substring として "hidden" を含む）とは
    // 区別するため、hidden 存在属性の具体的な出現形で判定する。
    assert!(!html.contains(r#" hidden="""#));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut t = ToggleTip::default();
    assert_eq!(t.state(), OpenState::Closed);
    assert!(render(&t.content(None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut t, "toggle", ""));
    assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&t.content(None, vec![], vec![])).contains("hidden"));
    assert!(render(&t.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));

    assert!(dispatch(&mut t, "open", ""));
    assert_eq!(t.state(), OpenState::Open);

    assert!(dispatch(&mut t, "close", ""));
    assert_eq!(t.state(), OpenState::Closed);

    assert!(!dispatch(&mut t, "no_such_action", ""));
    assert_eq!(t.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let t = ToggleTip::default();
    let html = render(&t.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let t = ToggleTip::new(OpenState::Open);
    let html = render(&render_for_hydration(&t));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = ToggleTip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored, t);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = ToggleTip::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_and_id_payloads_are_escaped_end_to_end() {
    let trigger = toggle_tip::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = toggle_tip::content(OpenState::Closed, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&toggle_tip::root(
        OpenState::Closed,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&toggle_tip::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
