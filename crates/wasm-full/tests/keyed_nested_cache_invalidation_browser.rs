//! ネストした keyed list の field 間キャッシュ無効化（イシュー #1340
//! 独立敵対レビュー指摘 A）の実ブラウザ統合テスト
//! （`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-client/src/keyed_apply.rs`（`ApplyOutcome::
//! invalidated_nested_fields`）・`crates/wasm-client/src/keyed_dom.rs`
//! （`KeyedListApplyResult::Achieved::invalidated_nested_fields`）・
//! `crates/wasm-full/src/lib.rs`（`Runtime::commit_keyed_list_result`）が
//! それぞれ純粋関数・native テストで検証済みの内部機構を、実際の
//! `Runtime` 経路（`data-bind-list` 2 個・親子ネスト構成）で end-to-end
//! 検証する。
//!
//! # シナリオ
//!
//! `groups`（1 グループ）の各アイテム内に `children`（ネストした keyed
//! list）を持つ最小 component（[`NestedListState`]）を用意する。1 回の
//! dispatch で「グループのラベル変更（`groups` を dirty 化、内容変更の
//! `Update` → `replace_item_children` で group アイテムの子孫がまるごと
//! 再構築される）」と「子アイテムの新規追加（`children` を dirty 化、
//! `Insert`）」を同時に起こす。
//!
//! `groups` の `replace_item_children` は group アイテムの子孫（ネストした
//! `children` keyed list を含む）をまるごと新しい view から再構築するため、
//! `children` field 自身の処理（`Insert` op 適用）と**同じライブ DOM 領域を
//! 二重に触る**。`Runtime::keyed_list_cache` が field ごとに独立している
//! ため、この副作用を無視すると `children` field の次回 diff 基準
//! （`previous`）が実際のライブ DOM と乖離し、`data-key` の重複挿入を
//! 引き起こす（`groups` → `children` の順で dirty 処理される場合に顕在化、
//! 独立敵対レビュー指摘 A 本文参照）。
//!
//! `Runtime::commit_keyed_list_result` によるネスト field 無効化により、
//! **dirty 処理順序に関わらず**（本ファイルは `groups`→`children`・
//! `children`→`groups` の両順序をそれぞれ固定テストで検証する）最終的な
//! ライブ DOM が `data-key` 重複なく正しい内容へ収束することを確認する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// `keyed_update_browser.rs::create_placeholder` と同じ意図。
fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// `groups`（1 アイテムのみ、`g1`）の各アイテム内に `children`（ネスト
/// keyed list）を持つ最小 component。
///
/// `groups` を複数アイテムにすると `data-bind-list="children"` が DOM 上に
/// 複数出現し `find_list_element`（単一 `query_selector` 一致）の対象が
/// 曖昧になる（本テストが検証する対象とは別の既知の制約）ため、意図的に
/// `groups` は 1 アイテムに限定する。
#[derive(Debug, Clone)]
struct NestedListState {
    label: String,
    children: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
    /// `true`: dirty 処理順序を `children` → `groups` にする。
    /// `false`（既定）: `groups` → `children`。
    children_first: bool,
}

impl NestedListState {
    const FIELD_GROUPS: &'static str = "groups";
    const FIELD_CHILDREN: &'static str = "children";

    fn new(children_first: bool) -> Self {
        Self {
            label: "g1".to_string(),
            children: vec![(1, "c1".to_string()), (2, "c2".to_string())],
            dirty: Vec::new(),
            children_first,
        }
    }
}

/// `data-action="grow"` の唯一のアクション: グループのラベル変更
/// （`groups` を dirty 化）と子アイテムの新規追加（`children` を dirty
/// 化）を同時に起こす。
struct GrowAction;

impl Component for NestedListState {
    type Action = GrowAction;

    fn update(&mut self, _action: Self::Action) {
        self.dirty.clear();
        self.label = "g1-renamed".to_string();
        self.children.push((3, "c3".to_string()));
        if self.children_first {
            self.dirty.push(Self::FIELD_CHILDREN);
            self.dirty.push(Self::FIELD_GROUPS);
        } else {
            self.dirty.push(Self::FIELD_GROUPS);
            self.dirty.push(Self::FIELD_CHILDREN);
        }
    }

    fn view(&self) -> Node {
        let children_nodes: Vec<(String, Node)> = self
            .children
            .iter()
            .map(|(id, content)| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "child-item")],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let children_list = keyed_list("ul", vec![], Self::FIELD_CHILDREN, children_nodes)
            .expect("test fixture nested keyed list must be valid");

        let group_item = el(
            "li",
            vec![("data-testid", "group-item")],
            vec![text(&self.label), children_list],
        );
        let groups_list = keyed_list(
            "ul",
            vec![],
            Self::FIELD_GROUPS,
            vec![("g1".to_string(), group_item)],
        )
        .expect("test fixture keyed groups list must be valid");

        el("div", vec![("id", "nested-root")], vec![groups_list])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "grow" => Some(GrowAction),
            _ => None,
        }
    }
}

impl DirtyTracked for NestedListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

/// keyed list 内容のみで `data-bind-*` 束縛点を持たないため常に `None`
/// （`keyed_update_browser.rs::ListState` の `BindingSource` 実装と同じ
/// 意図）。
impl BindingSource for NestedListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `root` 直下へ `data-action="grow"` の合成クリックターゲットを追加し、
/// クリックして dispatch する（`keyed_update_browser.rs::dispatch_rename`
/// と同じ手法、payload なし）。
fn dispatch_grow(document: &Document, root: &Element) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", "grow")
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// dispatch 後の `children` リストが重複 `data-key` なく `[c1, c2, c3]`
/// へ正しく収束していることを検証する共通アサーション。
fn assert_children_converged(root: &Element) {
    let child_nodes = root
        .query_selector_all("[data-testid='child-item']")
        .expect("query_selector_all must not fail");
    let mut keys: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for i in 0..child_nodes.length() {
        let node = child_nodes.item(i).expect("index within length must exist");
        let element: Element = node.unchecked_into();
        keys.push(
            element
                .get_attribute("data-key")
                .unwrap_or_else(|| "<missing>".to_string()),
        );
        texts.push(element.text_content().unwrap_or_default());
    }

    assert_eq!(
        child_nodes.length(),
        3,
        "重複挿入が起きていなければちょうど 3 件のはず（実測 keys: {keys:?}）"
    );
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    sorted_keys.dedup();
    assert_eq!(
        sorted_keys.len(),
        3,
        "data-key が重複していないはず（実測: {keys:?}）"
    );
    assert_eq!(
        texts,
        vec!["c1".to_string(), "c2".to_string(), "c3".to_string()],
        "内容も正しい並びへ収束しているはず（実測: {texts:?}）"
    );

    let group_item = root
        .query_selector("[data-testid='group-item']")
        .expect("query_selector must not fail")
        .expect("group item must exist");
    assert!(
        group_item
            .text_content()
            .unwrap_or_default()
            .contains("g1-renamed"),
        "グループラベルも更新されているはず"
    );
}

/// 独立敵対レビュー指摘 A 回帰固定（dirty 処理順序 1）: `groups` →
/// `children` の順で dirty 処理される場合。`groups` の内容変更 Update が
/// `children` を含む部分木をまるごと再構築した**直後**に `children`
/// field 自身の `Insert` 処理が走る、最も直接的に重複を誘発する順序。
#[wasm_bindgen_test]
fn nested_field_cache_invalidated_regardless_of_order_groups_then_children() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "nested-cache-root-groups-first");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = NestedListState::new(false);
    let runtime =
        Runtime::mount("nested-cache-root-groups-first", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_grow(&document, root);

    assert_children_converged(root);
}

/// 独立敵対レビュー指摘 A 回帰固定（dirty 処理順序 2）: `children` →
/// `groups` の順で dirty 処理される場合。コーディネーター指摘「この順
/// なら children 適用後に groups が再構築して再び乖離する対称ケースが
/// ある」を踏まえ、順序を反転しても同じく重複なく収束することを固定する。
#[wasm_bindgen_test]
fn nested_field_cache_invalidated_regardless_of_order_children_then_groups() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "nested-cache-root-children-first");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = NestedListState::new(true);
    let runtime =
        Runtime::mount("nested-cache-root-children-first", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_grow(&document, root);

    assert_children_converged(root);

    // 収束確認: 直後にもう 1 tick（内容変化なしの再送）を送っても、
    // ネスト field 無効化により毎回 cache-miss フォールバックを経由する
    // 設計であっても収束状態が崩れない（無効化が「壊す」方向へ働かない
    // ことの追加確認）。
    dispatch_grow(&document, root);
    let child_nodes = root
        .query_selector_all("[data-testid='child-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(
        child_nodes.length(),
        4,
        "2 回目の grow で c4 が追加され重複なく 4 件になるはず"
    );
}
