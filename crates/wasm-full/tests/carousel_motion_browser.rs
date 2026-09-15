//! `fandhe_frontend_wasm_full::carousel_motion`（イシュー #2541）の実
//! ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `magnetic_browser.rs` と同方針: `wire_carousel_motion_events` を手組み
//! DOM へ直接配線し（`Runtime::mount` は経由しない）、opt-in root/
//! `item-group`/`item` の解決・pointer capture・`--fandhe-carousel-index`
//! への書き込み・settle 後の `"goto"` dispatch・5px 未満の移動での
//! no-op・opt-in 属性なしの carousel への無影響を検証する。
//! `fandhe_frontend_animation::carousel::snap_target` 自体の計算ロジックは
//! 責務境界どおり native テスト（`crates/frontend-animation/src/
//! carousel.rs`）で検証済み。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "carousel-motion")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_wasm_full::carousel_motion::{
    wire_carousel_motion_events, CAROUSEL_DRAGGING_STATE_ATTR, CAROUSEL_DRAG_ATTR,
};
use fandhe_frontend_wasm_full::events::ActionRef;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, HtmlElement, MouseEvent, MouseEventInit, PointerEvent, PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms)
            .expect("setTimeout must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("setTimeout promise must not reject");
}

/// `root`（opt-in `drag_attr_value` 付き）> `item-group`（3 `item`、各
/// `width: 100px`）を組み立てて返す。`drag_attr_value` が `None` なら
/// opt-in 属性自体を付けない。
fn build_dom(document: &Document, drag_attr_value: Option<&str>) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    if let Some(value) = drag_attr_value {
        root.set_attribute(CAROUSEL_DRAG_ATTR, value).unwrap();
    }

    let item_group = document.create_element("div").unwrap();
    item_group.set_attribute("data-scope", "carousel").unwrap();
    item_group.set_attribute("data-part", "item-group").unwrap();

    for _ in 0..3 {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "carousel").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        let html_item = item.clone().dyn_into::<HtmlElement>().unwrap();
        let style = html_item.style();
        style.set_property("display", "inline-block").unwrap();
        style.set_property("width", "100px").unwrap();
        style.set_property("height", "40px").unwrap();
        item_group.append_child(&item).unwrap();
    }
    root.append_child(&item_group).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    (root, item_group)
}

fn dispatch_pointer_event(target: &Element, kind: &str, client_x: i32, pointer_id: i32) {
    let init = PointerEventInit::new();
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x);
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = PointerEvent::new_with_event_init_dict(kind, &init).unwrap();
    target.dispatch_event(&event).unwrap();
}

fn read_index(element: &Element) -> Option<f64> {
    element
        .dyn_ref::<HtmlElement>()
        .and_then(|el| {
            el.style()
                .get_property_value("--fandhe-carousel-index")
                .ok()
        })
        .and_then(|v| v.trim().parse::<f64>().ok())
}

#[wasm_bindgen_test]
async fn drag_release_dispatches_goto_after_settle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be set after pointerdown"
    );
    // 150px 左へドラッグ（1 スライド = 100px の 1.5 スライド分）。
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    let dragging_value = read_index(&item_group).expect("progress must be written while dragging");
    assert!(
        (dragging_value - 1.5).abs() < 1e-6,
        "dragging value should be 1.5: {dragging_value}"
    );

    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    sleep_ms(2_000).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        (final_value - 2.0).abs() < 0.01,
        "settled value should converge to 2.0: {final_value}"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be removed after settle"
    );
    let actions = dispatched.borrow();
    assert_eq!(actions.len(), 1, "goto should dispatch exactly once");
    assert_eq!(actions[0].action, "goto");
    assert_eq!(actions[0].payload, "2");
}

#[wasm_bindgen_test]
async fn small_move_settles_back_to_origin_index() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    // 2px の移動は snap_target を四捨五入すると起点の index 0 のまま。
    dispatch_pointer_event(&item_group, "pointermove", 2, 1);
    dispatch_pointer_event(&item_group, "pointerup", 2, 1);
    sleep_ms(500).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        final_value.abs() < 0.01,
        "tiny move should settle back to index 0: {final_value}"
    );
}

#[wasm_bindgen_test]
async fn carousel_without_opt_in_attribute_is_untouched() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, None);
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);
    sleep_ms(500).await;

    assert!(
        read_index(&item_group).is_none(),
        "no opt-in attribute should leave --fandhe-carousel-index unset"
    );
    assert!(dispatched.borrow().is_empty());
}

/// codex-review 指摘 是正（イシュー #2541 第 3 ラウンド）の回帰: `"goto"`
/// dispatch は spring 収束を待たず release 時に同期実行される。収束中に
/// `next-trigger` を操作すると進行中の spring は打ち切られ（見た目の
/// 上書きが止まる）、`data-fandhe-carousel-dragging` も除去されるが、
/// release 時に確定済みの dispatch 自体は取り消されない（1 回のみ）。
#[wasm_bindgen_test]
async fn nav_trigger_click_cancels_pending_settle_without_reverting_dispatch() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let next_trigger = document.create_element("button").unwrap();
    next_trigger
        .set_attribute("data-scope", "carousel")
        .unwrap();
    next_trigger
        .set_attribute("data-part", "next-trigger")
        .unwrap();
    root.append_child(&next_trigger).unwrap();

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    assert_eq!(
        dispatched.borrow().len(),
        1,
        "goto should dispatch synchronously at release"
    );
    assert_eq!(dispatched.borrow()[0].payload, "2");

    let click_init = MouseEventInit::new();
    click_init.set_bubbles(true);
    click_init.set_cancelable(true);
    let click_event = MouseEvent::new_with_mouse_event_init_dict("click", &click_init).unwrap();
    next_trigger.dispatch_event(&click_event).unwrap();

    let frozen_value = read_index(&item_group).expect("progress must remain written");
    sleep_ms(300).await;
    let after_click_value = read_index(&item_group).expect("progress must remain written");
    assert!(
        (frozen_value - after_click_value).abs() < 1e-9,
        "spring should stop writing --fandhe-carousel-index once cancelled: \
         {frozen_value} vs {after_click_value}"
    );
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "cancelled settle must not dispatch a stale goto later"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared when settle is cancelled"
    );
}

/// codex-review 指摘 是正（イシュー #2541 第 3 ラウンド）の回帰: pointer
/// capture が暗黙に失われ `pointerup`/`pointercancel` が一切届かなくても
/// （root 外での release 等）、`lostpointercapture` だけでドラッグ終了処理
/// （settle への goto dispatch・`DragMeta` の回収）が行われ、次の
/// `pointerdown` が「進行中のドラッグがある」判定で拒否され続けない。
#[wasm_bindgen_test]
async fn lostpointercapture_recovers_drag_state_when_pointerup_is_missed() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    // pointerup は一切発火せず、capture 喪失のみを模した
    // `lostpointercapture` を直接発火する（root 外での release や OS 都合
    // による暗黙の capture 喪失を模す）。
    dispatch_pointer_event(&item_group, "lostpointercapture", -150, 1);

    assert_eq!(
        dispatched.borrow().len(),
        1,
        "lostpointercapture alone must still finalize the drag with a goto dispatch"
    );
    assert_eq!(dispatched.borrow()[0].payload, "2");

    dispatch_pointer_event(&item_group, "pointerdown", 0, 2);
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "a fresh pointerdown must be accepted after lostpointercapture recovery"
    );
}
