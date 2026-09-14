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
    ///
    /// # 要素あたり 1 pointer に限定する（codex-review P1 是正）
    ///
    /// [`DragController`] は要素ごとに起点（`DragStart`）を 1 個しか
    /// 保持しないため、同一要素へ 2 本目のポインタ（2 本指操作等）が
    /// `pointerdown` すると起点が上書きされ、片方だけ離した
    /// `pointerup`/`pointercancel` が共有状態を `on_release` してしまう
    /// （残っているはずのもう一本の追跡が孤立する）。すでに `active` に
    /// 同じ要素を追跡するエントリがあれば、この `pointerdown` は無視する
    /// （最初に押した 1 本だけがその要素のドラッグを所有する）。
    ///
    /// # 同一 `pointer_id` の stale エントリを自己解除する（Cursor Bugbot
    /// 是正「Stale drag can lock element」、PR #2565）
    ///
    /// UA は `pointerup`/`pointercancel` を送るまで同じ `pointer_id` を
    /// 再利用しない（ある `pointer_id` の `pointerdown` は、その ID が
    /// 既に離されている場合にのみ発生する）。したがって `active` に同じ
    /// `pointer_id` のエントリが残っていれば、それは前回セッションの
    /// 解放漏れ（[`handle_pointermove`] の `buttons() == 0` 自己修復・
    /// 下記 [`wire_drag_gesture`] の `window` 側 release リスナーの
    /// いずれも取りこぼした場合）による stale な残留であると確定できる。
    /// 新規ポインタ操作を拒否する前に、まずこの pointer_id を
    /// [`release_drag`] で解放してから通常の判定へ進む。これにより、
    /// 何らかの理由で解放を取りこぼしたドラッグが同じ `pointer_id` を
    /// 永久にロックし続けることはない。
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
        let pointer_id = pointer_event.pointer_id();
        if active.borrow().contains_key(&pointer_id) {
            release_drag(pointer_id, active);
        }
        let already_tracked = active
            .borrow()
            .values()
            .any(|(existing, _)| existing.is_same_node(Some(&drag_element)));
        if already_tracked {
            return;
        }
        let Some(controller) = controller_for(&drag_element) else {
            return;
        };
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

    /// `pointer_id` の追跡を解除し、[`DragController::on_release`]・
    /// [`DRAGGING_STATE_ATTR`] 除去を行う（`handle_pointerup_or_cancel`・
    /// 下記 stale ガードの共通処理）。追跡が無ければ no-op。
    fn release_drag(pointer_id: i32, active: &ActiveDrags) {
        let Some((drag_element, controller)) = active.borrow_mut().remove(&pointer_id) else {
            return;
        };
        controller.borrow_mut().on_release();
        let _ = drag_element.remove_attribute(DRAGGING_STATE_ATTR);
    }

    /// `pointermove`: 追跡中の `pointer_id` のみ
    /// [`DragController::on_pointer_move`] へ座標を渡す。
    ///
    /// # stale な追跡の自己解除（`buttons == 0` ガード、`angle_slider.rs`
    /// の同名節と同型）
    ///
    /// [`handle_pointerdown`] の `set_pointer_capture` が失敗する（`Result`
    /// を無視している）、または実装によっては暗黙 capture が外れる等で
    /// ポインタが `root` の外へ出ると、そこで発生する `pointerup`/
    /// `pointercancel` は `root` に配線したリスナーへ到達せず `active` の
    /// エントリが残留する（`DRAGGING_STATE_ATTR` が外れない「幽霊ドラッグ」）。
    /// `pointermove` は capture 中でなくとも要素外の移動で発火しうるため、
    /// `buttons() == 0`（どのボタンも押されていない）を都度確認し、
    /// 該当すれば `release_drag` で追跡を自己修復する（Bugbot 指摘の是正）。
    ///
    /// # このガードだけでは不十分（Cursor Bugbot 再指摘「Stale drag can
    /// lock element」是正、PR #2565）
    ///
    /// 本ガードは `root` へ配線したリスナー上で動くため、`root` に発火
    /// する後続の `pointermove` が無ければ実行されない。ポインタが
    /// `root` の外へ完全に離脱したまま二度と `root` 内へ戻らずに離される
    /// （capture 失敗時に典型的なケース）と、`pointermove` 自体が
    /// `root` へ到達しないため本ガードも発火せず、`active` エントリが
    /// 永久に残留し得る（同じ `pointer_id` を使う次のドラッグ開始が
    /// [`handle_pointerdown`] の stale 自己解除で救済されるが、
    /// `DRAGGING_STATE_ATTR` は救済されるまで残置される）。これを塞ぐ
    /// 第 2 の防御として [`wire_drag_gesture`] は `pointerup`/
    /// `pointercancel` を `root` に加えて `window` へも capture フェーズで
    /// 登録し、要素の位置に関わらず解放を捕捉する。
    fn handle_pointermove(event: &Event, active: &ActiveDrags) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let pointer_id = pointer_event.pointer_id();
        let entry = active.borrow().get(&pointer_id).map(|(_, c)| Rc::clone(c));
        let Some(controller) = entry else {
            return;
        };
        if pointer_event.buttons() == 0 {
            release_drag(pointer_id, active);
            return;
        }
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
        release_drag(pointer_event.pointer_id(), active);
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

    /// `root` 自身・子孫のうち [`DRAG_ATTR`] を持つ opt-in 要素すべてに
    /// 先行して [`DragController`] を attach する（`content_height.rs::
    /// sync_content_height` の「root 自身が対象パーツである場合も
    /// 同期対象に含める」パターンと同型。`query_selector_all` は子孫のみを
    /// 列挙し root 自身を含まないため個別に確認する）。
    ///
    /// # 接触前に `touch-action: none` を確定させる（codex-review P1 是正）
    ///
    /// [`controller_for`] は従来 `pointerdown` ハンドラから初めて呼ばれ、
    /// `DragController::attach` の `touch-action: none` 設定もそこで
    /// 初めて行われていた。しかしタッチのスクロール可否は UA が最初の
    /// タッチ接触を処理する時点で確定するため、`pointerdown` イベント
    /// ハンドラ内（＝その接触が既に処理された後）で設定するのでは遅く、
    /// 最初のタッチドラッグがスクロールに奪われて `pointercancel` になる
    /// （モジュール doc「`touch-action: none`」節）。SSR 出力に静的付与
    /// された opt-in 要素はマウント/ハイドレート時点で DOM に存在する
    /// ため、配線直後にここで先行 attach しておけば実際の接触より確実に
    /// 早く `touch-action: none` が反映される。
    ///
    /// # 再描画後にも再実行が必要（codex-review P1 是正、PR #2565）
    ///
    /// [`crate::lib::Runtime::mount`]/[`Runtime::hydrate`] からの初回配線
    /// 直後だけでなく、[`crate::lib::Runtime::apply_subtree_swap`]（構造
    /// フォールバック再描画・View Transitions 更新の両方が経由する唯一の
    /// DOM 差し替え実装）が `root` の子ノードを丸ごと新規ノードへ置き換え
    /// た**直後にも**呼び直す必要がある。イベント委譲用の 5 リスナーは
    /// `root` 自身が差し替えられないため再登録不要（`apply_subtree_swap`
    /// の doc コメント参照）だが、`touch-action: none` は要素ごとの
    /// インラインスタイル（[`DragController::attach`]）であり、新規
    /// ノードは属性・スタイルを引き継がない。呼び直しを欠くと、再描画後に
    /// 新規生成された opt-in 要素は最初の `pointerdown` ハンドラ内で初めて
    /// `touch-action: none` が設定されることになり、タッチデバイスでは
    /// その設定が UA のスクロール判定より後になるため初回タッチドラッグが
    /// `pointercancel` で中断する（本節冒頭の理由と同型）。[`controller_for`]
    /// が `retain(is_connected)` で切断済みエントリを間引き `is_same_node`
    /// で重複登録を避けるため、本関数は何度呼んでも安全（冪等）。
    pub(crate) fn resync_drag_gesture_attachments(root: &Element) {
        if root.has_attribute(DRAG_ATTR) {
            let _ = controller_for(root);
        }
        let selector = format!("[{DRAG_ATTR}]");
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return;
        };
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Some(element) = node.dyn_ref::<Element>() else {
                continue;
            };
            let _ = controller_for(element);
        }
    }

    /// `root` へドラッグ検知の 5 リスナー（pointerdown/pointermove/
    /// pointerup/pointercancel/keydown）を委譲登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`gesture.rs::wire_gesture` と同じく登録回数を
    /// 定数個に抑える方針（A04 対策）で全リスナーを capture フェーズで
    /// 登録する（子孫の `stopPropagation()` に対する頑健性、同モジュール
    /// と同じ理由）。登録後に [`resync_drag_gesture_attachments`] で
    /// 既存の opt-in 要素へ `touch-action: none` を先行反映する。
    ///
    /// # `pointerup`/`pointercancel` は `window` にも登録する（Cursor
    /// Bugbot 是正「Stale drag can lock element」、PR #2565）
    ///
    /// `root` 単独では、ポインタが `root` の外へ出たまま離される
    /// ケース（[`handle_pointermove`] doc 参照）を取りこぼす。`pointerup`/
    /// `pointercancel` の解放ハンドラを `window` の capture フェーズへも
    /// 追加登録することで、ターゲット要素の位置に関わらず解放を確実に
    /// 捕捉する。`window` は `root` の祖先であるため、`root` 内で発生した
    /// イベントは両リスナーから 2 回呼ばれるが、[`release_drag`] は
    /// `active` から見つからない `pointer_id` を no-op とするため冪等
    /// （2 回目は何もしない）。`Runtime::mount`/`Runtime::hydrate` は
    /// `root` ごとに 1 回だけ本関数を呼ぶため、`window` への登録回数も
    /// 有界（A04 方針に反しない）。`window()` が取得できない非ブラウザ
    /// 環境（native テスト等）では `root` 側のみで動作し続ける
    /// fail-safe（`let _ =` で結果を握り潰し、致命的エラーにしない）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する
    /// （`root` への登録のみ。`window` への追加登録の失敗は fail-safe
    /// に握り潰す、上記節参照）。
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
        // `root` 外での release を捕捉するための追加登録（本関数 doc
        // 「`pointerup`/`pointercancel` は `window` にも登録する」節）。
        // 失敗は fail-safe に握り潰す（`window()` 不在の非ブラウザ環境等）。
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback_and_bool(
                "pointerup",
                pointerup_or_cancel_closure.as_ref().unchecked_ref(),
                true,
            );
            let _ = window.add_event_listener_with_callback_and_bool(
                "pointercancel",
                pointerup_or_cancel_closure.as_ref().unchecked_ref(),
                true,
            );
        }
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

        resync_drag_gesture_attachments(&root);

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_drag_gesture;

/// [`wiring::resync_drag_gesture_attachments`] を `crate::lib::Runtime` から
/// 呼べるよう再エクスポートする（[`crate::lib::Runtime::apply_subtree_swap`]
/// が構造フォールバック再描画・View Transitions 更新後に呼び直す唯一の
/// 経路、codex-review P1 是正・PR #2565）。`wiring` モジュール自体は非公開
/// のため、`wire_drag_gesture` と同型の再エクスポートで crate 内へ公開する。
#[cfg(target_arch = "wasm32")]
pub(crate) use wiring::resync_drag_gesture_attachments;

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
