//! Dialog フォーカストラップ（`fandhe_frontend_wasm_full::focus_trap`、イシュー #586、
//! 親 #584）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/focus_trap.rs` の native 単体テスト（`#[cfg(test)]`）は
//! DOM 非依存の純粋ロジック層（`should_trap`/`is_tabbable`/`initial_focus_index`/
//! `next_trap_index`）までを検証済みである。本ファイルはその先、
//! `focus_trap::FocusTrapController`（`#[cfg(target_arch = "wasm32")]` 配線層）が
//! 実 DOM 上で document へ keydown リスナーを登録し、実際の合成 Tab イベントに
//! 対して初期フォーカス・Tab 循環・trigger 復帰を正しく行うことを検証する。
//!
//! フィクスチャの HTML はすべて `fandhe-frontend-headless-ui` の Dialog 自由関数 +
//! `fandhe_frontend_core::render`（既定エスケープ）/ `fandhe_frontend_core::el` で
//! 組み立て、`format!` 等による HTML 文字列直接組み立て・`raw_html()` は
//! 使用しない（`.claude/rules/coding-rust.md`）。
//!
//! # 検証観点（実装計画 §5 ステップ 6 に対応）
//!
//! (a) `push_trap` で初期フォーカスが最初の tabbable へ移る
//! (b) `data-autofocus` 付き要素が優先される
//! (c) tabbable ゼロの content では content 自身へ `tabindex="-1"` 付与のうえ
//!     フォーカス
//! (d) 末尾要素フォーカス中の合成 Tab で先頭へ循環、先頭での Shift+Tab で
//!     末尾へ循環（`default_prevented` も検証）
//! (e) `pop_trap` で trigger（スナップショット済み active element）へフォーカス
//!     復帰
//! (f) `aria-modal="false"`（非 modal）の content は `push_trap` が `None` を
//!     返しトラップ・初期フォーカス移動が発生しない
//! (g) 入れ子 Dialog: 最上位のみ Tab 循環し、上位 pop 後は下位トラップが復活
//!     する
//! (h) controller `Drop` 後は合成 Tab が no-op（登録・解除の対称性の回帰固定）
//! (i) disabled / tabindex="-1" の要素が循環からスキップされる

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, render};
use fandhe_frontend_headless_ui::dialog;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_wasm_full::focus_trap::FocusTrapController;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。
///
/// `wasm-full/tests/overlay_close_browser.rs::create_placeholder` と同じ意図で、
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
/// （`overlay_close_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染対策）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `keydown` イベント（`key="Tab"`、`shift_key` 指定）を生成する。
/// `focus_trap::wiring` のリスナーは document へ登録されるため、
/// `bubbles: true` にして自然な発火経路に近づける
/// （`overlay_close_browser.rs::keydown_event` と同じ意図）。
fn tab_event(shift: bool) -> Event {
    let init = KeyboardEventInit::new();
    init.set_key("Tab");
    init.set_shift_key(shift);
    init.set_bubbles(true);
    init.set_cancelable(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .unchecked_into::<Event>()
}

/// 現在の `document.activeElement` の `id` を返す（`None` の場合は空文字列に
/// フォールバックし、アサーション失敗時のメッセージを読みやすくする）。
fn active_element_id(document: &Document) -> String {
    document
        .active_element()
        .map(|el| el.id())
        .unwrap_or_default()
}

/// trigger 要素（`button`）を id 付きで組み立てる。
fn trigger_html(id: &str) -> String {
    render(&el("button", vec![("id", id), ("type", "button")], vec![]))
}

/// `aria-modal="true"` の Dialog content を、渡された子要素 HTML を
/// `raw_html()` を使わず直接ノードとして埋め込んで組み立てる。
///
/// 子要素は `buttons`（`(id, extra_attrs)` のタプル列）で指定する。
fn mount_modal_dialog(
    document: &Document,
    container: &Element,
    content_id: &str,
    buttons: &[(&str, &[(&str, &str)])],
) -> Element {
    let children: Vec<fandhe_frontend_core::Node> = buttons
        .iter()
        .map(|(id, extra_attrs)| {
            let mut attrs: Vec<(&str, &str)> = vec![("id", *id), ("type", "button")];
            attrs.extend(extra_attrs.iter().copied());
            el("button", attrs, vec![])
        })
        .collect();

    let html = render(&dialog::content(
        OpenState::Open,
        dialog::DialogRole::Dialog,
        true,
        dialog::ContentIds {
            id: Some(content_id),
            ..Default::default()
        },
        vec![],
        children,
    ));
    container
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    document
        .get_element_by_id(content_id)
        .expect("content element must exist")
}

/// `aria-modal="false"`（非 modal）の Dialog content を組み立てる。
fn mount_non_modal_dialog(document: &Document, container: &Element, content_id: &str) -> Element {
    let html = render(&dialog::content(
        OpenState::Open,
        dialog::DialogRole::Dialog,
        false,
        dialog::ContentIds {
            id: Some(content_id),
            ..Default::default()
        },
        vec![],
        vec![el(
            "button",
            vec![("id", "focus-trap-non-modal-button"), ("type", "button")],
            vec![],
        )],
    ));
    container
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    document
        .get_element_by_id(content_id)
        .expect("content element must exist")
}

fn focus(el: &Element) {
    el.clone()
        .dyn_into::<HtmlElement>()
        .expect("element must be an HtmlElement")
        .focus()
        .expect("focus() must not fail");
}

// --- (a)/(b) 初期フォーカス ---

#[wasm_bindgen_test]
fn push_trap_focuses_first_tabbable_by_default() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-initial-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-initial-content",
        &[("focus-trap-initial-a", &[]), ("focus-trap-initial-b", &[])],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    assert_eq!(active_element_id(&document), "focus-trap-initial-a");
    controller.pop_trap(index);
}

#[wasm_bindgen_test]
fn push_trap_prefers_data_autofocus() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-autofocus-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-autofocus-content",
        &[
            ("focus-trap-autofocus-a", &[]),
            ("focus-trap-autofocus-b", &[("data-autofocus", "")]),
            ("focus-trap-autofocus-c", &[]),
        ],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    assert_eq!(active_element_id(&document), "focus-trap-autofocus-b");
    controller.pop_trap(index);
}

// --- (c) tabbable ゼロ ---

#[wasm_bindgen_test]
fn push_trap_focuses_content_itself_when_no_tabbable_children() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-empty-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(&document, &placeholder, "focus-trap-empty-content", &[]);

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    assert_eq!(active_element_id(&document), "focus-trap-empty-content");
    assert_eq!(
        content.get_attribute("tabindex").as_deref(),
        Some("-1"),
        "tabbable な子が無い content には固定リテラル tabindex=\"-1\" を付与すること"
    );
    controller.pop_trap(index);
}

// --- (d) Tab 循環 ---

#[wasm_bindgen_test]
fn tab_from_last_wraps_to_first_and_prevents_default() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-cycle-forward-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-cycle-forward-content",
        &[
            ("focus-trap-cycle-forward-a", &[]),
            ("focus-trap-cycle-forward-b", &[]),
        ],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    let last = document
        .get_element_by_id("focus-trap-cycle-forward-b")
        .expect("last button must exist");
    focus(&last);

    let event = tab_event(false);
    document
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert_eq!(active_element_id(&document), "focus-trap-cycle-forward-a");
    assert!(
        event.default_prevented(),
        "トラップ活性時の Tab は既定動作を prevent すること"
    );
    controller.pop_trap(index);
}

#[wasm_bindgen_test]
fn shift_tab_from_first_wraps_to_last() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-cycle-backward-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-cycle-backward-content",
        &[
            ("focus-trap-cycle-backward-a", &[]),
            ("focus-trap-cycle-backward-b", &[]),
        ],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    let first = document
        .get_element_by_id("focus-trap-cycle-backward-a")
        .expect("first button must exist");
    focus(&first);

    let event = tab_event(true);
    document
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert_eq!(active_element_id(&document), "focus-trap-cycle-backward-b");
    controller.pop_trap(index);
}

// --- (i) disabled / tabindex="-1" はスキップ ---

#[wasm_bindgen_test]
fn tab_skips_disabled_and_negative_tabindex_elements() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-skip-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-skip-content",
        &[
            ("focus-trap-skip-a", &[]),
            ("focus-trap-skip-disabled", &[("disabled", "")]),
            ("focus-trap-skip-negative", &[("tabindex", "-1")]),
            ("focus-trap-skip-b", &[]),
        ],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, None)
        .expect("aria-modal=true dialog must be trapped");

    // 初期フォーカスは先頭 tabbable（a）。
    assert_eq!(active_element_id(&document), "focus-trap-skip-a");

    document
        .dispatch_event(&tab_event(false))
        .expect("dispatch_event must not fail");
    assert_eq!(
        active_element_id(&document),
        "focus-trap-skip-b",
        "disabled・tabindex=-1 の要素をスキップして次の tabbable へ進むこと"
    );

    controller.pop_trap(index);
}

// --- (e) trigger 復帰 ---

#[wasm_bindgen_test]
fn pop_trap_restores_focus_to_snapshotted_trigger() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-restore-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let trigger_id = "focus-trap-restore-trigger";
    placeholder
        .insert_adjacent_html("beforeend", &trigger_html(trigger_id))
        .expect("insert_adjacent_html must not fail");
    let trigger = document
        .get_element_by_id(trigger_id)
        .expect("trigger element must exist");
    focus(&trigger);
    assert_eq!(active_element_id(&document), trigger_id);

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-restore-content",
        &[("focus-trap-restore-a", &[])],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let index = controller
        .push_trap(&content, Some(&trigger))
        .expect("aria-modal=true dialog must be trapped");
    assert_eq!(active_element_id(&document), "focus-trap-restore-a");

    controller.pop_trap(index);
    assert_eq!(
        active_element_id(&document),
        trigger_id,
        "pop_trap は push 時点でフォーカスされていた trigger へ復帰すること"
    );
}

// --- (f) 非 modal は push_trap が None ---

#[wasm_bindgen_test]
fn push_trap_returns_none_for_non_modal_dialog() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-non-modal-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_non_modal_dialog(&document, &placeholder, "focus-trap-non-modal-content");

    let before = active_element_id(&document);
    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let result = controller.push_trap(&content, None);

    assert_eq!(
        result, None,
        "aria-modal=\"false\" の content はトラップ対象外であること"
    );
    assert_eq!(
        active_element_id(&document),
        before,
        "push_trap が None を返す場合、初期フォーカス移動は発生しないこと"
    );
}

// --- (g) 入れ子 Dialog ---

#[wasm_bindgen_test]
fn nested_dialogs_only_topmost_cycles_and_lower_resumes_after_pop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-nested-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let outer = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-nested-outer",
        &[
            ("focus-trap-nested-outer-a", &[]),
            ("focus-trap-nested-outer-b", &[]),
        ],
    );
    let inner = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-nested-inner",
        &[
            ("focus-trap-nested-inner-a", &[]),
            ("focus-trap-nested-inner-b", &[]),
        ],
    );

    let controller = FocusTrapController::new(&document).expect("controller must be created");
    let outer_index = controller
        .push_trap(&outer, None)
        .expect("outer dialog must be trapped");
    assert_eq!(active_element_id(&document), "focus-trap-nested-outer-a");

    let inner_index = controller
        .push_trap(&inner, None)
        .expect("inner dialog must be trapped");
    assert_eq!(active_element_id(&document), "focus-trap-nested-inner-a");

    // 最上位（inner）のみが Tab 循環の対象。
    let last_inner = document
        .get_element_by_id("focus-trap-nested-inner-b")
        .expect("inner last button must exist");
    focus(&last_inner);
    document
        .dispatch_event(&tab_event(false))
        .expect("dispatch_event must not fail");
    assert_eq!(
        active_element_id(&document),
        "focus-trap-nested-inner-a",
        "最上位（inner）のみが Tab 循環の対象であること"
    );

    controller.pop_trap(inner_index);

    // inner を pop した後は outer トラップが復活し、再び Tab 循環の対象になる。
    let last_outer = document
        .get_element_by_id("focus-trap-nested-outer-b")
        .expect("outer last button must exist");
    focus(&last_outer);
    document
        .dispatch_event(&tab_event(false))
        .expect("dispatch_event must not fail");
    assert_eq!(
        active_element_id(&document),
        "focus-trap-nested-outer-a",
        "inner pop 後は outer トラップの Tab 循環が復活すること"
    );

    controller.pop_trap(outer_index);
}

// --- (h) Drop 後は no-op ---

#[wasm_bindgen_test]
fn tab_after_controller_drop_is_no_op() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "focus-trap-drop-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let content = mount_modal_dialog(
        &document,
        &placeholder,
        "focus-trap-drop-content",
        &[("focus-trap-drop-a", &[]), ("focus-trap-drop-b", &[])],
    );

    {
        let controller = FocusTrapController::new(&document).expect("controller must be created");
        let index = controller
            .push_trap(&content, None)
            .expect("aria-modal=true dialog must be trapped");
        controller.pop_trap(index);
    } // ここで controller が Drop され、keydown リスナーが解除される。

    let last = document
        .get_element_by_id("focus-trap-drop-b")
        .expect("last button must exist");
    focus(&last);

    let event = tab_event(false);
    document
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert_eq!(
        active_element_id(&document),
        "focus-trap-drop-b",
        "controller Drop 後は Tab フォーカス循環が起きないこと"
    );
    assert!(
        !event.default_prevented(),
        "controller Drop 後は Tab の既定動作を prevent しないこと"
    );
}
