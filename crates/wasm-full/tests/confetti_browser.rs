//! `fandhe_frontend_wasm_full::confetti::wire_confetti`（イシュー #2533）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/confetti.rs` の native テストは定数の安定性のみを検証
//! 済みである。本ファイルはクリック委譲・ロケータ契約（`root` 包含・
//! `data-fandhe-confetti-canvas` マーカー必須）が実 DOM 上で機能することを、
//! `gesture_browser.rs`（手組み DOM への直接 `wire_confetti` 呼び出し、
//! `Runtime::mount` は経由しない）と同方針で検証する。
//!
//! `fandhe-frontend-animation::confetti::fire` が実際にパーティクルを
//! canvas へ描画し発火完了時にクリアすることは、責務境界どおり
//! `crates/frontend-animation/tests/confetti_canvas_browser.rs` が検証
//! 済みである。本ファイルは `wasm-full` が canvas 系 web-sys feature
//! （`CanvasRenderingContext2d`/`HtmlCanvasElement`）を一切追加しない方針
//! （`crates/wasm-full/Cargo.toml` の `confetti` feature コメント参照）を
//! 保つため、`fire()` が実行されたことを canvas の `width`/`height`
//! 反映属性（`HTMLCanvasElement.width`/`.height` は content attribute へ
//! 反映される、WHATWG canvas 仕様）の変化という、素の
//! `Element::get_attribute` のみで観測できる副作用で判定する。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "confetti")]

use fandhe_frontend_wasm_full::confetti::{
    wire_confetti, CONFETTI_CANVAS_ATTR, CONFETTI_TRIGGER_ATTR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::
/// RemoveOnDrop` と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click`（bubbles: true）を `target` へ発火する
/// （`chart_range_browser.rs::click` と同型）。
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

/// `root`（id 付き）> `trigger`（`data-fandhe-confetti-trigger="canvas_id"`）
/// と、`root` 配下の `canvas`（`id=canvas_id`・
/// `data-fandhe-confetti-canvas`・CSS サイズ指定込み）を組み立てて返す。
/// `mark_canvas` が `false` の場合はマーカー属性を付けない（ロケータ契約の
/// 否定テスト用）。
fn build_dom(
    document: &Document,
    root_id: &str,
    canvas_id: &str,
    mark_canvas: bool,
) -> (Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let trigger = document.create_element("button").unwrap();
    trigger
        .set_attribute(CONFETTI_TRIGGER_ATTR, canvas_id)
        .unwrap();
    root.append_child(&trigger).unwrap();

    let canvas = document.create_element("canvas").unwrap();
    canvas.set_id(canvas_id);
    if mark_canvas {
        canvas.set_attribute(CONFETTI_CANVAS_ATTR, "").unwrap();
    }
    let html_canvas = canvas
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("canvas must cast to HtmlElement for style access");
    html_canvas.style().set_property("width", "80px").unwrap();
    html_canvas.style().set_property("height", "60px").unwrap();
    root.append_child(&canvas).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, trigger, canvas)
}

#[wasm_bindgen_test]
async fn click_on_trigger_fires_and_resizes_canvas_to_css_size() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger, canvas) =
        build_dom(&document, "confetti-root-1", "confetti-canvas-1", true);
    let _guard = RemoveOnDrop(root.clone());

    assert_eq!(
        canvas.get_attribute("width"),
        None,
        "fire() 実行前は width content attribute が未設定のはず"
    );

    wire_confetti(root.clone()).expect("wire_confetti must not fail");
    click(&trigger);

    assert_eq!(
        canvas.get_attribute("width").as_deref(),
        Some("80"),
        "fire() が CSS 表示サイズへ canvas.width を設定するはず"
    );
    assert_eq!(
        canvas.get_attribute("height").as_deref(),
        Some("60"),
        "fire() が CSS 表示サイズへ canvas.height を設定するはず"
    );
}

#[wasm_bindgen_test]
async fn click_is_ignored_when_canvas_lacks_marker_attribute() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger, canvas) =
        build_dom(&document, "confetti-root-2", "confetti-canvas-2", false);
    let _guard = RemoveOnDrop(root.clone());

    wire_confetti(root.clone()).expect("wire_confetti must not fail");
    click(&trigger);

    assert_eq!(
        canvas.get_attribute("width"),
        None,
        "data-fandhe-confetti-canvas を持たない要素は解決対象にならないはず"
    );
}

#[wasm_bindgen_test]
async fn click_is_ignored_when_target_canvas_is_outside_root() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("confetti-root-3");
    let trigger = document.create_element("button").unwrap();
    trigger
        .set_attribute(CONFETTI_TRIGGER_ATTR, "confetti-canvas-3")
        .unwrap();
    root.append_child(&trigger).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _root_guard = RemoveOnDrop(root.clone());

    // canvas は root の外（body 直下）に配置する。
    let canvas = document.create_element("canvas").unwrap();
    canvas.set_id("confetti-canvas-3");
    canvas.set_attribute(CONFETTI_CANVAS_ATTR, "").unwrap();
    document.body().unwrap().append_child(&canvas).unwrap();
    let _canvas_guard = RemoveOnDrop(canvas.clone());

    wire_confetti(root.clone()).expect("wire_confetti must not fail");
    click(&trigger);

    assert_eq!(
        canvas.get_attribute("width"),
        None,
        "root 配下にない canvas は解決対象にならないはず"
    );
}

#[wasm_bindgen_test]
async fn click_with_missing_target_id_does_not_panic() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("confetti-root-4");
    let trigger = document.create_element("button").unwrap();
    trigger
        .set_attribute(CONFETTI_TRIGGER_ATTR, "does-not-exist")
        .unwrap();
    root.append_child(&trigger).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    wire_confetti(root.clone()).expect("wire_confetti must not fail");
    // panic しないことそのものが本テストの検証対象。
    click(&trigger);
}

#[wasm_bindgen_test]
async fn repeated_click_replaces_loop_without_panicking() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger, canvas) =
        build_dom(&document, "confetti-root-5", "confetti-canvas-5", true);
    let _guard = RemoveOnDrop(root.clone());

    wire_confetti(root.clone()).expect("wire_confetti must not fail");
    // 連続クリックで `ActiveLoops` 内の既存ハンドルが差し替わる経路
    // （`confetti.rs::handle_click` doc 参照）を exercise する。
    // 2 回目以降のクリックで旧 `AnimationLoop` が drop されても panic
    // しないことが検証対象。
    click(&trigger);
    click(&trigger);
    click(&trigger);

    assert_eq!(canvas.get_attribute("width").as_deref(), Some("80"));
}
