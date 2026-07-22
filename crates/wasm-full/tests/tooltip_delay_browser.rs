//! Tooltip の `openDelay`/`closeDelay`/`interactive`
//! （`fandhe_frontend_wasm_full::tooltip`、イシュー #587、親 #584）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/tooltip.rs` の native 単体テスト（`#[cfg(test)]`）は
//! DOM 非依存の純粋ロジック層（[`TooltipDelayConfig::from_attrs`]・
//! [`transition`]）までを検証済みである。本ファイルはその先、
//! `tooltip::TooltipDelayController`（`#[cfg(target_arch = "wasm32")]`
//! 配線層）が実 DOM 上で trigger/content 要素へ直接登録したリスナーと
//! 実際の `setTimeout`/`clearTimeout` を用いて、正しいタイミングで
//! [`TooltipDelayRequest`] を発行する（あるいは発行しない）ことを検証する。
//!
//! フィクスチャの HTML はすべて `fandhe-frontend-headless-ui` の Tooltip
//! 自由関数 + `fandhe_frontend_core::render`（既定エスケープ）で組み立て、
//! `format!` 等による HTML 文字列直接組み立て・`raw_html()` は使用しない
//! （`.claude/rules/coding-rust.md`）。
//!
//! # 検証観点（実装計画 §5 ステップ 4 に対応）
//!
//! (a) `openDelay` 経過後に表示要求が発行される
//! (b) `openDelay` 満了前の早期 `pointerleave` で表示要求が発行されない
//! (c) `interactive=true` のとき content 内へのポインタ移動で非表示要求を
//!     取り消し、表示が維持される
//! (d) `interactive=false` のとき content 内へのポインタ移動しても
//!     `closeDelay` 経過後に非表示要求が発行される
//! (e) `focusin`/`focusout` は遅延なしで即時に表示/非表示要求を発行する
//! (f) `remove_tooltip` 後・`TooltipDelayController` `Drop` 後は保留中の
//!     タイマーが発火せず、イベントを発しても要求が発行されない
//!     （登録・解除・タイマー破棄の対称性の回帰固定）
//! (g) 攻撃者が注入した `<script>` を含む children テキストが既定エスケープ
//!     されタグとして解釈されないこと（XSS 回帰）
//! (h) ポインタとフォーカスは独立した入力チャネルであり、どちらか一方が
//!     まだ表示継続を要求している間はもう一方の離脱イベントで非表示に
//!     しないこと（イシュー #587 の Cursor Bugbot 指摘の回帰、`src/tooltip.rs`
//!     `transition` の `stay_open` 判定参照）

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::tooltip;
use fandhe_frontend_wasm_full::tooltip::{TooltipDelayController, TooltipDelayRequest};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`overlay_close_browser.rs::create_placeholder` と同じ意図。一意な id に
/// より同一テストバイナリ内の複数テストが要素・リスナーを奪い合わない）。
fn create_placeholder(document: &Document, id: &str) -> Element {
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

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`overlay_close_browser.rs::RemoveOnDrop` と同じ意図）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `ms` ミリ秒だけ実時間で待つ（`setTimeout` を `Promise` 化して `await` する。
/// `nav_browser.rs::next_animation_frame` と同じ「実タイマーを使うブラウザ
/// テストの決定的な待機」方針。遅延値は 10ms 級に短縮し、待機は遅延値の
/// 十分な倍数で行うことでフレークを避ける、実装計画 §8 リスク対応）。
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

/// 合成 `pointerenter`/`pointerleave`/`focusin`/`focusout` イベントを生成する
/// （いずれもバブリングしないイベント種別だが、`target` 要素へ直接
/// `dispatch_event` するため `bubbles` の指定は不要。
/// `tooltip::wiring` のリスナーは対象要素へ直接登録されるため、これで
/// 十分に捕捉される）。
fn synthetic_event(kind: &str) -> Event {
    let init = EventInit::new();
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

fn dispatch(target: &Element, kind: &str) {
    target
        .dispatch_event(&synthetic_event(kind))
        .expect("dispatch_event must not fail");
}

/// 単一の Tooltip（root/trigger/positioner/content）を `container` 配下へ
/// 展開し、`(root, trigger, content)` を返す。`open_delay_ms`/`close_delay_ms`/
/// `interactive` は root の `data-*` 属性へ付与する
/// （呼び出し側 `attrs` 経由のオプトイン方式、`tooltip.rs` モジュール冒頭
/// doc §契約参照）。
fn mount_tooltip(
    document: &Document,
    container: &Element,
    id_prefix: &str,
    open_delay_ms: &str,
    close_delay_ms: &str,
    interactive: &str,
    label: &str,
) -> (Element, Element, Element) {
    let root_id = format!("{id_prefix}-root");
    let trigger_id = format!("{id_prefix}-trigger");
    let content_id = format!("{id_prefix}-content");
    let html = render(&tooltip::root(
        OpenState::Closed,
        vec![
            ("id", &root_id),
            ("data-open-delay", open_delay_ms),
            ("data-close-delay", close_delay_ms),
            ("data-interactive", interactive),
        ],
        vec![
            tooltip::trigger(
                OpenState::Closed,
                false,
                Some(&content_id),
                vec![("id", &trigger_id)],
                vec![fandhe_frontend_core::text("Trigger")],
            ),
            tooltip::positioner(
                vec![],
                vec![tooltip::content(
                    OpenState::Open,
                    Some(&content_id),
                    vec![],
                    vec![fandhe_frontend_core::text(label)],
                )],
            ),
        ],
    ));
    container
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root = document
        .get_element_by_id(&root_id)
        .expect("root element must exist");
    let trigger = document
        .get_element_by_id(&trigger_id)
        .expect("trigger element must exist");
    let content = document
        .get_element_by_id(&content_id)
        .expect("content element must exist");
    (root, trigger, content)
}

/// 直近の要求を蓄積するだけの記録用コールバック（dispatch・DOM 更新は
/// 行わない。本モジュールの責務分離を検証側でも尊重する、
/// `overlay_close_browser.rs::recording_controller` と同じ意図）。
type Requests = std::rc::Rc<std::cell::RefCell<Vec<TooltipDelayRequest>>>;

fn recording_controller(document: &Document) -> (TooltipDelayController, Requests) {
    let window = document
        .default_view()
        .expect("document.defaultView must exist in browser test environment");
    let requests: Requests = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let recorder = requests.clone();
    let controller = TooltipDelayController::new(&window, move |request| {
        recorder.borrow_mut().push(request);
    });
    (controller, requests)
}

// --- (a)/(b): openDelay 経過後に表示要求・早期 leave での取消 ---

#[wasm_bindgen_test]
async fn open_delay_elapses_before_requesting_open() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-open-delay-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-a",
        "20",
        "20",
        "false",
        "Tip A",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "pointerenter");
    assert!(
        requests.borrow().is_empty(),
        "openDelay 満了前は表示要求が発行されないこと"
    );

    sleep_ms(100).await;
    assert_eq!(
        requests.borrow().len(),
        1,
        "openDelay 満了後に表示要求が 1 回発行されること"
    );
    assert_eq!(requests.borrow()[0].index, index);

    controller.remove_tooltip(index);
}

#[wasm_bindgen_test]
async fn early_leave_before_open_delay_cancels_open_request() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-early-leave-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-b",
        "50",
        "20",
        "false",
        "Tip B",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "pointerenter");
    dispatch(&trigger, "pointerleave");
    sleep_ms(150).await;
    assert!(
        requests.borrow().is_empty(),
        "openDelay 満了前の pointerleave はタイマーを取消し、表示要求を発行しないこと"
    );

    controller.remove_tooltip(index);
}

// --- (c)/(d): interactive on/off での content 内ポインタ移動の効果差 ---

#[wasm_bindgen_test]
async fn interactive_true_keeps_open_when_pointer_moves_into_content() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-interactive-true-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-c",
        "0",
        "30",
        "true",
        "Tip C",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "pointerenter");
    assert_eq!(
        requests.borrow().len(),
        1,
        "openDelay=0 のため即時に表示要求が発行されること"
    );

    dispatch(&trigger, "pointerleave");
    dispatch(&content, "pointerenter");
    sleep_ms(120).await;
    assert_eq!(
        requests.borrow().len(),
        1,
        "interactive=true では content への進入が close タイマーを取消し、非表示要求は発行されないこと"
    );

    controller.remove_tooltip(index);
}

#[wasm_bindgen_test]
async fn interactive_false_closes_even_when_pointer_moves_into_content() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-interactive-false-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-d",
        "0",
        "20",
        "false",
        "Tip D",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "pointerenter");
    assert_eq!(
        requests.borrow().len(),
        1,
        "openDelay=0 のため即時に表示要求が発行されること"
    );

    dispatch(&trigger, "pointerleave");
    dispatch(&content, "pointerenter");
    sleep_ms(100).await;
    assert_eq!(
        requests.borrow().len(),
        2,
        "interactive=false では content への進入は無視され、closeDelay 満了後に非表示要求が発行されること"
    );
    assert_eq!(requests.borrow()[1].index, index);

    controller.remove_tooltip(index);
}

// --- (e): focusin/focusout は遅延なしで即時 ---

#[wasm_bindgen_test]
async fn focus_opens_and_blur_closes_immediately_ignoring_delay() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-focus-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-e",
        "5000",
        "5000",
        "false",
        "Tip E",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "focusin");
    assert_eq!(
        requests.borrow().len(),
        1,
        "focusin は openDelay を無視して即時に表示要求を発行すること"
    );

    dispatch(&trigger, "focusout");
    assert_eq!(
        requests.borrow().len(),
        2,
        "focusout は closeDelay を無視して即時に非表示要求を発行すること"
    );
    assert_eq!(requests.borrow()[0].index, index);
    assert_eq!(requests.borrow()[1].index, index);

    controller.remove_tooltip(index);
}

// --- (h): ポインタ/フォーカス競合の解決（イシュー #587 Cursor Bugbot 指摘） ---

#[wasm_bindgen_test]
async fn blur_does_not_close_while_pointer_still_hovers_trigger() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-blur-hover-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-i",
        "0",
        "20",
        "false",
        "Tip I",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    // Tab でフォーカスして即時表示（openDelay=0）。
    dispatch(&trigger, "focusin");
    assert_eq!(
        requests.borrow().len(),
        1,
        "focusin で即時表示要求が出ること"
    );

    // ポインタが trigger 上にまだある状態を再現してから Tab で移動（blur）。
    dispatch(&trigger, "pointerenter");
    dispatch(&trigger, "focusout");
    sleep_ms(60).await;
    assert_eq!(
        requests.borrow().len(),
        1,
        "ポインタが trigger 上にまだある間は blur で非表示要求を発行しないこと\
         （イシュー #587 Cursor Bugbot 指摘の回帰）"
    );

    // 最後にポインタも離れれば closeDelay 経過後に非表示要求が発行される。
    dispatch(&trigger, "pointerleave");
    sleep_ms(60).await;
    assert_eq!(
        requests.borrow().len(),
        2,
        "両方の入力チャネルが離脱した後は非表示要求が発行されること"
    );

    controller.remove_tooltip(index);
}

#[wasm_bindgen_test]
async fn pointer_leave_does_not_close_while_trigger_still_focused() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-leave-focus-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-j",
        "0",
        "20",
        "false",
        "Tip J",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    // ポインタで即時表示（openDelay=0）した後、キーボードフォーカスも
    // trigger にある状態を再現してからポインタだけ離脱する。
    dispatch(&trigger, "pointerenter");
    assert_eq!(
        requests.borrow().len(),
        1,
        "pointerenter で即時表示要求が出ること"
    );

    dispatch(&trigger, "focusin");
    dispatch(&trigger, "pointerleave");
    sleep_ms(60).await;
    assert_eq!(
        requests.borrow().len(),
        1,
        "trigger がまだフォーカスされている間は pointerleave で非表示要求を発行しないこと\
         （イシュー #587 Cursor Bugbot 指摘の回帰）"
    );

    // フォーカスも外れれば即時（フォーカス/blur は遅延なし）に非表示要求が発行される。
    dispatch(&trigger, "focusout");
    assert_eq!(
        requests.borrow().len(),
        2,
        "フォーカスも外れれば非表示要求が発行されること"
    );

    controller.remove_tooltip(index);
}

// --- (f): remove_tooltip・Drop 後のタイマー・リスナー無効化（回帰） ---

#[wasm_bindgen_test]
async fn removed_tooltip_pending_timer_does_not_fire() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-removed-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-f",
        "20",
        "20",
        "false",
        "Tip F",
    );
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .register_tooltip(&_root, &trigger, &content)
        .expect("register_tooltip must succeed");

    dispatch(&trigger, "pointerenter");
    controller.remove_tooltip(index);
    sleep_ms(100).await;
    assert!(
        requests.borrow().is_empty(),
        "remove_tooltip 後は保留中の openDelay タイマーが発火せず、表示要求も発行されないこと"
    );

    // remove 後にイベントを発してもリスナーは既に解除済みのため反応しない。
    dispatch(&trigger, "pointerenter");
    dispatch(&trigger, "focusin");
    sleep_ms(50).await;
    assert!(
        requests.borrow().is_empty(),
        "remove_tooltip 後は trigger へのイベントに反応しないこと（リスナー解除の回帰）"
    );
}

#[wasm_bindgen_test]
async fn controller_drop_cancels_pending_timers_and_listeners() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-drop-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (_root, trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-g",
        "20",
        "20",
        "false",
        "Tip G",
    );
    let requests: Requests = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    {
        let window = document
            .default_view()
            .expect("document.defaultView must exist in browser test environment");
        let recorder = requests.clone();
        let controller = TooltipDelayController::new(&window, move |request| {
            recorder.borrow_mut().push(request);
        });
        controller
            .register_tooltip(&_root, &trigger, &content)
            .expect("register_tooltip must succeed");
        dispatch(&trigger, "pointerenter");
        // `controller` はここでスコープを抜けて Drop される
        // （保留中の openDelay タイマー・登録済みリスナーが対称的に解除される
        // ことを期待する、`overlay.rs::OverlayCloseController` と同じ A04 対策）。
    }

    sleep_ms(100).await;
    assert!(
        requests.borrow().is_empty(),
        "TooltipDelayController の Drop 後は保留中タイマーが発火しないこと"
    );

    dispatch(&trigger, "pointerenter");
    sleep_ms(50).await;
    assert!(
        requests.borrow().is_empty(),
        "TooltipDelayController の Drop 後は trigger へのイベントに反応しないこと（リスナー解除の回帰）"
    );
}

// --- (g): XSS 回帰（children テキストの既定エスケープ） ---

#[wasm_bindgen_test]
fn content_label_with_script_payload_is_escaped_as_text() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .unwrap();
    let placeholder = create_placeholder(&document, "tooltip-xss-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let payload = "<script>window.__tooltip_xss__=1</script>";
    let (_root, _trigger, content) = mount_tooltip(
        &document,
        &placeholder,
        "tooltip-h",
        "10",
        "10",
        "false",
        payload,
    );

    assert!(
        content.query_selector("script").unwrap().is_none(),
        "children テキストの <script> はタグとして解釈されず、既定エスケープされたテキストであること"
    );
    assert!(
        content.text_content().unwrap().contains("<script>"),
        "エスケープされたテキストとして <script> の文字列表現が content 内に残ること"
    );
}
