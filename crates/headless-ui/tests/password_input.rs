//! PasswordInput（イシュー #740）の統合テスト。
//!
//! `crates/headless-ui/src/password_input.rs` の inline unit tests がパーツ
//! 単体の属性出力を固定するのに対し、本ファイルは「root > control(input +
//! visibility_trigger) + label + indicator」の組み立て全体の data-*/ARIA
//! 対応・dispatch 統合・SSR/hydration 両経路・XSS 回帰・**パスワード値の
//! 非出力**をクレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::password_input::{
    self, PasswordAutocomplete, PasswordInput, PasswordInputProps,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

fn props(id: &str) -> PasswordInputProps<'_> {
    PasswordInputProps {
        id,
        disabled: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    }
}

#[test]
fn full_assembly_wires_root_control_input_trigger_label_and_indicator() {
    let field_props = props("login-password");
    let input = password_input::input(false, &field_props, vec![]);
    let trigger = password_input::visibility_trigger(
        false,
        &field_props,
        vec![("aria-label", "Show password")],
        vec![],
    );
    let control = password_input::control(false, &field_props, vec![], vec![input, trigger]);
    let label = password_input::label(&field_props, vec![], vec![text("Password")]);
    let indicator = password_input::indicator(false, vec![], vec![]);
    let root = password_input::root(false, &field_props, vec![], vec![label, control, indicator]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="password-input""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"data-part="input""#));
    assert!(html.contains(r#"data-part="visibility-trigger""#));
    assert!(html.contains(r#"data-part="indicator""#));
    assert!(html.contains(r#"data-state="hidden""#));
    assert!(html.contains(r#"type="password""#));
    assert!(html.contains(r#"for="login-password-input""#));
    assert!(html.contains(r#"id="login-password-input""#));
    assert!(html.contains(r#"aria-controls="login-password-input""#));
    assert!(html.contains("Password"));
    assert!(html.contains("Show password"));
    assert!(!html.contains("value="));
}

#[test]
fn dispatch_toggle_flips_data_state_and_input_type_across_parts() {
    let field_props = props("pw");
    let mut p = PasswordInput::default();
    assert!(!p.is_visible());
    assert!(render(&p.input(&field_props, vec![])).contains(r#"type="password""#));

    assert!(dispatch(&mut p, "toggle", ""));
    assert!(p.is_visible());
    assert!(render(&p.root(&field_props, vec![], vec![])).contains(r#"data-state="visible""#));
    assert!(render(&p.control(&field_props, vec![], vec![])).contains(r#"data-state="visible""#));
    assert!(render(&p.input(&field_props, vec![])).contains(r#"type="text""#));
    assert!(render(&p.visibility_trigger(&field_props, vec![], vec![]))
        .contains(r#"aria-pressed="true""#));
    assert!(render(&p.indicator(vec![], vec![])).contains(r#"data-state="visible""#));

    assert!(dispatch(&mut p, "hide", ""));
    assert!(!p.is_visible());

    assert!(dispatch(&mut p, "show", ""));
    assert!(p.is_visible());

    assert!(!dispatch(&mut p, "no_such_action", ""));
    assert!(p.is_visible());
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr_and_no_value() {
    let p = PasswordInput::default();
    let html = render(&p.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="hidden""#));
    assert!(!html.contains("value="));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let p = PasswordInput::new(true);
    let html = render(&render_for_hydration(&p));
    assert!(html.contains(r#"data-hydrate-visible="visible""#));

    let restored = PasswordInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
    assert_eq!(restored, p);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["VISIBLE", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-visible".to_string(), bogus.to_string())];
        let err = PasswordInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn id_payload_is_escaped_end_to_end() {
    let field_props = props(ATTR_BREAK_PAYLOAD);
    let html = render(&password_input::label(&field_props, vec![], vec![]));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let field_props = props("pw");
    let html = render(&password_input::root(
        false,
        &field_props,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn children_text_is_escaped_end_to_end() {
    let field_props = props("pw");
    let html = render(&password_input::label(
        &field_props,
        vec![],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}
