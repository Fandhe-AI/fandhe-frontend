//! PinInput（イシュー #739）の統合テスト。
//!
//! `crates/headless-ui/src/pin_input.rs` の inline unit tests がパーツ単体の
//! 属性出力・状態機械の遷移を固定するのに対し、本ファイルは「root >
//! control(input × count) + label + hidden_input」の組み立て全体の
//! data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・XSS 回帰を
//! クレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::pin_input::{self, PinInput, PinInputKind, PinInputProps};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_control_inputs_label_and_hidden_input() {
    let props = PinInputProps::default();
    let inputs: Vec<_> = (0..4)
        .map(|i| {
            pin_input::input(
                i,
                4,
                "",
                PinInputKind::Numeric,
                false,
                true,
                &props,
                false,
                vec![],
            )
        })
        .collect();
    let control = pin_input::control(vec![], inputs);
    let label = pin_input::label(
        false,
        &props,
        vec![],
        vec![fandhe_frontend_core::text("Enter OTP")],
    );
    let hidden_input = pin_input::hidden_input("otp", "", false, vec![]);
    let root = pin_input::root(false, &props, vec![], vec![label, control, hidden_input]);

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
    assert!(html.contains(r#"data-index="0""#));
    assert!(html.contains(r#"data-index="3""#));
    assert!(html.contains("Enter OTP"));
    assert!(html.contains(r#"type="hidden""#));
}

#[test]
fn full_assembly_with_invalid_readonly_required_props_wires_all_new_data_attrs() {
    // ark-ui 公式 Data Attributes 表との突合（イシュー #1615）: root/label/
    // input が data-invalid/data-readonly を、label のみ data-required を、
    // input のみ aria-invalid/native readonly を出力することを、公開 API
    // のみを使った組み立て全体で固定する。
    let props = PinInputProps {
        disabled: false,
        readonly: true,
        invalid: true,
        required: true,
    };
    let label = pin_input::label(false, &props, vec![], vec![]);
    let input0 = pin_input::input(
        0,
        2,
        "1",
        PinInputKind::Numeric,
        false,
        false,
        &props,
        false,
        vec![],
    );
    let root = pin_input::root(false, &props, vec![], vec![label, input0]);

    let html = render(&root);
    assert_eq!(html.matches(r#"data-invalid="""#).count(), 3);
    assert_eq!(html.matches(r#"data-readonly="""#).count(), 3);
    assert_eq!(html.matches(r#"data-required="""#).count(), 1);
    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(html.contains(r#"readonly="""#));
    assert!(html.contains(r#"data-filled="""#));
}

#[test]
fn dispatch_input_backspace_and_paste_flow_via_public_api() {
    let mut p = PinInput::new(4, PinInputKind::Numeric);
    let props = PinInputProps::default();

    assert!(dispatch(&mut p, "input", "1"));
    assert!(dispatch(&mut p, "input", "2"));
    assert_eq!(p.value(), "12");
    assert!(render(&p.input(0, false, false, &props, vec![])).contains(r#"value="1""#));

    // ark-ui の Backspace は「現在桁を消去し前の桁へ移動」（イシュー
    // #1615）。focused は現在 2（3 桁目、空）のため、1 回目は no-op で
    // 現在桁が既に空のまま focused=1 へ移動し、2 回目で digit(1) を
    // 実際に消去する。
    assert!(dispatch(&mut p, "backspace", ""));
    assert_eq!(p.value(), "12");
    assert_eq!(p.focused_index(), Some(1));
    assert!(dispatch(&mut p, "backspace", ""));
    assert_eq!(p.value(), "1");
    assert_eq!(p.focused_index(), Some(0));

    assert!(dispatch(&mut p, "paste", "5678"));
    assert_eq!(p.value(), "5678");
    assert!(p.is_complete());
    assert!(render(&p.root(&props, vec![], vec![])).contains(r#"data-complete="""#));

    assert!(dispatch(&mut p, "clear", ""));
    assert_eq!(p.value(), "");
    assert!(!p.is_complete());

    assert!(!dispatch(&mut p, "no_such_action", ""));
}

#[test]
fn dispatch_delete_prev_next_flow_via_public_api() {
    // ark-ui Keyboard Support 表の Delete/ArrowLeft/ArrowRight を公開 API
    // 経由で固定する（イシュー #1615 で新設した dispatch 語彙）。
    let mut p = PinInput::new(3, PinInputKind::Numeric);

    assert!(dispatch(&mut p, "input", "1"));
    assert!(dispatch(&mut p, "input", "2"));
    // focused は現在 2（3 桁目、空）。
    assert!(dispatch(&mut p, "prev", ""));
    assert_eq!(p.focused_index(), Some(1));
    assert!(dispatch(&mut p, "prev", ""));
    assert_eq!(p.focused_index(), Some(0));

    assert!(dispatch(&mut p, "next", ""));
    assert_eq!(p.focused_index(), Some(1));

    assert!(dispatch(&mut p, "delete", ""));
    assert_eq!(p.digit(1), "");
    // Delete はフォーカスを移動しない。
    assert_eq!(p.focused_index(), Some(1));

    assert!(dispatch(&mut p, "backspace", ""));
    assert_eq!(p.digit(1), "");
    // Backspace は前の桁へフォーカスを移す（現在桁は既に空）。
    assert_eq!(p.focused_index(), Some(0));
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
    let html = render(&pin_input::root(
        false,
        &PinInputProps::default(),
        vec![],
        vec![hidden_input],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&pin_input::root(
        false,
        &PinInputProps::default(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
