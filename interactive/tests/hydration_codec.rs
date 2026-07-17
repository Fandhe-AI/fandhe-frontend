//! TASK-11.1c（#72、REQ-11）: ハイドレーション属性コーデックの往復・
//! 敵対的入力耐性テスト。
//!
//! [`Hydrate::hydration_attrs`]（サーバー側: 状態 → 属性）と
//! [`Hydrate::from_hydration_attrs`]（クライアント側: 属性 → 状態）は、
//! `wasm-full`/`wasm-thin`（TASK-11.2/11.3）が DOM から読み取った属性値を
//! 復元する際の唯一の契約点である（`docs/interactive-api.md` 第 3 節・
//! TASK-11.1a で確定した凍結 API）。属性値はクライアント制御下になり得る
//! （DevTools 等での改ざんを含む）ため、`from_hydration_attrs` は panic
//! せず `Result` で失敗を返す契約になっている（本クレートの不変条件 3）。
//! 本ファイルは公開トレイト経由で「往復の正しさ」と「不正入力に対する
//! panic 耐性（Err を返して安全に失敗する）」の双方を固定する。
//! `codec::encode_list`/`decode_list` は `codec` モジュール内の単体テストで
//! 別途カバーする（`lib.rs` 参照）。

use rws_interactive::{Action, AppState, Component, Hydrate, HydrateError};
use std::collections::HashMap;

/// [`Hydrate::hydration_attrs`] の戻り値を `HashMap` へ変換し、キー名でアクセスできるようにする。
fn attrs_map(state: &AppState) -> HashMap<String, String> {
    state.hydration_attrs().into_iter().collect()
}

/// `state.hydration_attrs()` → `AppState::from_hydration_attrs` の往復ヘルパー。
/// ラウンドトリップが必ず成功する前提のテストでのみ使用する。
fn roundtrip(state: &AppState) -> AppState {
    let attrs = state.hydration_attrs();
    AppState::from_hydration_attrs(&attrs).expect("roundtrip should succeed")
}

// --- 通常状態の往復 -----------------------------------------------------------

#[test]
fn roundtrip_preserves_default_state() {
    let s = AppState::new();
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_preserves_counter_draft_and_items() {
    let mut s = AppState::new();
    s.counter = 42;
    s.draft = "draft text".to_string();
    s.items = vec!["one".into(), "two".into(), "three".into()];
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_preserves_negative_counter() {
    let mut s = AppState::new();
    s.counter = -7;
    assert_eq!(roundtrip(&s), s);
}

// --- 区切り文字・エスケープ文字混在項目の往復（データ注入不能の回帰） -------

#[test]
fn roundtrip_survives_item_containing_separator_char() {
    let mut s = AppState::new();
    s.items = vec!["before\u{1f}after".to_string()];
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_survives_item_containing_backslash() {
    let mut s = AppState::new();
    s.items = vec!["path\\to\\file".to_string()];
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_survives_item_with_separator_and_backslash_mixed() {
    let mut s = AppState::new();
    s.items = vec![
        "a\u{1f}b\\c".to_string(),
        "\\\u{1f}\\\u{1f}".to_string(),
        "plain".to_string(),
    ];
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_does_not_let_separator_forge_extra_item_boundary() {
    // 区切り文字を含む 1 項目が、デコード後に複数項目へ分裂しない
    // （項目境界の偽装＝データ注入ができないことの直接確認）。
    let mut s = AppState::new();
    s.items = vec!["fake\u{1f}boundary\u{1f}injection".to_string()];
    let restored = roundtrip(&s);
    assert_eq!(restored.items.len(), 1);
    assert_eq!(restored.items[0], "fake\u{1f}boundary\u{1f}injection");
}

// --- 空リスト vs 空文字列 1 件のリスト（エンコード衝突の回帰） -------------

#[test]
fn empty_list_and_single_empty_item_list_are_distinguishable() {
    let mut empty = AppState::new();
    empty.items = Vec::new();

    let mut single_empty = AppState::new();
    single_empty.items = vec!["".to_string()];

    let empty_attrs = attrs_map(&empty);
    let single_attrs = attrs_map(&single_empty);
    assert_ne!(
        empty_attrs["data-hydrate-items"],
        single_attrs["data-hydrate-items"]
    );

    assert_eq!(roundtrip(&empty).items, Vec::<String>::new());
    assert_eq!(roundtrip(&single_empty).items, vec!["".to_string()]);
}

// --- マルチバイト・制御文字・HTML メタ文字を含む項目の往復 -----------------

#[test]
fn roundtrip_survives_japanese_and_emoji_items() {
    let mut s = AppState::new();
    s.items = vec!["こんにちは世界".to_string(), "🎉絵文字テスト🎉".to_string()];
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_survives_newlines_tabs_and_quotes() {
    let mut s = AppState::new();
    s.items = vec!["line1\nline2\ttabbed \"quoted\" 'single'".to_string()];
    s.draft = "draft\nwith\nnewlines".to_string();
    assert_eq!(roundtrip(&s), s);
}

#[test]
fn roundtrip_survives_html_meta_characters_in_items() {
    // items のエスケープはコーデックの責務であり、HTML エスケープは
    // render 側（rws_core::render）の責務。ここでは「コーデックが
    // メタ文字を変質させずに往復させる」ことのみを確認する
    // （HTML エスケープの検証は xss_escape.rs の役割）。
    let mut s = AppState::new();
    s.items = vec!["<script>alert(1)</script>".to_string(), "a & b".to_string()];
    assert_eq!(roundtrip(&s), s);
}

// --- from_hydration_attrs の敵対的入力耐性（panic せず Err を返す） --------
//
// `docs/interactive-api.md` 第 4 節・判断 4: `from_hydration_attrs` は
// 改ざんされうるクライアント入力を前提に `Result` を返し、`unwrap()`/
// `panic!` を使わない。フォールバック戦略（既定値に倒すかエラーを
// 伝播するか）は呼び出し側（`rws-wasm-full`）に委ねる設計であるため、
// 本クレート自身は「値へフォールバックする」のではなく「Err を返して
// panic しない」ことを保証する。

fn attrs_with(counter: &str, draft: &str, items: &str) -> Vec<(String, String)> {
    vec![
        ("data-hydrate-counter".to_string(), counter.to_string()),
        ("data-hydrate-draft".to_string(), draft.to_string()),
        ("data-hydrate-items".to_string(), items.to_string()),
    ]
}

#[test]
fn from_hydration_attrs_returns_err_on_non_numeric_counter() {
    let attrs = attrs_with("not-a-number", "", "");
    let err = AppState::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn from_hydration_attrs_returns_err_on_empty_counter() {
    let attrs = attrs_with("", "", "");
    let err = AppState::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn from_hydration_attrs_returns_err_on_huge_counter_string() {
    // i64 の範囲を超える巨大な数値文字列はパース失敗として Err を返す。
    let huge = "99999999999999999999999999999999999999";
    let attrs = attrs_with(huge, "", "");
    let err = AppState::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn from_hydration_attrs_returns_err_on_missing_attr() {
    let err = AppState::from_hydration_attrs(&[]).unwrap_err();
    assert!(matches!(err, HydrateError::MissingAttr(_)));
}

#[test]
fn from_hydration_attrs_does_not_panic_on_trailing_incomplete_escape() {
    // items 文字列の末尾がエスケープ文字で切れている（次の 1 文字が
    // 存在しない）敵対的入力。codec::decode_list は None ケースでバック
    // スラッシュ単体を残す安全側フォールバックを持つ（lib.rs 参照）。
    // ここでは公開 API 経由で panic しないことのみを確認する。
    let truncated_escape = "\u{1f}broken\\";
    let attrs = attrs_with("0", "", truncated_escape);
    let restored = AppState::from_hydration_attrs(&attrs).expect("should not panic nor error");
    assert_eq!(restored.items.len(), 1);
}

#[test]
fn from_hydration_attrs_does_not_panic_on_unknown_escape_sequence() {
    // `\` の直後が既知のエスケープ対象（`\`/`u`）でない場合。
    let unknown_escape = "\u{1f}item\\x";
    let attrs = attrs_with("0", "", unknown_escape);
    let restored = AppState::from_hydration_attrs(&attrs).expect("should not panic nor error");
    assert_eq!(restored.items.len(), 1);
}

#[test]
fn from_hydration_attrs_does_not_panic_on_control_characters() {
    // 制御文字だらけの items 文字列。区切り文字自体も大量に含む。
    let control_heavy: String = "\u{1f}".repeat(50);
    let attrs = attrs_with("0", "", &control_heavy);
    let restored = AppState::from_hydration_attrs(&attrs).expect("should not panic nor error");
    // 50 個の区切り文字はすべて「前置区切り」として扱われ、50 個の
    // 空文字列項目に復元される（panic しないことが主眼）。
    assert_eq!(restored.items.len(), 50);
    assert!(restored.items.iter().all(|item| item.is_empty()));
}

#[test]
fn from_hydration_attrs_does_not_panic_on_long_input() {
    // 長大入力（DoS 耐性の素朴な確認。単純に処理が完了し panic しないこと）。
    let long_item = "x".repeat(100_000);
    let items_joined = format!("\u{1f}{long_item}");
    let attrs = attrs_with("0", "", &items_joined);
    let restored = AppState::from_hydration_attrs(&attrs).expect("should not panic nor error");
    assert_eq!(restored.items, vec![long_item]);
}

// --- 極端なカウンター値の往復（saturating_add/sub との組み合わせ回帰） -----

#[test]
fn from_hydration_attrs_restores_i64_max_counter() {
    let attrs = attrs_with(&i64::MAX.to_string(), "", "");
    let restored = AppState::from_hydration_attrs(&attrs).expect("i64::MAX is a valid i64");
    assert_eq!(restored.counter, i64::MAX);
}

#[test]
fn from_hydration_attrs_restores_i64_min_counter() {
    let attrs = attrs_with(&i64::MIN.to_string(), "", "");
    let restored = AppState::from_hydration_attrs(&attrs).expect("i64::MIN is a valid i64");
    assert_eq!(restored.counter, i64::MIN);
}

#[test]
fn increment_from_extreme_restored_max_value_does_not_panic() {
    // ハイドレーション経由で復元された i64::MAX に対して increment しても
    // saturating_add により overflow panic しないことを固定する
    // （state_management.rs 側の dispatch 経由テストと対をなす回帰）。
    let attrs = attrs_with(&i64::MAX.to_string(), "", "");
    let mut restored = AppState::from_hydration_attrs(&attrs).unwrap();
    restored.update(Action::Increment);
    assert_eq!(restored.counter, i64::MAX);
}

#[test]
fn decrement_from_extreme_restored_min_value_does_not_panic() {
    let attrs = attrs_with(&i64::MIN.to_string(), "", "");
    let mut restored = AppState::from_hydration_attrs(&attrs).unwrap();
    restored.update(Action::Decrement);
    assert_eq!(restored.counter, i64::MIN);
}

#[test]
fn reset_counter_recovers_from_extreme_restored_value() {
    let attrs_max = attrs_with(&i64::MAX.to_string(), "", "");
    let mut restored = AppState::from_hydration_attrs(&attrs_max).unwrap();
    restored.update(Action::Reset);
    assert_eq!(restored.counter, 0);

    let attrs_min = attrs_with(&i64::MIN.to_string(), "", "");
    let mut restored_min = AppState::from_hydration_attrs(&attrs_min).unwrap();
    restored_min.update(Action::Reset);
    assert_eq!(restored_min.counter, 0);
}

// --- hydration_attrs のキー集合固定（wasm-full/TASK-11.4 との契約） --------

#[test]
fn hydration_attrs_key_set_is_fixed_to_three_entries() {
    let s = AppState::new();
    let attrs = s.hydration_attrs();
    let mut keys: Vec<&str> = attrs.iter().map(|(k, _)| k.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "data-hydrate-counter",
            "data-hydrate-draft",
            "data-hydrate-items",
        ]
    );
}
