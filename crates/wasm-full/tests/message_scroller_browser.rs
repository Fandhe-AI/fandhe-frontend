//! `fandhe_frontend_wasm_full::message_scroller`（イシュー #2122、親 #2120）の
//! 実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/tests/message_scroller_native.rs` は headless-ui 出力
//! とのドリフト検知を担う。本ファイルはその先、**実ブラウザ（headless
//! Chromium、`wasm-pack test --headless --chrome`）上での**最下部追従・
//! 新着検知・履歴読み込み時のスクロール位置維持の実 DOM 挙動を検証する
//! （`content_height_browser.rs`/`questionnaire_browser.rs` と同型の実 DOM
//! 検証パターンを踏襲する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el_owned, render, text, Node};
use fandhe_frontend_headless_ui::message_scroller::{
    content, jump_to_latest, load_more, root, viewport, MessageScrollerRootProps,
    MessageScrollerStuck as HeadlessStuck,
};
use fandhe_frontend_wasm_full::events::ActionRef;
use fandhe_frontend_wasm_full::message_scroller::{wire_message_scroller_events, ACTION_LOAD_MORE};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`content_height_browser.rs::create_container` と同型）。
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`content_height_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 固定高さの子要素（`style="height:{px}px"`）を組み立てる
/// （`content_height_browser.rs::fixed_height_child` と同型。`scrollHeight`
/// を決定的にするため）。
fn fixed_height_child(px: u32) -> Node {
    el_owned(
        "div",
        vec![("style".to_string(), format!("height:{px}px"))],
        vec![],
    )
}

/// `instance_id` の Message Scroller 1 インスタンスを組み立てる。viewport
/// は CSSOM で `height:100px; overflow-y:auto` を固定し、`scrollHeight`/
/// `scrollTop`/`clientHeight` を決定的にする。
fn build_message_scroller(instance_id: &str, stuck: HeadlessStuck, item_heights: &[u32]) -> Node {
    let items: Vec<Node> = item_heights
        .iter()
        .map(|px| fixed_height_child(*px))
        .collect();
    root(
        MessageScrollerRootProps {
            stuck,
            has_new: false,
        },
        vec![("id", instance_id)],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], items)],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    )
}

fn mount(container: &Element, node: &Node) -> Element {
    container.set_inner_html(&render(node));
    container
        .first_element_child()
        .expect("message-scroller root must exist")
}

fn find_viewport(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="viewport"]"#)
        .expect("query_selector must not fail")
        .expect("viewport must exist")
}

fn find_jump_to_latest(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="jump-to-latest"]"#)
        .expect("query_selector must not fail")
        .expect("jump-to-latest must exist")
}

fn find_load_more(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="load-more"]"#)
        .expect("query_selector must not fail")
        .expect("load-more must exist")
}

fn find_content(instance_root: &Element) -> Element {
    instance_root
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("content must exist")
}

fn overflow_anchor(viewport: &Element) -> String {
    viewport
        .dyn_ref::<HtmlElement>()
        .expect("viewport must be HtmlElement")
        .style()
        .get_property_value("overflow-anchor")
        .expect("get_property_value must not fail")
}

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

fn dispatch_click(target: &Element) {
    target
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
}

/// `viewport.scrollTop` を書き換えた上で合成 `scroll` イベントを送出する
/// （利用者スクロールの模擬。決定性のため合成イベントを使う）。
fn simulate_user_scroll(viewport: &Element, scroll_top: i32) {
    viewport.set_scroll_top(scroll_top);
    let init = EventInit::new();
    init.set_bubbles(false);
    let event = Event::new_with_event_init_dict("scroll", &init)
        .expect("Event::new must not fail for scroll");
    viewport
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

/// `condition` が真になるまでマイクロタスク/タイマー待機を繰り返す
/// （`headless_avatar_browser.rs::wait_for` と同型。`MutationObserver`
/// コールバックはマイクロタスクキューで実行されるため、同期的な DOM 変異
/// 直後には観測できない）。
async fn wait_for(mut condition: impl FnMut() -> bool) {
    use wasm_bindgen::closure::Closure;

    for _ in 0..200 {
        if condition() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&wasm_bindgen::JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10,
                )
                .expect("setTimeout must not fail");
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("timeout promise must resolve");
    }
}

// --- 初期同期 ---

#[wasm_bindgen_test]
async fn initial_sync_bottom_scrolls_to_bottom_and_sets_overflow_anchor_none() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-1", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    assert_eq!(viewport.scroll_top(), expected);
    assert_eq!(overflow_anchor(&viewport), "none");
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("hidden"));
    assert!(!jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn initial_sync_free_does_not_scroll() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-2", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    assert_eq!(
        viewport.scroll_top(),
        0,
        "free の初期状態はスクロールしないこと"
    );
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
    assert!(!jump.has_attribute("hidden"));
}

#[wasm_bindgen_test]
async fn initial_sync_unknown_stuck_value_does_not_scroll_fail_closed() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-initial-evil-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-evil", HeadlessStuck::Bottom, &[60, 60, 60]);
    container.set_inner_html(&render(&node));
    let instance_root = container
        .first_element_child()
        .expect("message-scroller root must exist");
    // 改ざんを模して data-stuck を不明値へ書き換える。
    instance_root
        .set_attribute("data-stuck", "evil")
        .expect("set_attribute must not fail");
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    assert_eq!(
        viewport.scroll_top(),
        0,
        "不明な data-stuck 値では自動スクロールしないこと（fail-closed）"
    );
}

// --- 利用者スクロール ---

#[wasm_bindgen_test]
async fn user_scroll_away_from_bottom_sets_free_and_shows_jump_to_latest() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-scroll-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-3", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
    assert!(!jump.has_attribute("hidden"));
}

#[wasm_bindgen_test]
async fn jump_to_latest_click_scrolls_to_bottom_and_restores_stuck() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-jump-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-4", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    dispatch_click(&jump);

    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
    let expected = viewport.scroll_height() - viewport.client_height();
    assert_eq!(viewport.scroll_top(), expected);
    assert!(jump.has_attribute("hidden"));
    assert!(!jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn jump_to_latest_click_is_noop_when_disabled_ancestor() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-jump-disabled-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-5", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 0);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let jump = find_jump_to_latest(&instance_root);
    jump.set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");
    dispatch_click(&jump);

    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free"),
        "disabled 祖先を持つ jump-to-latest クリックは no-op であること"
    );
}

// --- 新着検知 ---

/// 空リスト（`content` 配下に既存の子ノードが無い状態）への初回追加は、
/// 追加ノードに `previousSibling` が無いため構造的には先頭挿入と
/// 見分けがつかないが、`nextSibling` も無い（前後どちらにも既存ノードが
/// 無い）ことから初期成長（Grow）として扱われるべきである。Prepend と
/// 誤判定されると、Free 状態で viewport より高い新着メッセージを追加
/// したときに位置補正でスクロール位置が動き、かつ `data-has-new` が
/// 付与されず新着通知の契約に反する（codex-review 指摘 #2122 line 789）。
#[wasm_bindgen_test]
async fn append_to_empty_list_while_free_marks_has_new_not_prepend() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-append-empty-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 初期状態は 1 件もメッセージを持たない空リスト。
    let node = build_message_scroller("ms-empty-1", HeadlessStuck::Free, &[]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let before_top = viewport.scroll_top();
    // viewport（height:100px）より高い新着メッセージを追加し、Prepend と
    // 誤判定された場合の位置補正が確実に発生する条件を作る。
    let new_child = render(&fixed_height_child(200));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "空リストへの初回追加は Grow として data-has-new を付与すること"
    );
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "空リストへの初回追加を Prepend と誤判定して scrollTop を動かさないこと"
    );
}

#[wasm_bindgen_test]
async fn append_while_free_marks_has_new_without_scrolling() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-append-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-6", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let before_top = viewport.scroll_top();
    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(instance_root.has_attribute("data-has-new"));
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "free での末尾追記は scrollTop を変えないこと"
    );
    let jump = find_jump_to_latest(&instance_root);
    assert!(jump.has_attribute("data-visible"));
}

#[wasm_bindgen_test]
async fn append_while_bottom_follows_to_new_bottom_without_has_new() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-append-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-7", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(viewport.scroll_top(), expected);
    assert!(!instance_root.has_attribute("data-has-new"));
}

// --- 履歴読み込み時の位置維持（先頭挿入） ---

#[wasm_bindgen_test]
async fn prepend_while_free_preserves_visual_position() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-prepend-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-8", HeadlessStuck::Free, &[60, 60, 60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    // free のまま、利用者が少し下へスクロールした状態を作る。
    simulate_user_scroll(&viewport, 40);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free")
    );
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // 先頭へ 2 件（各 40px）挿入する。
    let prepend_html = format!(
        "{}{}",
        render(&fixed_height_child(40)),
        render(&fixed_height_child(40))
    );
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 80;
    wait_for(|| viewport.scroll_height() == before_height + expected_delta).await;
    assert_eq!(viewport.scroll_height(), before_height + expected_delta);
    wait_for(|| viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(viewport.scroll_top(), before_top + expected_delta);
}

#[wasm_bindgen_test]
async fn prepend_while_bottom_stays_at_bottom_after_correction() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-prepend-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-9", HeadlessStuck::Bottom, &[60, 60, 60]);
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let prepend_html = render(&fixed_height_child(50));
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(viewport.scroll_top(), expected);
    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
}

#[wasm_bindgen_test]
async fn streaming_text_replacement_is_classified_as_grow_not_prepend() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-stream-text-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // ストリーミング本文を表示する束縛点（`bind_text` が
    // `set_text_content` で書き換える対象）を模した、初期状態が空の
    // block 要素を 1 件持つメッセージを組み立てる。`line-height` を固定
    // することで、テキスト追加前後の高さ変化（0px → 20px）をフォントに
    // 依存せず決定的にする。
    let body_span = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px".to_string(),
        )],
        vec![],
    );
    let message_item = el_owned("div", vec![], vec![body_span]);
    // viewport（100px）を既存の固定高さ項目（90px）でほぼ埋めておき、
    // ストリーミング本文の増分（0px → 20px）で `scrollHeight` が
    // `clientHeight`（100px）を超えて実際に増加するようにする
    // （`scrollHeight` は `clientHeight` を下回らないため、埋めずに
    // 増分だけでは高さ変化が観測できない）。
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-stream-text")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![fixed_height_child(90), message_item])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let body_el = content_el
        .query_selector("span")
        .expect("query_selector must not fail")
        .expect("body span must exist");

    // `fandhe-frontend-wasm-client::binding_dom::apply_one`（`bind_text`）
    // と同じ DOM API（`Node::set_text_content`）で本文を書き換える。この
    // `childList` レコードは既存の子を丸ごと入れ替えるため、追加された
    // 唯一の子ノードは `previousSibling` を持たない（旧実装がこれを
    // 先頭挿入＝Prepend と誤判定していたケース、レビュー指摘 #2122:
    // codex-review P1 / Cursor Bugbot 双方）。
    let viewport = find_viewport(&instance_root);
    body_el.set_text_content(Some("Hello"));

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "本文テキストの置換が誤って Prepend 判定されず、Grow として \
         data-has-new が立つこと"
    );
    // Prepend と誤判定されていれば `correct_prepend` によるスクロール
    // 補正が働き、Free でも scrollTop が動く。Grow として正しく分類
    // されていれば Free 状態は自動スクロールしないため、scrollTop は
    // 不変（0）のままであることも確認する。
    assert_eq!(viewport.scroll_top(), 0);
}

#[wasm_bindgen_test]
async fn prepend_into_keyed_list_wrapper_inside_content_is_detected() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-keyed-list-prepend-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // `docs/design/wasm-full-architecture.md` §31.8 がサポート経路とする
    // 「`content` 配下を keyed list（`fandhe_frontend_core::keyed::
    // keyed_list`）で差分更新する」構成を再現する。`keyed_list` の出力は
    // `content` 自身ではなく、`data-bind-list` を持つ子要素（リストの
    // 親）を挟む。真の先頭挿入はこの `data-bind-list` 要素への
    // `insertBefore` として観測されるため、`content` 自身への挿入のみを
    // 対象とする実装ではこのケースを取りこぼす（先頭挿入が誤って Grow
    // 扱いになる）。
    let list = keyed_list(
        "div",
        vec![],
        "messages",
        vec![
            ("a".to_string(), fixed_height_child(60)),
            ("b".to_string(), fixed_height_child(60)),
            ("c".to_string(), fixed_height_child(60)),
        ],
    )
    .expect("keyed_list must not fail for well-formed items");
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-keyed-list-prepend")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![list])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let viewport = find_viewport(&instance_root);
    let content_el = find_content(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    // free のまま、利用者が少し下へスクロールした状態を作る
    // （`prepend_while_free_preserves_visual_position` と同じ検証形）。
    simulate_user_scroll(&viewport, 40);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    let list_wrapper = content_el
        .query_selector("[data-bind-list]")
        .expect("query_selector must not fail")
        .expect("data-bind-list wrapper must exist");
    let prepend_html = render(&fixed_height_child(40));
    list_wrapper
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 40;
    wait_for(|| viewport.scroll_height() == before_height + expected_delta).await;
    assert_eq!(viewport.scroll_height(), before_height + expected_delta);
    // `MutationObserver` コールバックはマイクロタスクで走るため、高さの
    // 反映（同期的な DOM 更新）と `scrollTop` 補正の反映（非同期）とで
    // 別々に `wait_for` する必要がある（上の高さの `wait_for` はマイクロ
    // タスクを待たずに真になり得るため、`scrollTop` の `wait_for` を
    // 省略すると補正の反映前に読んでしまう。
    // `prepend_while_free_preserves_visual_position` と同型）。
    wait_for(|| viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(
        viewport.scroll_top(),
        before_top + expected_delta,
        "keyed_list の data-bind-list 要素への先頭挿入が Prepend として \
         検知され、scrollTop が高さ増分だけ補正されること"
    );
}

// --- load-more 通知 ---

#[wasm_bindgen_test]
async fn load_more_click_notifies_app_with_instance_id_payload() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-load-more-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-10", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(instance_root.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail");

    let load_more = find_load_more(&instance_root);
    dispatch_click(&load_more);

    assert_eq!(received.borrow().len(), 1);
    let action_ref = received.borrow()[0].clone();
    assert_eq!(action_ref.action, ACTION_LOAD_MORE);
    assert_eq!(action_ref.payload, "ms-10");
}

#[wasm_bindgen_test]
async fn load_more_click_is_noop_when_loading_or_disabled() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-load-more-loading-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-11", HeadlessStuck::Free, &[60, 60, 60]);
    let instance_root = mount(&container, &node);

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(instance_root.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail");

    let load_more = find_load_more(&instance_root);
    load_more
        .set_attribute("data-loading", "")
        .expect("set_attribute must not fail");
    dispatch_click(&load_more);
    assert!(
        received.borrow().is_empty(),
        "data-loading 中は通知しないこと"
    );

    load_more
        .remove_attribute("data-loading")
        .expect("remove_attribute must not fail");
    load_more
        .set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");
    dispatch_click(&load_more);
    assert!(
        received.borrow().is_empty(),
        "data-disabled 中は通知しないこと"
    );
}

// --- 非搭載アプリへの副作用なし ---

#[wasm_bindgen_test]
async fn wiring_is_noop_when_no_message_scroller_instance_present() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-no-instance-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let plain = el_owned("div", vec![], vec![text("no message-scroller here")]);
    container.set_inner_html(&render(&plain));
    let root_el = container
        .first_element_child()
        .expect("plain root must exist");

    let received: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();
    wire_message_scroller_events(root_el.clone(), move |action_ref: ActionRef| {
        received_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_message_scroller_events must not fail on non-instrumented root");

    dispatch_click(&root_el);
    assert!(received.borrow().is_empty());
}

// --- 複数インスタンスの独立性 ---

#[wasm_bindgen_test]
async fn two_instances_in_same_container_are_independent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-two-instances-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let wrapper = el_owned(
        "div",
        vec![],
        vec![
            build_message_scroller("ms-a", HeadlessStuck::Bottom, &[60, 60, 60]),
            build_message_scroller("ms-b", HeadlessStuck::Free, &[60, 60, 60]),
        ],
    );
    container.set_inner_html(&render(&wrapper));
    let outer = container.first_element_child().expect("wrapper must exist");

    wire_message_scroller_events(outer.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let instance_a = outer
        .query_selector("#ms-a")
        .expect("query_selector must not fail")
        .expect("ms-a must exist");
    let instance_b = outer
        .query_selector("#ms-b")
        .expect("query_selector must not fail")
        .expect("ms-b must exist");
    let content_b = find_content(&instance_b);

    // b（free）へ追記しても a（bottom）の data-* へ波及しないこと。
    let new_child = render(&fixed_height_child(40));
    content_b
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_b.has_attribute("data-has-new")).await;
    assert!(instance_b.has_attribute("data-has-new"));
    assert!(
        !instance_a.has_attribute("data-has-new"),
        "片方への追記がもう片方の data-* へ波及しないこと"
    );
    assert_eq!(
        instance_a.get_attribute("data-stuck").as_deref(),
        Some("bottom")
    );
}

// --- ネストした message-scroller インスタンスの分離（Bugbot 指摘 #2122） ---

/// `outer_id`/`outer_stuck` の外側インスタンス `content` 配下に、
/// `inner_id`/`inner_stuck` の内側インスタンスを 1 件そのまま埋め込んだ
/// ネスト構造を組み立てる（`build_message_scroller` の入れ子版）。
fn build_nested_message_scroller(
    outer_id: &str,
    outer_stuck: HeadlessStuck,
    inner_id: &str,
    inner_stuck: HeadlessStuck,
) -> Node {
    let inner = build_message_scroller(inner_id, inner_stuck, &[60, 60, 60]);
    root(
        MessageScrollerRootProps {
            stuck: outer_stuck,
            has_new: false,
        },
        vec![("id", outer_id)],
        vec![
            viewport(
                "",
                vec![("style", "height:200px;overflow-y:auto")],
                vec![content(vec![], vec![fixed_height_child(60), inner])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    )
}

#[wasm_bindgen_test]
async fn nested_instance_append_does_not_leak_into_outer_classification() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-nested-append-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_nested_message_scroller(
        "ms-nested-outer",
        HeadlessStuck::Bottom,
        "ms-nested-inner",
        HeadlessStuck::Free,
    );
    container.set_inner_html(&render(&node));
    let outer_el = container
        .first_element_child()
        .expect("outer message-scroller root must exist");

    wire_message_scroller_events(outer_el.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let inner_el = outer_el
        .query_selector("#ms-nested-inner")
        .expect("query_selector must not fail")
        .expect("ms-nested-inner must exist");
    let inner_content = find_content(&inner_el);

    // 内側（free）の content への追記は、内側自身の data-has-new のみを
    // 立て、外側（bottom）の分類（data-stuck・data-has-new）へ波及しない
    // こと（Bugbot 指摘 #2122: `first_added_element_top` の
    // `content.contains(target)` 判定のみではネストしたインスタンスの
    // `content` への挿入が外側の分類に漏れ込んでいた）。
    let new_child = render(&fixed_height_child(20));
    inner_content
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| inner_el.has_attribute("data-has-new")).await;
    assert!(inner_el.has_attribute("data-has-new"));
    assert!(
        !outer_el.has_attribute("data-has-new"),
        "内側インスタンスへの追記が外側の data-has-new へ波及しないこと"
    );
    assert_eq!(
        outer_el.get_attribute("data-stuck").as_deref(),
        Some("bottom"),
        "内側インスタンスへの追記で外側の data-stuck が変化しないこと"
    );
}

#[wasm_bindgen_test]
async fn nested_instance_prepend_does_not_leak_into_outer_grow_classification() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-nested-prepend-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_nested_message_scroller(
        "ms-nested-outer-2",
        HeadlessStuck::Free,
        "ms-nested-inner-2",
        HeadlessStuck::Free,
    );
    container.set_inner_html(&render(&node));
    let outer_el = container
        .first_element_child()
        .expect("outer message-scroller root must exist");

    wire_message_scroller_events(outer_el.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let inner_el = outer_el
        .query_selector("#ms-nested-inner-2")
        .expect("query_selector must not fail")
        .expect("ms-nested-inner-2 must exist");
    let inner_content = find_content(&inner_el);
    let outer_content = find_content(&outer_el);

    // 同一 MutationObserver バッチ内で、先に内側 content の先頭へ挿入
    // （内側自身は Prepend）し、続けて外側 content 末尾へ追記（外側自身は
    // Grow）する。内側の先頭挿入が record_list の先頭に記録されるため、
    // ネスト除外（`closest_matching(content, target, PART_ROOT)`）を欠いた
    // 旧実装（`content.contains(target)` のみの判定）では外側の分類
    // ループが record_list を先頭から走査した際に内側の Prepend 判定へ
    // 誤って引きずられ、外側自身の Grow（Free なら data-has-new 付与）が
    // 抑止されてしまう（Bugbot 指摘 #2122 line 722）。
    let prepend_html = render(&fixed_height_child(20));
    inner_content
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");
    let append_html = render(&fixed_height_child(20));
    outer_content
        .insert_adjacent_html("beforeend", &append_html)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| outer_el.has_attribute("data-has-new")).await;
    assert!(
        outer_el.has_attribute("data-has-new"),
        "外側自身の末尾追記（Grow）が内側の先頭挿入（Prepend）に引きずられず data-has-new を立てること"
    );
}
