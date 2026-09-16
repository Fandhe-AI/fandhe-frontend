//! `fandhe_frontend_wasm_full::carousel_motion`（イシュー #2541）の実
//! ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `magnetic_browser.rs` と同方針: `wire_carousel_motion_events` を手組み
//! DOM へ直接配線し（`Runtime::mount` は経由しない）、opt-in root/
//! `item-group`/`item` の解決・pointer capture・`--fandhe-carousel-index`
//! への書き込み・settle 後の `"goto"` dispatch・5px 未満の移動での
//! no-op・opt-in 属性なしの carousel への無影響を検証する。
//! `fandhe_frontend_animation::carousel::snap_target` 自体の計算ロジックは
//! 責務境界どおり native テスト（`crates/frontend-animation/src/
//! carousel.rs`）で検証済み。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "carousel-motion")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_wasm_full::carousel_motion::{
    wire_carousel_motion_events, CAROUSEL_DRAGGING_STATE_ATTR, CAROUSEL_DRAG_ATTR,
    CAROUSEL_GOTO_ACTION_ATTR,
};
use fandhe_frontend_wasm_full::events::ActionRef;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, HtmlElement, MouseEvent, MouseEventInit, PointerEvent, PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

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

/// `root`（opt-in `drag_attr_value` 付き）> `item-group`（3 `item`、各
/// `width: 100px`）を組み立てて返す。`drag_attr_value` が `None` なら
/// opt-in 属性自体を付けない。`root` には実際の headless-ui 出力
/// （`crates/headless-ui/src/carousel.rs::root`）と同じ
/// `data-scope="carousel"`/`data-part="root"` を付与する（codex-review
/// 指摘 是正 P1「入れ子 carousel の root/item-group 誤結合」の回帰検知
/// （[`nested_carousel_...`] 系テスト）が `ANATOMY_ROOT_SELECTOR`
/// （`[data-scope="carousel"][data-part="root"]`）による carousel root
/// 解決へ依存するため、イシュー #2541 第 5 ラウンド）。
fn build_dom(document: &Document, drag_attr_value: Option<&str>) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_attribute("data-scope", "carousel").unwrap();
    root.set_attribute("data-part", "root").unwrap();
    if let Some(value) = drag_attr_value {
        root.set_attribute(CAROUSEL_DRAG_ATTR, value).unwrap();
    }

    let item_group = document.create_element("div").unwrap();
    item_group.set_attribute("data-scope", "carousel").unwrap();
    item_group.set_attribute("data-part", "item-group").unwrap();

    for _ in 0..3 {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "carousel").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        let html_item = item.clone().dyn_into::<HtmlElement>().unwrap();
        let style = html_item.style();
        style.set_property("display", "inline-block").unwrap();
        style.set_property("width", "100px").unwrap();
        style.set_property("height", "40px").unwrap();
        item_group.append_child(&item).unwrap();
    }
    root.append_child(&item_group).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    (root, item_group)
}

/// 既定で主ボタン（`button() == 0`）・最初の接触点（`is_primary() ==
/// true`）の pointer イベントを組み立てて配送する。`PointerEventInit` は
/// これらの既定値を実ブラウザのマウス操作と異なり `0`/`false` のまま
/// 残す（`is_primary` 未指定は `false`）ため、明示しないと
/// `handle_pointerdown` の主ボタン・最初の接触点ガード（PR #2581 レビュー
/// 是正「右クリック・中クリックでドラッグを開始しない」節参照）に
/// 弾かれてしまう（`drag_gesture_browser.rs` の同名ヘルパと同型の対応）。
fn dispatch_pointer_event(target: &Element, kind: &str, client_x: i32, pointer_id: i32) {
    let init = PointerEventInit::new();
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x);
    init.set_button(0);
    init.set_is_primary(true);
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = PointerEvent::new_with_event_init_dict(kind, &init).unwrap();
    target.dispatch_event(&event).unwrap();
}

fn read_index(element: &Element) -> Option<f64> {
    element
        .dyn_ref::<HtmlElement>()
        .and_then(|el| {
            el.style()
                .get_property_value("--fandhe-carousel-index")
                .ok()
        })
        .and_then(|v| v.trim().parse::<f64>().ok())
}

#[wasm_bindgen_test]
async fn drag_release_dispatches_goto_after_settle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be set after pointerdown"
    );
    // 150px 左へドラッグ（1 スライド = 100px の 1.5 スライド分）。
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    let dragging_value = read_index(&item_group).expect("progress must be written while dragging");
    assert!(
        (dragging_value - 1.5).abs() < 1e-6,
        "dragging value should be 1.5: {dragging_value}"
    );

    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    sleep_ms(2_000).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        (final_value - 2.0).abs() < 0.01,
        "settled value should converge to 2.0: {final_value}"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be removed after settle"
    );
    let actions = dispatched.borrow();
    assert_eq!(actions.len(), 1, "goto should dispatch exactly once");
    assert_eq!(actions[0].action, "goto");
    assert_eq!(actions[0].payload, "2");
}

#[wasm_bindgen_test]
async fn small_move_settles_back_to_origin_index() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    // 2px の移動は snap_target を四捨五入すると起点の index 0 のまま。
    dispatch_pointer_event(&item_group, "pointermove", 2, 1);
    dispatch_pointer_event(&item_group, "pointerup", 2, 1);
    sleep_ms(500).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        final_value.abs() < 0.01,
        "tiny move should settle back to index 0: {final_value}"
    );
}

#[wasm_bindgen_test]
async fn carousel_without_opt_in_attribute_is_untouched() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, None);
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);
    sleep_ms(500).await;

    assert!(
        read_index(&item_group).is_none(),
        "no opt-in attribute should leave --fandhe-carousel-index unset"
    );
    assert!(dispatched.borrow().is_empty());
}

/// codex-review 指摘 是正（イシュー #2541 第 3 ラウンド、PR #2581 レビュー
/// で「Suppressed click still cancels spring」是正に伴い前提を修正）の
/// 回帰: `"goto"` dispatch は spring 収束を待たず release 時に同期実行
/// される。release 直後の（`suppress_click` で抑止される）合成 click を
/// 消費したあとで、著者が nav trigger を**改めて**操作すると進行中の
/// spring は打ち切られ（見た目の上書きが止まる）、
/// `data-fandhe-carousel-dragging` も除去されるが、release 時に確定済みの
/// dispatch 自体は取り消されない（1 回のみ）。
#[wasm_bindgen_test]
async fn nav_trigger_click_cancels_pending_settle_without_reverting_dispatch() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let next_trigger = document.create_element("button").unwrap();
    next_trigger
        .set_attribute("data-scope", "carousel")
        .unwrap();
    next_trigger
        .set_attribute("data-part", "next-trigger")
        .unwrap();
    root.append_child(&next_trigger).unwrap();

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    assert_eq!(
        dispatched.borrow().len(),
        1,
        "goto should dispatch synchronously at release"
    );
    assert_eq!(dispatched.borrow()[0].payload, "2");

    let click_init = MouseEventInit::new();
    click_init.set_bubbles(true);
    click_init.set_cancelable(true);

    // 実ブラウザが release 直後に発火する合成 click を模して先に消費する
    // （`suppress_click` により抑止される最初の click。Cursor Bugbot 指摘
    // 是正「Suppressed click still cancels spring」により、これ自体は
    // 進行中の spring を打ち切らない）。
    let synthetic_release_click =
        MouseEvent::new_with_mouse_event_init_dict("click", &click_init).unwrap();
    item_group.dispatch_event(&synthetic_release_click).unwrap();

    // 消費済みのため、著者が**改めて** next-trigger を操作した genuine な
    // click のみが以降の判定に届く。
    let genuine_click = MouseEvent::new_with_mouse_event_init_dict("click", &click_init).unwrap();
    next_trigger.dispatch_event(&genuine_click).unwrap();

    let frozen_value = read_index(&item_group).expect("progress must remain written");
    // 打ち切り直後、まだ 1 tick も走っていない spring は途中の小数
    // progress（この操作では 2.0 未満）を書いたままのはずだが、
    // `CarouselTrack::cancel_and_snap` が確定 index（dispatch した
    // `"2"`）へ即座に書き戻す（codex-review 指摘 是正、イシュー #2541
    // 第 6 ラウンド「同一 index への no-op 更新だと途中の小数 index が
    // 復元されない」）。呼び出し側の action ハンドラがこの genuine click
    // に対して何も再描画しない（no-op）構成でも、この書き込みだけで
    // 表示は確定 index と一致する。
    assert!(
        (frozen_value - 2.0).abs() < 1e-9,
        "cancelled settle must snap immediately to the confirmed target index: {frozen_value}"
    );
    sleep_ms(300).await;
    let after_click_value = read_index(&item_group).expect("progress must remain written");
    assert!(
        (frozen_value - after_click_value).abs() < 1e-9,
        "spring should stop writing --fandhe-carousel-index once cancelled: \
         {frozen_value} vs {after_click_value}"
    );
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "cancelled settle must not dispatch a stale goto later"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared when settle is cancelled"
    );
}

/// codex-review 指摘 是正（イシュー #2541 第 3 ラウンド）の回帰: pointer
/// capture が暗黙に失われ `pointerup`/`pointercancel` が一切届かなくても
/// （root 外での release 等）、`lostpointercapture` だけでドラッグ終了処理
/// （settle への goto dispatch・`DragMeta` の回収）が行われ、次の
/// `pointerdown` が「進行中のドラッグがある」判定で拒否され続けない。
#[wasm_bindgen_test]
async fn lostpointercapture_recovers_drag_state_when_pointerup_is_missed() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    // pointerup は一切発火せず、capture 喪失のみを模した
    // `lostpointercapture` を直接発火する（root 外での release や OS 都合
    // による暗黙の capture 喪失を模す）。
    dispatch_pointer_event(&item_group, "lostpointercapture", -150, 1);

    assert_eq!(
        dispatched.borrow().len(),
        1,
        "lostpointercapture alone must still finalize the drag with a goto dispatch"
    );
    assert_eq!(dispatched.borrow()[0].payload, "2");

    dispatch_pointer_event(&item_group, "pointerdown", 0, 2);
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "a fresh pointerdown must be accepted after lostpointercapture recovery"
    );
}

/// codex-review 指摘 是正（イシュー #2541 第 3 ラウンド）の回帰:
/// [`CAROUSEL_GOTO_ACTION_ATTR`] を carousel root へ指定すると、ドラッグ
/// 確定時の dispatch はその名前を使う（複数 carousel を同一 Runtime 配下に
/// 置いても `decode_action` 側でどの carousel の操作か判別できる）。
#[wasm_bindgen_test]
async fn goto_action_attr_overrides_dispatched_action_name() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());
    root.set_attribute(CAROUSEL_GOTO_ACTION_ATTR, "carousel-hero:goto")
        .unwrap();

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    let actions = dispatched.borrow();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action, "carousel-hero:goto");
    assert_eq!(actions[0].payload, "2");
}

/// codex-review/Cursor Bugbot 指摘 是正（PR #2581 レビュー）の回帰:
/// `on_action` の同期 dispatch が（`Runtime::apply_subtree_swap` を
/// 模して）release 直後に `item-group` を丸ごと新規ノードへ差し替えても、
/// 進行中の spring は新しい（表示中の）`item-group` へ retarget され、
/// 不可視になった旧ノードを更新し続けたまま止まって見えることはない。
#[wasm_bindgen_test]
async fn redraw_during_dispatch_retargets_spring_to_new_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let new_item_group_cell: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let redraw_document = document.clone();
    let redraw_root = root.clone();
    let redraw_cell = new_item_group_cell.clone();
    wire_carousel_motion_events(root.clone(), move |_action_ref: ActionRef| {
        // `Runtime::apply_subtree_swap` を模す: 子ノードを丸ごと新規
        // `item-group`（同じ 3 `item` 構成）へ差し替える。旧 `item_group`
        // はこの時点で文書から切断される。
        while let Some(child) = redraw_root.first_child() {
            let _ = redraw_root.remove_child(&child);
        }
        let (_replacement_root, new_item_group) = build_dom(&redraw_document, None);
        let new_item_group_detached = new_item_group.clone();
        new_item_group_detached.remove();
        let _ = redraw_root.append_child(&new_item_group_detached);
        *redraw_cell.borrow_mut() = Some(new_item_group_detached);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    let new_item_group = new_item_group_cell
        .borrow()
        .clone()
        .expect("dispatch must have installed a replacement item-group synchronously");
    assert!(
        !item_group.is_connected(),
        "old item-group must be detached by the simulated redraw"
    );

    sleep_ms(4_000).await;

    let new_value =
        read_index(&new_item_group).expect("spring must retarget onto the new item-group");
    assert!(
        (new_value - 2.0).abs() < 0.01,
        "spring must continue converging on the new (displayed) DOM: {new_value}"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared once the retargeted settle completes"
    );
}

/// codex-review 指摘 是正の回帰（イシュー #2541 第 4 ラウンド）:
/// carousel root 自身が `Runtime` の mount root と**異なる**（`root` 配下に
/// nest された）構成で redraw が起きると、`TrackRegistry` は旧
/// carousel root をキーに持ち続けてしまう。是正前は、redraw 後に**別の**
/// carousel への pointerdown が [`slot_for`] の遅延掃除
/// （`retain(|(root,_,_)| root.is_connected())`）を誘発した時点で、旧
/// （切断済み）キーのエントリが間引かれ、進行中の spring を保持する
/// `TrackSlot` ごと drop されて `AnimationLoop` が中断していた
/// （`redraw_during_dispatch_retargets_spring_to_new_dom` は carousel
/// root == mount root の特別扱い経路〔`resolve_replacement` 参照〕を通る
/// ため、この回帰を検知できない）。
#[wasm_bindgen_test]
async fn redraw_of_nested_carousel_root_survives_other_carousel_pointerdown() {
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&wrapper).unwrap();
    let _wrapper_guard = RemoveOnDrop(wrapper.clone());

    // `build_dom` は既定で `<body>` 直下へ追加するため、mount root
    // （`wrapper`）と carousel root（`root`）が異なる要素になるよう
    // 明示的に `wrapper` 配下へ付け替える。
    let (root, item_group) = build_dom(&document, Some(""));
    wrapper.append_child(&root).unwrap();

    let new_root_cell: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let redraw_document = document.clone();
    let redraw_wrapper = wrapper.clone();
    let redraw_cell = new_root_cell.clone();
    wire_carousel_motion_events(wrapper.clone(), move |_action_ref: ActionRef| {
        // `Runtime::apply_subtree_swap` を模す: `wrapper` 配下の carousel
        // root ごと新規要素（同じ opt-in 属性・item 構成）へ差し替える。
        // 旧 `root` はこの時点で文書から切断される。
        while let Some(child) = redraw_wrapper.first_child() {
            let _ = redraw_wrapper.remove_child(&child);
        }
        let (new_root, _new_item_group) = build_dom(&redraw_document, Some(""));
        new_root.remove();
        let _ = redraw_wrapper.append_child(&new_root);
        *redraw_cell.borrow_mut() = Some(new_root);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    let new_root = new_root_cell
        .borrow()
        .clone()
        .expect("dispatch must have installed a replacement carousel root synchronously");
    assert!(
        !root.is_connected(),
        "old carousel root must be detached by the simulated redraw"
    );

    // 別の（無関係な）carousel への pointerdown で `slot_for` の遅延掃除を
    // 誘発する。retarget と同時にレジストリのキーが新 root へ移されて
    // いなければ、ここで進行中の spring を保持する `TrackSlot` が間引かれ
    // `AnimationLoop` が中断する。タップのみ（move なし）で `"goto"` は
    // dispatch しないため、`wrapper` を再度全消去する競合は起きない。
    let (other_root, other_item_group) = build_dom(&document, Some(""));
    wrapper.append_child(&other_root).unwrap();
    let _other_guard = RemoveOnDrop(other_root.clone());
    dispatch_pointer_event(&other_item_group, "pointerdown", 0, 2);
    dispatch_pointer_event(&other_item_group, "pointerup", 0, 2);

    sleep_ms(4_000).await;

    let new_item_group = new_root
        .query_selector("[data-scope=\"carousel\"][data-part=\"item-group\"]")
        .unwrap()
        .expect("replacement item-group must exist under the replacement carousel root");
    let new_value = read_index(&new_item_group)
        .expect("spring must keep writing to the replacement item-group after registry cleanup");
    assert!(
        (new_value - 2.0).abs() < 0.01,
        "spring must keep converging on the replacement carousel root even after another \
         carousel's pointerdown triggers registry cleanup: {new_value}"
    );
    assert!(
        !new_root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared on the replacement root once the \
         retargeted settle completes"
    );
}

/// codex-review 指摘 是正（PR #2581 レビュー）の回帰: 主ボタン
/// （`button() == 0`）以外の pointerdown（右クリック・中クリック等）は
/// ドラッグを開始しない。
#[wasm_bindgen_test]
async fn non_primary_button_does_not_start_drag() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    wire_carousel_motion_events(root.clone(), move |_action_ref: ActionRef| {
        panic!("non-primary-button pointerdown must not lead to a dispatch");
    })
    .unwrap();

    let init = PointerEventInit::new();
    init.set_pointer_id(1);
    init.set_client_x(0);
    init.set_button(2); // 右クリック相当。
                        // 右クリックも通常は最初の接触点であるため `is_primary` は明示的に
                        // 真にしておく（本テストが `button` チェックのみを検証することを
                        // 確実にする。既定値の `false` に依存すると `is_primary` チェックと
                        // 混同しかねない）。
    init.set_is_primary(true);
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = PointerEvent::new_with_event_init_dict("pointerdown", &init).unwrap();
    item_group.dispatch_event(&event).unwrap();

    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "right-click pointerdown must not start a drag"
    );

    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    assert!(
        read_index(&item_group).is_none(),
        "no drag should have been started, so no progress should be written"
    );
}

/// Cursor Bugbot 指摘 是正（PR #2581 レビュー、「Suppressed click still
/// cancels spring」）の回帰: ドラッグ確定後の release が偶然 nav trigger の
/// 上で起きても、それに続く合成 `click`（`suppress_click` で抑止される）は
/// 進行中の spring を打ち切ってはならない（抑止対象の click は nav
/// trigger の意図的な操作ではないため）。
#[wasm_bindgen_test]
async fn suppressed_click_over_nav_trigger_does_not_cancel_settle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let next_trigger = document.create_element("button").unwrap();
    next_trigger
        .set_attribute("data-scope", "carousel")
        .unwrap();
    next_trigger
        .set_attribute("data-part", "next-trigger")
        .unwrap();
    root.append_child(&next_trigger).unwrap();

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    // 5px 超の移動で `moved = true` となり、release 時に `suppress_click`
    // が立つ（実ドラッグとして確定）。
    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "goto should dispatch at release"
    );
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "settle is still converging right after release"
    );

    // ブラウザが release 直後に発火する合成 click を模す（たまたま nav
    // trigger の上で release されたケース）。`suppress_click` により抑止
    // される最初の click のため、`invalidate_settle_on_nav_trigger_click`
    // は呼ばれてはならない。
    let click_init = MouseEventInit::new();
    click_init.set_bubbles(true);
    click_init.set_cancelable(true);
    let click_event = MouseEvent::new_with_mouse_event_init_dict("click", &click_init).unwrap();
    next_trigger.dispatch_event(&click_event).unwrap();

    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "the suppressed synthetic click must not cancel the pending settle"
    );

    sleep_ms(4_000).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        (final_value - 2.0).abs() < 0.01,
        "settle must still converge normally after the suppressed click: {final_value}"
    );
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "the suppressed click must not trigger any additional dispatch"
    );
    assert!(!root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR));
}

/// Cursor Bugbot 指摘 是正（PR #2581 レビュー、`drag_gesture.rs::
/// wire_drag_gesture` の「`pointerup`/`pointercancel` は `window` にも
/// 登録する」節と同型）の回帰: pointer capture が確定する前
/// （[`CLICK_GUARD_PX`] 未満の移動しかしていない）に carousel の DOM
/// 部分木の外で `pointerup` が発生しても、`window` への委譲登録により
/// ドラッグ終了処理が確実に行われ、`DragMeta` が残留して以降の
/// `pointerdown` を拒否し続けることはない。
#[wasm_bindgen_test]
async fn pointerup_outside_root_before_capture_is_recovered_via_window() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    // 移動量は CLICK_GUARD_PX (5px) 未満のため pointer capture は確定
    // しない。
    dispatch_pointer_event(&item_group, "pointermove", 2, 1);

    // `root`/`item_group` の外（`<body>` 自身）で release する。`root` への
    // 委譲登録だけでは `root` が祖先に含まれないため取りこぼす。
    let body: Element = document.body().unwrap().dyn_into().unwrap();
    dispatch_pointer_event(&body, "pointerup", 2, 1);

    // `CLICK_GUARD_PX` 未満の移動しかしていないタップは `"goto"` を
    // dispatch しない（イシュー #2541 codex-review 指摘 是正「ドラッグ
    // していないタップでは goto を dispatch しない」）。window 経由の
    // 回収自体は起きている——後続の `!root.has_attribute(...)` アサーション
    // が dragging 状態属性の除去（`track` 側の後始末）で「確実に処理
    // された」ことを検証する。
    assert!(
        dispatched.borrow().is_empty(),
        "a tap-level move below CLICK_GUARD_PX must not dispatch goto"
    );

    // 2px の微小な移動は着地 index も起点と同じになり得るため、settle
    // （`--fandhe-carousel-index` の spring 収束）完了まで少し待ってから
    // dragging 状態属性の除去を確認する（`small_move_settles_back_to_
    // origin_index` と同型）。
    sleep_ms(1_000).await;
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared once recovered"
    );

    dispatch_pointer_event(&item_group, "pointerdown", 0, 2);
    assert!(
        root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "a fresh pointerdown must be accepted after the recovered release"
    );
}

/// codex-review 指摘 是正の回帰（イシュー #2541 第 4 ラウンド）: settle
/// アニメーション（収束中の spring）を移動なしのタップ（`pointerdown` →
/// 即 `pointerup`）で中断しても、確定済みの着地 index（release 時に既に
/// `"goto"` dispatch 済み）とは異なる index へ視覚的に収束してはならない。
/// 是正前は `pointerdown` のたびに `CarouselTrack::attach` で新規
/// インスタンスを生成しており、中断時点の途中経過進行度から最寄り index
/// を再計算してしまっていた（`last_target` が失われるため）。
#[wasm_bindgen_test]
async fn tap_during_settle_resumes_to_committed_target() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some("loop"));
    let _guard = RemoveOnDrop(root.clone());

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    // 右へ 60px ドラッグ（進行度 -0.6、3 スライド loop で末尾 index 2 へ
    // 折り返す）。release まで 150ms 待って速度を陳腐化させ
    // （`crate::drag::STALE_VELOCITY_THRESHOLD_MS` = 100ms）、着地 index が
    // 進行度の丸めのみで決定的に 2 になるようにする（本テストの再現条件を
    // 速度の推定タイミング揺れから独立させるため）。
    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", 60, 1);
    sleep_ms(150).await;
    dispatch_pointer_event(&item_group, "pointerup", 60, 1);
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "drag release must dispatch goto exactly once"
    );
    assert_eq!(dispatched.borrow()[0].payload, "2");

    // settle（spring 収束）がまだ完了していないタイミングで、移動なしの
    // タップ（別ポインタ）を割り込ませる。
    sleep_ms(100).await;
    dispatch_pointer_event(&item_group, "pointerdown", 60, 2);
    dispatch_pointer_event(&item_group, "pointerup", 60, 2);

    // タップは "goto" を再 dispatch しない。
    assert_eq!(
        dispatched.borrow().len(),
        1,
        "a non-moving tap must not dispatch goto again"
    );

    sleep_ms(2_000).await;

    let final_value = read_index(&item_group).expect("progress must remain written after settle");
    assert!(
        (final_value - 2.0).abs() < 0.01,
        "tap during settle must resume converging on the already-committed index 2, not a value \
         derived from the interrupted mid-flight progress: {final_value}"
    );
    assert!(
        !root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "dragging state attribute should be cleared once the resumed settle completes"
    );
}

/// codex-review 指摘 是正 P1 の回帰（イシュー #2541 第 5 ラウンド）:
/// opt-in（drag）していない carousel が、opt-in している別の carousel の
/// `item` 内へ入れ子で存在する構成で、内側 carousel 上をドラッグしても
/// 外側の opt-in・状態（`TrackSlot`/`slide_count`）を借用してはならない。
/// `handle_pointerdown` の `carousel_root`（[`CAROUSEL_ROOT_SELECTOR`]、
/// opt-in 属性を持つ最も近い祖先）と `item_group_el` の anatomy 上の
/// carousel root（`ANATOMY_ROOT_SELECTOR`、opt-in の有無を問わない）が
/// 一致することを検証してから操作を開始する是正の直接確認。
#[wasm_bindgen_test]
async fn nested_non_opt_in_carousel_does_not_borrow_outer_drag_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer_root, outer_item_group) = build_dom(&document, Some(""));
    let _outer_guard = RemoveOnDrop(outer_root.clone());

    // 内側 carousel（opt-in なし）を外側の item-group 内へ入れ子で追加する。
    let (inner_root, inner_item_group) = build_dom(&document, None);
    inner_root.remove();
    outer_item_group.append_child(&inner_root).unwrap();

    let dispatched: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched_for_cb = dispatched.clone();
    wire_carousel_motion_events(outer_root.clone(), move |action_ref: ActionRef| {
        dispatched_for_cb.borrow_mut().push(action_ref);
    })
    .unwrap();

    // 内側 carousel の item 上でドラッグ操作を行う。
    dispatch_pointer_event(&inner_item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&inner_item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&inner_item_group, "pointerup", -150, 1);

    assert!(
        !outer_root.has_attribute(CAROUSEL_DRAGGING_STATE_ATTR),
        "内側 carousel への操作が外側の opt-in root を巻き込んではならない"
    );
    assert!(
        dispatched.borrow().is_empty(),
        "opt-in していない内側 carousel の操作は goto を dispatch してはならない"
    );
    assert!(
        read_index(&inner_item_group).is_none(),
        "内側 carousel の item-group へ外側の spring が書き込んではならない"
    );
    assert!(
        read_index(&outer_item_group).is_none(),
        "外側の item-group も無関係な内側操作で書き換えられてはならない"
    );
}

/// [`build_dom`] と同型だが `item` の枚数を明示できる（既定は常に 3
/// 枚固定のため、枚数が異なる「別の carousel」を模すテスト専用）。
fn build_dom_with_item_count(
    document: &Document,
    drag_attr_value: Option<&str>,
    item_count: usize,
) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_attribute("data-scope", "carousel").unwrap();
    root.set_attribute("data-part", "root").unwrap();
    if let Some(value) = drag_attr_value {
        root.set_attribute(CAROUSEL_DRAG_ATTR, value).unwrap();
    }

    let item_group = document.create_element("div").unwrap();
    item_group.set_attribute("data-scope", "carousel").unwrap();
    item_group.set_attribute("data-part", "item-group").unwrap();

    for _ in 0..item_count {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "carousel").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        let html_item = item.clone().dyn_into::<HtmlElement>().unwrap();
        let style = html_item.style();
        style.set_property("display", "inline-block").unwrap();
        style.set_property("width", "100px").unwrap();
        style.set_property("height", "40px").unwrap();
        item_group.append_child(&item).unwrap();
    }
    root.append_child(&item_group).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    (root, item_group)
}

/// codex-review 指摘 是正の回帰（イシュー #2541 第 6 ラウンド）: 再描画後の
/// carousel を文書順の位置だけで同一視すると、位置は同じでも構造が違う
/// （＝別の）carousel を「同じ carousel の再描画」と誤認してしまう
/// （例: 一覧をフィルタで絞り込み、同じ位置に枚数の異なる別 carousel が
/// 現れる）。`resolve_replacement` は再描画先の `item` 枚数が pointerdown
/// 時点の枚数と食い違えば「別 carousel」と判定し、無関係な spring の
/// retarget を行わずに打ち切る（見つからなかった場合と同じフォール
/// バック）。
#[wasm_bindgen_test]
async fn redraw_with_mismatched_slide_count_does_not_retarget_spring() {
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&wrapper).unwrap();
    let _wrapper_guard = RemoveOnDrop(wrapper.clone());

    let (root, item_group) = build_dom(&document, Some(""));
    wrapper.append_child(&root).unwrap();

    let new_item_group_cell: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let redraw_document = document.clone();
    let redraw_wrapper = wrapper.clone();
    let redraw_cell = new_item_group_cell.clone();
    wire_carousel_motion_events(wrapper.clone(), move |_action_ref: ActionRef| {
        // 同じ位置へ、枚数の異なる（＝構造上別の）carousel を差し替える。
        while let Some(child) = redraw_wrapper.first_child() {
            let _ = redraw_wrapper.remove_child(&child);
        }
        let (new_root, new_item_group) = build_dom_with_item_count(&redraw_document, Some(""), 2);
        new_root.remove();
        let _ = redraw_wrapper.append_child(&new_root);
        *redraw_cell.borrow_mut() = Some(new_item_group);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    let new_item_group = new_item_group_cell
        .borrow()
        .clone()
        .expect("dispatch must have installed a replacement carousel synchronously");

    sleep_ms(4_000).await;

    assert!(
        read_index(&new_item_group).is_none(),
        "枚数の異なる別 carousel へ旧 spring を retarget してはならない"
    );
}

/// codex-review 指摘 是正の回帰（イシュー #2541 第 7 ラウンド）: 位置・
/// 枚数が同じでも `aria-label` が異なる（＝構造上別の）carousel へ
/// 旧 spring を retarget してはならない（`resolve_replacement` の
/// `aria-label` 突き合わせ、モジュール doc参照）。枚数一致チェックだけでは
/// 同じ枚数の別 carousel が入れ替わる構成を見分けられない。
#[wasm_bindgen_test]
async fn redraw_with_mismatched_aria_label_does_not_retarget_spring() {
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&wrapper).unwrap();
    let _wrapper_guard = RemoveOnDrop(wrapper.clone());

    let (root, item_group) = build_dom(&document, Some(""));
    root.set_attribute("aria-label", "Featured products")
        .unwrap();
    wrapper.append_child(&root).unwrap();

    let new_item_group_cell: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let redraw_document = document.clone();
    let redraw_wrapper = wrapper.clone();
    let redraw_cell = new_item_group_cell.clone();
    wire_carousel_motion_events(wrapper.clone(), move |_action_ref: ActionRef| {
        // 同じ位置・同じ枚数（3 枚）だが `aria-label` が異なる別 carousel
        // へ差し替える。
        while let Some(child) = redraw_wrapper.first_child() {
            let _ = redraw_wrapper.remove_child(&child);
        }
        let (new_root, new_item_group) = build_dom_with_item_count(&redraw_document, Some(""), 3);
        new_root
            .set_attribute("aria-label", "Recommended products")
            .unwrap();
        new_root.remove();
        let _ = redraw_wrapper.append_child(&new_root);
        *redraw_cell.borrow_mut() = Some(new_item_group);
    })
    .unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    let new_item_group = new_item_group_cell
        .borrow()
        .clone()
        .expect("dispatch must have installed a replacement carousel synchronously");

    sleep_ms(4_000).await;

    assert!(
        read_index(&new_item_group).is_none(),
        "aria-label の異なる別 carousel へ旧 spring を retarget してはならない"
    );
}

/// codex-review 指摘 是正の回帰（イシュー #2541 第 6 ラウンド）: 収束中の
/// spring が、この `CarouselTrack` を経由しない別経路（例: 自動再生の
/// `"goto"` が SSR 再描画で `item_group` のインライン style を直接書き
/// 換える）による外部書き込みを検知した場合、古い着地先で毎フレーム
/// 上書きし続けてはならない。
#[wasm_bindgen_test]
async fn external_index_write_during_settle_stops_stale_spring() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item_group) = build_dom(&document, Some(""));
    let _guard = RemoveOnDrop(root.clone());

    wire_carousel_motion_events(root.clone(), move |_action_ref: ActionRef| {}).unwrap();

    dispatch_pointer_event(&item_group, "pointerdown", 0, 1);
    dispatch_pointer_event(&item_group, "pointermove", -150, 1);
    dispatch_pointer_event(&item_group, "pointerup", -150, 1);

    // spring が数フレーム書き込むまで少し待つ（着地先 2.0 へ向けて収束中、
    // まだ完了していない）。
    sleep_ms(50).await;

    // この `CarouselTrack` を経由しない外部書き込みを模す。
    let html_item_group = item_group.clone().dyn_into::<HtmlElement>().unwrap();
    html_item_group
        .style()
        .set_property("--fandhe-carousel-index", "0.75")
        .unwrap();

    // 元の spring が生きていれば着地先 2.0 へ向けて上書きを続けるはず
    // だが、収束完了まで十分な時間（他テストの `sleep_ms(4_000)` 相当）
    // 待っても外部書き込みの値のまま変わらないことを確認する。
    sleep_ms(4_000).await;

    let value = read_index(&item_group).expect("value must remain set");
    assert!(
        (value - 0.75).abs() < 1e-6,
        "打ち切られた spring が外部書き込みを上書きし続けてはならない: {value}"
    );
}
