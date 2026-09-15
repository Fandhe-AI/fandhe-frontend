//! `fandhe_frontend_wasm_full::drag_gesture::wire_drag_gesture`（pointer
//! capture ベースの汎用ドラッグ配線、イシュー #2535）の実ブラウザ統合
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/drag_gesture.rs` の native テストは純粋層
//! （`arrow_key_direction`/`parse_drag_axis`）までを検証済み。本ファイルは
//! その先、`wire_drag_gesture` が実 DOM 上で pointerdown/pointermove/
//! pointerup・keydown（矢印キー）に応じて `--fandhe-drag-x`/`-y`
//! （`fandhe_frontend_animation::drag::DragController` が書き込む CSS
//! カスタムプロパティ）・`data-fandhe-dragging` を正しく更新することを
//! `gesture_browser.rs` と同方針（手組み DOM への直接 `wire_drag_gesture`
//! 呼び出し、`Runtime::mount` は経由しない）で検証する。
//!
//! `setPointerCapture` は合成イベント環境で `NotFoundError` を投げうる
//! （`angle_slider.rs::reattach_pointer_capture` doc 参照）ため、
//! アサーションは `has_pointer_capture` ではなく `data-fandhe-dragging` の
//! 付け外しと `--fandhe-drag-x`/`-y` の値で行う。制約コンテナ
//! （`DRAG_CONSTRAINTS_ATTR`）を持たない構成のみを検証するため、release 時
//! の最終位置は常に release 直前の現在位置と一致し（spring は起動しない）、
//! ブラウザの `prefers-reduced-motion` 設定に依存しない決定的な結果になる。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "drag-gesture")]

use fandhe_frontend_animation::drag::{DRAG_X_PROPERTY, DRAG_Y_PROPERTY};
use fandhe_frontend_wasm_full::drag_gesture::{
    wire_drag_gesture, DRAGGING_STATE_ATTR, DRAG_ATTR, DRAG_AXIS_ATTR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, KeyboardEventInit};
use web_sys::{PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::RemoveOnDrop`
/// と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// root（opt-in 属性なし）> draggable（`DRAG_ATTR` opt-in、`tabindex="0"`
/// でキーボード操作可能）の 2 階層 DOM を組み立てる。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let draggable = document.create_element("div").unwrap();
    draggable.set_attribute(DRAG_ATTR, "").unwrap();
    draggable.set_attribute("tabindex", "0").unwrap();
    root.append_child(&draggable).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, draggable)
}

/// `buttons` を 1（メインボタン押下中）で組み立てる既定ヘルパ。
/// `handle_pointermove` は `buttons() == 0` を stale 追跡の自己解除条件に
/// 使う（`pointermove_with_no_buttons_pressed_self_heals_stale_tracking`
/// 参照）ため、押下中を模す既存テスト群は `buttons` を明示しておく必要が
/// ある（実ブラウザの `pointermove` は押下中は非 0 の `buttons` を持つ）。
fn pointer_event(kind: &str, pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    pointer_event_with_buttons(kind, pointer_id, client_x, client_y, 1)
}

fn dispatch_key(target: &Element, kind: &str, key: &str) {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init)
        .expect("KeyboardEvent::new must not fail");
    target
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
}

/// `element` の `name` カスタムプロパティを `f64` として読む（未設定は
/// `None`）。`fandhe_frontend_animation::drag::write_dom` は px 単位付きで
/// 書き込む（`DomTarget::style_property(..., "px")`、codex-review/Bugbot
/// 是正）ため、末尾の `"px"` を剥がしてから数値へパースする。
fn custom_property_px(element: &Element, name: &str) -> Option<f64> {
    let value = element
        .dyn_ref::<HtmlElement>()
        .expect("element must be HtmlElement")
        .style()
        .get_property_value(name)
        .expect("get_property_value must not fail");
    if value.is_empty() {
        None
    } else {
        value
            .strip_suffix("px")
            .unwrap_or(&value)
            .parse::<f64>()
            .ok()
    }
}

#[wasm_bindgen_test]
fn pointer_drag_updates_position_and_dragging_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-pointer-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 1, 100.0, 100.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointerdown 後は data-fandhe-dragging が付与されているべき"
    );

    root.dispatch_event(&pointer_event("pointermove", 1, 140.0, 130.0))
        .expect("dispatch_event must not fail");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        (x - 40.0).abs() < 0.01,
        "pointermove の client_x 差分 40px がそのまま反映されるべき: x={x}"
    );
    assert!(
        (y - 30.0).abs() < 0.01,
        "pointermove の client_y 差分 30px がそのまま反映されるべき: y={y}"
    );

    root.dispatch_event(&pointer_event("pointerup", 1, 140.0, 130.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointerup 後は data-fandhe-dragging が外れているべき"
    );
    // 制約コンテナが無いため release 時のクランプは no-op で、位置は
    // pointerup 直前の値のまま維持される（モジュール doc 参照）。
    let x_after = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    assert!(
        (x_after - 40.0).abs() < 0.01,
        "制約が無い release は位置を変えないはず: x_after={x_after}"
    );
}

#[wasm_bindgen_test]
fn pointer_cancel_also_clears_dragging_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-cancel-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 2, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(draggable.has_attribute(DRAGGING_STATE_ATTR));

    root.dispatch_event(&pointer_event("pointercancel", 2, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointercancel 後も data-fandhe-dragging が外れているべき"
    );
}

#[wasm_bindgen_test]
fn arrow_key_nudges_position_respecting_axis() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-keyboard-root");
    draggable.set_attribute(DRAG_AXIS_ATTR, "x").unwrap();
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    dispatch_key(&draggable, "keydown", "ArrowRight");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        x > 0.0,
        "ArrowRight は x を正方向へ動かすべき（1 ステップ分）: x={x}"
    );
    assert!(
        y == 0.0,
        "data-fandhe-drag-axis=\"x\" のとき ArrowUp/Down 相当の y は動かないべき: y={y}"
    );

    dispatch_key(&draggable, "keydown", "ArrowUp");
    let y_after_up = custom_property_px(&draggable, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        y_after_up == 0.0,
        "軸制約 x では ArrowUp は無視されるべき: y_after_up={y_after_up}"
    );
}

#[wasm_bindgen_test]
fn arrow_key_on_editable_target_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-editable-root");
    let _guard = RemoveOnDrop(root.clone());

    let input = document.create_element("input").unwrap();
    draggable.append_child(&input).unwrap();

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    dispatch_key(&input, "keydown", "ArrowRight");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY);
    assert!(
        x.is_none(),
        "input 上の矢印キーは drag nudge を発火しないべき: x={x:?}"
    );
}

/// Bugbot 是正の回帰（「Arrow keys captured from descendants」、PR #2565
/// 第 4 ラウンド）: opt-in 要素の子孫にある `<button>` へフォーカスが
/// あるときの矢印キーは、`input`/`textarea`/`select`/`contenteditable`
/// のいずれでもないにもかかわらず nudge を発火しない（子孫ウィジェット
/// 自身の矢印キー操作を奪わない）。
#[wasm_bindgen_test]
fn arrow_key_on_descendant_button_is_not_captured() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-descendant-button-root");
    let _guard = RemoveOnDrop(root.clone());

    let button = document.create_element("button").unwrap();
    draggable.append_child(&button).unwrap();

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    dispatch_key(&button, "keydown", "ArrowRight");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY);
    assert!(
        x.is_none(),
        "子孫 button 上の矢印キーは drag nudge を発火しないべき: x={x:?}"
    );
}

/// `buttons` 値を明示できる `pointer_event` 拡張版
/// （既定 `pointer_event` は `buttons` 未指定＝ 0 のため、押下中の移動を
/// 模すテストは本ヘルパを使う）。
fn pointer_event_with_buttons(
    kind: &str,
    pointer_id: i32,
    client_x: f64,
    client_y: f64,
    buttons: u16,
) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    init.set_buttons(buttons);
    // `handle_pointerdown` の「メインボタン（0）かつ最初の接触点のみ
    // ドラッグを開始する」ガード（Bugbot 是正「Non-primary buttons
    // start drags」、PR #2565 第 4 ラウンド）への対応。`PointerEventInit`
    // の既定値は `button = 0`・`isPrimary = false`（W3C Pointer Events
    // 仕様）であり、`is_primary` を明示しないと本ヘルパで組み立てた
    // 合成イベントはすべてガードで無視されてしまう。実ブラウザの
    // メインボタン押下・単一タッチ接触を模すため、両方を明示する。
    init.set_button(0);
    init.set_is_primary(true);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("PointerEvent must cast to Event")
}

/// codex-review P1 是正の回帰: 同一要素へ 2 本目のポインタが
/// `pointerdown` しても起点は上書きされず、最初のポインタのみが
/// ドラッグを所有する（2 本目の `pointerdown` は無視される）。
#[wasm_bindgen_test]
fn second_pointerdown_on_same_element_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-multi-pointer-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 1, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    // 2 本目の指（別 pointer_id）が同じ要素へ pointerdown しても無視される。
    draggable
        .dispatch_event(&pointer_event("pointerdown", 2, 50.0, 50.0))
        .expect("dispatch_event must not fail");

    // 2 本目の pointer_id での move は追跡されていないため無視され、
    // 1 本目の起点（client 0,0）のままの移動量が反映される。
    root.dispatch_event(&pointer_event_with_buttons("pointermove", 1, 10.0, 0.0, 1))
        .expect("dispatch_event must not fail");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    assert!(
        (x - 10.0).abs() < 0.01,
        "1 本目の起点からの移動量がそのまま反映されるべき: x={x}"
    );

    // 2 本目の pointer_id での pointerup は「追跡なし」として無視され、
    // 1 本目のドラッグはまだ dragging 状態を維持する。
    root.dispatch_event(&pointer_event("pointerup", 2, 50.0, 50.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "2 本目 pointer_id の pointerup は無関係のため dragging 状態は維持されるべき"
    );

    root.dispatch_event(&pointer_event("pointerup", 1, 10.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "1 本目 pointer_id の pointerup で dragging 状態が外れるべき"
    );
}

/// Bugbot 指摘の是正回帰: `set_pointer_capture` 失敗等で `pointerup`/
/// `pointercancel` が `root` に届かない場合でも、`buttons() == 0` の
/// `pointermove` 1 件で追跡が自己解除される（幽霊ドラッグにならない）。
#[wasm_bindgen_test]
fn pointermove_with_no_buttons_pressed_self_heals_stale_tracking() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-stale-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 3, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(draggable.has_attribute(DRAGGING_STATE_ATTR));

    // pointerup を取り逃した状態を模す: buttons=0 の pointermove が
    // 届くと、それだけで追跡が解除される（root 外での release を捕まえ
    // 損ねても幽霊ドラッグにならない）。
    root.dispatch_event(&pointer_event_with_buttons("pointermove", 3, 5.0, 5.0, 0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "buttons() == 0 の pointermove で dragging 状態が自己解除されるべき"
    );
}

/// Cursor Bugbot 指摘「Stale drag can lock element」是正の回帰
/// （PR #2565）: `root` の外（`document` 自身）で発生した `pointerup` は
/// `root` の capture リスナーには到達しないが、`wire_drag_gesture` が
/// `window` にも追加登録した capture リスナーが解放する。解放後は同じ
/// `pointer_id` で新規ドラッグを開始できる。
#[wasm_bindgen_test]
fn pointerup_outside_root_is_caught_by_window_listener() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-outside-release-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 11, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(draggable.has_attribute(DRAGGING_STATE_ATTR));

    // `root.dispatch_event` ではなく `document.dispatch_event` を使い、
    // root のサブツリー外で発生した release を模す。
    document
        .dispatch_event(&pointer_event("pointerup", 11, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "root 外で発生した pointerup も window リスナーで解放されるべき"
    );

    // 解放済みのため、同じ pointer_id で新規ドラッグを開始できる
    // （幽霊ドラッグとして要素をロックし続けない）。
    draggable
        .dispatch_event(&pointer_event("pointerdown", 11, 20.0, 20.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "解放済みの pointer_id は新規ドラッグを開始できるべき"
    );
}

/// Cursor Bugbot 指摘の是正回帰（第 2 の防御層）: `root`・`window` の
/// いずれにも release イベントが一切到達しなかった極端なケース（capture
/// 完全失陥等）でも、同一 `pointer_id` の再 `pointerdown`（UA は release
/// 済みの `pointer_id` しか再利用しないため、これ自体が stale の証拠）で
/// [`handle_pointerdown`] が自己解除し、新規ドラッグの起点を再設定する。
#[wasm_bindgen_test]
fn pointerdown_with_stale_same_pointer_id_self_heals() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-stale-pointerdown-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event("pointerdown", 21, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(draggable.has_attribute(DRAGGING_STATE_ATTR));

    // release イベントが一切届かなかった状況を模し、同じ pointer_id で
    // 再度 pointerdown する。
    draggable
        .dispatch_event(&pointer_event("pointerdown", 21, 5.0, 5.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "同一 pointer_id の再 pointerdown は stale を解除し新規ドラッグを開始するべき"
    );

    root.dispatch_event(&pointer_event("pointermove", 21, 15.0, 5.0))
        .expect("dispatch_event must not fail");
    let x = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    assert!(
        (x - 10.0).abs() < 0.01,
        "起点は再 pointerdown 時の座標 (5, 5) で更新されているべき: x={x}"
    );
}

/// codex-review P1・Cursor Bugbot 是正の回帰「Keyboard nudge desyncs
/// pointer drag」（PR #2565）: pointer ドラッグ進行中に矢印キーで nudge
/// すると、`handle_keydown` が配線層（`active` マップ・
/// `DRAGGING_STATE_ATTR`）を先に正規解放してから nudge を適用するため、
/// 中断後の pointerup も新規 pointerdown も正常に機能する。
#[wasm_bindgen_test]
fn keyboard_nudge_interrupts_pointer_drag_without_desync() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-keyboard-interrupt-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    // pointer ドラッグを開始し、途中まで移動する。
    draggable
        .dispatch_event(&pointer_event("pointerdown", 41, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event_with_buttons("pointermove", 41, 10.0, 0.0, 1))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "pointer ドラッグ中は dragging 状態を持つべき"
    );

    // ドラッグ中に矢印キーで nudge する。
    dispatch_key(&draggable, "keydown", "ArrowRight");

    // nudge が中断済み pointer ドラッグの配線状態を正規解放するため、
    // dragging 状態は外れているべき（コントローラ側だけが解放され配線層
    // が取り残される不整合の回帰）。
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "キーボード nudge は進行中の pointer ドラッグを解放し dragging 状態を外すべき"
    );

    // 中断前の移動量 (10) の上に nudge 1 ステップ分が積まれているはず。
    let x_after_nudge = custom_property_px(&draggable, DRAG_X_PROPERTY).unwrap_or(0.0);
    assert!(
        x_after_nudge > 10.0,
        "nudge は中断前の位置の上に加算されるべき: x_after_nudge={x_after_nudge}"
    );

    // 中断済みの pointer_id での pointerup は「追跡なし」の no-op となり
    // dragging 状態を壊さない（release_drag が既に active から除去済み）。
    root.dispatch_event(&pointer_event("pointerup", 41, 10.0, 0.0))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "中断済み pointer_id の pointerup 後も dragging 状態は付かないべき"
    );

    // 中断で active マップのエントリが正しく除去されていれば、同じ
    // pointer_id で新規ドラッグを開始できる（配線層が取り残されていると
    // `already_tracked` 判定で新規 pointerdown が拒否される回帰）。
    draggable
        .dispatch_event(&pointer_event("pointerdown", 41, 20.0, 20.0))
        .expect("dispatch_event must not fail");
    assert!(
        draggable.has_attribute(DRAGGING_STATE_ATTR),
        "中断後は同じ pointer_id でも新規ドラッグを開始できるべき"
    );
}

/// `button`（0 以外）・`is_primary`（`false`）を明示できる
/// `pointer_event_with_buttons` 拡張版。Bugbot 是正「Non-primary buttons
/// start drags」の回帰テスト専用ヘルパ。
fn pointer_event_with_button_and_primary(
    kind: &str,
    pointer_id: i32,
    client_x: f64,
    client_y: f64,
    button: i16,
    is_primary: bool,
) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    init.set_buttons(1);
    init.set_button(button);
    init.set_is_primary(is_primary);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("PointerEvent must cast to Event")
}

/// Bugbot Medium 是正の回帰（PR #2565 第 4 ラウンド）: 右クリック
/// （`button() == 2`）の `pointerdown` はドラッグを開始しない
/// （`data-fandhe-dragging` が付かない・pointer capture を握らない）。
#[wasm_bindgen_test]
fn non_primary_button_pointerdown_does_not_start_drag() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-non-primary-button-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event_with_button_and_primary(
            "pointerdown",
            51,
            0.0,
            0.0,
            2,
            true,
        ))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "右クリック（button 2）の pointerdown はドラッグを開始しないべき"
    );
    assert!(
        !draggable.has_pointer_capture(51),
        "右クリックの pointerdown で pointer capture を握らないべき"
    );
}

/// Bugbot Medium 是正の回帰: `is_primary() == false`（2 本目以降の
/// タッチ接触等）の `pointerdown` もドラッグを開始しない。
#[wasm_bindgen_test]
fn non_primary_pointer_pointerdown_does_not_start_drag() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, draggable) = build_dom(&document, "drag-non-primary-pointer-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_drag_gesture(root.clone()).expect("wire_drag_gesture must not fail");

    draggable
        .dispatch_event(&pointer_event_with_button_and_primary(
            "pointerdown",
            52,
            0.0,
            0.0,
            0,
            false,
        ))
        .expect("dispatch_event must not fail");
    assert!(
        !draggable.has_attribute(DRAGGING_STATE_ATTR),
        "is_primary() が false の pointerdown はドラッグを開始しないべき"
    );
}
