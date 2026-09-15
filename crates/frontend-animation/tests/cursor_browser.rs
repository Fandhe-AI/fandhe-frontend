//! [`CursorAnimator`]/[`write_position`]（イシュー #2542）の実ブラウザ
//! 統合テスト。
//!
//! `crates/frontend-animation/src/cursor.rs` の native テストは
//! [`CursorFollower`]（DOM 非依存の spring 演算）のみを検証済みである。
//! 本ファイルはその先、実ブラウザの `requestAnimationFrame`（[`RafDriver`]）
//! と実 DOM への CSS カスタムプロパティ書き込みを通しても同じ契約が
//! 成り立つことを固定する（`spring_via_raf_dom_browser.rs` と同方針）。

#![cfg(target_arch = "wasm32")]

use fandhe_animation::spring::SpringConfig;
use fandhe_frontend_animation::cursor::{
    write_position, CursorAnimator, CURSOR_X_PROPERTY, CURSOR_Y_PROPERTY,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `spring_via_raf_dom_browser.rs::sleep_ms` と同型の決定的な待機。
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

fn build_div() -> web_sys::HtmlElement {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&div)
        .expect("append_child must not fail for a detached div");
    div
}

fn read_property(element: &web_sys::HtmlElement, name: &str) -> f64 {
    element
        .style()
        .get_property_value(name)
        .expect("get_property_value must not fail")
        .trim_end_matches("px")
        .parse()
        .expect("written value must be a valid f64 px string")
}

#[wasm_bindgen_test]
fn write_position_sets_both_properties_with_px_unit() {
    let div = build_div();
    write_position(&div, 12.5, -3.0);
    assert_eq!(
        div.style().get_property_value(CURSOR_X_PROPERTY).unwrap(),
        "12.5px"
    );
    assert_eq!(
        div.style().get_property_value(CURSOR_Y_PROPERTY).unwrap(),
        "-3px"
    );
    div.remove();
}

#[wasm_bindgen_test]
async fn move_to_settles_to_target_via_raf() {
    let div = build_div();
    let mut animator = CursorAnimator::new(div.clone(), SpringConfig::default(), false);
    animator.move_to(100.0, 40.0);

    // `Spring::settle_duration()` は入力次第だが SpringConfig::default() の
    // 典型的な収束時間は 1 秒未満である。余裕を持って 2 秒待つ。
    sleep_ms(2000).await;

    let x = read_property(&div, CURSOR_X_PROPERTY);
    let y = read_property(&div, CURSOR_Y_PROPERTY);
    assert!((x - 100.0).abs() < 0.5, "x should settle near target: {x}");
    assert!((y - 40.0).abs() < 0.5, "y should settle near target: {y}");

    div.remove();
}

#[wasm_bindgen_test]
async fn move_to_after_settling_reconverges_to_new_target() {
    let div = build_div();
    let mut animator = CursorAnimator::new(div.clone(), SpringConfig::default(), false);
    animator.move_to(10.0, 10.0);
    sleep_ms(2000).await;

    // 収束後にループが自動停止していても、再度 move_to すればループが
    // 再起動し新しい目標へ収束することを固定する（`running` フラグに
    // 依存した再起動判定の回帰テスト）。
    animator.move_to(-50.0, 200.0);
    sleep_ms(2000).await;

    let x = read_property(&div, CURSOR_X_PROPERTY);
    let y = read_property(&div, CURSOR_Y_PROPERTY);
    assert!((x - (-50.0)).abs() < 0.5, "x should reconverge: {x}");
    assert!((y - 200.0).abs() < 0.5, "y should reconverge: {y}");

    div.remove();
}

#[wasm_bindgen_test]
fn first_move_to_snaps_to_target_without_animating_from_origin() {
    let div = build_div();
    let mut animator = CursorAnimator::new(div.clone(), SpringConfig::default(), false);

    // 構築直後の最初の `move_to` は spring を経由せず即座にスナップする
    // ため、`sleep`/rAF を待たず呼び出し直後に目標値が反映されているはず
    // （原点 `(0.0, 0.0)` からの「飛び出し」がないことの回帰テスト、
    // イシュー #2542 レビュー指摘）。
    animator.move_to(300.0, 150.0);

    let x = read_property(&div, CURSOR_X_PROPERTY);
    let y = read_property(&div, CURSOR_Y_PROPERTY);
    assert_eq!(x, 300.0);
    assert_eq!(y, 150.0);

    div.remove();
}

#[wasm_bindgen_test]
fn reduced_motion_writes_immediately_without_raf() {
    let div = build_div();
    let mut animator = CursorAnimator::new(div.clone(), SpringConfig::default(), true);
    animator.move_to(77.0, -12.0);

    // reduced=true は spring を経由しないため、次の rAF を待たず
    // 呼び出し直後に反映される。
    let x = read_property(&div, CURSOR_X_PROPERTY);
    let y = read_property(&div, CURSOR_Y_PROPERTY);
    assert_eq!(x, 77.0);
    assert_eq!(y, -12.0);

    div.remove();
}
