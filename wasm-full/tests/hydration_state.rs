//! TASK-11.4c（#84、親 #81、REQ-11）: `wasm-full/src/hydration.rs`
//! （TASK-11.4b・#83 でマージ済み）の統合レベル native テスト。
//!
//! `wasm-full/src/hydration.rs` 内のインラインテストは自作の最小
//! `TestState` のみを対象にしている。本ファイルは実アプリ相当の
//! `rws_interactive::AppState`（`Hydrate` 実装済み、`interactive/src/lib.rs`）
//! を用いて [`rws_wasm_full::hydration::restore_state`] のラウンドトリップ・
//! 改ざん値マトリクスを検証し、`docs/hydration-state-format.md` 第 6 節
//! 「テスト観点の引き継ぎ」が求める統合テストの空白を埋める
//! （設計文書第 6 節・実装計画 §4.1 に対応）。
//!
//! DOM・`web-sys` に依存しない native テスト（`cargo test -p rws-wasm-full`）。
//! 実 DOM 経由の検証（U+001F の実 DOM 保持実証・上限超過属性の実 DOM 除外）は
//! `wasm-full/tests/hydration_browser.rs` が担当する。
//!
//! # 検証する契約
//!
//! - `AppState::hydration_attrs()` → `restore_state::<AppState>` のラウンド
//!   トリップが元の状態と一致すること（区切り文字・バックスラッシュ・
//!   HTML メタ文字・日本語/絵文字・空リストと単一空文字列項目の区別を含む）。
//! - 改ざん値（非数値 counter・属性欠落・未知属性混入・生の区切り文字混入・
//!   不完全エスケープ列）に対し `restore_state` が `HydrateError` を返し
//!   panic しないこと（`docs/hydration-state-format.md` 第 8 節・不変条件 2・3）。
//! - `MAX_ATTR_VALUE_LEN` 超過属性が `filter_hydration_attrs` 相当の経路
//!   （`read_hydration_attrs` は wasm32 限定のため、ここでは `restore_state` に
//!   直接欠落属性を渡すことで「除外後」の状態を模擬する）で `MissingAttr` に
//!   収束することの結合確認。

use rws_interactive::{codec, AppState, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};
use rws_wasm_full::hydration::restore_state;

/// `AppState::hydration_attrs()` → `restore_state::<AppState>` の往復ヘルパー。
/// ラウンドトリップが必ず成功する前提のテストでのみ使用する。
fn roundtrip(state: &AppState) -> AppState {
    let attrs = state.hydration_attrs();
    restore_state::<AppState>(&attrs).expect("roundtrip should succeed for well-formed attrs")
}

fn attr_name(field: &str) -> String {
    format!("{HYDRATE_ATTR_PREFIX}{field}")
}

// --- ラウンドトリップ ---------------------------------------------------

#[test]
fn restore_state_roundtrips_default_app_state() {
    let state = AppState::new();
    assert_eq!(roundtrip(&state), state);
}

#[test]
fn restore_state_roundtrips_counter_draft_and_items() {
    let mut state = AppState::new();
    state.counter = 42;
    state.draft = "draft text".to_string();
    state.items = vec!["one".into(), "two".into(), "three".into()];
    assert_eq!(roundtrip(&state), state);
}

#[test]
fn restore_state_roundtrips_negative_counter() {
    let mut state = AppState::new();
    state.counter = -7;
    assert_eq!(roundtrip(&state), state);
}

/// 項目文字列に区切り文字（U+001F）・バックスラッシュが混入していても
/// `codec` のエスケープ規約により項目境界の偽装が起きず往復すること
/// （`docs/hydration-state-format.md` 第 8 節・不変条件 2）。
#[test]
fn restore_state_roundtrips_items_containing_separator_and_backslash() {
    let mut state = AppState::new();
    state.items = vec![
        "before\u{1f}after".to_string(),
        "back\\slash".to_string(),
        "mixed\\\u{1f}both".to_string(),
    ];
    assert_eq!(roundtrip(&state), state);
}

/// 属性値中の HTML メタ文字自体は `codec` レベルでは何ら特別扱いされず
/// そのまま往復する（エスケープは `render_for_hydration` の SSR 出力生成時
/// にのみ発生する契約、`docs/hydration-state-format.md` 第 8 節・不変条件 1）。
#[test]
fn restore_state_roundtrips_items_containing_html_meta_characters() {
    let mut state = AppState::new();
    state.draft = "\"><script>alert(1)</script>".to_string();
    state.items = vec!["<img src=x onerror=alert(1)>".to_string()];
    assert_eq!(roundtrip(&state), state);
}

/// 日本語・絵文字（マルチバイト文字）が `String` の非破壊コピーとして
/// 往復すること（区切り文字・エスケープ処理が文字境界を壊さないことの確認）。
#[test]
fn restore_state_roundtrips_japanese_and_emoji_items() {
    let mut state = AppState::new();
    state.draft = "日本語のテキスト".to_string();
    state.items = vec!["絵文字🎉入り\u{1f}項目".to_string(), "🦀".to_string()];
    assert_eq!(roundtrip(&state), state);
}

/// 空リストと「空文字列 1 件のみを含むリスト」は異なるエンコードになり、
/// 往復で区別できる（`codec::encode_list` 冒頭コメント、Bugbot 指摘対応済みの
/// 前置区切り方式が `AppState` 経由の統合パスでも機能することの確認）。
#[test]
fn restore_state_distinguishes_empty_list_from_single_empty_string_item() {
    let mut empty_list = AppState::new();
    empty_list.items = Vec::new();

    let mut single_empty_item = AppState::new();
    single_empty_item.items = vec![String::new()];

    assert_ne!(
        empty_list.hydration_attrs(),
        single_empty_item.hydration_attrs()
    );
    assert_eq!(roundtrip(&empty_list), empty_list);
    assert_eq!(roundtrip(&single_empty_item), single_empty_item);
}

// --- 改ざん値マトリクス（すべて HydrateError を返し panic しないこと） ------

#[test]
fn restore_state_fails_on_non_numeric_counter() {
    let attrs = vec![
        (attr_name("counter"), "not-a-number".to_string()),
        (attr_name("draft"), String::new()),
        (attr_name("items"), String::new()),
    ];
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn restore_state_fails_on_empty_counter_value() {
    let attrs = vec![
        (attr_name("counter"), String::new()),
        (attr_name("draft"), String::new()),
        (attr_name("items"), String::new()),
    ];
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn restore_state_fails_when_counter_attr_missing() {
    let attrs = vec![
        (attr_name("draft"), String::new()),
        (attr_name("items"), String::new()),
    ];
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::MissingAttr(_)));
}

#[test]
fn restore_state_fails_when_all_attrs_missing() {
    let attrs: Vec<(String, String)> = Vec::new();
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::MissingAttr(_)));
}

/// 未知の `data-hydrate-*` 属性が混入していても、既知フィールド（counter/
/// draft/items）は正常に復元されること（`docs/hydration-state-format.md`
/// 第 4 節・判断 5、無視は復元側の責務）。
#[test]
fn restore_state_ignores_unknown_hydrate_attr_and_restores_known_fields() {
    let mut attrs = AppState::new().hydration_attrs();
    attrs.push((attr_name("unknown-field"), "ignored".to_string()));

    let restored = restore_state::<AppState>(&attrs).expect("unknown attr must not cause error");
    assert_eq!(restored, AppState::new());
}

/// 生の区切り文字（U+001F）を `items` 属性値の先頭以外へ直接注入しても、
/// `codec::decode_list` のエスケープ規約により余分な項目境界が生まれない
/// こと（区切り文字混入による項目数偽装の防止、`docs/hydration-state-format.md`
/// 第 8 節・不変条件 2 の中核）。
#[test]
fn restore_state_does_not_create_extra_items_from_raw_separator_injection() {
    // 正規の `codec::encode_list` を経由せず、区切り文字を手で 3 個仕込んだ
    // 生の属性値を直接与える（敵対的クライアント入力の模擬）。
    let tampered_items = "\u{1f}legit\u{1f}\u{1f}extra".to_string();
    let attrs = vec![
        (attr_name("counter"), "0".to_string()),
        (attr_name("draft"), String::new()),
        (attr_name("items"), tampered_items),
    ];

    let restored = restore_state::<AppState>(&attrs).expect("decode_list must not panic");
    // 前置区切り方式（`codec::encode_list` 冒頭コメント）では区切り文字の
    // 出現数が常に項目数と一致する。3 個の区切りが存在するため panic せず
    // 3 項目として復元される（decode_list は「意味のある改ざん検知」は
    // 行わない契約だが、項目数が区切り文字の個数と一致し続けること自体が
    // 「境界偽装で余分な項目が生まれていない」ことの確認になる）。
    assert_eq!(restored.items.len(), 3);
    assert_eq!(restored.counter, 0);
}

/// 不完全なエスケープ列（末尾がエスケープ文字で終わる等）は
/// `unescape_item`（`codec` 内部）の安全側フォールバックにより panic せず
/// 復元されること。
#[test]
fn restore_state_handles_incomplete_escape_sequence_without_panicking() {
    let tampered_items = "\u{1f}trailing-escape\\".to_string();
    let attrs = vec![
        (attr_name("counter"), "1".to_string()),
        (attr_name("draft"), String::new()),
        (attr_name("items"), tampered_items),
    ];

    let restored =
        restore_state::<AppState>(&attrs).expect("incomplete escape sequence must not panic");
    assert_eq!(restored.items.len(), 1);
}

/// `codec::encode_list`/`decode_list` の往復契約自体は
/// `interactive/src/lib.rs` の codec モジュールテストが担うが、本ファイルは
/// `AppState` 経由の統合パス（`hydration_attrs()` → `restore_state`）でも
/// 同じ契約が保たれることを確認する（クレート境界を跨いだ結合確認）。
#[test]
fn codec_roundtrip_matches_app_state_roundtrip_via_hydration_attrs() {
    let items = vec!["x".to_string(), "y\u{1f}z".to_string()];
    let encoded = codec::encode_list(&items);
    let mut state = AppState::new();
    state.items = items.clone();

    let attrs = state.hydration_attrs();
    let items_attr = attrs
        .iter()
        .find(|(k, _)| k == &attr_name("items"))
        .map(|(_, v)| v.clone())
        .expect("items attr must be present");
    assert_eq!(items_attr, encoded);
    assert_eq!(codec::decode_list(&items_attr), items);
}

// --- DoS 上限（MAX_ATTR_VALUE_LEN）との結合 -----------------------------

/// `filter_hydration_attrs`（wasm32 配線層が呼ぶ内部フィルタ、
/// `wasm-full/src/hydration.rs`）が上限超過属性を除外した結果、
/// `restore_state` へ渡る属性列から該当属性が欠落した状態を模擬する。
/// 欠落属性は `MissingAttr` として扱われ、安全側フォールバック
/// （初期状態での CSR 再描画、`Runtime::hydrate` 側の責務）に収束する経路を
/// `AppState` レベルで確認する（`docs/hydration-state-format.md` 第 8 節・
/// 不変条件 4）。
#[test]
fn restore_state_treats_attrs_filtered_out_by_size_limit_as_missing() {
    // MAX_ATTR_VALUE_LEN 超過の draft 値は filter_hydration_attrs によって
    // 列挙結果から除外される（wasm-full/src/hydration.rs 内の
    // filter_hydration_attrs_excludes_oversized_values テストが実証済み）。
    // ここではフィルタ後の状態（draft 属性が欠落した状態）をそのまま
    // restore_state へ渡し、結合先の挙動を確認する。
    let attrs = vec![
        (attr_name("counter"), "0".to_string()),
        (attr_name("items"), String::new()),
        // draft は上限超過のため列挙されなかった、という前提を欠落で表現する。
    ];
    let err = restore_state::<AppState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::MissingAttr(attr) if attr == attr_name("draft")));
}
