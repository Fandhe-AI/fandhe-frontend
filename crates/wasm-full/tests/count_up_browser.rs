//! `fandhe_frontend_wasm_full::count_up::wire_count_up`（イシュー #2539）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `count_up.rs` の native テストは属性パース（トリガー・duration）のみを
//! 検証済みである。本ファイルは opt-in 要素の解決・初期テキストの解析・
//! `MutationObserver` による外部更新の再補間・`in-view` トリガーの起動が
//! 実 DOM 上で機能することを、`hold_to_confirm_browser.rs`（手組み DOM
//! への直接 `wire_*` 呼び出し、`Runtime::mount` は経由しない）と同方針で
//! 検証する。
//!
//! `fandhe-frontend-animation::count_up` 自体の補間計算・`textContent`
//! 書き込みロジックは責務境界どおり `crates/frontend-animation/tests/
//! count_up_browser.rs` で検証済みであり、本ファイルは実 DOM 上での配線
//! （候補解決・トリガー判定・再補間の起動）のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "count-up")]

use fandhe_frontend_wasm_full::count_up::{
    wire_count_up, COUNT_UP_ATTR, COUNT_UP_DURATION_MS_ATTR, COUNT_UP_TRIGGER_ATTR,
    COUNT_UP_TRIGGER_IN_VIEW,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`magnetic_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `set_timeout` を `await` 可能にする（`magnetic_browser.rs::sleep_ms`
/// と同型）。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist in browser test");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// `root`（id 付き）> `dd`（[`COUNT_UP_ATTR`] 付き、`text` を初期テキスト
/// とする）を組み立てて返す。
fn build_dom(
    document: &Document,
    root_id: &str,
    text: &str,
    extra_attrs: &[(&str, &str)],
) -> (Element, HtmlElement) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let dd = document.create_element("dd").unwrap();
    dd.set_attribute(COUNT_UP_ATTR, "").unwrap();
    for (name, value) in extra_attrs {
        dd.set_attribute(name, value).unwrap();
    }
    dd.set_text_content(Some(text));
    root.append_child(&dd).unwrap();

    document.body().unwrap().append_child(&root).unwrap();
    let html_dd = dd
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("dd must cast to HtmlElement");
    (root, html_dd)
}

/// テストは実時間の `sleep_ms` で完了を待つため、既定 duration
/// （`DEFAULT_COUNT_UP_DURATION_MS` = 1200ms）ではなく短い duration へ
/// 明示的に上書きする（`frontend-animation::count_up_browser.rs` の
/// `start_interpolates_from_zero_to_final_value` と同方針、PR #2580 で
/// 検知した「既定 duration より短い待機時間で FAIL する」回帰の是正）。
const TEST_DURATION_MS: &str = "80";

#[wasm_bindgen_test]
async fn immediate_trigger_counts_up_to_final_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, dd) = build_dom(
        &document,
        "count-up-root-1",
        "1,000",
        &[(COUNT_UP_DURATION_MS_ATTR, TEST_DURATION_MS)],
    );
    let _guard = RemoveOnDrop(root.clone());

    wire_count_up(&root).expect("wire_count_up must not fail");

    sleep_ms(300).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "1,000",
        "十分に待てば最終値（書式保存の元の文字列）へ収束するはず"
    );
}

#[wasm_bindgen_test]
async fn external_text_update_reinterpolates_to_new_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, dd) = build_dom(
        &document,
        "count-up-root-2",
        "10",
        &[(COUNT_UP_DURATION_MS_ATTR, TEST_DURATION_MS)],
    );
    let _guard = RemoveOnDrop(root.clone());

    wire_count_up(&root).expect("wire_count_up must not fail");
    sleep_ms(300).await;
    assert_eq!(dd.text_content().unwrap(), "10");

    // アプリ側の `set_text` 相当（外部更新）を模して textContent を直接
    // 書き換える。`MutationObserver` がこれを検知し、現在値 (10) → 新値
    // (20) へ再補間を開始するはず。
    dd.set_text_content(Some("20"));

    sleep_ms(300).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "20",
        "外部更新後、新しい値へ再補間して収束するはず"
    );
}

#[wasm_bindgen_test]
async fn non_numeric_text_is_left_unchanged() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, dd) = build_dom(&document, "count-up-root-3", "N/A", &[]);
    let _guard = RemoveOnDrop(root.clone());

    wire_count_up(&root).expect("wire_count_up must not fail");

    sleep_ms(300).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "N/A",
        "数値として解析できないテキストは変更しないはず"
    );
}

/// PR #2580 codex-review P1 指摘の回帰テスト: 進行中の補間中に非数値
/// （"N/A" 等）へ外部更新された場合、補間が停止し古い数値で上書きし
/// 続けないこと。
#[wasm_bindgen_test]
async fn external_non_numeric_update_stops_active_interpolation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, dd) = build_dom(
        &document,
        "count-up-root-5",
        "1000",
        &[(COUNT_UP_DURATION_MS_ATTR, "5000")],
    );
    let _guard = RemoveOnDrop(root.clone());

    wire_count_up(&root).expect("wire_count_up must not fail");
    // duration を長く（5000ms）取り、確実に補間の途中で割り込む。
    sleep_ms(50).await;

    dd.set_text_content(Some("N/A"));
    sleep_ms(50).await;
    let after_update = dd.text_content().unwrap();
    assert_eq!(
        after_update, "N/A",
        "非数値への外部更新直後はそのまま反映されるはず"
    );

    // 進行中の補間が止まっていなければ、次の rAF で古い数値により
    // "N/A" が上書きされてしまう。
    sleep_ms(200).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "N/A",
        "非数値への外部更新後は進行中の補間が停止し、古い数値で上書きされ続けないはず"
    );
}

/// `in_view_browser.rs::create_scroll_fixture` と同型のスクロール
/// コンテナ方式で、対象要素をブラウザ既定ビューポート幅に依存せず
/// 決定的に非交差状態へ置く。
fn build_off_screen_dom(
    document: &Document,
    root_id: &str,
    text: &str,
    extra_attrs: &[(&str, &str)],
) -> (Element, HtmlElement) {
    let container = document.create_element("div").unwrap();
    container.set_id(root_id);
    container
        .set_attribute("style", "height:100px;overflow-y:auto")
        .unwrap();

    let top_spacer = document.create_element("div").unwrap();
    top_spacer.set_attribute("style", "height:300px").unwrap();
    container.append_child(&top_spacer).unwrap();

    let dd = document.create_element("dd").unwrap();
    dd.set_attribute(COUNT_UP_ATTR, "").unwrap();
    for (name, value) in extra_attrs {
        dd.set_attribute(name, value).unwrap();
    }
    dd.set_text_content(Some(text));
    container.append_child(&dd).unwrap();

    let bottom_spacer = document.create_element("div").unwrap();
    bottom_spacer
        .set_attribute("style", "height:300px")
        .unwrap();
    container.append_child(&bottom_spacer).unwrap();

    document.body().unwrap().append_child(&container).unwrap();
    let html_dd = dd
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("dd must cast to HtmlElement");
    (container, html_dd)
}

/// PR #2580 codex-review P1 再指摘の回帰テスト: in-view 待機中（画面外）に
/// 外部更新で目標値が変わった場合、待機中はその新しい書式の開始値 (0) を
/// 表示し続け、旧目標値の最終表示を残さないこと。残していると、後で画面内
/// へ進入した際 `start_count_up` が 0 起点で書き始めるため、最終値 → 0 の
/// ちらつきが起きる。
#[wasm_bindgen_test]
async fn in_view_pending_update_shows_new_start_value_not_stale_final() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (container, dd) = build_off_screen_dom(
        &document,
        "count-up-root-6",
        "1000",
        &[
            (COUNT_UP_TRIGGER_ATTR, COUNT_UP_TRIGGER_IN_VIEW),
            (COUNT_UP_DURATION_MS_ATTR, TEST_DURATION_MS),
        ],
    );
    let _guard = RemoveOnDrop(container.clone());

    wire_count_up(&container).expect("wire_count_up must not fail");
    sleep_ms(50).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "0",
        "画面外では開始値 (0) が即座に表示されるはず"
    );

    // 画面外のまま外部更新（アプリの `set_text` 相当）で目標値を変える。
    dd.set_text_content(Some("2,000"));
    sleep_ms(50).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "0",
        "画面外での外部更新後も、旧目標値の最終表示ではなく新しい書式の \
         開始値 (0) を表示し続けるはず（PR #2580 codex-review P1 再指摘）"
    );

    // ビューポート内へスクロールして交差させ、新しい目標値まで到達する
    // ことを確認する（ちらつきが起きても最終的な到達値は同じため、この
    // アサーションだけでは上のちらつき回帰を検知できない）。
    container.set_scroll_top(300);
    sleep_ms(300).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "2,000",
        "画面内進入後、待機中に更新された新しい目標値へ到達するはず"
    );
}

#[wasm_bindgen_test]
async fn in_view_trigger_eventually_reaches_final_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, dd) = build_dom(
        &document,
        "count-up-root-4",
        "50",
        &[
            (COUNT_UP_TRIGGER_ATTR, COUNT_UP_TRIGGER_IN_VIEW),
            (COUNT_UP_DURATION_MS_ATTR, TEST_DURATION_MS),
        ],
    );
    let _guard = RemoveOnDrop(root.clone());

    wire_count_up(&root).expect("wire_count_up must not fail");

    // ビューポート内に配置されるため（既定の headless Chrome ビューポート
    // 内）、IntersectionObserver は速やかに交差を報告し補間が始まるはず。
    sleep_ms(300).await;
    assert_eq!(
        dd.text_content().unwrap(),
        "50",
        "ビューポート内の要素は in-view トリガーでも最終的に値へ到達するはず"
    );
}
