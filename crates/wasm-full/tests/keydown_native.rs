//! `fandhe_frontend_wasm_full::events` の汎用 keydown 属性契約（イシュー #3753、親 #3752）の
//! native テスト。
//!
//! 純粋層（[`action_from_keydown`] ほか）は web-sys に依存しないため、`wasm32` ターゲットや
//! 実 DOM を介さず公開 API 経由で検証できる（`keynav_native.rs` と同じ 2 層構成方針）。
//! リスナー登録・`preventDefault()` の実呼び出しなど配線層の検証は #3754 が担う。

use std::collections::HashMap;

use fandhe_frontend_wasm_full::events::{
    action_from_keydown, ignore_repeat_from_attr, is_composing_keydown, keydown_payload,
    parse_keys, prevent_default_from_attr, AttrSource, KeyModifiers, KeydownInput, KeysParseError,
    ACTION_KEYDOWN_ATTR, KEYDOWN_IGNORE_REPEAT_ATTR, KEYDOWN_PREVENT_DEFAULT_ATTR, KEYS_ATTR,
    MAX_KEYS_ATTR_LEN, MAX_KEY_NAME_LEN, MAX_KEY_TOKENS,
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
        repeat: false,
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
        parse_keys(&["a"; MAX_KEY_TOKENS + 1].join(" ")),
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

fn el_with(keys: &str, extra: &[(&str, &str)]) -> Fake {
    let mut f = el(Some(keys));
    for (k, v) in extra {
        f.0.insert((*k).to_string(), (*v).to_string());
    }
    f
}

fn repeating(key: &str) -> KeydownInput<'_> {
    KeydownInput {
        repeat: true,
        ..input(key, NONE)
    }
}

#[test]
fn repeat_is_not_suppressed_without_attr() {
    let r = action_from_keydown(&el(Some("Enter")), &repeating("Enter")).unwrap();
    assert!(!r.repeat_suppressed);
}

#[test]
fn repeat_is_suppressed_with_opt_in_attr() {
    for v in ["", "true"] {
        let t = el_with("Enter", &[(KEYDOWN_IGNORE_REPEAT_ATTR, v)]);
        assert!(
            action_from_keydown(&t, &repeating("Enter"))
                .unwrap()
                .repeat_suppressed
        );
        // 最初の keydown（repeat == false）は抑止されない。
        assert!(
            !action_from_keydown(&t, &input("Enter", NONE))
                .unwrap()
                .repeat_suppressed
        );
    }
}

#[test]
fn repeat_false_attr_value_does_not_suppress() {
    let t = el_with("Enter", &[(KEYDOWN_IGNORE_REPEAT_ATTR, "false")]);
    assert!(
        !action_from_keydown(&t, &repeating("Enter"))
            .unwrap()
            .repeat_suppressed
    );
}

#[test]
fn repeat_opt_in_still_requires_key_match_and_no_ime() {
    let t = el_with("Enter", &[(KEYDOWN_IGNORE_REPEAT_ATTR, "")]);
    assert!(action_from_keydown(&t, &repeating("a")).is_none());
    let composing = KeydownInput {
        is_composing: true,
        ..repeating("Enter")
    };
    assert!(action_from_keydown(&t, &composing).is_none());
}

#[test]
fn repeat_suppression_keeps_prevent_default() {
    let t = el_with(
        "Enter",
        &[
            (KEYDOWN_IGNORE_REPEAT_ATTR, ""),
            (KEYDOWN_PREVENT_DEFAULT_ATTR, ""),
        ],
    );
    let r = action_from_keydown(&t, &repeating("Enter")).unwrap();
    assert!(r.repeat_suppressed);
    assert!(r.prevent_default);
    assert_eq!(r.action_ref.payload, "Enter");
}

#[test]
fn ignore_repeat_attr_grammar() {
    assert!(ignore_repeat_from_attr(Some("")));
    assert!(ignore_repeat_from_attr(Some("true")));
    assert!(!ignore_repeat_from_attr(None));
    assert!(!ignore_repeat_from_attr(Some("false")));
    assert!(!ignore_repeat_from_attr(Some("1")));
}

// ---- data-payload の合成（イシュー #3764、設計記録 §40.6）----

fn el_with_payload(keys: &str, payload: &str) -> Fake {
    let mut e = el(Some(keys));
    e.0.insert("data-payload".to_string(), payload.to_string());
    e
}

#[test]
fn data_payload_takes_priority_over_key_token() {
    let r = action_from_keydown(&el_with_payload("Escape", "42"), &input("Escape", NONE)).unwrap();
    assert_eq!(r.action_ref.payload, "42");
}

#[test]
fn data_payload_wins_with_modifiers() {
    let m = KeyModifiers {
        ctrl: true,
        shift: true,
        ..NONE
    };
    let r = action_from_keydown(
        &el_with_payload("Control+Shift+Enter", "7"),
        &input("Enter", m),
    )
    .unwrap();
    assert_eq!(r.action_ref.payload, "7");
}

#[test]
fn empty_data_payload_is_used_as_empty_string() {
    let r = action_from_keydown(&el_with_payload("Escape", ""), &input("Escape", NONE)).unwrap();
    assert_eq!(r.action_ref.payload, "");
}

#[test]
fn same_data_payload_for_every_matching_key() {
    let e = el_with_payload("Enter Space", "x");
    for key in ["Enter", " "] {
        let r = action_from_keydown(&e, &input(key, NONE)).unwrap();
        assert_eq!(r.action_ref.payload, "x");
    }
}

#[test]
fn data_payload_does_not_affect_matching() {
    let e = el_with_payload("Escape", "x");
    assert!(action_from_keydown(&e, &input("Enter", NONE)).is_none());
    let mut ime = input("Escape", NONE);
    ime.is_composing = true;
    assert!(action_from_keydown(&e, &ime).is_none());
    let mut no_keys = el_with_payload("Escape", "x");
    no_keys.0.remove(KEYS_ATTR);
    assert!(action_from_keydown(&no_keys, &input("Escape", NONE)).is_none());
    assert!(action_from_keydown(&el_with_payload("Bad++", "x"), &input("Escape", NONE)).is_none());
}

#[test]
fn data_payload_keeps_repeat_and_prevent_default_results() {
    for with_payload in [false, true] {
        let mut e = el(Some("Escape"));
        if with_payload {
            e.0.insert("data-payload".to_string(), "x".to_string());
        }
        e.0.insert(KEYDOWN_IGNORE_REPEAT_ATTR.to_string(), String::new());
        e.0.insert(KEYDOWN_PREVENT_DEFAULT_ATTR.to_string(), String::new());
        let mut i = input("Escape", NONE);
        i.repeat = true;
        let r = action_from_keydown(&e, &i).unwrap();
        assert!(r.repeat_suppressed);
        assert!(r.prevent_default);
    }
}

#[test]
fn script_like_data_payload_passes_through_as_string() {
    let p = "<script>alert(1)</script>";
    let r = action_from_keydown(&el_with_payload("Escape", p), &input("Escape", NONE)).unwrap();
    assert_eq!(r.action_ref.payload, p);
}
