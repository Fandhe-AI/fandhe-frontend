//! ハイドレーション状態注入の実装（TASK-11.4b、イシュー #83）。
//!
//! フォーマット規約は `docs/hydration-state-format.md`（TASK-11.4a・#82、正の
//! 規範文書）が確定済みであり、本モジュールはその第 5 節が凍結した API 表面
//! （[`read_hydration_attrs`]・[`restore_state`]）を実装するのみで、フォーマット
//! 自体（属性命名・codec）を再定義・再実装しない。属性名プレフィックスは
//! [`rws_interactive::HYDRATE_ATTR_PREFIX`] を単一の真実として扱う。
//!
//! `events.rs`（TASK-11.2b・#75）・`dom.rs`（TASK-11.2c・#76）と同じ 2 層構成を
//! 踏襲する。
//!
//! - **純粋ロジック層**（[`restore_state`]・[`filter_hydration_attrs`]）: DOM・
//!   `web-sys` に依存せず、native の `cargo test` で検証できる。
//! - **wasm32 配線層**（[`read_hydration_attrs`]）: `#[cfg(target_arch = "wasm32")]`
//!   でゲートし、native ビルドへ `web-sys::Element` 依存を混入させない。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - SSR 側の対（サーバー Rust が担う状態保持・属性出力の責務）は
//!   `rws_interactive::render_for_hydration`（`interactive/src/lib.rs:287`）で
//!   完結済みであり、本モジュールは一切変更しない。
//! - `data-hydrate-*` 属性値は改ざんされうるクライアント入力として扱う。
//!   復元は `restore_state` → `C::from_hydration_attrs` の `Result` 経路のみを
//!   通し、`unwrap()`/`panic!` を使わない（不変条件 2・3）。
//! - `Runtime::hydrate`（TASK-11.2d・#77）との結合: `docs/hydration-state-format.md`
//!   第 5.1 節に従い、#77 マージ時点で `Runtime<C>` が存在すれば
//!   `read_hydration_attrs` → `restore_state` の順に呼び出し、`Err` 時は引数の
//!   `component`（初期状態）のまま CSR 再描画（`Runtime::mount` 相当）へ
//!   フォールバックする契約とする。本モジュールは `Runtime` 型に一切依存しない
//!   関数群として設計されているため、#77 の未マージ状態でも独立して成立する。
//!   本コミット時点で `wasm-full/src/lib.rs` に `Runtime<C>` は未存在のため、
//!   結合コード自体は #77・#83 のうち後にマージされる側が実装する（未着手）。

use rws_interactive::{Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// 1 属性値の長さ上限（バイト数）。
///
/// `data-hydrate-*` 属性値は改ざんされうるクライアント入力であり、上限を
/// 設けない場合は巨大な属性値を注入されるとパース処理（`codec::decode_list`
/// 等）のコストが状態注入 1 回あたり無制限に増大し得る（A05 相当の DoS
/// 耐性、`docs/hydration-state-format.md` 第 4 節・判断 7 / 第 8 節・不変条件 4
/// で TASK-11.4b への実装検討事項として引き継がれた事項の決定）。
///
/// 64 KiB は通常のフォーム入力・動的リスト用途（数値・短い文字列・数十件
/// 程度の文字列配列）を十分許容しつつ、無制限の巨大属性値を弾く値として
/// 選定した（`docs/hydration-state-format.md` が対象とする「単純な値」制約
/// との整合。将来より精密な上限が必要になった場合は Issue 化して見直す）。
///
/// 上限超過の属性は [`filter_hydration_attrs`] が列挙対象から除外する。
/// 除外された属性は復元側（[`restore_state`] → `Hydrate::from_hydration_attrs`）
/// から見ると「欠落した属性」と区別がつかず、結果として
/// [`HydrateError::MissingAttr`] を経由し安全側フォールバック（初期状態での
/// CSR 再描画）に収束する（`unwrap()`/`panic!` を使わない不変条件を維持した
/// まま DoS 耐性を確保する設計）。
pub const MAX_ATTR_VALUE_LEN: usize = 64 * 1024;

/// `HYDRATE_ATTR_PREFIX` で始まる属性のみを抽出する内部フィルタ。
///
/// 未知の `data-hydrate-*` 属性（アプリの `Hydrate` 実装が使わないフィールド名）
/// は除外せずそのまま通す。未知フィールドの無視は復元側
/// （`Hydrate::from_hydration_attrs`）の契約であり
/// （`docs/hydration-state-format.md` 第 4 節・判断 5）、本関数の責務は
/// プレフィックス絞り込みのみに留める。
///
/// [`MAX_ATTR_VALUE_LEN`] を超える値を持つ属性は、DoS 耐性のため列挙結果から
/// 除外する。除外時に出力するログ（`web_sys::console` 等）には属性名のみを
/// 含め、値の内容は一切含めない（A09: 機微情報非露出、
/// `docs/hydration-state-format.md` 第 8 節・不変条件 6）。
///
/// wasm32 配線層（[`read_hydration_attrs`]）・native テストの双方から呼ばれる
/// DOM 非依存の純粋関数。呼び出し元は wasm32 ターゲットの `wiring` モジュール
/// と `#[cfg(test)]` のみのため、native の非テストビルド（`cargo check
/// --workspace` 等）では未使用と誤検出される。dead_code 抑制はそのための
/// もので、ロジック自体が不要という意味ではない。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn filter_hydration_attrs(pairs: impl Iterator<Item = (String, String)>) -> Vec<(String, String)> {
    pairs
        .filter(|(name, value)| {
            name.starts_with(HYDRATE_ATTR_PREFIX) && value.len() <= MAX_ATTR_VALUE_LEN
        })
        .collect()
}

/// `data-hydrate-*` 属性列から状態を復元する（クライアント側責務）。
///
/// `docs/hydration-state-format.md` 第 5 節が凍結した API。`C::from_hydration_attrs`
/// （`rws_interactive::Hydrate`）へ委譲する薄いラッパーであり、フォーマット
/// 固有の追加ロジックは持たない。DOM・`web-sys` に依存しない純粋関数のため、
/// native の `cargo test`（wasm32 ターゲット不要）で直接検証できる。
///
/// # Errors
///
/// 属性の欠落・値の形式不正（例: 数値パース失敗）の場合に
/// [`HydrateError`] を返す。panic しない（改ざんされうるクライアント入力を
/// 前提とした防御的処理、`docs/hydration-state-format.md` 第 8 節・不変条件 2）。
/// 呼び出し側（`Runtime::hydrate` 等）は `Err` を「初期状態での CSR 再描画へ
/// フォールバックすべき」シグナルとして扱う契約（同書第 4 節・判断 6）。
pub fn restore_state<C: Hydrate>(attrs: &[(String, String)]) -> Result<C, HydrateError> {
    C::from_hydration_attrs(attrs)
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`dom.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::filter_hydration_attrs;
    use web_sys::Element;

    /// `root` 要素の属性一覧から `data-hydrate-*` 属性を列挙する（サーバーが
    /// 出力した状態のクライアント側読み取り）。
    ///
    /// `docs/hydration-state-format.md` 第 5 節が凍結した API。`root.attributes()`
    /// （`web_sys::NamedNodeMap`）を index 走査し、各 `web_sys::Attr` の
    /// `name()`/`value()` を読み取るのみで、`set_inner_html` 等の DOM
    /// **再構築** API は一切呼ばない（読み取り専用、`docs/hydration-state-format.md`
    /// 第 8 節・不変条件 1 の「新たな迂回経路を作らない」を DOM 操作面でも
    /// 徹底する）。
    ///
    /// `attr.name()` によるプレフィックス絞り込みを `attr.value()` 呼び出しの
    /// **前**に行い、`data-hydrate-*` 以外の属性は値を一切読み取らない。
    /// root には（改ざんされうる）任意の属性が付与され得るため、無関係な
    /// 属性に巨大な値を仕込まれても Rust 側へコピーしない設計とし、
    /// [`MAX_ATTR_VALUE_LEN`]（`super::MAX_ATTR_VALUE_LEN`）による DoS 耐性
    /// （`docs/hydration-state-format.md` 第 8 節・不変条件 4）を値取得コスト
    /// の面でも徹底する。値長上限フィルタ自体は
    /// [`filter_hydration_attrs`]（純粋ロジック層）へ委譲する。
    ///
    /// 復元本体は行わない（[`super::restore_state`] の責務）。
    pub fn read_hydration_attrs(root: &Element) -> Vec<(String, String)> {
        let attrs = root.attributes();
        let len = attrs.length();
        let mut pairs = Vec::with_capacity(len as usize);
        for i in 0..len {
            // `NamedNodeMap::item` は index が範囲外の場合 `None` を返す
            // （`web-sys` バインディングの Rust 側表現）。`length()` の範囲内
            // 走査のため通常到達しないが、`unwrap()` による panic を避け
            // `if let` で防御的に扱う。
            if let Some(attr) = attrs.item(i) {
                let name = attr.name();
                // `data-hydrate-*` 以外の属性は `value()` を呼ばずに読み飛ばす
                // （Bugbot 指摘対応: フィルタ前の全属性値コピーによる DoS
                // 制限バイパスを避ける）。
                if name.starts_with(super::HYDRATE_ATTR_PREFIX) {
                    pairs.push((name, attr.value()));
                }
            }
        }
        filter_hydration_attrs(pairs.into_iter())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::read_hydration_attrs;

#[cfg(test)]
mod tests {
    use super::*;

    /// native テスト専用の最小 `Hydrate` 実装。
    ///
    /// `rws_interactive::AppState`（counter/draft/items の 3 フィールド）と
    /// 同型の構成を最小限で再現し、`restore_state` のラウンドトリップ・
    /// エラー経路を DOM 非依存で検証する。
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestState {
        counter: i64,
        draft: String,
        items: Vec<String>,
    }

    impl Hydrate for TestState {
        fn hydration_attrs(&self) -> Vec<(String, String)> {
            vec![
                (
                    format!("{HYDRATE_ATTR_PREFIX}counter"),
                    self.counter.to_string(),
                ),
                (format!("{HYDRATE_ATTR_PREFIX}draft"), self.draft.clone()),
                (
                    format!("{HYDRATE_ATTR_PREFIX}items"),
                    rws_interactive::codec::encode_list(&self.items),
                ),
            ]
        }

        fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
            let find = |name: &str| -> Result<&str, HydrateError> {
                attrs
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.as_str())
                    .ok_or_else(|| HydrateError::MissingAttr(name.to_string()))
            };
            let counter_attr = format!("{HYDRATE_ATTR_PREFIX}counter");
            let draft_attr = format!("{HYDRATE_ATTR_PREFIX}draft");
            let items_attr = format!("{HYDRATE_ATTR_PREFIX}items");

            let counter =
                find(&counter_attr)?
                    .parse::<i64>()
                    .map_err(|_| HydrateError::InvalidValue {
                        attr: counter_attr.clone(),
                        reason: "not a valid integer".to_string(),
                    })?;
            let draft = find(&draft_attr)?.to_string();
            let items = rws_interactive::codec::decode_list(find(&items_attr)?);

            Ok(TestState {
                counter,
                draft,
                items,
            })
        }
    }

    #[test]
    fn restore_state_roundtrips_via_hydration_attrs() {
        let state = TestState {
            counter: 3,
            draft: "draft text".to_string(),
            items: vec!["a".to_string(), "b".to_string()],
        };
        let attrs = state.hydration_attrs();
        let restored: TestState = restore_state(&attrs).expect("valid attrs");
        assert_eq!(restored, state);
    }

    #[test]
    fn restore_state_fails_on_invalid_numeric_value() {
        let attrs = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}counter"),
                "not-a-number".to_string(),
            ),
            (format!("{HYDRATE_ATTR_PREFIX}draft"), String::new()),
            (format!("{HYDRATE_ATTR_PREFIX}items"), String::new()),
        ];
        let err = restore_state::<TestState>(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn restore_state_fails_on_missing_attr() {
        let attrs: Vec<(String, String)> = Vec::new();
        let err = restore_state::<TestState>(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn filter_hydration_attrs_keeps_only_prefixed_names() {
        let pairs = vec![
            (format!("{HYDRATE_ATTR_PREFIX}counter"), "1".to_string()),
            ("data-testid".to_string(), "root".to_string()),
            ("id".to_string(), "interactive-root".to_string()),
        ];
        let filtered = filter_hydration_attrs(pairs.into_iter());
        assert_eq!(
            filtered,
            vec![(format!("{HYDRATE_ATTR_PREFIX}counter"), "1".to_string())]
        );
    }

    /// 未知の `data-hydrate-*` 属性（アプリの `Hydrate` 実装が使わないフィールド
    /// 名）はここでは除外せずそのまま通す契約
    /// （`docs/hydration-state-format.md` 第 4 節・判断 5、無視は復元側の責務）。
    #[test]
    fn filter_hydration_attrs_passes_through_unknown_hydrate_fields() {
        let pairs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}unknown-field"),
            "x".to_string(),
        )];
        let filtered = filter_hydration_attrs(pairs.into_iter());
        assert_eq!(
            filtered,
            vec![(
                format!("{HYDRATE_ATTR_PREFIX}unknown-field"),
                "x".to_string()
            )]
        );
    }

    /// DoS 耐性（第 4 節・判断 7 / 第 8 節・不変条件 4）: 上限を超える属性値は
    /// 除外され、復元側では `MissingAttr` として扱われる（安全側フォールバック
    /// に収束することを確認）。
    #[test]
    fn filter_hydration_attrs_excludes_oversized_values() {
        let oversized_value = "x".repeat(MAX_ATTR_VALUE_LEN + 1);
        let pairs = vec![(format!("{HYDRATE_ATTR_PREFIX}draft"), oversized_value)];
        let filtered = filter_hydration_attrs(pairs.into_iter());
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_hydration_attrs_keeps_values_at_the_size_limit() {
        let boundary_value = "x".repeat(MAX_ATTR_VALUE_LEN);
        let pairs = vec![(
            format!("{HYDRATE_ATTR_PREFIX}draft"),
            boundary_value.clone(),
        )];
        let filtered = filter_hydration_attrs(pairs.into_iter());
        assert_eq!(
            filtered,
            vec![(format!("{HYDRATE_ATTR_PREFIX}draft"), boundary_value)]
        );
    }
}
