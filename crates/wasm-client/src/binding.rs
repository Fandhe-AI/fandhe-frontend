//! 束縛点ベースの最小更新: 純粋ロジック層（イシュー #343）。
//!
//! `docs/design/dom-binding-update-design.md`（#340 設計確定書）§3〜§4 が
//! 定める「`data-bind-*` 属性の 1 回走査で構築した束縛点対応表 +
//! `DirtyTracked::dirty_fields()`（#341）から駆動する最小更新」のうち、
//! `web-sys`（実 DOM）に依存しない部分をここに切り出す。DOM 非依存のため
//! `cargo test -p fandhe-frontend-wasm-client`（native）で検証でき、wasm ビルドを介さない
//! （`wasm-client` 既存の 2 層構成、`lib.rs` の方針を踏襲）。
//!
//! 実 DOM への適用（`query_selector_all` による走査・`set_text_content` 等の
//! 呼び出し）は `wasm32` 配線層 [`crate::binding_dom`] が本モジュールの型を
//! 消費して行う。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-core::bind`（#342）が SSR 出力に埋め込む `data-bind-text` /
//! `data-bind-attr` / `data-bind-class` マーカー属性値（`"<name>:<field>"`
//! 空白区切りトークン列）が本モジュールの入力契約そのものである。
//! `fandhe-frontend-interactive::DirtyTracked`（#341）の `dirty_fields()` が返す
//! `&'static str` と、本モジュールが DOM から読み出す実行時 `String` の
//! フィールド名は**文字列比較**で照合する（`&'static str` 側はコンパイル時に
//! 確定した有限集合であり、外部入力からの偽装余地はない。設計書 §3.2 の
//! 実装確定）。

/// 束縛点 1 件の種別（設計書 §3.1・§4.1 の 3 種別）。
///
/// `data-bind-list`（keyed list、構造変化）は #344 のスコープであり、
/// 本列挙には含めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
    /// `data-bind-text` — 要素の唯一のテキスト子ノードを更新する。
    Text,
    /// `data-bind-attr` の 1 トークン分 — 指定属性値を更新する。
    Attr(String),
    /// `data-bind-class` の 1 トークン分 — 指定 class のオン/オフを切り替える。
    Class(String),
}

/// 走査済み束縛点 1 件（`field` は DOM 属性値から読んだ実行時 `String`）。
///
/// 実 DOM ノードへの参照は持たない（wasm32 非依存を保つため）。
/// [`crate::binding_dom::BindingTable`] がこの型と `web_sys::Element` の
/// 組で対応表を構築する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingSpec {
    /// 束縛先の state フィールド名。
    pub field: String,
    /// 束縛の種別。
    pub kind: BindingKind,
}

/// 状態フィールドの現在値（更新適用の入力）。
///
/// `Text` はテキスト束縛・属性束縛の両方に使う。`Flag` は class 束縛にのみ
/// 使い、属性束縛へ渡された場合は `"true"`/`"false"` 文字列として出力する
/// （[`crate::binding_dom::BindingTable::apply_dirty`] の責務）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundValue {
    /// テキスト・属性値として出力する文字列。
    Text(String),
    /// class のオン/オフを表す真偽値。
    Flag(bool),
}

/// field 名 → 現在値の読み出し契約。
///
/// `#343` の消費側（本クレートのテスト、および `#345` の `wasm-full` 適用層）
/// が状態コンポーネント側にこの trait を実装し、
/// [`crate::binding_dom::BindingTable::apply_dirty`] / `apply_update` から
/// 呼ばれる。field に対応する値がない場合（未知 field・型不一致等）は
/// `None` を返し、呼び出し側は当該束縛を no-op として扱う（fail-closed、
/// panic しない）。
pub trait BindingSource {
    /// `field` の現在値を返す。存在しない・読み出せない場合は `None`。
    fn bound_value(&self, field: &str) -> Option<BoundValue>;
}

/// `data-bind-attr` / `data-bind-class` 属性値の属性・class 名として妥当かを
/// 判定する。
///
/// 英数字・`-`・`_` のみを許可し、空文字列を拒否する。加えて、
/// `setAttribute("onclick", value)` のような呼び出しは状態値を実行可能な
/// イベントハンドラへ昇格させてしまうため、大小文字を無視して `on` で
/// 始まる名前を拒否する（fail-closed。DOM 改ざんにより不正な
/// `data-bind-attr="onclick:draft"` が注入された場合の実行コード昇格を
/// ここで遮断する。設計書 §9 不変条件 2・`core/src/bind.rs` が明記する
/// 「消費側の契約」の履行）。
///
/// URL スキーム等「値の内容」の検証は設計書 §9 の確定通り本関数の責務では
/// ない（既存 SSR 経路と同等の残存リスク）。
fn is_valid_binding_name(name: &str) -> bool {
    !name.is_empty()
        && !name.to_ascii_lowercase().starts_with("on")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `"<name>:<field>"` 空白区切りトークン列をパースする（`data-bind-attr` /
/// `data-bind-class` 属性値の共通形式、設計書 §3.1）。
///
/// 不正トークンは黙って skip する（fail-closed。DOM 改ざん・部分的破損を
/// 前提とした安全側フォールバック、`DirtyTracked` ドキュメントの不変条件 4
/// と同じ方針）:
///
/// - コロンが 0 個・2 個以上のトークン
/// - `name` または `field` が空文字列
/// - `name` が [`is_valid_binding_name`] を満たさない（記号混入・`on*` 接頭辞）
///
/// 戻り値は `(name, field)` の組の列。順序はトークン出現順（決定的）。
pub fn parse_binding_tokens(raw: &str) -> Vec<(String, String)> {
    raw.split_whitespace()
        .filter_map(|token| {
            let (name, field) = token.split_once(':')?;
            // split_once(':') はコロンが 0 個なら None を返し早期 return 済み。
            // ここでは「2 個以上のコロン」を検出するため、残りにさらに ':' が
            // 含まれるかを見る（例: "a:b:c" は name="a" field="b:c" となり、
            // field 内のコロンは許容しない — 予約文字のため、`field` に ':' を
            // 含むトークンは skip する）。
            if field.contains(':') {
                return None;
            }
            if name.is_empty() || field.is_empty() {
                return None;
            }
            if !is_valid_binding_name(name) {
                return None;
            }
            Some((name.to_string(), field.to_string()))
        })
        .collect()
}

/// 1 要素分のマーカー属性値 3 種（`data-bind-text` / `data-bind-attr` /
/// `data-bind-class` の生の属性値、それぞれ未設定なら `None`）から
/// [`BindingSpec`] 列を構築する（走査ロジックの DOM 非依存部分）。
///
/// 決定的な順序（text → attr（トークン出現順） → class（トークン出現順））
/// で返す。不正な attr/class トークンは [`parse_binding_tokens`] の
/// fail-closed 方針により黙って skip される。
pub fn element_binding_specs(
    bind_text: Option<&str>,
    bind_attr: Option<&str>,
    bind_class: Option<&str>,
) -> Vec<BindingSpec> {
    let mut specs = Vec::new();

    if let Some(field) = bind_text {
        if !field.is_empty() {
            specs.push(BindingSpec {
                field: field.to_string(),
                kind: BindingKind::Text,
            });
        }
    }

    if let Some(raw) = bind_attr {
        for (attr, field) in parse_binding_tokens(raw) {
            specs.push(BindingSpec {
                field,
                kind: BindingKind::Attr(attr),
            });
        }
    }

    if let Some(raw) = bind_class {
        for (class, field) in parse_binding_tokens(raw) {
            specs.push(BindingSpec {
                field,
                kind: BindingKind::Class(class),
            });
        }
    }

    specs
}

/// Node 木を再帰走査し、`data-bind-text` / `data-bind-attr` /
/// `data-bind-class` マーカーから [`BindingSpec`] 列を収集する（イシュー
/// #380）。
///
/// # 呼び出し文脈
///
/// アプリのテストコードが `Component::view()` の戻り値（`fandhe_frontend_core::Node`）を
/// 本関数に渡し、[`unresolved_binding_specs`] で `BindingSource`
/// 実装（例: `AppState`）と突き合わせることで、view 側の束縛点マーカーと
/// 状態フィールドの整合をテスト時に検証する。`fw gate` への束縛点チェック
/// 追加は `docs/design/gate-design.md` §7 で非採用と確定済み（#353）だが、
/// 本関数は `fw gate` の `test` チェック（`cargo test`）経由で間接的に
/// カバーされる位置付けであり、§7 の非採用判断とは矛盾しない
/// （`docs/design/dom-binding-update-design.md` #380 追補節）。
///
/// # 実装方針（実挙動とのドリフト防止）
///
/// トークンパーサを新設せず、実 DOM 走査（[`crate::binding_dom`]）と同じ
/// [`element_binding_specs`] へ委譲する。検証ロジックと実行時ロジックが
/// 同一関数を通ることで、両者が非同期に変更されて乖離する余地を構造的に
/// 排除する。
///
/// # スコープ外
///
/// - `Node::Text` は素通り、`Node::RawHtml` は走査しない（HTML パースを
///   本関数へ持ち込まない。raw_html 経路は既存の raw-html レビューゲートの
///   領分であり、本関数の責務ではない）
/// - `data-bind-list`（keyed list、`fandhe_frontend_core::keyed::BIND_LIST_ATTR`）は
///   3 マーカーに含まれないため収集されない。`items` 等の keyed
///   フィールドが誤って「未束縛」と判定されることはない
///
/// 戻り値の順序は木の走査順（深さ優先・子ノード出現順）であり、決定的。
pub fn collect_binding_specs(node: &fandhe_frontend_core::Node) -> Vec<BindingSpec> {
    let mut specs = Vec::new();
    collect_binding_specs_into(node, &mut specs);
    specs
}

/// [`collect_binding_specs`] の再帰実装。
fn collect_binding_specs_into(node: &fandhe_frontend_core::Node, out: &mut Vec<BindingSpec>) {
    match node {
        fandhe_frontend_core::Node::Element {
            attrs, children, ..
        } => {
            let find = |name: &str| {
                attrs
                    .iter()
                    .find(|(attr_name, _)| attr_name == name)
                    .map(|(_, value)| value.as_str())
            };
            out.extend(element_binding_specs(
                find(fandhe_frontend_core::BIND_TEXT_ATTR),
                find(fandhe_frontend_core::BIND_ATTR_ATTR),
                find(fandhe_frontend_core::BIND_CLASS_ATTR),
            ));
            for child in children {
                collect_binding_specs_into(child, out);
            }
        }
        // Text はマーカー属性を持たない。RawHtml は HTML パースを要するため
        // スコープ外（上記 rustdoc 参照）。
        fandhe_frontend_core::Node::Text(_) | fandhe_frontend_core::Node::RawHtml(_) => {}
    }
}

/// [`collect_binding_specs`] で収集した束縛点のうち、`source.bound_value`
/// が `None` を返すもの（= 実行時に無音 no-op となるもの）を返す（イシュー
/// #380）。空 `Vec` なら view のマーカーと `source` のフィールドが整合して
/// いる。
///
/// # 実行時 fail-closed 契約との関係
///
/// [`BindingSource::bound_value`] は未知 field に対し panic せず `None` を
/// 返す設計（本ファイル冒頭 doc 参照）であり、この不変条件には一切手を
/// 入れない。本関数はその「無音の no-op」をテスト時に可視化し、CI の
/// `test` チェックで検出可能にするための読み取り専用ユーティリティに
/// 限定する。
pub fn unresolved_binding_specs<S: BindingSource>(
    node: &fandhe_frontend_core::Node,
    source: &S,
) -> Vec<BindingSpec> {
    collect_binding_specs(node)
        .into_iter()
        .filter(|spec| source.bound_value(&spec.field).is_none())
        .collect()
}

/// [`fandhe_frontend_interactive::AppState`] を [`BindingSource`] へ接続する（イシュー
/// #345）。孤児則（orphan rule）により、この実装は `BindingSource`（本クレート
/// 定義）と `AppState`（`fandhe_frontend_interactive` 定義）のいずれか一方を所有する
/// クレートでのみ書ける。`AppState` を所有する `fandhe-frontend-interactive` は
/// `wasm-client`（DOM 依存クレート）へ依存できない設計方針であるため、
/// 実装先は `wasm-client` 側一択となる（`docs/design/dom-binding-update-design.md`
/// #345 実装確定節）。
///
/// `counter`/`draft` の 2 フィールドのみを扱う。`items`（keyed list）は
/// [`BindingSource`] の対象外（[`crate::binding_dom::BindingTable`] の
/// text/attr/class 更新経路ではなく、[`crate::keyed_diff`]/[`crate::keyed_dom`]
/// の構造変化専用経路が扱う。設計書 §5 が定める「構造変化を表現できる唯一の
/// 経路」の原則をクライアント側の型でも保つ）。未知 field は `None`
/// （fail-closed、`BindingSource` のドキュメント参照）。
impl BindingSource for fandhe_frontend_interactive::AppState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            f if f == fandhe_frontend_interactive::AppState::FIELD_COUNTER => {
                Some(BoundValue::Text(self.counter.to_string()))
            }
            f if f == fandhe_frontend_interactive::AppState::FIELD_DRAFT => {
                Some(BoundValue::Text(self.draft.clone()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_binding_tokens_parses_multiple_tokens() {
        let tokens = parse_binding_tokens("aria-pressed:liked disabled:busy");
        assert_eq!(
            tokens,
            vec![
                ("aria-pressed".to_string(), "liked".to_string()),
                ("disabled".to_string(), "busy".to_string()),
            ]
        );
    }

    #[test]
    fn parse_binding_tokens_skips_missing_colon() {
        assert_eq!(parse_binding_tokens("noColonToken"), Vec::new());
    }

    #[test]
    fn parse_binding_tokens_skips_empty_name_or_field() {
        assert_eq!(parse_binding_tokens(":field"), Vec::new());
        assert_eq!(parse_binding_tokens("name:"), Vec::new());
    }

    #[test]
    fn parse_binding_tokens_skips_extra_colon_in_field() {
        // field に予約文字 ':' を含むトークンは skip する（fail-closed）。
        assert_eq!(parse_binding_tokens("name:a:b"), Vec::new());
    }

    #[test]
    fn parse_binding_tokens_skips_on_prefixed_attribute_names_case_insensitively() {
        assert_eq!(parse_binding_tokens("onclick:draft"), Vec::new());
        assert_eq!(parse_binding_tokens("ONCLICK:draft"), Vec::new());
        assert_eq!(parse_binding_tokens("OnMouseOver:draft"), Vec::new());
    }

    #[test]
    fn parse_binding_tokens_skips_symbol_polluted_names() {
        assert_eq!(parse_binding_tokens("a b=c:draft"), Vec::new());
        assert_eq!(parse_binding_tokens("a<script>:draft"), Vec::new());
    }

    #[test]
    fn parse_binding_tokens_allows_hyphen_and_underscore_names() {
        let tokens = parse_binding_tokens("data-x_1:counter");
        assert_eq!(
            tokens,
            vec![("data-x_1".to_string(), "counter".to_string())]
        );
    }

    #[test]
    fn element_binding_specs_returns_all_specs_for_co_located_markers() {
        let specs = element_binding_specs(
            Some("counter"),
            Some("aria-pressed:liked disabled:busy"),
            Some("liked:liked"),
        );
        assert_eq!(
            specs,
            vec![
                BindingSpec {
                    field: "counter".to_string(),
                    kind: BindingKind::Text,
                },
                BindingSpec {
                    field: "liked".to_string(),
                    kind: BindingKind::Attr("aria-pressed".to_string()),
                },
                BindingSpec {
                    field: "busy".to_string(),
                    kind: BindingKind::Attr("disabled".to_string()),
                },
                BindingSpec {
                    field: "liked".to_string(),
                    kind: BindingKind::Class("liked".to_string()),
                },
            ]
        );
    }

    #[test]
    fn element_binding_specs_returns_empty_for_no_markers() {
        assert_eq!(element_binding_specs(None, None, None), Vec::new());
    }

    #[test]
    fn element_binding_specs_skips_invalid_attr_tokens_but_keeps_valid_ones() {
        let specs = element_binding_specs(None, Some("onclick:draft valid-attr:field"), None);
        assert_eq!(
            specs,
            vec![BindingSpec {
                field: "field".to_string(),
                kind: BindingKind::Attr("valid-attr".to_string()),
            }]
        );
    }

    #[test]
    fn element_binding_specs_ignores_empty_text_field() {
        assert_eq!(element_binding_specs(Some(""), None, None), Vec::new());
    }

    #[test]
    fn app_state_bound_value_returns_counter_and_draft_as_text() {
        let mut state = fandhe_frontend_interactive::AppState::new();
        state.counter = 5;
        state.draft = "hello".to_string();
        assert_eq!(
            state.bound_value(fandhe_frontend_interactive::AppState::FIELD_COUNTER),
            Some(BoundValue::Text("5".to_string()))
        );
        assert_eq!(
            state.bound_value(fandhe_frontend_interactive::AppState::FIELD_DRAFT),
            Some(BoundValue::Text("hello".to_string()))
        );
    }

    #[test]
    fn app_state_bound_value_returns_none_for_items_and_unknown_fields() {
        let state = fandhe_frontend_interactive::AppState::new();
        // items は keyed list 専用経路（`keyed_diff`/`keyed_dom`）が扱うため、
        // BindingSource（text/attr/class 更新経路）の対象外とする。
        assert_eq!(
            state.bound_value(fandhe_frontend_interactive::AppState::FIELD_ITEMS),
            None
        );
        assert_eq!(state.bound_value("unknown-field"), None);
    }

    // --- collect_binding_specs / unresolved_binding_specs（イシュー #380） ---

    use fandhe_frontend_core::{el, text, Node, BIND_ATTR_ATTR, BIND_CLASS_ATTR, BIND_TEXT_ATTR};

    /// `BindingSource` テストダブル。任意の field 集合を「解決可能」として
    /// 振る舞わせ、`unresolved_binding_specs` の突き合わせロジックだけを
    /// `AppState` から切り離して検証する。
    struct FakeSource {
        known_fields: Vec<&'static str>,
    }

    impl BindingSource for FakeSource {
        fn bound_value(&self, field: &str) -> Option<BoundValue> {
            if self.known_fields.contains(&field) {
                Some(BoundValue::Text("dummy".to_string()))
            } else {
                None
            }
        }
    }

    #[test]
    fn collect_binding_specs_walks_nested_elements_in_document_order() {
        let node = el(
            "div",
            vec![],
            vec![
                el("span", vec![(BIND_TEXT_ATTR, "counter")], vec![text("0")]),
                el("input", vec![(BIND_ATTR_ATTR, "value:draft")], vec![]),
            ],
        );
        let specs = collect_binding_specs(&node);
        assert_eq!(
            specs,
            vec![
                BindingSpec {
                    field: "counter".to_string(),
                    kind: BindingKind::Text,
                },
                BindingSpec {
                    field: "draft".to_string(),
                    kind: BindingKind::Attr("value".to_string()),
                },
            ]
        );
    }

    #[test]
    fn collect_binding_specs_collects_multiple_markers_on_one_element() {
        let node = el(
            "button",
            vec![
                (BIND_TEXT_ATTR, "counter"),
                (BIND_ATTR_ATTR, "aria-pressed:liked"),
                (BIND_CLASS_ATTR, "liked:liked"),
            ],
            vec![text("0")],
        );
        let specs = collect_binding_specs(&node);
        assert_eq!(specs.len(), 3);
    }

    #[test]
    fn collect_binding_specs_ignores_raw_html_nodes() {
        // RawHtml は HTML パースを要するためスコープ外
        // （rustdoc の「スコープ外」節を固定する回帰テスト）。
        let node = el(
            "div",
            vec![],
            vec![Node::RawHtml(format!(
                "<span {BIND_TEXT_ATTR}=\"counter\">0</span>"
            ))],
        );
        assert_eq!(collect_binding_specs(&node), Vec::new());
    }

    #[test]
    fn collect_binding_specs_ignores_bind_list_marker() {
        // data-bind-list（keyed list）は 3 マーカーに含まれないため収集
        // されない。keyed 経路との分離を固定する。
        let node = el(
            "ul",
            vec![(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "items")],
            vec![],
        );
        assert_eq!(collect_binding_specs(&node), Vec::new());
    }

    #[test]
    fn collect_binding_specs_returns_empty_for_tree_without_markers() {
        let node = el("div", vec![], vec![text("plain")]);
        assert_eq!(collect_binding_specs(&node), Vec::new());
    }

    #[test]
    fn unresolved_binding_specs_detects_typo_field() {
        // "countr" は BindingSource 側に存在しないため、実行時には
        // no-op（無音の表示更新停止）となる不整合を検出する。
        let node = el("span", vec![(BIND_TEXT_ATTR, "countr")], vec![text("0")]);
        let source = FakeSource {
            known_fields: vec!["counter"],
        };
        let unresolved = unresolved_binding_specs(&node, &source);
        assert_eq!(
            unresolved,
            vec![BindingSpec {
                field: "countr".to_string(),
                kind: BindingKind::Text,
            }]
        );
    }

    #[test]
    fn unresolved_binding_specs_is_empty_when_all_fields_known() {
        let node = el("span", vec![(BIND_TEXT_ATTR, "counter")], vec![text("0")]);
        let source = FakeSource {
            known_fields: vec!["counter"],
        };
        assert_eq!(unresolved_binding_specs(&node, &source), Vec::new());
    }
}
