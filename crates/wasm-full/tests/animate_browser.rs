//! `fandhe_frontend_animation::animate`（`element.animate()` WAAPI 薄いラッパ、
//! イシュー #2398）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `fandhe-frontend-animation` は wasm クレートでも native `cargo test` へ
//! 持ち込まない 2 層構成（`crates/wasm-full/src/stagger_index.rs`・
//! `content_height.rs` と同型）を採るため、pure 層（`easing_to_css`・
//! `keyframes_to_waapi`）は `crates/frontend-animation/src/animate.rs` 内の
//! native `#[cfg(test)]` で検証済みであり、本ファイルは wasm32 層
//! （`animate`/`AnimationHandle::finished`）を `fandhe-frontend-wasm-full`
//! （既に cdylib+rlib・`wasm-bindgen-futures`〔dev-dependency〕・`browser-test`
//! ジョブ枠を持つ）から呼ぶ実ブラウザ経路で検証する（`frontend-animation` を
//! 新規に cdylib 化するより最小の diff、実装計画参照）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::animate::{animate, AnimateOptions, WaapiKeyframe};
use wasm_bindgen_test::*;
use web_sys::Document;

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用の detached でない（`getComputedStyle` が有効値を返すよう
/// document に接続した）div を 1 個生成する
/// （`tooltip_delay_browser.rs::create_placeholder` と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> web_sys::Element {
    let element = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    element.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&element)
        .expect("append_child must not fail for a detached div");
    element
}

/// テスト末尾で要素を document から確実に除去する RAII ガード
/// （`tooltip_delay_browser.rs::RemoveOnDrop` と同じ意図）。
struct RemoveOnDrop(web_sys::Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

#[wasm_bindgen_test]
async fn animate_finished_resolves_and_applies_forwards_fill() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window
        .document()
        .expect("document must exist in browser test environment");
    let element = create_placeholder(&document, "animate-browser-target");
    let _guard = RemoveOnDrop(element.clone());

    let frames = vec![
        WaapiKeyframe {
            offset: 0.0,
            easing: Some("linear".to_string()),
            properties: vec![("opacity".to_string(), "0".to_string())],
        },
        WaapiKeyframe {
            offset: 1.0,
            easing: None,
            properties: vec![("opacity".to_string(), "1".to_string())],
        },
    ];
    let options = AnimateOptions {
        // CI 実行時間を節約するため短く保つ（他ブラウザテストと同じ方針）。
        duration_ms: 20.0,
        easing: None,
        fill: Some("forwards".to_string()),
        iterations: None,
    };

    let handle = animate(&element, &frames, &options)
        .expect("element.animate() must not throw for a valid keyframes/options pair");
    handle
        .finished()
        .await
        .expect("finished Promise must resolve for a normal (non-cancelled) animation");

    let computed = window
        .get_computed_style(&element)
        .expect("getComputedStyle must not throw")
        .expect("getComputedStyle must return a value for an attached element");
    let opacity = computed
        .get_property_value("opacity")
        .expect("get_property_value must not throw for a known CSS property");
    assert_eq!(
        opacity, "1",
        "fill: \"forwards\" の効果で終了状態（opacity: 1）が維持されているはず"
    );
}
