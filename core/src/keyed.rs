//! keyed list プリミティブ（構造変化の唯一の経路、イシュー #344）。
//!
//! `docs/design/dom-binding-update-design.md`（#340 設計確定書）第 5 節が
//! 定める、実 DOM 直接更新方針（イシュー #336）における「リストの挿入・
//! 削除・並べ替え（構造変化）を表現できる唯一の経路」。汎用 diff・仮想 DOM
//! は実装しない（同書第 5・7 節で確定済み）。
//!
//! SSR/SSG 出力には [`BIND_LIST_ATTR`]（`data-bind-list="<field>"`）が
//! リスト親要素に、[`KEY_ATTR`]（`data-key="<key>"`）が各子要素に現れる。
//! `wasm-full` の CSR 側（イシュー #343/#345）はこの 2 属性を走査してキー
//! 照合を行い、`set_inner_html` による全置換ではなく最小の DOM 操作
//! （insert/remove/move）を適用する契約になっている。**本モジュールが
//! 生成するのはこの属性形式の `Node` 木のみ**であり、キー照合・DOM 適用
//! そのものは #343/#345 のスコープ（本モジュールの責務外）。
//!
//! # 不変条件（本クレート冒頭 doc の不変条件 1・2 の継承）
//!
//! [`keyed_list`] は既存の [`crate::Node`] 木を組み立てるだけであり、新しい
//! `Node` バリアント・新しいレンダリング経路・新しいエスケープ処理を追加
//! しない。出力される `data-key`/`data-bind-list` の属性値は
//! [`crate::render`] の既定エスケープを常に経由する（不変条件 1）。エスケープ
//! を迂回する経路は本モジュールには存在しない（不変条件 2 を弱めない）。

use crate::Node;

/// リスト束縛のマーカー属性名。
///
/// keyed list の親要素に付与され、値はリスト化対象のフィールド名
/// （`&'static str`）。`wasm-full`（#343/#345）はこの属性を走査してリスト
/// 親要素を特定する契約値であり、値は本モジュールの `render()` 出力上で
/// 固定される（設計書 §3.1 で凍結）。
pub const BIND_LIST_ATTR: &str = "data-bind-list";

/// キー属性名。
///
/// keyed list の各子要素に付与され、値はアプリ側が指定した一意キー。
/// `wasm-full`（#343/#345）はこの属性値でキー照合を行い、挿入・削除・
/// 並べ替えを最小の DOM 操作へ変換する契約値（設計書 §3.1 で凍結）。
pub const KEY_ATTR: &str = "data-key";

/// [`keyed_list`] 構築時の fail-closed エラー。
///
/// いずれの異常系も `panic!`/`unwrap()` ではなく `Err` として安全側に倒す
/// （ライブラリコードでの panic 回避規約、OWASP A05 安全でない設計への対抗）。
/// 「衝突・欠落したキーを持つ不正な HTML を出力しない」という fail-closed の
/// 目的を、`render()` 呼び出し時点ではなく**構築時点**で満たす（不正な状態を
/// そもそも表現不能にする設計。`docs/design/dom-binding-update-design.md`
/// §5.2 改訂内容を参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedListError {
    /// キーが空文字列（キー欠落）。`index` は `items` 内の位置。
    EmptyKey {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 同一リスト内でキーが重複している（直下の子スコープのみが対象）。
    DuplicateKey {
        /// 最初に当該キーが出現したインデックス。
        first_index: usize,
        /// 重複が検出されたインデックス。
        duplicate_index: usize,
    },
    /// 子ノードが `Node::Element` でなく、`data-key` 属性を付与できない。
    NonElementItem {
        /// `items` 内のインデックス。
        index: usize,
    },
    /// 呼び出し側が渡した属性列に予約マーカー属性
    /// （[`BIND_LIST_ATTR`] / [`KEY_ATTR`]）が既に含まれている。
    ReservedAttr {
        /// 衝突した予約属性名。
        attr: &'static str,
    },
}

impl std::fmt::Display for KeyedListError {
    /// エラーメッセージは英語・固定文言 + インデックスのみとし、キー値・
    /// 項目内容（アプリ状態）は含めない（ログ・エラーメッセージへの機微
    /// 情報非露出、OWASP A09 対策。設計書 §9 不変条件 7 を継承）。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyedListError::EmptyKey { index } => {
                write!(f, "keyed_list: empty key at item index {index}")
            }
            KeyedListError::DuplicateKey {
                first_index,
                duplicate_index,
            } => write!(
                f,
                "keyed_list: duplicate key at item index {duplicate_index} \
                 (first seen at index {first_index})"
            ),
            KeyedListError::NonElementItem { index } => {
                write!(
                    f,
                    "keyed_list: item at index {index} is not an Element node"
                )
            }
            KeyedListError::ReservedAttr { attr } => {
                write!(f, "keyed_list: reserved attribute \"{attr}\" is reserved")
            }
        }
    }
}

impl std::error::Error for KeyedListError {}

/// keyed list を構築する。構造変化（挿入・削除・並べ替え）を表現できる
/// **唯一の経路**（設計書第 5 節）。
///
/// 呼び出し側は親要素のタグ名・属性・リスト化対象フィールド名・
/// `(キー, 子ノード)` のペア列を渡す。成功時は次の形の `Node::Element` を
/// 返す。
///
/// - 親要素: `attrs` の末尾に `data-bind-list="<field>"` を付加したもの。
/// - 各子要素: 元の属性列の末尾に `data-key="<key>"` を付加したもの。
///
/// キーの一意性検査は**直下の子のみ**が対象（子孫にネストした
/// `keyed_list` 呼び出しのキー空間とは独立）。
///
/// # Errors
///
/// - [`KeyedListError::EmptyKey`][]: いずれかのキーが空文字列。
/// - [`KeyedListError::DuplicateKey`][]: 同一リスト内でキーが重複。
/// - [`KeyedListError::NonElementItem`]: 子ノードが `Node::Element` でない
///   （`data-key` を付与する対象を持たないため）。
/// - [`KeyedListError::ReservedAttr`]: `attrs` または各子要素の属性列に
///   [`BIND_LIST_ATTR`] / [`KEY_ATTR`] が既に含まれている（マーカー属性の
///   重複・偽装を構造的に防止）。
///
/// # Examples
///
/// ```
/// use rws_core::{el, text, render, keyed::keyed_list};
///
/// let list = keyed_list(
///     "ul",
///     vec![("data-testid", "item-list")],
///     "items",
///     vec![
///         ("a".to_string(), el("li", vec![], vec![text("item-a")])),
///         ("b".to_string(), el("li", vec![], vec![text("item-b")])),
///     ],
/// )
/// .expect("valid keyed list");
///
/// assert_eq!(
///     render(&list),
///     concat!(
///         r#"<ul data-testid="item-list" data-bind-list="items">"#,
///         r#"<li data-key="a">item-a</li>"#,
///         r#"<li data-key="b">item-b</li>"#,
///         "</ul>",
///     ),
/// );
/// ```
pub fn keyed_list(
    tag: &'static str,
    attrs: Vec<(&str, &str)>,
    field: &'static str,
    items: Vec<(String, Node)>,
) -> Result<Node, KeyedListError> {
    // (1) 親属性への予約属性の混入を拒否する。呼び出し側が data-bind-list を
    // 直接指定できると、実際のリスト構造と食い違う値で #343/#345 のキー照合
    // 契約を偽装できてしまうため fail-closed で遮断する。
    reject_reserved_attr(&attrs, BIND_LIST_ATTR)?;
    reject_reserved_attr(&attrs, KEY_ATTR)?;

    // (2)(3) 各子要素の検証: Element であること・キー非空・キー一意性。
    // HashSet は直下スコープのみを対象とし O(n) で判定する（非再帰、DoS 耐性）。
    let mut seen_keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut first_index_of: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    let mut children = Vec::with_capacity(items.len());

    for (index, (key, item)) in items.iter().enumerate() {
        if key.is_empty() {
            return Err(KeyedListError::EmptyKey { index });
        }
        if let Some(&first_index) = first_index_of.get(key.as_str()) {
            return Err(KeyedListError::DuplicateKey {
                first_index,
                duplicate_index: index,
            });
        }
        seen_keys.insert(key.as_str());
        first_index_of.insert(key.as_str(), index);

        let Node::Element {
            tag: item_tag,
            attrs: item_attrs,
            children: item_children,
        } = item
        else {
            return Err(KeyedListError::NonElementItem { index });
        };

        // 子要素の既存属性にも同じ予約属性チェックをかける（親と同じ理由）。
        let item_attrs_pairs: Vec<(&str, &str)> = item_attrs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        reject_reserved_attr(&item_attrs_pairs, KEY_ATTR)?;
        reject_reserved_attr(&item_attrs_pairs, BIND_LIST_ATTR)?;

        let mut new_attrs = item_attrs.clone();
        new_attrs.push((KEY_ATTR.to_string(), key.clone()));
        children.push(Node::Element {
            tag: item_tag,
            attrs: new_attrs,
            children: item_children.clone(),
        });
    }

    // (4) 親 Node::Element を組み立てる。data-bind-list は呼び出し側 attrs の
    // 後ろへ決定的順序で付加する（出力バイトの決定性・SSR/SSG 一致の土台）。
    let mut parent_attrs: Vec<(String, String)> = attrs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    parent_attrs.push((BIND_LIST_ATTR.to_string(), field.to_string()));

    Ok(Node::Element {
        tag,
        attrs: parent_attrs,
        children,
    })
}

/// 呼び出し側属性列に予約マーカー属性が含まれていないか検査する。
fn reject_reserved_attr(
    attrs: &[(&str, &str)],
    reserved: &'static str,
) -> Result<(), KeyedListError> {
    if attrs.iter().any(|(k, _)| *k == reserved) {
        return Err(KeyedListError::ReservedAttr { attr: reserved });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{el, li, render, text, ul};

    /// 正常系: SSR 出力のバイトを固定する。属性の付加順序
    /// （呼び出し側 attrs の後ろに data-bind-list / data-key）を保証する。
    #[test]
    fn keyed_list_renders_expected_byte_output() {
        let list = keyed_list(
            "ul",
            vec![("data-testid", "item-list")],
            "items",
            vec![
                ("a".to_string(), el("li", vec![], vec![text("item-a")])),
                ("b".to_string(), el("li", vec![], vec![text("item-b")])),
            ],
        )
        .expect("valid keyed list");

        assert_eq!(
            render(&list),
            concat!(
                r#"<ul data-testid="item-list" data-bind-list="items">"#,
                r#"<li data-key="a">item-a</li>"#,
                r#"<li data-key="b">item-b</li>"#,
                "</ul>",
            ),
        );
    }

    /// 決定性: 同一入力を 2 回構築しても同一出力になる（SSR/SSG 出力一致の
    /// 保証・新レンダリング経路を追加しないことの回帰確認）。
    #[test]
    fn keyed_list_output_is_deterministic() {
        let build = || {
            keyed_list(
                "ul",
                vec![],
                "items",
                vec![
                    ("x".to_string(), el("li", vec![], vec![text("x")])),
                    ("y".to_string(), el("li", vec![], vec![text("y")])),
                ],
            )
            .expect("valid keyed list")
        };
        assert_eq!(render(&build()), render(&build()));
    }

    /// 空 items は正常系（空リスト状態）。親要素のみが出力される。
    #[test]
    fn keyed_list_with_empty_items_renders_parent_only() {
        let list = keyed_list("ul", vec![], "items", vec![]).expect("valid keyed list");
        assert_eq!(render(&list), r#"<ul data-bind-list="items"></ul>"#);
    }

    /// ネスト: 子孫に別の keyed_list を含んでも、一意性検査は直下スコープの
    /// みが対象であるため正常に構築できる（設計書 §3.1 直下子スコープ規約）。
    #[test]
    fn nested_keyed_list_is_allowed() {
        let inner = keyed_list(
            "ul",
            vec![],
            "children",
            vec![("c1".to_string(), el("li", vec![], vec![text("c1")]))],
        )
        .expect("valid inner keyed list");

        let outer = keyed_list(
            "div",
            vec![],
            "groups",
            vec![("g1".to_string(), el("li", vec![], vec![inner]))],
        )
        .expect("valid outer keyed list");

        let html = render(&outer);
        assert!(html.contains(r#"data-bind-list="groups""#));
        assert!(html.contains(r#"data-bind-list="children""#));
        assert!(html.contains(r#"data-key="g1""#));
        assert!(html.contains(r#"data-key="c1""#));
    }

    /// 異常系: 空文字列キーは EmptyKey で拒否される。
    #[test]
    fn empty_key_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![(String::new(), el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::EmptyKey { index: 0 });
    }

    /// 異常系: 同一リスト内のキー重複は DuplicateKey で拒否される。
    #[test]
    fn duplicate_key_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![
                ("a".to_string(), el("li", vec![], vec![])),
                ("b".to_string(), el("li", vec![], vec![])),
                ("a".to_string(), el("li", vec![], vec![])),
            ],
        )
        .unwrap_err();
        assert_eq!(
            err,
            KeyedListError::DuplicateKey {
                first_index: 0,
                duplicate_index: 2,
            }
        );
    }

    /// 異常系: 子が Element でない（Text）場合は NonElementItem で拒否される。
    #[test]
    fn non_element_item_is_rejected() {
        let err =
            keyed_list("ul", vec![], "items", vec![("a".to_string(), text("bare"))]).unwrap_err();
        assert_eq!(err, KeyedListError::NonElementItem { index: 0 });
    }

    /// 異常系: 親属性に予約属性 data-bind-list を渡すと ReservedAttr で拒否
    /// される（マーカー属性の偽装防止）。
    #[test]
    fn reserved_attr_on_parent_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![(BIND_LIST_ATTR, "fake")],
            "items",
            vec![("a".to_string(), el("li", vec![], vec![]))],
        )
        .unwrap_err();
        assert_eq!(
            err,
            KeyedListError::ReservedAttr {
                attr: BIND_LIST_ATTR
            }
        );
    }

    /// 異常系: 子要素の属性に予約属性 data-key を渡すと ReservedAttr で拒否
    /// される。
    #[test]
    fn reserved_attr_on_item_is_rejected() {
        let err = keyed_list(
            "ul",
            vec![],
            "items",
            vec![("a".to_string(), el("li", vec![(KEY_ATTR, "fake")], vec![]))],
        )
        .unwrap_err();
        assert_eq!(err, KeyedListError::ReservedAttr { attr: KEY_ATTR });
    }

    /// PoC-5 相当デモ: `interactive` クレートの list_section（項目 + 削除
    /// ボタン、`data-action="remove_item"`/`data-payload`）と同型の構造を
    /// `keyed_list` で構築できることを固定する（受け入れ条件 3 の証跡）。
    #[test]
    fn poc5_style_dynamic_list_is_expressible_as_keyed_list() {
        let raw_items = ["牛乳を買う".to_string(), "掃除する".to_string()];
        let items: Vec<(String, Node)> = raw_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let key = i.to_string();
                (
                    key.clone(),
                    li(
                        vec![],
                        vec![
                            text(item.clone()),
                            el(
                                "button",
                                vec![
                                    ("data-action", "remove_item"),
                                    ("data-payload", &key),
                                    ("data-testid", "remove-btn"),
                                ],
                                vec![text("削除")],
                            ),
                        ],
                    ),
                )
            })
            .collect();

        let list = keyed_list("ul", vec![("data-testid", "item-list")], "items", items).unwrap();
        let html = render(&list);

        assert!(html.contains(r#"data-bind-list="items""#));
        assert!(html.contains(r#"data-key="0""#));
        assert!(html.contains(r#"data-key="1""#));
        assert!(html.contains(r#"data-action="remove_item""#));
        assert!(html.contains("牛乳を買う"));
    }

    /// 非影響回帰: `keyed_list` を使わない既存ノード構築の `render()` 出力が
    /// バイト不変であることを固定する（#342 の同旨テストと対をなす）。
    #[test]
    fn existing_node_construction_output_is_unaffected() {
        let tree = ul(
            vec![],
            vec![
                li(vec![], vec![text("item1")]),
                li(vec![], vec![text("item2")]),
            ],
        );
        assert_eq!(render(&tree), "<ul><li>item1</li><li>item2</li></ul>");
    }
}
