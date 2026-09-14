//! spring（`fandhe_animation::spring::Spring`）を [`RafDriver`]+[`DomTarget`]
//! で実ブラウザ上で駆動し、収束することを確認する統合デモ
//! （イシュー #2403/#2517 受け入れ条件 4）。
//!
//! `crates/animation/src/driver.rs` の native テスト
//! （`spring_driven_by_manual_driver_writes_monotone_settle`）は `ManualDriver`
//! （決定的な delta 列）で同じ結線を検証済みである。本テストはその先、
//! 実ブラウザの `requestAnimationFrame`/`performance.now()`（[`RafDriver`]）と
//! 実 DOM の `style`（[`DomTarget`]）を使っても同じ契約が成り立つことを
//! 固定する。

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_animation::driver::Driver;
use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_animation::target::Target;
use fandhe_frontend_animation::dom_target::DomTarget;
use fandhe_frontend_animation::raf_driver::{AnimationLoop, RafDriver};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `tooltip_delay_browser.rs::sleep_ms` と同じ意図: 実タイマーを
/// `Promise` 化して `await` する決定的な待機。
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

#[wasm_bindgen_test]
async fn spring_driven_by_raf_driver_and_dom_target_settles_to_to_value() {
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

    const FROM: f64 = 0.0;
    const TO: f64 = 1.0;
    let spring = Spring::new(SpringConfig::default(), FROM, TO, 0.0)
        .expect("有効な spring パラメータのため None にならない");
    let settle_ms = (spring.settle_duration() * 1000.0).ceil() as i32;

    let mut driver =
        RafDriver::new().expect("RafDriver::new must succeed in a browser environment");
    let mut target = DomTarget::style_property(div.clone(), "opacity", "");
    let elapsed = Rc::new(RefCell::new(0.0_f64));

    let elapsed_for_step = elapsed.clone();
    let anim_loop = AnimationLoop::start(move || {
        if let Some(dt) = driver.tick() {
            let mut t = elapsed_for_step.borrow_mut();
            *t += dt;
            target.write(spring.at(*t).value);
        }
        true
    });

    // settle_duration 経過まで実時間で待つ（tick 数固定ではなく実フレーム
    // レート非依存の判定、実装計画の検証観点どおり）。余裕を持たせるため
    // 2 倍 + 固定マージンで待機する。
    sleep_ms(settle_ms * 2 + 200).await;
    anim_loop.stop();

    let opacity: f64 = div
        .style()
        .get_property_value("opacity")
        .expect("get_property_value must not fail")
        .parse()
        .expect("written opacity must be a valid f64 string");
    assert!(
        (opacity - TO).abs() < 0.01,
        "spring が to 値へ収束していない: opacity={opacity}"
    );

    div.remove();
}
