//! keyed list の DOM 適用: wasm32 配線層（イシュー #345）。
//!
//! [`crate::keyed_diff`]（DOM 非依存の純粋 diff 層）が計画した操作列
//! （[`crate::keyed_diff::KeyedOp`]）を実 DOM（`web-sys`）へ適用する。
//! `set_inner_html` / `insert_adjacent_html` / `raw_html` を**一切呼ばない**
//! （`crate::binding_dom` と同じ不変条件 1・2・4）。挿入ノードは
//! `rws_core::Node` 木から `create_element`/`set_text_content`/`append_child`
//! でプログラム的に構築し、移動は既存ノード参照を保持したまま
//! `insert_before` のみで行う（既存 DOM ノードを再生成しないことが
//! フォーカス・入力途中の値の保持に直結する、設計書 §5.3）。
//!
//! `rws_core::Node::RawHtml` 子は fail-closed で skip し、
//! `web_sys::console` へ英語固定文言の警告を出す（本経路にエスケープ迂回を
//! 組み込まない、設計書 §9 不変条件 4）。

use crate::keyed_diff::{diff_keys, KeyedOp};
use rws_core::keyed::KEY_ATTR;
use rws_core::Node;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

/// `list_field` の keyed list 親要素（`data-bind-list="<field>"`）を新しい
/// `Node` 木から探す（純粋ヘルパー、DOM 非依存）。
///
/// `component.view()` の出力木全体から深さ優先で探索する。見つからない
/// 場合は `None`（呼び出し側は当該フィールドの更新を no-op とする、
/// fail-closed）。
pub fn find_keyed_list_node<'a>(node: &'a Node, list_field: &str) -> Option<&'a Node> {
    if let Node::Element { attrs, .. } = node {
        if attrs
            .iter()
            .any(|(k, v)| k == rws_core::keyed::BIND_LIST_ATTR && v == list_field)
        {
            return Some(node);
        }
    }
    if let Node::Element { children, .. } = node {
        for child in children {
            if let Some(found) = find_keyed_list_node(child, list_field) {
                return Some(found);
            }
        }
    }
    None
}

/// 実 DOM 上で `field` に対応する keyed list 親要素
/// （`[data-bind-list="<field>"]`）を探す。`root` 配下を 1 度だけ
/// `query_selector` で探索する（`crate::binding_dom::BindingTable::scan` と
/// 同様、DOM 改ザン等による予期しない複数一致は最初の 1 件を採用する）。
///
/// # Errors
///
/// `query_selector` 自体が失敗した場合（不正なセレクタ文字列等、通常は
/// 到達しない）に `Err` を返す。要素が見つからない場合は `Ok(None)`。
pub fn find_list_element(
    root: &Element,
    field: &str,
) -> Result<Option<Element>, wasm_bindgen::JsValue> {
    let selector = format!("[{}=\"{field}\"]", rws_core::keyed::BIND_LIST_ATTR);
    root.query_selector(&selector)
        .map_err(|_| wasm_bindgen::JsValue::from_str("query_selector failed for keyed list"))
}

/// keyed list 親要素（`Node::Element`）の直下の子から `(key, &Node)` 列を
/// 取り出す（`data-key` 属性を持たない子・非 Element 子は fail-closed で
/// skip する）。
fn list_item_nodes(list_node: &Node) -> Vec<(String, &Node)> {
    let Node::Element { children, .. } = list_node else {
        return Vec::new();
    };
    children
        .iter()
        .filter_map(|child| {
            let Node::Element { attrs, .. } = child else {
                return None;
            };
            attrs
                .iter()
                .find(|(k, _)| k == KEY_ATTR)
                .map(|(_, key)| (key.clone(), child))
        })
        .collect()
}

/// 実 DOM 上の keyed list 親要素直下の子から、現在の `data-key` 列を読み出す。
fn dom_item_keys(list_element: &Element) -> Vec<String> {
    let mut keys = Vec::new();
    let mut maybe_child = list_element.first_element_child();
    while let Some(child) = maybe_child {
        if let Some(key) = child.get_attribute(KEY_ATTR) {
            keys.push(key);
        }
        maybe_child = child.next_element_sibling();
    }
    keys
}

/// `key` に対応する既存の子要素を探す（`data-key` 属性の完全一致）。
fn find_child_by_key(list_element: &Element, key: &str) -> Option<Element> {
    let mut maybe_child = list_element.first_element_child();
    while let Some(child) = maybe_child {
        if child.get_attribute(KEY_ATTR).as_deref() == Some(key) {
            return Some(child);
        }
        maybe_child = child.next_element_sibling();
    }
    None
}

/// `index` 番目（0-origin）の子要素を返す（`insert_before` の参照ノード
/// 決定に使う。`index` が子要素数以上なら `None` = 末尾追加）。
fn nth_element_child(list_element: &Element, index: usize) -> Option<Element> {
    let mut maybe_child = list_element.first_element_child();
    let mut i = 0;
    while let Some(child) = maybe_child {
        if i == index {
            return Some(child);
        }
        i += 1;
        maybe_child = child.next_element_sibling();
    }
    None
}

/// `rws_core::Node` から実 DOM 要素をプログラム的に構築する
/// （`create_element`/`set_text_content`/`append_child` の再帰、
/// `innerHTML`/`insert_adjacent_html` 不使用）。
///
/// `Node::RawHtml` 子は fail-closed で skip し `console` へ警告する
/// （不変条件 4）。属性名・タグ名は `rws_interactive::AppState::view()` 等
/// 呼び出し側の `&'static str`/コンパイル時に固定された文字列であることを
/// 前提とするが、`set_attribute` 自体は DOM 標準 API であり `on*` 属性名を
/// 渡した場合でも `setAttribute` はイベントハンドラを実行コード化しない
/// （`element.onclick = ...` とは異なる。属性値は `escape_html` を経由しない
/// 生文字列だが、`setAttribute`/`set_text_content` は HTML パースを行わない
/// ため XSS 経路にならない）。
///
/// [`apply_keyed_list`]（本モジュール内、keyed list 挿入）に加え、
/// `rws-wasm-full` の遷移描画（`nav.rs`、イシュー #374）からも
/// `rws_wasm_client::build_dom_node` として呼ばれる公開 API（`lib.rs` の
/// `pub use keyed_dom::build_dom_node` 経由）。挿入先で `RawHtml` を `None`
/// として fail-closed に拒否する契約は呼び出し元を問わず不変（不変条件 4
/// を遷移経路にも継承）。
pub fn build_dom_node(document: &Document, node: &Node) -> Option<web_sys::Node> {
    match node {
        Node::Text(text) => Some(document.create_text_node(text).into()),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let element = document.create_element(tag).ok()?;
            for (name, value) in attrs {
                let _ = element.set_attribute(name, value);
            }
            for child in children {
                if let Some(child_node) = build_dom_node(document, child) {
                    let _ = element.append_child(&child_node);
                }
            }
            Some(element.into())
        }
        Node::RawHtml(_) => {
            // keyed list 経由の挿入ノードに raw_html を混入させる経路を
            // 構造的に持たない（設計書 §9 不変条件 4）。内容は含めない
            // 固定英語文言でログのみ残す（不変条件 6）。
            web_sys::console::warn_1(
                &"rws-wasm-client: keyed_dom skipped a RawHtml node (unsupported in keyed list insertion)".into(),
            );
            None
        }
    }
}

/// [`crate::keyed_diff::diff_keys`] が計画した操作列を `list_element` へ
/// 適用する（本モジュールの公開エントリポイント）。
///
/// `new_list_node` は `component.view()` が返す木のうち、
/// [`find_keyed_list_node`] で特定した `field` の keyed list 親ノード
/// （呼び出し側で特定済みのものを渡す設計。`wasm-full::Runtime` が
/// `dirty_fields()` に含まれる keyed list 対象 field ごとに本関数を呼ぶ
/// 想定）。`list_element` は実 DOM 上の対応する親要素（`data-bind-list`
/// で走査済み）。
///
/// キー照合に失敗する要素（`Insert` で `build_dom_node` が `None` を返す
/// ケース = `RawHtml` 子や不正タグ名）は skip し、当該 1 件のみ未適用のまま
/// 残す（fail-closed。他の正当な操作の適用を妨げない）。
pub fn apply_keyed_list(document: &Document, list_element: &Element, new_list_node: &Node) {
    let new_items = list_item_nodes(new_list_node);
    let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();
    let old_keys = dom_item_keys(list_element);

    let ops = diff_keys(&old_keys, &new_keys);
    for op in ops {
        match op {
            KeyedOp::Remove { key } => {
                if let Some(child) = find_child_by_key(list_element, &key) {
                    let _ = list_element.remove_child(&child);
                }
            }
            KeyedOp::Insert { index, key } => {
                let Some((_, node)) = new_items.iter().find(|(k, _)| k == &key) else {
                    continue;
                };
                let Some(new_child) = build_dom_node(document, node) else {
                    continue;
                };
                let reference = nth_element_child(list_element, index);
                let reference_web_node: Option<web_sys::Node> =
                    reference.map(|el| el.unchecked_into());
                let _ = list_element.insert_before(&new_child, reference_web_node.as_ref());
            }
            KeyedOp::Move { index, key } => {
                let Some(existing) = find_child_by_key(list_element, &key) else {
                    continue;
                };
                let reference = nth_element_child(list_element, index);
                // 移動元と移動先参照が同一要素の場合、`insert_before` は
                // no-op（DOM 標準仕様: 挿入前に自身を除去してから挿入する
                // ため同一ノード指定は何も動かさない）。
                let reference_web_node: Option<web_sys::Node> =
                    reference.map(|el| el.unchecked_into());
                let existing_web_node: web_sys::Node = existing.unchecked_into();
                let _ = list_element.insert_before(&existing_web_node, reference_web_node.as_ref());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rws_core::{keyed::keyed_list, li, text};
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn doc() -> Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn make_list_element(doc: &Document, keys: &[&str]) -> Element {
        let ul = doc.create_element("ul").unwrap();
        for key in keys {
            let li = doc.create_element("li").unwrap();
            li.set_attribute(KEY_ATTR, key).unwrap();
            li.set_text_content(Some(key));
            ul.append_child(&li).unwrap();
        }
        ul
    }

    fn keyed_items(keys: &[&str]) -> Node {
        let items: Vec<(String, Node)> = keys
            .iter()
            .map(|k| (k.to_string(), li(vec![], vec![text(*k)])))
            .collect();
        keyed_list("ul", vec![], "items", items).expect("valid keyed list")
    }

    /// 挿入: 末尾追加で既存ノードの同一性が保たれる（フォーカス保持の土台）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_appends_new_item_without_touching_existing_nodes() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let existing_first = list_element.first_element_child().unwrap();

        let new_tree = keyed_items(&["a", "b", "c"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 3);
        assert!(existing_first.is_same_node(Some(&list_element.first_element_child().unwrap())));
        let last = list_element.children().item(2).unwrap();
        assert_eq!(last.get_attribute(KEY_ATTR).as_deref(), Some("c"));
    }

    /// 削除: 中間項目のみが除去され、他ノードは同一参照のまま残る。
    #[wasm_bindgen_test]
    fn apply_keyed_list_removes_middle_item_only() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b", "c"]);

        let new_tree = keyed_items(&["a", "c"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 2);
        let keys: Vec<Option<String>> = (0..2)
            .map(|i| {
                list_element
                    .children()
                    .item(i)
                    .unwrap()
                    .get_attribute(KEY_ATTR)
            })
            .collect();
        assert_eq!(keys, vec![Some("a".to_string()), Some("c".to_string())]);
    }

    /// 移動: 既存ノード参照を保持したまま並び替わる（再生成されないことを
    /// `is_same_node` で確認）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_moves_item_preserving_node_identity() {
        let document = doc();
        let list_element = make_list_element(&document, &["a", "b"]);
        let node_b = list_element.children().item(1).unwrap();

        let new_tree = keyed_items(&["b", "a"]);
        apply_keyed_list(&document, &list_element, &new_tree);

        let first = list_element.first_element_child().unwrap();
        assert!(first.is_same_node(Some(&node_b)));
        assert_eq!(first.get_attribute(KEY_ATTR).as_deref(), Some("b"));
    }

    /// XSS 構造的排除: script 文字列を含む項目を挿入しても script 要素は
    /// 生成されず、テキストとして安全に格納される（innerHTML 不使用の
    /// 回帰固定）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_script_like_text_as_plain_text_not_element() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let malicious = "<script>alert(1)</script>";
        let items: Vec<(String, Node)> = vec![("x".to_string(), li(vec![], vec![text(malicious)]))];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        assert_eq!(list_element.children().length(), 1);
        assert_eq!(list_element.query_selector("script").unwrap(), None);
        let li_el = list_element.first_element_child().unwrap();
        assert_eq!(li_el.text_content().as_deref(), Some(malicious));
    }
}
