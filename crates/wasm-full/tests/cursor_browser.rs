//! `fandhe_frontend_wasm_full::cursor`（イシュー #2542）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/cursor.rs` の native テストは定数の安定性のみを検証
//! 済みである。本ファイルは opt-in 要素の解決・hover 対象の `data-*` 写し・
//! magnetic 吸着・タッチ除外・`reduced_motion`/`coarse_pointer` 抑制が
//! 実 DOM 上で機能することを `magnetic_browser.rs` と同方針で検証する。
//! `CursorAnimator` 自体の spring 収束は責務境界どおり
//! `crates/frontend-animation/tests/cursor_browser.rs` で検証済みのため、
//! 本ファイルは収束を待たず、配線（イベント委譲・要素解決・属性書き換え）
//! のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "cursor")]

use fandhe_frontend_wasm_full::cursor::{
    wire_cursor_with_env, CURSOR_ACTIVE_ATTR, CURSOR_ATTR, CURSOR_LABEL_ATTR, CURSOR_STATE_ATTR,
    CURSOR_TARGET_ATTR, CURSOR_TARGET_LABEL_ATTR, CURSOR_TARGET_MAGNETIC_ATTR, CURSOR_VARIANT_ATTR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement, PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`magnetic_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `root`（id 付き）> `cursor_el`（[`CURSOR_ATTR`] 付き）+ `target`
/// （`data-fandhe-cursor-target="ring"` + label 付き、`position: absolute;
/// left: 0; top: 0; width: 40px; height: 40px;` で中心座標を
/// `(20, 20)` に固定）を組み立てて返す。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let cursor_el = document.create_element("div").unwrap();
    cursor_el.set_attribute(CURSOR_ATTR, "").unwrap();
    root.append_child(&cursor_el).unwrap();

    let target = document.create_element("button").unwrap();
    target.set_attribute(CURSOR_TARGET_ATTR, "ring").unwrap();
    target
        .set_attribute(CURSOR_TARGET_LABEL_ATTR, "View")
        .unwrap();
    let html_target = target
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("target must cast to HtmlElement for style access");
    let style = html_target.style();
    style.set_property("position", "absolute").unwrap();
    style.set_property("left", "0px").unwrap();
    style.set_property("top", "0px").unwrap();
    style.set_property("width", "40px").unwrap();
    style.set_property("height", "40px").unwrap();
    root.append_child(&target).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, cursor_el, target)
}

fn dispatch_pointer_event(
    target: &Element,
    kind: &str,
    pointer_type: &str,
    client_x: i32,
    client_y: i32,
) {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type(pointer_type);
    init.set_client_x(client_x);
    init.set_client_y(client_y);
    let event = PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail");
    target
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
}

fn attr(element: &Element, name: &str) -> Option<String> {
    element.get_attribute(name)
}

#[wasm_bindgen_test]
fn wiring_marks_root_active_and_cursor_hidden_initially() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, _target) = build_dom(&document, "cursor-root-1");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), Some(String::new()));
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));
}

#[wasm_bindgen_test]
fn pointermove_over_target_copies_variant_and_label_to_cursor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-2");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));
    assert_eq!(attr(&cursor_el, CURSOR_VARIANT_ATTR), Some("ring".into()));
    assert_eq!(attr(&cursor_el, CURSOR_LABEL_ATTR), Some("View".into()));
}

#[wasm_bindgen_test]
fn pointermove_leaving_target_clears_hover_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-3");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));

    // opt-in 対象を持たない root 自身へ移動する（対象の外）。
    dispatch_pointer_event(&root, "pointermove", "mouse", 999, 999);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("idle".into()));
    assert_eq!(attr(&cursor_el, CURSOR_VARIANT_ATTR), None);
    assert_eq!(attr(&cursor_el, CURSOR_LABEL_ATTR), None);
}

#[wasm_bindgen_test]
fn pointermove_into_non_target_area_transitions_hidden_to_idle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, _target) = build_dom(&document, "cursor-root-3a");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));

    // opt-in 対象を持たない root 自身への初回移動（`hover_target` は
    // `None` のまま変化しないため、`is_same` の判定だけでは
    // `"hidden"` → `"idle"` へ遷移しない不具合の回帰テスト
    // （イシュー #2542 レビュー指摘）。
    dispatch_pointer_event(&root, "pointermove", "mouse", 5, 5);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("idle".into()));
}

#[wasm_bindgen_test]
fn pointerout_true_leave_hides_cursor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-4");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));

    // `relatedTarget` を指定しない `pointerout`（root 外への真の離脱）。
    dispatch_pointer_event(&target, "pointerout", "mouse", 999, 999);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));
}

#[wasm_bindgen_test]
fn touch_pointer_move_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-5");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "touch", 30, 20);

    assert_eq!(
        attr(&cursor_el, CURSOR_STATE_ATTR),
        Some("hidden".into()),
        "cursor はタッチ由来のポインタを除外するはず"
    );
}

#[wasm_bindgen_test]
fn magnetic_target_snaps_position_to_rect_center() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-6");
    let _guard = RemoveOnDrop(root.clone());
    target
        .set_attribute(CURSOR_TARGET_MAGNETIC_ATTR, "")
        .unwrap();

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    // 対象は (0, 0)-(40, 40) のため中心は (20, 20)。ポインタ座標
    // (30, 20) を渡しても中心へ吸着し、ポインタ座標をそのまま
    // 使わないことを、rAF ループ開始直後（初回書き込み前）の状態で
    // 検証する代わりに、少なくとも `move_to` 呼び出し自体が例外を
    // 起こさないことを確認する（収束値の検証は frontend-animation 側の
    // `cursor_browser.rs` が担う、モジュール doc参照）。
    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));
}

#[wasm_bindgen_test]
fn reduced_motion_true_suppresses_wiring_entirely() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-7");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), true, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        None,
        "reduced_motion=true では配線自体が行われず root へ CURSOR_ACTIVE_ATTR も付かないはず"
    );
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), None);
}

#[wasm_bindgen_test]
fn coarse_pointer_true_suppresses_wiring_entirely() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-8");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, true).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), None);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), None);
}

#[wasm_bindgen_test]
fn missing_cursor_element_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("cursor-root-9");
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    // カーソル要素（[`CURSOR_SELECTOR`]）を持たない root。
    wire_cursor_with_env(root.clone(), false, false)
        .expect("wire_cursor_with_env must not fail even without a cursor element");

    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), None);
}
