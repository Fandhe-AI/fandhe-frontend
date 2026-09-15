//! `fandhe_frontend_animation::svg_path`（イシュー #2519、親 #2508）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/svg_path.rs` の
//! `initial_dash_values`/`default_options` 検証）は DOM 非依存の純粋関数を
//! 検証済みである。本ファイルは以下を実ブラウザ（headless Chromium）上で
//! 検証する。
//!
//! 1. [`total_length`] が実 `<path>` の `getTotalLength()` と一致すること
//! 2. [`draw_path_with_reduced_motion`] 呼び出し後、初期
//!    `stroke-dasharray`/`stroke-dashoffset` が書き込まれ、`finished()` 待機後
//!    に `stroke-dashoffset` が `0px` になること（`fill: forwards` の効果）
//! 3. `reduced_motion: true` では `Ok(None)` を返し DOM を一切変更しないこと
//! 4. `<path>` でない要素（`<div>`）では安全に `Err`/`Ok(None)` になること

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::svg_path::{
    default_options, draw_path_with_reduced_motion, total_length,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// 水平線分 1 本の `<path>`（`d="M0,0 L100,0"`、全長 100）を `<svg>` へ
/// 追加して `document.body` へ append する。全長が既知の単純パスにする
/// ことで `getTotalLength()` の期待値を固定できる。
fn append_line_path() -> web_sys::Element {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let svg = document
        .create_element_ns(Some(SVG_NS), "svg")
        .expect("create_element_ns must not fail for svg");
    let path = document
        .create_element_ns(Some(SVG_NS), "path")
        .expect("create_element_ns must not fail for path");
    path.set_attribute("d", "M0,0 L100,0")
        .expect("set_attribute must not fail for d");
    svg.append_child(&path)
        .expect("append_child must not fail for path into svg");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&svg)
        .expect("append_child must not fail for svg into body");
    path
}

fn stroke_dashoffset_of(element: &web_sys::Element) -> String {
    let window = web_sys::window().expect("window must exist");
    let style = window
        .get_computed_style(element)
        .expect("getComputedStyle must not fail")
        .expect("getComputedStyle must return a style declaration");
    style
        .get_property_value("stroke-dashoffset")
        .expect("get_property_value must not fail for stroke-dashoffset")
}

#[wasm_bindgen_test]
fn total_length_matches_get_total_length() {
    let path = append_line_path();
    let length = total_length(&path).expect("total_length must succeed for a path element");
    assert!(
        (length - 100.0).abs() < 0.5,
        "expected total_length close to 100.0, got {length}"
    );
}

#[wasm_bindgen_test]
fn total_length_fails_for_non_geometry_element() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for div");
    assert!(total_length(&div).is_err());
}

#[wasm_bindgen_test]
async fn draw_path_with_reduced_motion_false_animates_to_zero_offset() {
    let path = append_line_path();
    let options = default_options();

    let handle = draw_path_with_reduced_motion(&path, &options, false)
        .expect("draw_path_with_reduced_motion must not error")
        .expect("draw_path_with_reduced_motion must return a handle when not reduced");

    // 初期値は全長（100px 前後）が書き込まれているはず。
    let initial = stroke_dashoffset_of(&path);
    assert!(
        initial.ends_with("px") && initial != "0px",
        "expected non-zero initial stroke-dashoffset, got {initial}"
    );

    handle.finished().await.expect("finished() must not reject");

    let after = stroke_dashoffset_of(&path);
    assert_eq!(
        after, "0px",
        "expected stroke-dashoffset to reach 0px after fill: forwards animation"
    );
}

#[wasm_bindgen_test]
fn draw_path_with_reduced_motion_true_returns_none_and_does_not_touch_dom() {
    let path = append_line_path();
    let options = default_options();

    let handle = draw_path_with_reduced_motion(&path, &options, true)
        .expect("draw_path_with_reduced_motion must not error even when reduced");
    assert!(handle.is_none());

    let style = path
        .dyn_ref::<web_sys::SvgElement>()
        .expect("path must be an SvgElement")
        .style();
    assert_eq!(
        style
            .get_property_value("stroke-dashoffset")
            .expect("get_property_value must not fail"),
        "",
        "reduced motion must not write any style"
    );
}

#[wasm_bindgen_test]
fn draw_path_with_reduced_motion_returns_none_for_non_geometry_element() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let div: web_sys::Element = document
        .create_element("div")
        .expect("create_element must not fail for div");
    let options = default_options();

    let handle = draw_path_with_reduced_motion(&div, &options, false)
        .expect("draw_path_with_reduced_motion must not error for a non-geometry element");
    assert!(handle.is_none());
}
