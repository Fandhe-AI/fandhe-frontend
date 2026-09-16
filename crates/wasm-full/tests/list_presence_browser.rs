//! `list_presence::capture_before`/`play_exit_after`（イシュー #2544）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `stagger_index_browser.rs` と同型の最小 component（[`ListState`]）を
//! 使い、`Runtime::apply_update_for_dirty` の keyed list `Remove` 経由で
//! ゴースト（`data-state="exiting"`）が list 末尾へ現れ、フレームワーク
//! 属性を持たず、keyed 行数から除外されることを固定する。

#![cfg(all(target_arch = "wasm32", feature = "presence"))]

use fandhe_frontend_core::keyed::{keyed_list, KEY_ATTR};
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::list_presence::PRESENCE_AUTO_ATTR;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

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

/// ゴーストへ実測可能な `animation-duration` を与える `<style>` を
/// `document.head` へ 1 回だけ挿入する（`position_browser.rs::
/// ensure_fixed_floating_size_stylesheet` と同じ「一度だけ挿入」方針）。
///
/// `fandhe_frontend_animation::presence::remove_when_settled` は
/// computed `animation-duration`/`animation-delay` の合計が 0ms なら
/// ゴーストを**同一呼び出し内で同期的に**除去する（フェイルセーフ、
/// `presence.rs` doc 参照）。本テストファイルは CSS を一切読み込まない
/// ため、このスタイルシートを注入しないと `play_exit_after` の呼び出し
/// 直後（`dispatch_action` から戻った時点）には既にゴーストが除去済み
/// になり、`remove_leaves_exiting_ghost_without_framework_attrs` の
/// 属性検証（`data-state`/`aria-hidden`/剥離済みフレームワーク属性）が
/// 対象を見失う。
fn ensure_exit_animation_stylesheet(document: &Document) {
    const STYLE_ID: &str = "fandhe-test-presence-exit-style";
    if document.get_element_by_id(STYLE_ID).is_some() {
        return;
    }
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a plain style element");
    style.set_id(STYLE_ID);
    style.set_text_content(Some(
        "[data-state='exiting']{animation:fandhe-test-presence-exit 50ms linear;}\
         @keyframes fandhe-test-presence-exit{from{opacity:1;}to{opacity:0;}}",
    ));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a detached style element");
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

/// 動的リスト 1 件のみを持つ最小 component（`stagger_index_browser.rs::
/// ListState` と同型。`presence_auto` フラグでオプトインの有無を切替）。
#[derive(Debug, Clone)]
struct ListState {
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
    presence_auto: bool,
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
            presence_auto: true,
        }
    }

    fn new_without_presence_auto(initial: &[(u64, &str)]) -> Self {
        Self {
            presence_auto: false,
            ..Self::new(initial)
        }
    }
}

enum ListAction {
    Remove { id: u64 },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
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
                        vec![("data-testid", "presence-item")],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let mut parent_attrs = vec![("id", "presence-list")];
        if self.presence_auto {
            parent_attrs.push((PRESENCE_AUTO_ATTR, ""));
        }
        let list = keyed_list("ul", parent_attrs, "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "presence-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
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

impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

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

fn keyed_row_count(list: &Element) -> usize {
    list.query_selector_all(&format!("[{KEY_ATTR}]"))
        .expect("query_selector_all must not fail")
        .length() as usize
}

/// 受け入れ条件: [`PRESENCE_AUTO_ATTR`] 付きリストで `Remove` すると、
/// list 末尾にゴースト（`data-state="exiting"`）が現れ、`data-key` を
/// 持たない（keyed 行数から除外される）。
#[wasm_bindgen_test]
fn remove_leaves_exiting_ghost_without_framework_attrs() {
    let document = web_sys::window().unwrap().document().unwrap();
    ensure_exit_animation_stylesheet(&document);
    let placeholder = create_placeholder(&document, "presence-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("presence-root-container-1", state).expect("mount must succeed");
    let root = runtime.root();
    let list = root
        .query_selector("#presence-list")
        .expect("query_selector must not fail")
        .expect("presence-list must exist");

    assert_eq!(keyed_row_count(&list), 3, "削除前は 3 keyed 行");

    dispatch_action(&document, root, "remove", "2");

    assert_eq!(
        keyed_row_count(&list),
        2,
        "削除後は keyed 行が 2 件（ゴーストは data-key を持たないため \
         数えられない）"
    );

    let ghosts = list
        .query_selector_all("[data-state='exiting']")
        .expect("query_selector_all must not fail");
    assert_eq!(ghosts.length(), 1, "ゴーストが 1 件生成されること");

    let ghost = ghosts
        .get(0)
        .expect("index 0 must exist")
        .dyn_into::<HtmlElement>()
        .expect("ghost must be an HtmlElement");
    assert!(
        !ghost.has_attribute(KEY_ATTR),
        "ゴーストは data-key を剥がされていること"
    );
    assert!(
        !ghost.has_attribute("data-action"),
        "ゴーストは data-action を剥がされていること"
    );
    assert_eq!(ghost.get_attribute("aria-hidden").as_deref(), Some("true"));
}

/// 受け入れ条件（オプトアウト）: [`PRESENCE_AUTO_ATTR`] を持たないリスト
/// は `Remove` してもゴーストを生成しない（既存の即時 `remove_child`
/// のまま）。
#[wasm_bindgen_test]
fn remove_without_presence_auto_attr_does_not_create_ghost() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "presence-root-container-2");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new_without_presence_auto(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("presence-root-container-2", state).expect("mount must succeed");
    let root = runtime.root();
    let list = root
        .query_selector("#presence-list")
        .expect("query_selector must not fail")
        .expect("presence-list must exist");

    dispatch_action(&document, root, "remove", "2");

    assert_eq!(keyed_row_count(&list), 2, "削除後は keyed 行が 2 件");
    let ghosts = list
        .query_selector_all("[data-state='exiting']")
        .expect("query_selector_all must not fail");
    assert_eq!(
        ghosts.length(),
        0,
        "オプトインしていないリストはゴーストを生成しないこと"
    );
}

/// 受け入れ条件（後続更新の回帰）: ゴースト存在中に別の keyed 更新
/// （さらなる `Remove`）を行っても keyed 行数が壊れない。
#[wasm_bindgen_test]
fn subsequent_update_while_ghost_present_keeps_keyed_rows_consistent() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "presence-root-container-3");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("presence-root-container-3", state).expect("mount must succeed");
    let root = runtime.root();
    let list = root
        .query_selector("#presence-list")
        .expect("query_selector must not fail")
        .expect("presence-list must exist");

    dispatch_action(&document, root, "remove", "2");
    assert_eq!(keyed_row_count(&list), 2);

    dispatch_action(&document, root, "remove", "1");
    assert_eq!(
        keyed_row_count(&list),
        1,
        "ゴースト存在中の追加削除でも keyed 行数は正しく詰まること"
    );
}
