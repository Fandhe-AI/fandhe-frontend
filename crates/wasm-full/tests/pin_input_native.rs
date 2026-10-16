//! `pin_input` 配線の純粋ロジック層の native テスト（イシュー #3834）。
//!
//! 不変条件「入力の受理・拒否と focused は状態機械だけが決め、DOM 層は状態を読み戻して
//! 反映するだけ」を、実際の `PinInput` へ配線と同じ dispatch 列を流して検証する
//! （`simulate_input` が配線の「dispatch → 読み戻し → 反映計画」を再現する）。
//! PR #3855 のレビュー指摘 11 件はいずれも「DOM 層が受理を推測する」ことの別インスタンス
//! だったため、各指摘を状態機械の結果から導かれる計画として固定する。
//! 実ブラウザでの挙動は `pin_input_browser.rs`（wasm32）が担う。

#![cfg(feature = "pin-input")]

use fandhe_frontend_headless_ui::pin_input::{PinInput, PinInputKind};
use fandhe_frontend_interactive::dispatch;
use fandhe_frontend_wasm_client::BoundValue;
use fandhe_frontend_wasm_full::events::ActionRef;
use fandhe_frontend_wasm_full::pin_input::{
    bound_value_for, classify_input_value, classify_paste_text, deletion_dispatches, field_name,
    input_dispatches, key_dispatches, key_intent, parse_digit_index, parse_field, paste_dispatches,
    reflect_plan, reflect_plan_scoped, snapshot_from_bound, InputValue, KeyIntent, PasteText,
    PinInputField, PinInputSnapshot, ReflectPlan, ValueScope, MAX_PASTE_CHARS,
};

/// ホストの `BindingSource::bound_value`（予約 field を状態へ委譲する最小実装）。
fn host_read(pin: &PinInput, ordinal: usize) -> impl Fn(&str) -> Option<BoundValue> + '_ {
    move |field| {
        let (n, f) = parse_field(field)?;
        (n == ordinal).then(|| bound_value_for(pin, f))
    }
}

/// 配線の `input` 経路を再現する: `focus(index)` → `input(typed)` を dispatch し、
/// 状態を読み戻して DOM 反映計画を作る。`live` は DOM の各桁のライブ値（`typed` 入力済み）。
fn simulate_input(pin: &mut PinInput, index: usize, typed: char, live: &[&str]) -> ReflectPlan {
    for action in input_dispatches(index, typed) {
        assert!(dispatch(pin, &action.action, &action.payload));
    }
    let snapshot = snapshot_from_bound(0, pin.count(), host_read(pin, 0));
    let live: Vec<String> = live.iter().map(|s| (*s).to_string()).collect();
    reflect_plan(snapshot.as_ref(), &live, Some(index), None)
}

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
fn input_dispatches_order_and_payload_are_unmodified() {
    // 小文字のまま無加工で渡す（正規化は状態機械の責務）。
    let [focus, value] = input_dispatches(2, 'k');
    assert_eq!(
        (focus.action.as_str(), focus.payload.as_str()),
        ("focus", "2")
    );
    assert_eq!(
        (value.action.as_str(), value.payload.as_str()),
        ("input", "k")
    );
}

#[test]
fn field_names_round_trip_and_reject_non_canonical_forms() {
    for (ordinal, field) in [
        (0, PinInputField::Focused),
        (3, PinInputField::Focused),
        (0, PinInputField::Digit(0)),
        (12, PinInputField::Digit(7)),
    ] {
        assert_eq!(
            parse_field(&field_name(ordinal, field)),
            Some((ordinal, field))
        );
    }
    assert_eq!(
        field_name(1, PinInputField::Digit(2)),
        "pin-input.1.digit.2"
    );
    for bad in [
        "",
        "pin-input.",
        "pin-input.focused",
        "pin-input.+0.focused",
        "pin-input.00.focused",
        "pin-input.0.digit.",
        "pin-input.0.digit.+1",
        "pin-input.0.digit.01",
        "pin-input.0.other",
        "other.0.focused",
    ] {
        assert_eq!(parse_field(bad), None, "{bad}");
    }
}

#[test]
fn snapshot_is_fail_closed_when_any_field_is_unreadable_or_malformed() {
    let pin = PinInput::new(3, PinInputKind::Numeric);
    assert!(snapshot_from_bound(0, 3, host_read(&pin, 0)).is_some());
    // ホストが予約 field を実装していない。
    assert_eq!(snapshot_from_bound(0, 3, |_| None), None);
    // 別インスタンス（ordinal 不一致）の field しか答えない。
    assert_eq!(snapshot_from_bound(0, 3, host_read(&pin, 1)), None);
    // 型不正・桁超過・focused が範囲外。
    let read = |field: &str| match parse_field(field)?.1 {
        PinInputField::Focused => Some(BoundValue::Text("3".to_string())),
        PinInputField::Digit(_) => Some(BoundValue::Text(String::new())),
    };
    assert_eq!(snapshot_from_bound(0, 3, read), None);
    let read = |field: &str| match parse_field(field)?.1 {
        PinInputField::Focused => Some(BoundValue::Flag(true)),
        PinInputField::Digit(_) => Some(BoundValue::Text(String::new())),
    };
    assert_eq!(snapshot_from_bound(0, 3, read), None);
    let read = |field: &str| match parse_field(field)?.1 {
        PinInputField::Focused => Some(BoundValue::Text(String::new())),
        PinInputField::Digit(_) => Some(BoundValue::Text("ab".to_string())),
    };
    assert_eq!(snapshot_from_bound(0, 3, read), None);
}

#[test]
fn reflect_plan_without_snapshot_never_writes_values() {
    let live = vec!["a".to_string(), String::new()];
    // 読み戻せない: 値は触らない。フォーカスは fallback（focusin の選択桁）だけ。
    assert_eq!(
        reflect_plan(None, &live, Some(0), None),
        ReflectPlan::default()
    );
    assert_eq!(
        reflect_plan(None, &live, Some(0), Some(1)),
        ReflectPlan {
            value_writes: Vec::new(),
            focus: Some(1)
        }
    );
    // focusin（value_index=None）は値を書き換えない。
    let snapshot = PinInputSnapshot {
        focused: Some(0),
        digits: vec![String::new(), String::new()],
    };
    assert_eq!(
        reflect_plan(Some(&snapshot), &live, None, None),
        ReflectPlan {
            value_writes: Vec::new(),
            focus: Some(0)
        }
    );
}

// ---- PR #3855 レビュー指摘 1〜11 の回帰（状態機械の結果から計画が導かれること） ----

/// 指摘 1: focus を状態機械より先に動かさない。フォーカス先は dispatch 後の `focused` だけが決める。
#[test]
fn finding_01_focus_target_comes_from_state_after_dispatch() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    let plan = simulate_input(&mut pin, 1, '5', &["", "5", "", ""]);
    assert_eq!(pin.digit(1), "5");
    assert_eq!(pin.focused_index(), Some(2));
    assert_eq!(plan.focus, pin.focused_index());
    assert!(plan.value_writes.is_empty());
}

/// 指摘 2: kind を DOM 側で知らなくても、Numeric へ 'a' を入れた DOM 値は状態機械の値（空）へ戻り、
/// フォーカスは動かない（状態機械が拒否した結果）。
#[test]
fn finding_02_rejected_char_is_restored_from_state_without_kind() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    let plan = simulate_input(&mut pin, 0, 'a', &["a", "", "", ""]);
    assert_eq!(pin.digit(0), "");
    assert_eq!(plan.value_writes, vec![(0, String::new())]);
    assert_eq!(plan.focus, Some(0));
}

/// 指摘 3: root 自身が PinInput Root でも、予約 field は ordinal 0 で解決できる
/// （Root の列挙は配線層の責務。field 名は `pin-input.0.*`）。
#[test]
fn finding_03_self_root_resolves_to_ordinal_zero_fields() {
    let pin = PinInput::new(2, PinInputKind::Numeric);
    assert_eq!(
        parse_field("pin-input.0.focused"),
        Some((0, PinInputField::Focused))
    );
    assert!(snapshot_from_bound(0, 2, host_read(&pin, 0)).is_some());
}

/// 指摘 4: 再描画で要素が差し替わっても、フォーカスは常に状態機械の focused へ引き直される
/// （`focusin` の "focus" dispatch 後も計画は focused を指す）。
#[test]
fn finding_04_focusin_plan_targets_state_focused_after_rerender() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    assert!(dispatch(&mut pin, "focus", "2"));
    let snapshot = snapshot_from_bound(0, 4, host_read(&pin, 0)).unwrap();
    let live = vec![String::new(); 4];
    let plan = reflect_plan(Some(&snapshot), &live, None, Some(0));
    assert_eq!(plan.focus, Some(2));
}

/// 指摘 5（状態機械の性質のみ。配線そのものは検証しない。回帰は `pin_input_browser.rs` の
/// `ancestor_action_owner_is_dispatched_exactly_once` が担う）: `data-action-input` の所有者がいる桁は
/// 本モジュールが dispatch しない。ここでは「二重 dispatch すると 1 回の入力で
/// 2 桁埋まる」ことを状態機械で示し、単一 dispatch の期待値を固定する。
#[test]
fn finding_05_state_machine_property_double_dispatch_fills_two_digits() {
    let mut once = PinInput::new(4, PinInputKind::Numeric);
    simulate_input(&mut once, 0, '7', &["7", "", "", ""]);
    assert_eq!(once.value(), "7");

    let mut twice = PinInput::new(4, PinInputKind::Numeric);
    simulate_input(&mut twice, 0, '7', &["7", "", "", ""]);
    dispatch(&mut twice, "input", "7");
    assert_eq!(twice.value(), "77");
}

/// 指摘 6: 古い value 属性・ライブ値が拒否文字と一致しても、受理は状態機械の結果だけで決まる。
/// ライブ値 'a' のまま（状態は空）なら値は空へ戻る。
#[test]
fn finding_06_live_value_is_never_evidence_of_acceptance() {
    let mut pin = PinInput::new(3, PinInputKind::Numeric);
    let plan = simulate_input(&mut pin, 1, 'x', &["", "x", ""]);
    assert_eq!(pin.digit(1), "");
    assert_eq!(plan.value_writes, vec![(1, String::new())]);
    assert_eq!(plan.focus, Some(1));
}

/// 指摘 7: 拒否した入力を旧値へ戻しても再 dispatch されない。旧値 '3' の桁へ 'a' を入れて拒否されたら
/// 値は状態の '3' へ戻り、dispatch は focus+input の 1 組だけ（桁値は '3' のまま）。
#[test]
fn finding_07_rejected_input_restores_old_value_without_redispatch() {
    let mut pin = PinInput::new(3, PinInputKind::Numeric);
    simulate_input(&mut pin, 0, '3', &["3", "", ""]);
    assert_eq!(pin.focused_index(), Some(1));
    let plan = simulate_input(&mut pin, 0, 'a', &["a", "", ""]);
    assert_eq!(pin.digit(0), "3");
    assert_eq!(plan.value_writes, vec![(0, "3".to_string())]);
    // "focus"(0) で focused=0 に戻り、拒否された入力では前進しない。
    assert_eq!(plan.focus, Some(0));
    assert_eq!(pin.value(), "3");
}

/// 指摘 8: 同じ文字の再入力は状態機械が受理して focused が次桁へ進む。ビューが変わらなくても
/// フォーカスは状態機械の focused に従う（文字比較で拒否扱いにしない）。
#[test]
fn finding_08_same_char_retype_follows_state_focused() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    simulate_input(&mut pin, 0, '1', &["1", "", "", ""]);
    let plan = simulate_input(&mut pin, 0, '1', &["1", "", "", ""]);
    assert_eq!(pin.digit(0), "1");
    assert_eq!(pin.focused_index(), Some(1));
    assert_eq!(plan.focus, Some(1));
    assert!(plan.value_writes.is_empty());
}

/// 指摘 9（状態機械の性質のみ。配線そのものは検証しない。回帰は `pin_input_browser.rs` の
/// `app_owned_digits_get_no_generic_focus_and_still_reflect_state` が担う）: カスタム桁は汎用 "focus" を送らない。ここでは汎用 "focus" がインスタンス情報なしで状態を書き換えうる
/// （桁 0 のアプリ状態が桁 2 へ移る）ことを示し、送らない設計の根拠を固定する。
#[test]
fn finding_09_state_machine_property_generic_focus_moves_state() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    assert_eq!(pin.focused_index(), None);
    assert!(dispatch(&mut pin, "focus", "2"));
    assert_eq!(pin.focused_index(), Some(2));
}

/// 指摘 10: AlphanumericUppercase の既存桁 'A' へ 'a' を再入力。状態機械は正規化して受理し focused が進む。
/// DOM は kind を知らずに、フォーカスは状態の focused、値は状態の 'A' へそろう。
#[test]
fn finding_10_uppercase_retype_normalizes_in_state_and_advances_focus() {
    let mut pin = PinInput::new(4, PinInputKind::AlphanumericUppercase);
    simulate_input(&mut pin, 0, 'a', &["a", "", "", ""]);
    assert_eq!(pin.digit(0), "A");
    let plan = simulate_input(&mut pin, 0, 'a', &["a", "", "", ""]);
    assert_eq!(pin.digit(0), "A");
    assert_eq!(pin.focused_index(), Some(1));
    assert_eq!(plan.focus, Some(1));
    assert_eq!(plan.value_writes, vec![(0, "A".to_string())]);
}

/// 指摘 11: 非連続桁（桁 3）を選んで入力した場合、"focus"(3) → "input" で桁 3 へ書かれ、
/// 先頭の空き桁（桁 0）へは書かれない。
#[test]
fn finding_11_non_contiguous_digit_receives_the_input() {
    let mut pin = PinInput::new(5, PinInputKind::Numeric);
    let plan = simulate_input(&mut pin, 3, '9', &["", "", "", "9", ""]);
    assert_eq!(pin.digit(3), "9");
    assert_eq!(pin.digit(0), "");
    assert_eq!(pin.focused_index(), Some(4));
    assert_eq!(plan.focus, Some(4));
}

/// 最終桁では focused が留まり、フォーカスも最終桁に留まる。
#[test]
fn last_digit_keeps_focus_on_last() {
    let mut pin = PinInput::new(3, PinInputKind::Numeric);
    let plan = simulate_input(&mut pin, 2, '4', &["", "", "4"]);
    assert_eq!(plan.focus, Some(2));
}

/// 値の書き込みは `input` のあった桁だけ。ネイティブ削除で DOM だけ空になった別の桁を状態から復活させない。
#[test]
fn value_writes_are_limited_to_the_typed_digit() {
    let mut pin = PinInput::new(4, PinInputKind::Numeric);
    for (i, c) in ['1', '2', '3'].into_iter().enumerate() {
        simulate_input(&mut pin, i, c, &["", "", "", ""]);
    }
    // 桁 2 は DOM 上でネイティブ Backspace により空（状態は '3' のまま）。桁 0 に '5' を入力。
    let plan = simulate_input(&mut pin, 0, '5', &["5", "2", "", ""]);
    assert_eq!(pin.digit(2), "3");
    assert!(plan.value_writes.iter().all(|(i, _)| *i == 0));
    // 桁 0 のライブ値が状態と同じなら書き込みなし。
    assert!(plan.value_writes.is_empty());
}
// ---- #3835: keydown・paste・削除・IME の対応付け ----

/// 配線が流す dispatch 列（`focus` → 本体）をそのまま状態機械へ流し、すべて名前契約どおり
/// 受理される（`dispatch` が `true`）ことを確認したうえで反映計画を作る。
fn simulate_actions(
    pin: &mut PinInput,
    actions: [ActionRef; 2],
    live: &[&str],
    scope: ValueScope,
) -> ReflectPlan {
    for action in actions {
        assert!(
            dispatch(pin, &action.action, &action.payload),
            "{} は decode_action の名前契約と一致すること",
            action.action
        );
    }
    let snapshot = snapshot_from_bound(0, pin.count(), host_read(pin, 0));
    let live: Vec<String> = live.iter().map(|s| (*s).to_string()).collect();
    reflect_plan_scoped(snapshot.as_ref(), &live, scope, None)
}

fn filled(kind: PinInputKind, count: usize, text: &str) -> PinInput {
    let mut pin = PinInput::new(count, kind);
    let empty = vec![""; count];
    for (i, c) in text.chars().enumerate() {
        simulate_input(&mut pin, i, c, &empty);
    }
    pin
}

#[test]
fn key_intent_truth_table() {
    let plain = |key: &str| key_intent(key, false, false, false, false, false, 0);
    assert_eq!(plain("Backspace"), Some(KeyIntent::Backspace));
    assert_eq!(plain("Delete"), Some(KeyIntent::Delete));
    assert_eq!(plain("ArrowLeft"), Some(KeyIntent::Prev));
    assert_eq!(plain("ArrowRight"), Some(KeyIntent::Next));
    for other in [
        "a",
        "Enter",
        "Tab",
        "Home",
        "End",
        "ArrowUp",
        "backspace",
        "",
    ] {
        assert_eq!(plain(other), None, "{other}");
    }
    // 修飾キー・変換中・keyCode 229 はネイティブに任せる。
    let k = |ctrl, alt, shift, meta, composing, code| {
        key_intent("Backspace", ctrl, alt, shift, meta, composing, code)
    };
    assert_eq!(k(true, false, false, false, false, 8), None);
    assert_eq!(k(false, true, false, false, false, 8), None);
    assert_eq!(k(false, false, true, false, false, 8), None);
    assert_eq!(k(false, false, false, true, false, 8), None);
    assert_eq!(k(false, false, false, false, true, 8), None);
    assert_eq!(k(false, false, false, false, false, 229), None);
}

#[test]
fn key_dispatches_send_focus_then_action_with_empty_payload() {
    for (intent, name) in [
        (KeyIntent::Backspace, "backspace"),
        (KeyIntent::Delete, "delete"),
        (KeyIntent::Prev, "prev"),
        (KeyIntent::Next, "next"),
    ] {
        let [focus, body] = key_dispatches(3, intent);
        assert_eq!(
            (focus.action.as_str(), focus.payload.as_str()),
            ("focus", "3")
        );
        assert_eq!((body.action.as_str(), body.payload.as_str()), (name, ""));
    }
}

#[test]
fn backspace_clears_the_digit_and_moves_focus_back() {
    let mut pin = filled(PinInputKind::Numeric, 4, "123");
    // 桁 2 へ DOM フォーカスがあり、ライブ値は '3'（keydown は preventDefault 済みで DOM は未変更）。
    let plan = simulate_actions(
        &mut pin,
        key_dispatches(2, KeyIntent::Backspace),
        &["1", "2", "3", ""],
        ValueScope::One(2),
    );
    assert_eq!(pin.digit(2), "");
    assert_eq!(plan.value_writes, vec![(2, String::new())]);
    assert_eq!(plan.focus, Some(1));
    // 先頭桁では留まる。
    let plan = simulate_actions(
        &mut pin,
        key_dispatches(0, KeyIntent::Backspace),
        &["1", "2", "", ""],
        ValueScope::One(0),
    );
    assert_eq!(pin.digit(0), "");
    assert_eq!(plan.focus, Some(0));
}

#[test]
fn delete_clears_the_digit_without_moving_focus() {
    let mut pin = filled(PinInputKind::Numeric, 4, "123");
    let plan = simulate_actions(
        &mut pin,
        key_dispatches(1, KeyIntent::Delete),
        &["1", "2", "3", ""],
        ValueScope::One(1),
    );
    assert_eq!(pin.digit(1), "");
    assert_eq!(plan.value_writes, vec![(1, String::new())]);
    assert_eq!(plan.focus, Some(1));
    assert_eq!(pin.digit(2), "3");
}

#[test]
fn arrows_move_focus_only_and_stop_at_the_ends() {
    let mut pin = filled(PinInputKind::Numeric, 3, "12");
    let live = ["1", "2", ""];
    let mut go = |index, intent| {
        simulate_actions(
            &mut pin,
            key_dispatches(index, intent),
            &live,
            ValueScope::None,
        )
    };
    let plan = go(1, KeyIntent::Prev);
    assert_eq!(
        plan,
        ReflectPlan {
            value_writes: Vec::new(),
            focus: Some(0)
        }
    );
    assert_eq!(go(0, KeyIntent::Prev).focus, Some(0));
    let plan = go(1, KeyIntent::Next);
    assert_eq!(
        plan,
        ReflectPlan {
            value_writes: Vec::new(),
            focus: Some(2)
        }
    );
    let plan = go(2, KeyIntent::Next);
    assert_eq!(plan.focus, Some(2));
    assert!(plan.value_writes.is_empty());
}

#[test]
fn paste_distributes_from_the_first_digit_regardless_of_origin() {
    // RFC 8628 ユーザーコード形（区切りは状態機械が除去）。桁 5 で貼り付けても先頭から充填される。
    let mut pin = PinInput::new(8, PinInputKind::AlphanumericUppercase);
    let text = "WDJB-MJHT";
    let [focus, body] = paste_dispatches(5, text);
    assert_eq!(body.payload, text, "DOM は無加工で渡す");
    assert_eq!(focus.payload, "5");
    let plan = simulate_actions(&mut pin, [focus, body], &[""; 8], ValueScope::All);
    assert_eq!(pin.value(), "WDJBMJHT");
    assert_eq!(pin.digit(0), "W");
    assert_eq!(pin.digit(7), "T");
    assert_eq!(plan.value_writes.len(), 8);
    assert_eq!(plan.value_writes[0], (0, "W".to_string()));

    let mut pin = PinInput::new(8, PinInputKind::Numeric);
    simulate_actions(
        &mut pin,
        paste_dispatches(0, "1234 5678"),
        &[""; 8],
        ValueScope::All,
    );
    assert_eq!(pin.value(), "12345678");

    // 小文字は大文字そろえ（状態機械の入力補助）で大文字になり、ライブ値との差だけ書く。
    let mut pin = PinInput::new(8, PinInputKind::AlphanumericUppercase);
    let live = ["w", "d", "j", "b", "m", "j", "h", "t"];
    let plan = simulate_actions(
        &mut pin,
        paste_dispatches(0, "wdjb-mjht"),
        &live,
        ValueScope::All,
    );
    assert_eq!(pin.value(), "WDJBMJHT");
    assert_eq!(plan.value_writes.len(), 8);
}

#[test]
fn rejected_paste_leaves_state_and_writes_only_differing_digits() {
    for bad in ["123456789", "12a4", "----", ""] {
        let mut pin = PinInput::new(4, PinInputKind::Numeric);
        let plan = simulate_actions(
            &mut pin,
            paste_dispatches(1, bad),
            &["", "", "", ""],
            ValueScope::All,
        );
        assert_eq!(pin.value(), "", "{bad:?}");
        assert!(plan.value_writes.is_empty(), "{bad:?}");
    }
}

#[test]
fn classify_paste_text_enforces_the_char_limit_only() {
    let at_limit = "9".repeat(MAX_PASTE_CHARS);
    assert_eq!(classify_paste_text(&at_limit), PasteText::Text(&at_limit));
    assert_eq!(classify_paste_text(""), PasteText::Text(""));
    let over = "9".repeat(MAX_PASTE_CHARS + 1);
    assert_eq!(classify_paste_text(&over), PasteText::Oversized);
    // 文字数で数える（バイト数ではない）。
    let multibyte = "あ".repeat(MAX_PASTE_CHARS);
    assert_eq!(classify_paste_text(&multibyte), PasteText::Text(&multibyte));
    assert_eq!(
        classify_paste_text(&"あ".repeat(MAX_PASTE_CHARS + 1)),
        PasteText::Oversized
    );
    assert_eq!(
        classify_paste_text(&"9".repeat(1_000_000)),
        PasteText::Oversized
    );
}

#[test]
fn deletion_dispatches_choose_backspace_only_for_backward_input_types() {
    for backward in [
        "deleteContentBackward",
        "deleteWordBackward",
        "deleteSoftLineBackward",
        "deleteHardLineBackward",
    ] {
        let [_, body] = deletion_dispatches(2, Some(backward));
        assert_eq!(body.action, "backspace", "{backward}");
    }
    for other in [
        Some("deleteContentForward"),
        Some("deleteByCut"),
        Some("historyUndo"),
        Some(""),
        None,
    ] {
        let [focus, body] = deletion_dispatches(2, other);
        assert_eq!(body.action, "delete", "{other:?}");
        assert_eq!(focus.payload, "2");
    }
}

#[test]
fn autofill_multi_char_value_fills_all_digits_via_paste() {
    let mut pin = PinInput::new(6, PinInputKind::Numeric);
    // 桁 0 の input.value が autofill で "123456" になった（ライブ値）。他の桁は空。
    let plan = simulate_actions(
        &mut pin,
        paste_dispatches(0, "123456"),
        &["123456", "", "", "", "", ""],
        ValueScope::All,
    );
    assert_eq!(pin.value(), "123456");
    assert_eq!(plan.value_writes.len(), 6);
    assert_eq!(plan.value_writes[0], (0, "1".to_string()));
}

#[test]
fn reflect_plan_scoped_scopes_and_legacy_form_agree() {
    let snapshot = PinInputSnapshot {
        focused: Some(1),
        digits: vec!["1".into(), String::new(), "3".into()],
    };
    let live: Vec<String> = vec!["x".into(), "y".into(), "3".into()];
    assert!(
        reflect_plan_scoped(Some(&snapshot), &live, ValueScope::None, None)
            .value_writes
            .is_empty()
    );
    assert_eq!(
        reflect_plan_scoped(Some(&snapshot), &live, ValueScope::One(1), None).value_writes,
        vec![(1, String::new())]
    );
    assert_eq!(
        reflect_plan_scoped(Some(&snapshot), &live, ValueScope::All, None).value_writes,
        vec![(0, "1".to_string()), (1, String::new())]
    );
    assert_eq!(
        reflect_plan(Some(&snapshot), &live, Some(0), None),
        reflect_plan_scoped(Some(&snapshot), &live, ValueScope::One(0), None)
    );
    assert_eq!(
        reflect_plan(Some(&snapshot), &live, None, None),
        reflect_plan_scoped(Some(&snapshot), &live, ValueScope::None, None)
    );
}
