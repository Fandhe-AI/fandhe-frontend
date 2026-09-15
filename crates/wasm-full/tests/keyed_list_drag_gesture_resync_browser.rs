//! codex-review P1 是正（イシュー #2535、PR #2565）の実ブラウザ統合
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `stagger_index_browser.rs` と同じ理由（製品コンポーネント
//! `interactive::AppState` では検証したい keyed list 操作を直接誘発
//! できない）で、本ファイル専用の最小 component（[`ListState`]）を使い、
//! `Runtime::apply_update_for_dirty` の keyed list 構造反映（`Insert`）
//! 経由で新規挿入された `data-fandhe-drag` 要素へ `touch-action: none`
//! が最初の `pointerdown` より前に先行反映されることを固定する
//! （`Self::apply_subtree_swap` のみが再同期していた漏れの是正。
//! `resync_drag_gesture_attachments` doc 参照）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "drag-gesture")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::drag_gesture::DRAG_ATTR;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// `stagger_index_browser.rs::create_placeholder` と同じ意図。
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

/// `stagger_index_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `stagger_index_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// 動的リスト 1 件のみを持つ最小 component。各行が opt-in ドラッグ要素
/// （[`DRAG_ATTR`]）であり、`items` のみが dirty field となる
/// （`stagger_index_browser.rs::ListState` と同型）。
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
                        vec![("data-testid", "drag-item"), (DRAG_ATTR, "")],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let list = keyed_list("ul", vec![("id", "drag-list")], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "drag-root")], vec![list])
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
/// （`stagger_index_browser.rs::ListState` の `BindingSource` 実装と同じ
/// 意図）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `stagger_index_browser.rs::dispatch_action` と同じ手法。
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

/// 受け入れ条件（本 PR の主眼）: 初期 1 件から末尾へ 1 件 `Insert` すると、
/// **一度も `pointerdown` を発火していない**新規挿入要素にも
/// `touch-action: none` が先行反映される（初回タッチが UA のスクロール
/// 判定に間に合わず `pointercancel` で中断する不具合の回帰固定）。
#[wasm_bindgen_test]
fn insert_prewires_touch_action_on_new_drag_item_before_any_pointerdown() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "drag-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a")]);
    let runtime = Runtime::mount("drag-root-container-1", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "append", "2:b");

    let items = root
        .query_selector_all("[data-testid='drag-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(items.length(), 2, "挿入後は 2 行になっていること");

    let inserted = items
        .get(1)
        .expect("second item must exist after insert")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");
    assert_eq!(
        inserted.style().get_property_value("touch-action"),
        Ok("none".to_string()),
        "新規挿入された data-fandhe-drag 要素は pointerdown 前から \
         touch-action: none を持つこと"
    );
}
