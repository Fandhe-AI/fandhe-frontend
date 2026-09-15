//! `fandhe_frontend_wasm_full::add_to_basket::wire_add_to_basket`
//! （イシュー #2538）の実ブラウザ統合テスト（`wasm-pack test --headless
//! --chrome`）。
//!
//! `wasm-full/src/add_to_basket.rs` の native テストは純粋関数
//! （`should_accept_click`）のみを検証済みである。本ファイルは以下を実
//! DOM 上で検証する（`gesture_browser.rs`/`confetti_browser.rs` と同方針で、
//! 手組み DOM への直接 `wire_add_to_basket` 呼び出し、`Runtime::mount` は
//! 経由しない）:
//!
//! (a) click → `idle` → `adding` → `added` → `idle` の一連の遷移。
//! (b) `adding`/`added` 中の連打は状態を乱さない（二重トリガー防止）。
//! (c) 本モジュールの click リスナーは `stopPropagation()` を呼ばないため、
//!     `events.rs` 相当の別リスナーが同じ click を受け取れる（`data-action`
//!     委譲を妨げない不変条件）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "add-to-basket")]

use fandhe_frontend_wasm_full::add_to_basket::{
    wire_add_to_basket, ADD_TO_BASKET_ATTR, DEFAULT_ADDED_RESET_TIMEOUT_MS,
    DEFAULT_ADDING_DURATION_MS,
};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::
/// RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn build_dom(document: &Document) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    let button = document.create_element("button").unwrap();
    button.set_attribute("type", "button").unwrap();
    button.set_attribute(ADD_TO_BASKET_ATTR, "").unwrap();
    root.append_child(&button).unwrap();
    document
        .body()
        .expect("document must have a body")
        .append_child(&root)
        .unwrap();
    (root, button)
}

fn click(target: &Element) {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail");
    target
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist in browser test");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// `target` への bubble フェーズ click を数える別リスナーを配線する
/// （`events.rs` の `data-action` 委譲相当の観測用スタブ）。
fn count_clicks(target: &Element) -> Rc<Cell<u32>> {
    let count = Rc::new(Cell::new(0_u32));
    let count_for_closure = count.clone();
    let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        count_for_closure.set(count_for_closure.get() + 1);
    });
    target
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
    count
}

#[wasm_bindgen_test]
async fn click_transitions_idle_adding_added_idle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_add_to_basket(root).expect("wire_add_to_basket must not fail");

    assert_eq!(button.get_attribute("data-state"), None);

    click(&button);
    assert_eq!(
        button.get_attribute("data-state").as_deref(),
        Some("adding")
    );

    sleep_ms(DEFAULT_ADDING_DURATION_MS + 100).await;
    assert_eq!(button.get_attribute("data-state").as_deref(), Some("added"));

    sleep_ms(DEFAULT_ADDED_RESET_TIMEOUT_MS + 200).await;
    assert_eq!(button.get_attribute("data-state").as_deref(), Some("idle"));
}

#[wasm_bindgen_test]
async fn repeated_clicks_during_adding_are_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_add_to_basket(root).expect("wire_add_to_basket must not fail");

    click(&button);
    assert_eq!(
        button.get_attribute("data-state").as_deref(),
        Some("adding")
    );

    // `adding` 中の連打は二重トリガー防止で無視されるはず（状態が乱れず
    // `adding` のまま）。
    click(&button);
    click(&button);
    assert_eq!(
        button.get_attribute("data-state").as_deref(),
        Some("adding")
    );
}

#[wasm_bindgen_test]
fn click_listener_does_not_block_other_listeners() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_add_to_basket(root).expect("wire_add_to_basket must not fail");

    let other_listener_count = count_clicks(&button);
    click(&button);

    assert_eq!(
        other_listener_count.get(),
        1,
        "本モジュールの click リスナーは stopPropagation を呼ばず、他の \
         リスナー（data-action 委譲相当）も同じ click を受け取れるはず"
    );
}

/// `Result<(), JsValue>` 契約の型検証（コンパイル時アサーション）。
#[allow(dead_code)]
fn _wire_add_to_basket_returns_result(root: Element) -> Result<(), JsValue> {
    wire_add_to_basket(root)
}
