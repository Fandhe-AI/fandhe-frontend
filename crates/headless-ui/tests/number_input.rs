//! NumberInput（イシュー #738）の統合テスト。
//!
//! `crates/headless-ui/src/number_input.rs` の inline unit tests がパーツ
//! 単体の属性出力・状態機械の正規化/dispatch/hydration を固定するのに対し、
//! 本ファイルは
//! `root(label + control(input, increment_trigger, decrement_trigger))`
//! という全体の組み立てにおける data-*/ARIA 対応・境界到達時のトリガー
//! disabled 連動・SSR/hydration 両経路をクレート外部から（公開 API のみを
//! 使って）固定する（`tests/select.rs` と同粒度）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::number_input::{self, NumberInput, NumberInputFlags};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate, HydrateError};

#[test]
fn full_assembly_wires_control_input_and_triggers() {
    let n = NumberInput::new(Some(40.0), 0.0, 100.0, 1.0);

    let label = n.label(
        NumberInputFlags::default(),
        Some("qty-input"),
        vec![],
        vec![text("Quantity")],
    );
    let input = n.input(
        "qty",
        Some("qty-input"),
        NumberInputFlags::default(),
        vec![],
    );
    let increment = n.increment_trigger(Some("qty-input"), false, vec![], vec![]);
    let decrement = n.decrement_trigger(Some("qty-input"), false, vec![], vec![]);
    let control = n.control(
        NumberInputFlags::default(),
        vec![],
        vec![input, increment, decrement],
    );
    let value_text = n.value_text(NumberInputFlags::default(), vec![]);
    let root = n.root(
        NumberInputFlags::default(),
        vec![],
        vec![label, control, value_text],
    );

    let html = render(&root);
    assert!(html.contains(r#"data-scope="number-input" data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"for="qty-input""#));
    assert!(html.contains("Quantity"));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"data-part="input""#));
    assert!(html.contains(r#"role="spinbutton""#));
    assert!(html.contains(r#"aria-valuemin="0""#));
    assert!(html.contains(r#"aria-valuemax="100""#));
    assert!(html.contains(r#"aria-valuenow="40""#));
    assert!(html.contains(r#"id="qty-input""#));
    assert!(html.contains(r#"data-part="increment-trigger""#));
    assert!(html.contains(r#"data-part="decrement-trigger""#));
    assert!(html.contains(r#"aria-controls="qty-input""#));
    assert!(html.contains(r#"data-part="value-text""#));
    assert!(html.contains(">40<"));
}

/// ValueText パーツ（イシュー #1613）が [`NumberInput::formatted_value`]
/// と一致する文字列を出力し、full assembly から独立して単体でも使えることを
/// 固定する。
#[test]
fn value_text_reflects_formatted_value() {
    let n = NumberInput::new(Some(7.5), 0.0, 10.0, 0.5);
    let html = render(&n.value_text(NumberInputFlags::default(), vec![]));
    assert!(html.contains(r#"data-part="value-text""#));
    assert!(html.contains(">7.5<"));
    assert_eq!(n.formatted_value(), "7.5");
}

#[test]
fn increment_trigger_becomes_disabled_at_max_via_convenience_method() {
    let n = NumberInput::new(Some(100.0), 0.0, 100.0, 1.0);
    let html = render(&n.increment_trigger(None, false, vec![], vec![]));
    assert!(html.contains(r#"disabled="""#));
    assert!(html.contains(r#"data-disabled="""#));
}

#[test]
fn decrement_trigger_becomes_disabled_at_min_via_convenience_method() {
    let n = NumberInput::new(Some(0.0), 0.0, 100.0, 1.0);
    let html = render(&n.decrement_trigger(None, false, vec![], vec![]));
    assert!(html.contains(r#"disabled="""#));
    assert!(html.contains(r#"data-disabled="""#));
}

#[test]
fn triggers_stay_enabled_mid_range() {
    let n = NumberInput::new(Some(50.0), 0.0, 100.0, 1.0);
    let inc_html = render(&n.increment_trigger(None, false, vec![], vec![]));
    let dec_html = render(&n.decrement_trigger(None, false, vec![], vec![]));
    assert!(!inc_html.contains("disabled"));
    assert!(!dec_html.contains("disabled"));
}

#[test]
fn overall_disabled_flag_forces_trigger_disabled_regardless_of_bounds() {
    let n = NumberInput::new(Some(50.0), 0.0, 100.0, 1.0);
    let inc_html = render(&n.increment_trigger(None, true, vec![], vec![]));
    assert!(inc_html.contains(r#"disabled="""#));
}

#[test]
fn dispatch_and_ssr_hydration_round_trip_via_full_state_machine() {
    let mut n = NumberInput::new(Some(0.0), 0.0, 10.0, 1.0);

    let ssr_html = render(&number_input::input(
        "qty",
        None,
        Some(n.formatted_value())
            .filter(|s| !s.is_empty())
            .as_deref(),
        "0",
        "10",
        NumberInputFlags::default(),
        vec![],
    ));
    assert!(ssr_html.contains(r#"value="0""#));

    assert!(dispatch(&mut n, "increment", ""));
    assert_eq!(n.value(), Some(1.0));

    let hydrate_html = render(&render_for_hydration(&n));
    assert!(hydrate_html.contains(r#"data-hydrate-value="1""#));

    let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
    assert_eq!(restored, n);
}

#[test]
fn hydration_rejects_tampered_step_without_panicking() {
    let attrs = vec![
        ("data-hydrate-value".to_string(), "5".to_string()),
        ("data-hydrate-min".to_string(), "0".to_string()),
        ("data-hydrate-max".to_string(), "10".to_string()),
        ("data-hydrate-step".to_string(), "-1".to_string()),
    ];
    let err = NumberInput::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}
