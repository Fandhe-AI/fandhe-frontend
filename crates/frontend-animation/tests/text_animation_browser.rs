//! `fandhe_frontend_animation::text_animation`（イシュー #2532）の実
//! ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/text_animation.rs`）は
//! [`typewriter_frame`]/[`scramble_frame`] の純粋計算のみを検証済みである。
//! 本ファイルは [`play_typewriter`]/[`play_scramble`] が実 DOM の `Element`
//! の `textContent` を rAF ループで実際に書き換え、完了時に目標テキストへ
//! 収束することを検証する（`crates/frontend-animation/tests/
//! magnetic_browser.rs` と同方針、DOM 非依存の計算ロジック自体は本ファイル
//! の対象外）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::text_animation::{play_scramble, play_typewriter};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード。
struct RemoveOnDrop(web_sys::Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

async fn sleep_ms(ms: i32) {
    let window = web_sys::window().unwrap();
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

#[wasm_bindgen_test]
async fn play_typewriter_reaches_target_text_after_duration() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("span").unwrap();
    element.set_text_content(Some("hello"));
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let animation = play_typewriter(&element, 60.0);
    assert!(animation.is_some());

    sleep_ms(300).await;
    assert_eq!(element.text_content().as_deref(), Some("hello"));
}

#[wasm_bindgen_test]
async fn play_scramble_reaches_target_text_after_duration() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("span").unwrap();
    element.set_text_content(Some("world"));
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let animation = play_scramble(&element, 60.0);
    assert!(animation.is_some());

    sleep_ms(300).await;
    assert_eq!(element.text_content().as_deref(), Some("world"));
}
