//! pointer capture ベースの汎用ドラッグ DOM 配線（イシュー #2535、親 #2530）。
//!
//! carousel（#2541）・list の並べ替え（#2544）の基盤となる、
//! `fandhe-frontend-animation` の [`fandhe_frontend_animation::drag::DragController`]
//! （軸制約・範囲クランプ・離脱速度推定 + spring 復帰の演算・DOM 書き込み）を
//! pointer/keyboard イベントへ配線する。3 層構成
//! （`docs/design/motion-reference-adoption-policy.md` §6）に従い、本モジュールは
//! イベントから `(client_x, client_y, time_stamp)` を抽出して渡すのみで、
//! 演算・rAF ループ・spring 計算は一切持たない。
//!
//! # 命名衝突の回避（[`crate::headless_file_upload::wire_drag_and_drop`] との区別）
//!
//! `headless_file_upload.rs` の `wire_drag_and_drop`（HTML5 `DragEvent`
//! ベースのファイルドロップ、イシュー #1609）と本モジュールは全くの別物
//! （後者は Pointer Events ベースの汎用ドラッグ）である。読者の混同を防ぐ
//! ため、feature 名は `drag-gesture`、モジュール名は `drag_gesture`、
//! `data-*` 属性はすべて `data-fandhe-drag*` プレフィックスで統一し、
//! HTML5 Drag and Drop API 側の語彙（`data-fandhe-drag-and-drop-*` 等）とは
//! 意図的に重ねない。
//!
//! # `data-*` 属性契約（Issue 記載パターンとの意図的な差分）
//!
//! イシュー本文は `data-state="dragging"` を例示するが、`gesture.rs` が
//! 「`data-state`/`data-active` は headless-ui 各部品が既に状態機械として
//! 使用済みの語彙であり、無関係な意味で上書きすると混乱する」という理由で
//! `data-fandhe-*` プレフィックスを採用した判断（同ファイル冒頭 doc）を
//! 本モジュールでも踏襲し、`data-state="dragging"` は採用しない。
//!
//! | 属性 | 種別 | 役割 |
//! |---|---|---|
//! | [`DRAG_ATTR`] | opt-in（著者が SSR 出力に静的付与） | ドラッグ対象要素を宣言する |
//! | [`DRAG_AXIS_ATTR`] | opt-in（省略可、値 `"x"`/`"y"`） | 軸制約。省略時・未知の値は自由（2 軸） |
//! | [`DRAG_CONSTRAINTS_ATTR`] | opt-in（省略可、値は無視する存在属性） | この属性を持つ最も近い祖先を範囲制約コンテナとして使う |
//! | [`DRAGGING_STATE_ATTR`] | 状態（wasm-full が動的に付け外す、存在で真） | ドラッグ中であることを示す |
//!
//! 範囲制約は「祖先要素の `getBoundingClientRect()`」で表現し、著者が
//! 数値を `data-*` に書く方式は採らない（CSS レイアウト変更に自動追随
//! する）。
//!
//! # 位置の書き込み先
//!
//! `fandhe_frontend_animation::drag::DragController` が CSS カスタム
//! プロパティ（`--fandhe-drag-x`/`--fandhe-drag-y`）へ書き込む
//! （`docs/design/animation-core-architecture.md` §2/§6 の演算・DOM 適用
//! は `fandhe-frontend-animation` の責務という分担どおり）。
//!
//! # 要素あたり 1 個の [`DragController`] を持続させる
//!
//! ドラッグの位置は複数回の pointerdown/pointerup セッションをまたいで
//! 蓄積する累積オフセットのため、[`DragController`] は要素単位で持続させる
//! 必要がある。`Element` は `Hash` を実装しないため、`sidebar.rs` の
//! `REGISTERED_KEYDOWN_ROOTS`（`RefCell<Vec<(Element, Rc<..>)>>` +
//! `is_same_node` 線形探索）と同じパターンを踏襲する（[`wiring::drag_controllers`]
//! 参照）。
//!
//! # dispatch 非依存（`angle_slider.rs` との違い）
//!
//! `angle_slider.rs` は `Component` の再描画（`rerender_subtree`）を挟んだ
//! 要素再解決（`PartKey`）を持つが、本モジュールは `dispatch` チャネルを
//! 持たない属性専用配線であり再描画統合が無いため、その再解決は不要
//! （`gesture.rs`/`sidebar.rs` と同型の「dispatch を伴わない純粋な視覚配線」）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_gesture` の直後で `Self::wire_drag_gesture` を呼ぶ
//! （feature `drag-gesture`、既定 on）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`DRAG_AXIS_ATTR`] の値は完全一致の許可リスト（`"x"`/`"y"` のみ、
//! それ以外は無制約 [`fandhe_frontend_animation::drag::DragAxis::Free`]
//! へフォールバック）で解釈し、DOM/セレクタ文字列へ利用者制御の生文字列を
//! 埋め込まない。範囲制約は数値パースではなく要素参照（`closest`）で
//! 表現するため、不正な数値文字列によるクラッシュ・NaN 伝播の経路自体が
//! 生まれない。[`DRAGGING_STATE_ATTR`] は常に空文字列 `""` の存在属性のみ。

/// opt-in（著者が SSR 出力に静的付与）: ドラッグ対象要素を宣言する。
pub const DRAG_ATTR: &str = "data-fandhe-drag";
/// opt-in（省略可、値 `"x"`/`"y"`）: 軸制約。
pub const DRAG_AXIS_ATTR: &str = "data-fandhe-drag-axis";
/// opt-in（省略可、値は無視する存在属性）: 範囲制約コンテナの宣言。
pub const DRAG_CONSTRAINTS_ATTR: &str = "data-fandhe-drag-constraints";
/// 状態（wasm-full が動的に付け外す、存在で真）: ドラッグ中であることを示す。
pub const DRAGGING_STATE_ATTR: &str = "data-fandhe-dragging";

/// キーボード（矢印キー）1 回あたりの移動量（px）。
pub const DRAG_KEYBOARD_STEP_PX: f64 = 8.0;

/// `KeyboardEvent::key()` が矢印キーであれば移動方向の単位ベクトルを返す。
#[must_use]
pub fn arrow_key_direction(key: &str) -> Option<(f64, f64)> {
    match key {
        "ArrowLeft" => Some((-1.0, 0.0)),
        "ArrowRight" => Some((1.0, 0.0)),
        "ArrowUp" => Some((0.0, -1.0)),
        "ArrowDown" => Some((0.0, 1.0)),
        _ => None,
    }
}

/// [`DRAG_AXIS_ATTR`] の値を [`fandhe_frontend_animation::drag::DragAxis`]
/// へ解釈する。完全一致の許可リスト方式（`"x"`/`"y"` のみ）で、
/// それ以外の値・属性欠落は無制約（`Free`）へフォールバックする
/// （不正値で panic しない、モジュール doc のセキュリティ不変条件）。
#[must_use]
pub fn parse_drag_axis(value: Option<&str>) -> fandhe_frontend_animation::drag::DragAxis {
    use fandhe_frontend_animation::drag::DragAxis;
    match value {
        Some("x") => DragAxis::X,
        Some("y") => DragAxis::Y,
        _ => DragAxis::Free,
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        arrow_key_direction, parse_drag_axis, DRAGGING_STATE_ATTR, DRAG_ATTR, DRAG_AXIS_ATTR,
        DRAG_CONSTRAINTS_ATTR, DRAG_KEYBOARD_STEP_PX,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    // `fandhe_animation` はワークスペース内の推移依存（本クレートの直接
    // 依存は `fandhe-frontend-animation` のみ）であり、名前解決に公開
    // するには再エクスポート経由が必要（`crate::lib` の同種コメント参照）。
    use fandhe_frontend_animation::drag::{DragConstraint, DragController};
    use fandhe_frontend_animation::fandhe_animation::interpolate::Vec2;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, KeyboardEvent, PointerEvent};

    thread_local! {
        /// 要素単位で持続する [`DragController`] のレジストリ（モジュール
        /// doc「要素あたり 1 個の `DragController` を持続させる」参照）。
        static DRAG_CONTROLLERS: RefCell<Vec<(Element, Rc<RefCell<DragController>>)>> =
            const { RefCell::new(Vec::new()) };
    }

    /// pointer_id ごとに追跡中のドラッグ対象要素と、その
    /// [`DragController`] を保持する共有マップの型エイリアス
    /// （`clippy::type_complexity` 回避、`raf_driver.rs::SharedClosureSlot`
    /// と同型）。
    type ActiveDrags = Rc<RefCell<HashMap<i32, (Element, Rc<RefCell<DragController>>)>>>;

    /// `element` に紐づく [`DragController`] を返す（無ければ
    /// [`DragController::attach`] で新規作成して登録する）。呼び出しの
    /// たびに、もはや文書に接続されていない要素のエントリを間引く
    /// （`sidebar.rs::connected_registered_keydown_roots` と同型の遅延
    /// 掃除、明示的な解除リスナーを持たない設計）。
    fn controller_for(element: &Element) -> Option<Rc<RefCell<DragController>>> {
        DRAG_CONTROLLERS.with(|cell| {
            let mut controllers = cell.borrow_mut();
            controllers.retain(|(existing, _)| existing.is_connected());
            if let Some((_, controller)) = controllers
                .iter()
                .find(|(existing, _)| existing.is_same_node(Some(element)))
            {
                return Some(Rc::clone(controller));
            }
            let html_element = element.clone().dyn_into::<HtmlElement>().ok()?;
            let axis = parse_drag_axis(element.get_attribute(DRAG_AXIS_ATTR).as_deref());
            let controller = Rc::new(RefCell::new(DragController::attach(html_element, axis)));
            controllers.push((element.clone(), Rc::clone(&controller)));
            Some(controller)
        })
    }

    /// `root` 配下・`opt_in_attr` を持つ最も近い祖先（自身含む）を返す
    /// （`gesture.rs::closest_opted_in` と同一ロジックの独立定義。
    /// 意図はモジュール doc「命名衝突の回避」節と同じく文脈の分離）。
    fn closest_opted_in(root: &Element, target: &Element, opt_in_attr: &str) -> Option<Element> {
        let selector = format!("[{opt_in_attr}]");
        let matched = target.closest(&selector).ok().flatten()?;
        root.contains(Some(&matched)).then_some(matched)
    }

    /// `target` が編集可能要素（フォームコントロール・`contenteditable`
    /// 領域内）か（`gesture.rs::is_editable_target` と同一ロジックの
    /// 独立定義）。矢印キーによる nudge を、通常のテキスト入力・選択操作
    /// と衝突させないためのガード。
    fn is_editable_target(target: &Element) -> bool {
        matches!(target.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
            || target
                .dyn_ref::<HtmlElement>()
                .is_some_and(HtmlElement::is_content_editable)
    }

    /// `drag_element` の [`super::DRAG_CONSTRAINTS_ATTR`] 祖先コンテナを
    /// 実測し、`controller` へ反映する（`root` 内に祖先が無ければ
    /// 無制約）。
    ///
    /// # 現在のオフセットを差し引く計算
    ///
    /// `drag_element.get_bounding_client_rect()` は既に適用済みの
    /// `--fandhe-drag-x`/`-y` オフセット（[`DragController::position`]）を
    /// 反映した座標を返すため、そのまま使うと「現在位置からの残り移動量」
    /// ではなく「元のレイアウト位置からの移動量」がずれる。オフセット
    /// 適用前の素の矩形位置（`element_rect - offset`）を逆算してから
    /// コンテナ矩形との差で min/max を求める。
    fn measure_and_apply_constraint(
        root: &Element,
        drag_element: &Element,
        controller: &Rc<RefCell<DragController>>,
    ) {
        let constraint =
            closest_opted_in(root, drag_element, DRAG_CONSTRAINTS_ATTR).map(|container| {
                let container_rect = container.get_bounding_client_rect();
                let element_rect = drag_element.get_bounding_client_rect();
                let offset = controller.borrow().position();
                let natural_left = element_rect.left() - offset.x;
                let natural_right = element_rect.right() - offset.x;
                let natural_top = element_rect.top() - offset.y;
                let natural_bottom = element_rect.bottom() - offset.y;
                DragConstraint {
                    min_x: container_rect.left() - natural_left,
                    max_x: container_rect.right() - natural_right,
                    min_y: container_rect.top() - natural_top,
                    max_y: container_rect.bottom() - natural_bottom,
                }
            });
        controller.borrow_mut().set_constraint(constraint);
    }

    /// `pointerdown`: opt-in 要素を解決し、pointer capture・範囲制約の
    /// 実測・[`DragController::on_pointer_down`] 呼び出し・
    /// [`DRAGGING_STATE_ATTR`] 付与・pointer_id 追跡登録を行う。
    fn handle_pointerdown(root: &Element, event: &Event, active: &ActiveDrags) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        let Some(drag_element) = closest_opted_in(root, &target, DRAG_ATTR) else {
            return;
        };
        let Some(controller) = controller_for(&drag_element) else {
            return;
        };
        let pointer_id = pointer_event.pointer_id();
        let _ = drag_element.set_pointer_capture(pointer_id);
        measure_and_apply_constraint(root, &drag_element, &controller);
        controller.borrow_mut().on_pointer_down(
            Vec2 {
                x: pointer_event.client_x() as f64,
                y: pointer_event.client_y() as f64,
            },
            event.time_stamp(),
        );
        let _ = set_dom_attribute(&drag_element, DRAGGING_STATE_ATTR, "");
        active
            .borrow_mut()
            .insert(pointer_id, (drag_element, controller));
    }

    /// `pointermove`: 追跡中の `pointer_id` のみ
    /// [`DragController::on_pointer_move`] へ座標を渡す。
    fn handle_pointermove(event: &Event, active: &ActiveDrags) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let pointer_id = pointer_event.pointer_id();
        let entry = active.borrow().get(&pointer_id).map(|(_, c)| Rc::clone(c));
        let Some(controller) = entry else {
            return;
        };
        controller.borrow_mut().on_pointer_move(
            Vec2 {
                x: pointer_event.client_x() as f64,
                y: pointer_event.client_y() as f64,
            },
            event.time_stamp(),
        );
    }

    /// `pointerup`/`pointercancel`: 追跡を解除し
    /// [`DragController::on_release`] を呼んで [`DRAGGING_STATE_ATTR`] を
    /// 外す（同一 [`Closure`] を両イベント名で登録しリスナー数を節約、
    /// `gesture.rs::handle_pointerup_or_cancel` と同型）。
    fn handle_pointerup_or_cancel(event: &Event, active: &ActiveDrags) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let pointer_id = pointer_event.pointer_id();
        let Some((drag_element, controller)) = active.borrow_mut().remove(&pointer_id) else {
            return;
        };
        controller.borrow_mut().on_release();
        let _ = drag_element.remove_attribute(DRAGGING_STATE_ATTR);
    }

    /// `keydown`: フォーカス中の opt-in 要素上で矢印キーが押されたら
    /// [`DragController::nudge`] を呼ぶ。編集可能要素上（`gesture.rs` と
    /// 同じ理由）は発火しない。
    fn handle_keydown(root: &Element, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some((dx, dy)) = arrow_key_direction(&keyboard_event.key()) else {
            return;
        };
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        if is_editable_target(&target) {
            return;
        }
        let Some(drag_element) = closest_opted_in(root, &target, DRAG_ATTR) else {
            return;
        };
        let Some(controller) = controller_for(&drag_element) else {
            return;
        };
        measure_and_apply_constraint(root, &drag_element, &controller);
        controller.borrow_mut().nudge(Vec2 {
            x: dx * DRAG_KEYBOARD_STEP_PX,
            y: dy * DRAG_KEYBOARD_STEP_PX,
        });
        keyboard_event.prevent_default();
    }

    /// `root` へドラッグ検知の 5 リスナー（pointerdown/pointermove/
    /// pointerup/pointercancel/keydown）を委譲登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`gesture.rs::wire_gesture` と同じく登録回数を
    /// 定数個に抑える方針（A04 対策）で全リスナーを capture フェーズで
    /// 登録する（子孫の `stopPropagation()` に対する頑健性、同モジュール
    /// と同じ理由）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_drag_gesture(root: Element) -> Result<(), JsValue> {
        let active: ActiveDrags = Rc::new(RefCell::new(HashMap::new()));

        let pointerdown_root = root.clone();
        let pointerdown_active = Rc::clone(&active);
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&pointerdown_root, &event, &pointerdown_active);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerdown_closure.forget();

        let pointermove_active = Rc::clone(&active);
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&event, &pointermove_active);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointermove",
            pointermove_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointermove_closure.forget();

        let pointerup_active = Rc::clone(&active);
        let pointerup_or_cancel_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerup_or_cancel(&event, &pointerup_active);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerup",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
            true,
        )?;
        root.add_event_listener_with_callback_and_bool(
            "pointercancel",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerup_or_cancel_closure.forget();

        let keydown_root = root.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event);
        });
        root.add_event_listener_with_callback_and_bool(
            "keydown",
            keydown_closure.as_ref().unchecked_ref(),
            true,
        )?;
        keydown_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_drag_gesture;

#[cfg(test)]
mod tests {
    use super::{arrow_key_direction, parse_drag_axis};

    #[test]
    fn arrow_key_direction_matches_four_keys() {
        assert_eq!(arrow_key_direction("ArrowLeft"), Some((-1.0, 0.0)));
        assert_eq!(arrow_key_direction("ArrowRight"), Some((1.0, 0.0)));
        assert_eq!(arrow_key_direction("ArrowUp"), Some((0.0, -1.0)));
        assert_eq!(arrow_key_direction("ArrowDown"), Some((0.0, 1.0)));
        assert_eq!(arrow_key_direction("Enter"), None);
        assert_eq!(arrow_key_direction(""), None);
    }

    #[test]
    fn parse_drag_axis_accepts_only_exact_x_or_y() {
        use fandhe_frontend_animation::drag::DragAxis;
        assert_eq!(parse_drag_axis(Some("x")), DragAxis::X);
        assert_eq!(parse_drag_axis(Some("y")), DragAxis::Y);
        assert_eq!(parse_drag_axis(Some("X")), DragAxis::Free);
        assert_eq!(parse_drag_axis(Some("both")), DragAxis::Free);
        assert_eq!(parse_drag_axis(None), DragAxis::Free);
    }
}
