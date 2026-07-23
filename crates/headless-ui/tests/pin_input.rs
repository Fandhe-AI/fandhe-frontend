//! PinInput（イシュー #739）の統合テスト。
//!
//! `crates/headless-ui/src/pin_input.rs` の inline unit tests がパーツ単体の
//! 属性出力・状態機械の遷移を固定するのに対し、本ファイルは「root >
//! control(input × count) + label + hidden_input」の組み立て全体の
//! data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・XSS 回帰を
//! クレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::pin_input::{self, PinInput, PinInputKind};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_control_inputs_label_and_hidden_input() {
    let inputs: Vec<_> = (0..4)
        .map(|i| {
            pin_input::input(
                i,
                4,
                "",
                PinInputKind::Numeric,
                false,
                true,
                false,
                false,
                vec![],
            )
        })
        .collect();
    let control = pin_input::control(vec![], inputs);
    let label = pin_input::label(false, vec![], vec![fandhe_frontend_core::text("Enter OTP")]);
    let hidden_input = pin_input::hidden_input("otp", "", false, vec![]);
    let root = pin_input::root(false, false, vec![], vec![label, control, hidden_input]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="pin-input""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"data-part="input""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="hidden-input""#));
    assert!(html.contains(r#"aria-label="PIN digit 1 of 4""#));
    assert!(html.contains(r#"aria-label="PIN digit 4 of 4""#));
    assert!(html.contains(r#"autocomplete="one-time-code""#));
    assert!(html.contains("Enter OTP"));
    assert!(html.contains(r#"type="hidden""#));
}

#[test]
fn dispatch_input_backspace_and_paste_flow_via_public_api() {
    let mut p = PinInput::new(4, PinInputKind::Numeric);

    assert!(dispatch(&mut p, "input", "1"));
    assert!(dispatch(&mut p, "input", "2"));
    assert_eq!(p.value(), "12");
    assert!(render(&p.input(0, false, false, false, vec![])).contains(r#"value="1""#));

    assert!(dispatch(&mut p, "backspace", ""));
    assert_eq!(p.value(), "1");

    assert!(dispatch(&mut p, "paste", "5678"));
    assert_eq!(p.value(), "5678");
    assert!(p.is_complete());
    assert!(render(&p.root(false, vec![], vec![])).contains(r#"data-complete="""#));

    assert!(dispatch(&mut p, "clear", ""));
    assert_eq!(p.value(), "");
    assert!(!p.is_complete());

    assert!(!dispatch(&mut p, "no_such_action", ""));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let p = PinInput::default();
    let html = render(&p.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut p = PinInput::new(4, PinInputKind::Numeric);
    dispatch(&mut p, "paste", "9876");
    let html = render(&render_for_hydration(&p));
    assert!(html.contains(r#"data-hydrate-count="4""#));
    assert!(html.contains(r#"data-hydrate-kind="numeric""#));

    let restored = PinInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
    assert_eq!(restored.value(), p.value());
    assert_eq!(restored.kind(), p.kind());
}

#[test]
fn hydration_tampered_count_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-count".to_string(), "-1".to_string()),
        ("data-hydrate-kind".to_string(), "numeric".to_string()),
        ("data-hydrate-values".to_string(), String::new()),
    ];
    let err = PinInput::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn name_value_payloads_are_escaped_end_to_end() {
    let hidden_input =
        pin_input::hidden_input(ATTR_BREAK_PAYLOAD, ATTR_BREAK_PAYLOAD, false, vec![]);
    let html = render(&pin_input::root(false, false, vec![], vec![hidden_input]));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&pin_input::root(
        false,
        false,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
