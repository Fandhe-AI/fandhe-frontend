//! `fandhe_frontend_wasm_full::svg_path::wire_svg_path`（イシュー #2519、
//! 親 #2508）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! 全長計算・`stroke-dasharray`/`stroke-dashoffset` の計算・WAAPI 呼び出し
//! 自体は責務どおり `crates/frontend-animation/tests/svg_path_browser.rs`
//! が検証済みである。本ファイルは `wire_svg_path` が `data-*` 属性の
//! 走査・opt-in 判定のみを担うことを検証する（`in_view_browser.rs` と
//! 同型の DOM 手組み + 直接呼び出し方針、`Runtime::mount` は経由しない）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "svg-path")]

use fandhe_frontend_wasm_full::svg_path::{wire_svg_path, SVG_PATH_DRAW_ATTR};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// テスト末尾で DOM を確実に除去する RAII ガード（`confetti_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn get_computed_stroke_dashoffset(element: &Element) -> String {
    let window = web_sys::window().expect("window must exist");
    let style = window
        .get_computed_style(element)
        .expect("getComputedStyle must not fail")
        .expect("getComputedStyle must return a style declaration");
    style
        .get_property_value("stroke-dashoffset")
        .expect("get_property_value must not fail")
}

#[wasm_bindgen_test]
fn wire_svg_path_applies_initial_dash_style_to_marked_path() {
    let document: Document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("svg-path-root-1");

    let svg = document.create_element_ns(Some(SVG_NS), "svg").unwrap();
    let path = document.create_element_ns(Some(SVG_NS), "path").unwrap();
    path.set_attribute("d", "M0,0 L100,0").unwrap();
    path.set_attribute(SVG_PATH_DRAW_ATTR, "").unwrap();
    svg.append_child(&path).unwrap();
    root.append_child(&svg).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    wire_svg_path(&root).expect("wire_svg_path must not fail");

    let offset = get_computed_stroke_dashoffset(&path);
    assert!(
        offset.ends_with("px") && offset != "0px",
        "expected non-zero initial stroke-dashoffset immediately after wiring, got {offset}"
    );
}

#[wasm_bindgen_test]
fn wire_svg_path_ignores_elements_without_marker_attribute() {
    let document: Document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("svg-path-root-2");

    let svg = document.create_element_ns(Some(SVG_NS), "svg").unwrap();
    let path = document.create_element_ns(Some(SVG_NS), "path").unwrap();
    path.set_attribute("d", "M0,0 L100,0").unwrap();
    // マーカー属性を付けない。
    svg.append_child(&path).unwrap();
    root.append_child(&svg).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    wire_svg_path(&root).expect("wire_svg_path must not fail");

    // `stroke-dashoffset` の既定計算値は `"0px"`（未設定の初期値）である
    // ため、書き込み後の値（100px 前後の非ゼロ値）と区別して既定値のまま
    // であることを確認する（マーカーを持たない path は書き換え対象に
    // ならないはず）。
    let offset = get_computed_stroke_dashoffset(&path);
    assert_eq!(
        offset, "0px",
        "expected default stroke-dashoffset, got {offset}"
    );
}

#[wasm_bindgen_test]
fn wire_svg_path_does_not_panic_for_marked_non_geometry_element() {
    let document: Document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("svg-path-root-3");

    let div = document.create_element("div").unwrap();
    div.set_attribute(SVG_PATH_DRAW_ATTR, "").unwrap();
    root.append_child(&div).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    // 対象外要素（SVGGeometryElement でない）へのマーカー付与は panic せず
    // 静かに無視されることそのものが検証対象。
    wire_svg_path(&root).expect("wire_svg_path must not fail");
}
