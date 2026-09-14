//! `view_transition_name::set_view_transition_name`（イシュー #2515）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! 本 API は `Runtime` を経由しないアプリ直接利用 API（`stagger_index`
//! 等と異なり keyed list 統合を要しない）のため、素の `HtmlElement` に
//! 対して直接呼ぶ最小テストで足りる。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::view_transition_name::set_view_transition_name;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

fn create_placeholder(document: &Document) -> Element {
    let el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&el)
        .expect("append_child must not fail for a detached div");
    el
}

struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

#[wasm_bindgen_test]
fn valid_name_is_written_to_inline_style() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist in browser test environment");
    let guard = RemoveOnDrop(create_placeholder(&document));
    let element: &HtmlElement = guard.0.dyn_ref().expect("div must cast to HtmlElement");

    assert!(set_view_transition_name(element, "row-42"));
    assert_eq!(
        element
            .style()
            .get_property_value("view-transition-name")
            .expect("get_property_value must not fail for a set property"),
        "row-42"
    );
}

#[wasm_bindgen_test]
fn invalid_name_is_rejected_without_overwriting_existing_value() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist in browser test environment");
    let guard = RemoveOnDrop(create_placeholder(&document));
    let element: &HtmlElement = guard.0.dyn_ref().expect("div must cast to HtmlElement");

    assert!(set_view_transition_name(element, "logo"));

    // 注入試行文字列（`;`/`}` を含む）はいずれも拒否され、既存値が保持される。
    for attempt in ["", "Logo", "none", "a;color:red", "a}body{color:red"] {
        assert!(!set_view_transition_name(element, attempt));
    }
    assert_eq!(
        element
            .style()
            .get_property_value("view-transition-name")
            .expect("get_property_value must not fail for a set property"),
        "logo"
    );
}
