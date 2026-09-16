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
use web_sys::{Document, Element, FormData, HtmlElement, HtmlFormElement, HtmlInputElement};

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

    let ghost =
        insert_exit_ghost(&container, &rows[0], false).expect("切り離された行はゴースト化される");
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
        insert_exit_ghost(&container, &rows[0], false).is_none(),
        "接続済みの行はゴースト化されないこと"
    );
}

#[wasm_bindgen_test]
fn insert_exit_ghost_is_none_when_key_still_present_despite_disconnection() {
    // codex-review 指摘の回帰（イシュー #2544）: 同じキーの行のタグ変更等
    // で旧要素が DOM から切り離されても、更新後のキー集合に当該キーが
    // 残っている（`key_still_present: true`）場合は実際の `Remove` では
    // ないためゴースト化しない。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-7");
    let _guard = RemoveOnDrop(container.clone());

    let row = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row.set_attribute(KEY_ATTR, "a").unwrap();
    container.append_child(&row).unwrap();
    let rows = snapshot_rows(&container, KEY_ATTR);

    // タグ変更を模す: 旧要素を DOM から切り離す（`Remove` と同じ見た目）。
    row.remove();

    assert!(
        insert_exit_ghost(&container, &rows[0], true).is_none(),
        "キーが残存する要素置換はゴースト化されないこと"
    );
}

#[wasm_bindgen_test]
fn insert_exit_ghost_disables_form_controls_in_self_and_descendants() {
    // codex-review 指摘の回帰（イシュー #2544）: `inert`/`aria-hidden` は
    // フォーム送信データの構築規則（HTML Standard）からの除外条件では
    // ないため、退場ゴースト内の入力欄を `disabled` にして送信・制約
    // 検証の対象から明示的に外す。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-8");
    let _guard = RemoveOnDrop(container.clone());

    let row = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row.set_attribute(KEY_ATTR, "a").unwrap();
    let input = document
        .create_element("input")
        .expect("create_element must not fail for input");
    input.set_attribute("name", "field").unwrap();
    row.append_child(&input).unwrap();
    container.append_child(&row).unwrap();

    let rows = snapshot_rows(&container, KEY_ATTR);
    row.remove();

    let ghost =
        insert_exit_ghost(&container, &rows[0], false).expect("切り離された行はゴースト化される");
    let descendant_input = ghost
        .query_selector("input")
        .expect("query_selector must not fail")
        .expect("input が子孫に存在すること");
    assert!(
        descendant_input.has_attribute("disabled"),
        "ゴースト内の input は送信対象から除外するため disabled になること"
    );
}

/// フォーム内リストの退場ゴーストは `fieldset` ごと `disabled` になり、
/// `required` 未入力のゴーストが `form.checkValidity()` を偽にせず、
/// `new FormData(form)` の同名エントリが残存行の 1 件のままであること
/// （PR #2582 codex-review P1、`ticker_browser.rs::
/// ensure_copies_disables_form_controls_in_clones` と同型）。
#[wasm_bindgen_test]
fn insert_exit_ghost_excludes_ghost_from_submission_and_validation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let form = document
        .create_element("form")
        .unwrap()
        .dyn_into::<HtmlFormElement>()
        .unwrap();
    let _guard = RemoveOnDrop(form.clone().into());
    document.body().unwrap().append_child(&form).unwrap();
    let container = create_container(&document, "presence-root-10");
    form.append_child(&container).unwrap();

    let make_row = |key: &str, value: &str| {
        let row = document.create_element("fieldset").unwrap();
        row.set_attribute(KEY_ATTR, key).unwrap();
        let input = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input.set_name("note");
        input.set_required(true);
        input.set_value(value);
        row.append_child(&input).unwrap();
        container.append_child(&row).unwrap();
        row
    };
    let leaving = make_row("a", "");
    let _staying = make_row("b", "filled");

    let rows = snapshot_rows(&container, KEY_ATTR);
    leaving.remove();
    insert_exit_ghost(&container, &rows[0], false).expect("切り離された行はゴースト化される");

    assert!(
        form.check_validity(),
        "ゴースト内の required 未入力欄が制約検証を阻んではならない"
    );
    let data = FormData::new_with_form(&form).unwrap();
    assert_eq!(
        data.get_all("note").length(),
        1,
        "同名の送信エントリは残存行の 1 件だけであるはず"
    );
}

#[wasm_bindgen_test]
fn insert_exit_ghost_isolates_radio_from_original_group() {
    // codex-review 指摘の回帰（イシュー #2544、PR #2585 レビュー）:
    // 選択済み radio A の行を削除し、同じ更新で同一 name の radio B を
    // 選択すると、A をゴーストとして再挿入する際に「checkedness が true
    // の要素がツリーへ挿入されると同じグループの他要素の checkedness を
    // false にする」という HTML Standard の同期規則が発火し、B の選択が
    // 意図せず解除されていた。`disabled` はグループ membership からの
    // 除外条件ではないため、`name` 属性の除去でグループ自体から隔離する。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "presence-root-9");
    let _guard = RemoveOnDrop(container.clone());

    let row_a = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row_a.set_attribute(KEY_ATTR, "a").unwrap();
    let radio_a = document
        .create_element("input")
        .expect("create_element must not fail for input")
        .dyn_into::<HtmlInputElement>()
        .expect("input element must cast to HtmlInputElement");
    radio_a.set_type("radio");
    radio_a.set_name("grp");
    radio_a.set_checked(true);
    row_a.append_child(&radio_a).unwrap();

    let row_b = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    row_b.set_attribute(KEY_ATTR, "b").unwrap();
    let radio_b = document
        .create_element("input")
        .expect("create_element must not fail for input")
        .dyn_into::<HtmlInputElement>()
        .expect("input element must cast to HtmlInputElement");
    radio_b.set_type("radio");
    radio_b.set_name("grp");
    row_b.append_child(&radio_b).unwrap();

    container.append_child(&row_a).unwrap();
    container.append_child(&row_b).unwrap();

    let rows = snapshot_rows(&container, KEY_ATTR);
    row_a.remove();
    // 同じ更新で B を選択する（A のゴースト再挿入前に発生する想定）。
    radio_b.set_checked(true);

    let ghost =
        insert_exit_ghost(&container, &rows[0], false).expect("切り離された行はゴースト化される");

    assert!(
        radio_b.checked(),
        "A のゴースト再挿入で B の選択が解除されてはならない"
    );
    let ghost_radio = ghost
        .query_selector("input")
        .expect("query_selector must not fail")
        .expect("input が子孫に存在すること")
        .dyn_into::<HtmlInputElement>()
        .expect("input element must cast to HtmlInputElement");
    assert!(
        !ghost_radio.has_attribute("name"),
        "ゴースト内の radio は元の group から隔離するため name を持たないこと"
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
