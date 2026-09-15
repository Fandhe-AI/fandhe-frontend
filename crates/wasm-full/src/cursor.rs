//! カスタムカーソル（cursor 部品、Motion+ Cursor 相当）の DOM 配線層
//! （イシュー #2542、親 #2530/#2476）。
//!
//! # 責務境界
//!
//! ポインタ追従位置の spring 演算・rAF 駆動・DOM 書き込み
//! （[`fandhe_frontend_animation::cursor::CursorAnimator`]）は
//! `fandhe-frontend-animation` の責務であり、本モジュールは以下のみを
//! 担う（3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）:
//!
//! 1. `root` へのポインタイベント委譲登録（`magnetic.rs`/`gesture.rs` と
//!    同じ「登録回数を定数個に抑える」方針、A04 対策）
//! 2. カーソル要素本体（[`CURSOR_SELECTOR`]）・hover 対象
//!    （[`CURSOR_TARGET_SELECTOR`]）の解決
//! 3. hover 対象の `data-*` 値（バリアント・ラベル・magnetic opt-in）の
//!    カーソル要素への写し（[`CURSOR_VARIANT_ATTR`]/[`CURSOR_LABEL_ATTR`]）
//! 4. `prefers-reduced-motion: reduce`・`pointer: coarse` のいずれかが
//!    真なら配線自体を行わない（`magnetic.rs` と同じ「wire 時に検出し、
//!    該当すれば配線しない」方針）
//!
//! # 無効化条件が二重である理由
//!
//! タッチ端末は一般に `pointer: coarse` を報告するため
//! `matchMedia("(pointer: coarse)")` だけでも大半のタッチ端末を除外
//! できるが、タッチ対応の 2-in-1 デバイス等では `pointer: coarse` と
//! `hover`/マウスが共存しうる。[`pointermove`]/[`pointerout`] ハンドラも
//! `crate::gesture::is_touch_pointer` でタッチ由来のポインタイベントを
//! 個別に除外する（`magnetic.rs` と同じ多重防御）。加えて
//! [`crate::pre_styled_ui_cursor_css`]（`crates/pre-styled-ui/src/
//! cursor.rs` の `CURSOR_CSS`）側にも独立の `@media` フェイルセーフを
//! 持たせる（Issue #2542 受け入れ条件「JS 側と CSS 側の二重のフェイル
//! セーフ」）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_text_animation` の直後で `Self::wire_cursor` を呼ぶ
//! （feature `cursor`、既定 on。`lib.rs` クレートドキュメントの対応表は
//! `mount`/`hydrate` の呼び出し順 = `Cargo.toml` の `default` 配列の順を
//! 不変条件とするため、既存配線群の末尾へ追加する）。`dispatch` チャネル
//! を持たない属性専用配線のため（`Self::wire_magnetic`/
//! `Self::wire_gesture` と同型）、`Component`/`binding_table`/
//! `keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! カーソル要素へ写す値（[`CURSOR_VARIANT_ATTR`]/[`CURSOR_LABEL_ATTR`]）
//! は hover 対象が SSR 出力に静的に持つ `data-*` 属性値の写しであり、
//! `set_attribute`/`remove_attribute` のみで完結する（HTML 文字列・
//! セレクタ文字列の組み立てには使わない）。位置の DOM 書き込みは
//! `fandhe_frontend_animation::cursor::write_position` が固定のプロパティ
//! 名へ `f64` 演算結果のみを書き込む（同モジュール doc 参照）。
//!
//! # DOM 再描画後の再同期（イシュー #2583 レビュー指摘の是正）
//!
//! `dispatch` チャネルを持たない属性専用配線（doc 冒頭「Runtime への
//! 統合」節）のため、`Runtime::rerender`/構造フォールバック再描画
//! （`Runtime::apply_subtree_swap`、`root` 自身は差し替えず子だけを
//! 丸ごと入れ替える）が起きても本モジュールへは一切通知されない。
//! `wire_cursor_with_env` 配線時に一度だけ解決した [`CURSOR_SELECTOR`]
//! 要素・`CursorAnimator` をクロージャへ固定保持すると、再描画後は
//! イベントハンドラが DOM から切り離された旧要素を更新し続け、新しい
//! カーソル要素には状態属性が一切付かないため CSS 既定の
//! `display: none` のまま表示されない（`root` の
//! [`CURSOR_ACTIVE_ATTR`] は残留するためネイティブカーソルも隠れたまま
//! になる）。`sidebar.rs::wire_mobile` と同型の `MutationObserver`
//! （`childList: true, subtree: true`、`root` へ 1 個だけ登録）で
//! `wiring::resync_cursor_element` を呼び、カーソル要素・`CursorAnimator`
//! を都度再解決する。要素が見つからなくなった場合は
//! [`CURSOR_ACTIVE_ATTR`] を外してネイティブカーソルを復元する。

/// opt-in（著者が SSR 出力に静的に付与）: カスタムカーソル要素本体への
/// マーカー（値なし存在属性）。`root` 配下に 1 個を想定する。
pub const CURSOR_ATTR: &str = "data-fandhe-cursor";
/// [`CURSOR_ATTR`] 要素の走査セレクタ。
pub const CURSOR_SELECTOR: &str = "[data-fandhe-cursor]";
/// opt-in（著者が SSR 出力に静的に付与）: hover 対象へのマーカー。値は
/// バリアント名（例 `"ring"`、空文字可）。
pub const CURSOR_TARGET_ATTR: &str = "data-fandhe-cursor-target";
/// [`CURSOR_TARGET_ATTR`] 要素の走査セレクタ（`closest()` に渡す）。
pub const CURSOR_TARGET_SELECTOR: &str = "[data-fandhe-cursor-target]";
/// opt-in（著者が SSR 出力に静的に付与）: hover 中にカーソルへ表示する
/// 短いラベルテキスト。
pub const CURSOR_TARGET_LABEL_ATTR: &str = "data-fandhe-cursor-target-label";
/// opt-in（著者が SSR 出力に静的に付与）: hover 中はポインタ座標では
/// なく対象の中心へ吸着する（値なし存在属性）。
pub const CURSOR_TARGET_MAGNETIC_ATTR: &str = "data-fandhe-cursor-target-magnetic";
/// 状態（wasm-full が root へ 1 回だけ付与、値なし存在属性）: 配線が
/// 成立した（ネイティブカーソルを隠す CSS のフック）。
pub const CURSOR_ACTIVE_ATTR: &str = "data-fandhe-cursor-active";
/// 状態（カーソル要素へ動的に付け外す）: `"hidden"`/`"idle"`/`"hover"`。
pub const CURSOR_STATE_ATTR: &str = "data-fandhe-cursor-state";
/// 状態（カーソル要素へ動的に付け外す）: hover 対象の [`CURSOR_TARGET_ATTR`]
/// 値の写し。
pub const CURSOR_VARIANT_ATTR: &str = "data-fandhe-cursor-variant";
/// 状態（カーソル要素へ動的に付け外す）: hover 対象の
/// [`CURSOR_TARGET_LABEL_ATTR`] 値の写し。
pub const CURSOR_LABEL_ATTR: &str = "data-fandhe-cursor-label";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        CURSOR_LABEL_ATTR, CURSOR_SELECTOR, CURSOR_STATE_ATTR, CURSOR_TARGET_LABEL_ATTR,
        CURSOR_TARGET_MAGNETIC_ATTR, CURSOR_TARGET_SELECTOR, CURSOR_VARIANT_ATTR,
    };
    use crate::gesture::is_touch_pointer;
    use fandhe_frontend_animation::cursor::CursorAnimator;
    use fandhe_frontend_animation::fandhe_animation::spring::SpringConfig;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, HtmlElement, MouseEvent, MutationObserver, MutationObserverInit,
        PointerEvent,
    };

    /// 現在 hover 中の対象（`pointermove` で別要素へ移った際の `data-*`
    /// 書き換え抑制・`pointerout` での解除に使う追跡スロット、
    /// `magnetic.rs::ActiveMagnetic` と同型の考え方）。
    type ActiveTarget = Rc<RefCell<Option<Element>>>;

    /// 現在配線中のカーソル要素・`CursorAnimator` の組（DOM 再描画で
    /// 入れ替わり得る可変状態、モジュール doc「DOM 再描画後の再同期」
    /// 節参照）。
    struct CursorState {
        cursor_el: HtmlElement,
        animator: Rc<RefCell<CursorAnimator>>,
    }

    /// [`CursorState`] を保持するスロット。カーソル要素が見つからない間は
    /// `None`（イベントハンドラは no-op）。
    type CursorStateSlot = Rc<RefCell<Option<CursorState>>>;

    /// 直近の `pointermove` で書き込んだカーソル座標（`root` 配下にいる間
    /// のみ `Some`、`pointerout` での真の離脱時に `None` へ戻す）。DOM
    /// 再描画でカーソル要素が作り直された際、この座標を新要素へ即座に
    /// 書き戻すことで「次の `pointermove` まで両カーソルが消えたまま」
    /// になる不具合を防ぐ（PR #2583 レビュー指摘 P1-1 の是正）。
    type LastPointerSlot = Rc<RefCell<Option<(f64, f64)>>>;

    /// `event.target()` を `Element` として取得する（`gesture.rs`/
    /// `magnetic.rs` の同名関数と同型）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `event` の `relatedTarget` を `Element` として取得する。
    fn related_target_element(event: &Event) -> Option<Element> {
        let mouse = event.dyn_ref::<MouseEvent>()?;
        mouse.related_target()?.dyn_into::<Element>().ok()
    }

    /// `event` の `clientX`/`clientY` が `boundary` の矩形内にあるか
    /// （`PointerEvent` でない場合は判定不能として `false`）。
    fn pointer_within_rect(event: &Event, boundary: &Element) -> bool {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return false;
        };
        let rect = boundary.get_bounding_client_rect();
        let x = f64::from(pointer_event.client_x());
        let y = f64::from(pointer_event.client_y());
        x >= rect.left() && x <= rect.right() && y >= rect.top() && y <= rect.bottom()
    }

    /// `root` 配下・[`CURSOR_TARGET_SELECTOR`] に一致する最も近い祖先
    /// （自身含む）を返す。
    fn resolve_cursor_target(root: &Element, target: &Element) -> Option<Element> {
        let matched = target.closest(CURSOR_TARGET_SELECTOR).ok().flatten()?;
        root.contains(Some(&matched)).then_some(matched)
    }

    /// `cursor_el` の `data-*` を `hidden`/`variant`/`label` なしへ戻す。
    fn clear_hover_state(cursor_el: &HtmlElement) {
        crate::dom::set_dom_attribute(cursor_el, CURSOR_STATE_ATTR, "idle");
        let _ = cursor_el.remove_attribute(CURSOR_VARIANT_ATTR);
        let _ = cursor_el.remove_attribute(CURSOR_LABEL_ATTR);
    }

    /// `pointermove`: hover 対象を解決し、対象が変わった場合のみ
    /// カーソル要素の `data-*` を書き換える（変わらない限り書き換えを
    /// 抑制する、`magnetic.rs::handle_pointermove` と同型）。magnetic
    /// opt-in（[`CURSOR_TARGET_MAGNETIC_ATTR`]）を持つ対象へは
    /// `getBoundingClientRect()` の中心へ、それ以外はポインタ座標へ
    /// 追従させる。タッチ由来のポインタは除外する。
    ///
    /// `hover_target` が対象なし（`None`）で前回と「変わらない」
    /// （`is_same`）場合でも、現在の `data-*` 状態が `"hidden"`
    /// （`root` へ初めて入った直後・`pointerout` で離脱した後の再入場）
    /// であれば `"idle"` へ強制遷移させる（`is_same` は variant/label の
    /// 書き換え抑制のみを意図しており、hidden→idle の遷移自体は
    /// hover 対象の同一性とは独立に必要なため。イシュー #2542 レビュー
    /// 指摘: 初回移動・再入場時にカーソルが表示されない不具合の修正）。
    ///
    /// `root` に [`super::CURSOR_ACTIVE_ATTR`] が未付与（初回 mount/hydrate
    /// で有効な座標がまだ無かった場合、`resync_cursor_element` が付与を
    /// 見送っている）ならここで付与する。ネイティブカーソルを隠す
    /// `!important` の CSS フックは、有効な座標が判明しカスタムカーソルが
    /// 表示可能になった瞬間にのみ有効化する（PR #2583 レビュー指摘 P1 の
    /// 是正: 座標未確定のまま `CURSOR_ACTIVE_ATTR` を付与すると、ネイティブ
    /// カーソルだけが即座に隠れカスタムカーソルは `hidden` のままという
    /// 「両方消える」窓が生じていた）。
    fn handle_pointermove(
        root: &Element,
        state: &CursorStateSlot,
        active: &ActiveTarget,
        last_pointer: &LastPointerSlot,
        event: &Event,
    ) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if is_touch_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let state_ref = state.borrow();
        let Some(CursorState {
            cursor_el,
            animator,
        }) = state_ref.as_ref()
        else {
            return;
        };
        if !root.has_attribute(super::CURSOR_ACTIVE_ATTR) {
            crate::dom::set_dom_attribute(root, super::CURSOR_ACTIVE_ATTR, "");
        }

        let hover_target =
            event_target_element(event).and_then(|target| resolve_cursor_target(root, &target));

        {
            let mut active_ref = active.borrow_mut();
            let is_same = match (active_ref.as_ref(), hover_target.as_ref()) {
                (Some(previous), Some(next)) => previous == next,
                (None, None) => true,
                _ => false,
            };
            let was_hidden =
                cursor_el.get_attribute(CURSOR_STATE_ATTR).as_deref() == Some("hidden");
            if !is_same {
                *active_ref = hover_target.clone();
                match &hover_target {
                    Some(target) => {
                        let variant = target
                            .get_attribute(super::CURSOR_TARGET_ATTR)
                            .unwrap_or_default();
                        let label = target
                            .get_attribute(CURSOR_TARGET_LABEL_ATTR)
                            .unwrap_or_default();
                        crate::dom::set_dom_attribute(cursor_el, CURSOR_VARIANT_ATTR, &variant);
                        crate::dom::set_dom_attribute(cursor_el, CURSOR_LABEL_ATTR, &label);
                        crate::dom::set_dom_attribute(cursor_el, CURSOR_STATE_ATTR, "hover");
                    }
                    None => clear_hover_state(cursor_el),
                }
            } else if was_hidden && hover_target.is_none() {
                crate::dom::set_dom_attribute(cursor_el, CURSOR_STATE_ATTR, "idle");
            }
        }

        let (x, y) = match &hover_target {
            Some(target) if target.has_attribute(CURSOR_TARGET_MAGNETIC_ATTR) => {
                let rect = target.get_bounding_client_rect();
                (
                    rect.left() + rect.width() / 2.0,
                    rect.top() + rect.height() / 2.0,
                )
            }
            _ => (
                f64::from(pointer_event.client_x()),
                f64::from(pointer_event.client_y()),
            ),
        };
        animator.borrow_mut().move_to(x, y);
        *last_pointer.borrow_mut() = Some((x, y));
    }

    /// `pointerout`: `root` からの真の離脱で追跡を解除する。タッチ由来の
    /// ポインタは除外する。
    ///
    /// 離脱判定は原則 `relatedTarget` が `root` 配下に留まっているかで
    /// 行うが、`relatedTarget` が取得できない場合
    /// （`Event::related_target()` が `None`）は座標フォールバック
    /// （[`pointer_within_rect`]）で判定する。DOM 再描画で hover 対象
    /// ノードが除去された瞬間、ブラウザは `relatedTarget` を確定できない
    /// まま `pointerout` を発火させることがあり、これを無条件に「真の
    /// 離脱」と扱うと、ポインタが `root` 内に留まっているのに
    /// `last_pointer` がクリアされて再描画後のカーソル復元
    /// （`resync_cursor_element`）が「位置不明」側へ倒れ、次の
    /// `pointermove` まで両カーソルとも消えたままになる（Cursor Bugbot
    /// 指摘、PR #2583 レビュー）。
    fn handle_pointerout(
        root: &Element,
        state: &CursorStateSlot,
        active: &ActiveTarget,
        last_pointer: &LastPointerSlot,
        event: &Event,
    ) {
        if event
            .dyn_ref::<PointerEvent>()
            .is_some_and(|pointer_event| is_touch_pointer(&pointer_event.pointer_type()))
        {
            return;
        }
        let left_root = match related_target_element(event) {
            Some(related) => !root.contains(Some(&related)),
            None => !pointer_within_rect(event, root),
        };
        if left_root {
            let state_ref = state.borrow();
            if let Some(CursorState { cursor_el, .. }) = state_ref.as_ref() {
                crate::dom::set_dom_attribute(cursor_el, CURSOR_STATE_ATTR, "hidden");
                let _ = cursor_el.remove_attribute(CURSOR_VARIANT_ATTR);
                let _ = cursor_el.remove_attribute(CURSOR_LABEL_ATTR);
            }
            // カーソルを隠す際は root の CURSOR_ACTIVE_ATTR も外し、ネイティブ
            // カーソルを復元する（PR #2583 レビュー指摘の是正）。これを怠ると
            // `cursor: none !important` の CSS フックが残留したまま両カーソル
            // とも消えた状態になる。有効な座標を再取得した時点
            // （`handle_pointermove`）で再付与する。
            let _ = root.remove_attribute(super::CURSOR_ACTIVE_ATTR);
            *active.borrow_mut() = None;
            *last_pointer.borrow_mut() = None;
        }
    }

    /// `root` 配下から現在の [`super::CURSOR_SELECTOR`] 要素を再解決し、
    /// `state` が保持する要素と異なる場合（初回配線、または DOM
    /// 再描画で要素が作り直された場合）[`CursorState`] を作り直す
    /// （モジュール doc「DOM 再描画後の再同期」節）。要素が見つからなく
    /// なった場合は [`super::CURSOR_ACTIVE_ATTR`] を外してネイティブ
    /// カーソルを復元し、`state`/`active` を `None` へ戻す。
    ///
    /// `last_pointer` が `Some((x, y))`（＝再描画の瞬間もポインタが
    /// `root` 配下に留まっていた）場合、新しい要素・`CursorAnimator` を
    /// その座標で即座に可視化し、[`super::CURSOR_ACTIVE_ATTR`] も同時に
    /// 付与してネイティブカーソルを隠す（`CursorAnimator::move_to` の
    /// 初回呼び出しは spring を経由せず直接スナップするため、次の
    /// `pointermove` を待たずに正しい位置へ表示できる）。`None`
    /// （ポインタが `root` の外にいる／位置が未知）の場合は
    /// [`super::CURSOR_ACTIVE_ATTR`] を付与しない（＝ネイティブカーソルを
    /// 隠さない）まま `hidden` で次の `pointermove` を待つ
    /// （`handle_pointermove` が有効な座標を得た時点で付与する。PR #2583
    /// レビュー指摘 P1 の是正: 座標未確定のまま無条件付与すると、ネイティブ
    /// カーソルだけが先に隠れカスタムカーソルは `hidden` のままという
    /// 「両方消える」窓が生じていた）。
    fn resync_cursor_element(
        root: &Element,
        state: &CursorStateSlot,
        active: &ActiveTarget,
        last_pointer: &LastPointerSlot,
    ) {
        let resolved = root
            .query_selector(CURSOR_SELECTOR)
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok());

        let mut state_ref = state.borrow_mut();
        let unchanged = match (state_ref.as_ref(), &resolved) {
            (Some(current), Some(next)) => &current.cursor_el == next,
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }

        match resolved {
            Some(cursor_el) => {
                let restore_position = *last_pointer.borrow();
                if restore_position.is_some() {
                    crate::dom::set_dom_attribute(root, super::CURSOR_ACTIVE_ATTR, "");
                } else {
                    // 位置未知（root 外／真の離脱後）のまま再同期する場合、
                    // 旧要素から引き継いだ CURSOR_ACTIVE_ATTR を残さない
                    // （PR #2583 レビュー指摘の是正: この分岐が active 属性を
                    // 除去しないと、ネイティブカーソルが隠れたまま新しい
                    // pointermove まで両カーソルとも表示されない）。
                    let _ = root.remove_attribute(super::CURSOR_ACTIVE_ATTR);
                }
                let initial_state = if restore_position.is_some() {
                    "idle"
                } else {
                    "hidden"
                };
                crate::dom::set_dom_attribute(&cursor_el, CURSOR_STATE_ATTR, initial_state);
                let mut animator =
                    CursorAnimator::new(cursor_el.clone(), SpringConfig::default(), false);
                if let Some((x, y)) = restore_position {
                    animator.move_to(x, y);
                }
                *state_ref = Some(CursorState {
                    cursor_el,
                    animator: Rc::new(RefCell::new(animator)),
                });
            }
            None => {
                let _ = root.remove_attribute(super::CURSOR_ACTIVE_ATTR);
                *state_ref = None;
            }
        }
        *active.borrow_mut() = None;
    }

    /// `root` へカスタムカーソルのポインタイベント委譲を登録する
    /// （テスト注入用。`reduced_motion`/`coarse_pointer` のいずれかが
    /// `true` なら何も登録せず、root へ [`super::CURSOR_ACTIVE_ATTR`] も
    /// 付けない——モジュール doc「無効化条件」節）。[`CURSOR_SELECTOR`]
    /// 要素が `root` 配下に無い場合も no-op。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_cursor_with_env(
        root: Element,
        reduced_motion: bool,
        coarse_pointer: bool,
    ) -> Result<(), JsValue> {
        if reduced_motion || coarse_pointer {
            return Ok(());
        }
        if root.query_selector(CURSOR_SELECTOR)?.is_none() {
            return Ok(());
        }

        let state: CursorStateSlot = Rc::new(RefCell::new(None));
        let active: ActiveTarget = Rc::new(RefCell::new(None));
        let last_pointer: LastPointerSlot = Rc::new(RefCell::new(None));
        // 初回解決（`resync_cursor_element` を流用し、以降の再描画時と
        // 同じ経路で `CursorState`・`CURSOR_ACTIVE_ATTR` を初期化する）。
        resync_cursor_element(&root, &state, &active, &last_pointer);

        let pointermove_root = root.clone();
        let pointermove_state = Rc::clone(&state);
        let pointermove_active = Rc::clone(&active);
        let pointermove_last_pointer = Rc::clone(&last_pointer);
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(
                &pointermove_root,
                &pointermove_state,
                &pointermove_active,
                &pointermove_last_pointer,
                &event,
            );
        });
        root.add_event_listener_with_callback_and_bool(
            "pointermove",
            pointermove_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointermove_closure.forget();

        let pointerout_root = root.clone();
        let pointerout_state = Rc::clone(&state);
        let pointerout_active = Rc::clone(&active);
        let pointerout_last_pointer = Rc::clone(&last_pointer);
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerout(
                &pointerout_root,
                &pointerout_state,
                &pointerout_active,
                &pointerout_last_pointer,
                &event,
            );
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerout_closure.forget();

        // `MutationObserver`: `Runtime::rerender`/構造フォールバック再描画
        // （`root` 自身は差し替えず子だけを入れ替える）でカーソル要素が
        // 作り直されるのを検知し、`CursorState` を再解決する
        // （`sidebar.rs::wire_mobile` と同型、モジュール doc「DOM 再描画後の
        // 再同期」節）。`childList`/`subtree` のみ監視するため、本モジュール
        // 自身が書き込む `data-*` 属性変更では再発火しない。
        let observer_root = root.clone();
        let observer_state = Rc::clone(&state);
        let observer_active = Rc::clone(&active);
        let observer_last_pointer = Rc::clone(&last_pointer);
        let observer_callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                if !observer_root.is_connected() {
                    return;
                }
                resync_cursor_element(
                    &observer_root,
                    &observer_state,
                    &observer_active,
                    &observer_last_pointer,
                );
            },
        );
        let observer = MutationObserver::new(observer_callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(&root, &init)?;
        observer_callback.forget();

        Ok(())
    }

    /// [`wire_cursor_with_env`] を実際の `prefers-reduced-motion`/
    /// `pointer: coarse` 検出結果で呼ぶ（`Runtime::mount`/`hydrate` から
    /// 呼ばれる本番経路）。`matchMedia` 自体の呼び出し失敗は抑制側
    /// （`true`）へ fail-closed する（`magnetic.rs::detect_reduced_motion`
    /// と同じ方針、security.md A05）。
    ///
    /// # Errors
    ///
    /// [`wire_cursor_with_env`] の失敗を伝播する。
    pub fn wire_cursor(root: Element) -> Result<(), JsValue> {
        let reduced_motion = fandhe_frontend_animation::reduced_motion::prefers_reduced_motion();
        let coarse_pointer = web_sys::window()
            .and_then(|window| window.match_media("(pointer: coarse)").ok().flatten())
            .map(|list| list.matches())
            .unwrap_or(true);
        wire_cursor_with_env(root, reduced_motion, coarse_pointer)
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_cursor, wire_cursor_with_env};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(CURSOR_ATTR, "data-fandhe-cursor");
        assert_eq!(CURSOR_SELECTOR, "[data-fandhe-cursor]");
        assert_eq!(CURSOR_TARGET_ATTR, "data-fandhe-cursor-target");
        assert_eq!(CURSOR_TARGET_SELECTOR, "[data-fandhe-cursor-target]");
        assert_eq!(CURSOR_TARGET_LABEL_ATTR, "data-fandhe-cursor-target-label");
        assert_eq!(
            CURSOR_TARGET_MAGNETIC_ATTR,
            "data-fandhe-cursor-target-magnetic"
        );
        assert_eq!(CURSOR_ACTIVE_ATTR, "data-fandhe-cursor-active");
        assert_eq!(CURSOR_STATE_ATTR, "data-fandhe-cursor-state");
        assert_eq!(CURSOR_VARIANT_ATTR, "data-fandhe-cursor-variant");
        assert_eq!(CURSOR_LABEL_ATTR, "data-fandhe-cursor-label");
    }
}
