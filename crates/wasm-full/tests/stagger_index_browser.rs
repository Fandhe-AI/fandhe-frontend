//! `stagger_index::sync_stagger_index`（イシュー #2397）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `keyed_update_browser.rs` と同じ理由（製品コンポーネント
//! `interactive::AppState` では検証したい keyed list 操作を直接誘発
//! できない）で、本ファイル専用の最小 component（[`ListState`]）を使い、
//! `Runtime::apply_update_for_dirty` の keyed list 構造反映（`Insert`/
//! `Move`/`Remove`）経由で各行要素の `--fandhe-motion-stagger-index` が
//! 0 始まりの DOM 順位置へ再同期されることを固定する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::stagger_index::STAGGER_INDEX_VAR;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

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

/// `keyed_update_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `keyed_update_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// 動的リスト 1 件のみを持つ最小 component（`keyed_update_browser.rs::ListState`
/// と同型。`items` のみが dirty field となる）。
#[derive(Debug, Clone)]
struct ListState {
    /// `(安定キー, 表示内容)` の順序付きリスト。
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
}

impl ListState {
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str)]) -> Self {
        Self {
            items: initial
                .iter()
                .map(|(id, text)| (*id, text.to_string()))
                .collect(),
            dirty: Vec::new(),
        }
    }
}

/// `ListState::decode_action` が復号する型付きアクション。
enum ListAction {
    /// 末尾へ `(id, content)` を 1 件追加する（`Insert` を誘発）。
    Append { id: u64, content: String },
    /// 先頭と末尾を入れ替える（`Move` を誘発。要素数 2 件以上が前提）。
    SwapFirstLast,
    /// `id` の項目を削除する（`Remove` を誘発）。
    Remove { id: u64 },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::Append { id, content } => {
                self.items.push((id, content));
                self.dirty.push(Self::FIELD_ITEMS);
            }
            ListAction::SwapFirstLast => {
                let len = self.items.len();
                if len >= 2 {
                    self.items.swap(0, len - 1);
                    self.dirty.push(Self::FIELD_ITEMS);
                }
            }
            ListAction::Remove { id } => {
                let before = self.items.len();
                self.items.retain(|(existing, _)| *existing != id);
                if self.items.len() != before {
                    self.dirty.push(Self::FIELD_ITEMS);
                }
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|(id, content)| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "stagger-item")],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let list = keyed_list("ul", vec![("id", "stagger-list")], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "stagger-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            // payload 形式: "<id>:<content>"。
            "append" => {
                let (id_str, content) = payload.split_once(':')?;
                Some(ListAction::Append {
                    id: id_str.parse::<u64>().ok()?,
                    content: content.to_string(),
                })
            }
            "swap_first_last" => Some(ListAction::SwapFirstLast),
            // payload 形式: "<id>"。
            "remove" => Some(ListAction::Remove {
                id: payload.parse::<u64>().ok()?,
            }),
            _ => None,
        }
    }
}

impl DirtyTracked for ListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

/// keyed list 内容はテキストノードのみで束縛点を使わないため常に `None`
/// （`keyed_update_browser.rs::ListState` の `BindingSource` 実装と同じ
/// 意図）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `root` へ合成クリックターゲットを追加してクリックし、`data-action`
/// dispatch を発火する（`keyed_update_browser.rs::dispatch_action` と
/// 同じ手法）。
fn dispatch_action(document: &Document, root: &Element, action: &str, payload: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", action)
        .expect("set_attribute must not fail");
    trigger
        .set_attribute("data-payload", payload)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// `root` 配下の `[data-testid='stagger-item']` 各要素の
/// `--fandhe-motion-stagger-index` を DOM 順に読み取る。
fn read_stagger_indices(root: &Element) -> Vec<String> {
    let nodes = root
        .query_selector_all("[data-testid='stagger-item']")
        .expect("query_selector_all must not fail");
    let mut out = Vec::with_capacity(nodes.length() as usize);
    for i in 0..nodes.length() {
        let node = nodes.get(i).expect("index within length must exist");
        let html = node
            .dyn_ref::<HtmlElement>()
            .expect("keyed list item must be an HtmlElement");
        out.push(
            html.style()
                .get_property_value(STAGGER_INDEX_VAR)
                .expect("get_property_value must not fail"),
        );
    }
    out
}

/// 受け入れ条件（挿入位置）: 初期 3 件から末尾へ 1 件 `Insert` すると、
/// 4 行すべてが `"0".."3"` の連番になる。
#[wasm_bindgen_test]
fn insert_renumbers_all_rows_as_contiguous_sequence() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "stagger-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("stagger-root-container-1", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "append", "4:d");

    assert_eq!(
        read_stagger_indices(root),
        vec!["0", "1", "2", "3"],
        "挿入後は全 4 行が DOM 順の連番になること"
    );
}

/// 受け入れ条件（並べ替え）: 先頭・末尾を `Move` した後も、既存行の
/// index が新しい DOM 順へ追随する。
#[wasm_bindgen_test]
fn move_renumbers_rows_to_new_dom_order() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "stagger-root-container-2");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("stagger-root-container-2", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "swap_first_last", "");

    assert_eq!(
        read_stagger_indices(root),
        vec!["0", "1", "2"],
        "並べ替え後も連番 0..N-1 が維持されること"
    );
    assert_eq!(
        runtime
            .component()
            .items
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>(),
        vec![3, 2, 1],
        "先頭・末尾の入れ替えが状態へ反映されていること"
    );
}

/// 受け入れ条件（削除）: 1 件 `Remove` した後、残り行の index が
/// 詰め直され欠番が残らない。
#[wasm_bindgen_test]
fn remove_renumbers_remaining_rows_without_gaps() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "stagger-root-container-3");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("stagger-root-container-3", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "remove", "2");

    assert_eq!(
        read_stagger_indices(root),
        vec!["0", "1"],
        "削除後は残り 2 行が欠番なしの連番になること"
    );
}
