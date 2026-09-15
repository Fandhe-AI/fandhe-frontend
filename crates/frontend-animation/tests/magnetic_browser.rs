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

use fandhe_frontend_animation::magnetic::{current_offset, write_offset};
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

/// [`current_offset`] は [`write_offset`] が書き込んだ値をそのまま読み戻す
/// （Bugbot 指摘の是正、イシュー #2550: `getBoundingClientRect()` が返す
/// transform 込みの矩形から本関数の戻り値を減算することで静止位置の中心を
/// 復元できることの前提となる往復契約）。負値・0 を含む往復を固定する。
#[wasm_bindgen_test]
fn current_offset_round_trips_write_offset() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");

    write_offset(&html_element, 7.25, -3.5);
    assert_eq!(current_offset(&html_element), (7.25, -3.5));

    write_offset(&html_element, 0.0, 0.0);
    assert_eq!(current_offset(&html_element), (0.0, 0.0));
}

/// カスタムプロパティが未設定（`write_offset` 未呼び出し）の要素に対しては
/// `current_offset` が `(0.0, 0.0)` へ fail-safe すること（新規要素へ最初に
/// 進入した際にパニック・NaN 伝播しないことの回帰固定）。
#[wasm_bindgen_test]
fn current_offset_defaults_to_zero_when_unset() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    let _guard = RemoveOnDrop(element.clone());

    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("div must cast to HtmlElement");

    assert_eq!(current_offset(&html_element), (0.0, 0.0));
}
