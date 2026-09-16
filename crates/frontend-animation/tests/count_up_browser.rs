//! `fandhe_frontend_animation::count_up`（イシュー #2539）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/count_up.rs`）は
//! [`NumberText::parse`]/[`NumberText::render`] の純粋計算のみを検証済み
//! である。本ファイルは [`start`] が実 DOM の `HtmlElement` の
//! `textContent` を実際に 0 から目標値へ補間しながら書き込み、
//! [`write_final`] が即時に最終値を書き込むことを検証する（いずれも
//! [`value_text_node`] で解決した数値テキストノードの `data` のみを対象に
//! し、兄弟の子要素には触れない）
//! （`crates/frontend-animation/tests/spring_via_raf_dom_browser.rs` と
//! 同方針）。

#![cfg(target_arch = "wasm32")]

use std::cell::Cell;
use std::rc::Rc;

use fandhe_frontend_animation::count_up::{start, value_text_node, write_final, NumberText};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, Text};

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

/// `text` を唯一の子テキストノードに持つ `div` と、その数値テキスト
/// ノード（[`value_text_node`] の解決結果）を返す。
fn create_div_with_text(text: &str) -> (HtmlElement, Text, RemoveOnDrop) {
    let (element, guard) = create_div();
    element.set_text_content(Some(text));
    let node = value_text_node(&element).expect("number text node must resolve");
    (element, node, guard)
}

#[wasm_bindgen_test]
async fn start_interpolates_from_zero_to_final_value() {
    let (element, node, _guard) = create_div_with_text("100");
    let format = NumberText::parse("100").expect("\"100\" must parse");
    let last_value = Rc::new(Cell::new(None));
    let self_write_count = Rc::new(Cell::new(0u32));

    let _handle = start(
        node,
        format,
        0.0,
        100.0,
        80.0,
        Rc::clone(&last_value),
        Rc::clone(&self_write_count),
    );

    // `start` は rAF を待たず同期的に開始値を書き込む契約（SSR 最終値の
    // ちらつき対策）。タイマー待ちで途中フレームを覗く判定は負荷次第で
    // 補間が先に終わり得るため（PR #2580 Bugbot Low 指摘）、同期的な初期
    // 書き込みだけを決定的に検証する。
    assert_eq!(element.text_content().unwrap(), "0");
    assert_eq!(last_value.get(), Some(0.0));
    assert_eq!(self_write_count.get(), 1);

    // duration を十分に超えて待てば最終値へ収束する。
    sleep_ms(300).await;
    assert_eq!(element.text_content().unwrap(), "100");
    assert_eq!(last_value.get(), Some(100.0));
}

#[wasm_bindgen_test]
fn write_final_writes_formatted_value_immediately_and_updates_last_value() {
    let (element, node, _guard) = create_div_with_text("$0.00");
    let format = NumberText::parse("$0.00").expect("\"$0.00\" must parse");
    let last_value = Rc::new(Cell::new(None));
    let self_write_count = Rc::new(Cell::new(0u32));

    write_final(&node, &format, 1234.5, &last_value, &self_write_count);

    assert_eq!(element.text_content().unwrap(), "$1,234.50");
    assert_eq!(last_value.get(), Some(1234.5));
    assert_eq!(self_write_count.get(), 1);
}

/// PR #2580 codex-review P1 指摘の回帰テスト: `stat::value_text` の公開
/// 契約どおり数値テキストの兄弟に `value_unit`（単位 `<span>`）・
/// `up_indicator`（`<span aria-hidden>`）が並ぶ構成で、書き込みは数値
/// テキストノードだけを書き換え、子要素を削除しないこと。
#[wasm_bindgen_test]
async fn writes_only_number_text_node_and_preserves_sibling_children() {
    let (element, _guard) = create_div();
    let document = web_sys::window().unwrap().document().unwrap();
    element
        .append_child(&document.create_text_node("1,234"))
        .unwrap();
    let unit = document.create_element("span").unwrap();
    unit.set_text_content(Some("%"));
    element.append_child(&unit).unwrap();
    let arrow = document.create_element("span").unwrap();
    arrow.set_attribute("aria-hidden", "true").unwrap();
    arrow.set_text_content(Some("▲"));
    element.append_child(&arrow).unwrap();

    let format = NumberText::parse("1,234").expect("\"1,234\" must parse");
    let node = value_text_node(&element).expect("number text node must resolve");
    let last_value = Rc::new(Cell::new(None));
    let self_write_count = Rc::new(Cell::new(0u32));

    write_final(&node, &format, 500.0, &last_value, &self_write_count);
    assert_eq!(element.child_element_count(), 2, "子要素が保持されること");
    assert_eq!(element.text_content().unwrap(), "500%▲");
    assert_eq!(
        value_text_node(&element).map(|node| node.data()),
        Some("500".to_string())
    );

    let _handle = start(
        node,
        format,
        0.0,
        1234.0,
        80.0,
        last_value,
        self_write_count,
    );
    assert_eq!(
        element.text_content().unwrap(),
        "0%▲",
        "同期初期書き込みも子要素を保持"
    );
    sleep_ms(300).await;
    assert_eq!(
        element.child_element_count(),
        2,
        "補間完了後も子要素が保持されること"
    );
    assert_eq!(element.text_content().unwrap(), "1,234%▲");
}

/// PR #2580 codex-review P1 指摘の回帰テスト: 再補間の開始値は表示文字列
/// の再解析ではなく直近に書き込んだ数値（`last_value`）から得ること。
/// 桁区切り `.` 書式（"1.234.567"）の途中値 123456 は "123.456" と表示され、
/// 再解析すると小数 123.456 へ誤解釈される（1/1000 への急落）。
#[wasm_bindgen_test]
fn last_value_keeps_numeric_value_where_reparsing_display_would_misread_it() {
    let (element, node, _guard) = create_div_with_text("1.234.567");
    let format = NumberText::parse("1.234.567").expect("\"1.234.567\" must parse");
    let last_value = Rc::new(Cell::new(None));
    let self_write_count = Rc::new(Cell::new(0u32));

    write_final(&node, &format, 123456.0, &last_value, &self_write_count);

    let shown = element.text_content().unwrap();
    assert_eq!(shown, "123.456");
    let reparsed = NumberText::parse(&shown).map(|n| n.value());
    assert_eq!(
        reparsed,
        Some(123.456),
        "表示の再解析は小数へ誤読する（前提の確認）"
    );
    assert_eq!(
        last_value.get(),
        Some(123456.0),
        "再補間の開始値は数値のまま保持されること"
    );
}
