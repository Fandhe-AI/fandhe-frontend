//! `hydrate()` の実ブラウザ最小スモークテスト（TASK-6.2c、#49）。
//!
//! `.github/workflows/ci.yml` の `browser-test` ジョブは `wasm-client/Cargo.toml`
//! の存在ガードにより、本クレート新設と同時に
//! `wasm-pack test --headless --chrome wasm-client` を自動実行する状態になる
//! （`docs/hydration-api.md` 第 7 節）。本ファイルはそのジョブを green に保つ
//! ための最小限のスモークテストであり、REQ-6 受け入れ基準の**正式な実証**
//! （クリック発火・状態復元を含む網羅的なブラウザ E2E）は TASK-6.3b（#66）の
//! `hydration_browser.rs` へ引き継ぐ（本ファイルはその前段の green 保証措置）。
//!
//! 検証する不変条件（`docs/hydration-api.md` 第 6 節）:
//! - `hydrate()` は root 不在時に `panic!` せず `Err` を返す（不変条件 5・6）。
//! - `hydrate()` は既存 DOM（`data-hydrate="like"` を持つ要素）を破壊せず
//!   （`set_inner_html` を呼ばない、不変条件 2）、`click` イベントで
//!   `class_list`（`liked` クラス）のみを更新する（不変条件 3）。

#![cfg(target_arch = "wasm32")]

use rws_wasm_client::hydrate;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナを document body へ 1 個生成し、一意な id を振る。
/// `wasm-full/tests/perf_browser.rs` の `create_scenario_container` と同じ
/// 意図（wasm-bindgen-test はテスト間で DOM をリセットしないため）。
fn create_container(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// `rws_app::detail_page` が出力するボタン相当（`id="like-btn"
/// data-hydrate="like"`）の合成 DOM をルート配下に構築する。
/// サーバー出力済み DOM を模しており、`hydrate()` はこれを再構築せず
/// リスナーを後付けするだけであることを検証する。
fn append_like_button(document: &Document, root: &Element) -> Element {
    let button = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    button
        .set_attribute("id", "like-btn")
        .expect("set_attribute must not fail");
    button
        .set_attribute("data-hydrate", "like")
        .expect("set_attribute must not fail");
    button.set_text_content(Some("いいね"));
    root.append_child(&button)
        .expect("append_child must not fail for a detached button");
    button
}

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

/// `hydrate()` が root 不在時に `panic!` せず `Err` を返すことを確認する
/// （不変条件 5・6: `unwrap()`/`panic!` を用いず、内部情報を含まないエラーで返す）。
#[wasm_bindgen_test]
fn hydrate_returns_err_without_panicking_when_root_missing() {
    let result = hydrate("no-such-root-id");
    assert!(result.is_err(), "存在しない root_id では Err を返すこと");
}

/// `hydrate()` が既存 DOM を再構築せず（`set_inner_html` を呼ばない不変条件 2）、
/// `data-hydrate="like"` 要素へ `click` リスナーを後付けし、ハンドラが
/// `class_list`（`liked` クラス）のみを更新することを確認する（不変条件 3）。
#[wasm_bindgen_test]
fn hydrate_wires_click_listener_without_rebuilding_existing_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = create_container(&document, "hydrate-smoke-root");
    let button = append_like_button(&document, &root);

    // hydrate() 前後で「いいね」ボタンの text_content が変化しない
    // （既存 DOM を再構築しない不変条件の直接確認）。
    let text_before = button.text_content();

    hydrate("hydrate-smoke-root").expect("hydrate must succeed when root and target exist");

    assert_eq!(
        button.text_content(),
        text_before,
        "hydrate() 実行だけでは既存テキストが変化しないこと（再構築していない証跡）"
    );
    assert!(
        !button.class_list().contains("liked"),
        "hydrate() 実行直後（クリック前）は liked クラスがまだ付与されていないこと"
    );

    button
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        button.class_list().contains("liked"),
        "click 後付けリスナーにより liked クラスが付与されること"
    );
    assert_eq!(
        button.text_content(),
        text_before,
        "click ハンドラは class_list のみを更新し、テキスト内容を書き換えないこと"
    );
}
