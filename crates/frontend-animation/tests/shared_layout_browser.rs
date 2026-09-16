//! `fandhe_frontend_animation::shared_layout` を実ブラウザ上で駆動し、
//! 旧要素（アンマウント済み）の視覚矩形から新要素が FLIP 補正で移動・
//! 収束することを確認する統合テスト（イシュー #2536。
//! `flip_browser.rs` と同型の待機戦略）。

#![cfg(target_arch = "wasm32")]

use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_frontend_animation::shared_layout::{self, SharedSnapshot};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `flip_browser.rs::sleep_ms` と同じ意図: 実タイマーを `Promise` 化して
/// `await` する決定的な待機。
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

fn make_positioned_div(
    document: &web_sys::Document,
    left: &str,
    top: &str,
    width: &str,
    height: &str,
) -> web_sys::HtmlElement {
    let div = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    let style = div.style();
    style
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    style
        .set_property("left", left)
        .expect("set_property must not fail");
    style
        .set_property("top", top)
        .expect("set_property must not fail");
    style
        .set_property("width", width)
        .expect("set_property must not fail");
    style
        .set_property("height", height)
        .expect("set_property must not fail");
    div
}

fn settle_ms() -> f64 {
    Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は有効な spring パラメータのため None にならない")
        .settle_duration()
        * 1000.0
}

#[wasm_bindgen_test]
async fn play_shared_moves_new_element_from_old_rect_then_settles() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let body = document
        .body()
        .expect("document body must exist in browser test environment");

    let old = make_positioned_div(&document, "0px", "0px", "50px", "50px");
    body.append_child(&old)
        .expect("append_child must not fail for a detached div");

    let snapshot: SharedSnapshot = shared_layout::snapshot([("hero".to_string(), old.clone())]);
    old.remove();

    let new_el = make_positioned_div(&document, "200px", "100px", "100px", "100px");
    body.append_child(&new_el)
        .expect("append_child must not fail for a detached div");

    let mut played = shared_layout::play_shared(
        &snapshot,
        &[("hero".to_string(), new_el.clone())],
        SpringConfig::default(),
    );
    assert_eq!(
        played.len(),
        1,
        "旧要素と新要素は同一 id のため再生されるはず"
    );

    // Invert 直後: 新要素の見た目上の矩形は旧要素の First（0,0 50x50）と
    // 一致するはず（`flip_browser.rs::play_applies_invert_then_settles_to_
    // identity_transform` と同じ検証手段）。
    let rect = new_el.get_bounding_client_rect();
    assert!((rect.x() - 0.0).abs() < 0.5, "x={}", rect.x());
    assert!((rect.y() - 0.0).abs() < 0.5, "y={}", rect.y());
    assert!((rect.width() - 50.0).abs() < 0.5, "width={}", rect.width());
    assert!(
        (rect.height() - 50.0).abs() < 0.5,
        "height={}",
        rect.height()
    );

    sleep_ms((settle_ms() * 2.0).ceil() as i32 + 200).await;
    let (_, animation) = played.pop().expect("played は 1 件のはず");
    assert!(
        animation.is_done(),
        "settle_duration の倍以上待てば収束しているはず"
    );
    drop(animation);

    let transform_after_settle = new_el
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform_after_settle.is_empty(),
        "収束後は transform が解除されているはず: {transform_after_settle}"
    );

    new_el.remove();
}

#[wasm_bindgen_test]
fn play_shared_skips_same_node() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let body = document
        .body()
        .expect("document body must exist in browser test environment");

    let element = make_positioned_div(&document, "0px", "0px", "50px", "50px");
    body.append_child(&element)
        .expect("append_child must not fail for a detached div");

    // `snapshot` した要素をそのまま `after` にも渡す（同一ノード）。
    let snapshot = shared_layout::snapshot([("hero".to_string(), element.clone())]);
    let played = shared_layout::play_shared(
        &snapshot,
        &[("hero".to_string(), element.clone())],
        SpringConfig::default(),
    );
    assert!(
        played.is_empty(),
        "同一ノードは `crate::flip` の対象領域のため再生されないはず"
    );
    let transform = element
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform.is_empty(),
        "同一ノードの組は transform を書き込まないはず"
    );

    element.remove();
}

#[wasm_bindgen_test]
fn play_shared_skips_when_old_element_still_connected() {
    let document = web_sys::window()
        .expect("window must exist in browser test environment")
        .document()
        .expect("document must exist");
    let body = document
        .body()
        .expect("document body must exist in browser test environment");

    let old = make_positioned_div(&document, "0px", "0px", "50px", "50px");
    body.append_child(&old)
        .expect("append_child must not fail for a detached div");

    // `old` を DOM から外さないまま snapshot する（同時表示は曖昧のため
    // 対象外、モジュール doc「突合ルール」参照）。
    let snapshot = shared_layout::snapshot([("hero".to_string(), old.clone())]);

    let new_el = make_positioned_div(&document, "200px", "100px", "100px", "100px");
    body.append_child(&new_el)
        .expect("append_child must not fail for a detached div");

    let played = shared_layout::play_shared(
        &snapshot,
        &[("hero".to_string(), new_el.clone())],
        SpringConfig::default(),
    );
    assert!(
        played.is_empty(),
        "旧要素がまだ DOM に接続中の場合は再生されないはず"
    );
    let transform = new_el
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform.is_empty(),
        "スキップ時は transform を書き込まないはず"
    );

    old.remove();
    new_el.remove();
}
