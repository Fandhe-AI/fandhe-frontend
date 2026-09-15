//! `fandhe_frontend_wasm_full::hold_to_confirm::wire_hold_to_confirm`
//! （イシュー #2538）の実ブラウザ統合テスト（`wasm-pack test --headless
//! --chrome`）。
//!
//! `wasm-full/src/hold_to_confirm.rs` の native テストは純粋関数
//! （`hold_progress`/`parse_hold_duration_ms`）のみを検証済みである。
//! 本ファイルは以下を実 DOM 上で検証する（`gesture_browser.rs`/
//! `confetti_browser.rs` と同方針で、手組み DOM への直接
//! `wire_hold_to_confirm` 呼び出し、`Runtime::mount` は経由しない）:
//!
//! (a) 十分な時間 pointerdown を保持し続けると `data-state="confirmed"`
//!     になり、合成 click が 1 回だけ発火する（`data-action` 委譲相当の
//!     観測として `click` イベント自体をリスナーで数える）。
//! (b) 目標時間より早く pointerup すると `data-state` は `confirmed` に
//!     ならず、click も発火しない。
//! (c) 短い（保持なしの）pointerdown → pointerup は click を発火しない
//!     （ネイティブ `<button>` の即時 click 相当の誤操作防止）。
//! (d) キーボード（Enter）保持でも (a) と同様に確定する。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "hold-to-confirm")]

use fandhe_frontend_wasm_full::hold_to_confirm::{
    wire_hold_to_confirm, DEFAULT_CONFIRMED_RESET_TIMEOUT_MS, HOLD_DURATION_MS_ATTR,
    HOLD_PROGRESS_VAR, HOLD_TO_CONFIRM_ATTR,
};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, HtmlElement, KeyboardEvent, KeyboardEventInit, PointerEvent,
    PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `root` > `button[data-fandhe-hold-to-confirm][data-fandhe-hold-duration-ms="80"]`
/// を組み立てて返す。保持時間を短く固定し（80ms）、テストの `sleep` 相当を
/// 短時間に抑える。
fn build_dom(document: &Document, duration_ms: &str) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    let button = document.create_element("button").unwrap();
    button.set_attribute("type", "button").unwrap();
    button.set_attribute(HOLD_TO_CONFIRM_ATTR, "").unwrap();
    button
        .set_attribute(HOLD_DURATION_MS_ATTR, duration_ms)
        .unwrap();
    root.append_child(&button).unwrap();
    document
        .body()
        .expect("document must have a body")
        .append_child(&root)
        .unwrap();
    (root, button)
}

fn pointer_event(kind: &str) -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent construction must not fail")
}

fn keyboard_event(kind: &str, key: &str) -> KeyboardEvent {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init)
        .expect("KeyboardEvent construction must not fail")
}

/// `target` へクリックカウンタを配線し、以後の発火数を数えるハンドルを
/// 返す（`Rc<Cell<u32>>`、`.forget()` されたリスナー自身は drop されない）。
fn count_clicks(target: &Element) -> Rc<Cell<u32>> {
    let count = Rc::new(Cell::new(0_u32));
    let count_for_closure = count.clone();
    let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        count_for_closure.set(count_for_closure.get() + 1);
    });
    target
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
    count
}

/// `element` の `--fandhe-motion-hold-progress` インラインスタイル値を
/// 読む（未設定なら空文字）。
fn hold_progress_style_value(element: &Element) -> String {
    element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must cast to HtmlElement for style access")
        .style()
        .get_property_value(HOLD_PROGRESS_VAR)
        .unwrap_or_default()
}

/// `ms` ミリ秒待つ（`wasm-bindgen-futures` の `JsFuture` + `setTimeout`）。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist in browser test");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

#[wasm_bindgen_test]
async fn holding_past_duration_confirms_and_dispatches_click_once() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "80");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_hold_to_confirm(root).expect("wire_hold_to_confirm must not fail");

    let clicks = count_clicks(&button);
    button
        .dispatch_event(&pointer_event("pointerdown"))
        .unwrap();

    sleep_ms(200).await;

    assert_eq!(
        button.get_attribute("data-state").as_deref(),
        Some("confirmed"),
        "80ms 保持後は confirmed になるはず"
    );
    assert_eq!(clicks.get(), 1, "確定時の合成 click は 1 回のみのはず");

    // 確定表示の自動リセット後、`data-state` の除去だけでなく進行度
    // （塗りつぶし）も 0 へ戻ることを確認する（是正済みバグの回帰
    // テスト: 以前は data-state だけ消えて塗りつぶしが 100% のまま
    // 残留していた）。
    sleep_ms(DEFAULT_CONFIRMED_RESET_TIMEOUT_MS + 200).await;
    assert_eq!(
        button.get_attribute("data-state"),
        None,
        "自動リセット後は data-state が除去されるはず"
    );
    assert_eq!(
        hold_progress_style_value(&button),
        "0",
        "自動リセット後は進行度も 0 へ戻るはず"
    );
}

#[wasm_bindgen_test]
async fn releasing_before_duration_does_not_confirm() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "500");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_hold_to_confirm(root).expect("wire_hold_to_confirm must not fail");

    let clicks = count_clicks(&button);
    button
        .dispatch_event(&pointer_event("pointerdown"))
        .unwrap();
    sleep_ms(50).await;
    button.dispatch_event(&pointer_event("pointerup")).unwrap();
    sleep_ms(100).await;

    assert_ne!(
        button.get_attribute("data-state").as_deref(),
        Some("confirmed"),
        "目標時間より早く離した場合は確定しないはず"
    );
    assert_eq!(clicks.get(), 0, "早期離脱では click が発火しないはず");
}

#[wasm_bindgen_test]
async fn short_click_without_hold_does_not_dispatch_click() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "500");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_hold_to_confirm(root).expect("wire_hold_to_confirm must not fail");

    let clicks = count_clicks(&button);
    button
        .dispatch_event(&pointer_event("pointerdown"))
        .unwrap();
    button.dispatch_event(&pointer_event("pointerup")).unwrap();
    sleep_ms(20).await;

    assert_eq!(
        clicks.get(),
        0,
        "短押し（保持なし）では click が発火しないはず"
    );
}

#[wasm_bindgen_test]
async fn holding_enter_key_confirms() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "80");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_hold_to_confirm(root).expect("wire_hold_to_confirm must not fail");

    let clicks = count_clicks(&button);
    button
        .dispatch_event(&keyboard_event("keydown", "Enter"))
        .unwrap();

    sleep_ms(200).await;

    assert_eq!(
        button.get_attribute("data-state").as_deref(),
        Some("confirmed"),
        "Enter キー保持でも 80ms 後は confirmed になるはず"
    );
    assert_eq!(clicks.get(), 1);
}

/// codex-review P1 指摘の回帰テスト: タッチ/ペンの暗黙 pointer capture 下
/// では要素外への物理移動でも `pointerleave` は発火しない
/// （`hold_to_confirm.rs::handle_pointermove` rustdoc 参照）。本テストは
/// 実際の暗黙 capture までは再現しないが、`pointerleave` を一切発火させず
/// `pointermove`（要素外の座標）のみを送ることで、ヒットテスト経路
/// （`Document::element_from_point`）単体が早期離脱を検知できることを
/// 確認する。
#[wasm_bindgen_test]
async fn moving_pointer_outside_element_cancels_hold_via_hit_test() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, button) = build_dom(&document, "80");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_hold_to_confirm(root).expect("wire_hold_to_confirm must not fail");

    let clicks = count_clicks(&button);
    button
        .dispatch_event(&pointer_event("pointerdown"))
        .unwrap();
    sleep_ms(20).await;

    // ビューポート外（明確に要素の外）の座標で pointermove を送る。
    // `pointerleave` は一切発火させない（暗黙 capture 下の実挙動を模す）。
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_client_x(-1000);
    init.set_client_y(-1000);
    let move_event = PointerEvent::new_with_event_init_dict("pointermove", &init)
        .expect("PointerEvent construction must not fail");
    button.dispatch_event(&move_event).unwrap();

    sleep_ms(200).await;

    assert_ne!(
        button.get_attribute("data-state").as_deref(),
        Some("confirmed"),
        "要素外への pointermove（ヒットテスト経路）で中断されるはず"
    );
    assert_eq!(
        clicks.get(),
        0,
        "要素外への pointermove 後は click が発火しないはず"
    );
}

/// `JsValue` 経由での `wire_hold_to_confirm` 戻り値の型を静的に確認する
/// （テストではなく、`Result<(), JsValue>` 契約の型検証のためのコンパイル
/// 時アサーション）。
#[allow(dead_code)]
fn _wire_hold_to_confirm_returns_result(root: Element) -> Result<(), JsValue> {
    wire_hold_to_confirm(root)
}
