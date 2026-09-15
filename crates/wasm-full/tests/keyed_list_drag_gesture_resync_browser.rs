//! codex-review P1 是正（イシュー #2535、PR #2565）の実ブラウザ統合
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `stagger_index_browser.rs` と同じ理由（製品コンポーネント
//! `interactive::AppState` では検証したい keyed list 操作を直接誘発
//! できない）で、本ファイル専用の最小 component（[`ListState`]）を使い、
//! `Runtime::apply_update_for_dirty` の keyed list 構造反映（`Insert`/
//! `Update`）経由で
//!
//! 1. 新規挿入された `data-fandhe-drag` 要素へ `touch-action: none` が
//!    最初の `pointerdown` より前に先行反映されること（`Self::
//!    apply_subtree_swap` のみが再同期していた漏れの是正）
//! 2. 既存行の内容変更（`KeyedOp::Update`）で
//!    `fandhe_frontend_wasm_client::keyed_dom::sync_attrs` が `style`
//!    属性を丸ごと削除した後も、`touch-action: none` と移動済みの
//!    位置（CSS カスタムプロパティ）が復元されること（`controller_for`
//!    が既存コントローラを返すだけで DOM へ書き戻さなかった漏れの是正、
//!    PR #2565 第 2 ラウンド codex-review P1 指摘）
//!
//! を固定する（`resync_drag_gesture_attachments`/`DragController::
//! resync_dom` doc 参照）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "drag-gesture")]

use fandhe_frontend_animation::drag::{DRAG_X_PROPERTY, DRAG_Y_PROPERTY};
use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::drag_gesture::{DRAG_ATTR, DRAG_AXIS_ATTR};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement, PointerEvent, PointerEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// `stagger_index_browser.rs::create_placeholder` と同じ意図。
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

/// `stagger_index_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `stagger_index_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// 動的リスト 1 件のみを持つ最小 component。各行が opt-in ドラッグ要素
/// （[`DRAG_ATTR`]）であり、`items` のみが dirty field となる
/// （`stagger_index_browser.rs::ListState` と同型）。`axis` は
/// [`DRAG_AXIS_ATTR`] へ反映する軸制約（`None` は省略＝自由）。
#[derive(Debug, Clone)]
struct ListState {
    /// `(安定キー, 表示内容, 軸制約)` の順序付きリスト。
    items: Vec<(u64, String, Option<&'static str>)>,
    dirty: Vec<&'static str>,
}

impl ListState {
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str)]) -> Self {
        Self {
            items: initial
                .iter()
                .map(|(id, text)| (*id, text.to_string(), None))
                .collect(),
            dirty: Vec::new(),
        }
    }
}

/// `ListState::decode_action` が復号する型付きアクション。
enum ListAction {
    /// 末尾へ `(id, content)` を 1 件追加する（`Insert` を誘発）。
    Append { id: u64, content: String },
    /// 既存キーの表示内容を書き換える（`KeyedOp::Update` を誘発。
    /// キーは不変のため対象要素・`DragController` は同一のまま）。
    Rename { id: u64, content: String },
    /// 既存キーの [`DRAG_AXIS_ATTR`] を書き換える（`KeyedOp::Update` を
    /// 誘発。codex-review P1 是正の回帰、PR #2565 第 3 ラウンド）。
    SetAxis { id: u64, axis: &'static str },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::Append { id, content } => {
                self.items.push((id, content, None));
                self.dirty.push(Self::FIELD_ITEMS);
            }
            ListAction::Rename { id, content } => {
                if let Some(entry) = self
                    .items
                    .iter_mut()
                    .find(|(existing, _, _)| *existing == id)
                {
                    entry.1 = content;
                    self.dirty.push(Self::FIELD_ITEMS);
                }
            }
            ListAction::SetAxis { id, axis } => {
                if let Some(entry) = self
                    .items
                    .iter_mut()
                    .find(|(existing, _, _)| *existing == id)
                {
                    entry.2 = Some(axis);
                    self.dirty.push(Self::FIELD_ITEMS);
                }
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|(id, content, axis)| {
                let mut attrs = vec![("data-testid", "drag-item"), (DRAG_ATTR, "")];
                if let Some(axis) = axis {
                    attrs.push((DRAG_AXIS_ATTR, axis));
                }
                (id.to_string(), el("li", attrs, vec![text(content)]))
            })
            .collect();
        let list = keyed_list("ul", vec![("id", "drag-list")], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "drag-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            // payload 形式: "<id>:<content>"。
            "append" => {
                let (id_str, content) = payload.split_once(':')?;
                Some(ListAction::Append {
                    id: id_str.parse::<u64>().ok()?,
                    content: content.to_string(),
                })
            }
            "rename" => {
                let (id_str, content) = payload.split_once(':')?;
                Some(ListAction::Rename {
                    id: id_str.parse::<u64>().ok()?,
                    content: content.to_string(),
                })
            }
            "set-axis" => {
                let (id_str, axis) = payload.split_once(':')?;
                let axis = match axis {
                    "x" => "x",
                    "y" => "y",
                    _ => return None,
                };
                Some(ListAction::SetAxis {
                    id: id_str.parse::<u64>().ok()?,
                    axis,
                })
            }
            _ => None,
        }
    }
}

impl DirtyTracked for ListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

/// keyed list 内容はテキストノードのみで束縛点を使わないため常に `None`
/// （`stagger_index_browser.rs::ListState` の `BindingSource` 実装と同じ
/// 意図）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `stagger_index_browser.rs::dispatch_action` と同じ手法。
fn dispatch_action(document: &Document, root: &Element, action: &str, payload: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", action)
        .expect("set_attribute must not fail");
    trigger
        .set_attribute("data-payload", payload)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// 受け入れ条件（本 PR の主眼）: 初期 1 件から末尾へ 1 件 `Insert` すると、
/// **一度も `pointerdown` を発火していない**新規挿入要素にも
/// `touch-action: none` が先行反映される（初回タッチが UA のスクロール
/// 判定に間に合わず `pointercancel` で中断する不具合の回帰固定）。
#[wasm_bindgen_test]
fn insert_prewires_touch_action_on_new_drag_item_before_any_pointerdown() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "drag-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a")]);
    let runtime = Runtime::mount("drag-root-container-1", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "append", "2:b");

    let items = root
        .query_selector_all("[data-testid='drag-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(items.length(), 2, "挿入後は 2 行になっていること");

    let inserted = items
        .get(1)
        .expect("second item must exist after insert")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");
    assert_eq!(
        inserted.style().get_property_value("touch-action"),
        Ok("none".to_string()),
        "新規挿入された data-fandhe-drag 要素は pointerdown 前から \
         touch-action: none を持つこと"
    );
}

/// `drag_gesture_browser.rs::pointer_event` と同じ意図・同じ理由で
/// `buttons` を 1（メインボタン押下中）に固定する（Bugbot High 是正
/// 「Resync browser test doesn't set buttons」、PR #2565 第 3 ラウンド）。
/// `handle_pointermove` は `buttons() == 0` を stale drag の自己解除条件
/// に使うため、`buttons` 未設定（既定 0）のまま `pointermove` を発火
/// すると位置更新前に `release_drag` が呼ばれ、意図した経路
/// （実際に位置が動くこと）を検証できない。
fn pointer_event(kind: &str, pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x as i32);
    init.set_client_y(client_y as i32);
    init.set_buttons(1);
    init.set_button(0);
    init.set_is_primary(true);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
        .into()
}

/// `element` の CSS カスタムプロパティを px 単位数値として読み取る
/// （`drag_gesture_browser.rs::custom_property_px` と同じ意図）。
fn custom_property_px(element: &Element, name: &str) -> Option<f64> {
    let html = element.dyn_ref::<HtmlElement>()?;
    let raw = html.style().get_property_value(name).ok()?;
    raw.trim_end_matches("px").trim().parse::<f64>().ok()
}

/// 受け入れ条件（codex-review P1 是正、PR #2565 第 2 ラウンド）: 既存行を
/// ポインタでドラッグして位置を移動させた後、キーを保ったまま内容だけを
/// 書き換える（`KeyedOp::Update`）と、`fandhe_frontend_wasm_client::
/// keyed_dom::sync_attrs` が `style` 属性を丸ごと削除するため
/// `touch-action: none` と移動済みの CSS カスタムプロパティが一時的に
/// 失われる。`Runtime::apply_update_for_dirty` が `Update` 適用後にも
/// `Self::resync_drag_gesture_attachments` を呼び直すため、
/// `DragController::resync_dom` により両方が復元されること（表示位置と
/// 保持位置の食い違いが残らないこと）を固定する。
#[wasm_bindgen_test]
fn update_of_existing_row_restores_touch_action_and_position_after_sync_attrs_strips_style() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "drag-root-container-2");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a")]);
    let runtime = Runtime::mount("drag-root-container-2", state).expect("mount must succeed");
    let root = runtime.root();

    let item = root
        .query_selector("[data-testid='drag-item']")
        .expect("query_selector must not fail")
        .expect("initial item must exist after mount");

    // マウント直後の先行 attach（`wire_drag_gesture` 内の
    // `resync_drag_gesture_attachments`）により、ドラッグ前から
    // touch-action: none が既に設定されていること（前提の確認）。
    assert_eq!(
        item.dyn_ref::<HtmlElement>()
            .expect("keyed list item must be an HtmlElement")
            .style()
            .get_property_value("touch-action"),
        Ok("none".to_string()),
        "マウント直後から touch-action: none が先行反映されていること"
    );

    // ポインタでドラッグして (10, 10) だけ移動させる（制約コンテナが無い
    // ため release 時に spring は起動せず、release 直前の位置がそのまま
    // 保持される、`drag_gesture_browser.rs` module doc と同じ前提）。
    item.dispatch_event(&pointer_event("pointerdown", 1, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointermove", 1, 10.0, 10.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointerup", 1, 10.0, 10.0))
        .expect("dispatch_event must not fail");

    let x_before = custom_property_px(&item, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y_before = custom_property_px(&item, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        (x_before - 10.0).abs() < 0.01 && (y_before - 10.0).abs() < 0.01,
        "ドラッグ後は (10, 10) へ移動していること: x={x_before} y={y_before}"
    );

    // キーは保ったまま内容だけを書き換える（KeyedOp::Update を誘発）。
    dispatch_action(&document, root, "rename", "1:renamed");

    let item_after = root
        .query_selector("[data-testid='drag-item']")
        .expect("query_selector must not fail")
        .expect("item must still exist after rename (same key)");
    assert!(
        item_after.is_same_node(Some(&item)),
        "Update は既存要素を差し替えないこと（同一ノードのまま）"
    );
    assert_eq!(
        item_after
            .dyn_ref::<HtmlElement>()
            .expect("keyed list item must be an HtmlElement")
            .style()
            .get_property_value("touch-action"),
        Ok("none".to_string()),
        "sync_attrs が style 属性を削除した後も touch-action: none が \
         再同期経路で復元されること"
    );
    let x_after = custom_property_px(&item_after, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y_after = custom_property_px(&item_after, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        (x_after - 10.0).abs() < 0.01 && (y_after - 10.0).abs() < 0.01,
        "移動済みの位置 (10, 10) も再同期経路で復元されること: \
         x={x_after} y={y_after}"
    );
}

/// 受け入れ条件（codex-review P1 是正、PR #2565 第 3 ラウンド）: 既存行の
/// [`DRAG_AXIS_ATTR`] を x → y へ書き換える（`KeyedOp::Update` を誘発）
/// と、`controller_for` が既存コントローラをそのまま返すだけで軸を読み
/// 直さない不具合により、以後のドラッグが旧軸（x）に従い続けてしまう。
/// `resync_drag_gesture_attachments` が再同期時に現在の属性値を
/// `DragController::set_axis` へ反映するため、Update 後のドラッグは
/// 新軸（y）に従うことを固定する。
#[wasm_bindgen_test]
fn update_changing_axis_attribute_applies_new_axis_to_subsequent_drag() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "drag-root-container-3");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a")]);
    let runtime = Runtime::mount("drag-root-container-3", state).expect("mount must succeed");
    let root = runtime.root();

    let item = root
        .query_selector("[data-testid='drag-item']")
        .expect("query_selector must not fail")
        .expect("initial item must exist after mount");

    // 軸制約なし（自由）の状態で一度ドラッグし、コントローラを新規
    // attach させておく（`resync_one_drag_element` の「既存コントローラ
    // のみ再同期する」分岐を確実に通す前提を作る）。
    item.dispatch_event(&pointer_event("pointerdown", 1, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointermove", 1, 5.0, 5.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointerup", 1, 5.0, 5.0))
        .expect("dispatch_event must not fail");

    // data-fandhe-drag-axis="y" へ変更（KeyedOp::Update を誘発）。
    dispatch_action(&document, root, "set-axis", "1:y");

    let item_after = root
        .query_selector("[data-testid='drag-item']")
        .expect("query_selector must not fail")
        .expect("item must still exist after axis update (same key)");
    assert!(
        item_after.is_same_node(Some(&item)),
        "Update は既存要素を差し替えないこと（同一ノードのまま）"
    );
    assert_eq!(
        item_after.get_attribute(DRAG_AXIS_ATTR),
        Some("y".to_string()),
        "data-fandhe-drag-axis 属性自体は y へ更新されていること（前提の確認）"
    );

    // y 軸のみのドラッグを試みる。旧軸（自由・実質 x 移動可）のまま
    // 残っていれば x も動くが、新軸（y）が反映されていれば x は固定
    // される。
    item_after
        .dispatch_event(&pointer_event("pointerdown", 2, 0.0, 0.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointermove", 2, 20.0, 20.0))
        .expect("dispatch_event must not fail");
    root.dispatch_event(&pointer_event("pointerup", 2, 20.0, 20.0))
        .expect("dispatch_event must not fail");

    // 第 2 のドラッグ起点（`origin_position`）は第 1 のドラッグ終了時点の
    // 位置 (5, 5)。軸 y が反映されていれば x 成分はこの起点値のまま固定
    // され、y 成分のみ移動量 (+20) が加算される。
    let x_final = custom_property_px(&item_after, DRAG_X_PROPERTY).unwrap_or(0.0);
    let y_final = custom_property_px(&item_after, DRAG_Y_PROPERTY).unwrap_or(0.0);
    assert!(
        (x_final - 5.0).abs() < 0.01,
        "軸が y へ更新された後は x 成分が第 2 ドラッグ開始時の値で \
         固定されるべき: x={x_final}"
    );
    assert!(
        (y_final - 25.0).abs() < 0.01,
        "軸が y へ更新された後は y 成分が自由に動くべき: y={y_final}"
    );
}
