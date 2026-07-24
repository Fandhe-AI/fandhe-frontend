//! JsonTreeView（JSON 風データ構造のツリー表示）headless コンポーネント
//! （イシュー #829、`docs/policy/intentional-non-adoption.md` §7・
//! `docs/design/component-coverage-map.md` の保留解除）。
//!
//! ark-ui の `utilities/json-tree-view`
//!（`.claude/skills/ark-ui/references/components/utilities/json-tree-view.md`）
//! 相当の部品は「実装済み [`mod@crate::tree_view`]（イシュー #753）の派生
//! として実装できる見込み」として保留されていた。本モジュールはその保留を
//! 解除し、[`mod@crate::tree_view`] の 12 anatomy パーツ・[`TreeView`] 状態
//! 機械（展開集合 + 選択値）を**そのまま再利用**しつつ、決定的な JSON 風
//! データ構造 [`JsonValue`] をツリー表示するための最小限の追加（`key`/`value`
//! の 2 anatomy パーツと変換ロジック [`render_json`]）のみを提供する。
//!
//! # tree_view（#753）の派生であることの位置づけ
//!
//! - 構造部（root/tree/branch/branch-control/branch-indicator/branch-content/
//!   branch-indent-guide/item/item-indicator、`data-scope="tree-view"`）は
//!   [`crate::tree_view`] の既存パーツ関数をそのまま呼ぶ。新しい状態機械は
//!   持たず、[`TreeView`]（本モジュールでは [`crate::tree_view::TreeView`] を
//!   再エクスポートする）の展開集合（[`crate::state::MultiSelect`]）・選択値
//!   （[`crate::state::SingleSelect`]）・dispatch（`"expand"`/`"collapse"`/
//!   `"toggle"`/`"select"`/`"deselect"`）・hydration をそのまま利用する
//!   （`fandhe-frontend-pre-styled-ui` の既存 styled TreeView recipe が
//!   インデント・開閉・選択・focus-visible の CSS をそのまま適用できる）。
//! - JSON 固有の追加パーツは `key`（`span`、オブジェクトキー/配列 index の
//!   表示）と `value`（`span[data-kind]`、値テキストの型別表示）の 2 個のみで、
//!   これらは新設の `data-scope="json-tree-view"`（[`ANATOMY`]）に属する
//!   （`tree-view` スコープの構造部とは別スコープ。既存 styled TreeView
//!   recipe を壊さないため）。
//!
//! # データモデル（外部依存ゼロ）
//!
//! [`JsonValue`] は決定的な静的 enum であり、`serde_json::Value` 等の外部
//! クレートに依存しない（`.claude/rules/coding-rust.md` の core/headless-ui
//! 外部依存ゼロ方針）。`Object` は `HashMap` ではなく `Vec<(String,
//! JsonValue)>`（挿入順を保持する決定的なペア列）で表現し、同一入力に対する
//! [`render_json`] の出力が常にバイト単位で一致することを保証する。
//!
//! # ノード識別子（RFC 6901 JSON Pointer）
//!
//! `data-value`（dispatch payload・展開/選択状態のキー）には
//! [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) の JSON Pointer 記法
//! （`~0`→`~`・`~1`→`/` の逆写像を適用したエスケープ済みセグメントを `/` で
//! 連結）を使う。ルートは空文字列 `""`。キーに `/` や `~` を含むデータでも
//! ポインタの一意性が壊れない（[`escape_pointer_segment`] 参照）。配列要素は
//! 10 進の index 文字列をセグメントとして使う。
//!
//! # 値の表示テキスト
//!
//! - `Null` → `null` / `Bool` → `true`/`false` / `Number` → `f64` の
//!   `Display`（Rust 標準ライブラリは最短往復表現を返すため決定的。
//!   `NaN`/`±inf` は JSON 非準拠の値だが `Display` 出力をそのまま
//!   fail-safe に描画する。既定エスケープ経由のため XSS 上の懸念はない）
//! - `String` → 前後に `"` を付けた JSON 風プレビュー（引用符もテキスト
//!   ノードの一部として [`fandhe_frontend_core::render`] の既定エスケープを
//!   通す）
//! - `Object`/`Array`（ブランチ要約）→ `Object(N)` / `Array(N)`
//!   （`N` は子要素数、決定的なサマリ文字列）
//!
//! # セキュリティ不変条件
//!
//! - `key`/`value` の属性名（`data-kind` 含む）はすべて `&'static str`
//!   リテラルで固定しており、[`JsonValue::kind`] が返す語彙
//!   （`"null"`/`"bool"`/`"number"`/`"string"`/`"array"`/`"object"`）も
//!   `&'static str` のみを返すため、動的値が属性名・`data-kind` 属性値の
//!   スロットへ混入する経路はない（[`mod@crate::anatomy`]/[`crate::data_attrs`]
//!   の既存不変条件をそのまま継承する）。
//! - 動的値（キー文字列・値の表示テキスト・JSON Pointer・呼び出し側
//!   `attrs`/`children`）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - dispatch payload・hydration 属性は [`crate::tree_view::TreeView`] の
//!   既存 fail-closed 保証（改ざんされうるクライアント入力として扱い、
//!   panic せず `HydrateError` を返す）をそのまま継承する。新たな
//!   hydration/dispatch 面は追加しない。
//!
//! # DoS（深い再帰）に関する注記
//!
//! [`render_json`]/[`expanded_to_depth`] はいずれも入力 [`JsonValue`] の
//! 深さに比例する再帰呼び出しを行う（[`crate::tree_view::TreeView::render_level`]
//! と同型の特性）。表示対象は開発者が管理する静的データを想定しており、
//! 極端に深いネストを持つ入力を与えるとスタック消費が増大する点に注意する
//! （新規の脅威面ではなく、既存 `tree_view` の再帰特性の延長）。
//!
//! # out-of-scope（本イシュー #829 のスコープ外）
//!
//! - **CSR 挙動層**: クリック→dispatch の実 DOM 配線・キーボードナビゲーション
//!   は [`crate::tree_view`] と同じく `fandhe-frontend-wasm-full` 後続イシュー
//!   の責務。
//! - **ark-ui の JS 固有オプション**: `collapseStringsAfterLength`（長い文字列
//!   の切り詰め表示）・`groupArraysAfterLength`（大配列のグルーピング表示）・
//!   `showNonenumerable`・lazy loading・rename はいずれも動的・非決定的な
//!   挙動または JS 固有の実行時機能であり、静的・決定的実装の範囲外として
//!   本モジュールでは提供しない。

use crate::anatomy::{anatomy, Anatomy};
use crate::tree_view::{
    branch, branch_content, branch_control, branch_indent_guide, branch_indicator, item,
    item_indicator, root as tree_root, TreeViewAction,
};
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_interactive::Component;

// [`crate::tree_view::TreeView`]（状態機械）・[`crate::state::OpenState`]
// （分岐の開閉状態）を呼び出し側が本モジュールのみに依存して使えるよう
// 再エクスポートする（[`crate::pre_styled_ui`] 側のパターンと同型の判断）。
pub use crate::tree_view::TreeView;

/// JsonTreeView 固有パーツ（`key`/`value`）の anatomy（`data-scope="json-tree-view"`）。
///
/// 構造部（root/tree/branch/...）は [`crate::tree_view::ANATOMY`]
/// （`data-scope="tree-view"`）のまま変更しない。本 anatomy は JSON 固有の
/// 2 パーツのみを持つ、既存 tree_view recipe に影響を与えない別スコープ。
const ANATOMY: Anatomy = anatomy("json-tree-view");

/// 決定的な JSON 風データ構造 1 値（ark-ui `createJsonTreeCollection`
/// 相当を外部依存ゼロの自前 enum で表現する）。
///
/// `Object` は挿入順を保持する `Vec<(String, JsonValue)>` で表現し、
/// `HashMap`（反復順序が非決定的）は使わない（[`render_json`] の出力
/// 決定性の前提）。
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON の `null`。
    Null,
    /// JSON の真偽値。
    Bool(bool),
    /// JSON の数値（`f64` で表現する。`serde_json` 等の Number 型は
    /// 外部依存ゼロ方針のため導入しない）。
    Number(f64),
    /// JSON の文字列。
    String(String),
    /// JSON の配列。
    Array(Vec<JsonValue>),
    /// JSON のオブジェクト（挿入順を保持するペア列）。
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// この値の種別を表す固定語彙（`data-kind` 属性値としてそのまま使う）。
    ///
    /// `&'static str` のみを返すため、[`value`] の `data-kind` 属性値スロットへ
    /// 動的文字列が混入する経路はない（モジュール doc §セキュリティ不変条件参照）。
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    /// 子を 1 個以上持つブランチ（`Object`/`Array` で子要素が非空）かどうか。
    /// [`crate::tree_view::TreeNode::is_branch`] と同じ判断基準。
    #[must_use]
    pub fn is_branch(&self) -> bool {
        match self {
            Self::Object(entries) => !entries.is_empty(),
            Self::Array(items) => !items.is_empty(),
            _ => false,
        }
    }

    /// この値の表示テキスト（モジュール doc §値の表示テキスト参照）。
    #[must_use]
    pub fn display_text(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{s}\""),
            Self::Array(items) => format!("Array({})", items.len()),
            Self::Object(entries) => format!("Object({})", entries.len()),
        }
    }
}

/// RFC 6901 JSON Pointer のセグメントエスケープ（`~`→`~0`・`/`→`~1`）。
/// 置換順序が重要（`~` を先に変換しないと、後続の `/`→`~1` 変換で生成される
/// `~1` 自体が再度 `~`→`~0` 変換の対象になり二重変換されてしまう）。
#[must_use]
fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// 親ポインタ（ルートは `""`）にセグメントを 1 段連結した子ポインタを返す。
#[must_use]
fn child_pointer(parent: &str, segment: &str) -> String {
    format!("{parent}/{}", escape_pointer_segment(segment))
}

/// Key パーツ（`span`）。オブジェクトキー/配列 index のラベル表示のみを担う
/// 装飾用パーツ（[`crate::tree_view::branch_text`] と同型の最小主義）。
#[must_use]
pub fn key<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("key", "span", attrs, children)
}

/// Value パーツ（`span[data-kind]`）。値の表示テキストを型別に描画する。
/// `kind` は [`JsonValue::kind`] が返す固定語彙のみを受け取り、属性値スロット
/// への動的値混入経路を作らない（モジュール doc §セキュリティ不変条件参照）。
#[must_use]
pub fn value<'a>(kind: &'static str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-kind", kind)];
    merged.extend(attrs);
    ANATOMY.part("value", "span", merged, children)
}

/// [`JsonValue`] 木を現在の展開・選択状態（`tree`）で再帰的に描画する。
///
/// [`crate::tree_view::TreeView::render_nodes`] の JSON 版であり、同様に
/// 各階層で `aria-posinset`/`aria-setsize`/`aria-level`/`data-depth` を
/// 決定的に計算しつつ、構造部（[`crate::tree_view`] のパーツ関数）と JSON
/// 固有パーツ（[`key`]/[`value`]）を組み合わせて完全なマークアップを
/// 組み立てる。ルートは 1 個のノードとして返す（[`crate::tree_view::TreeNode`]
/// のような兄弟列ではなく単一の値を起点とするため、戻り値は `Vec<Node>` では
/// なく `Node` 1 個）。
#[must_use]
pub fn render_json(tree: &TreeView, root: &JsonValue) -> Node {
    render_node(tree, root, "", None, 1, 1, 0)
}

/// [`render_json`] の再帰実装本体。
#[allow(clippy::too_many_arguments)]
fn render_node(
    tree: &TreeView,
    val: &JsonValue,
    pointer: &str,
    key_label: Option<&str>,
    posinset: usize,
    setsize: usize,
    depth: usize,
) -> Node {
    let level_s = (depth + 1).to_string();
    let posinset_s = posinset.to_string();
    let setsize_s = setsize.to_string();
    let depth_s = depth.to_string();
    let is_selected = tree.is_selected(pointer);
    let key_node = key_label.map(|k| key(Vec::new(), vec![text(k)]));

    if val.is_branch() {
        let state = tree.branch_state(pointer);
        let mut control_children = vec![branch_indicator(state, Vec::new(), Vec::new())];
        control_children.extend(key_node);
        control_children.push(value(
            val.kind(),
            Vec::new(),
            vec![text(val.display_text())],
        ));

        let child_depth = depth + 1;
        let child_nodes = match val {
            JsonValue::Object(entries) => {
                let setsize = entries.len();
                entries
                    .iter()
                    .enumerate()
                    .map(|(idx, (k, v))| {
                        let child_ptr = child_pointer(pointer, k);
                        render_node(
                            tree,
                            v,
                            &child_ptr,
                            Some(k.as_str()),
                            idx + 1,
                            setsize,
                            child_depth,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            JsonValue::Array(items) => {
                let setsize = items.len();
                items
                    .iter()
                    .enumerate()
                    .map(|(idx, v)| {
                        let idx_label = idx.to_string();
                        let child_ptr = child_pointer(pointer, &idx_label);
                        render_node(
                            tree,
                            v,
                            &child_ptr,
                            Some(&idx_label),
                            idx + 1,
                            setsize,
                            child_depth,
                        )
                    })
                    .collect::<Vec<_>>()
            }
            // `is_branch()` が true を返す分岐は Object/Array のみ。
            _ => unreachable!("is_branch() が true の値は Object/Array のみ"),
        };

        branch(
            state,
            pointer,
            is_selected,
            false,
            &level_s,
            &posinset_s,
            &setsize_s,
            &depth_s,
            Vec::new(),
            vec![
                branch_control(state, is_selected, false, Vec::new(), control_children),
                branch_content(
                    state,
                    Vec::new(),
                    vec![
                        branch_indent_guide(Vec::new(), Vec::new()),
                        tree_root(Vec::new(), child_nodes),
                    ],
                ),
            ],
        )
    } else {
        let mut item_children = vec![item_indicator(is_selected, Vec::new(), Vec::new())];
        item_children.extend(key_node);
        item_children.push(value(
            val.kind(),
            Vec::new(),
            vec![text(val.display_text())],
        ));

        item(
            pointer,
            is_selected,
            false,
            &level_s,
            &posinset_s,
            &setsize_s,
            &depth_s,
            Vec::new(),
            item_children,
        )
    }
}

/// ark-ui `defaultExpandedDepth` 相当の決定的初期展開ヘルパ。ルートから
/// `depth` 段目まで（`depth == 0` なら何も展開しない）のブランチをすべて
/// 展開済みにした [`TreeView`] を返す。
#[must_use]
pub fn expanded_to_depth(root: &JsonValue, depth: usize) -> TreeView {
    let mut tree = TreeView::default();
    expand_to_depth_rec(&mut tree, root, "", 0, depth);
    tree
}

fn expand_to_depth_rec(
    tree: &mut TreeView,
    val: &JsonValue,
    pointer: &str,
    current_depth: usize,
    max_depth: usize,
) {
    if current_depth >= max_depth || !val.is_branch() {
        return;
    }
    tree.update(TreeViewAction::Expand(pointer.to_string()));
    match val {
        JsonValue::Object(entries) => {
            for (k, v) in entries {
                let child_ptr = child_pointer(pointer, k);
                expand_to_depth_rec(tree, v, &child_ptr, current_depth + 1, max_depth);
            }
        }
        JsonValue::Array(items) => {
            for (idx, v) in items.iter().enumerate() {
                let idx_label = idx.to_string();
                let child_ptr = child_pointer(pointer, &idx_label);
                expand_to_depth_rec(tree, v, &child_ptr, current_depth + 1, max_depth);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::dispatch;

    fn sample() -> JsonValue {
        JsonValue::Object(vec![
            ("name".to_string(), JsonValue::String("Ada".to_string())),
            ("age".to_string(), JsonValue::Number(36.0)),
            ("active".to_string(), JsonValue::Bool(true)),
            ("nickname".to_string(), JsonValue::Null),
            (
                "tags".to_string(),
                JsonValue::Array(vec![
                    JsonValue::String("admin".to_string()),
                    JsonValue::String("owner".to_string()),
                ]),
            ),
        ])
    }

    // --- JsonValue: kind/is_branch/display_text ---

    #[test]
    fn kind_returns_expected_literal_per_variant() {
        assert_eq!(JsonValue::Null.kind(), "null");
        assert_eq!(JsonValue::Bool(true).kind(), "bool");
        assert_eq!(JsonValue::Number(1.0).kind(), "number");
        assert_eq!(JsonValue::String("x".to_string()).kind(), "string");
        assert_eq!(JsonValue::Array(vec![]).kind(), "array");
        assert_eq!(JsonValue::Object(vec![]).kind(), "object");
    }

    #[test]
    fn is_branch_true_only_for_nonempty_array_or_object() {
        assert!(!JsonValue::Null.is_branch());
        assert!(!JsonValue::Array(vec![]).is_branch());
        assert!(!JsonValue::Object(vec![]).is_branch());
        assert!(JsonValue::Array(vec![JsonValue::Null]).is_branch());
        assert!(JsonValue::Object(vec![("a".to_string(), JsonValue::Null)]).is_branch());
    }

    #[test]
    fn display_text_matches_expected_format_per_variant() {
        assert_eq!(JsonValue::Null.display_text(), "null");
        assert_eq!(JsonValue::Bool(true).display_text(), "true");
        assert_eq!(JsonValue::Bool(false).display_text(), "false");
        assert_eq!(JsonValue::Number(1.5).display_text(), "1.5");
        assert_eq!(JsonValue::String("hi".to_string()).display_text(), "\"hi\"");
        assert_eq!(
            JsonValue::Array(vec![JsonValue::Null, JsonValue::Null]).display_text(),
            "Array(2)"
        );
        assert_eq!(
            JsonValue::Object(vec![("a".to_string(), JsonValue::Null)]).display_text(),
            "Object(1)"
        );
    }

    // --- JSON Pointer エスケープ ---

    #[test]
    fn escape_pointer_segment_escapes_tilde_and_slash() {
        assert_eq!(escape_pointer_segment("a/b"), "a~1b");
        assert_eq!(escape_pointer_segment("a~b"), "a~0b");
        assert_eq!(escape_pointer_segment("a~1b"), "a~01b");
        assert_eq!(escape_pointer_segment("plain"), "plain");
    }

    #[test]
    fn child_pointer_distinguishes_keys_that_would_otherwise_collide() {
        // "a/b" と "a"→"b" のネストは、エスケープなしでは同じ "/a/b" 経路に
        // 見えてしまう。エスケープにより一意性が保たれることを固定する。
        let flat = child_pointer("", "a/b");
        let nested_outer = child_pointer("", "a");
        let nested_inner = child_pointer(&nested_outer, "b");
        assert_ne!(flat, nested_inner);
        assert_eq!(flat, "/a~1b");
        assert_eq!(nested_inner, "/a/b");
    }

    // --- render_json: 決定性・ノード木変換 ---

    #[test]
    fn render_json_is_deterministic() {
        let tree = TreeView::default();
        let data = sample();
        let html_a = render(&render_json(&tree, &data));
        let html_b = render(&render_json(&tree, &data));
        assert_eq!(html_a, html_b);
    }

    #[test]
    fn render_json_root_object_has_level_1_and_depth_0() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        assert!(html.contains(r#"aria-level="1""#));
        assert!(html.contains(r#"data-depth="0""#));
        assert!(html.contains(r#"data-value=""#));
    }

    #[test]
    fn render_json_outputs_data_kind_per_value_type() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        assert!(html.contains(r#"data-kind="string""#));
        assert!(html.contains(r#"data-kind="number""#));
        assert!(html.contains(r#"data-kind="bool""#));
        assert!(html.contains(r#"data-kind="null""#));
        assert!(html.contains(r#"data-kind="array""#));
        assert!(html.contains(r#"data-kind="object""#));
    }

    #[test]
    fn render_json_nested_array_element_has_incremented_level_and_depth() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        // "tags" (depth=1, level=2) の子要素 (depth=2, level=3)。
        assert!(html.contains(r#"aria-level="2""#));
        assert!(html.contains(r#"data-depth="1""#));
        assert!(html.contains(r#"aria-level="3""#));
        assert!(html.contains(r#"data-depth="2""#));
    }

    #[test]
    fn render_json_object_top_level_setsize_matches_entry_count() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        // sample() のトップレベルは 5 エントリ。
        assert!(html.contains(r#"aria-setsize="5""#));
    }

    #[test]
    fn render_json_array_child_pointer_uses_numeric_index() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        assert!(html.contains(r#"data-value="/tags/0""#));
        assert!(html.contains(r#"data-value="/tags/1""#));
    }

    #[test]
    fn render_json_leaf_has_no_aria_expanded() {
        let tree = TreeView::default();
        let data = JsonValue::Object(vec![("a".to_string(), JsonValue::Null)]);
        let html = render(&render_json(&tree, &data));
        // ルート自身は object なので aria-expanded を持つが、葉ノード "a" の
        // 直近コンテキストとしては item（aria-expanded なし）であることを、
        // ルート含め aria-expanded がちょうど 1 回しか出現しないことで固定する。
        assert_eq!(html.matches("aria-expanded").count(), 1);
    }

    #[test]
    fn render_json_key_part_renders_object_key_label() {
        let tree = TreeView::default();
        let html = render(&render_json(&tree, &sample()));
        assert!(html.contains(r#"data-part="key""#));
        assert!(html.contains(">name<"));
        assert!(html.contains(">tags<"));
    }

    // --- TreeView dispatch との統合 ---

    #[test]
    fn render_json_reflects_expand_dispatch_on_root() {
        let mut tree = TreeView::default();
        let data = sample();
        let collapsed = render(&render_json(&tree, &data));
        assert!(collapsed.contains(r#"aria-expanded="false""#));

        assert!(dispatch(&mut tree, "expand", ""));
        let expanded = render(&render_json(&tree, &data));
        assert!(expanded.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn render_json_reflects_select_dispatch_on_nested_pointer() {
        let mut tree = TreeView::default();
        let data = sample();
        assert!(dispatch(&mut tree, "select", "/name"));
        let html = render(&render_json(&tree, &data));
        assert!(html.contains(r#"data-value="/name""#));
        assert!(html.contains(r#"data-selected="""#));
    }

    // --- expanded_to_depth ---

    #[test]
    fn expanded_to_depth_zero_expands_nothing() {
        let tree = expanded_to_depth(&sample(), 0);
        assert!(!tree.is_expanded(""));
    }

    #[test]
    fn expanded_to_depth_one_expands_only_root() {
        let tree = expanded_to_depth(&sample(), 1);
        assert!(tree.is_expanded(""));
        assert!(!tree.is_expanded("/tags"));
    }

    #[test]
    fn expanded_to_depth_two_expands_root_and_its_branch_children() {
        let tree = expanded_to_depth(&sample(), 2);
        assert!(tree.is_expanded(""));
        assert!(tree.is_expanded("/tags"));
    }

    #[test]
    fn expanded_to_depth_is_deterministic() {
        let a = expanded_to_depth(&sample(), 2);
        let b = expanded_to_depth(&sample(), 2);
        assert_eq!(a, b);
    }

    #[test]
    fn expanded_to_depth_does_not_expand_leaves() {
        // 葉ノード（number/string/bool/null）はブランチではないため展開集合に
        // 加えられない（dispatch "expand" を発行しない）ことを固定する。
        let tree = expanded_to_depth(&sample(), 5);
        assert!(!tree.is_expanded("/name"));
        assert!(!tree.is_expanded("/age"));
    }

    // --- 呼び出し側 data-scope/data-part 偽装除去（fail-closed 回帰） ---

    #[test]
    fn key_and_value_drop_caller_supplied_scope_and_part() {
        let html_key = render(&key(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![text("k")],
        ));
        assert!(html_key.contains(r#"data-scope="json-tree-view""#));
        assert!(html_key.contains(r#"data-part="key""#));
        assert!(!html_key.contains("attacker"));

        let html_value = render(&value(
            "string",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![text("v")],
        ));
        assert!(html_value.contains(r#"data-scope="json-tree-view""#));
        assert!(html_value.contains(r#"data-part="value""#));
        assert!(html_value.contains(r#"data-kind="string""#));
        assert!(!html_value.contains("attacker"));
    }
}
