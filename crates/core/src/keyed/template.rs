//! keyed list アイテムの構造同型判定と束縛点パス導出（イシュー #1384）。
//!
//! 親（#1383）が動機づける「静的骨格を 1 回だけ構築し、以後は
//! `cloneNode(true)` + 動的値の書き込みのみ」方式（solid / lit / vue 系の
//! テンプレート clone 方式）の**前段の純 Rust 部分**を担う。keyed list の
//! Insert 対象アイテム群が「同一構造（タグ・属性の並びが一致し、テキスト
//! 値のみ異なる）」かを判定し、動的値（テキスト）の位置を「アイテムルート
//! からの子インデックス列（束縛点パス）」として静的に導出する。
//!
//! # 責務境界
//!
//! - 本モジュールは [`crate::Node`] 木に対する**純 Rust 処理**のみを行い、
//!   JS 境界呼び出しはゼロ（`fandhe-frontend-wasm-client` への依存を持たない）。
//! - DOM への実適用（`clone_node` + 束縛点書き込み）は本モジュールの
//!   スコープ外（後続イシュー #1385、`fandhe-frontend-wasm-client` 側）。
//! - 判定は fail-safe: 行ごとに形が違う場合・想定外パターン（`RawHtml`
//!   混入・非 [`crate::Node::Element`] ルート等）は必ず [`None`] を返し、
//!   呼び出し側は従来経路（個別生成）へフォールバックする。誤って
//!   「同型」と判定する（false positive）ことは決してない設計とする。
//!
//! # クレート間契約（#1385 が前提とする内容）
//!
//! [`ItemTemplate::text_paths`] が返す各パスの子インデックスは
//! [`crate::Node::Element::children`] のインデックスである。
//! `fandhe-frontend-wasm-client` の `build_dom_node_with_namespace` は
//! 「`Node` 子 1 件 = DOM 子 1 件」の 1:1 対応で DOM を構築する契約
//! （子 1 件でも構築に失敗すれば部分木全体が失敗する fail-closed）を
//! 持つため、本モジュールが導出するパスは #1385 が `first_child`/
//! `next_sibling` で DOM を辿るときの位置と同一になる。この対応は
//! [`Node::RawHtml`]（0 個以上の DOM ノードへ展開され 1:1 対応を壊す
//! 唯一の腕）を同型判定で即不成立にすることで構造的に保たれる。
//!
//! 本モジュールは HTML を一切生成・出力せず、束縛点は
//! [`crate::Node::Text`] の位置のみを指す。テキスト値の DOM への反映は
//! DOM Text ノードの値設定（`innerHTML` 不使用）を前提とし、既定エスケープ
//! （REQ-1）の迂回経路を新設しない。

use crate::keyed::KEY_ATTR;
use crate::Node;

/// アイテム群から導出した行テンプレート。
///
/// 先頭アイテムを深さ優先・前順（文書順）で走査して得た、テキスト束縛点
/// （[`crate::Node::Text`]）の位置集合。[`derive_item_template`] が同型
/// 判定成立時にのみ構築する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTemplate {
    /// テキスト束縛点パス列（文書順）。各要素はアイテムルートからの
    /// 子インデックス列（[`crate::Node::Element::children`] のインデックス）。
    text_paths: Vec<Vec<usize>>,
}

impl ItemTemplate {
    /// テキスト束縛点パス列（文書順）を返す。
    ///
    /// `#1385` はこのパス列を DOM 側の `first_child`/`next_sibling` 走査へ
    /// そのまま対応させる契約（本モジュール doc「クレート間契約」節参照）。
    pub fn text_paths(&self) -> &[Vec<usize>] {
        &self.text_paths
    }
}

/// items（keyed_list アイテムの [`Node`] 参照列）から同型テンプレートを
/// 導出する。
///
/// 先頭アイテムを基準に、2 件目以降を対でロックステップ再帰比較する
/// （逐次比較。ハッシュテーブルは使わず、衝突による false positive が
/// 原理的に存在しない）。不成立（非同型混在・[`Node::RawHtml`] 混入・
/// 非 [`Node::Element`] ルート・空列）は [`None`] を返す（本モジュール
/// doc「責務境界」節の fail-safe 方針）。
///
/// 同型判定規則:
/// 1. [`Node::Element`] 同士: `tag` 一致、`attrs` は長さ・並び順・属性名・
///    属性値がすべて一致。ただし**ルート（深さ 0）の [`KEY_ATTR`] の値のみ
///    差異を許容**する（`keyed_list` が各アイテムルート末尾へ付与する
///    行固有値のため）。ルート以外の `KEY_ATTR` は値も一致必須。子は
///    同数で対ごとに再帰。
/// 2. [`Node::Text`] 同士: 常に同型（値は自由 = 束縛点）。
/// 3. [`Node::RawHtml`][Node::RawHtml]: どちらか一方でも出現したら即不成立。
/// 4. variant 不一致・ルートが非 [`Node::Element`]: 不成立。
pub fn derive_item_template(items: &[&Node]) -> Option<ItemTemplate> {
    let (first, rest) = items.split_first()?;

    // ルートは Element である必要がある（keyed_list の各アイテムは
    // 常に Element ルートで構築される契約、本関数 doc 規則 4）。
    if !matches!(first, Node::Element { .. }) {
        return None;
    }

    // 単一アイテム（rest が空）でも fail-safe 契約（RawHtml 混入の検知、
    // 本モジュール doc「責務境界」節）を必ず通すため、first を自分自身に
    // 対して同型判定する。`rest` を走査するペアワイズ比較のみに頼ると
    // items.len() == 1 のときループが 0 回になり、first の部分木に
    // RawHtml が含まれていても検知できず #1385 の Node-to-DOM 1:1 不変
    // 条件を破って Some を返してしまう（nodes_isomorphic は RawHtml 同士
    // の比較も variant 一致とはみなさず `_ => false` 腕で必ず不成立に
    // するため、自己比較でも正しく検知できる）。
    if !nodes_isomorphic(first, first, true) {
        return None;
    }

    for other in rest {
        if !nodes_isomorphic(first, other, true) {
            return None;
        }
    }

    let mut text_paths = Vec::new();
    let mut current_path = Vec::new();
    collect_text_paths(first, &mut current_path, &mut text_paths);

    Some(ItemTemplate { text_paths })
}

/// `a`・`b` が同型（タグ・属性・子構造が一致し、[`Node::Text`] の値のみ
/// 差異を許容）かを再帰判定する。`is_root` はルート要素の
/// [`KEY_ATTR`] 値差異を許容する特例のためのフラグ（本モジュール doc
/// 「同型判定規則」節の規則 1）。
fn nodes_isomorphic(a: &Node, b: &Node, is_root: bool) -> bool {
    match (a, b) {
        (Node::Text(_), Node::Text(_)) => true,
        (
            Node::Element {
                tag: tag_a,
                attrs: attrs_a,
                children: children_a,
            },
            Node::Element {
                tag: tag_b,
                attrs: attrs_b,
                children: children_b,
            },
        ) => {
            if tag_a != tag_b {
                return false;
            }
            if !attrs_isomorphic(attrs_a, attrs_b, is_root) {
                return false;
            }
            if children_a.len() != children_b.len() {
                return false;
            }
            children_a
                .iter()
                .zip(children_b.iter())
                .all(|(ca, cb)| nodes_isomorphic(ca, cb, false))
        }
        // RawHtml はどちらか一方でも出現したら即不成立（規則 3）。
        // variant 不一致（Text↔Element 等）も同じ腕でまとめて不成立。
        _ => false,
    }
}

/// 属性列 `a`・`b` が同型か判定する。長さ・並び順・属性名が一致し、
/// 属性値も一致することを要求する。ただし `is_root` かつ属性名が
/// [`KEY_ATTR`] のペアに限り値の差異を許容する（`keyed_list` がルート
/// アイテムへ付与する行固有キー値のため）。
fn attrs_isomorphic(a: &[(String, String)], b: &[(String, String)], is_root: bool) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|((na, va), (nb, vb))| {
        if na != nb {
            return false;
        }
        if is_root && na == KEY_ATTR {
            return true;
        }
        va == vb
    })
}

/// `node` を深さ優先・前順で走査し、[`Node::Text`] の出現位置
/// （ルートからの子インデックス列）を `out` へ文書順で記録する。
/// `path` は再帰の呼び出し元へ戻る際に必ず末尾要素を pop し、
/// 兄弟間で汚染しない。
fn collect_text_paths(node: &Node, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    match node {
        Node::Text(_) => out.push(path.clone()),
        Node::Element { children, .. } => {
            for (i, child) in children.iter().enumerate() {
                path.push(i);
                collect_text_paths(child, path, out);
                path.pop();
            }
        }
        Node::RawHtml(_) => {
            // derive_item_template は RawHtml 混入を事前に不成立扱いする
            // ため、ここへ到達するのは呼び出し側契約違反時のみ。
            // fail-safe として何も記録しない（束縛点扱いしない）。
        }
    }
}

/// `item` の各束縛点パス（`template.text_paths()`）を解決してテキスト値を
/// 文書順で返す。パスが [`Node::Text`] へ解決できなければ [`None`]
/// （構造がテンプレートと一致しない、防御的 fail-safe）。
///
/// `#1385` が DOM 側の書き込み値取得を再実装せずに済むための補助であり、
/// 往復性質テスト（`text_values` で取得した値を書き戻すと元アイテムに
/// 一致すること）の基盤にもなる。
pub fn text_values<'a>(item: &'a Node, template: &ItemTemplate) -> Option<Vec<&'a str>> {
    template
        .text_paths
        .iter()
        .map(|path| resolve_text_path(item, path))
        .collect()
}

/// `path`（アイテムルートからの子インデックス列）を辿り、末端の
/// [`Node::Text`] の値を返す。途中で子インデックスが範囲外・末端が
/// [`Node::Text`] でない場合は [`None`]。
fn resolve_text_path<'a>(node: &'a Node, path: &[usize]) -> Option<&'a str> {
    match path.split_first() {
        None => match node {
            Node::Text(s) => Some(s.as_str()),
            _ => None,
        },
        Some((&idx, rest)) => match node {
            Node::Element { children, .. } => resolve_text_path(children.get(idx)?, rest),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{el_owned as el, raw_html, text};

    fn row(key: &str, label: &str) -> Node {
        el(
            "tr",
            vec![(KEY_ATTR.to_string(), key.to_string())],
            vec![el("td", vec![], vec![text(label)])],
        )
    }

    // ---- 成立系 ----

    #[test]
    fn flat_multiple_text_isomorphic() {
        let a = el(
            "li",
            vec![],
            vec![text("a"), el("b", vec![], vec![text("x")])],
        );
        let b = el(
            "li",
            vec![],
            vec![text("c"), el("b", vec![], vec![text("y")])],
        );
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert_eq!(template.text_paths(), &[vec![0], vec![1, 0]]);
    }

    #[test]
    fn nested_two_levels() {
        let a = row("1", "hello");
        let b = row("2", "world");
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert_eq!(template.text_paths(), &[vec![0, 0]]);
        assert_eq!(text_values(&a, &template).unwrap(), vec!["hello"]);
        assert_eq!(text_values(&b, &template).unwrap(), vec!["world"]);
    }

    #[test]
    fn attrs_with_static_class_match() {
        let a = el(
            "div",
            vec![("class".to_string(), "row".to_string())],
            vec![text("a")],
        );
        let b = el(
            "div",
            vec![("class".to_string(), "row".to_string())],
            vec![text("b")],
        );
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_some());
    }

    #[test]
    fn root_key_attr_value_may_differ() {
        let a = row("key-1", "same");
        let b = row("key-2", "same");
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_some());
    }

    #[test]
    fn void_element_mixed() {
        let a = el("li", vec![], vec![el("br", vec![], vec![]), text("a")]);
        let b = el("li", vec![], vec![el("br", vec![], vec![]), text("b")]);
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert_eq!(template.text_paths(), &[vec![1]]);
    }

    #[test]
    fn zero_children_element() {
        let a = el("hr", vec![], vec![]);
        let b = el("hr", vec![], vec![]);
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert!(template.text_paths().is_empty());
    }

    #[test]
    fn consecutive_text_children() {
        let a = el("p", vec![], vec![text("a"), text("b")]);
        let b = el("p", vec![], vec![text("c"), text("d")]);
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert_eq!(template.text_paths(), &[vec![0], vec![1]]);
    }

    #[test]
    fn single_item_is_trivially_isomorphic() {
        let a = row("1", "solo");
        let items: Vec<&Node> = vec![&a];
        let template = derive_item_template(&items).expect("単一要素は常に同型");
        assert_eq!(template.text_paths(), &[vec![0, 0]]);
    }

    // ---- 不成立系 ----

    #[test]
    fn empty_items_is_none() {
        let items: Vec<&Node> = vec![];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn different_tag_is_none() {
        let a = el("div", vec![], vec![text("a")]);
        let b = el("span", vec![], vec![text("b")]);
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn different_attr_name_is_none() {
        let a = el(
            "div",
            vec![("class".to_string(), "x".to_string())],
            vec![text("a")],
        );
        let b = el(
            "div",
            vec![("id".to_string(), "x".to_string())],
            vec![text("b")],
        );
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn different_static_attr_value_is_none() {
        let a = el(
            "div",
            vec![("class".to_string(), "x".to_string())],
            vec![text("a")],
        );
        let b = el(
            "div",
            vec![("class".to_string(), "y".to_string())],
            vec![text("b")],
        );
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn different_attr_order_is_none() {
        let a = el(
            "div",
            vec![
                ("class".to_string(), "x".to_string()),
                ("id".to_string(), "y".to_string()),
            ],
            vec![text("a")],
        );
        let b = el(
            "div",
            vec![
                ("id".to_string(), "y".to_string()),
                ("class".to_string(), "x".to_string()),
            ],
            vec![text("b")],
        );
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn different_children_count_is_none() {
        let a = el("div", vec![], vec![text("a")]);
        let b = el("div", vec![], vec![text("a"), text("b")]);
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn text_element_swap_is_none() {
        let a = el("div", vec![], vec![text("a")]);
        let b = el("div", vec![], vec![el("span", vec![], vec![])]);
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn raw_html_mixed_is_none() {
        // ESCAPE-REVIEWED: RawHtml 混入時の不成立判定（規則 3）を検証する
        // テスト専用固定文字列。ユーザー入力を経由しない。
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: RawHtml 混入時の不成立判定を検証するテスト固定文字列、ユーザー入力なし"
        )]
        let a = el("div", vec![], vec![raw_html("<b>x</b>")]);
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: RawHtml 混入時の不成立判定を検証するテスト固定文字列、ユーザー入力なし"
        )]
        let b = el("div", vec![], vec![raw_html("<b>y</b>")]);
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn raw_html_only_one_side_is_none() {
        let a = el("div", vec![], vec![text("a")]);
        // ESCAPE-REVIEWED: 片側のみ RawHtml 混入時の不成立判定を検証する
        // テスト固定文字列、ユーザー入力なし。
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: 片側のみ RawHtml 混入時の不成立判定を検証するテスト固定文字列、ユーザー入力なし"
        )]
        let b = el("div", vec![], vec![raw_html("a")]);
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn raw_html_single_item_is_none() {
        // Bugbot 指摘（PR #1398）: items が単一要素だと nodes_isomorphic の
        // ペアワイズ比較（rest 走査）がループ 0 回で完全にスキップされ、
        // RawHtml を含む単一 Element ルートでも Some を返してしまう回帰。
        // 本モジュール doc「責務境界」節の fail-safe 契約
        // （RawHtml 混入は必ず None）と #1385 の Node-to-DOM 1:1 不変条件を
        // 単一アイテムでも保つことを固定する。
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: RawHtml 混入時の不成立判定を検証するテスト固定文字列、ユーザー入力なし"
        )]
        let a = el("div", vec![], vec![raw_html("<b>x</b>")]);
        let items: Vec<&Node> = vec![&a];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn non_element_root_is_none() {
        let a = text("a");
        let b = text("b");
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn non_root_key_attr_value_mismatch_is_none() {
        let a = el(
            "li",
            vec![],
            vec![el(
                "span",
                vec![(KEY_ATTR.to_string(), "1".to_string())],
                vec![text("a")],
            )],
        );
        let b = el(
            "li",
            vec![],
            vec![el(
                "span",
                vec![(KEY_ATTR.to_string(), "2".to_string())],
                vec![text("b")],
            )],
        );
        let items: Vec<&Node> = vec![&a, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn one_non_isomorphic_item_in_middle_is_none() {
        let a = row("1", "a");
        let b_bad = el("li", vec![], vec![text("bad")]);
        let c = row("3", "c");
        let items: Vec<&Node> = vec![&a, &b_bad, &c];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn one_non_isomorphic_item_at_start_is_none() {
        let a_bad = el("li", vec![], vec![text("bad")]);
        let b = row("2", "b");
        let items: Vec<&Node> = vec![&a_bad, &b];
        assert!(derive_item_template(&items).is_none());
    }

    #[test]
    fn one_non_isomorphic_item_at_end_is_none() {
        let a = row("1", "a");
        let b_bad = el("li", vec![], vec![text("bad")]);
        let items: Vec<&Node> = vec![&a, &b_bad];
        assert!(derive_item_template(&items).is_none());
    }

    // ---- パス検証 ----

    #[test]
    fn paths_resolve_to_text_and_match_text_count() {
        let a = el(
            "tr",
            vec![],
            vec![
                el("td", vec![], vec![text("1")]),
                el("td", vec![], vec![text("2")]),
                el("td", vec![], vec![text("3")]),
            ],
        );
        let b = el(
            "tr",
            vec![],
            vec![
                el("td", vec![], vec![text("4")]),
                el("td", vec![], vec![text("5")]),
                el("td", vec![], vec![text("6")]),
            ],
        );
        let items: Vec<&Node> = vec![&a, &b];
        let template = derive_item_template(&items).expect("同型のはず");
        assert_eq!(template.text_paths().len(), 3);
        assert_eq!(text_values(&a, &template).unwrap(), vec!["1", "2", "3"]);
        assert_eq!(text_values(&b, &template).unwrap(), vec!["4", "5", "6"]);
    }

    // ---- 性質テスト（外部依存ゼロ厳守: 固定シード xorshift64* による決定的疑似乱数） ----

    /// 決定的擬似乱数生成器（xorshift64*）。`core` は dev-dependencies も
    /// 含め外部依存ゼロを維持するため、性質テスト用の乱数は自前実装する
    /// （`.claude/rules/coding-rust.md` 「`core` は外部依存ゼロ」）。
    struct Xorshift64 {
        state: u64,
    }

    impl Xorshift64 {
        fn new(seed: u64) -> Self {
            Self { state: seed | 1 }
        }

        fn next_u64(&mut self) -> u64 {
            let mut x = self.state;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.state = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        fn next_range(&mut self, bound: usize) -> usize {
            (self.next_u64() % bound as u64) as usize
        }
    }

    const TAG_POOL: &[&str] = &["div", "span", "li", "p", "a", "b"];
    const ATTR_NAME_POOL: &[&str] = &["class", "id", "title"];

    /// ランダムな `Node` 木を生成する（タグは固定プールから選択、深さ上限
    /// つき）。ルートは常に `Element`（本モジュールの成立前提）。
    fn random_node(rng: &mut Xorshift64, depth: usize, is_root: bool) -> Node {
        // 深さ上限に達したら Text で打ち切る（無限再帰防止）。
        if depth == 0 || (!is_root && rng.next_range(3) == 0) {
            return text(format!("t{}", rng.next_u64() % 1000));
        }

        let tag = TAG_POOL[rng.next_range(TAG_POOL.len())];
        let attr_count = rng.next_range(3);
        let mut attrs = Vec::new();
        for _ in 0..attr_count {
            let name = ATTR_NAME_POOL[rng.next_range(ATTR_NAME_POOL.len())];
            // 同名属性の重複はテンプレート判定の本質と無関係なため避ける。
            if attrs.iter().any(|(n, _): &(String, String)| n == name) {
                continue;
            }
            attrs.push((name.to_string(), format!("v{}", rng.next_u64() % 100)));
        }
        let child_count = rng.next_range(4);
        let children = (0..child_count)
            .map(|_| random_node(rng, depth - 1, false))
            .collect();

        el(tag, attrs, children)
    }

    /// `node` 中の全 `Text` の値を再帰的に置換したコピーを返す
    /// （構造は不変、テキスト値のみ変える）。
    fn replace_all_text(node: &Node, rng: &mut Xorshift64) -> Node {
        match node {
            Node::Text(_) => text(format!("r{}", rng.next_u64() % 1000)),
            Node::Element {
                tag,
                attrs,
                children,
            } => Node::Element {
                tag,
                attrs: attrs.clone(),
                children: children.iter().map(|c| replace_all_text(c, rng)).collect(),
            },
            Node::RawHtml(s) => Node::RawHtml(s.clone()),
        }
    }

    /// `node` に構造変異を 1 箇所加えたコピーを返す。変異は必ず
    /// nodes_isomorphic を破る（タグ変更・属性増減/改名/値変更・
    /// 子増減・variant 入替のいずれか）。変異対象が存在しない場合（例:
    /// 子なし要素で子変異を狙った等）は `None` を返し、呼び出し側は
    /// 別シードで再試行する。
    fn mutate_structure(node: &Node, rng: &mut Xorshift64) -> Option<Node> {
        match node {
            Node::Element {
                tag,
                attrs,
                children,
            } => {
                // 変異の種類を選ぶ（子要素があれば子への再帰変異も候補に入れる）。
                let has_children = !children.is_empty();
                let kinds = if has_children { 5 } else { 4 };
                match rng.next_range(kinds) {
                    0 => {
                        // タグ変更（プール内の異なるタグを選ぶ）。
                        let mut new_tag = TAG_POOL[rng.next_range(TAG_POOL.len())];
                        let mut guard = 0;
                        while new_tag == *tag && guard < 16 {
                            new_tag = TAG_POOL[rng.next_range(TAG_POOL.len())];
                            guard += 1;
                        }
                        if new_tag == *tag {
                            return None;
                        }
                        Some(Node::Element {
                            tag: new_tag,
                            attrs: attrs.clone(),
                            children: children.clone(),
                        })
                    }
                    1 => {
                        // 属性追加。
                        let mut new_attrs = attrs.clone();
                        new_attrs.push(("data-mut".to_string(), "1".to_string()));
                        Some(Node::Element {
                            tag,
                            attrs: new_attrs,
                            children: children.clone(),
                        })
                    }
                    2 => {
                        // 属性削除（既存属性がなければ変異不成立）。
                        if attrs.is_empty() {
                            return None;
                        }
                        let mut new_attrs = attrs.clone();
                        new_attrs.remove(0);
                        Some(Node::Element {
                            tag,
                            attrs: new_attrs,
                            children: children.clone(),
                        })
                    }
                    3 => {
                        // 属性値変更（既存属性がなければ変異不成立）。
                        if attrs.is_empty() {
                            return None;
                        }
                        let mut new_attrs = attrs.clone();
                        new_attrs[0].1 = format!("{}-mut", new_attrs[0].1);
                        Some(Node::Element {
                            tag,
                            attrs: new_attrs,
                            children: children.clone(),
                        })
                    }
                    _ => {
                        // 子構造への再帰変異、または子の増減。
                        if rng.next_range(2) == 0 {
                            let mut new_children = children.clone();
                            new_children.push(text("extra"));
                            Some(Node::Element {
                                tag,
                                attrs: attrs.clone(),
                                children: new_children,
                            })
                        } else {
                            let idx = rng.next_range(children.len());
                            let mutated = mutate_structure(&children[idx], rng)?;
                            let mut new_children = children.clone();
                            new_children[idx] = mutated;
                            Some(Node::Element {
                                tag,
                                attrs: attrs.clone(),
                                children: new_children,
                            })
                        }
                    }
                }
            }
            Node::Text(_) => {
                // Text → Element への variant 入替。
                Some(el("span", vec![], vec![]))
            }
            Node::RawHtml(_) => None,
        }
    }

    /// `item` の各束縛点パス位置へ `values` を、ルートの `KEY_ATTR` へ
    /// `key` を書き込んだコピーを返す（往復性質テスト用）。DOM 適用
    /// （#1385）が行う操作を純 Rust 側で模した最小実装であり、本テスト
    /// 専用（本番コードには含めない）。
    fn apply_template(node: &Node, template: &ItemTemplate, values: &[&str], key: &str) -> Node {
        fn write_path(node: &Node, path: &[usize], value: &str) -> Node {
            match path.split_first() {
                None => text(value),
                Some((&idx, rest)) => match node {
                    Node::Element {
                        tag,
                        attrs,
                        children,
                    } => {
                        let mut new_children = children.clone();
                        new_children[idx] = write_path(&children[idx], rest, value);
                        Node::Element {
                            tag,
                            attrs: attrs.clone(),
                            children: new_children,
                        }
                    }
                    other => other.clone(),
                },
            }
        }

        let mut current = node.clone();
        for (path, value) in template.text_paths().iter().zip(values.iter()) {
            current = write_path(&current, path, value);
        }

        if let Node::Element {
            tag,
            attrs,
            children,
        } = current
        {
            let new_attrs = attrs
                .into_iter()
                .map(|(n, v)| {
                    if n == KEY_ATTR {
                        (n, key.to_string())
                    } else {
                        (n, v)
                    }
                })
                .collect();
            Node::Element {
                tag,
                attrs: new_attrs,
                children,
            }
        } else {
            current
        }
    }

    #[test]
    fn property_text_replacement_preserves_isomorphism() {
        let mut rng = Xorshift64::new(0xC0FF_EE01);
        for i in 0..300u64 {
            rng.state ^= i.wrapping_mul(0x9E37_79B9);
            let t = random_node(&mut rng, 4, true);
            // ルートが Text だけになるケース（is_root でも depth==0 で
            // Text を引く可能性はないが、念のため Element でなければ再生成）。
            if !matches!(t, Node::Element { .. }) {
                continue;
            }
            let t2 = replace_all_text(&t, &mut rng);

            let items: Vec<&Node> = vec![&t, &t2];
            let template =
                derive_item_template(&items).expect("テキスト置換のみの木同士は常に同型のはず");

            let expected: Vec<&str> = collect_all_text(&t2);
            let actual = text_values(&t2, &template).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn property_structural_mutation_breaks_isomorphism() {
        let mut rng = Xorshift64::new(0xDEAD_BEEF);
        let mut checked = 0;
        let mut attempts: u64 = 0;
        while checked < 200 && attempts < 5000 {
            attempts += 1;
            rng.state ^= attempts.wrapping_mul(0x9E37_79B9);
            let t = random_node(&mut rng, 4, true);
            if !matches!(t, Node::Element { .. }) {
                continue;
            }
            let Some(mutated) = mutate_structure(&t, &mut rng) else {
                continue;
            };
            if !matches!(mutated, Node::Element { .. }) {
                continue;
            }

            let items: Vec<&Node> = vec![&t, &mutated];
            assert!(
                derive_item_template(&items).is_none(),
                "構造変異後は必ず不成立のはず: {:?} vs {:?}",
                t,
                mutated
            );
            checked += 1;
        }
        assert!(checked >= 200, "十分な変異ケースを検証できなかった");
    }

    #[test]
    fn property_round_trip_write_back_matches_original() {
        let mut rng = Xorshift64::new(0x1234_5678);
        for i in 0..200u64 {
            rng.state ^= i.wrapping_mul(0x9E37_79B9);
            let base = random_node(&mut rng, 4, true);
            if !matches!(base, Node::Element { .. }) {
                continue;
            }
            // 各アイテムはルートの KEY_ATTR 値のみ異なる想定。base をルート
            // key を持つ形へ正規化する。
            let base = with_root_key(base, "base-key");
            let target = replace_all_text(&base, &mut rng);
            let target = with_root_key(target, "target-key");

            let items: Vec<&Node> = vec![&base, &target];
            let template = derive_item_template(&items)
                .expect("テキスト置換 + ルート key 差異のみは常に同型のはず");

            let values = text_values(&target, &template).unwrap();
            let rebuilt = apply_template(&base, &template, &values, "target-key");
            assert_eq!(rebuilt, target);
        }
    }

    #[test]
    fn property_derivation_is_deterministic() {
        let mut rng = Xorshift64::new(0x5EED_5EED);
        for i in 0..100u64 {
            rng.state ^= i.wrapping_mul(0x9E37_79B9);
            let a = random_node(&mut rng, 3, true);
            let b = replace_all_text(&a, &mut rng);
            if !matches!(a, Node::Element { .. }) {
                continue;
            }
            let items: Vec<&Node> = vec![&a, &b];
            let t1 = derive_item_template(&items);
            let t2 = derive_item_template(&items);
            assert_eq!(t1, t2);
        }
    }

    /// `node`（Element ルート前提）のルート属性列に `KEY_ATTR` を
    /// `key` の値で設定したコピーを返す（既存の同名属性があれば置換、
    /// なければ追加）。性質テストが「keyed_list が付与する行固有 key」
    /// を模すためのヘルパー。
    fn with_root_key(node: Node, key: &str) -> Node {
        match node {
            Node::Element {
                tag,
                mut attrs,
                children,
            } => {
                if let Some(existing) = attrs.iter_mut().find(|(n, _)| n == KEY_ATTR) {
                    existing.1 = key.to_string();
                } else {
                    attrs.push((KEY_ATTR.to_string(), key.to_string()));
                }
                Node::Element {
                    tag,
                    attrs,
                    children,
                }
            }
            other => other,
        }
    }

    /// `node` 中の全 `Text` 値を文書順で集める（性質テストの期待値算出用）。
    fn collect_all_text(node: &Node) -> Vec<&str> {
        let mut out = Vec::new();
        fn go<'a>(node: &'a Node, out: &mut Vec<&'a str>) {
            match node {
                Node::Text(s) => out.push(s.as_str()),
                Node::Element { children, .. } => {
                    for c in children {
                        go(c, out);
                    }
                }
                Node::RawHtml(_) => {}
            }
        }
        go(node, &mut out);
        out
    }
}
