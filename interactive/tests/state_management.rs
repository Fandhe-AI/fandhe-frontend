//! TASK-11.1c（#72、REQ-11）: `rws-interactive` の状態遷移・`dispatch` の
//! 本格網羅テスト。
//!
//! `interactive/src/lib.rs` 同梱の `#[cfg(test)] mod tests` は TASK-11.1b
//! （#71）のスモーク水準にとどまる（rustdoc の「スコープ外」節参照）。
//! 本ファイルは公開 API（`AppState` の各メソッド・`dispatch`）を対象に、
//! `wasm-full`/`wasm-thin`（TASK-11.2/11.3）がイベントハンドラ・JS グルー
//! から呼び出す際に前提とする「未知アクション・不正 payload で panic しない」
//! という契約（不変条件 4）を統合テストとして固定化する。
//!
//! 依存クレートは追加しない（REQ-3・`interactive/Cargo.toml` は
//! `rws-core`（path 依存）のみを維持する）。

use rws_interactive::{dispatch, Action, AppState, Component, Hydrate};

// --- AppState 既定値 -------------------------------------------------------

#[test]
fn new_and_default_produce_same_initial_state() {
    // `AppState::new()` は `Default` の薄いラッパー（lib.rs 参照）であり、
    // 両者が同一の既定状態を返すことを固定する。
    assert_eq!(AppState::new(), AppState::default());
}

#[test]
fn default_state_has_zero_counter_empty_draft_and_one_seed_item() {
    let s = AppState::default();
    assert_eq!(s.counter, 0);
    assert_eq!(s.draft, "");
    assert_eq!(s.items, vec!["最初の項目".to_string()]);
}

// --- increment / decrement / reset_counter --------------------------------

#[test]
fn increment_decrement_reset_sequence() {
    let mut s = AppState::new();
    s.update(Action::Increment);
    s.update(Action::Increment);
    s.update(Action::Increment);
    assert_eq!(s.counter, 3);
    s.update(Action::Decrement);
    assert_eq!(s.counter, 2);
    s.update(Action::Reset);
    assert_eq!(s.counter, 0);
}

#[test]
fn decrement_can_reach_negative_values() {
    // カウンターは正値専用ではない（UI 側に下限は設けられていない）。
    let mut s = AppState::new();
    s.update(Action::Decrement);
    s.update(Action::Decrement);
    assert_eq!(s.counter, -2);
}

#[test]
fn reset_counter_from_negative_value() {
    let mut s = AppState::new();
    s.update(Action::Decrement);
    s.update(Action::Decrement);
    s.update(Action::Reset);
    assert_eq!(s.counter, 0);
}

// --- set_draft / add_item ---------------------------------------------------

#[test]
fn set_draft_replaces_previous_value() {
    let mut s = AppState::new();
    s.update(Action::SetDraft("first".to_string()));
    s.update(Action::SetDraft("second".to_string()));
    assert_eq!(s.draft, "second");
}

#[test]
fn add_item_trims_ascii_whitespace_and_clears_draft() {
    let mut s = AppState::new();
    s.update(Action::SetDraft("  new item  ".to_string()));
    s.update(Action::AddItem);
    assert_eq!(s.items.last().unwrap(), "new item");
    assert_eq!(s.draft, "");
}

#[test]
fn add_item_ignores_empty_draft() {
    let mut s = AppState::new();
    let before = s.items.len();
    s.update(Action::SetDraft(String::new()));
    s.update(Action::AddItem);
    assert_eq!(s.items.len(), before);
}

#[test]
fn add_item_ignores_ascii_whitespace_only_draft() {
    let mut s = AppState::new();
    let before = s.items.clone();
    s.update(Action::SetDraft("   \t  ".to_string()));
    s.update(Action::AddItem);
    assert_eq!(s.items, before);
    // trim 失敗（空白のみ）の場合は draft をクリアしない仕様
    // （add_item の early return。lib.rs 参照）。
    assert_eq!(s.draft, "   \t  ");
}

#[test]
fn add_item_ignores_full_width_space_only_draft() {
    // Rust の `str::trim` は Unicode 空白を除去するため、全角空白（U+3000）
    // のみの draft も trim 後に空文字列となり、add_item に無視される。
    // 境界確認として明示的に固定する。
    let mut s = AppState::new();
    let before = s.items.len();
    s.update(Action::SetDraft("\u{3000}\u{3000}".to_string()));
    s.update(Action::AddItem);
    assert_eq!(s.items.len(), before);
}

#[test]
fn add_item_preserves_internal_whitespace() {
    let mut s = AppState::new();
    s.update(Action::SetDraft("  two words  ".to_string()));
    s.update(Action::AddItem);
    assert_eq!(s.items.last().unwrap(), "two words");
}

// --- remove_item -------------------------------------------------------------

#[test]
fn remove_item_removes_head() {
    let mut s = AppState::new();
    s.items = vec!["a".into(), "b".into(), "c".into()];
    s.update(Action::RemoveItem(0));
    assert_eq!(s.items, vec!["b".to_string(), "c".to_string()]);
}

#[test]
fn remove_item_removes_tail() {
    let mut s = AppState::new();
    s.items = vec!["a".into(), "b".into(), "c".into()];
    s.update(Action::RemoveItem(2));
    assert_eq!(s.items, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn remove_item_out_of_range_is_noop() {
    let mut s = AppState::new();
    s.items = vec!["a".into(), "b".into()];
    let before = s.items.clone();
    s.update(Action::RemoveItem(5));
    assert_eq!(s.items, before);
}

#[test]
fn remove_item_on_empty_list_is_noop_and_does_not_panic() {
    let mut s = AppState::new();
    s.items.clear();
    // 空リストに対する remove_item は範囲外呼び出しと同様 no-op であり
    // panic しないこと（不変条件 4 相当の安全側フォールバック）を確認する。
    s.update(Action::RemoveItem(0));
    assert!(s.items.is_empty());
}

// --- dispatch: 全アクションの網羅 -------------------------------------------

#[test]
fn dispatch_increment() {
    let mut s = AppState::new();
    dispatch(&mut s, "increment", "");
    assert_eq!(s.counter, 1);
}

#[test]
fn dispatch_decrement() {
    let mut s = AppState::new();
    dispatch(&mut s, "decrement", "");
    assert_eq!(s.counter, -1);
}

#[test]
fn dispatch_reset() {
    let mut s = AppState::new();
    s.update(Action::Increment);
    s.update(Action::Increment);
    dispatch(&mut s, "reset", "");
    assert_eq!(s.counter, 0);
}

#[test]
fn dispatch_set_draft_uses_payload() {
    let mut s = AppState::new();
    dispatch(&mut s, "set_draft", "hello world");
    assert_eq!(s.draft, "hello world");
}

#[test]
fn dispatch_add_item_ignores_payload_and_uses_current_draft() {
    // `add_item` アクションは payload を使わず、現在の draft を確定する
    // （lib.rs の dispatch 実装参照）。payload を渡しても無視されることを固定する。
    let mut s = AppState::new();
    s.update(Action::SetDraft("from draft".to_string()));
    dispatch(&mut s, "add_item", "ignored payload");
    assert_eq!(s.items.last().unwrap(), "from draft");
}

#[test]
fn dispatch_remove_item_parses_payload_as_index() {
    let mut s = AppState::new();
    s.items = vec!["a".into(), "b".into()];
    dispatch(&mut s, "remove_item", "0");
    assert_eq!(s.items, vec!["b".to_string()]);
}

#[test]
fn dispatch_full_action_sequence() {
    // 一連のユーザー操作を dispatch 経由で再現し、最終状態を確認する。
    let mut s = AppState::new();
    dispatch(&mut s, "increment", "");
    dispatch(&mut s, "increment", "");
    dispatch(&mut s, "decrement", "");
    dispatch(&mut s, "set_draft", "  task  ");
    dispatch(&mut s, "add_item", "");
    dispatch(&mut s, "remove_item", "0");

    assert_eq!(s.counter, 1);
    assert_eq!(s.draft, "");
    // 初期項目（先頭）を remove_item("0") で消し、新規追加分のみ残る。
    assert_eq!(s.items, vec!["task".to_string()]);
}

// --- dispatch の防御的動作（クライアント制御下の入力に対する DoS 耐性） ---

#[test]
fn dispatch_ignores_unknown_action_without_panicking() {
    let mut s = AppState::new();
    let before = s.clone();
    dispatch(&mut s, "no_such_action", "payload");
    assert_eq!(s, before);
}

#[test]
fn dispatch_ignores_empty_action_name() {
    let mut s = AppState::new();
    let before = s.clone();
    dispatch(&mut s, "", "");
    assert_eq!(s, before);
}

#[test]
fn dispatch_remove_item_with_non_numeric_payload_is_noop() {
    let mut s = AppState::new();
    let before = s.items.clone();
    dispatch(&mut s, "remove_item", "not-a-number");
    assert_eq!(s.items, before);
}

#[test]
fn dispatch_remove_item_with_negative_payload_is_noop() {
    // `usize` へのパースが失敗する（符号付き文字列は usize として不正）ため
    // no-op になる。
    let mut s = AppState::new();
    let before = s.items.clone();
    dispatch(&mut s, "remove_item", "-1");
    assert_eq!(s.items, before);
}

#[test]
fn dispatch_remove_item_with_empty_payload_is_noop() {
    let mut s = AppState::new();
    let before = s.items.clone();
    dispatch(&mut s, "remove_item", "");
    assert_eq!(s.items, before);
}

#[test]
fn dispatch_remove_item_with_overflowing_numeric_string_does_not_panic() {
    // `usize::MAX` を超える巨大な数値文字列は parse::<usize>() が Err を返す
    // ため、no-op になる（panic しないことがこのテストの主眼）。
    let mut s = AppState::new();
    let before = s.items.clone();
    let huge = "999999999999999999999999999999999999999999";
    dispatch(&mut s, "remove_item", huge);
    assert_eq!(s.items, before);
}

#[test]
fn dispatch_remove_item_with_usize_max_is_noop_when_out_of_range() {
    // usize::MAX 自体は正当にパースできる値だが、通常の items 長より
    // 大きいため範囲外 no-op になる。
    let mut s = AppState::new();
    let before = s.items.clone();
    dispatch(&mut s, "remove_item", &usize::MAX.to_string());
    assert_eq!(s.items, before);
}

// --- counter 境界（ハイドレーション経由の極端な復元値、dispatch 経由） ------
//
// `AppState::increment`/`decrement` は `saturating_add`/`saturating_sub` を
// 用いており、`i64::MAX` を `increment` する・`i64::MIN` を `decrement` する
// 呼び出しでも overflow panic しない（本クレートの不変条件 4、DoS 耐性）。
// クライアント制御下のハイドレーション属性値経由で極端な counter 値が
// 復元されるケースは `hydration_codec.rs` 側で確認済みであり、本ファイルは
// `dispatch` 経由での極端値からの状態遷移が panic しないことを固定する。

#[test]
fn dispatch_increment_at_i64_max_saturates_without_panicking() {
    let mut s = AppState {
        counter: i64::MAX,
        ..AppState::new()
    };
    dispatch(&mut s, "increment", "");
    assert_eq!(s.counter, i64::MAX);
}

#[test]
fn dispatch_decrement_at_i64_min_saturates_without_panicking() {
    let mut s = AppState {
        counter: i64::MIN,
        ..AppState::new()
    };
    dispatch(&mut s, "decrement", "");
    assert_eq!(s.counter, i64::MIN);
}

#[test]
fn reset_counter_recovers_from_extreme_restored_value() {
    // 極端な値からの回復手段として reset_counter が overflow を経由せず
    // 常に 0 へ戻せることを確認する（decrement/increment を挟まない限り
    // 安全に使えるフォールバック操作であることの回帰）。
    let mut restored = AppState {
        counter: i64::MAX,
        ..AppState::new()
    };
    restored.update(Action::Reset);
    assert_eq!(restored.counter, 0);

    let mut restored_min = AppState {
        counter: i64::MIN,
        ..AppState::new()
    };
    restored_min.update(Action::Reset);
    assert_eq!(restored_min.counter, 0);
}

// --- hydration_attrs の Hydrate トレイト経由呼び出し（TASK-11.1a 追従確認） -

#[test]
fn hydrate_trait_import_allows_method_call_on_app_state() {
    // `use rws_interactive::Hydrate` により、トレイトメソッドとして
    // `hydration_attrs`/`from_hydration_attrs` を呼べることを確認する
    // （具象の自由関数ではなくトレイト経由の呼び出しへ移行した契約）。
    let s = AppState::new();
    let attrs = s.hydration_attrs();
    let restored = AppState::from_hydration_attrs(&attrs).unwrap();
    assert_eq!(s, restored);
}
