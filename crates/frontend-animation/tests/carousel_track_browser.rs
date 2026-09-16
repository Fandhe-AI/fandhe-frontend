//! `fandhe_frontend_animation::carousel::CarouselTrack`（イシュー #2541）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/carousel.rs`）は
//! [`fandhe_frontend_animation::carousel::snap_target`] の純粋計算のみを
//! 検証済みである。本ファイルは `--fandhe-carousel-index` への実 DOM
//! 書き込みと settle コールバックの起動を検証する
//! （`crates/frontend-animation/tests/spring_via_raf_dom_browser.rs` と
//! 同方針）。

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_animation::carousel::{CarouselTrack, CAROUSEL_INDEX_PROPERTY};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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

fn make_item_group() -> web_sys::HtmlElement {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    document
        .body()
        .expect("document body must exist")
        .append_child(&div)
        .expect("append_child must not fail");
    div
}

fn read_index(element: &web_sys::HtmlElement) -> f64 {
    element
        .style()
        .get_property_value(CAROUSEL_INDEX_PROPERTY)
        .expect("get_property_value must not fail")
        .trim()
        .parse()
        .expect("written value must be a valid f64 string")
}

#[wasm_bindgen_test]
async fn drag_then_release_settles_index_and_calls_on_settle_once() {
    let element = make_item_group();
    let mut track = CarouselTrack::attach(element.clone(), 5, false);

    // 1 スライド = 100px。5px 未満の移動なら snap_target は 0 のまま
    // だが、ここでは 150px（1.5 スライド分）動かす。
    track.on_pointer_down(0.0, 0.0);
    track.on_pointer_move(-150.0, 100.0, 16.0);
    // ドラッグ中は連続値が書き込まれる（整数へスナップしていない）。
    let dragging_value = read_index(&element);
    assert!(
        (dragging_value - 1.5).abs() < 1e-9,
        "dragging value should be 1.5: {dragging_value}"
    );

    let settled = Rc::new(RefCell::new(None::<usize>));
    let settled_for_cb = settled.clone();
    // release は最後の move（t=16.0ms）から `STALE_VELOCITY_THRESHOLD_MS`
    // （100ms）超あとに呼ぶ（イシュー #2541 是正: 元は t=32.0（move から
    // わずか 16ms 後）で release していたため `is_velocity_stale` が
    // false のまま `estimate_velocity` が 150px/16ms ≈ 9375px/s という
    // 現実的だが本テストの意図〔速度の影響を受けない単純な丸め〕には
    // 大きすぎる速度を計算し、`snap_target` が 1.5 を 2 ではなく末尾
    // index 4 へ大きくオーバーシュートしていた。release を staleness
    // 閾値超あとへ遅らせることで速度が確実に 0 になり、`progress` の
    // 単純な丸めのみを検証する本来の意図どおりになる。
    let target = track.on_release(132.0, 100.0, move |index| {
        *settled_for_cb.borrow_mut() = Some(index);
    });
    assert_eq!(target, 2, "1.5 should round to nearest slide 2");

    // spring settle を実時間で待つ（`spring_via_raf_dom_browser.rs` と
    // 同型: settle_duration の実測値に依存せず十分なマージンで待つ）。
    sleep_ms(2_000).await;

    let final_value = read_index(&element);
    assert!(
        (final_value - 2.0).abs() < 0.01,
        "settled value should converge to 2.0: {final_value}"
    );
    assert_eq!(*settled.borrow(), Some(2));

    element.remove();
}

#[wasm_bindgen_test]
async fn attach_reads_existing_inline_style_as_initial_progress() {
    let element = make_item_group();
    element
        .style()
        .set_property(CAROUSEL_INDEX_PROPERTY, "3")
        .expect("set_property must not fail");
    let mut track = CarouselTrack::attach(element.clone(), 5, false);

    track.on_pointer_down(0.0, 0.0);
    track.on_pointer_move(0.0, 100.0, 16.0);
    // 移動量 0 なので進行度は attach 時点の初期値 3 のまま。
    let value = read_index(&element);
    assert!(
        (value - 3.0).abs() < 1e-9,
        "value should stay at 3.0: {value}"
    );

    element.remove();
}
