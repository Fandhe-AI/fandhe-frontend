//! `pin_input` 配線の純粋ロジック層の native テスト（イシュー #3834）。
//!
//! web-sys に依存しない判定関数（桁番号のパース・入力値の分類・kind 解決・事前フィルタ・
//! フォーカス移動先・dispatch 列）を固定する。実ブラウザでの挙動は #3836 の担当。

#![cfg(feature = "pin-input")]

use fandhe_frontend_headless_ui::pin_input::{PinInput, PinInputKind};
use fandhe_frontend_interactive::dispatch;
use fandhe_frontend_wasm_full::pin_input::{
    classify_input_value, input_accepted, input_dispatches, next_focus_index, parse_digit_index,
    prefilter_char, resolve_kind, InputValue, KindResolution,
};

#[test]
fn parse_digit_index_is_fail_closed() {
    assert_eq!(parse_digit_index(Some("0"), 6), Some(0));
    assert_eq!(parse_digit_index(Some("5"), 6), Some(5));
    assert_eq!(parse_digit_index(Some("6"), 6), None);
    assert_eq!(parse_digit_index(Some("-1"), 6), None);
    assert_eq!(parse_digit_index(Some("x"), 6), None);
    assert_eq!(parse_digit_index(Some(""), 6), None);
    assert_eq!(parse_digit_index(None, 6), None);
    assert_eq!(parse_digit_index(Some("0"), 0), None);
}

#[test]
fn classify_input_value_reads_at_most_two_chars() {
    assert_eq!(classify_input_value(""), InputValue::Empty);
    assert_eq!(classify_input_value("7"), InputValue::Single('7'));
    assert_eq!(classify_input_value("あ"), InputValue::Single('あ'));
    assert_eq!(classify_input_value("12"), InputValue::Multi);
    assert_eq!(
        classify_input_value(&"9".repeat(100_000)),
        InputValue::Multi
    );
}

#[test]
fn resolve_kind_covers_all_five_and_rejects_unknown() {
    for kind in [
        PinInputKind::Numeric,
        PinInputKind::Alphanumeric,
        PinInputKind::Alphabetic,
        PinInputKind::AlphanumericUppercase,
        PinInputKind::AlphabeticUppercase,
    ] {
        assert_eq!(
            resolve_kind(Some(kind.as_str())),
            KindResolution::Known(kind)
        );
    }
    assert_eq!(resolve_kind(None), KindResolution::Unknown);
    assert_eq!(resolve_kind(Some("<script>")), KindResolution::Tampered);
    assert_eq!(resolve_kind(Some("")), KindResolution::Tampered);
}

#[test]
fn prefilter_char_follows_kind() {
    let numeric = KindResolution::Known(PinInputKind::Numeric);
    assert_eq!(prefilter_char(numeric, '5'), Some('5'));
    assert_eq!(prefilter_char(numeric, 'a'), None);

    let upper = KindResolution::Known(PinInputKind::AlphanumericUppercase);
    assert_eq!(prefilter_char(upper, 'a'), Some('A'));
    assert_eq!(prefilter_char(upper, '3'), Some('3'));
    assert_eq!(prefilter_char(upper, '-'), None);

    assert_eq!(prefilter_char(KindResolution::Unknown, 'z'), Some('z'));
    assert_eq!(prefilter_char(KindResolution::Unknown, '9'), Some('9'));
    assert_eq!(prefilter_char(KindResolution::Unknown, 'é'), None);
    assert_eq!(prefilter_char(KindResolution::Unknown, '-'), None);
    assert_eq!(prefilter_char(KindResolution::Tampered, '1'), None);
}

#[test]
fn next_focus_index_stays_on_last_digit() {
    assert_eq!(next_focus_index(0, 6), Some(1));
    assert_eq!(next_focus_index(4, 6), Some(5));
    assert_eq!(next_focus_index(5, 6), Some(5));
    assert_eq!(next_focus_index(usize::MAX, 6), Some(5));
    assert_eq!(next_focus_index(0, 0), None);
}

#[test]
fn input_dispatches_order_and_payload() {
    let [focus, value] = input_dispatches(2, 'K');
    assert_eq!(
        (focus.action.as_str(), focus.payload.as_str()),
        ("focus", "2")
    );
    assert_eq!(
        (value.action.as_str(), value.payload.as_str()),
        ("input", "K")
    );
}

#[test]
fn dispatch_sequence_fills_digit_via_state_machine() {
    let mut pin = PinInput::new(4, PinInputKind::AlphanumericUppercase);
    let c = prefilter_char(KindResolution::Known(pin.kind()), 'q').unwrap();
    for action in input_dispatches(2, c) {
        assert!(dispatch(&mut pin, &action.action, &action.payload));
    }
    assert_eq!(pin.digit(2), "Q");
    assert_eq!(pin.digit(0), "");
}

#[test]
fn input_accepted_checks_rendered_value_attribute() {
    assert!(input_accepted(Some("7"), '7'));
    assert!(input_accepted(Some("K"), 'k'));
    assert!(!input_accepted(Some(""), 'a'));
    assert!(!input_accepted(None, 'a'));
    assert!(!input_accepted(Some("1"), 'a'));
    assert!(!input_accepted(Some("12"), '1'));
}
