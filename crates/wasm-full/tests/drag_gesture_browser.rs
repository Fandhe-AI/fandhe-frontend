//! `fandhe_frontend_wasm_full::drag_gesture::wire_drag_gesture`（pointer
//! capture ベースの汎用ドラッグ配線、イシュー #2535）の実ブラウザ統合
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/drag_gesture.rs` の native テストは純粋層
//! （`arrow_key_direction`/`parse_drag_axis`）までを検証済み。本ファイルは
//! その先、`wire_drag_gesture` が実 DOM 上で pointerdown/pointermove/
//! pointerup・keydown（矢印キー）に応じて `--fandhe-drag-x`/`-y`
//! （`fandhe_frontend_animation::drag::DragController` が書き込む CSS
//! カスタムプロパティ）・`data-fandhe-dragging` を正しく更新することを
//! `gesture_browser.rs` と同方針（手組み DOM への直接 `wire_drag_gesture`
//! 呼び出し、`Runtime::mount` は経由しない）で検証する。
//!
//! `setPointerCapture` は合成イベント環境で `NotFoundError` を投げうる
//! （`angle_slider.rs::reattach_pointer_capture` doc 参照）ため、
//! アサーションは `has_pointer_capture` ではなく `data-fandhe-dragging` の
//! 付け外しと `--fandhe-drag-x`/`-y` の値で行う。制約コンテナ
//! （`DRAG_CONSTRAINTS_ATTR`）を持たない構成のみを検証するため、release 時
//! の最終位置は常に release 直前の現在位置と一致し（spring は起動しない）、
//! ブラウザの `prefers-reduced-motion` 設定に依存しない決定的な結果になる。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "drag-gesture")]

use fandhe_frontend_animation::drag::{DRAG_X_PROPERTY, DRAG_Y_PROPERTY};
use fandhe_frontend_wasm_full::drag_gesture::{
    wire_drag_gesture, DRAGGING_STATE_ATTR, DRAG_ATTR, DRAG_AXIS_ATTR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, KeyboardEventInit};
use web_sys::{PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::RemoveOnDrop`
/// と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// root（opt-in 属性なし）> draggable（`DRAG_ATTR` opt-in、`tabindex="0"`
/// でキーボード操作可能）の 2 階層 DOM を組み立てる。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let draggable = document.create_element("div").unwrap();
    draggable.set_attribute(DRAG_ATTR, "").unwrap();
    draggable.set_attribute("tabindex", "0").unwrap();
    root.append_child(&draggable).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, draggable)
}

fn pointer_event(kind: &str, pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("PointerEvent must cast to Event")
}

fn dispatch_key(target: &Element, kind: &str, key: &str) {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init)
        .expect("KeyboardEvent::new must not fail");
    target
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
}

/// `element` の `name` カスタムプロパティを `f64` として読む（未設定は
/// `None`、`DomTarget::custom_property` が書き込む素の数値文字列を想定）。
fn custom_property_px(element: &Element, name: &str) -> Option<f64> {
    let value = element
        .dyn_ref::<HtmlElement>()
        .expect("element must be HtmlElement")
        .style()
        .get_property_value(name)
        .expect("get_property_value must not fail");
    if value.is_empty() {
        None
    } else {
        value.parse::<f64>().ok()
    }
}

#[wasm_bindgen_test]
fn pointer_drag_updates_position_and_dragging_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-pointer-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 1, 100.0, 100.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointerdown 後は data-fandhe-dragging が付与されているべき"
    );

    root.dispatch_event(&pointer_event("pointermove", 1, 140.0, 130.0))
        .expect("dispatch_event must not fail");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        (x - 40.0).abs() < 0.01,
        "pointermove の client_x 差分 40px がそのまま反映されるべき: x={x}"
    );
    assert!(
        (y - 30.0).abs() < 0.01,
        "pointermove の client_y 差分 30px がそのまま反映されるべき: y={y}"
    );

    root.dispatch_event(&pointer_event("pointerup", 1, 140.0, 130.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointerup 後は data-fandhe-dragging が外れているべき"
    );
    // 制約コンテナが無いため release 時のクランプは no-op で、位置は
    // pointerup 直前の値のまま維持される（モジュール doc 参照）。
    let x_after = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    assert!(
        (x_after - 40.0).abs() < 0.01,
        "制約が無い release は位置を変えないはず: x_after={x_after}"
    );
}

#[wasm_bindgen_test]
fn pointer_cancel_also_clears_dragging_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-cancel-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 2, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(draggable.has_attribute(DRAGGING_STATE_ATTR));

    root.dispatch_event(&pointer_event("pointercancel", 2, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointercancel 後も data-fandhe-dragging が外れているべき"
    );
}

#[wasm_bindgen_test]
fn arrow_key_nudges_position_respecting_axis() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-keyboard-root");
    draggable.set_attribute(DRAG_AXIS_ATTR, "x").unwrap();
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    dispatch_key(&draggable, "keydown", "ArrowRight");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        x > 0.0,
        "ArrowRight は x を正方向へ動かすべき（1 ステップ分）: x={x}"
    );
    assert!(
        y == 0.0,
        "data-fandhe-drag-axis=\"x\" のとき ArrowUp/Down 相当の y は動かないべき: y={y}"
    );

    dispatch_key(&draggable, "keydown", "ArrowUp");
    let y_after_up = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        y_after_up == 0.0,
        "軸制約 x では ArrowUp は無視されるべき: y_after_up={y_after_up}"
    );
}

#[wasm_bindgen_test]
fn arrow_key_on_editable_target_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-editable-root");
    let _guard = RemoveOnDrop(root.clone());

    let input = document.create_element("input").unwrap();
    draggable.append_child(&input).unwrap();

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    dispatch_key(&input, "keydown", "ArrowRight");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY);
    assert!(
        x.is_none(),
        "input 上の矢印キーは drag nudge を発火しないべき: x={x:?}"
    );
}
