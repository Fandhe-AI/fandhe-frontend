//! keyed list の DOM 適用: wasm32 配線層（イシュー #345）。
//!
//! [`crate::keyed_diff`]（DOM 非依存の純粋 diff 層）が計画した操作列
//! （[`crate::keyed_diff::KeyedOp`]）を実 DOM（`web-sys`）へ適用する。
//! `set_inner_html` / `insert_adjacent_html` / `raw_html` を**一切呼ばない**
//! （`crate::binding_dom` と同じ不変条件 1・2・4）。挿入ノードは
//! `fandhe_frontend_core::Node` 木から `create_element`/`set_text_content`/`append_child`
//! でプログラム的に構築し、移動は既存ノード参照を保持したまま
//! `insert_before` のみで行う（既存 DOM ノードを再生成しないことが
//! フォーカス・入力途中の値の保持に直結する、設計書 §5.3）。
//!
//! `fandhe_frontend_core::Node::RawHtml` を含む部分木は `web_sys::console` へ
//! 英語固定文言の警告を出したうえで、その `RawHtml` ノードを含む部分木
//! **全体**を構築失敗（`None`）として呼び出し元へ伝播する（本経路に
//! エスケープ迂回を組み込まない、設計書 §9 不変条件 4）。祖先ノードの
//! `build_dom_node_with_namespace` は子 1 件の `None` を無言で読み飛ばして
//! 残りだけを DOM へ反映することはせず、`?` で自身も `None` を返す
//! （`crate::subtree::replace_subtree` が要求する「`RawHtml` が部分木の
//! どこに現れても DOM を一切変更せず `Err` を返す」fail-closed 契約、
//! `lib.rs` クレート冒頭不変条件 7）。`apply_keyed_list` から見ると、この
//! 伝播の結果 `KeyedOp::Insert` の対象アイテムが丸ごと未適用のまま残る
//! （個別ノード単位ではなくアイテム単位の skip、イシュー #1121）。

use fandhe_frontend_core::keyed::KEY_ATTR;
use fandhe_frontend_core::Node;
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
            .any(|(k, v)| k == fandhe_frontend_core::keyed::BIND_LIST_ATTR && v == list_field)
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
    let selector = format!(
        "[{}=\"{field}\"]",
        fandhe_frontend_core::keyed::BIND_LIST_ATTR
    );
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

/// `fandhe_frontend_core::Node` から実 DOM 要素をプログラム的に構築する
/// （`create_element`/`set_text_content`/`append_child` の再帰、
/// `innerHTML`/`insert_adjacent_html` 不使用）。
///
/// `Node::RawHtml` を含む部分木は `console` へ警告したうえで部分木全体を
/// 構築失敗として呼び出し元へ伝播する fail-closed（不変条件 4）。属性名・
/// タグ名は `fandhe_frontend_interactive::AppState::view()` 等
/// 呼び出し側の `&'static str`/コンパイル時に固定された文字列であることを
/// 前提とするが、`set_attribute` 自体は DOM 標準 API であり `on*` 属性名を
/// 渡した場合でも `setAttribute` はイベントハンドラを実行コード化しない
/// （`element.onclick = ...` とは異なる。属性値は `escape_html` を経由しない
/// 生文字列だが、`setAttribute`/`set_text_content` は HTML パースを行わない
/// ため breakout 系 XSS 経路にならない）。
///
/// ただし URL スキーム経由の XSS（`href="javascript:..."` 等）は breakout を
/// 伴わないため上記の理由では防げない。`render_into`（fandhe-frontend-core）・
/// `binding_dom.rs` と同一の URL 検証（`srcset` のカンマ区切り候補分割検証
/// を含む）・イベントハンドラ属性ブロックを本経路にも適用する
/// （イシュー #373。`docs/policy/attribute-output-policy.md`）。
///
/// [`apply_keyed_list`]（本モジュール内、keyed list 挿入）に加え、
/// `fandhe-frontend-wasm-full` の遷移描画（`nav.rs`、イシュー #374）からも
/// `fandhe_frontend_wasm_client::build_dom_node` として呼ばれる公開 API（`lib.rs` の
/// `pub use keyed_dom::build_dom_node` 経由）。挿入先で `RawHtml` を `None`
/// として fail-closed に拒否する契約は呼び出し元を問わず不変（不変条件 4
/// を遷移経路にも継承）。
///
/// 祖先の名前空間を持たない単独呼び出し（HTML 名前空間が既定）向けの薄い
/// ラッパー。`svg`/`path` 等 SVG 要素を含む挿入は [`build_dom_node_with_namespace`]
/// （本モジュール内部の [`apply_keyed_list`] が使う）が名前空間を明示的に
/// 引き継ぐ。
pub fn build_dom_node(document: &Document, node: &Node) -> Option<web_sys::Node> {
    build_dom_node_with_namespace(document, node, None)
}

/// SVG 要素の名前空間 URI
/// （[MDN: Element.namespaceURI](https://developer.mozilla.org/en-US/docs/Web/API/Element/namespaceURI)）。
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// [`build_dom_node`] の名前空間対応版（内部実装、イシュー #843 Bugbot
/// 指摘「Runtime skips stroke DOM updates」是正の一部）。
///
/// `Document::create_element` は常に HTML 名前空間の要素を生成する。HTML
/// パーサは `<svg>` 配下を自動的に SVG 名前空間へ切り替えるが、
/// `create_element` によるプログラム的構築にはその挙動がないため、素朴に
/// `create_element("path")` すると HTML 名前空間の（ブラウザが SVG として
/// 描画しない）要素になってしまう。`crates/headless-ui/src/signature_pad.rs`
/// の SignaturePad が keyed list（`data-bind-list="strokes"`、親は
/// `<svg>`）でストロークを追加する経路がこの不具合の初出であり、
/// [`apply_keyed_list`] の `KeyedOp::Insert` は挿入先 `list_element` の
/// 実際の名前空間（`Element::namespace_uri()`）を `namespace` 引数へ渡す。
///
/// `tag` 自体が `"svg"` の場合は、渡された `namespace`（祖先から引き継いだ
/// 値）に関わらず SVG 名前空間へ切り替える。これにより `<svg>` 要素それ
/// 自体を含むノード木をまるごと新規構築するケース（`fandhe-frontend-wasm-full`
/// の遷移描画 `nav.rs` 等、`namespace` が `None` から始まる呼び出し）でも
/// HTML パーサと同じ挙動を再現する。決定した名前空間は子孫へそのまま
/// 引き継ぐ（`foreignObject` 等での名前空間の再切り替えは本経路のスコープ
/// 外、SignaturePad の SVG 構造には現れない）。
fn build_dom_node_with_namespace(
    document: &Document,
    node: &Node,
    namespace: Option<&str>,
) -> Option<web_sys::Node> {
    match node {
        Node::Text(text) => Some(document.create_text_node(text).into()),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let element_namespace = if *tag == "svg" {
                Some(SVG_NAMESPACE)
            } else {
                namespace
            };
            let element = match element_namespace {
                Some(ns) => document.create_element_ns(Some(ns), tag).ok()?,
                None => document.create_element(tag).ok()?,
            };
            for (name, value) in attrs {
                if fandhe_frontend_core::is_event_handler_attr(name) {
                    // イベントハンドラ属性は一律出力しない（不変条件 9 と同一）。
                    continue;
                }
                if fandhe_frontend_core::is_url_attr(name)
                    && !fandhe_frontend_core::is_safe_url(value)
                {
                    // 危険スキームの URL 属性は書き込まない（fail-closed）。
                    continue;
                }
                // `srcset` はカンマ区切りの URL 候補を持つ特殊構文のため
                // `is_url_attr` の対象外。`render_into`/`binding_dom.rs` と
                // 同一の `is_safe_srcset` で候補分割検証する
                // （イシュー #373 レビュー指摘対応）。
                if name.eq_ignore_ascii_case("srcset")
                    && !fandhe_frontend_core::is_safe_srcset(value)
                {
                    continue;
                }
                let _ = element.set_attribute(name, value);
            }
            // 子孫のいずれかが `RawHtml`（あるいは要素生成失敗）で `None` を
            // 返した場合、その子だけを無言で読み飛ばさず部分木全体を
            // 構築失敗として呼び出し元へ伝播する（fail-closed）。
            // `crate::subtree::replace_subtree` はこの関数を「`RawHtml` が
            // 部分木のどこに現れても DOM を一切変更せず `Err` を返す」契約
            // （不変条件 7）で呼ぶため、ここで子 1 件だけをスキップして
            // `Some` を返すと、`div` の子が `raw_html()` のケースのように
            // 危険なノードだけを除いた「一部だけ反映された」DOM が挿入され
            // てしまい契約違反になる。
            let mut built_children = Vec::with_capacity(children.len());
            for child in children {
                let child_node = build_dom_node_with_namespace(document, child, element_namespace)?;
                built_children.push(child_node);
            }
            for child_node in &built_children {
                let _ = element.append_child(child_node);
            }
            Some(element.into())
        }
        Node::RawHtml(_) => {
            // keyed list 経由の挿入ノードに raw_html を混入させる経路を
            // 構造的に持たない（設計書 §9 不変条件 4）。内容は含めない
            // 固定英語文言でログのみ残す（不変条件 6）。
            web_sys::console::warn_1(
                &"fandhe-frontend-wasm-client: keyed_dom skipped a RawHtml node (unsupported in keyed list insertion)".into(),
            );
            None
        }
    }
}

/// [`crate::keyed_apply::KeyedListDom`] の `web-sys` 実装アダプタ
/// （イシュー #1318）。
///
/// 走査アルゴリズム自体（旧 `dom_item_keys`/`find_child_by_key`/
/// `nth_element_child`/op 適用ループ）は [`crate::keyed_apply::apply_ops`]
/// へ等価移植済みであり、本 struct は「`web-sys` の実 DOM 呼び出し」を
/// トレイトメソッドへ 1:1 で委譲するだけの薄いアダプタに徹する
/// （本モジュール冒頭 doc の 2 層構成、`keyed_apply` モジュール doc 参照）。
struct WebSysKeyedDom<'a> {
    document: &'a Document,
    list_element: &'a Element,
    /// `new_list_node`（`component.view()` 側の `Node` 木）から抽出した
    /// `(key, &Node)` 列。`create_item` がキー引きでノードを探す。
    new_items: &'a [(String, &'a Node)],
    /// 挿入先 `list_element` の実際の名前空間（[`build_dom_node_with_namespace`]
    /// rustdoc 参照。SVG keyed list への挿入で HTML 名前空間の要素が生成
    /// されてしまう不具合の是正を維持する）。
    namespace: Option<&'a str>,
}

impl crate::keyed_apply::KeyedListDom for WebSysKeyedDom<'_> {
    type Handle = Element;
    type NewNode = web_sys::Node;

    fn first_element_child(&mut self) -> Option<Element> {
        self.list_element.first_element_child()
    }

    fn next_element_sibling(&mut self, child: &Element) -> Option<Element> {
        child.next_element_sibling()
    }

    fn item_key(&mut self, child: &Element) -> Option<String> {
        child.get_attribute(KEY_ATTR)
    }

    fn create_item(&mut self, key: &str) -> Option<web_sys::Node> {
        let (_, node) = self.new_items.iter().find(|(k, _)| k == key)?;
        build_dom_node_with_namespace(self.document, node, self.namespace)
    }

    fn insert_before(&mut self, node: web_sys::Node, reference: Option<&Element>) {
        let reference_web_node: Option<web_sys::Node> =
            reference.cloned().map(|el| el.unchecked_into());
        let _ = self
            .list_element
            .insert_before(&node, reference_web_node.as_ref());
    }

    fn move_before(&mut self, child: &Element, reference: Option<&Element>) {
        // 移動元と移動先参照が同一要素の場合、`insert_before` は no-op
        // （DOM 標準仕様: 挿入前に自身を除去してから挿入するため同一ノード
        // 指定は何も動かさない）。
        let reference_web_node: Option<web_sys::Node> =
            reference.cloned().map(|el| el.unchecked_into());
        let existing_web_node: web_sys::Node = child.clone().unchecked_into();
        let _ = self
            .list_element
            .insert_before(&existing_web_node, reference_web_node.as_ref());
    }

    fn remove_child(&mut self, child: &Element) {
        let _ = self.list_element.remove_child(child);
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
/// 残す（fail-closed。他の正当な操作の適用を妨げない）。走査アルゴリズム
/// 本体は [`crate::keyed_apply::apply_ops`]（イシュー #1318 で DOM 非依存へ
/// 切り出し済み、native `cargo test` で DOM 操作コストを決定的に検証する
/// 土台）。
pub fn apply_keyed_list(document: &Document, list_element: &Element, new_list_node: &Node) {
    let new_items = list_item_nodes(new_list_node);
    let new_keys: Vec<String> = new_items.iter().map(|(k, _)| k.clone()).collect();
    let namespace = list_element.namespace_uri();

    let mut dom = WebSysKeyedDom {
        document,
        list_element,
        new_items: &new_items,
        namespace: namespace.as_deref(),
    };
    crate::keyed_apply::apply_ops(&mut dom, &new_keys);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, keyed::keyed_list, li, text};
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

    /// `srcset` はカンマ区切りの URL 候補を持つ特殊構文のため `URL_ATTRS`
    /// （単一 URL 属性の正リスト）に非該当。候補の 1 件でも危険スキームを
    /// 含む場合、`build_dom_node` が `is_safe_srcset` による候補分割検証を
    /// 経由して `srcset` 属性そのものを書き込まないこと（イシュー #373
    /// レビュー指摘対応: keyed list 経由のプログラム的ノード構築でも
    /// `render_into`/`binding_dom.rs` と同一の保証を持たせる契約の実ブラウザ
    /// 証跡）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_drops_srcset_when_a_candidate_has_a_dangerous_scheme() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let items: Vec<(String, Node)> = vec![(
            "x".to_string(),
            li(
                vec![],
                vec![el(
                    "img",
                    vec![("srcset", "/safe.png 1x, javascript:alert(1) 2x")],
                    vec![],
                )],
            ),
        )];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let img = list_element.query_selector("img").unwrap().unwrap();
        assert!(
            img.get_attribute("srcset").is_none(),
            "srcset 候補の 1 件でも危険スキームを含む場合、属性全体が \
             書き込まれないこと（fail-closed）"
        );
    }

    /// 全候補が安全な URL である `srcset` は従来どおり反映されること
    /// （過剰ブロックでないことの確認）。
    #[wasm_bindgen_test]
    fn apply_keyed_list_keeps_srcset_when_all_candidates_are_safe() {
        let document = doc();
        let list_element = make_list_element(&document, &[]);

        let items: Vec<(String, Node)> = vec![(
            "x".to_string(),
            li(
                vec![],
                vec![el("img", vec![("srcset", "/a.png 1x, /b.png 2x")], vec![])],
            ),
        )];
        let new_tree = keyed_list("ul", vec![], "items", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let img = list_element.query_selector("img").unwrap().unwrap();
        assert_eq!(
            img.get_attribute("srcset").as_deref(),
            Some("/a.png 1x, /b.png 2x"),
            "全候補が安全な URL の srcset は反映されること"
        );
    }

    // --- SVG 名前空間（イシュー #843 Bugbot 指摘「Runtime skips stroke
    // DOM updates」の根本原因の 1 つ、`SignaturePad` の keyed list ストローク
    // 挿入回帰固定） ---

    /// SVG 名前空間の `<svg data-bind-list="strokes">` 親要素へ keyed list
    /// 挿入した `<path>` 子要素が、`document.create_element` 由来の HTML
    /// 名前空間ではなく SVG 名前空間で生成されること。
    #[wasm_bindgen_test]
    fn apply_keyed_list_inserts_svg_children_in_svg_namespace() {
        let document = doc();
        let list_element = document
            .create_element_ns(Some(SVG_NAMESPACE), "svg")
            .unwrap();
        list_element
            .set_attribute(fandhe_frontend_core::keyed::BIND_LIST_ATTR, "strokes")
            .unwrap();

        let items: Vec<(String, Node)> = vec![(
            "0".to_string(),
            el("path", vec![("d", "M0.00,0.00 L1.00,1.00")], vec![]),
        )];
        let new_tree = keyed_list("svg", vec![], "strokes", items).unwrap();
        apply_keyed_list(&document, &list_element, &new_tree);

        let path = list_element.query_selector("path").unwrap().unwrap();
        assert_eq!(
            path.namespace_uri().as_deref(),
            Some(SVG_NAMESPACE),
            "SVG keyed list へ挿入された <path> は SVG 名前空間で生成される \
             こと（HTML 名前空間だとブラウザが SVG として描画しない）"
        );
    }

    /// [`build_dom_node`]（公開 API）が `<svg>` 自体をルートとするノード木を
    /// 新規構築する場合も、`<svg>`・その子孫（`<path>`）の双方が SVG 名前
    /// 空間で生成されること（`fandhe-frontend-wasm-full` の遷移描画
    /// `nav.rs` のようにノード木をまるごと構築する呼び出し経路の回帰固定）。
    #[wasm_bindgen_test]
    fn build_dom_node_creates_svg_subtree_in_svg_namespace() {
        let document = doc();
        let svg_node = el(
            "svg",
            vec![("viewBox", "0 0 300 150")],
            vec![el("path", vec![("d", "M0.00,0.00 L1.00,1.00")], vec![])],
        );
        let built = build_dom_node(&document, &svg_node).unwrap();
        let svg_element: Element = built.unchecked_into();
        assert_eq!(svg_element.namespace_uri().as_deref(), Some(SVG_NAMESPACE));

        let path = svg_element.query_selector("path").unwrap().unwrap();
        assert_eq!(path.namespace_uri().as_deref(), Some(SVG_NAMESPACE));
    }
}
