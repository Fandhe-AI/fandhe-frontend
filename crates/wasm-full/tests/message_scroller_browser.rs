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

/// 会話の本文表示要素（`span`、`white-space: pre` + 明示的な改行文字で
/// `appendData`/`set_text_content` 前後の高さ変化をフォントメトリクスに
/// 依存せず決定的にする）と、その手前の固定高さ項目（90px、viewport
/// 100px をほぼ埋めて `scrollHeight` が `clientHeight` を超えて実際に
/// 増加するようにする、`streaming_text_replacement_is_classified_as_grow_not_prepend`
/// と同じ配慮）を持つメッセージ 1 件を組み立てる（会話リスト本体レベルの
/// 変更〔本文 span 経由〕をテストするための共有フィクスチャ）。
fn body_only_message_item() -> Node {
    let body_span = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("Hello")],
    );
    el_owned("div", vec![], vec![fixed_height_child(90), body_span])
}

/// [`body_only_message_item`] に加え、それ自身も `data-bind-list`
/// を持つネストした keyed list（`attachments`、項目 1 件の `span`）を
/// 持つメッセージ 1 件を組み立てる。会話リスト本体レベルの変更（本文
/// `span` 経由）と、会話リストより深いネストした二次的リストレベルの
/// 変更（添付 `span` 経由）を、同一メッセージ構造の中で区別してテスト
/// するための共有フィクスチャ（コーディネータ指摘、PR #2312: フラット/
/// ラップ × 経路〔childList 先頭/末尾追加・`set_text_content`・
/// `characterData`〕× 位置〔会話リスト直下/ネスト `attachments` 内〕の
/// テストマトリクス）。
fn message_item_with_attachment() -> Node {
    let attachment_label = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("attachment.pdf")],
    );
    let attachments = keyed_list(
        "div",
        vec![],
        "attachments",
        vec![("att-1".to_string(), attachment_label)],
    )
    .expect("keyed_list must not fail for well-formed items");
    let body_span = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("Hello")],
    );
    el_owned(
        "div",
        vec![],
        vec![fixed_height_child(90), body_span, attachments],
    )
}

/// `content` 自身が `data-bind-list="messages"` を持つ**フラット構成**
/// （`keyed_list()` は新規ラッパー要素を生成するため使えず、`content`
/// 自身へ直接属性を付与して手組みする）で、`content_children`（会話
/// リスト本体の直下に置くメッセージ要素群）を持つ scroller を組み立てる。
fn flat_layout_scroller(id: &str, stuck: HeadlessStuck, content_children: Vec<Node>) -> Node {
    root(
        MessageScrollerRootProps {
            stuck,
            has_new: false,
        },
        vec![("id", id)],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(
                    vec![("data-bind-list", "messages")],
                    content_children,
                )],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    )
}

/// `content` → `data-bind-list="messages"` の keyed_list を持つ**ラップ
/// 構成**で、`message_item`（会話リスト本体の唯一のメッセージ要素）を
/// 持つ scroller を組み立てる。
fn wrapped_layout_scroller(id: &str, stuck: HeadlessStuck, message_item: Node) -> Node {
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), message_item)],
    )
    .expect("keyed_list must not fail for well-formed items");
    root(
        MessageScrollerRootProps {
            stuck,
            has_new: false,
        },
        vec![("id", id)],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages])],
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

/// Promise マイクロタスクを 1 回消化するまで待つ（`headless_avatar_browser.rs::
/// microtask_tick` と同型）。`insert_adjacent_html` 等の同期的な DOM 変異は
/// `MutationRecord` を同期的にマイクロタスクキューへ積むため、本関数が作る
/// `Promise::resolve` の `then` コールバックはそれより後のマイクロタスクと
/// して実行される。よって本関数から復帰した時点で `handle_mutations`
/// コールバックの実行は完了していることが保証される（「何も起きない」
/// ことを確認する回帰テストで、固定 `sleep` に頼らず決定的に待機する）。
async fn microtask_tick() {
    let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("microtask promise must resolve");
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

/// codex-review 指摘（PR #2312、`message_scroller.rs:633`）の回帰テスト:
/// `data-disabled` が `jump` 自身ではなくインスタンス root（または root と
/// `jump` の間のラッパー）に付いている場合でも、load-more と同じく
/// クリックは no-op であるべき（`handle_click` の境界解決を `jump` から
/// `instance_root` へ変更した修正の検証）。
#[wasm_bindgen_test]
async fn jump_to_latest_click_is_noop_when_instance_root_disabled() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-jump-root-disabled-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_message_scroller("ms-5-root-disabled", HeadlessStuck::Bottom, &[60, 60, 60]);
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

    // `jump` 自身ではなく instance root へ `data-disabled` を付与する
    // （旧実装は境界に `jump` を渡していたため、この位置の祖先を見逃す）。
    instance_root
        .set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");

    let jump = find_jump_to_latest(&instance_root);
    dispatch_click(&jump);

    assert_eq!(
        instance_root.get_attribute("data-stuck").as_deref(),
        Some("free"),
        "instance root に data-disabled がある場合の jump-to-latest クリックは no-op であること"
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

/// `streaming_text_replacement_is_classified_as_grow_not_prepend` は
/// `set_text_content`（子ノードを丸ごと入れ替える `childList` レコード）
/// 経由のストリーミング更新を検証するが、既存 `Text` ノードへの
/// `nodeValue`/`appendData` による更新は `characterData` レコードとして
/// 観測され、`target` が `Text` ノードになる別経路である
/// （codex-review 指摘 PR #2312、`first_relevant_change`）。
/// `Bottom` 状態でこの経路の高さ増加が最下部追従を発火させることを
/// 固定する（`characterData` レコードが `Element` 判定で除外されたままだと
/// 何も起こらない）。
///
/// §31.8 がサポート経路とする正規のレイアウト（`content` →
/// `data-bind-list="messages"` → メッセージ要素 → `span` → `Text`）を
/// 使う: 会話リスト本体（`messages`）を経由した本文ストリーミングが、
/// 会話リスト自身を「ネストした bind-list」と誤検出して除外されない
/// ことを固定する（`data-bind-list` ラッパーを持たないレイアウトでは
/// この不具合を検出できなかった、codex P1 / Cursor Bugbot 再指摘
/// PR #2312）。
#[wasm_bindgen_test]
async fn character_data_streaming_update_follows_to_bottom_when_stuck_bottom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-character-data-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // `white-space: pre` + 明示的な改行文字で、フォントメトリクスに依存
    // せず `appendData` 前後の高さ変化（+20px）を決定的にする
    // （`fixed_height_child` と同じ「CSS で決定的にする」方針）。
    let body_span = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("Hello")],
    );
    let message_item = el_owned("div", vec![], vec![fixed_height_child(90), body_span]);
    // `content` → `data-bind-list="messages"` → メッセージ要素の正規の
    // keyed_list レイアウト（`prepend_into_keyed_list_wrapper_inside_content_is_detected`
    // と同じ構成）。
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), message_item)],
    )
    .expect("keyed_list must not fail for well-formed items");
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Bottom,
            has_new: false,
        },
        vec![("id", "ms-character-data-bottom")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let body_span_el = content_el
        .query_selector("span")
        .expect("query_selector must not fail")
        .expect("body span must exist");
    let text_node = body_span_el
        .first_child()
        .expect("body span must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("body span's first child must be a Text node");

    // `set_text_content`（`childList`）ではなく、既存 `Text` ノードの
    // `CharacterData::append_data` で更新する（`characterData` レコードを
    // 直接発火させる、`bind_text` の `set_text_content` 経路とは別の
    // ストリーミング更新経路）。
    text_node
        .append_data("\nWorld")
        .expect("append_data must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(
        viewport.scroll_top(),
        expected,
        "会話リスト（data-bind-list=\"messages\"）配下の characterData に \
         よるストリーミング更新で最下部へ追従すること"
    );
    assert!(!instance_root.has_attribute("data-has-new"));
}

/// 上記の `Free` 版: `characterData` レコード経由のストリーミング更新が
/// `data-has-new` を付与し、`scrollTop` は変化しないこと
/// （codex-review 指摘 PR #2312、`first_relevant_change`）。正規の
/// keyed_list レイアウト（`content` → `data-bind-list="messages"` →
/// メッセージ要素 → `span` → `Text`）を使う（上記 Bottom 版と同じ理由）。
#[wasm_bindgen_test]
async fn character_data_streaming_update_marks_has_new_when_stuck_free() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-character-data-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let body_span = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("Hello")],
    );
    // viewport（100px）を既存の固定高さ項目（90px）でほぼ埋めておき、
    // ストリーミング本文の増分（+20px）で `scrollHeight` が
    // `clientHeight`（100px）を超えて実際に増加するようにする
    // （Bottom 版・`streaming_text_replacement_is_classified_as_grow_not_prepend`
    // と同じ配慮）。
    let message_item = el_owned("div", vec![], vec![fixed_height_child(90), body_span]);
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), message_item)],
    )
    .expect("keyed_list must not fail for well-formed items");
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-character-data-free")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let body_span_el = content_el
        .query_selector("span")
        .expect("query_selector must not fail")
        .expect("body span must exist");
    let text_node = body_span_el
        .first_child()
        .expect("body span must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("body span's first child must be a Text node");

    let before_top = viewport.scroll_top();
    text_node
        .append_data("\nWorld")
        .expect("append_data must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "characterData によるストリーミング更新は free で data-has-new を \
         立てること"
    );
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "characterData によるストリーミング更新で free の scrollTop が \
         変化しないこと"
    );
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

/// Bugbot 指摘（PR #2312、`message_scroller.rs:772-784`）の回帰テスト:
/// メッセージ 1 件の内部にネストした keyed list（添付・リアクション・
/// ツールステップ等、それ自身も `data-bind-list` を持つ）への挿入は、
/// たとえその挿入が先頭位置（`previousSibling` 無し）であっても、会話
/// リスト本体（`content` 直下の最も浅い `data-bind-list`）への履歴先頭
/// 挿入とは扱わず、`scrollTop` 補正・`data-has-new` 付与のいずれも
/// 行わないこと（旧実装は任意の `data-bind-list` 子孫を会話リストと
/// みなし、この操作を Prepend と誤判定して `scrollTop` をずらしていた）。
#[wasm_bindgen_test]
async fn prepend_into_nested_bind_list_inside_message_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-nested-bind-list-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 会話リスト本体（`messages`）1 件のメッセージが、それ自身の内部に
    // ネストした keyed list（`attachments`）を 1 件持つ構成を再現する。
    let attachments = keyed_list(
        "div",
        vec![],
        "attachments",
        vec![("att-1".to_string(), fixed_height_child(20))],
    )
    .expect("keyed_list must not fail for well-formed items");
    // viewport（100px）を既存の固定高さ項目（90px）でほぼ埋めておき、
    // 添付リストへの追加分（30px）で `scrollHeight` が `clientHeight`
    // （100px）を超えて実際に増加するようにする（`scrollHeight` は
    // `clientHeight` を下回らないため、埋めずに増分だけでは高さ変化が
    // 観測できない。`streaming_text_replacement_is_classified_as_grow_not_prepend`
    // と同じ配慮）。
    let message_item = el_owned("div", vec![], vec![fixed_height_child(90), attachments]);
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), message_item)],
    )
    .expect("keyed_list must not fail for well-formed items");
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-nested-bind-list")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages])],
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

    // free のまま、利用者が少し下へスクロールした状態を作る（古い
    // メッセージを閲覧中を模す。`prepend_into_keyed_list_wrapper_inside_
    // content_is_detected` と同じ検証形）。
    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // 会話リスト（`messages`）ではなく、メッセージ内部にネストした
    // `attachments` リストへ、先頭位置（`previousSibling` 無し）で
    // 追加する。旧実装はこの `data-bind-list` を会話リストと誤認し、
    // Prepend として `scrollTop` を補正していた。
    let attachments_wrapper = content_el
        .query_selector("[data-bind-list=\"attachments\"]")
        .expect("query_selector must not fail")
        .expect("nested attachments data-bind-list wrapper must exist");
    let new_attachment = render(&fixed_height_child(30));
    attachments_wrapper
        .insert_adjacent_html("afterbegin", &new_attachment)
        .expect("insert_adjacent_html must not fail");

    // `insert_adjacent_html` による高さ変化は同期的に反映されるため、
    // `wait_for`（「真になるまで待つ」用途）ではなく `microtask_tick`
    // （`handle_mutations` コールバックの実行完了を保証する）を使って
    // 「何も起きない」ことを検証する。
    let expected_height = before_height + 30;
    assert_eq!(viewport.scroll_height(), expected_height);
    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "ネストした data-bind-list（会話リストより深い）への先頭挿入で \
         scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "ネストした data-bind-list への追加が data-has-new を \
         立てないこと"
    );
}

/// `prepend_into_nested_bind_list_inside_message_is_ignored`（childList
/// 経路）の characterData 版: メッセージ内部にネストした `attachments`
/// リスト内のテキスト（`Text` ノード）を `appendData` で更新しても、
/// 会話リスト（`messages`）レベルの `scrollTop` 補正・`data-has-new` 付与
/// のいずれも起きないこと。`is_within_nested_bind_list` は
/// childList 経路・characterData 経路の両方から同じ判定関数を使うため、
/// 本テストは両経路の整合を固定する（コーディネータ指摘、PR #2312）。
#[wasm_bindgen_test]
async fn character_data_update_inside_nested_bind_list_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-character-data-nested-bind-list-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 添付 1 件のテキストラベル（`white-space: pre` + 明示的な改行文字で
    // `appendData` 前後の高さ変化を決定的にする）。
    let attachment_label = el_owned(
        "span",
        vec![(
            "style".to_string(),
            "display:block;line-height:20px;white-space:pre".to_string(),
        )],
        vec![text("attachment.pdf")],
    );
    let attachments = keyed_list(
        "div",
        vec![],
        "attachments",
        vec![("att-1".to_string(), attachment_label)],
    )
    .expect("keyed_list must not fail for well-formed items");
    // viewport（100px）を既存の固定高さ項目（90px）でほぼ埋めておき、
    // `appendData` の増分（+20px）で `scrollHeight` が `clientHeight`
    // （100px）を超えて実際に増加するようにする（他の characterData
    // テストと同じ配慮）。
    let message_item = el_owned("div", vec![], vec![fixed_height_child(90), attachments]);
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), message_item)],
    )
    .expect("keyed_list must not fail for well-formed items");
    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-character-data-nested-bind-list")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages])],
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

    // free のまま、利用者が少し下へスクロールした状態を作る（古い
    // メッセージを閲覧中を模す）。
    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // 会話リスト（`messages`）ではなく、メッセージ内部にネストした
    // `attachments` リスト内のテキストを `appendData` で更新する。
    let label_el = content_el
        .query_selector("[data-bind-list=\"attachments\"] span")
        .expect("query_selector must not fail")
        .expect("attachment label span must exist");
    let text_node = label_el
        .first_child()
        .expect("attachment label must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("attachment label's first child must be a Text node");
    text_node
        .append_data("\n(uploaded)")
        .expect("append_data must not fail");

    let expected_height = before_height + 20;
    assert_eq!(viewport.scroll_height(), expected_height);
    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "ネストした data-bind-list 内の characterData 更新で scrollTop が \
         変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "ネストした data-bind-list 内の characterData 更新が data-has-new \
         を立てないこと"
    );
}

// --- フラット構成（content 自身が会話リスト本体） × 経路 × 位置の
// テストマトリクス（コーディネータ指摘、PR #2312 再指摘 3 件の根本原因
// 是正）。ラップ構成（`content` → `data-bind-list="messages"`）・
// リスト無し構成の同種テストは本ファイル前半に既存（
// `prepend_into_keyed_list_wrapper_inside_content_is_detected`・
// `prepend_into_nested_bind_list_inside_message_is_ignored`・
// `character_data_streaming_update_*`・`character_data_update_inside_nested_bind_list_is_ignored`・
// `streaming_text_replacement_is_classified_as_grow_not_prepend`・
// `append_while_*`・`prepend_while_*`）。本節はそれらと組み合わさって
// 「レイアウト {フラット/ラップ} × 経路 {childList 先頭/末尾追加・
// `set_text_content`・`characterData`} × 位置 {会話リスト直下/ネスト
// `attachments` 内}」の全マトリクスを固定する。 ---

#[wasm_bindgen_test]
async fn flat_layout_conversation_append_follows_to_bottom_when_stuck_bottom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-append-bottom-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-append-bottom",
        HeadlessStuck::Bottom,
        vec![fixed_height_child(60), fixed_height_child(60)],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    // `content` 自身が会話リスト本体（`data-bind-list="messages"`）の
    // ため、`content` への末尾追記が childList の target = 会話リスト
    // 本体そのものになる（フラット構成でのみ生じる経路）。
    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    let expected = viewport.scroll_height() - viewport.client_height();
    wait_for(|| viewport.scroll_top() == expected).await;
    assert_eq!(
        viewport.scroll_top(),
        expected,
        "フラット構成（content 自身が会話リスト）で末尾追記した場合に \
         Bottom で最下部へ追従すること"
    );
    assert!(!instance_root.has_attribute("data-has-new"));
}

#[wasm_bindgen_test]
async fn flat_layout_conversation_append_marks_has_new_when_stuck_free() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-append-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-append-free",
        HeadlessStuck::Free,
        vec![fixed_height_child(60), fixed_height_child(60)],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let before_top = viewport.scroll_top();
    let new_child = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("beforeend", &new_child)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "フラット構成で末尾追記した場合に free で data-has-new を \
         立てること"
    );
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "フラット構成での末尾追記は free の scrollTop を変えないこと"
    );
}

#[wasm_bindgen_test]
async fn flat_layout_conversation_prepend_preserves_position_when_stuck_free() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-prepend-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-prepend-free",
        HeadlessStuck::Free,
        vec![
            fixed_height_child(60),
            fixed_height_child(60),
            fixed_height_child(60),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 40);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // `content` 自身（会話リスト本体）への先頭挿入。
    let prepend_html = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 40;
    wait_for(|| viewport.scroll_height() == before_height + expected_delta).await;
    wait_for(|| viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(
        viewport.scroll_top(),
        before_top + expected_delta,
        "フラット構成での会話リスト本体（content 自身）への先頭挿入が \
         Prepend として検知され、scrollTop が高さ増分だけ補正されること"
    );
}

#[wasm_bindgen_test]
async fn flat_layout_text_content_replace_is_classified_as_grow_not_prepend() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-text-replace-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-text-replace",
        HeadlessStuck::Free,
        vec![body_only_message_item()],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let body_span = content_el
        .query_selector("span")
        .expect("query_selector must not fail")
        .expect("body span must exist");
    body_span.set_text_content(Some("Hello\nWorld"));

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "フラット構成での本文テキスト置換が Grow として data-has-new を \
         立てること（会話リスト本体〔content 自身〕への挿入＝Prepend と \
         誤判定しないこと）"
    );
    assert_eq!(viewport.scroll_top(), 0);
}

#[wasm_bindgen_test]
async fn flat_layout_character_data_update_marks_has_new_when_stuck_free() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-character-data-free-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-character-data-free",
        HeadlessStuck::Free,
        vec![body_only_message_item()],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let body_span_el = content_el
        .query_selector("span")
        .expect("query_selector must not fail")
        .expect("body span must exist");
    let text_node = body_span_el
        .first_child()
        .expect("body span must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("body span's first child must be a Text node");

    let before_top = viewport.scroll_top();
    text_node
        .append_data("\nWorld")
        .expect("append_data must not fail");

    wait_for(|| instance_root.has_attribute("data-has-new")).await;
    assert!(
        instance_root.has_attribute("data-has-new"),
        "フラット構成での characterData によるストリーミング更新は free \
         で data-has-new を立てること"
    );
    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "フラット構成での characterData によるストリーミング更新で free \
         の scrollTop が変化しないこと"
    );
}

/// 根本原因の再現テスト（コーディネータ指摘 #2/#3）: フラット構成
/// （`content` 自身が `data-bind-list="messages"`）で、旧実装の 2 段階
/// 祖先歩行ヒューリスティックは第 2 段階の探索が `content` に到達した
/// 時点で `content` 自身の属性を確認せず打ち切っていたため、メッセージ
/// 内部にネストした `attachments` を会話リスト本体と誤認していた。
#[wasm_bindgen_test]
async fn flat_layout_nested_bind_list_prepend_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-nested-bind-list-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-nested-bind-list",
        HeadlessStuck::Free,
        vec![message_item_with_attachment()],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    let attachments_wrapper = content_el
        .query_selector("[data-bind-list=\"attachments\"]")
        .expect("query_selector must not fail")
        .expect("nested attachments data-bind-list wrapper must exist");
    let new_attachment = render(&fixed_height_child(30));
    attachments_wrapper
        .insert_adjacent_html("afterbegin", &new_attachment)
        .expect("insert_adjacent_html must not fail");

    let expected_height = before_height + 30;
    assert_eq!(viewport.scroll_height(), expected_height);
    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "フラット構成でもネストした data-bind-list への先頭挿入で \
         scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "フラット構成でもネストした data-bind-list への追加が \
         data-has-new を立てないこと"
    );
}

/// 根本原因の再現テスト（コーディネータ指摘 #1）: childList 経路で
/// target が `data-bind-list` を持たない要素（ネストした `attachments`
/// リスト内の `span`）を `set_text_content` で更新した場合、旧実装は
/// 「`data-bind-list` を持たない target」の分岐（`bind_text` と同じ
/// 「身元不明だが成長として許容」扱い）へ落ち、ネスト除外を経由せず
/// Grow 扱いになっていた（`characterData`〔`appendData`〕で同じ要素を
/// 更新した場合は除外されるため経路間で挙動が食い違っていた）。
/// ラップ構成（`content` → `data-bind-list="messages"`）で固定する。
#[wasm_bindgen_test]
async fn nested_bind_list_text_content_replace_is_ignored_in_wrapped_layout() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-wrapped-nested-text-replace-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = wrapped_layout_scroller(
        "ms-wrapped-nested-text-replace",
        HeadlessStuck::Free,
        message_item_with_attachment(),
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();

    // `set_text_content`（childList、target は `data-bind-list` を
    // 持たない `span` 自身）でネストした添付ラベルを更新する。
    let attachment_span = content_el
        .query_selector("[data-bind-list=\"attachments\"] span")
        .expect("query_selector must not fail")
        .expect("attachment label span must exist");
    attachment_span.set_text_content(Some("renamed.pdf"));

    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "ネストした data-bind-list 内の span への set_text_content 更新で \
         scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "ネストした data-bind-list 内の span への set_text_content 更新が \
         data-has-new を立てないこと"
    );
}

/// 上記のフラット構成版（コーディネータ指摘 #1 + #2/#3 の複合）。
#[wasm_bindgen_test]
async fn flat_layout_nested_bind_list_text_content_replace_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-nested-text-replace-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-nested-text-replace",
        HeadlessStuck::Free,
        vec![message_item_with_attachment()],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();

    let attachment_span = content_el
        .query_selector("[data-bind-list=\"attachments\"] span")
        .expect("query_selector must not fail")
        .expect("attachment label span must exist");
    attachment_span.set_text_content(Some("renamed.pdf"));

    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "フラット構成でもネストした data-bind-list 内の span への \
         set_text_content 更新で scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "フラット構成でもネストした data-bind-list 内の span への \
         set_text_content 更新が data-has-new を立てないこと"
    );
}

/// フラット構成 × `characterData` × ネスト `attachments` 内の組み合わせ
/// （コーディネータ指摘 #2/#3 の characterData 経路版）。
#[wasm_bindgen_test]
async fn flat_layout_nested_bind_list_character_data_update_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-flat-nested-character-data-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = flat_layout_scroller(
        "ms-flat-nested-character-data",
        HeadlessStuck::Free,
        vec![message_item_with_attachment()],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();

    let attachment_span = content_el
        .query_selector("[data-bind-list=\"attachments\"] span")
        .expect("query_selector must not fail")
        .expect("attachment label span must exist");
    let text_node = attachment_span
        .first_child()
        .expect("attachment label must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("attachment label's first child must be a Text node");
    text_node
        .append_data(" (renamed)")
        .expect("append_data must not fail");

    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "フラット構成でもネストした data-bind-list 内の characterData \
         更新で scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "フラット構成でもネストした data-bind-list 内の characterData \
         更新が data-has-new を立てないこと"
    );
}

/// コーディネータ指摘（PR #2312 再指摘）の直接の再現: `content` 直下に
/// 素のメッセージ要素（`data-bind-list` を一切持たない）を置き、その
/// **内部だけ**で `attachments` の keyed list を使う構成（会話リスト
/// 自体は `data-bind-list` を持たない、`content` から見て `attachments`
/// は 2 階層深い）。旧実装（幅優先探索で「最も浅い `data-bind-list`」を
/// 機械的に選ぶ）は `content` の孫要素である `attachments` まで潜って
/// それを会話リスト本体と誤認していた。`conversation_list` は `content`
/// 自身・その直接の子（1 段のみ）にしか会話リストの候補を探さないため、
/// この構成では会話リスト本体は `content` 自身になる。
#[wasm_bindgen_test]
async fn no_list_content_with_nested_attachments_prepend_preserves_position_when_stuck_free() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-no-list-nested-prepend-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-no-list-nested-prepend")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(
                    vec![],
                    vec![
                        message_item_with_attachment(),
                        fixed_height_child(60),
                        fixed_height_child(60),
                    ],
                )],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 40);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // `content` 自身（`data-bind-list` を持たないため会話リスト本体と
    // みなされる）への先頭挿入。
    let prepend_html = render(&fixed_height_child(40));
    content_el
        .insert_adjacent_html("afterbegin", &prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 40;
    wait_for(|| viewport.scroll_height() == before_height + expected_delta).await;
    wait_for(|| viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(
        viewport.scroll_top(),
        before_top + expected_delta,
        "content 直下に素のメッセージ要素を置きその内部だけで attachments \
         keyed list を使う構成でも、content への先頭挿入が Prepend として \
         位置維持されること"
    );
}

/// 上記と同一レイアウトで、メッセージ内部にネストした `attachments`
/// （`content` から 2 階層深い）への childList 先頭挿入・characterData
/// 更新のいずれもが、会話リスト本体（`content` 自身）レベルの
/// `scrollTop` 補正・`data-has-new` 付与の対象にならないことを固定する。
#[wasm_bindgen_test]
async fn no_list_content_with_nested_attachments_update_is_ignored() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-no-list-nested-ignored-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", "ms-no-list-nested-ignored")],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![message_item_with_attachment()])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let instance_root = mount(&container, &node);
    let content_el = find_content(&instance_root);
    let viewport = find_viewport(&instance_root);

    wire_message_scroller_events(instance_root.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    simulate_user_scroll(&viewport, 10);
    wait_for(|| instance_root.get_attribute("data-stuck").as_deref() == Some("free")).await;
    let before_top = viewport.scroll_top();
    let before_height = viewport.scroll_height();

    // 会話リスト本体は `content` 自身（`data-bind-list` を持たない）だが、
    // メッセージ内部にネストした `attachments`（`content` から 2 階層
    // 深い）への先頭挿入は、旧実装の幅優先探索がこれを会話リスト本体と
    // 誤認していた対象。
    let attachments_wrapper = content_el
        .query_selector("[data-bind-list=\"attachments\"]")
        .expect("query_selector must not fail")
        .expect("nested attachments data-bind-list wrapper must exist");
    let new_attachment = render(&fixed_height_child(30));
    attachments_wrapper
        .insert_adjacent_html("afterbegin", &new_attachment)
        .expect("insert_adjacent_html must not fail");

    let expected_height = before_height + 30;
    assert_eq!(viewport.scroll_height(), expected_height);
    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "content 直下の素のメッセージ要素内部にネストした attachments \
         への先頭挿入で scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "content 直下の素のメッセージ要素内部にネストした attachments \
         への追加が data-has-new を立てないこと"
    );

    // characterData 経路（`appendData`）でも同じくネスト除外されることを
    // 併せて確認する。
    let attachment_span = content_el
        .query_selector("[data-bind-list=\"attachments\"] span")
        .expect("query_selector must not fail")
        .expect("attachment label span must exist");
    let text_node = attachment_span
        .first_child()
        .expect("attachment label must already have a Text child")
        .dyn_into::<web_sys::Text>()
        .expect("attachment label's first child must be a Text node");
    text_node
        .append_data(" (renamed)")
        .expect("append_data must not fail");

    microtask_tick().await;

    assert_eq!(
        viewport.scroll_top(),
        before_top,
        "characterData 経路でも同様に scrollTop が変化しないこと"
    );
    assert!(
        !instance_root.has_attribute("data-has-new"),
        "characterData 経路でも同様に data-has-new を立てないこと"
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

// --- 祖先インスタンスのスナップショット再同期（コーディネータ指摘 PR #2312 再指摘、round 8） ---

/// 外側 content 直下に「`data-bind-list` を持つ会話リスト本体
/// （messages）」と「`max-height` で高さが伸縮する内側 scroller」を兄弟
/// として持つネスト構造を組み立てる。内側 viewport は
/// `build_message_scroller`/`build_nested_message_scroller` の固定
/// `height` ではなく `max-height` を使う: `max-height` に達するまでは
/// 内側 viewport 自身の自然高さが content の伸長に追従して増え、外側の
/// `scrollHeight` も押し上げる（コーディネータ指摘の「内側 viewport が
/// max-height に達する前など」の状況を再現するための意図的な選択）。
fn build_nested_message_scroller_with_growable_inner_viewport(
    outer_id: &str,
    outer_stuck: HeadlessStuck,
    inner_id: &str,
) -> Node {
    // 外側 viewport（height:100px）に対し十分な高さを持たせ、
    // simulate_user_scroll で確実に「free」（最下部でない）状態を作れる
    // ようにする（total content height > viewport clientHeight が前提）。
    let messages = keyed_list(
        "div",
        vec![],
        "messages",
        vec![("m-1".to_string(), fixed_height_child(150))],
    )
    .expect("keyed_list must not fail for well-formed items");

    let inner = root(
        MessageScrollerRootProps {
            stuck: HeadlessStuck::Free,
            has_new: false,
        },
        vec![("id", inner_id)],
        vec![
            viewport(
                "",
                vec![("style", "max-height:400px;overflow-y:auto")],
                vec![content(vec![], vec![fixed_height_child(20)])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );

    root(
        MessageScrollerRootProps {
            stuck: outer_stuck,
            has_new: false,
        },
        vec![("id", outer_id)],
        vec![
            viewport(
                "",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![content(vec![], vec![messages, inner])],
            ),
            jump_to_latest("Jump to latest", false, vec![], vec![text("Jump")]),
            load_more(false, false, vec![], vec![text("Load more")]),
        ],
    )
}

/// 指摘の再現手順そのもの: (1) 外側 Free 状態で内側 scroller へ追加して
/// 外側の `scrollHeight` が増える、(2) 別バッチで外側自身の会話リスト
/// 本体へ履歴を先頭挿入する。`resolve_instance_from_record` は常に最も
/// 内側のインスタンスへ解決するため、`handle_mutations` が (1) のバッチで
/// 外側インスタンスの計測スナップショットを再同期しない旧実装では、
/// 外側の `last_scroll_height` が (1) の増分だけ古いまま残り、(2) の
/// `corrected_scroll_top` がその古い差分まで加算して過補正する
/// （修正前コードで本テストが FAIL することを確認済み）。
#[wasm_bindgen_test]
async fn ancestor_snapshot_resync_prevents_overcorrection_after_inner_growth() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "ms-ancestor-resync-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_nested_message_scroller_with_growable_inner_viewport(
        "ms-ancestor-resync-outer",
        HeadlessStuck::Free,
        "ms-ancestor-resync-inner",
    );
    let outer_el = mount(&container, &node);
    let outer_viewport = find_viewport(&outer_el);
    let outer_content = find_content(&outer_el);

    wire_message_scroller_events(outer_el.clone(), |_action_ref: ActionRef| {})
        .expect("wire_message_scroller_events must not fail");

    let inner_el = outer_el
        .query_selector("#ms-ancestor-resync-inner")
        .expect("query_selector must not fail")
        .expect("inner scroller must exist");
    let inner_content = find_content(&inner_el);

    // free のまま、利用者が少し下へスクロールした状態を作る。
    simulate_user_scroll(&outer_viewport, 10);
    wait_for(|| outer_el.get_attribute("data-stuck").as_deref() == Some("free")).await;
    assert_eq!(
        outer_el.get_attribute("data-stuck").as_deref(),
        Some("free")
    );

    let before_height = outer_viewport.scroll_height();
    let before_top = outer_viewport.scroll_top();

    // (1) 内側 scroller の content へ追加する。内側 viewport はまだ
    // max-height に達していないため、内側自身の自然高さが伸び、外側の
    // scrollHeight も押し上げられる。分類上は内側インスタンスの変異に
    // 分類されるため、外側の data-has-new・scrollTop へは波及しない
    // （既存の nested_instance_* テストと同じネスト分離の方針）。
    let inner_growth_html = render(&fixed_height_child(30));
    inner_content
        .insert_adjacent_html("beforeend", &inner_growth_html)
        .expect("insert_adjacent_html must not fail");

    wait_for(|| outer_viewport.scroll_height() == before_height + 30).await;
    assert_eq!(
        outer_viewport.scroll_height(),
        before_height + 30,
        "内側の追加で外側の scrollHeight も増えること（前提条件）"
    );
    assert_eq!(
        outer_viewport.scroll_top(),
        before_top,
        "内側インスタンスへの変異だけでは外側の scrollTop が動かないこと"
    );
    assert!(!outer_el.has_attribute("data-has-new"));

    // 次の MutationObserver バッチへ確実に分離してから (2) を行う。
    microtask_tick().await;

    // (2) 別バッチで、外側自身の会話リスト本体（messages）へ先頭挿入する
    // （履歴読み込み相当）。
    let messages_wrapper = outer_content
        .query_selector("[data-bind-list]")
        .expect("query_selector must not fail")
        .expect("messages data-bind-list wrapper must exist");
    let outer_prepend_html = render(&fixed_height_child(50));
    messages_wrapper
        .insert_adjacent_html("afterbegin", &outer_prepend_html)
        .expect("insert_adjacent_html must not fail");

    let expected_delta = 50;
    wait_for(|| outer_viewport.scroll_top() == before_top + expected_delta).await;
    assert_eq!(
        outer_viewport.scroll_top(),
        before_top + expected_delta,
        "外側自身の先頭挿入（+50px）分だけ補正され、(1) の内側成長分（+30px）が \
         祖先スナップショットの再同期漏れにより二重加算されて過補正されないこと \
         （コーディネータ指摘 PR #2312 再指摘、round 8）"
    );
}
