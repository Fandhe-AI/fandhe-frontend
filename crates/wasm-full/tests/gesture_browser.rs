//! `fandhe_frontend_wasm_full::gesture::wire_gesture`（hover/press ジェス
//! チャー配線、イシュー #2520）の実ブラウザ統合テスト（`wasm-pack test
//! --headless --chrome`）。
//!
//! `wasm-full/src/gesture.rs` の native テストは純粋層
//! （`is_touch_pointer`/`is_press_activation_key`）までを検証済み。本
//! ファイルはその先、`wire_gesture` が実 DOM 上で pointerover/pointerout・
//! pointerdown/pointerup/pointercancel/pointermove・keydown/keyup/
//! focusout に応じて `data-fandhe-hover`/`data-fandhe-press` を正しく
//! 付け外しすることを
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

/// codex-review 指摘の回帰固定（gesture.rs:203 付近）: `pointerout` の
/// hover 解除に非タッチ判定がなかったため、マウスで hover 中の要素へ
/// タッチが重なる（`pointerType: "touch"` の `pointerout` が発火する）と
/// マウス由来の hover 状態まで誤って消えていた。`handle_pointerover` と
/// 対称に `handle_pointerout` も非タッチ限定にする。
#[wasm_bindgen_test]
fn pointerout_from_touch_does_not_clear_mouse_hover() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-touch-pointerout-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerover", "mouse", None);
    assert!(child.has_attribute(HOVER_STATE_ATTR));

    dispatch_pointer(&child, "pointerout", "touch", None);
    assert!(
        child.has_attribute(HOVER_STATE_ATTR),
        "タッチ由来の pointerout はマウス由来の hover 状態を解除しないこと"
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

/// Bugbot 指摘の回帰固定（「Nested focusout clears live pointer press」）:
/// `handle_focusout` は keyboard 押下源（`active_keyboard_press` の
/// `origin`）との一致のみを見て解除するため、そもそも `keydown` が
/// 一度も発火していない（keyboard 押下源が存在しない）この状況では
/// 一切属性へ触れない。親を pointerdown した状態から、無関係な子
/// （opt-in）が focusout（`relatedTarget` なし。window blur 等、フォーカスが
/// 完全に離れる場合を模す）しても、親の press は物理的なポインタが
/// まだ押されたままである以上、生き続ける（真の解除は
/// `pointerup`/`pointercancel`/`pointerout` にのみ委ねる）。
#[wasm_bindgen_test]
fn focusout_on_nested_child_does_not_clear_live_pointer_press_on_ancestor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, parent, child) = build_dom(&document, "gesture-nested-focusout-test");
    child.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&parent, "pointerdown", "mouse", None);
    assert!(parent.has_attribute(PRESS_STATE_ATTR));

    dispatch_focusout(&child);
    assert!(
        parent.has_attribute(PRESS_STATE_ATTR),
        "無関係な子孫のフォーカス喪失で、押下中の祖先の pointer press が誤って解除されないこと"
    );

    // 真の解除は pointerup が担う。
    dispatch_pointer(&parent, "pointerup", "mouse", None);
    assert!(!parent.has_attribute(PRESS_STATE_ATTR));
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

/// Bugbot 指摘の回帰固定（`handle_focusout`、「Nested focusout clears
/// live pointer press」）: press opt-in の複合ウィジェット自体（`root`
/// 直下の `container`）でポインタを押下したままなら、内部の子要素間の
/// フォーカス移動はもちろん、`container` の外へフォーカスが完全に抜けても
/// （`relatedTarget` なし。window blur 等）、pointer 由来の press は
/// `focusout` では解除しない（真の解除は `pointerup`/`pointercancel`/
/// `pointerout` にのみ委ねる。フォーカス変化は物理的なポインタの押下状態を
/// 表さないため）。
#[wasm_bindgen_test]
fn focusout_never_clears_pointer_originated_press() {
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

    // item_b から container の外（relatedTarget なし）へ真に離脱しても、
    // pointer 由来の press は focusout では解除されない。
    dispatch_focusout_with_related(&item_b, None);
    assert!(
        container.has_attribute(PRESS_STATE_ATTR),
        "pointer 由来の press はフォーカス喪失では解除されないこと（pointerup のみが解除する）"
    );

    dispatch_pointer(&container, "pointerup", "mouse", None);
    assert!(!container.has_attribute(PRESS_STATE_ATTR));
}

/// `pointer_id` を指定した `pointerdown`/`pointerup` の発火（2 本指等の
/// 複数ポインタ同時押下を模す）。
fn dispatch_pointer_with_id(target: &Element, kind: &str, pointer_id: i32) {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type("touch");
    init.set_pointer_id(pointer_id);
    target
        .dispatch_event(
            PointerEvent::new_with_event_init_dict(kind, &init)
                .expect("PointerEvent::new")
                .as_ref(),
        )
        .expect("dispatch_event must not fail");
}

/// codex-review 指摘の回帰固定（gesture.rs:266 付近、同型: :216/:309）:
/// 同一要素を 2 本指（別 `pointer_id`）で押下した状態から片方だけ離しても、
/// もう片方の `pointer_id` がまだ押下中なら press 状態を維持すること。
/// 押下源を要素単位で集約し、全ポインタが解放されたときのみ解除する。
#[wasm_bindgen_test]
fn two_pointers_on_same_element_require_both_to_release_before_clearing_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-two-finger-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer_with_id(&child, "pointerdown", 1);
    dispatch_pointer_with_id(&child, "pointerdown", 2);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_pointer_with_id(&child, "pointerup", 1);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "同一要素への 2 本目の指がまだ押下中なら press 状態を維持すること"
    );

    dispatch_pointer_with_id(&child, "pointerup", 2);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "全ポインタが解放されたら press 状態を解除すること"
    );
}

/// codex-review 指摘の回帰固定（gesture.rs:341 付近）: 親自身が press
/// opt-in で Space（keydown）を受けて press を持った状態から、フォーカスが
/// 入れ子の子（同じく press opt-in）へ移動すると、`relatedTarget` が
/// 依然親配下にあるという理由だけで親自身の press が残留していた。
/// `press_target` が focusout の対象そのもの（親自身）である場合は
/// `relatedTarget` の位置に関わらず必ず解除する。
#[wasm_bindgen_test]
fn focusout_clears_own_keyboard_press_even_when_related_target_stays_inside() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, parent, child) = build_dom(&document, "gesture-parent-space-to-child-test");
    child.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&parent, "keydown", " ", false);
    assert!(parent.has_attribute(PRESS_STATE_ATTR));

    // 親自身が focusout の対象（フォーカスが子へ移動）。relatedTarget
    // （子）は親配下だが、親自身が直接フォーカスを失った以上、対応する
    // keyup は二度と親へ届かないため必ず解除する。
    dispatch_focusout_with_related(&parent, Some(&child));
    assert!(
        !parent.has_attribute(PRESS_STATE_ATTR),
        "press_target 自身が focusout の対象なら relatedTarget の位置に関わらず解除すること"
    );
}

/// codex-review 指摘の回帰固定（gesture.rs:292 付近）: opt-in 要素の
/// keydown ハンドラ（アプリケーションコード）が同期的に別要素へ
/// `focus()` すると、bubble フェーズ登録では target 自身のハンドラが
/// 先に走って focusout が完了した後に press が設定され、二度と解除
/// されず残留していた。`wire_gesture` は `keydown` を capture フェーズで
/// 登録するため、root のリスナーが target 自身のあらゆる bubble
/// リスナーより先に実行され、press 設定時点でまだ真にフォーカスを
/// 保持している。その後の `focus()` 呼び出しに伴う `focusout` が
/// 通常どおり即座に解除する。
#[wasm_bindgen_test]
fn keydown_handler_calling_focus_synchronously_does_not_leave_stale_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-keydown-sync-focus-test");
    let elsewhere = document.create_element("button").unwrap();
    root.append_child(&elsewhere).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // アプリケーションコードが child 自身へ登録した keydown ハンドラ
    // （bubble フェーズ、既定）。root の capture リスナーより後に実行
    // される想定で、同期的に別要素へフォーカスを移す。
    let elsewhere_for_closure = elsewhere.clone();
    let app_keydown = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |_event| {
        elsewhere_for_closure
            .clone()
            .unchecked_into::<web_sys::HtmlElement>()
            .focus()
            .expect("focus must not fail");
    });
    child
        .add_event_listener_with_callback("keydown", app_keydown.as_ref().unchecked_ref())
        .expect("add_event_listener_with_callback must not fail");
    app_keydown.forget();

    // `elsewhere.focus()` が実際に `focusout(child)` を発火するには、
    // `child` が実際にフォーカスを保持している必要がある（実際の
    // ユーザー操作を模す）。
    child
        .unchecked_ref::<web_sys::HtmlElement>()
        .focus()
        .expect("focus must not fail");

    dispatch_key(&child, "keydown", " ", false);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "keydown ハンドラが同期的にフォーカスを移しても press が残留しないこと"
    );
}

/// 状態モデル再設計（PR #2555 レビュー指摘の是正）の回帰固定: 同一要素を
/// pointer と keyboard の両方で押下した状態から、pointer 側だけ解放しても
/// keyboard 側がまだ活性化中なら press 状態を維持し、keyboard 側の解放で
/// 初めて解除すること（「押下状態をポインタとキーボードの両入力源から
/// 集約する」の是正）。
#[wasm_bindgen_test]
fn pointer_release_does_not_clear_press_while_keyboard_still_active() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) =
        build_dom(&document, "gesture-mixed-source-pointer-first-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerdown", "mouse", None);
    dispatch_key(&child, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_pointer(&child, "pointerup", "mouse", None);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "keyboard 側がまだ活性化中なら pointerup だけで press を解除しないこと"
    );

    dispatch_key(&child, "keyup", " ", false);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "両方の押下源が解放されたら press 状態を解除すること"
    );
}

/// 上記と逆順（keyboard を先に押し、pointer を後から押す）でも同じ契約が
/// 成り立つこと。keyup で pointer 側の press が誤って解除されないことを
/// 確認する（従来実装は `handle_keyup` が pointer 側の状態を確認せず
/// 無条件に解除していた）。
#[wasm_bindgen_test]
fn keyboard_release_does_not_clear_press_while_pointer_still_active() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) =
        build_dom(&document, "gesture-mixed-source-keyboard-first-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", " ", false);
    dispatch_pointer(&child, "pointerdown", "mouse", None);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_key(&child, "keyup", " ", false);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "pointer 側がまだ活性化中なら keyup だけで press を解除しないこと"
    );

    dispatch_pointer(&child, "pointerup", "mouse", None);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "両方の押下源が解放されたら press 状態を解除すること"
    );
}

/// codex-review 指摘の回帰固定（「子孫間のフォーカス移動でも元の
/// キーボード押下を解除する」）: press opt-in の親 `container` 配下に、
/// opt-in なしの `item_a` と opt-in ありの `item_b` がある場合、`item_a`
/// で Space を押すと `container` に press が設定される。そのまま
/// `item_b` へフォーカスが移動しても、`item_a` 自身が focusout の対象
/// （`origin`）である以上、`container` の press を確実に解除すること
/// （旧実装は `relatedTarget`（`item_b`）が `container` 配下に留まる
/// という理由だけで解除を省略していた）。
#[wasm_bindgen_test]
fn focusout_on_non_opted_in_descendant_clears_ancestor_press_even_when_focus_moves_to_opted_in_sibling(
) {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-descendant-focus-move-test");
    let container = document.create_element("div").unwrap();
    container.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let item_a = document.create_element("button").unwrap();
    let item_b = document.create_element("button").unwrap();
    item_b.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    container.append_child(&item_a).unwrap();
    container.append_child(&item_b).unwrap();
    root.append_child(&container).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // item_a（opt-in なし）で Space を押すと、closest opt-in 祖先である
    // container へ press が設定される。
    dispatch_key(&item_a, "keydown", " ", false);
    assert!(container.has_attribute(PRESS_STATE_ATTR));

    // item_a から item_b（container 配下の別要素）へフォーカスが移動する。
    dispatch_focusout_with_related(&item_a, Some(&item_b));
    assert!(
        !container.has_attribute(PRESS_STATE_ATTR),
        "keydown の発生元（item_a）自身が focusout した以上、relatedTarget の位置に\
         関わらず対応する container の press を解除すること"
    );
}

/// Bugbot 指摘の回帰固定（「Keydown capture leaves stale press」）:
/// `keyup` も `keydown` と同じく capture フェーズで登録するため、子孫
/// （grandchild）が自前の `keyup` ハンドラで `stopPropagation()` を
/// 呼んでバブルを止めても、root の capture リスナーは既に実行済みで
/// あり press 状態が正しく解除されること。
#[wasm_bindgen_test]
fn keyup_capture_registration_still_clears_press_even_if_descendant_stops_propagation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, grandchild) = build_dom(&document, "gesture-keyup-stop-propagation-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // 子孫（grandchild）自身の keyup ハンドラが stopPropagation する状況を
    // 模す（bubble フェーズ・既定登録）。
    let stop_propagation =
        wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            event.stop_propagation();
        });
    grandchild
        .add_event_listener_with_callback("keyup", stop_propagation.as_ref().unchecked_ref())
        .expect("add_event_listener_with_callback must not fail");
    stop_propagation.forget();

    dispatch_key(&grandchild, "keyup", " ", false);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "keyup が capture 登録のため子孫の stopPropagation の影響を受けず press が\
         解除されること"
    );
}

/// codex-review 指摘の回帰固定（「focusout も capture フェーズで購読
/// する」）: 中間祖先（`child`）自身の `focusout` ハンドラが
/// `stopPropagation()` を呼ぶ状況でも、root の capture リスナーはその
/// 中間祖先へ到達するより前に実行済みであり press が正しく解除される
/// こと。
#[wasm_bindgen_test]
fn focusout_capture_registration_still_clears_press_even_if_descendant_stops_propagation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, grandchild) = build_dom(&document, "gesture-focusout-stop-propagation-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    // grandchild（opt-in なし）で Space を押すと、closest opt-in 祖先で
    // ある child へ press が設定される。
    dispatch_key(&grandchild, "keydown", " ", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // child（grandchild と root の間にある中間祖先）自身の focusout
    // ハンドラが stopPropagation する状況を模す（bubble フェーズ・既定
    // 登録）。
    let stop_propagation =
        wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            event.stop_propagation();
        });
    child
        .add_event_listener_with_callback("focusout", stop_propagation.as_ref().unchecked_ref())
        .expect("add_event_listener_with_callback must not fail");
    stop_propagation.forget();

    dispatch_focusout(&grandchild);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "focusout が capture 登録のため中間祖先の stopPropagation の影響を受けず press が\
         解除されること"
    );
}

/// codex-review 指摘の回帰固定（「キーボード押下をキーごとに追跡
/// する」）: 同一要素で Space と Enter を同時に押した状態から片方だけ
/// 離しても、もう片方が活性化中なら press を解除しないこと。
#[wasm_bindgen_test]
fn space_and_enter_pressed_together_require_both_to_release_before_clearing_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-space-enter-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&child, "keydown", " ", false);
    dispatch_key(&child, "keydown", "Enter", false);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    dispatch_key(&child, "keyup", " ", false);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "Enter がまだ活性化中なら Space の keyup だけで press を解除しないこと"
    );

    dispatch_key(&child, "keyup", "Enter", false);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "両方の活性化キーが解放されたら press を解除すること"
    );
}

/// Bugbot 指摘の回帰固定（「Pointer press sticks after stop」）:
/// 子孫（`grandchild`）自身の `pointerup` ハンドラが `stopPropagation()`
/// を呼んでバブルを止めても、root の capture リスナーは既に実行済みで
/// あり press 状態が正しく解除されること（`keyup` の capture 化と対称）。
#[wasm_bindgen_test]
fn pointerup_capture_registration_still_clears_press_even_if_descendant_stops_propagation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, grandchild) = build_dom(&document, "gesture-pointerup-stop-propagation-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerdown", "mouse", None);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // 子孫（grandchild）自身の pointerup ハンドラが stopPropagation する
    // 状況を模す（bubble フェーズ・既定登録）。
    let stop_propagation =
        wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            event.stop_propagation();
        });
    grandchild
        .add_event_listener_with_callback("pointerup", stop_propagation.as_ref().unchecked_ref())
        .expect("add_event_listener_with_callback must not fail");
    stop_propagation.forget();

    dispatch_pointer(&grandchild, "pointerup", "mouse", None);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "pointerup が capture 登録のため子孫の stopPropagation の影響を受けず press が\
         解除されること"
    );
}

/// Bugbot 指摘の回帰固定（「Pointer id reuse leaks press」）:
/// `pointerup`/`pointercancel` を取りこぼした状態のまま同じ
/// `pointer_id` が別の opt-in 要素へ再利用されると、`handle_pointerdown`
/// が旧押下対象を解除せずに上書きしていたため、最初に押下した要素の
/// press 状態が永続的に残留していた。新しい `pointerdown` が旧押下対象を
/// 正しく解除しつつ、新しい対象へ press を設定すること。
#[wasm_bindgen_test]
fn pointerdown_with_reused_pointer_id_clears_stale_previous_target_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-pointer-id-reuse-test");
    let item_a = document.create_element("button").unwrap();
    item_a.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let item_b = document.create_element("button").unwrap();
    item_b.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    root.append_child(&item_a).unwrap();
    root.append_child(&item_b).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer_with_id(&item_a, "pointerdown", 7);
    assert!(item_a.has_attribute(PRESS_STATE_ATTR));

    // pointerup/pointercancel を取りこぼした状態のまま、同じ pointer_id
    // （7）が別要素（item_b）へ再利用される状況を模す。
    dispatch_pointer_with_id(&item_b, "pointerdown", 7);

    assert!(
        !item_a.has_attribute(PRESS_STATE_ATTR),
        "pointer_id の再利用時、旧押下対象（item_a）の press は解除されること"
    );
    assert!(
        item_b.has_attribute(PRESS_STATE_ATTR),
        "新しい押下対象（item_b）へ press が設定されること"
    );
}

/// `client_x`/`client_y` 座標を指定した `pointermove` を発火する
/// （タッチの暗黙 pointer capture 下での `elementFromPoint` による
/// ヒットテストを検証するため）。
fn dispatch_pointermove_at(target: &Element, client_x: i32, client_y: i32) {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type("touch");
    init.set_client_x(client_x);
    init.set_client_y(client_y);
    let event = PointerEvent::new_with_event_init_dict("pointermove", &init)
        .expect("PointerEvent::new must not fail");
    target
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
}

/// codex-review 指摘の回帰固定（「タッチの暗黙 pointer capture 中も
/// 要素外への離脱を検知する」）: `touch-action: none` を持つ opt-in
/// 要素をタッチで押下すると暗黙 pointer capture が働き、指を要素の外へ
/// 動かしても境界イベント（`pointerout`）が発生しない。`pointermove`
/// （すべてタッチの合成イベント。`elementFromPoint` によるヒットテスト
/// はタッチ限定の補完のため）でこの残余ケースを補完し、要素外座標への
/// `pointermove` で press が解除されること（要素内座標では維持される
/// こと）を確認する。
#[wasm_bindgen_test]
fn pointermove_outside_bounds_clears_press_during_implicit_capture() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, child, _grandchild) = build_dom(&document, "gesture-pointermove-capture-test");
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&child, "pointerdown", "touch", None);
    assert!(child.has_attribute(PRESS_STATE_ATTR));

    // 要素内の座標（実際の矩形の中心）への pointermove は press を維持する。
    let rect = child.get_bounding_client_rect();
    let inside_x = ((rect.left() + rect.right()) / 2.0) as i32;
    let inside_y = ((rect.top() + rect.bottom()) / 2.0) as i32;
    dispatch_pointermove_at(&child, inside_x, inside_y);
    assert!(
        child.has_attribute(PRESS_STATE_ATTR),
        "要素内座標への pointermove は press を維持すること"
    );

    // 要素外の座標（大きく離れた座標）への pointermove は、暗黙 pointer
    // capture 下で pointerout が一切発火しない状況でも press を解除する
    // こと。
    dispatch_pointermove_at(&child, -9999, -9999);
    assert!(
        !child.has_attribute(PRESS_STATE_ATTR),
        "要素外座標への pointermove は press を解除すること（暗黙 pointer capture の補完）"
    );
}

/// codex P1・Bugbot 指摘の回帰固定（「親の矩形外にある子孫上で press を
/// 誤解除しない」「Pointermove clears press too early」）: `overflow:
/// visible` で押下対象（`parent`）の矩形からはみ出した子孫
/// （`overflowing_child`）上に指がある状態で `pointermove` が発火しても、
/// `elementFromPoint` が子孫自身（＝押下対象の子孫）をヒットする限り
/// press を解除しないこと。矩形の内外比較（旧実装）ではこのケースを
/// 「離脱」と誤判定していた。
#[wasm_bindgen_test]
fn pointermove_over_overflowing_descendant_keeps_press_during_implicit_capture() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-pointermove-overflow-descendant-test");
    let parent = document.create_element("div").unwrap();
    parent.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    parent
        .unchecked_ref::<web_sys::HtmlElement>()
        .style()
        .set_property("position", "relative")
        .expect("set_property must not fail");

    // 子孫を parent の矩形外（下方向、ビューポート内に収まる正の座標）へ
    // 絶対配置で押し出す。overflow の既定値は visible のため、明示的な
    // overflow 宣言をしなくても子孫はクリック可能な位置に描画される。
    let overflowing_child = document.create_element("span").unwrap();
    let overflowing_child_style = overflowing_child
        .unchecked_ref::<web_sys::HtmlElement>()
        .style();
    overflowing_child_style
        .set_property("position", "absolute")
        .expect("set_property must not fail");
    overflowing_child_style
        .set_property("top", "120px")
        .expect("set_property must not fail");
    overflowing_child_style
        .set_property("left", "0px")
        .expect("set_property must not fail");
    overflowing_child_style
        .set_property("width", "50px")
        .expect("set_property must not fail");
    overflowing_child_style
        .set_property("height", "50px")
        .expect("set_property must not fail");
    overflowing_child_style
        .set_property("display", "block")
        .expect("set_property must not fail");
    parent.append_child(&overflowing_child).unwrap();
    root.append_child(&parent).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_pointer(&parent, "pointerdown", "touch", None);
    assert!(parent.has_attribute(PRESS_STATE_ATTR));

    let child_rect = overflowing_child.get_bounding_client_rect();
    let x = ((child_rect.left() + child_rect.right()) / 2.0) as i32;
    let y = ((child_rect.top() + child_rect.bottom()) / 2.0) as i32;

    // テスト前提の確認 1: 子孫が実際に parent の矩形からはみ出していること
    // （parent は空の div のため既定で高さがほぼ 0 になる）。
    let parent_rect = parent.get_bounding_client_rect();
    assert!(
        f64::from(y) > parent_rect.bottom(),
        "テスト前提: 子孫の座標が親の矩形の下端より外側にあること"
    );
    // テスト前提の確認 2: その座標で実際に elementFromPoint が子孫自身を
    // ヒットすること（レイアウト前提が崩れた場合に「press が解除された」
    // という誤解を招くメッセージではなく、前提条件の失敗として明示する）。
    assert!(
        document
            .element_from_point(x as f32, y as f32)
            .is_some_and(|hit| overflowing_child.contains(Some(&hit))),
        "テスト前提: elementFromPoint が子孫（overflowing_child）をヒットすること"
    );

    dispatch_pointermove_at(&overflowing_child, x, y);
    assert!(
        parent.has_attribute(PRESS_STATE_ATTR),
        "親の矩形外にはみ出した子孫上の pointermove では press を維持すること"
    );
}

/// Bugbot 指摘の回帰固定（「Typing space triggers ancestor press」）:
/// opt-in 親の内側に置かれた `input` へ通常のテキスト入力として Space を
/// 打つたびに、祖先の `PRESS_STATE_ATTR` が点滅してはならない。
#[wasm_bindgen_test]
fn keydown_on_editable_descendant_does_not_trigger_ancestor_press() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("gesture-editable-descendant-test");
    let parent = document.create_element("div").unwrap();
    parent.set_attribute(GESTURE_PRESS_ATTR, "").unwrap();
    let input = document.create_element("input").unwrap();
    parent.append_child(&input).unwrap();
    root.append_child(&parent).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());
    wire_gesture(root.clone()).expect("wire_gesture must not fail");

    dispatch_key(&input, "keydown", " ", false);
    assert!(
        !parent.has_attribute(PRESS_STATE_ATTR),
        "input 上での Space の通常入力は祖先の press を設定しないこと"
    );

    dispatch_key(&input, "keyup", " ", false);
    assert!(!parent.has_attribute(PRESS_STATE_ATTR));

    // Enter・textarea・contenteditable でも同様に press を設定しない。
    let textarea = document.create_element("textarea").unwrap();
    parent.append_child(&textarea).unwrap();
    dispatch_key(&textarea, "keydown", "Enter", false);
    assert!(!parent.has_attribute(PRESS_STATE_ATTR));

    let editable_div = document.create_element("div").unwrap();
    editable_div
        .set_attribute("contenteditable", "true")
        .unwrap();
    parent.append_child(&editable_div).unwrap();
    dispatch_key(&editable_div, "keydown", " ", false);
    assert!(
        !parent.has_attribute(PRESS_STATE_ATTR),
        "contenteditable 要素上の Space も祖先の press を設定しないこと"
    );
}
