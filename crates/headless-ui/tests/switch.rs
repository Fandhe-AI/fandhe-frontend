//! Switch（イシュー #537）の統合テスト。
//!
//! `crates/headless-ui/src/switch.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > control(thumb) + label +
//! hidden_input」の組み立て全体の data-*/ARIA 対応・dispatch 統合・
//! SSR/hydration 両経路・XSS 回帰をクレート外部から（公開 API のみを使って）
//! 固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::switch::{self, Switch};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_control_thumb_label_and_hidden_input() {
    let control = switch::control(
        true,
        false,
        vec![],
        vec![switch::thumb(true, vec![], vec![])],
    );
    let label = switch::label(true, vec![], vec![fandhe_frontend_core::text("Wi-Fi")]);
    let hidden_input = switch::hidden_input("wifi", "on", true, false, false, vec![]);
    let root = switch::root(true, false, vec![], vec![control, label, hidden_input]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="switch""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"data-part="thumb""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="hidden-input""#));
    assert!(html.contains(r#"data-state="checked""#));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"role="switch""#));
    assert!(html.contains(r#"name="wifi""#));
    assert!(html.contains("Wi-Fi"));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut s = Switch::default();
    assert!(!s.is_checked());
    assert!(!render(&s.hidden_input("wifi", "on", false, false, vec![])).contains(r#"checked="""#));

    assert!(dispatch(&mut s, "toggle", ""));
    assert!(s.is_checked());
    assert!(render(&s.root(false, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.control(false, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.thumb(vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.label(vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.hidden_input("wifi", "on", false, false, vec![])).contains(r#"checked="""#));

    assert!(dispatch(&mut s, "uncheck", ""));
    assert!(!s.is_checked());

    assert!(dispatch(&mut s, "check", ""));
    assert!(s.is_checked());

    assert!(!dispatch(&mut s, "no_such_action", ""));
    assert!(s.is_checked());
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let s = Switch::default();
    let html = render(&s.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="unchecked""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let s = Switch::new(true);
    let html = render(&render_for_hydration(&s));
    assert!(html.contains(r#"data-hydrate-checked="checked""#));

    let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
    assert_eq!(restored, s);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["CHECKED", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
        let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn name_value_payloads_are_escaped_end_to_end() {
    let hidden_input = switch::hidden_input(
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        false,
        false,
        false,
        vec![],
    );
    let html = render(&switch::root(false, false, vec![], vec![hidden_input]));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&switch::root(
        false,
        false,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
