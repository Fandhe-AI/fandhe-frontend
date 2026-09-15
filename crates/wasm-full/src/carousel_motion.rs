//! carousel の Motion+ Carousel 相当拡張（ドラッグ + spring スナップ）配線層
//! （イシュー #2541、親 #2530/#2476、feature `carousel-motion`）。
//!
//! # 責務境界（3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）
//!
//! ポインタ進行度 → 着地 index の計算・spring アニメーションの rAF 実行・
//! DOM 書き込みは [`fandhe_frontend_animation::carousel::CarouselTrack`] の
//! 責務。本モジュールは以下のみを担う:
//!
//! 1. opt-in 属性（[`CAROUSEL_DRAG_ATTR`]）を持つ carousel root への
//!    pointerdown/pointermove/pointerup/pointercancel の委譲登録
//!    （`root` 1 箇所へ 4 リスナー、`gesture.rs` と同じ「登録回数を定数個に
//!    抑える」方針）。
//! 2. `item-group`/先頭 `item` の解決・`getBoundingClientRect()` による
//!    1 スライド分の px 計測。
//! 3. settle 完了時、`on_action`（[`crate::lib::Runtime::wire`] の閉包）へ
//!    `"goto"` を dispatch する。
//!
//! # `data-*` 属性の命名（他クレートとの契約）
//!
//! [`CAROUSEL_DRAG_ATTR`]/[`CAROUSEL_DRAGGING_STATE_ATTR`] のリテラル値は
//! `fandhe_frontend_pre_styled_ui::carousel_motion` の同名定数と一致する
//! 必要がある（`crate::hold_to_confirm`/`crate::magnetic` と同じ「リテラル
//! の写し」契約。wasm-full は pre-styled-ui に依存しないため型共有は
//! できない）。ドリフトは
//! `crates/pre-styled-ui/tests/motion_carousel_css.rs` が本ファイルを
//! 読んで fail-closed に検知する。
//!
//! # `CarouselTrack` の保持責任
//!
//! [`fandhe_frontend_animation::carousel::CarouselTrack`] は settle 中の
//! spring アニメーション（`AnimationLoop`）を自身のフィールドに保持する。
//! settle 完了前に `CarouselTrack` 自体を drop すると `AnimationLoop` も
//! drop され、アニメーションが途中で停止してしまう
//! （`hold_to_confirm.rs::finish_confirmation` doc の「never drop here」と
//! 同じ制約）。本モジュールは carousel root ごとに 1 個の
//! `Rc<RefCell<Option<CarouselTrack>>>`（[`TrackSlot`]）を
//! [`TrackRegistry`] で保持し、次の pointerdown が**同じ** root 上で
//! 上書きするまでは settle 完了後も保持し続ける（`magnetic.rs::
//! ActiveMagnetic` と同型のパターン）。settle 完了コールバック自身は
//! この slot に触れない（tick 実行中に自分自身を drop しないため、
//! `hold_to_confirm.rs` と同じ「never drop from within the tick」規律）。
//!
//! # 複数 carousel 間の状態分離（[`TrackRegistry`]、codex-review/Cursor
//! Bugbot 指摘 是正）
//!
//! `wire_carousel_motion_events` は `Runtime` の mount root 1 個へ
//! 委譲登録する（[`crate::lib::Runtime::wire_carousel_motion`]、1
//! インスタンスにつき 1 回のみ呼ばれる）ため、素朴に単一の
//! `TrackSlot`/`DragMetaSlot` を共有すると、ある carousel の settle
//! アニメーション進行中に**別の** carousel root をドラッグし始めた
//! 時点で前者の状態が上書きされ、`AnimationLoop` が中断される（結果を
//! 待つコールバックが呼ばれないまま `goto` dispatch が失われる）。
//! 是正として [`TrackRegistry`]（carousel root ごとの
//! `(TrackSlot, DragMetaSlot)` を保持する小さな線形テーブル）を導入した。
//! `handle_pointerdown` は `target.closest(CAROUSEL_ROOT_SELECTOR)` で
//! 解決した carousel root をキーに [`slot_for`] を引いてから操作する
//! （pointerdown 自体がまだアクティブな [`DragMeta`] を持たないため
//! pointer_id 検索は使えない）。`handle_pointermove`/
//! `handle_pointer_release` は `event.target()` の部分木解決には頼らず
//! `pointer_id` をキーに [`find_active_slot`] で直接引く（`closest()` に
//! 頼らない理由は [`find_active_slot`] doc 参照）。`Element` は
//! `Hash`/`Eq` を実装しないため両者とも `Node::is_same_node` による線形
//! 探索で同一性判定する（ponytail: 1 ページ内の carousel 数は通常一桁〜
//! 十数個程度に収まる想定の簡略化。件数が実運用で問題になった場合は
//! `js_sys::WeakMap` 等キー付きコレクションへの切替を検討する）。

/// opt-in（著者が SSR 出力に静的に付与）: root へ付与するとドラッグ +
/// spring スナップを有効化するマーカー属性。値は `""`（非 loop）または
/// `"loop"`（末尾からの折り返し）のみを意味を持ち、それ以外は `""` 扱い
/// （[`parse_loop_opt_in`] 参照）。
pub const CAROUSEL_DRAG_ATTR: &str = "data-fandhe-carousel-drag";

/// 状態属性: ドラッグ中〜spring 収束完了まで carousel root へ付与される。
pub const CAROUSEL_DRAGGING_STATE_ATTR: &str = "data-fandhe-carousel-dragging";

/// [`CAROUSEL_DRAG_ATTR`] の属性値から loop モードを決める。属性が存在
/// しない（`None`）場合のみ opt-in 自体が無効（呼び出し側は属性の有無を
/// 別途 `closest()` で判定する）。値が `"loop"` なら `true`、それ以外
/// （`""` を含む）はすべて `false` 扱い（不正な opt-in 値は `""` 扱い、
/// 実装計画 §2.3 のセキュリティ方針）。
#[must_use]
pub fn parse_loop_opt_in(attr_value: Option<&str>) -> Option<bool> {
    attr_value.map(|value| value == "loop")
}

/// `item-group` の `data-orientation` 属性値から、ドラッグ軸が縦方向
/// （`client_y` を使う）かどうかを決める。`"vertical"` のみ縦方向、それ
/// 以外（未設定含む）は横方向。
#[must_use]
pub fn is_vertical_orientation(orientation: Option<&str>) -> bool {
    orientation == Some("vertical")
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use std::cell::RefCell;
    use std::rc::Rc;

    use fandhe_frontend_animation::carousel::CarouselTrack;
    use web_sys::{HtmlElement, PointerEvent};

    use super::{
        is_vertical_orientation, parse_loop_opt_in, CAROUSEL_DRAGGING_STATE_ATTR,
        CAROUSEL_DRAG_ATTR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use crate::events::ActionRef;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// `[data-fandhe-carousel-drag]` を `closest()` で辿るためのセレクタ。
    const CAROUSEL_ROOT_SELECTOR: &str = "[data-fandhe-carousel-drag]";
    /// `item-group` を `closest()` で辿るためのセレクタ
    /// （`crates/headless-ui/src/carousel.rs` の ANATOMY `data-scope`/
    /// `data-part` と一致）。
    const ITEM_GROUP_SELECTOR: &str = "[data-scope=\"carousel\"][data-part=\"item-group\"]";
    /// `item-group` 配下の `item` 一覧を数える・先頭要素を計測するための
    /// セレクタ。
    const ITEM_SELECTOR: &str = "[data-scope=\"carousel\"][data-part=\"item\"]";

    /// [`CarouselTrack`] の保持スロット（モジュール doc「`CarouselTrack`
    /// の保持責任」節参照）。
    type TrackSlot = Rc<RefCell<Option<CarouselTrack>>>;

    /// アクティブなドラッグの付随情報（[`CarouselTrack`] 本体とは別に
    /// pointerdown〜release の間だけ保持する）。
    struct DragMeta {
        pointer_id: i32,
        carousel_root: Element,
        item_group: HtmlElement,
        slide_px: f64,
        vertical: bool,
        origin_coord: f64,
        moved: bool,
    }

    type DragMetaSlot = Rc<RefCell<Option<DragMeta>>>;

    /// carousel root ごとの [`TrackSlot`]/[`DragMetaSlot`] を保持する
    /// レジストリ（モジュール doc「複数 carousel 間の状態分離」節参照）。
    type TrackRegistry = Rc<RefCell<Vec<(Element, TrackSlot, DragMetaSlot)>>>;

    /// `registry` から `carousel_root` に対応する `(TrackSlot,
    /// DragMetaSlot)` を引く。未登録なら新規に確保して登録する
    /// （carousel root の同一性は `Node::is_same_node` で判定する、
    /// モジュール doc「複数 carousel 間の状態分離」節参照）。
    ///
    /// # 切り離された root の遅延回収（codex-review 指摘 是正、イシュー
    /// #2541 第 2 ラウンド）
    ///
    /// `Runtime::apply_subtree_swap` 等で carousel の DOM が置換される
    /// と、旧 root は文書から切断される（`Element::is_connected()` が
    /// 偽になる）が、`TrackRegistry` は強参照で保持し続けるため、置換の
    /// たびにエントリが積み上がり `TrackSlot` の `CarouselTrack`
    /// （settle 中の `AnimationLoop` を含む）ごと解放されずに残ってしまう
    /// （メモリリーク・線形探索の劣化）。是正として、毎回の解決前に
    /// 切断済みエントリを間引く（`crates/wasm-full/src/drag_gesture.rs::
    /// controller_for_tracking_reuse` と同型の遅延掃除、明示的な
    /// unmount フックを持たない設計を踏襲）。間引かれたエントリの
    /// `TrackSlot`（他に参照を持つ呼び出し元がいなければ）が drop
    /// されると `CarouselTrack` も drop され、進行中の `AnimationLoop`
    /// も `Drop` で停止する（`carousel.rs` モジュール doc「`Carousel
    /// Track` を drop すると...」節参照）。
    fn slot_for(registry: &TrackRegistry, carousel_root: &Element) -> (TrackSlot, DragMetaSlot) {
        let mut entries = registry.borrow_mut();
        entries.retain(|(root, _, _)| root.is_connected());
        if let Some((_, track, drag)) = entries
            .iter()
            .find(|(root, _, _)| root.is_same_node(Some(carousel_root.as_ref())))
        {
            return (track.clone(), drag.clone());
        }
        let track: TrackSlot = Rc::new(RefCell::new(None));
        let drag: DragMetaSlot = Rc::new(RefCell::new(None));
        entries.push((carousel_root.clone(), track.clone(), drag.clone()));
        (track, drag)
    }

    /// ドラッグ後の合成 click を抑止するためのフラグ（5px 超の移動が
    /// あった場合のみ次の 1 回の click を capture フェーズで止める）。
    type SuppressClickFlag = Rc<std::cell::Cell<bool>>;

    /// 移動量がこれを超えたら「ドラッグ」とみなし、release 直後の click
    /// を抑止する（トリガー/インジケータの誤発火防止）。
    const CLICK_GUARD_PX: f64 = 5.0;

    /// `root` へポインタイベント 4 種・click（capture）を委譲登録する。
    ///
    /// `on_action` は settle 完了時の `"goto"` dispatch 依頼を呼び出し側
    /// （`crate::lib::Runtime::wire_carousel_motion`）へ渡す
    /// （`angle_slider::wire_angle_slider_events` と同型）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback[_and_bool]` の失敗を伝播する。
    pub fn wire_carousel_motion_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = Rc::new(RefCell::new(on_action));
        let registry: TrackRegistry = Rc::new(RefCell::new(Vec::new()));
        let suppress_click: SuppressClickFlag = Rc::new(std::cell::Cell::new(false));

        let pointerdown_registry = registry.clone();
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&event, &pointerdown_registry);
        });
        root.add_event_listener_with_callback(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
        )?;
        pointerdown_closure.forget();

        let pointermove_registry = registry.clone();
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&event, &pointermove_registry);
        });
        root.add_event_listener_with_callback(
            "pointermove",
            pointermove_closure.as_ref().unchecked_ref(),
        )?;
        pointermove_closure.forget();

        for event_name in ["pointerup", "pointercancel"] {
            let release_registry = registry.clone();
            let release_on_action = on_action.clone();
            let release_suppress = suppress_click.clone();
            let release_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                handle_pointer_release(
                    &event,
                    &release_registry,
                    &release_on_action,
                    &release_suppress,
                );
            });
            root.add_event_listener_with_callback(
                event_name,
                release_closure.as_ref().unchecked_ref(),
            )?;
            release_closure.forget();
        }

        let click_suppress = suppress_click;
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            if click_suppress.replace(false) {
                event.stop_propagation();
                event.prevent_default();
            }
        });
        root.add_event_listener_with_callback_and_bool(
            "click",
            click_closure.as_ref().unchecked_ref(),
            true,
        )?;
        click_closure.forget();

        Ok(())
    }

    /// carousel root（[`CAROUSEL_ROOT_SELECTOR`]）配下・`item-group`
    /// （[`ITEM_GROUP_SELECTOR`]）を解決し、`item` 数・1 スライド分の px を
    /// 計測してドラッグセッションを開始する。opt-in root/`item-group` が
    /// 見つからない・`data-disabled` 祖先を持つ・`item` が 0 件・計測結果
    /// が非正/非有限のいずれかは no-op（fail-safe）。
    fn handle_pointerdown(event: &Event, registry: &TrackRegistry) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        let Ok(Some(carousel_root)) = target.closest(CAROUSEL_ROOT_SELECTOR) else {
            return;
        };
        if carousel_root
            .closest("[data-disabled]")
            .ok()
            .flatten()
            .is_some()
        {
            return;
        }
        let Ok(Some(item_group_el)) = target.closest(ITEM_GROUP_SELECTOR) else {
            return;
        };
        let Ok(item_group) = item_group_el.clone().dyn_into::<HtmlElement>() else {
            return;
        };
        let loop_ = parse_loop_opt_in(carousel_root.get_attribute(CAROUSEL_DRAG_ATTR).as_deref())
            .unwrap_or(false);
        let vertical =
            is_vertical_orientation(item_group.get_attribute("data-orientation").as_deref());

        let Ok(items) = item_group_el.query_selector_all(ITEM_SELECTOR) else {
            return;
        };
        let slide_count = items.length() as usize;
        if slide_count == 0 {
            return;
        }
        let Some(first_item) = items
            .get(0)
            .and_then(|node| node.dyn_into::<HtmlElement>().ok())
        else {
            return;
        };
        // coverflow は item へ rotateY/translateZ 等の transform を適用する
        // （`crates/pre-styled-ui/src/carousel_motion.rs` 参照）ため、
        // `getBoundingClientRect()` は変形後の見かけ上の寸法を返し、現在
        // スライドによって計測値がぶれる（codex-review 指摘 是正）。
        // `offsetWidth`/`offsetHeight` は CSS `transform` の影響を受けない
        // レイアウト寸法（border 込み・margin 抜き）であるため、こちらを
        // 1 スライド分の基準として使う。
        let slide_px = f64::from(if vertical {
            first_item.offset_height()
        } else {
            first_item.offset_width()
        });
        if !slide_px.is_finite() || slide_px <= 0.0 {
            return;
        }

        let (track, drag) = slot_for(registry, &carousel_root);
        // 進行中のドラッグ（別ポインタの pointerdown）がある間は新規
        // pointerdown を無視する（codex-review 指摘 是正）。既存の
        // pointer_id を確認せず無条件に上書きすると、複数指の同時
        // pointerdown で操作中の指から別の指へ黙って乗っ取られてしまう。
        if drag.borrow().is_some() {
            return;
        }

        let _ = item_group.set_pointer_capture(pointer_event.pointer_id());
        let mut new_track = CarouselTrack::attach(item_group.clone(), slide_count, loop_);
        let coord = if vertical {
            f64::from(pointer_event.client_y())
        } else {
            f64::from(pointer_event.client_x())
        };
        new_track.on_pointer_down(coord, event.time_stamp());
        *track.borrow_mut() = Some(new_track);

        let _ = set_dom_attribute(&carousel_root, CAROUSEL_DRAGGING_STATE_ATTR, "");
        *drag.borrow_mut() = Some(DragMeta {
            pointer_id: pointer_event.pointer_id(),
            carousel_root,
            item_group,
            slide_px,
            vertical,
            origin_coord: coord,
            moved: false,
        });
    }

    /// `registry` 内の全 carousel root を横断し、`pointer_id` に一致する
    /// アクティブな [`DragMeta`] を持つ 1 件を探す
    /// （[`TrackSlot`]/[`DragMetaSlot`] の組を返す）。
    ///
    /// # `event.target()` からの `closest()` 解決をやめた理由（Cursor
    /// Bugbot 指摘 是正）
    ///
    /// 当初は `event.target()` を `CAROUSEL_ROOT_SELECTOR` で `closest()`
    /// して carousel root を解決していたが、`set_pointer_capture` が
    /// 失敗した場合やキャプチャが失われた場合（ブラウザ実装・OS 側の
    /// 事情でキャプチャが外れることがある）、`pointerup`/`pointercancel`
    /// は実際にポインタ直下にある要素へ配信されるため、carousel の DOM
    /// 部分木の外（`<body>` 等）で release されると `closest()` が
    /// `None` を返し、settle も `data-fandhe-carousel-dragging` 属性の
    /// 除去も一切実行されないまま状態が残留してしまっていた。
    /// アクティブな [`DragMeta`] は `registry` が pointer_id ごとに保持
    /// しているため、event.target() に頼らず pointer_id で直接引けば
    /// この問題は起きない。
    fn find_active_slot(
        registry: &TrackRegistry,
        pointer_id: i32,
    ) -> Option<(TrackSlot, DragMetaSlot)> {
        let entries = registry.borrow();
        entries.iter().find_map(|(_, track, drag)| {
            let is_match = drag
                .borrow()
                .as_ref()
                .is_some_and(|meta| meta.pointer_id == pointer_id);
            is_match.then(|| (track.clone(), drag.clone()))
        })
    }

    fn handle_pointermove(event: &Event, registry: &TrackRegistry) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some((track, drag)) = find_active_slot(registry, pointer_event.pointer_id()) else {
            return;
        };
        let mut drag_guard = drag.borrow_mut();
        let Some(meta) = drag_guard.as_mut() else {
            return;
        };
        let coord = if meta.vertical {
            f64::from(pointer_event.client_y())
        } else {
            f64::from(pointer_event.client_x())
        };
        if (coord - meta.origin_coord).abs() > CLICK_GUARD_PX {
            meta.moved = true;
        }
        let slide_px = meta.slide_px;
        if let Some(t) = track.borrow_mut().as_mut() {
            t.on_pointer_move(coord, slide_px, event.time_stamp());
        };
    }

    /// pointerup/pointercancel 共通ハンドラ。`drag` の `pointer_id` が
    /// このイベントと一致することを確認してから取り出し settle を
    /// 開始する（settle 完了時に `"goto"` を dispatch する）。
    ///
    /// # pointer_id を確認してから `take()` する理由（codex-review/Cursor
    /// Bugbot 指摘 是正）
    ///
    /// 当初は `drag_guard.take()` を pointer_id 確認より先に実行していた
    /// ため、マルチタッチ環境で無関係な別指の `pointerup`/`pointercancel`
    /// が先に配信されると、アクティブなドラッグの [`DragMeta`] が
    /// pointer_id 不一致にもかかわらず失われてしまい、本来のドラッグ
    /// 指の `pointerup` が届いても settle が開始されない不具合があった。
    /// 是正として `as_ref()` で照合してから一致した場合のみ `take()`
    /// する。
    ///
    /// # pointercancel では合成 click を抑止しない理由（codex-review/
    /// Cursor Bugbot 指摘 是正）
    ///
    /// `pointercancel` はブラウザ仕様上、続く合成 `click` イベントを
    /// 発火**しない**（`pointerup` のみが click の起点になる）。当初は
    /// event 種別を区別せず `moved` に応じて `suppress_click` を常に
    /// 立てていたため、cancel 後は誰も消費しない抑止フラグが残留し、
    /// 全く無関係な次の `click`（別の trigger/indicator クリック等）まで
    /// 誤って止めてしまっていた。是正として `pointerup` のときのみ
    /// `suppress_click` を設定する。
    fn handle_pointer_release(
        event: &Event,
        registry: &TrackRegistry,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        suppress_click: &SuppressClickFlag,
    ) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some((track, drag)) = find_active_slot(registry, pointer_event.pointer_id()) else {
            return;
        };
        let mut drag_guard = drag.borrow_mut();
        let meta = drag_guard
            .take()
            .expect("find_active_slot が Some を確認済み");
        drop(drag_guard);

        let _ = meta
            .item_group
            .release_pointer_capture(pointer_event.pointer_id());
        if event.type_() == "pointerup" {
            suppress_click.set(meta.moved);
        }

        let mut track_guard = track.borrow_mut();
        let Some(t) = track_guard.as_mut() else {
            let _ = meta
                .carousel_root
                .remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
            return;
        };
        let carousel_root = meta.carousel_root.clone();
        let on_action = on_action.clone();
        t.on_release(event.time_stamp(), meta.slide_px, move |index| {
            let _ = carousel_root.remove_attribute(CAROUSEL_DRAGGING_STATE_ATTR);
            (on_action.borrow_mut())(ActionRef {
                action: "goto".to_string(),
                payload: index.to_string(),
            });
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_carousel_motion_events;

#[cfg(test)]
mod tests {
    use super::{is_vertical_orientation, parse_loop_opt_in};

    #[test]
    fn parse_loop_opt_in_reads_loop_literal_only() {
        assert_eq!(parse_loop_opt_in(Some("loop")), Some(true));
        assert_eq!(parse_loop_opt_in(Some("")), Some(false));
        assert_eq!(parse_loop_opt_in(Some("bogus")), Some(false));
        assert_eq!(parse_loop_opt_in(None), None);
    }

    #[test]
    fn is_vertical_orientation_matches_literal_only() {
        assert!(is_vertical_orientation(Some("vertical")));
        assert!(!is_vertical_orientation(Some("horizontal")));
        assert!(!is_vertical_orientation(None));
    }

    #[test]
    fn attr_constants_are_stable_strings() {
        assert_eq!(super::CAROUSEL_DRAG_ATTR, "data-fandhe-carousel-drag");
        assert_eq!(
            super::CAROUSEL_DRAGGING_STATE_ATTR,
            "data-fandhe-carousel-dragging"
        );
    }
}
