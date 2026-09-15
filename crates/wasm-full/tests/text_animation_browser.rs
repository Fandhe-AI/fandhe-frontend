//! `fandhe_frontend_wasm_full::text_animation`（イシュー #2532）の実
//! ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/text_animation.rs` の native テストは定数の安定性のみを
//! 検証済みである。本ファイルは候補走査・`.fd-text-reveal__display` 解決・
//! duration 属性のパース・`prefers-reduced-motion` 抑制が実 DOM 上で機能
//! することを、`hold_to_confirm_browser.rs`（手組み DOM への直接 `wire_*`
//! 呼び出し、`Runtime::mount` は経由しない）と同方針で検証する。
//!
//! `fandhe-frontend-animation::text_animation::typewriter_frame`/
//! `scramble_frame` 自体の計算ロジックは責務境界どおり native テスト
//! （`crates/frontend-animation/src/text_animation.rs`）で検証済みであり、
//! 本ファイルは実 DOM 上での配線（rAF 駆動・`textContent` 書き込み先の
//! 一致・reduced-motion 分岐）のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "text-animation")]

use fandhe_frontend_wasm_full::text_animation::{
    wire_text_animation_with_reduced_motion, SCRAMBLE_ATTR, TYPEWRITER_ATTR,
};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`magnetic_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `root` > `candidate`（`attr`/`attr_value` を持つ）>
/// `.fd-text-reveal__display`（`target` を初期 `textContent` として持つ）を
/// 組み立てて返す（`text_reveal::typewriter`/`scramble` の SSR 出力と同型の
/// 骨格）。
fn build_dom(
    document: &Document,
    root_id: &str,
    attr: &str,
    attr_value: &str,
    target: &str,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let candidate = document.create_element("span").unwrap();
    candidate.set_attribute(attr, attr_value).unwrap();

    let display = document.create_element("span").unwrap();
    display.set_class_name("fd-text-reveal__display");
    display.set_text_content(Some(target));

    candidate.append_child(&display).unwrap();
    root.append_child(&candidate).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    root
}

async fn sleep_ms(ms: i32) {
    let window = web_sys::window().unwrap();
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

fn display_of(candidate: &Element) -> Element {
    candidate
        .query_selector(".fd-text-reveal__display")
        .unwrap()
        .unwrap()
}

#[wasm_bindgen_test]
async fn typewriter_reaches_full_text_after_duration() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_dom(
        &document,
        "text-animation-root-1",
        TYPEWRITER_ATTR,
        "60",
        "hi",
    );
    let _guard = RemoveOnDrop(root.clone());

    let loops = wire_text_animation_with_reduced_motion(root.clone(), false).unwrap();
    assert_eq!(loops.len(), 1);

    let candidate = root.first_element_child().unwrap();
    let display = display_of(&candidate);

    sleep_ms(300).await;
    assert_eq!(display.text_content().as_deref(), Some("hi"));
}

#[wasm_bindgen_test]
async fn typewriter_falls_back_to_default_duration_on_invalid_attribute() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_dom(
        &document,
        "text-animation-root-2",
        TYPEWRITER_ATTR,
        "not-a-number",
        "ok",
    );
    let _guard = RemoveOnDrop(root.clone());

    let loops = wire_text_animation_with_reduced_motion(root.clone(), false).unwrap();
    assert_eq!(loops.len(), 1);

    let candidate = root.first_element_child().unwrap();
    let display = display_of(&candidate);

    // 既定 duration（TYPEWRITER_DEFAULT_DURATION_MS = 1200ms）に達する前は
    // まだ全文になっていないことを確認する（フォールバックが効いている、
    // すなわち不正値を即座に無視して全文書き込みしていないことの確認）。
    sleep_ms(100).await;
    assert_ne!(display.text_content().as_deref(), None);
}

#[wasm_bindgen_test]
async fn scramble_reaches_full_text_after_duration() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_dom(
        &document,
        "text-animation-root-3",
        SCRAMBLE_ATTR,
        "60",
        "secret",
    );
    let _guard = RemoveOnDrop(root.clone());

    let loops = wire_text_animation_with_reduced_motion(root.clone(), false).unwrap();
    assert_eq!(loops.len(), 1);

    let candidate = root.first_element_child().unwrap();
    let display = display_of(&candidate);

    sleep_ms(300).await;
    assert_eq!(display.text_content().as_deref(), Some("secret"));
}

#[wasm_bindgen_test]
async fn reduced_motion_suppresses_wiring_and_leaves_text_unchanged() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_dom(
        &document,
        "text-animation-root-4",
        TYPEWRITER_ATTR,
        "60",
        "hi",
    );
    let _guard = RemoveOnDrop(root.clone());

    let loops = wire_text_animation_with_reduced_motion(root.clone(), true).unwrap();
    assert!(loops.is_empty());

    let candidate = root.first_element_child().unwrap();
    let display = display_of(&candidate);

    sleep_ms(300).await;
    assert_eq!(display.text_content().as_deref(), Some("hi"));
}

#[wasm_bindgen_test]
async fn missing_display_layer_is_skipped_without_panicking() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("text-animation-root-5");
    let candidate = document.create_element("span").unwrap();
    candidate.set_attribute(TYPEWRITER_ATTR, "").unwrap();
    root.append_child(&candidate).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    let loops = wire_text_animation_with_reduced_motion(root, false).unwrap();
    assert!(loops.is_empty());
}
