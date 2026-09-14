//! confetti（[`fire`]）を実ブラウザ上で発火し、canvas への描画・完了時の
//! クリアを確認する統合テスト（イシュー #2533 実装計画 §4.2）。
//!
//! `crates/animation/src/confetti.rs` の native テスト群（決定的物理演算）
//! と `confetti::tests::should_fire_*`（reduced-motion 純粋関数）は既に
//! 検証済みである。本テストはその先、実ブラウザの `requestAnimationFrame`
//! （[`RafDriver`]）・実 canvas（[`CanvasTarget`]）を使っても
//! ロケータ契約・発火完了・クリアが成り立つことを固定する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::confetti::{fire, fire_with_reduced_motion};
use fandhe_frontend_animation::fandhe_animation::confetti::ConfettiConfig;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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

/// `condition` が真になるまで最大 200 回、rAF または 50ms タイムアウトの
/// 早い方で待つ（`scroll_driver_browser.rs::wait_for`/
/// `in_view_browser.rs::wait_for` と同型。固定 `sleep_ms` より実フレーム
/// レート非依存で速い）。
async fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
    for _ in 0..200 {
        if condition() {
            return true;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let resolve_for_raf = resolve.clone();
            let raf_closure = Closure::once(move |_timestamp: f64| {
                resolve_for_raf.call0(&JsValue::NULL).ok();
            });
            window
                .request_animation_frame(raf_closure.as_ref().unchecked_ref())
                .expect("requestAnimationFrame must not fail");
            raf_closure.forget();

            let timeout_closure = Closure::once(move || {
                resolve.call0(&JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout_closure.as_ref().unchecked_ref(),
                    50,
                )
                .expect("setTimeout must not fail");
            timeout_closure.forget();
        });
        JsFuture::from(promise)
            .await
            .expect("promise must not reject");
    }
    condition()
}

/// `width`×`height`（CSS px）の `<canvas>` を `document.body` へ追加して
/// 返す。`getBoundingClientRect` が非ゼロを返すよう、幅・高さをインライン
/// style で明示する（`fire()` の描画領域 clamp 判定に必要）。
fn append_canvas(width: u32, height: u32) -> web_sys::Element {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let canvas = document
        .create_element("canvas")
        .expect("create_element must not fail for canvas");
    let html_element = canvas
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("canvas element must be an HtmlElement");
    html_element
        .style()
        .set_property("width", &format!("{width}px"))
        .expect("set_property must not fail for width");
    html_element
        .style()
        .set_property("height", &format!("{height}px"))
        .expect("set_property must not fail for height");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&canvas)
        .expect("append_child must not fail for a detached canvas");
    canvas
}

/// canvas 全域の `ImageData` を取得し、非透明（`alpha > 0`）のピクセルが
/// 1 件でもあるかを返す。
fn has_visible_pixels(canvas: &web_sys::Element) -> bool {
    let html_canvas = canvas
        .clone()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("canvas element must cast to HtmlCanvasElement");
    let context = html_canvas
        .get_context("2d")
        .expect("get_context must not fail")
        .expect("2d context must exist")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .expect("context must cast to CanvasRenderingContext2d");
    let width = html_canvas.width();
    let height = html_canvas.height();
    let image_data = context
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .expect("get_image_data must not fail for same-origin canvas");
    image_data
        .data()
        .0
        .as_chunks::<4>()
        .0
        .iter()
        .any(|px| px[3] > 0)
}

#[wasm_bindgen_test]
async fn fire_errors_for_non_canvas_element() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");

    let result = fire(&div, ConfettiConfig::default());
    assert!(
        result.is_err(),
        "canvas 要素でない要素への fire() は Err を返すはず"
    );
}

#[wasm_bindgen_test]
async fn fire_returns_none_for_zero_size_canvas() {
    // CSS サイズを明示しない canvas は `getBoundingClientRect` が
    // 0×0 を返す（`document.body` 未追加のため）。
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let canvas = document
        .create_element("canvas")
        .expect("create_element must not fail for canvas");

    let result = fire(&canvas, ConfettiConfig::default());
    assert!(
        matches!(result, Ok(None)),
        "描画領域 0 の canvas への fire() は Ok(None) を返すはず"
    );
}

#[wasm_bindgen_test]
async fn fire_draws_particles_then_clears_on_completion() {
    let canvas = append_canvas(100, 100);
    let config = ConfettiConfig {
        particle_count: 30,
        duration_secs: 0.15,
        ..ConfettiConfig::default()
    };

    let loop_handle = fire(&canvas, config)
        .expect("fire は canvas 要素に対して Err を返さない")
        .expect("通常環境・非ゼロサイズでは Some(AnimationLoop) を返すはず");

    // 数フレーム分ポーリングし、パーティクルが描画されていることを確認
    // する（固定 `sleep_ms` より実フレームレート非依存で速い）。
    assert!(
        wait_for(|| has_visible_pixels(&canvas)).await,
        "発火後は非透明ピクセルが描画されているはず"
    );

    // duration_secs 経過後は最終フレームで canvas がクリアされる。
    // 実フレームレート非依存の判定のため余裕を持って待機する
    // （`spring_via_raf_dom_browser.rs` と同じ方針）。
    sleep_ms(600).await;
    assert!(
        !has_visible_pixels(&canvas),
        "発火完了後は canvas がクリアされているはず"
    );

    // `AnimationLoop` の所有権契約（`confetti.rs` モジュール doc）どおり、
    // ハンドルはここまで生存させる。
    drop(loop_handle);
    canvas.remove();
}

#[wasm_bindgen_test]
async fn fire_with_reduced_motion_true_suppresses_firing() {
    // headless Chrome の実際の `prefers-reduced-motion` 判定結果に
    // 依存せず、抑制側の分岐を決定的に検証する（`reduced_motion` 明示
    // 注入、`fire_with_reduced_motion` doc 参照）。
    let canvas = append_canvas(100, 100);

    let result = fire_with_reduced_motion(&canvas, ConfettiConfig::default(), true);
    assert!(
        matches!(result, Ok(None)),
        "reduced_motion=true は Ok(None)（発火抑制）を返すはず"
    );
    assert_eq!(
        canvas.get_attribute("width"),
        None,
        "発火抑制時は canvas.set_width 等の副作用も一切起きないはず"
    );

    canvas.remove();
}
