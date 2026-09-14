//! `fandhe_frontend_wasm_full::gesture::wire_gesture`（hover/press ジェス
//! チャー配線、イシュー #2520）の実ブラウザ統合テスト（`wasm-pack test
//! --headless --chrome`）。
//!
//! `wasm-full/src/gesture.rs` の native テストは純粋層
//! （`is_touch_pointer`/`is_press_activation_key`）までを検証済み。本
//! ファイルはその先、`wire_gesture` が実 DOM 上で pointerover/pointerout・
//! pointerdown/pointerup/pointercancel・keydown/keyup に応じて
//! `data-fandhe-hover`/`data-fandhe-press` を正しく付け外しすることを
//! `chart_tooltip_browser.rs`/`focus_visible_browser.rs` と同方針（手組み
//! DOM への直接 `wire_gesture` 呼び出し、`Runtime::mount` は経由しない）
//! で検証する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::gesture::{
    wire_gesture, GESTURE_HOVER_ATTR, GESTURE_PRESS_ATTR, HOVER_STATE_ATTR, PRESS_STATE_ATTR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventTarget, FocusEvent, FocusEventInit, KeyboardEvent,
    KeyboardEventInit, PointerEvent, PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`keynav_browser.rs::RemoveOnDrop`
/// と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// root（opt-in 属性なし） > child（hover+press opt-in、id 付き）> grandchild
/// （opt-in なし、内部移動の被験対象）の 3 階層 DOM を組み立てる。
/// 返り値: `(root, child, grandchild)`。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let child = document.create_element("button").unwrap();
    child.set_attribute(GESTURE_HOVER_ATTR, "").unwrap();
    child.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    root.append_child(&child).unwrap();

    let grandchild = document.create_element("span").unwrap();
    child.append_child(&grandchild).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, child, grandchild)
}

fn pointer_event(kind: &str, pointer_type: &str, related: Option<&Element>) -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type(pointer_type);
    if let Some(related) = related {
        init.set_related_target(Some(related.unchecked_ref::<EventTarget>()));
    }
    PointerEvent::new_with_event_init_dict(kind, &init).expect("PointerEvent::new must not fail")
}

fn dispatch_pointer(target: &Element, kind: &str, pointer_type: &str, related: Option<&Element>) {
    target
        .dispatch_event(pointer_event(kind, pointer_type, related).as_ref())
        .expect("dispatch_event must not fail");
}

fn dispatch_key(target: &Element, kind: &str, key: &str, repeat: bool) {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_key(key);
    init.set_repeat(repeat);
    let event =
        KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init).expect("KeyboardEvent::new");
    target
        .dispatch_event(Event::from(event).as_ref())
        .expect("dispatch_event must not fail");
}

fn dispatch_focusout(target: &Element) {
    let init = FocusEventInit::new();
    init.set_bubbles(true);
    let event =
        FocusEvent::new_with_focus_event_init_dict("focusout", &init).expect("FocusEvent::new");
    target
        .dispatch_event(Event::from(event).as_ref())
        .expect("dispatch_event must not fail");
}

#[wasm_bindgen_test]
fn pointerover_sets_hover_for_non_touch_and_pointerout_clears_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-hover-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerover", "mouse", None);
    assert!(
        child.has_attribute(HOVER_STATE_ATTR),
        "非タッチ pointerover は hover 状態を付与する"
    );

    dispatch_pointer(&child, "pointerout", "mouse", None);
    assert!(
        !child.has_attribute(HOVER_STATE_ATTR),
        "related_target が対象外の pointerout は hover 状態を解除する"
    );
}

#[wasm_bindgen_test]
fn pointerover_ignores_touch_pointer() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-touch-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerover", "touch", None);
    assert!(
        !child.has_attribute(HOVER_STATE_ATTR),
        "タッチ由来の pointerover は疑似 hover を発生させない"
    );
}

#[wasm_bindgen_test]
fn pointerover_within_same_target_does_not_reenter() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, grandchild) = build_dom(&document, "gesture-internal-move-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerover", "mouse", None);
    assert!(child.has_attribute(HOVER_STATE_ATTR));

    // child 内の子要素（grandchild）間の移動は related_target が child 配下
    // に留まるため「離脱」と判定されない。
    dispatch_pointer(&child, "pointerout", "mouse", Some(&grandchild));
    assert!(
        child.has_attribute(HOVER_STATE_ATTR),
        "opt-in 要素内部への移動は hover 状態を解除しない"
    );
}

#[wasm_bindgen_test]
fn pointerdown_up_and_cancel_toggle_press_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-press-pointer-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerdown", "mouse", None);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_pointer(&child, "pointerup", "mouse", None);
    assert!(!child.has_attribute(PRESS_STATE_ATTR));

    dispatch_pointer(&child, "pointerdown", "touch", None);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "press はタッチも対象とする"
    );
    dispatch_pointer(&child, "pointercancel", "touch", None);
    assert!(!child.has_attribute(PRESS_STATE_ATTR));
}

#[wasm_bindgen_test]
fn pointerout_leaving_target_clears_press_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-press-leave-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerdown", "mouse", None);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // related_target が child 外（root）へ抜けるドラッグ離脱を模す。
    dispatch_pointer(&child, "pointerout", "mouse", Some(&root));
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "要素外へのドラッグ離脱は press 状態を解除する"
    );
}

#[wasm_bindgen_test]
fn keydown_up_activation_key_toggles_press_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-press-key-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", "Enter", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));
    dispatch_key(&child, "keyup", "Enter", false);
    assert!(!child.has_attribute(PRESS_STATE_ATTR));

    dispatch_key(&child, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));
    dispatch_key(&child, "keyup", " ", false);
    assert!(!child.has_attribute(PRESS_STATE_ATTR));

    dispatch_key(&child, "keydown", "Escape", false);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "非活性化キーは press 状態を付与しない"
    );
}

#[wasm_bindgen_test]
fn opted_out_element_never_receives_gesture_attributes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-opt-out-test");
    let plain = document.create_element("button").unwrap();
    root.append_child(&plain).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&plain, "pointerover", "mouse", None);
    dispatch_pointer(&plain, "pointerdown", "mouse", None);
    dispatch_key(&plain, "keydown", "Enter", false);

    assert!(!plain.has_attribute(HOVER_STATE_ATTR));
    assert!(!plain.has_attribute(PRESS_STATE_ATTR));
}

/// codex-review 指摘の回帰固定: 入れ子の opt-in 祖先（親・子とも
/// [`GESTURE_HOVER_ATTR`]）で子から真に離脱したとき、子だけでなく祖先
/// （親）の [`HOVER_STATE_ATTR`] も解除されること。`closest_opted_in`
/// （最も近い 1 件のみ）ではこの離脱で親の hover 状態が取り残されていた。
#[wasm_bindgen_test]
fn nested_opt_in_elements_both_update_hover_on_leave() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, parent, child) = build_dom(&document, "gesture-nested-hover-test");
    child.set_attribute(GESTURE_HOVER_ATTR, "").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // 外部から入れ子の子要素へ直接進入する（間に親専用の進入イベントを
    // 経由しない）。
    dispatch_pointer(&child, "pointerover", "mouse", None);
    assert!(child.has_attribute(HOVER_STATE_ATTR));
    assert!(
        parent.has_attribute(HOVER_STATE_ATTR),
        "内側要素への直接進入でも外側祖先の hover 状態が更新されること"
    );

    // 子から root 外（related_target なし = 完全に外部）へ離脱する。
    dispatch_pointer(&child, "pointerout", "mouse", None);
    assert!(!child.has_attribute(HOVER_STATE_ATTR));
    assert!(
        !parent.has_attribute(HOVER_STATE_ATTR),
        "子要素からの離脱で外側祖先（親）の hover 状態も解除されること"
    );
}

/// codex-review 指摘の回帰固定: keyboard（Enter/Space）由来の press 中に
/// ポインタが要素外へ出ても（`pointerout`）、press 状態を誤って解除しない
/// こと。従来実装は pointerout で無条件に press を解除しており、
/// keyboard 由来の press まで消してしまっていた。
#[wasm_bindgen_test]
fn pointerout_does_not_clear_keyboard_activated_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-keyboard-pointerout-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_pointer(&child, "pointerout", "mouse", None);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "keyboard 由来の press は pointerout で解除されないこと"
    );

    dispatch_key(&child, "keyup", " ", false);
    assert!(!child.has_attribute(PRESS_STATE_ATTR));
}

/// codex-review 指摘の回帰固定: Space 押下（keydown）中にフォーカスが別
/// 要素へ移動すると、後続の `keyup` は新しいフォーカス先へ発火し元要素の
/// [`PRESS_STATE_ATTR`] が残り続けていた。`focusout` での即時解除で
/// これを塞ぐ。
#[wasm_bindgen_test]
fn focusout_clears_press_state_left_by_keyboard_activation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-focusout-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // keyup を経由せずフォーカスが離れる（Tab で他要素へ移動した状況を
    // 模す）。
    dispatch_focusout(&child);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "keyup を待たずフォーカス離脱時点で press 状態が解除されること"
    );
}
