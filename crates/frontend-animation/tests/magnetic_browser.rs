//! `fandhe_frontend_animation::magnetic`（イシュー #2550）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/magnetic.rs`）は
//! [`compute_pull`] の純粋計算のみを検証済みである。本ファイルは
//! [`write_offset`] が実 DOM の `HtmlElement` へ 2 つの CSS カスタム
//! プロパティ（`--fandhe-motion-magnetic-x`/`-y`）を実際に書き込むことを
//! 検証する（`crates/frontend-animation/tests/scroll_driver_browser.rs` と
//! 同方針、DOM 非依存の計算ロジック自体は本ファイルの対象外）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::magnetic::{rendered_offset, write_offset};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード。
struct RemoveOnDrop(web_sys::Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

#[wasm_bindgen_test]
fn write_offset_sets_both_custom_properties_with_px_unit() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");

    write_offset(&html_element, 4.5, -2.0);

    let style = html_element.style();
    assert_eq!(
        style
            .get_property_value("--fandhe-motion-magnetic-x")
            .unwrap(),
        "4.5px"
    );
    assert_eq!(
        style
            .get_property_value("--fandhe-motion-magnetic-y")
            .unwrap(),
        "-2px"
    );
}

#[wasm_bindgen_test]
fn write_offset_overwrites_previous_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");

    write_offset(&html_element, 10.0, 10.0);
    write_offset(&html_element, 0.0, 0.0);

    let style = html_element.style();
    assert_eq!(
        style
            .get_property_value("--fandhe-motion-magnetic-x")
            .unwrap(),
        "0px"
    );
    assert_eq!(
        style
            .get_property_value("--fandhe-motion-magnetic-y")
            .unwrap(),
        "0px"
    );
}

/// magnetic 由来の `translate` のみを消費する `transform`（`cta_banner_magnetic`
/// 実装が実際に適用する CSS と同型、`transition` なし）を設定した要素へ
/// `write_offset` を適用すると、`transition` が無いため即座に描画へ反映
/// され、[`rendered_offset`] が [`write_offset`] の値をそのまま返す
/// （codex-review P1 指摘の是正、イシュー #2550: 「実際の描画上の移動量」
/// を読む契約の回帰固定。`transition` 途中の値は本テストの対象外——実
/// ブラウザで transition の中間フレームを決定的に捕捉する手段がないため、
/// ここでは「transition が無い場合は即時反映される」という前提のみを
/// 固定し、途中値の解析自体は native テスト
/// `parse_transform_translation_*` が担う）。
#[wasm_bindgen_test]
fn rendered_offset_reflects_write_offset_without_transition() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");
    html_element
        .style()
        .set_property(
            "transform",
            "translate(var(--fandhe-motion-magnetic-x, 0px), var(--fandhe-motion-magnetic-y, 0px))",
        )
        .unwrap();

    write_offset(&html_element, 7.0, -3.0);
    let (x, y) = rendered_offset(&html_element);
    assert!((x - 7.0).abs() < 1e-6, "x should reflect write_offset: {x}");
    assert!(
        (y - -3.0).abs() < 1e-6,
        "y should reflect write_offset: {y}"
    );

    write_offset(&html_element, 0.0, 0.0);
    let (x, y) = rendered_offset(&html_element);
    assert!((x).abs() < 1e-6, "x should reset to 0: {x}");
    assert!((y).abs() < 1e-6, "y should reset to 0: {y}");
}

/// `transform` を一切適用していない要素（≒ magnetic opt-in の CSS 契約を
/// 満たさない未整備の要素）に対しては `rendered_offset` が `(0.0, 0.0)` へ
/// fail-safe すること（新規要素へ最初に進入した際にパニック・NaN 伝播し
/// ないことの回帰固定）。
#[wasm_bindgen_test]
fn rendered_offset_defaults_to_zero_when_no_transform_applied() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");

    assert_eq!(rendered_offset(&html_element), (0.0, 0.0));
}
