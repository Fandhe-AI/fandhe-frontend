//! `fandhe_frontend_wasm_full::events` の汎用 keydown 属性契約（イシュー #3753、親 #3752）の
//! native テスト。
//!
//! 純粋層（[`action_from_keydown`] ほか）は web-sys に依存しないため、`wasm32` ターゲットや
//! 実 DOM を介さず公開 API 経由で検証できる（`keynav_native.rs` と同じ 2 層構成方針）。
//! リスナー登録・`preventDefault()` の実呼び出しなど配線層の検証は #3754 が担う。

use std::collections::HashMap;

use fandhe_frontend_wasm_full::events::{
    action_from_keydown, is_composing_keydown, keydown_payload, parse_keys,
    prevent_default_from_attr, AttrSource, KeyModifiers, KeydownInput, KeysParseError,
    ACTION_KEYDOWN_ATTR, KEYDOWN_PREVENT_DEFAULT_ATTR, KEYS_ATTR, MAX_KEYS_ATTR_LEN,
    MAX_KEY_NAME_LEN, MAX_KEY_TOKENS,
};

struct Fake(HashMap<String, String>);

impl AttrSource for Fake {
    fn attr(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

fn el(keys: Option<&str>) -> Fake {
    let mut m = HashMap::new();
    m.insert(ACTION_KEYDOWN_ATTR.to_string(), "stop".to_string());
    if let Some(k) = keys {
        m.insert(KEYS_ATTR.to_string(), k.to_string());
    }
    Fake(m)
}

fn input(key: &str, modifiers: KeyModifiers) -> KeydownInput<'_> {
    KeydownInput {
        key,
        modifiers,
        is_composing: false,
        key_code: 0,
    }
}

const NONE: KeyModifiers = KeyModifiers {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
};

#[test]
fn matching_key_dispatches_with_payload() {
    let r = action_from_keydown(&el(Some("Escape")), &input("Escape", NONE)).unwrap();
    assert_eq!(r.action_ref.action, "stop");
    assert_eq!(r.action_ref.payload, "Escape");
    assert!(!r.prevent_default);
}

#[test]
fn non_matching_key_is_ignored() {
    assert!(action_from_keydown(&el(Some("Escape")), &input("Enter", NONE)).is_none());
    assert!(action_from_keydown(&el(Some("Escape")), &input("Tab", NONE)).is_none());
    let multi = el(Some("Escape Enter"));
    assert!(action_from_keydown(&multi, &input("Enter", NONE)).is_some());
    assert!(action_from_keydown(&multi, &input("a", NONE)).is_none());
}

#[test]
fn modifiers_must_match_exactly() {
    let ctrl = KeyModifiers { ctrl: true, ..NONE };
    let ctrl_shift = KeyModifiers {
        ctrl: true,
        shift: true,
        ..NONE
    };
    let t = el(Some("Control+Enter"));
    assert!(action_from_keydown(&t, &input("Enter", ctrl)).is_some());
    assert!(action_from_keydown(&t, &input("Enter", NONE)).is_none());
    assert!(action_from_keydown(&t, &input("Enter", ctrl_shift)).is_none());
    let plain = el(Some("Escape"));
    assert!(action_from_keydown(&plain, &input("Escape", ctrl)).is_none());
    let all = KeyModifiers {
        ctrl: true,
        alt: true,
        shift: true,
        meta: true,
    };
    assert_eq!(keydown_payload("X", all), "Control+Alt+Shift+Meta+X");
}

#[test]
fn ime_composition_is_excluded() {
    let t = el(Some("Enter Escape"));
    let mut i = input("Enter", NONE);
    i.is_composing = true;
    assert!(action_from_keydown(&t, &i).is_none());
    let mut i = input("Escape", NONE);
    i.key_code = 229;
    assert!(action_from_keydown(&t, &i).is_none());
    assert!(is_composing_keydown(true, 0));
    assert!(is_composing_keydown(false, 229));
    assert!(!is_composing_keydown(false, 13));
}

#[test]
fn invalid_attribute_values_fail_closed() {
    for bad in [
        "",
        "   ",
        "+Escape",
        "Escape+",
        "Control++K",
        "Ctrl+K",
        "control+K",
        "Control+Control+K",
        "Esc\u{7}ape",
        "Escape Ctrl+K",
    ] {
        assert!(parse_keys(bad).is_err(), "{bad:?}");
        assert!(
            action_from_keydown(&el(Some(bad)), &input("Escape", NONE)).is_none(),
            "{bad:?}"
        );
    }
    assert!(action_from_keydown(&el(None), &input("Escape", NONE)).is_none());
    assert_eq!(
        parse_keys(&"a".repeat(MAX_KEYS_ATTR_LEN + 1)),
        Err(KeysParseError::TooLong)
    );
    assert_eq!(
        parse_keys(&vec!["a"; MAX_KEY_TOKENS + 1].join(" ")),
        Err(KeysParseError::TooManyTokens)
    );
    assert_eq!(
        parse_keys(&"a".repeat(MAX_KEY_NAME_LEN + 1)),
        Err(KeysParseError::InvalidKeyName)
    );
}

#[test]
fn action_name_must_be_present_and_non_empty() {
    let mut no_action = el(Some("Escape"));
    no_action.0.remove(ACTION_KEYDOWN_ATTR);
    assert!(action_from_keydown(&no_action, &input("Escape", NONE)).is_none());
    let mut empty = el(Some("Escape"));
    empty
        .0
        .insert(ACTION_KEYDOWN_ATTR.to_string(), String::new());
    assert!(action_from_keydown(&empty, &input("Escape", NONE)).is_none());
}

#[test]
fn space_and_plus_aliases() {
    let t = el(Some("Space Plus"));
    assert_eq!(
        action_from_keydown(&t, &input(" ", NONE))
            .unwrap()
            .action_ref
            .payload,
        "Space"
    );
    assert_eq!(
        action_from_keydown(&t, &input("+", NONE))
            .unwrap()
            .action_ref
            .payload,
        "Plus"
    );
}

#[test]
fn shift_is_case_sensitive_as_reported_by_browsers() {
    let shift = KeyModifiers {
        shift: true,
        ..NONE
    };
    assert!(action_from_keydown(&el(Some("Shift+A")), &input("A", shift)).is_some());
    assert!(action_from_keydown(&el(Some("Shift+a")), &input("A", shift)).is_none());
}

#[test]
fn prevent_default_is_opt_in() {
    assert!(prevent_default_from_attr(Some("")));
    assert!(prevent_default_from_attr(Some("true")));
    for v in [None, Some("false"), Some("1"), Some("yes")] {
        assert!(!prevent_default_from_attr(v));
    }
    let mut t = el(Some("Escape"));
    t.0.insert(KEYDOWN_PREVENT_DEFAULT_ATTR.to_string(), String::new());
    assert!(
        action_from_keydown(&t, &input("Escape", NONE))
            .unwrap()
            .prevent_default
    );
    assert!(action_from_keydown(&t, &input("Enter", NONE)).is_none());
}
