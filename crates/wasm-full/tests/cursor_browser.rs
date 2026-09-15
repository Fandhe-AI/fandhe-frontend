//! `fandhe_frontend_wasm_full::cursor`（イシュー #2542）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/cursor.rs` の native テストは定数の安定性のみを検証
//! 済みである。本ファイルは opt-in 要素の解決・hover 対象の `data-*` 写し・
//! magnetic 吸着・タッチ除外・`reduced_motion`/`coarse_pointer` 抑制が
//! 実 DOM 上で機能することを `magnetic_browser.rs` と同方針で検証する。
//! `CursorAnimator` 自体の spring 収束は責務境界どおり
//! `crates/frontend-animation/tests/cursor_browser.rs` で検証済みのため、
//! 本ファイルは収束を待たず、配線（イベント委譲・要素解決・属性書き換え）
//! のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "cursor")]

use fandhe_frontend_wasm_full::cursor::{
    wire_cursor_with_env, CURSOR_ACTIVE_ATTR, CURSOR_ATTR, CURSOR_LABEL_ATTR, CURSOR_STATE_ATTR,
    CURSOR_TARGET_ATTR, CURSOR_TARGET_LABEL_ATTR, CURSOR_TARGET_MAGNETIC_ATTR, CURSOR_VARIANT_ATTR,
};
use js_sys::Promise;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement, PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`magnetic_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `root`（id 付き）> `cursor_el`（[`CURSOR_ATTR`] 付き）+ `target`
/// （`data-fandhe-cursor-target="ring"` + label 付き、`position: absolute;
/// left: 0; top: 0; width: 40px; height: 40px;` で中心座標を
/// `(20, 20)` に固定）を組み立てて返す。
fn build_dom(document: &Document, root_id: &str) -> (Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);

    let cursor_el = document.create_element("div").unwrap();
    cursor_el.set_attribute(CURSOR_ATTR, "").unwrap();
    root.append_child(&cursor_el).unwrap();

    let target = document.create_element("button").unwrap();
    target.set_attribute(CURSOR_TARGET_ATTR, "ring").unwrap();
    target
        .set_attribute(CURSOR_TARGET_LABEL_ATTR, "View")
        .unwrap();
    let html_target = target
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("target must cast to HtmlElement for style access");
    let style = html_target.style();
    style.set_property("position", "absolute").unwrap();
    style.set_property("left", "0px").unwrap();
    style.set_property("top", "0px").unwrap();
    style.set_property("width", "40px").unwrap();
    style.set_property("height", "40px").unwrap();
    root.append_child(&target).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, cursor_el, target)
}

fn dispatch_pointer_event(
    target: &Element,
    kind: &str,
    pointer_type: &str,
    client_x: i32,
    client_y: i32,
) {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type(pointer_type);
    init.set_client_x(client_x);
    init.set_client_y(client_y);
    let event = PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail");
    target
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
}

fn attr(element: &Element, name: &str) -> Option<String> {
    element.get_attribute(name)
}

#[wasm_bindgen_test]
fn wiring_does_not_mark_root_active_until_position_known() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, _target) = build_dom(&document, "cursor-root-1");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    // 有効な座標がまだ無い配線直後は CURSOR_ACTIVE_ATTR を付与しない
    // （ネイティブカーソルは隠さない）。PR #2583 レビュー指摘 P1: 無条件
    // 付与すると、ネイティブカーソルだけが先に隠れカスタムカーソルは
    // hidden のままという「両方消える」窓が生じていた。
    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), None);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));
}

#[wasm_bindgen_test]
fn pointermove_over_target_copies_variant_and_label_to_cursor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-2");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    // 初回の有効な pointermove で CURSOR_ACTIVE_ATTR が付与される
    // （カスタムカーソルの表示可能化と同時にネイティブカーソルを隠す）。
    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), Some(String::new()));
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));
    assert_eq!(attr(&cursor_el, CURSOR_VARIANT_ATTR), Some("ring".into()));
    assert_eq!(attr(&cursor_el, CURSOR_LABEL_ATTR), Some("View".into()));
}

#[wasm_bindgen_test]
fn pointermove_leaving_target_clears_hover_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-3");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));

    // opt-in 対象を持たない root 自身へ移動する（対象の外）。
    dispatch_pointer_event(&root, "pointermove", "mouse", 999, 999);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("idle".into()));
    assert_eq!(attr(&cursor_el, CURSOR_VARIANT_ATTR), None);
    assert_eq!(attr(&cursor_el, CURSOR_LABEL_ATTR), None);
}

#[wasm_bindgen_test]
fn pointermove_into_non_target_area_transitions_hidden_to_idle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, _target) = build_dom(&document, "cursor-root-3a");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));

    // opt-in 対象を持たない root 自身への初回移動（`hover_target` は
    // `None` のまま変化しないため、`is_same` の判定だけでは
    // `"hidden"` → `"idle"` へ遷移しない不具合の回帰テスト
    // （イシュー #2542 レビュー指摘）。
    dispatch_pointer_event(&root, "pointermove", "mouse", 5, 5);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("idle".into()));
}

#[wasm_bindgen_test]
fn pointerout_true_leave_hides_cursor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-4");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));

    // `relatedTarget` を指定しない `pointerout`（root 外への真の離脱）。
    dispatch_pointer_event(&target, "pointerout", "mouse", 999, 999);

    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hidden".into()));
}

/// codex-review 指摘の回帰（PR #2583、discussion_r4020369847）: 真の離脱で
/// カーソルを `hidden` にする際、`root` の [`CURSOR_ACTIVE_ATTR`] が残留
/// すると `cursor: none !important` が効いたままネイティブカーソルも
/// 表示されない。離脱で外れ、再入場（有効な座標を得た pointermove）で
/// 再び付与されることを検証する。
#[wasm_bindgen_test]
fn pointerout_true_leave_clears_root_active_attr_and_reentry_restores_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, _cursor_el, target) = build_dom(&document, "cursor-root-13");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), Some(String::new()));

    dispatch_pointer_event(&target, "pointerout", "mouse", 999, 999);
    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        None,
        "真の離脱では root の CURSOR_ACTIVE_ATTR も外れ、ネイティブカーソルが\
         復元されるべき"
    );

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        Some(String::new()),
        "再入場で有効な座標を得たら CURSOR_ACTIVE_ATTR を再付与するべき"
    );
}

/// Cursor Bugbot 指摘の回帰（PR #2583 レビュー）: 構造再描画でホバー中の
/// 対象ノードが除去されると、ブラウザは `relatedTarget` を確定できない
/// まま `pointerout` を発火させることがある。座標が `root` の矩形内に
/// 留まっている限り、これを「真の離脱」と誤判定して `last_pointer` を
/// クリアしてはならない（誤判定すると再描画後の位置復元が「位置不明」側
/// へ倒れ、次の `pointermove` まで両カーソルとも消えたままになる）。
#[wasm_bindgen_test]
fn pointerout_without_related_target_but_within_root_bounds_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-12");
    let _guard = RemoveOnDrop(root.clone());
    let html_root = root
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("root must cast to HtmlElement for style access");
    let style = html_root.style();
    style.set_property("position", "absolute").unwrap();
    style.set_property("left", "0px").unwrap();
    style.set_property("top", "0px").unwrap();
    style.set_property("width", "100px").unwrap();
    style.set_property("height", "100px").unwrap();

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");
    dispatch_pointer_event(&target, "pointermove", "mouse", 20, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));

    // relatedTarget 未指定（`PointerEventInit` が既定で `None`）の
    // pointerout。座標 (20, 20) は上記で広げた root の矩形内に留まる。
    dispatch_pointer_event(&target, "pointerout", "mouse", 20, 20);

    assert_eq!(
        attr(&cursor_el, CURSOR_STATE_ATTR),
        Some("hover".into()),
        "relatedTarget 不明でも座標が root 内に留まっていれば離脱扱い\
         しないはず"
    );
}

#[wasm_bindgen_test]
fn touch_pointer_move_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-5");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "touch", 30, 20);

    assert_eq!(
        attr(&cursor_el, CURSOR_STATE_ATTR),
        Some("hidden".into()),
        "cursor はタッチ由来のポインタを除外するはず"
    );
}

#[wasm_bindgen_test]
fn magnetic_target_snaps_position_to_rect_center() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-6");
    let _guard = RemoveOnDrop(root.clone());
    target
        .set_attribute(CURSOR_TARGET_MAGNETIC_ATTR, "")
        .unwrap();

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");

    // 対象は (0, 0)-(40, 40) のため中心は (20, 20)。ポインタ座標
    // (30, 20) を渡しても中心へ吸着し、ポインタ座標をそのまま
    // 使わないことを、rAF ループ開始直後（初回書き込み前）の状態で
    // 検証する代わりに、少なくとも `move_to` 呼び出し自体が例外を
    // 起こさないことを確認する（収束値の検証は frontend-animation 側の
    // `cursor_browser.rs` が担う、モジュール doc参照）。
    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), Some("hover".into()));
}

#[wasm_bindgen_test]
fn reduced_motion_true_suppresses_wiring_entirely() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-7");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), true, false).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        None,
        "reduced_motion=true では配線自体が行われず root へ CURSOR_ACTIVE_ATTR も付かないはず"
    );
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), None);
}

#[wasm_bindgen_test]
fn coarse_pointer_true_suppresses_wiring_entirely() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-8");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, true).expect("wire_cursor_with_env must not fail");

    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);

    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), None);
    assert_eq!(attr(&cursor_el, CURSOR_STATE_ATTR), None);
}

/// `setTimeout(ms)` を 1 回だけ発行し解決を待つ、`MutationObserver`
/// コールバック（マイクロタスク）の発火を跨いで状態変化を確認するための
/// 猶予待機ヘルパー（`sidebar_browser.rs::sleep_ms` と同型）。
async fn sleep_ms(ms: i32) {
    let promise = Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                ms,
            )
            .expect("setTimeout must not fail");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");
}

/// イシュー #2583 レビュー指摘の回帰: `Runtime::apply_subtree_swap`
/// （`root` 自身は差し替えず子だけを丸ごと入れ替える構造フォールバック
/// 再描画）を模して、配線済みの `cursor_el`/`target` を `root` から
/// 取り除き新しい要素へ差し替える。`MutationObserver` 経由で
/// `CursorState` が再解決され、新しいカーソル要素へ `pointermove` の
/// 効果（`hover` 状態・`data-*` 写し）が届くこと、旧カーソル要素は
/// 更新されないままであることを確認する。
#[wasm_bindgen_test]
async fn cursor_element_is_resynced_after_structural_rerender() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, old_cursor_el, old_target) = build_dom(&document, "cursor-root-10");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");
    assert_eq!(
        attr(&old_cursor_el, CURSOR_STATE_ATTR),
        Some("hidden".into())
    );

    // 再描画前にポインタ位置を確定させる（`last_pointer` を `Some` に
    // する）。これが無いと再描画後も位置未知のままで
    // `CURSOR_ACTIVE_ATTR` が付かないのは正しい挙動（本テストが検証
    // したいのは「位置既知のまま再描画された場合に維持される」こと）。
    dispatch_pointer_event(&old_target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), Some(String::new()));

    // 構造フォールバック再描画を模す: 旧要素を除去し、新しい
    // cursor_el/target を同じ root 直下へ追加する。
    root.remove_child(&old_cursor_el).unwrap();
    root.remove_child(&old_target).unwrap();
    let (_dummy_root, new_cursor_el, new_target) = build_dom(&document, "cursor-root-10-donor");
    let _donor_guard = RemoveOnDrop(_dummy_root.clone());
    _dummy_root.remove_child(&new_cursor_el).unwrap();
    _dummy_root.remove_child(&new_target).unwrap();
    root.append_child(&new_cursor_el).unwrap();
    root.append_child(&new_target).unwrap();

    // `MutationObserver` コールバックはマイクロタスクで発火するため、
    // 猶予を与えてから再同期後の状態を確認する。
    sleep_ms(0).await;

    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        Some(String::new()),
        "再描画後も root の CURSOR_ACTIVE_ATTR は残ったまま（ネイティブ\
         カーソルは引き続き隠れる）はず"
    );
    assert_eq!(
        attr(&new_cursor_el, CURSOR_STATE_ATTR),
        Some("idle".into()),
        "再描画時点でポインタ位置が既知だった場合、新しいカーソル要素は\
         idle（可視）として再初期化されるはず"
    );

    dispatch_pointer_event(&new_target, "pointermove", "mouse", 30, 20);

    assert_eq!(
        attr(&new_cursor_el, CURSOR_STATE_ATTR),
        Some("hover".into()),
        "新しいカーソル要素が pointermove の効果を受け取れるはず"
    );
    assert_eq!(
        attr(&new_cursor_el, CURSOR_VARIANT_ATTR),
        Some("ring".into())
    );
    assert_eq!(
        attr(&old_cursor_el, CURSOR_STATE_ATTR),
        Some("hover".into()),
        "DOM から切り離された旧カーソル要素は除去直前の状態のまま更新され\
         続けないはず"
    );
}

/// 再描画でカーソル要素自体が消えた場合、[`CURSOR_ACTIVE_ATTR`] が
/// 外れてネイティブカーソルが復元されることの回帰
/// （イシュー #2583 レビュー指摘）。
#[wasm_bindgen_test]
async fn cursor_active_attr_is_removed_when_cursor_element_disappears() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, cursor_el, target) = build_dom(&document, "cursor-root-11");
    let _guard = RemoveOnDrop(root.clone());

    wire_cursor_with_env(root.clone(), false, false).expect("wire_cursor_with_env must not fail");
    dispatch_pointer_event(&target, "pointermove", "mouse", 30, 20);
    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), Some(String::new()));

    root.remove_child(&cursor_el).unwrap();
    sleep_ms(0).await;

    assert_eq!(
        attr(&root, CURSOR_ACTIVE_ATTR),
        None,
        "カーソル要素が消えた場合はネイティブカーソルを復元するはず"
    );
}

#[wasm_bindgen_test]
fn missing_cursor_element_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.create_element("div").unwrap();
    root.set_id("cursor-root-9");
    document.body().unwrap().append_child(&root).unwrap();
    let _guard = RemoveOnDrop(root.clone());

    // カーソル要素（[`CURSOR_SELECTOR`]）を持たない root。
    wire_cursor_with_env(root.clone(), false, false)
        .expect("wire_cursor_with_env must not fail even without a cursor element");

    assert_eq!(attr(&root, CURSOR_ACTIVE_ATTR), None);
}
