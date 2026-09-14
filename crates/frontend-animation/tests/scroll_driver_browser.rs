//! `fandhe_frontend_animation::scroll_driver`（イシュー #2521、親 `#2499`）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/scroll_driver.rs` の
//! `compute_progress_tests`/`native_no_panic_tests`）は純粋計算・native
//! no-panic を検証済みである。本ファイルは以下を実ブラウザ（headless
//! Chromium）上で検証する。
//!
//! 1. [`ScrollDriver::start`]/[`ScrollDriver::mark_dirty`] が同一フレーム内
//!    の複数呼び出しを 1 回の `recompute` へ coalesce すること
//! 2. [`update_element_progress`] が実 DOM 要素に対し、実スクロールに応じて
//!    単調に変化する custom property を書き込むこと

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_animation::dom_target::DomTarget;
use fandhe_frontend_animation::scroll_driver::{
    update_element_progress, ScrollDriver, SCROLL_PROGRESS_PROPERTY,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// 1 rAF フレーム分だけ待つ（`ScrollDriver` の dirty-flag 反映・
/// スクロール後の測定タイミング確認用）。
async fn wait_one_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .request_animation_frame(callback.unchecked_ref())
            .expect("requestAnimationFrame must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("requestAnimationFrame promise must not reject");
}

#[wasm_bindgen_test]
async fn scroll_driver_coalesces_multiple_mark_dirty_into_one_recompute_per_frame() {
    let call_count = Rc::new(RefCell::new(0_u32));
    let call_count_for_recompute = call_count.clone();
    let driver = ScrollDriver::start(move || {
        *call_count_for_recompute.borrow_mut() += 1;
    });

    // 開始直後の 1 フレーム目は無条件で 1 回呼ばれる契約（初期スクロール
    // 位置の反映）。まずこの 1 回を消化してから同一フレーム内の複数
    // `mark_dirty()` 呼び出しの coalesce を検証する。
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        1,
        "開始直後 1 フレーム目は無条件で 1 回呼ばれるはず"
    );

    // 同一フレーム内で複数回 mark_dirty しても、次フレームの recompute は
    // 1 回だけであることを確認する。
    driver.mark_dirty();
    driver.mark_dirty();
    driver.mark_dirty();
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        2,
        "複数回の mark_dirty は 1 フレームにつき 1 回の recompute へ coalesce されるはず"
    );

    // mark_dirty を呼ばないフレームでは recompute が増えないことも確認する。
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        2,
        "mark_dirty を呼ばないフレームでは recompute が増えないはず"
    );

    driver.stop();
}

#[wasm_bindgen_test]
async fn update_element_progress_writes_monotone_progress_as_scroll_advances() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");

    // 3000px の spacer の直後に高さ 200px の target を body 直下へ配置する。
    // ページ全体（`window`）をスクロールして target をビューポートへ近づける
    // 構成は、本番配線（`wire_scroll_driver_with_env` が `document`/`window`
    // へ `scroll`/`resize` リスナーを張る設計）に対応する最も素直な検証方法
    // である。`compute_progress` は `rect_top` の単調減少に対して
    // （clamp を介しても）単調非減少を保つ純関数のため、実際のビューポート
    // サイズ（headless Chrome の既定値）に依存せず本テストは決定的に成立
    // する。
    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height:3000px")
        .expect("set_attribute must not fail");

    let target_el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    target_el
        .set_attribute("style", "height:200px")
        .expect("set_attribute must not fail");

    let body = document
        .body()
        .expect("document body must exist in browser test environment");
    body.append_child(&spacer)
        .expect("append_child must not fail for a detached spacer");
    body.append_child(&target_el)
        .expect("append_child must not fail for a detached target");

    let element: web_sys::Element = target_el.clone().into();
    let mut target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);

    // scrollY == 0: target は spacer 3000px 分下にあり、どの一般的な
    // ビューポート高さでも `rect_top` はビューポート高さを大きく超える
    // ため、progress は 0.0 に clamp される（決定的）。
    window.scroll_to_with_x_and_y(0.0, 0.0);
    wait_one_frame().await;
    let progress_at_0 = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    window.scroll_to_with_x_and_y(0.0, 1500.0);
    wait_one_frame().await;
    let progress_at_mid = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    // scrollY == 3000: target はページ最上部近くまで到達し、`rect_top` は
    // 十分小さくなるため progress は 1.0 に clamp される（決定的）。
    window.scroll_to_with_x_and_y(0.0, 3000.0);
    wait_one_frame().await;
    let progress_at_full = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    for (label, value) in [
        ("progress_at_0", progress_at_0),
        ("progress_at_mid", progress_at_mid),
        ("progress_at_full", progress_at_full),
    ] {
        assert!(
            (0.0..=1.0).contains(&value),
            "{label} は 0.0..=1.0 の範囲であるはず: {value}"
        );
    }
    assert_eq!(progress_at_0, 0.0, "スクロール前は未進入で 0.0 のはず");
    assert!(
        progress_at_mid >= progress_at_0,
        "スクロールを進めると progress は単調非減少のはず: {progress_at_0} -> {progress_at_mid}"
    );
    assert!(
        progress_at_full >= progress_at_mid,
        "スクロールを進めると progress は単調非減少のはず: {progress_at_mid} -> {progress_at_full}"
    );
    assert!(
        progress_at_full > progress_at_0,
        "十分なスクロール後は未進入時より progress が増加しているはず"
    );

    let written = target_el
        .style()
        .get_property_value(SCROLL_PROGRESS_PROPERTY)
        .expect("get_property_value must not fail");
    assert_eq!(
        written
            .parse::<f64>()
            .expect("written value must be a valid f64 string"),
        progress_at_full,
        "custom property には最後に書き込んだ progress がそのまま反映されているはず"
    );

    // 後続テストへの汚染防止（ページスクロール位置・追加した DOM 要素を
    // 元に戻す）。
    window.scroll_to_with_x_and_y(0.0, 0.0);
    spacer.remove();
    target_el.remove();
}
