//! オーバーレイ共通閉鎖制御（`fandhe_frontend_wasm_full::overlay`、イシュー #585、
//! 親 #584）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/overlay.rs` の native 単体テスト（`#[cfg(test)]`）は
//! DOM 非依存の純粋ロジック層（`OverlayKind`/opt-out 判定/スタック閉鎖判定）
//! までを検証済みである。本ファイルはその先、`overlay::OverlayCloseController`
//! （`#[cfg(target_arch = "wasm32")]` 配線層）が実 DOM 上で document へ
//! keydown/pointerdown リスナーを登録し、実際の合成イベントに対して
//! 正しい閉鎖要求を発行する（あるいは発行しない）ことを検証する。
//!
//! フィクスチャの HTML はすべて `fandhe-frontend-headless-ui` の Dialog/Popover
//! 自由関数 + `fandhe_frontend_core::render`（既定エスケープ）で組み立て、
//! `format!` 等による HTML 文字列直接組み立て・`raw_html()` は使用しない
//! （`.claude/rules/coding-rust.md`）。
//!
//! # 検証観点（実装計画 §5 ステップ 4 に対応）
//!
//! (a) 合成 `KeyboardEvent(key="Escape")` で最上位のみ close 要求が発火する
//! (b) content 外への合成 pointerdown で close 要求が発火する
//! (c) content 内 / trigger 上の pointerdown では発火しない
//! (d) `data-close-on-escape="false"` / `data-close-on-interact-outside="false"`
//!     の opt-out が効く
//! (e) `remove_overlay` 後・controller `Drop` 後はイベントを発しても発火しない
//!     （登録・解除の対称性の回帰固定）
//! (f) 未知 `data-scope`・改ざん属性で panic せず no-op

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::dialog;
use fandhe_frontend_headless_ui::popover;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_wasm_full::overlay::{OverlayCloseController, OverlayCloseRequest};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。
///
/// `wasm-full/tests/runtime_browser.rs::create_placeholder` と同じ意図で、
/// 一意な id により同一テストバイナリ内の複数テストが要素・リスナーを
/// 奪い合わないようにする。
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
/// （`runtime_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染対策）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `keydown` イベント（`key` 指定）を生成する。document へ登録された
/// リスナーへ直接 `dispatch_event` するため `bubbles` は不要（document 自身
/// が対象）だが、実ブラウザの自然な発火経路に近づけるため `true` にしておく。
fn keydown_event(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_key(key);
    init.set_bubbles(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .unchecked_into::<Event>()
}

/// 合成 `pointerdown` イベントを生成する。`overlay::wiring` のリスナーは
/// document へ登録されるため、テスト側は対象要素へ `bubbles: true` で
/// dispatch し、document までバブリングさせる。
fn pointerdown_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("pointerdown", &init).expect("Event::new must not fail")
}

/// 直近の閉鎖要求を蓄積するだけの記録用コールバック（dispatch・DOM 更新は
/// 行わない。本モジュールの責務分離を検証側でも尊重する）。
type Requests = std::rc::Rc<std::cell::RefCell<Vec<OverlayCloseRequest>>>;

fn recording_controller(document: &Document) -> (OverlayCloseController, Requests) {
    let requests: Requests = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let recorder = requests.clone();
    let controller = OverlayCloseController::new(document, move |request| {
        recorder.borrow_mut().push(request);
    })
    .expect("OverlayCloseController::new must succeed");
    (controller, requests)
}

/// 単一の Dialog（trigger + content）を `container` 配下へ展開し、
/// `(trigger, content)` を返す。
fn mount_dialog(document: &Document, container: &Element, id_prefix: &str) -> (Element, Element) {
    let trigger_id = format!("{id_prefix}-trigger");
    let content_id = format!("{id_prefix}-content");
    let html = render(&dialog::root(
        OpenState::Open,
        vec![],
        vec![
            dialog::trigger(OpenState::Open, None, vec![("id", &trigger_id)], vec![]),
            dialog::content(
                OpenState::Open,
                dialog::DialogRole::Dialog,
                true,
                dialog::ContentIds {
                    id: Some(&content_id),
                    ..Default::default()
                },
                vec![],
                vec![],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let trigger = document
        .get_element_by_id(&trigger_id)
        .expect("trigger element must exist");
    let content = document
        .get_element_by_id(&content_id)
        .expect("content element must exist");
    (trigger, content)
}

// --- (a) Escape: 最上位のみ発火 ---

#[wasm_bindgen_test]
fn escape_closes_topmost_only_in_nested_overlays() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-escape-nested-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (outer_trigger, outer_content) = mount_dialog(&document, &placeholder, "overlay-esc-outer");

    // Popover を outer_content の子として追加する（入れ子オーバーレイの再現）。
    let popover_html = render(&popover::root(
        OpenState::Open,
        vec![],
        vec![
            popover::trigger(
                OpenState::Open,
                false,
                None,
                vec![("id", "overlay-esc-inner-trigger")],
                vec![],
            ),
            popover::content(
                OpenState::Open,
                Some("overlay-esc-inner-content"),
                None,
                None,
                vec![],
                vec![],
            ),
        ],
    ));
    outer_content
        .insert_adjacent_html("beforeend", &popover_html)
        .expect("insert_adjacent_html must not fail");
    let inner_content = document
        .get_element_by_id("overlay-esc-inner-content")
        .expect("inner content element must exist");

    let (controller, requests) = recording_controller(&document);
    let outer_index = controller
        .push_overlay(&outer_content, Some(&outer_trigger))
        .expect("dialog scope must be recognized");
    let inner_index = controller
        .push_overlay(&inner_content, None)
        .expect("popover scope must be recognized");

    document
        .dispatch_event(&keydown_event("Escape"))
        .expect("dispatch_event must not fail");

    let recorded = requests.borrow().clone();
    assert_eq!(
        recorded.len(),
        1,
        "Escape は最上位 1 件のみを閉鎖対象にすること: {recorded:?}"
    );
    assert_eq!(
        recorded[0].index, inner_index,
        "最上位（後から push した inner）が対象であること"
    );
    assert_ne!(recorded[0].index, outer_index);
}

#[wasm_bindgen_test]
fn escape_with_non_escape_key_does_not_fire() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-escape-other-key-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-esc-otherkey");
    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");

    document
        .dispatch_event(&keydown_event("Enter"))
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "Escape 以外のキーでは発火しないこと"
    );
}

// --- (b)/(c) 外側/内側 pointerdown ---

#[wasm_bindgen_test]
fn pointerdown_outside_content_closes_it() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-pointerdown-outside-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-outside");
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");

    // placeholder 自身（content/trigger のいずれの子孫でもない）へ pointerdown。
    placeholder
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    let recorded = requests.borrow().clone();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].index, index);
}

#[wasm_bindgen_test]
fn pointerdown_inside_content_does_not_close() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-pointerdown-inside-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-inside");
    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");

    content
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "content 内側の pointerdown では閉鎖しないこと"
    );
}

#[wasm_bindgen_test]
fn pointerdown_on_trigger_does_not_close() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-pointerdown-trigger-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-trigger");
    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");

    // trigger 上の pointerdown は「外側」扱いにしない
    // （閉鎖直後の click でトグルが再度開く競合を避けるため、overlay.rs doc 参照）。
    trigger
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "trigger 上の pointerdown では閉鎖しないこと"
    );
}

// --- (d) opt-out 属性 ---

#[wasm_bindgen_test]
fn close_on_escape_false_opts_out_of_escape_closing() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-escape-optout-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content_id = "overlay-escape-optout-content";
    let html = render(&dialog::root(
        OpenState::Open,
        vec![],
        vec![dialog::content(
            OpenState::Open,
            dialog::DialogRole::Dialog,
            true,
            dialog::ContentIds {
                id: Some(content_id),
                ..Default::default()
            },
            vec![("data-close-on-escape", "false")],
            vec![],
        )],
    ));
    placeholder.set_inner_html(&html);
    let content = document
        .get_element_by_id(content_id)
        .expect("content element must exist");

    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, None)
        .expect("dialog scope must be recognized");

    document
        .dispatch_event(&keydown_event("Escape"))
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "data-close-on-escape=\"false\" の opt-out が効くこと"
    );
}

#[wasm_bindgen_test]
fn close_on_interact_outside_false_opts_out_of_outside_closing() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-outside-optout-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content_id = "overlay-outside-optout-content";
    let html = render(&dialog::root(
        OpenState::Open,
        vec![],
        vec![dialog::content(
            OpenState::Open,
            dialog::DialogRole::Dialog,
            true,
            dialog::ContentIds {
                id: Some(content_id),
                ..Default::default()
            },
            vec![("data-close-on-interact-outside", "false")],
            vec![],
        )],
    ));
    placeholder.set_inner_html(&html);
    let content = document
        .get_element_by_id(content_id)
        .expect("content element must exist");

    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, None)
        .expect("dialog scope must be recognized");

    placeholder
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "data-close-on-interact-outside=\"false\" の opt-out が効くこと"
    );
}

// --- (e) 登録・解除の対称性 ---

#[wasm_bindgen_test]
fn remove_overlay_stops_future_close_requests_for_it() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-remove-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-remove");
    let (controller, requests) = recording_controller(&document);
    let index = controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");
    assert_eq!(controller.stack_len(), 1);

    controller.remove_overlay(index);
    assert_eq!(controller.stack_len(), 0);

    document
        .dispatch_event(&keydown_event("Escape"))
        .expect("dispatch_event must not fail");
    placeholder
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "remove_overlay 後は当該オーバーレイに対する閉鎖要求が発火しないこと"
    );
}

#[wasm_bindgen_test]
fn dropping_controller_unregisters_document_listeners() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-drop-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let (trigger, content) = mount_dialog(&document, &placeholder, "overlay-drop");
    let (controller, requests) = recording_controller(&document);
    controller
        .push_overlay(&content, Some(&trigger))
        .expect("dialog scope must be recognized");

    // controller を drop すると keydown/pointerdown リスナーが document から
    // 解除される（`OverlayCloseController` の `Drop` 実装、`Closure::forget`
    // を使わない登録・解除の対称性の回帰固定）。
    drop(controller);

    document
        .dispatch_event(&keydown_event("Escape"))
        .expect("dispatch_event must not fail");
    placeholder
        .dispatch_event(&pointerdown_event())
        .expect("dispatch_event must not fail");

    assert!(
        requests.borrow().is_empty(),
        "controller を drop した後はリスナーが解除され、以後イベントを発しても発火しないこと"
    );
}

// --- (f) 未知 scope・改ざん属性で panic しない ---

#[wasm_bindgen_test]
fn push_overlay_with_unknown_scope_returns_none_and_does_not_panic() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-unknown-scope-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let bogus = document
        .create_element("div")
        .expect("create_element must not fail");
    bogus
        .set_attribute("data-scope", "drawer")
        .expect("set_attribute must not fail");
    placeholder
        .append_child(&bogus)
        .expect("append_child must not fail");

    let (controller, requests) = recording_controller(&document);
    let index = controller.push_overlay(&bogus, None);
    assert_eq!(index, None, "未知 data-scope は登録されないこと");
    assert_eq!(controller.stack_len(), 0);

    document
        .dispatch_event(&keydown_event("Escape"))
        .expect("dispatch_event must not fail");
    assert!(requests.borrow().is_empty());
}

#[wasm_bindgen_test]
fn push_overlay_without_data_scope_attribute_returns_none() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "overlay-missing-scope-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let bare = document
        .create_element("div")
        .expect("create_element must not fail");
    placeholder
        .append_child(&bare)
        .expect("append_child must not fail");

    let (controller, _requests) = recording_controller(&document);
    assert_eq!(controller.push_overlay(&bare, None), None);
    assert_eq!(controller.stack_len(), 0);
}

#[wasm_bindgen_test]
fn remove_overlay_with_out_of_range_index_does_not_panic() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let (controller, _requests) = recording_controller(&document);

    // スタックが空の状態で範囲外 index を渡しても panic しないこと。
    controller.remove_overlay(0);
    controller.remove_overlay(999);
    assert_eq!(controller.stack_len(), 0);
}
