//! `fandhe_frontend_wasm_full::message_scroller`（イシュー #2122、親 #2120）の
//! 実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/tests/message_scroller_native.rs` は headless-ui 出力
//! とのドリフト検知を担う。本ファイルはその先、**実ブラウザ（headless
//! Chromium、`wasm-pack test --headless --chrome`）上での**最下部追従・
//! 新着検知・履歴読み込み時のスクロール位置維持の実 DOM 挙動を検証する
//! （`content_height_browser.rs`/`questionnaire_browser.rs` と同型の実 DOM
//! 検証パターンを踏襲する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el_owned, render, text, Node};
use fandhe_frontend_headless_ui::message_scroller::{
    content, jump_to_latest, load_more, root, viewport, MessageScrollerRootProps,
    MessageScrollerStuck as HeadlessStuck,
};
use fandhe_frontend_wasm_full::events::ActionRef;
use fandhe_frontend_wasm_full::message_scroller::{wire_message_scroller_events, ACTION_LOAD_MORE};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`content_height_browser.rs::create_container` と同型）。
fn create_container(document: &Document, id: &str) -> Element {
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`content_height_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 固定高さの子要素（`style="height:{px}px"`）を組み立てる
/// （`content_height_browser.rs::fixed_height_child` と同型。`scrollHeight`
/// を決定的にするため）。
fn fixed_height_child(px: u32) -> Node {
    el_owned(
        "div",
        vec![("style".to_string(), format!("height:{px}px"))],
        vec![],
    )
}

/// `instance_id` の Message Scroller 1 インスタンスを組み立てる。viewport
/// は CSSOM で `height:100px; overflow-y:auto` を固定し、`scrollHeight`/
/// `scrollTop`/`clientHeight` を決定的にする。
fn build_message_scroller(instance_id: &str, stuck: HeadlessStuck, item_heights: &[u32]) -> Node {
    let items: Vec<Node> = item_heights
        .iter()
        .map(|px| fixed_height_child(*px))
        .collect();
    root(
        MessageScrollerRootProps {
            stuck,
            has_new: false,
        },
        vec![("id", instance_id)],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], items)],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    )
}

fn mount(container: &Element, node: &Node) -> Element {
    container.set_inner_html(&render(node));
    container
        .first_element_child()
        .expect("message-scroller root must exist")
}

fn find_viewport(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="viewport"]"#)
        .expect("query_selector must not fail")
        .expect("viewport must exist")
}

fn find_jump_to_latest(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="jump-to-latest"]"#)
        .expect("query_selector must not fail")
        .expect("jump-to-latest must exist")
}

fn find_load_more(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="load-more"]"#)
        .expect("query_selector must not fail")
        .expect("load-more must exist")
}

fn find_content(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("content must exist")
}

fn overflow_anchor(viewport: &Element) -> String {
    viewport
        .dyn_ref::<HtmlElement>()
        .expect("viewport must be HtmlElement")
        .style()
        .get_property_value("overflow-anchor")
        .expect("get_property_value must not fail")
}

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

fn dispatch_click(target: &Element) {
    target
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
}

/// `viewport.scrollTop` を書き換えた上で合成 `scroll` イベントを送出する
/// （利用者スクロールの模擬。決定性のため合成イベントを使う）。
fn simulate_user_scroll(viewport: &Element, scroll_top: i32) {
    viewport.set_scroll_top(scroll_top);
    let init = EventInit::new();
    init.set_bubbles(false);
    let event = Event::new_with_event_init_dict("scroll", &init)
        .expect("Event::new must not fail for scroll");
    viewport
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

/// `condition` が真になるまでマイクロタスク/タイマー待機を繰り返す
/// （`headless_avatar_browser.rs::wait_for` と同型。`MutationObserver`
/// コールバックはマイクロタスクキューで実行されるため、同期的な DOM 変異
/// 直後には観測できない）。
async fn wait_for(mut condition: impl FnMut() -> bool) {
    use wasm_bindgen::closure::Closure;

    for _ in 0..200 {
        if condition() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&wasm_bindgen::JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10,
                )
                .expect("setTimeout must not fail");
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("timeout promise must resolve");
    }
}

// --- 初期同期 ---

#[wasm_bindgen_test]
async fn initial_sync_bottom_scrolls_to_bottom_and_sets_overflow_anchor_none() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-1", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    assert_eq!(viewport.scroll_top(), expected);
    assert_eq!(overflow_anchor(&viewport), "none");
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("hidden"));
    assert!(!jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn initial_sync_free_does_not_scroll() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-2", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    assert_eq!(
        viewport.scroll_top(),
        0,
        "free の初期状態はスクロールしないこと"
    );
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
    assert!(!jump.has_attribute("hidden"));
}

#[wasm_bindgen_test]
async fn initial_sync_unknown_stuck_value_does_not_scroll_fail_closed() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-evil-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-evil", HeadlessStuck::Bottom, &[60, 60, 60]);
    container.set_inner_html(&render(&node));
    let instance_root = container
        .first_element_child()
        .expect("message-scroller root must exist");
    // 改ざんを模して data-stuck を不明値へ書き換える。
    instance_root
        .set_attribute("data-stuck", "evil")
        .expect("set_attribute must not fail");
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    assert_eq!(
        viewport.scroll_top(),
        0,
        "不明な data-stuck 値では自動スクロールしないこと（fail-closed）"
    );
}

// --- 利用者スクロール ---

#[wasm_bindgen_test]
async fn user_scroll_away_from_bottom_sets_free_and_shows_jump_to_latest() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-scroll-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-3", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
    assert!(!jump.has_attribute("hidden"));
}

#[wasm_bindgen_test]
async fn jump_to_latest_click_scrolls_to_bottom_and_restores_stuck() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-jump-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-4", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    dispatch_click(&jump);

    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
    let expected = viewport.scroll_height() - viewport.client_height();
    assert_eq!(viewport.scroll_top(), expected);
    assert!(jump.has_attribute("hidden"));
    assert!(!jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn jump_to_latest_click_is_noop_when_disabled_ancestor() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-jump-disabled-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-5", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    jump.set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");
    dispatch_click(&jump);

    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free"),
        "disabled 祖先を持つ jump-to-latest クリックは no-op であること"
    );
}

// --- 新着検知 ---

#[wasm_bindgen_test]
async fn append_while_free_marks_has_new_without_scrolling() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-append-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-6", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let before_top = viewport.scroll_top();
    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(instance_root.has_attribute("data-has-new"));
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "free での末尾追記は scrollTop を変えないこと"
    );
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn append_while_bottom_follows_to_new_bottom_without_has_new() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-append-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-7", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(viewport.scroll_top(), expected);
    assert!(!instance_root.has_attribute("data-has-new"));
}

// --- 履歴読み込み時の位置維持（先頭挿入） ---

#[wasm_bindgen_test]
async fn prepend_while_free_preserves_visual_position() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-prepend-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-8", HeadlessStuck::Free, &[60, 60, 60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    // free のまま、利用者が少し下へスクロールした状態を作る。
    simulate_user_scroll(&viewport, 40);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // 先頭へ 2 件（各 40px）挿入する。
    let prepend_html = format!(
        "{}{}",
        render(&fixed_height_child(40)),
        render(&fixed_height_child(40))
    );
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 80;
    wait_for(|| viewport.scroll_height() == before_height + expected_delta).await;
    assert_eq!(viewport.scroll_height(), before_height + expected_delta);
    wait_for(|| viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(viewport.scroll_top(), before_top + expected_delta);
}

#[wasm_bindgen_test]
async fn prepend_while_bottom_stays_at_bottom_after_correction() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-prepend-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-9", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let prepend_html = render(&fixed_height_child(50));
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(viewport.scroll_top(), expected);
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
}

// --- load-more 通知 ---

#[wasm_bindgen_test]
async fn load_more_click_notifies_app_with_instance_id_payload() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-load-more-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-10", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(instance_root.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail");

    let load_more = find_load_more(&instance_root);
    dispatch_click(&load_more);

    assert_eq!(received.borrow().len(), 1);
    let action_ref = received.borrow()[0].clone();
    assert_eq!(action_ref.action, ACTION_LOAD_MORE);
    assert_eq!(action_ref.payload, "ms-10");
}

#[wasm_bindgen_test]
async fn load_more_click_is_noop_when_loading_or_disabled() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-load-more-loading-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-11", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(instance_root.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail");

    let load_more = find_load_more(&instance_root);
    load_more
        .set_attribute("data-loading", "")
        .expect("set_attribute must not fail");
    dispatch_click(&load_more);
    assert!(
        received.borrow().is_empty(),
        "data-loading 中は通知しないこと"
    );

    load_more
        .remove_attribute("data-loading")
        .expect("remove_attribute must not fail");
    load_more
        .set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");
    dispatch_click(&load_more);
    assert!(
        received.borrow().is_empty(),
        "data-disabled 中は通知しないこと"
    );
}

// --- 非搭載アプリへの副作用なし ---

#[wasm_bindgen_test]
async fn wiring_is_noop_when_no_message_scroller_instance_present() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-no-instance-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let plain = el_owned("div", vec![], vec![text("no message-scroller here")]);
    container.set_inner_html(&render(&plain));
    let root_el = container
        .first_element_child()
        .expect("plain root must exist");

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(root_el.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail on non-instrumented root");

    dispatch_click(&root_el);
    assert!(received.borrow().is_empty());
}

// --- 複数インスタンスの独立性 ---

#[wasm_bindgen_test]
async fn two_instances_in_same_container_are_independent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-two-instances-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let wrapper = el_owned(
        "div",
        vec![],
        vec![
            build_message_scroller("ms-a", HeadlessStuck::Bottom, &[60, 60, 60]),
            build_message_scroller("ms-b", HeadlessStuck::Free, &[60, 60, 60]),
        ],
    );
    container.set_inner_html(&render(&wrapper));
    let outer = container.first_element_child().expect("wrapper must exist");

    wire_message_scroller_events(outer.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let instance_a = outer
        .query_selector("#ms-a")
        .expect("query_selector must not fail")
        .expect("ms-a must exist");
    let instance_b = outer
        .query_selector("#ms-b")
        .expect("query_selector must not fail")
        .expect("ms-b must exist");
    let content_b = find_content(&instance_b);

    // b（free）へ追記しても a（bottom）の data-* へ波及しないこと。
    let new_child = render(&fixed_height_child(40));
    content_b
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_b.has_attribute("data-has-new")).await;
    assert!(instance_b.has_attribute("data-has-new"));
    assert!(
        !instance_a.has_attribute("data-has-new"),
        "片方への追記がもう片方の data-* へ波及しないこと"
    );
    assert_eq!(
        instance_a.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
}
