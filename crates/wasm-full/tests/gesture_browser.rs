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
    dispatch_focusout_with_related(target, None);
}

/// `relatedTarget`（新しくフォーカスを得た要素）を指定できる版。
/// 複合ウィジェット内でのフォーカス移動（`related` がまだ press_target
/// 配下）を模すのに使う。
fn dispatch_focusout_with_related(target: &Element, related: Option<&Element>) {
    let init = FocusEventInit::new();
    init.set_bubbles(true);
    if let Some(related) = related {
        init.set_related_target(Some(related.unchecked_ref::<EventTarget>()));
    }
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

/// cursor(Bugbot) 指摘の是正確認: `handle_keydown` は `event.target()` を
/// そのまま信頼し、`document.activeElement` による追加検証を行わない
/// （かつて存在した `is_still_focused` ガードは、プログラム的な
/// `dispatch_event`（実フォーカスを伴わない）で press が一切設定され
/// なくなる副作用を持っていたため撤去した）。`elsewhere` へ実際に
/// フォーカスが移っていても `child` への keydown 発火は `child` へ
/// press を設定する。
#[wasm_bindgen_test]
fn keydown_sets_press_based_on_event_target_regardless_of_actual_focus() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-keydown-focus-race-test");
    let elsewhere = document.create_element("button").unwrap();
    root.append_child(&elsewhere).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    elsewhere
        .unchecked_ref::<web_sys::HtmlElement>()
        .focus()
        .expect("focus must not fail");
    dispatch_key(&child, "keydown", "Enter", false);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "document.activeElement が別要素でも event.target() 基準で press を設定すること"
    );
}

/// codex-review 指摘の回帰固定（gesture.rs:211 付近）: 親子とも press
/// opt-in の場合に親の余白で `pointerdown` してから子上へドラッグして
/// `pointerup` すると、`pointerup` の `event.target()`（子）から祖先を
/// 再計算すると子だけが解除され、実際に press された親が残留する。
/// `active_pointer_press` で pointerdown 時の実要素を保持して解除する。
#[wasm_bindgen_test]
fn pointerup_on_nested_child_clears_the_actually_pressed_parent() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, parent, child) = build_dom(&document, "gesture-nested-press-test");
    child.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // 親の余白（parent 自身が target）で押下する。
    dispatch_pointer(&parent, "pointerdown", "mouse", None);
    assert!(
        parent.has_attribute(PRESS_STATE_ATTR),
        "親の余白での pointerdown は親へ press を設定する"
    );

    // 子上（target = child）で pointerup する。
    dispatch_pointer(&child, "pointerup", "mouse", None);
    assert!(
        !parent.has_attribute(PRESS_STATE_ATTR),
        "子上での pointerup でも実際に押下された親の press が解除されること"
    );
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "子は元々 press されていないため press 状態を持たないこと"
    );
}

/// cursor(Bugbot) 指摘の回帰固定（`handle_focusout` が `closest_opted_in`
/// ではなく [`opted_in_ancestors`] を辿るようになったことの確認）: 親子
/// とも press opt-in で親を pointerdown した状態から、子（opt-in）で
/// focusout（`relatedTarget` なし、真の離脱）すると、`closest_opted_in`
/// （focusout の target=子 から最も近い 1 件）では子自身が対象になり
/// 親の press が見逃されていた。祖先を辿ることで親も解除されること。
#[wasm_bindgen_test]
fn focusout_on_nested_child_clears_the_actually_pressed_parent() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, parent, child) = build_dom(&document, "gesture-nested-focusout-test");
    child.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&parent, "pointerdown", "mouse", None);
    assert!(parent.has_attribute(PRESS_STATE_ATTR));

    dispatch_focusout(&child);
    assert!(
        !parent.has_attribute(PRESS_STATE_ATTR),
        "子の focusout でも実際に押下された親の press が祖先探索で解除されること"
    );
}

/// codex-review 指摘の回帰固定: フォーカス移動を伴わずに複数ポインタ
/// （別指・別マウス）が別々の opt-in 要素を順に押下したとき、後発の
/// pointerdown が先発の押下要素の状態を上書きして解放漏れを起こさない
/// こと（`ActivePress` を `pointer_id` ごとに独立管理する）。
#[wasm_bindgen_test]
fn independent_pointers_press_and_release_without_clobbering_each_other() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-multi-pointer-test");
    let item_a = document.create_element("button").unwrap();
    item_a.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let item_b = document.create_element("button").unwrap();
    item_b.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    root.append_child(&item_a).unwrap();
    root.append_child(&item_b).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    let pointer_a = PointerEventInit::new();
    pointer_a.set_bubbles(true);
    pointer_a.set_pointer_type("touch");
    pointer_a.set_pointer_id(1);
    item_a
        .dispatch_event(
            PointerEvent::new_with_event_init_dict("pointerdown", &pointer_a)
                .expect("PointerEvent::new")
                .as_ref(),
        )
        .expect("dispatch_event must not fail");

    let pointer_b = PointerEventInit::new();
    pointer_b.set_bubbles(true);
    pointer_b.set_pointer_type("touch");
    pointer_b.set_pointer_id(2);
    item_b
        .dispatch_event(
            PointerEvent::new_with_event_init_dict("pointerdown", &pointer_b)
                .expect("PointerEvent::new")
                .as_ref(),
        )
        .expect("dispatch_event must not fail");

    assert!(
        item_a.has_attribute(PRESS_STATE_ATTR),
        "後発ポインタの押下で先発要素の press が上書きされないこと"
    );
    assert!(item_b.has_attribute(PRESS_STATE_ATTR));

    let pointer_a_up = PointerEventInit::new();
    pointer_a_up.set_bubbles(true);
    pointer_a_up.set_pointer_type("touch");
    pointer_a_up.set_pointer_id(1);
    item_a
        .dispatch_event(
            PointerEvent::new_with_event_init_dict("pointerup", &pointer_a_up)
                .expect("PointerEvent::new")
                .as_ref(),
        )
        .expect("dispatch_event must not fail");

    assert!(
        !item_a.has_attribute(PRESS_STATE_ATTR),
        "pointer_id が一致する pointerup で対象要素の press が解除されること"
    );
    assert!(
        item_b.has_attribute(PRESS_STATE_ATTR),
        "別ポインタの解放は無関係な要素の press に影響しないこと"
    );
}

/// Cursor Bugbot 指摘の回帰固定（`handle_focusout`）: press opt-in の
/// 複合ウィジェット自体（`root` 直下の `container`）でポインタを押下した
/// まま、内部の子要素間でフォーカスが移動しただけ（`relatedTarget` が
/// 依然 `container` 配下）では press を解除しないこと。真に `container`
/// の外へフォーカスが抜けたときのみ解除する。
#[wasm_bindgen_test]
fn focusout_within_pressed_composite_widget_does_not_clear_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-composite-focusout-test");
    let container = document.create_element("div").unwrap();
    container.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let item_a = document.create_element("button").unwrap();
    let item_b = document.create_element("button").unwrap();
    container.append_child(&item_a).unwrap();
    container.append_child(&item_b).unwrap();
    root.append_child(&container).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // container 自身（余白相当）で pointerdown する。
    dispatch_pointer(&container, "pointerdown", "mouse", None);
    assert!(container.has_attribute(PRESS_STATE_ATTR));

    // item_a から item_b への内部フォーカス移動（両方とも container 配下）。
    dispatch_focusout_with_related(&item_a, Some(&item_b));
    assert!(
        container.has_attribute(PRESS_STATE_ATTR),
        "ポインタ押下中に複合ウィジェット内でフォーカスが移動しただけでは press を解除しないこと"
    );

    // item_b から container の外（relatedTarget なし）へ真に離脱する。
    dispatch_focusout_with_related(&item_b, None);
    assert!(
        !container.has_attribute(PRESS_STATE_ATTR),
        "フォーカスが複合ウィジェットの外へ真に抜けたら press を解除すること"
    );
}
