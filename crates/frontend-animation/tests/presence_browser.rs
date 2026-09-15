//! `fandhe_frontend_animation::presence` の実ブラウザ統合テスト
//! （イシュー #2544）。`flip_browser.rs`/`confetti_canvas_browser.rs` と
//! 同型の `wasm-pack test --headless --chrome` ハーネス。
//!
//! 純粋層（`parse_css_time_list`/`total_animation_ms`）は
//! `src/presence.rs` の native `cargo test` が検証済み。本ファイルは
//! DOM 計測・ゴースト挿入・実タイマー除去のみを対象とする。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::presence::{
    ensure_positioned, insert_exit_ghost, remove_when_settled, snapshot_rows, RowSnapshot,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

const KEY_ATTR: &str = "data-key";

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

fn create_container(document: &Document, id: &str) -> Element {
    let el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    el.set_id(id);
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

/// `<style>` を `document.head()` へ挿入する
/// （`position_browser.rs::ensure_fixed_floating_size_stylesheet` と同型）。
fn install_stylesheet(document: &Document, css: &str) -> Element {
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a plain style element");
    style.set_text_content(Some(css));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a detached style element");
    style
}

#[wasm_bindgen_test]
fn insert_exit_ghost_places_disconnected_row_at_original_coordinates() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-1");
    let _guard = RemoveOnDrop(container.clone());

    let row = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row.set_attribute(KEY_ATTR, "a").unwrap();
    container.append_child(&row).unwrap();

    let rows = snapshot_rows(&container, KEY_ATTR);
    assert_eq!(rows.len(), 1, "撮影前は 1 行接続されていること");

    // `Remove` を模す: 実 DOM から取り除く（元要素そのもの、clone ではない）。
    row.remove();

    let ghost = insert_exit_ghost(&container, &rows[0]).expect("切り離された行はゴースト化される");
    assert!(
        ghost.is_connected(),
        "ゴーストは再挿入され接続済みであること"
    );
    assert_eq!(
        ghost.get_attribute("data-state").as_deref(),
        Some("exiting")
    );
    assert_eq!(ghost.get_attribute("aria-hidden").as_deref(), Some("true"));
    assert!(ghost.has_attribute("inert"));
    let style = ghost.style();
    assert_eq!(style.get_property_value("position").unwrap(), "absolute");
    assert_eq!(
        style.get_property_value("top").unwrap(),
        format!("{}px", rows[0].top)
    );
    assert_eq!(
        style.get_property_value("left").unwrap(),
        format!("{}px", rows[0].left)
    );
}

#[wasm_bindgen_test]
fn insert_exit_ghost_is_none_when_element_still_connected() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-2");
    let _guard = RemoveOnDrop(container.clone());

    let row = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row.set_attribute(KEY_ATTR, "a").unwrap();
    container.append_child(&row).unwrap();
    let rows = snapshot_rows(&container, KEY_ATTR);

    // `row` を取り除かないまま呼ぶ（構造変化が Remove を伴わなかった場合）。
    assert!(
        insert_exit_ghost(&container, &rows[0]).is_none(),
        "接続済みの行はゴースト化されないこと"
    );
}

#[wasm_bindgen_test]
fn remove_when_settled_removes_immediately_without_animation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-3");
    let _guard = RemoveOnDrop(container.clone());

    let ghost = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<HtmlElement>()
        .unwrap();
    container.append_child(&ghost).unwrap();
    assert!(ghost.is_connected());

    // `animation-duration` を持たない（既定 `0s`）ため、除去は同期的に
    // 起きる想定。
    remove_when_settled(ghost.clone());
    assert!(
        !ghost.is_connected(),
        "animation 未定義のゴーストは即座に除去されること"
    );
}

#[wasm_bindgen_test]
async fn remove_when_settled_waits_for_computed_animation_duration() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-4");
    let _guard = RemoveOnDrop(container.clone());

    let style_el = install_stylesheet(
        &document,
        ".presence-test-exit { animation: fd-test-noop 60ms; } \
         @keyframes fd-test-noop { from { opacity: 1; } to { opacity: 1; } }",
    );
    let _style_guard = RemoveOnDrop(style_el);

    let ghost = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<HtmlElement>()
        .unwrap();
    ghost.set_class_name("presence-test-exit");
    container.append_child(&ghost).unwrap();

    remove_when_settled(ghost.clone());
    assert!(
        ghost.is_connected(),
        "60ms アニメーション中は除去されていないこと"
    );

    // 60ms + 50ms 余裕 + 実測ずれ吸収のマージン。
    sleep_ms(200).await;
    assert!(
        !ghost.is_connected(),
        "アニメーション時間 + 余裕経過後は除去されていること"
    );
}

#[wasm_bindgen_test]
fn ensure_positioned_is_idempotent_and_only_upgrades_static() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-5");
    let _guard = RemoveOnDrop(container.clone());

    let list = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<HtmlElement>()
        .unwrap();
    container.append_child(&list).unwrap();

    ensure_positioned(&list);
    assert_eq!(
        list.style().get_property_value("position").unwrap(),
        "relative",
        "static な要素は relative へ昇格すること"
    );

    // 著者が明示した `position` は上書きしない。
    let list2 = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<HtmlElement>()
        .unwrap();
    list2
        .style()
        .set_property("position", "sticky")
        .expect("set_property must not fail");
    container.append_child(&list2).unwrap();
    ensure_positioned(&list2);
    assert_eq!(
        list2.style().get_property_value("position").unwrap(),
        "sticky",
        "著者が明示した position は上書きされないこと"
    );

    ensure_positioned(&list);
    assert_eq!(
        list.style().get_property_value("position").unwrap(),
        "relative",
        "冪等: 再度呼んでも relative のまま"
    );
}

#[wasm_bindgen_test]
fn snapshot_rows_records_key_and_geometry_for_each_direct_child() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-6");
    let _guard = RemoveOnDrop(container.clone());

    for key in ["a", "b", "c"] {
        let row = document
            .create_element("div")
            .expect("create_element must not fail for a plain div");
        row.set_attribute(KEY_ATTR, key).unwrap();
        container.append_child(&row).unwrap();
    }

    let rows: Vec<RowSnapshot> = snapshot_rows(&container, KEY_ATTR);
    assert_eq!(
        rows.iter().map(|r| r.key.clone()).collect::<Vec<_>>(),
        vec!["a", "b", "c"],
        "DOM 順に key を記録すること"
    );
}
