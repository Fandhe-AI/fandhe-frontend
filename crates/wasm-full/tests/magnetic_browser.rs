//! `fandhe_frontend_wasm_full::magnetic`（イシュー #2550）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/magnetic.rs` の native テストは定数の安定性のみを検証
//! 済みである。本ファイルは opt-in 要素の解決・`getBoundingClientRect()`
//! による中心座標計測・CSS カスタムプロパティ書き込み・タッチ除外・
//! `prefers-reduced-motion` 抑制が実 DOM 上で機能することを、
//! `gesture_browser.rs`（手組み DOM への直接 `wire_*` 呼び出し、
//! `Runtime::mount` は経由しない）と同方針で検証する。
//!
//! `fandhe-frontend-animation::magnetic::compute_pull` 自体の計算ロジックは
//! 責務境界どおり native テスト（`crates/frontend-animation/src/
//! magnetic.rs`）で検証済みであり、本ファイルは実 DOM 上での配線
//! （イベント委譲・要素解決・書き込み先の一致）のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "magnetic")]

use fandhe_frontend_wasm_full::magnetic::{wire_magnetic_with_reduced_motion, MAGNETIC_ATTR};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement, PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `root`（id 付き）> `button`（[`MAGNETIC_ATTR`] 付き、`position: absolute;
/// left: 0; top: 0; width: 40px; height: 40px;` で中心座標を
/// `(20, 20)` に固定）を組み立てて返す。中心座標を決定的にすることで、
/// `getBoundingClientRect()` に依存する計算結果を厳密に検証できる。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let button = document.create_element("button").unwrap();
    button.set_attribute(MAGNETIC_ATTR, "").unwrap();
    let html_button = button
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("button must cast to HtmlElement for style access");
    let style = html_button.style();
    style.set_property("position", "absolute").unwrap();
    style.set_property("left", "0px").unwrap();
    style.set_property("top", "0px").unwrap();
    style.set_property("width", "40px").unwrap();
    style.set_property("height", "40px").unwrap();
    root.append_child(&button).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, button)
}

/// `client_x`/`client_y`/`pointer_type` を指定したポインタイベントを
/// `target` へ発火する（`gesture_browser.rs::dispatch_pointermove_at` と
/// 同型）。
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

/// `element` の `--fandhe-motion-magnetic-x`/`-y` の現在値を読む。
fn magnetic_offset(element: &Element) -> (String, String) {
    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must cast to HtmlElement");
    let style = html_element.style();
    (
        style
            .get_property_value("--fandhe-motion-magnetic-x")
            .unwrap_or_default(),
        style
            .get_property_value("--fandhe-motion-magnetic-y")
            .unwrap_or_default(),
    )
}

#[wasm_bindgen_test]
fn pointermove_writes_computed_offset_as_css_custom_properties() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "magnetic-root-1");
    let _guard = RemoveOnDrop(root.clone());

    wire_magnetic_with_reduced_motion(root.clone(), false)
        .expect("wire_magnetic_with_reduced_motion must not fail");

    // 中心座標は (20, 20)。client (30, 20) への移動で
    // dx = (30 - 20) * MAGNETIC_STRENGTH(0.3) = 3.0, dy = 0.0。
    dispatch_pointer_event(&button, "pointermove", "mouse", 30, 20);

    let (x, y) = magnetic_offset(&button);
    assert_eq!(x, "3px", "dx はストレンス係数どおりの値になるはず");
    assert_eq!(y, "0px", "dy は中心と同じ y 座標のため 0 のはず");
}

#[wasm_bindgen_test]
fn pointerout_true_leave_resets_offset_to_zero() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "magnetic-root-2");
    let _guard = RemoveOnDrop(root.clone());

    wire_magnetic_with_reduced_motion(root.clone(), false)
        .expect("wire_magnetic_with_reduced_motion must not fail");

    dispatch_pointer_event(&button, "pointermove", "mouse", 30, 20);
    let (x, _) = magnetic_offset(&button);
    assert_eq!(x, "3px", "移動後は非ゼロのオフセットが書き込まれているはず");

    // `relatedTarget` を指定しない `pointerout`（要素外への真の離脱）で
    // 中立位置へリセットされる。
    dispatch_pointer_event(&button, "pointerout", "mouse", 999, 999);

    let (x, y) = magnetic_offset(&button);
    assert_eq!(
        x, "0px",
        "pointerout でオフセットが中立位置へリセットされるはず"
    );
    assert_eq!(
        y, "0px",
        "pointerout でオフセットが中立位置へリセットされるはず"
    );
}

#[wasm_bindgen_test]
fn touch_pointer_move_does_not_write_offset() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "magnetic-root-3");
    let _guard = RemoveOnDrop(root.clone());

    wire_magnetic_with_reduced_motion(root.clone(), false)
        .expect("wire_magnetic_with_reduced_motion must not fail");

    // magnetic は `gesture.rs` と同じ理由でタッチ由来のポインタを除外する
    // （モジュール doc「タッチ除外」節: タップ操作中に指の微動で `click`
    // 発火位置がずれるのを防ぐ）ため、タッチでは書き込まれないことを
    // 確認する。
    dispatch_pointer_event(&button, "pointermove", "touch", 30, 20);
    let (x, y) = magnetic_offset(&button);
    assert_eq!(x, "", "magnetic はタッチ由来のポインタを除外するはず");
    assert_eq!(y, "", "magnetic はタッチ由来のポインタを除外するはず");
}

#[wasm_bindgen_test]
fn reduced_motion_true_suppresses_wiring_entirely() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "magnetic-root-4");
    let _guard = RemoveOnDrop(root.clone());

    wire_magnetic_with_reduced_motion(root.clone(), true)
        .expect("wire_magnetic_with_reduced_motion must not fail");

    dispatch_pointer_event(&button, "pointermove", "mouse", 30, 20);

    let (x, y) = magnetic_offset(&button);
    assert_eq!(
        x, "",
        "reduced_motion=true では配線自体が行われず未設定のままのはず"
    );
    assert_eq!(y, "", "reduced_motion=true では配線自体が行われないはず");
}

#[wasm_bindgen_test]
fn pointermove_outside_opted_in_element_resets_previous_target() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "magnetic-root-5");
    let _guard = RemoveOnDrop(root.clone());

    wire_magnetic_with_reduced_motion(root.clone(), false)
        .expect("wire_magnetic_with_reduced_motion must not fail");

    dispatch_pointer_event(&button, "pointermove", "mouse", 30, 20);
    let (x, _) = magnetic_offset(&button);
    assert_eq!(x, "3px");

    // opt-in 要素の外（`root` 自身、`MAGNETIC_ATTR` を持たない）へ移動する
    // と、追跡中だった `button` のオフセットがリセットされる。
    dispatch_pointer_event(&root, "pointermove", "mouse", 999, 999);

    let (x, y) = magnetic_offset(&button);
    assert_eq!(
        x, "0px",
        "opt-in 要素の外へ移動した際、前の対象がリセットされるはず"
    );
    assert_eq!(y, "0px");
}
