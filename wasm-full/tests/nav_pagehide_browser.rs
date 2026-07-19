//! `pagehide` によるリロード時スクロール位置保存の実ブラウザ検証
//! （Bugbot 指摘、PR #423 / イシュー #406 追加分）。
//!
//! # 責務境界・独立ファイルにした理由
//!
//! `wasm-full/tests/nav_browser.rs` が配線層（`rws_wasm_full::nav::
//! start_router`）の大半のブラウザ挙動を検証済み。本ファイルはそこへ
//! 追加した `pagehide` リスナー（[`rws_wasm_full::nav`] モジュール doc・
//! `mod wiring::start_router` 参照）のみを対象とする。
//!
//! `wasm-pack test`/`cargo test --target wasm32-unknown-unknown` は
//! テスト**ファイル単位**で別ページ（別 Wasm インスタンス）を起動するため、
//! `nav_browser.rs` の既存 14 テストと同一ページ・同一 `window` を共有
//! しない。本ファイルを独立させたのは、実ブラウザで合成 `pagehide` を
//! `window.dispatch_event` すると（`isTrusted: false` の合成イベントでも）
//! 以降の合成クリック遷移が同一ページ上で機能しなくなる実測済みの副作用が
//! あるため（Chrome 側のページライフサイクル関連の内部状態遷移によるものと
//! 推測されるが、原因はブラウザ実装依存で本リポジトリ側のコード起因ではない）。
//! `nav_browser.rs` 側のクリック遷移・popstate 系テスト群を汚染しないよう、
//! `pagehide` を実際に dispatch する検証だけを本ファイルへ隔離する。

#![cfg(target_arch = "wasm32")]

use rws_app::{assemble_list_page, DemoItemsLoader};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// `wasm-full/tests/nav_browser.rs::create_app_root` と同型のフィクスチャ
/// 構築（`page_shell` の `<body>` 直下に `<div id="app-root">` を単独で置く
/// 構造の再現）。本ファイル専用に最小限だけ複製する（ファイル単位で別ページ
/// のため `mod` 越しの共有はできない、`nav_browser.rs` と同じ制約）。
fn create_app_root(document: &Document, container_test_id: &str, initial_html: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(container_test_id);
    container.set_inner_html(initial_html);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

fn ssr_equivalent_list_inner_html() -> String {
    let body = assemble_list_page(&DemoItemsLoader, &()).expect("infallible loader");
    rws_core::render(&body)
}

/// `nav_browser.rs::append_tall_spacer` と同型（イシュー #406）。ヘッドレス
/// ビューポートより十分に高い（3000px）スペーサーを追加し、
/// `window.scroll_to_with_x_and_y` が実際にスクロール可能にする。
fn append_tall_spacer(document: &Document, container: &Element) {
    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height: 3000px;")
        .expect("set_attribute must not fail");
    container
        .append_child(&spacer)
        .expect("append_child must not fail");
}

/// history state（URL）を `path` へ揃える（`nav_browser.rs::set_location_path`
/// と同型）。
fn set_location_path(path: &str) {
    let window = web_sys::window().expect("window must exist");
    let history = window.history().expect("history must exist");
    history
        .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
        .expect("replace_state must not fail in test environment");
}

/// `pagehide`（リロード・外部遷移・タブクローズ相当）を dispatch すると、
/// 現在エントリの `history.state` へその時点のスクロール位置が
/// `replace_state` で書き戻されること。
///
/// 従来は `push_state` 直後のエントリの `state` が `JsValue::NULL` のまま
/// 残り、そのページ上でリロードすると復元先の記録が存在せずスクロール
/// 位置が失われていた（Bugbot 指摘、PR #423）。`push_and_render` が内部で
/// 呼ぶ `push_state_with_url` と同じ引数形（state = `JsValue::NULL`）を
/// history API で直接再現し、実クリックイベント経由の遷移は使わない
/// （検証対象は「`pagehide` が現在エントリへ書き戻すこと」自体であり、
/// クリック遷移そのものの契約は `nav_browser.rs` 側が別途固定済みのため）。
#[wasm_bindgen_test]
fn pagehide_saves_current_scroll_position_to_history_state() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let container = create_app_root(
        &document,
        "nav-pagehide-test-root",
        &ssr_equivalent_list_inner_html(),
    );
    append_tall_spacer(&document, &container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // `push_state` 直後のエントリ（state は JsValue::NULL のまま）を作る。
    let history = window.history().expect("history must exist");
    history
        .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/items/1"))
        .expect("push_state must not fail in test environment");
    assert!(
        history.state().expect("state must not fail").is_null(),
        "テスト前提条件: push_state 直後のエントリの state は JsValue::NULL のままであること"
    );

    // 新規エントリ上でスクロールしてから離脱（リロード相当）する。
    window.scroll_to_with_x_and_y(0.0, 650.0);
    assert!(
        window.scroll_y().unwrap() > 0.0,
        "テスト前提条件: pagehide 前にスクロール済みであること"
    );

    let pagehide_event = web_sys::Event::new("pagehide").expect("Event construction must not fail");
    window
        .dispatch_event(&pagehide_event)
        .expect("dispatch_event must not fail");

    let saved_state = history
        .state()
        .expect("state must not fail")
        .as_string()
        .expect("pagehide 後の history.state は文字列エンコードされたスクロールレコードであること");
    let (x, y) = rws_wasm_full::nav::decode_scroll_state(&saved_state)
        .expect("pagehide 後の history.state は decode_scroll_state で復号できる形式であること");
    assert_eq!(x, 0.0);
    assert!(
        (y - 650.0).abs() < 1.0,
        "pagehide は現在エントリへ現在のスクロール位置（650）を書き戻すこと: {y}"
    );

    container.remove();
}
