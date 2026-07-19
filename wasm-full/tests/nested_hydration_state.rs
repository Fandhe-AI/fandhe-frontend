//! イシュー #163: ハイドレーション状態注入のネスト構造対応（`rws_interactive::codec::Value`）
//! を用いた `restore_state` 統合テスト。
//!
//! `wasm-full/tests/hydration_state.rs` は既存の凍結フォーマット（数値・文字列・
//! 文字列配列のみ）を対象とする `AppState` の統合テストであり、本ファイルは
//! ネスト構造（オブジェクト・マップ・リストの入れ子）を持つ独自の `Hydrate`
//! 実装（`NestedState`）に対して同水準の検証（ラウンドトリップ・改ざん値・
//! panic-free）を行う。フォーマット自体の設計根拠は
//! `docs/design/hydration-nested-state.md` を正とする。
//!
//! DOM・`web-sys` に依存しない native テスト（`cargo test -p rws-wasm-full`）。

use rws_interactive::codec::{self, Value};
use rws_interactive::{Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};
use rws_wasm_full::hydration::restore_state;

/// ネストしたオブジェクト（`user: { name, tags: [...] }`）・トップレベルの
/// リストを含む、複雑な状態を表すテスト用アプリ状態。
///
/// `hydration_attrs`/`from_hydration_attrs` は `codec::Value` を経由して
/// 1 属性値へネスト構造をエンコード/デコードする（イシュー #163 が引き継いだ
/// 「ネスト構造等の複雑な状態」への一般化の実装例）。
#[derive(Debug, Clone, PartialEq)]
struct NestedState {
    user: UserProfile,
    counters: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq)]
struct UserProfile {
    name: String,
    tags: Vec<String>,
    active: bool,
}

impl UserProfile {
    fn to_value(&self) -> Value {
        Value::Map(vec![
            ("name".to_string(), Value::Str(self.name.clone())),
            (
                "tags".to_string(),
                Value::List(self.tags.iter().cloned().map(Value::Str).collect()),
            ),
            ("active".to_string(), Value::Bool(self.active)),
        ])
    }

    fn from_value(value: &Value) -> Result<Self, String> {
        let Value::Map(entries) = value else {
            return Err("user must be a map".to_string());
        };
        let find = |key: &str| entries.iter().find(|(k, _)| k == key).map(|(_, v)| v);

        let name = match find("name") {
            Some(Value::Str(s)) => s.clone(),
            _ => return Err("user.name must be a string".to_string()),
        };
        let tags = match find("tags") {
            Some(Value::List(items)) => items
                .iter()
                .map(|v| match v {
                    Value::Str(s) => Ok(s.clone()),
                    _ => Err("user.tags items must be strings".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err("user.tags must be a list".to_string()),
        };
        let active = match find("active") {
            Some(Value::Bool(b)) => *b,
            _ => return Err("user.active must be a bool".to_string()),
        };

        Ok(UserProfile { name, tags, active })
    }
}

impl Hydrate for NestedState {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let root = Value::Map(vec![
            ("user".to_string(), self.user.to_value()),
            (
                "counters".to_string(),
                Value::List(self.counters.iter().copied().map(Value::Int).collect()),
            ),
        ]);
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}state"),
            codec::encode_value(&root),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}state");
        let raw = attrs
            .iter()
            .find(|(k, _)| k == &attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;

        let root = codec::decode_value(raw).map_err(|e| e.into_hydrate_error(&attr_name))?;
        let Value::Map(entries) = &root else {
            return Err(HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "root value must be a map".to_string(),
            });
        };
        let find = |key: &str| entries.iter().find(|(k, _)| k == key).map(|(_, v)| v);

        let user = find("user")
            .ok_or_else(|| HydrateError::MissingAttr(format!("{attr_name}.user")))
            .and_then(|v| {
                UserProfile::from_value(v).map_err(|reason| HydrateError::InvalidValue {
                    attr: attr_name.clone(),
                    reason,
                })
            })?;
        let counters = match find("counters") {
            Some(Value::List(items)) => items
                .iter()
                .map(|v| match v {
                    Value::Int(i) => Ok(*i),
                    _ => Err(HydrateError::InvalidValue {
                        attr: attr_name.clone(),
                        reason: "counters items must be integers".to_string(),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name.clone(),
                    reason: "counters must be a list".to_string(),
                })
            }
        };

        Ok(NestedState { user, counters })
    }
}

fn sample_state() -> NestedState {
    NestedState {
        user: UserProfile {
            name: "Alice".to_string(),
            tags: vec!["admin".to_string(), "beta".to_string()],
            active: true,
        },
        counters: vec![1, -2, 3],
    }
}

#[test]
fn restore_state_roundtrips_nested_state() {
    let state = sample_state();
    let attrs = state.hydration_attrs();
    let restored: NestedState =
        restore_state(&attrs).expect("roundtrip should succeed for well-formed attrs");
    assert_eq!(restored, state);
}

/// 項目文字列に区切り文字・バックスラッシュ・日本語/絵文字が混在しても、
/// ネストしたラウンドトリップが panic せず成立すること（データ注入耐性の
/// 統合確認、`docs/design/hydration-nested-state.md` 参照）。
#[test]
fn restore_state_roundtrips_nested_state_with_adversarial_strings() {
    let state = NestedState {
        user: UserProfile {
            name: "name:\u{1f}with\\sep".to_string(),
            tags: vec!["日本語🎉".to_string(), String::new()],
            active: false,
        },
        counters: vec![i64::MAX, i64::MIN, 0],
    };
    let attrs = state.hydration_attrs();
    let restored: NestedState =
        restore_state(&attrs).expect("roundtrip should survive adversarial strings");
    assert_eq!(restored, state);
}

#[test]
fn restore_state_fails_on_missing_state_attr() {
    let attrs: Vec<(String, String)> = Vec::new();
    let err = restore_state::<NestedState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::MissingAttr(_)));
}

#[test]
fn restore_state_fails_on_corrupted_value_encoding_without_panicking() {
    let attrs = vec![(
        format!("{HYDRATE_ATTR_PREFIX}state"),
        "not-a-valid-value-encoding-with-unknown-tag".to_string(),
    )];
    let err = restore_state::<NestedState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

/// `MAX_VALUE_DEPTH`（`rws_interactive::codec`）を超えるネストを持つ改ざん
/// 入力に対し、panic（スタックオーバーフロー含む）せず `HydrateError` を
/// 返すこと（A05 相当の DoS 耐性、`docs/design/hydration-nested-state.md` 参照）。
#[test]
fn restore_state_fails_on_excessively_deep_nesting_without_panicking() {
    let mut deep = Value::Int(0);
    for _ in 0..(codec::MAX_VALUE_DEPTH + 10) {
        deep = Value::List(vec![deep]);
    }
    let root = Value::Map(vec![
        (
            "user".to_string(),
            Value::Map(vec![
                ("name".to_string(), Value::Str("x".to_string())),
                ("tags".to_string(), Value::List(vec![])),
                ("active".to_string(), Value::Bool(true)),
            ]),
        ),
        ("counters".to_string(), deep),
    ]);
    let attrs = vec![(
        format!("{HYDRATE_ATTR_PREFIX}state"),
        codec::encode_value(&root),
    )];
    let err = restore_state::<NestedState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

/// 型不一致（`user` フィールドが文字列であるべきところに整数を混入）は
/// アプリ側の `from_value` 検証で `HydrateError` として捕捉され、panic
/// しないこと。
#[test]
fn restore_state_fails_on_type_mismatch_in_nested_field() {
    let root = Value::Map(vec![
        (
            "user".to_string(),
            Value::Map(vec![
                ("name".to_string(), Value::Int(123)), // 本来は Str
                ("tags".to_string(), Value::List(vec![])),
                ("active".to_string(), Value::Bool(true)),
            ]),
        ),
        ("counters".to_string(), Value::List(vec![])),
    ]);
    let attrs = vec![(
        format!("{HYDRATE_ATTR_PREFIX}state"),
        codec::encode_value(&root),
    )];
    let err = restore_state::<NestedState>(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}
