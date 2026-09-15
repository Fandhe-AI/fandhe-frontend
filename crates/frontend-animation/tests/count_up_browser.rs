//! `fandhe_frontend_animation::count_up`（イシュー #2539）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/count_up.rs`）は
//! [`NumberText::parse`]/[`NumberText::render`] の純粋計算のみを検証済み
//! である。本ファイルは [`start`] が実 DOM の `HtmlElement` の
//! `textContent` を実際に 0 から目標値へ補間しながら書き込み、
//! [`write_final`] が即時に最終値を書き込むことを検証する
//! （`crates/frontend-animation/tests/spring_via_raf_dom_browser.rs` と
//! 同方針）。

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_animation::count_up::{start, write_final, NumberText};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード
/// （`magnetic_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(web_sys::Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `spring_via_raf_dom_browser.rs::sleep_ms` と同じ意図: 実タイマーを
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

fn create_div() -> (HtmlElement, RemoveOnDrop) {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let element = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    document
        .body()
        .expect("document body must exist")
        .append_child(&element)
        .expect("append_child must not fail");
    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");
    (html_element, RemoveOnDrop(element))
}

#[wasm_bindgen_test]
async fn start_interpolates_from_zero_to_final_value() {
    let (element, _guard) = create_div();
    let format = NumberText::parse("100").expect("\"100\" must parse");
    let last_written = Rc::new(RefCell::new(String::new()));

    let _handle = start(element.clone(), format, 0.0, 100.0, 80.0, last_written);

    // 開始直後（1 フレーム目付近）は目標値未満のはず。
    sleep_ms(16).await;
    let mid = element
        .text_content()
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap_or(f64::NAN);
    assert!(mid.is_finite(), "途中フレームは数値であること: {mid}");
    assert!(mid < 100.0, "途中フレームは目標値未満であること: {mid}");

    // duration を十分に超えて待てば最終値へ収束する。
    sleep_ms(300).await;
    assert_eq!(element.text_content().unwrap(), "100");
}

#[wasm_bindgen_test]
fn write_final_writes_formatted_value_immediately_and_updates_last_written() {
    let (element, _guard) = create_div();
    let format = NumberText::parse("$0.00").expect("\"$0.00\" must parse");
    let last_written = Rc::new(RefCell::new(String::new()));

    write_final(&element, &format, 1234.5, &last_written);

    assert_eq!(element.text_content().unwrap(), "$1,234.50");
    assert_eq!(*last_written.borrow(), "$1,234.50");
}
