//! magnetic pull（ポインタに追従して吸い付く CTA）の DOM 配線層（イシュー
//! #2550、親 #2530/#2476）。
//!
//! # 責務境界
//!
//! ポインタ座標からオフセットを計算する純粋関数
//! （[`fandhe_frontend_animation::magnetic::compute_pull`]）と、CSS
//! カスタムプロパティへの書き込み（[`fandhe_frontend_animation::magnetic::write_offset`]）
//! は `fandhe-frontend-animation` の責務であり、本モジュールは以下のみを
//! 担う（3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）:
//!
//! 1. `root` へのポインタイベント委譲登録（`gesture.rs` と同じ「登録回数を
//!    定数個に抑える」方針、A04 対策）
//! 2. opt-in 要素（[`MAGNETIC_ATTR`]）の解決・`getBoundingClientRect()` に
//!    よる中心座標の計測
//! 3. 現在追従中の要素の追跡（`pointermove` で別要素へ移った際に前の要素の
//!    オフセットをリセットする）
//! 4. `prefers-reduced-motion: reduce` 時は配線自体を行わない（confetti と
//!    同じ「wire 時に検出し、reduced なら配線しない」方針。ポインタ移動の
//!    たびに判定するより安全でコストも低い）
//!
//! # 静止位置の中心のキャッシュ（codex-review P1 指摘の是正、イシュー #2550）
//!
//! `getBoundingClientRect()` は要素へ現在適用中の `transform`
//! （`--fandhe-motion-magnetic-x`/`-y` が駆動する）を含んだ矩形を返す。
//! CSS `transition` が完了する前に `pointermove` が発火すると、矩形は
//! 「補間途中の描画位置」を反映するため、`getComputedStyle` で読んだ
//! 「今の実際の移動量」を引いても静止位置には戻らない場合がある——
//! ただし新しい対象へ**進入した瞬間**であれば、その時点の実際の移動量
//! （[`fandhe_frontend_animation::magnetic::rendered_offset`]、
//! transition 完了前でも常に「今描画されている値」を返す）を 1 回だけ
//! 使って静止位置の中心を計算し、以降の `pointermove` ではこのキャッシュ
//! された中心を再利用する（矩形を再計測しない）ことで、イベントの
//! タイミングに依存しない決定的な計算にできる。resize・スクロールで
//! 静止位置自体が変わる可能性はあるが、追跡セッション（進入から離脱まで）
//! は通常数百 ms 程度であり、その間の resize/スクロールは実用上の
//! 頻度が低いため許容する既知の制約とする（`docs/design/
//! motion-reference-adoption-policy.md` に想定する Motion+ 参照実装も
//! 同様の割り切りを置く）。
//!
//! # `pointerenter`/`pointerleave` を使わない理由
//!
//! `pointerenter`/`pointerleave` はバブルしないため、`root` への委譲登録
//! では受信できない。`gesture.rs` と同じ委譲パターン（`pointermove`/
//! `pointerout` を root で購読し `closest()`/`related_within` で判定）を
//! 採用する。
//!
//! # タッチ除外（`gesture::is_touch_pointer` の再利用）
//!
//! `pointermove`/`pointerout` はいずれもタッチ由来のポインタを除外する
//! （`gesture.rs::handle_pointerover`/`handle_pointerout` と同じ判定）。
//! タッチでは `pointerdown` → `pointermove`（指の物理的な微動を含む）→
//! `pointerup` → `click` の順でイベントが発火するため、除外しない場合
//! タップの最中にボタンがポインタ追従で移動し、`click` が発火する場所が
//! タップ開始位置からずれてしまう（実用上のタップ操作を阻害する）。
//! `crate::gesture` は feature 非依存で常時コンパイルされるため、
//! `magnetic` feature のみでも `is_touch_pointer` を安全に参照できる。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_add_to_basket` の直後で `Self::wire_magnetic` を呼ぶ
//! （feature `magnetic`、既定 on。`lib.rs` クレートドキュメントの対応表は
//! `mount`/`hydrate` の呼び出し順 = `Cargo.toml` の `default` 配列の順を
//! 不変条件とするため、既存配線群の末尾へ追加する）。`dispatch` チャネルを
//! 持たない属性専用配線のため（`Self::wire_gesture`/`Self::wire_confetti`
//! と同型）、`Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`MAGNETIC_ATTR`] は存在属性のみ（値を持たない）で、ポインタ座標・
//! 利用者制御の入力を属性値・セレクタへ混ぜない。DOM への書き込みは
//! `fandhe_frontend_animation::magnetic::write_offset` が固定のプロパティ
//! 名へ `f64` 演算結果のみを書き込む（同モジュール doc 参照）。

/// opt-in（著者が SSR 出力に静的に付与）: ポインタ追従（magnetic pull）を
/// 有効化するボタン等の要素へのマーカー（値なし存在属性）。
pub const MAGNETIC_ATTR: &str = "data-fandhe-magnetic";
/// opt-in 要素の走査セレクタ（`closest()` に渡す）。
pub const MAGNETIC_SELECTOR: &str = "[data-fandhe-magnetic]";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::MAGNETIC_SELECTOR;
    use crate::gesture::is_touch_pointer;
    use fandhe_frontend_animation::magnetic::{compute_pull, rendered_offset, write_offset};
    use fandhe_frontend_animation::magnetic::{MAGNETIC_MAX_PULL_PX, MAGNETIC_STRENGTH};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, MouseEvent, PointerEvent};

    /// 現在追従中の opt-in 要素と、進入時に一度だけ計測した静止位置の中心
    /// （モジュール doc「静止位置の中心のキャッシュ」節参照）。`pointermove`
    /// で別要素へ移った際に前の要素のオフセットをリセットするための追跡
    /// スロットでもある（`gesture.rs` の押下源追跡と同型の考え方）。
    struct ActiveMagneticEntry {
        element: Element,
        /// 進入時に計測した静止位置の中心（`(center_x, center_y)`）。
        rest_center: (f64, f64),
    }

    type ActiveMagnetic = Rc<RefCell<Option<ActiveMagneticEntry>>>;

    /// `magnetic_target` への新規進入時、`getBoundingClientRect()` の中心
    /// から [`rendered_offset`] の値を差し引いて静止位置の中心を計算する
    /// （モジュール doc「静止位置の中心のキャッシュ」節参照）。
    fn measure_rest_center(magnetic_target: &Element, html_element: &HtmlElement) -> (f64, f64) {
        let rect = magnetic_target.get_bounding_client_rect();
        let (offset_x, offset_y) = rendered_offset(html_element);
        (
            rect.left() + rect.width() / 2.0 - offset_x,
            rect.top() + rect.height() / 2.0 - offset_y,
        )
    }

    /// `event.target()` を `Element` として取得する（`gesture.rs`
    /// `event_target_element` と同型）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `event` の `relatedTarget` を `Element` として取得する（`gesture.rs`
    /// `related_target_element` と同型。`PointerEvent` は `MouseEvent` を
    /// 継承するため `MouseEvent` として読む）。
    fn related_target_element(event: &Event) -> Option<Element> {
        let mouse = event.dyn_ref::<MouseEvent>()?;
        mouse.related_target()?.dyn_into::<Element>().ok()
    }

    /// `related`（`relatedTarget`）が `boundary`（含む）配下に留まっている
    /// か（`gesture.rs` `related_within` と同型）。
    fn related_within(event: &Event, boundary: &Element) -> bool {
        related_target_element(event).is_some_and(|related| boundary.contains(Some(&related)))
    }

    /// `root` 配下・[`super::MAGNETIC_SELECTOR`] に一致する最も近い祖先
    /// （自身含む）を返す。
    fn resolve_magnetic_target(root: &Element, target: &Element) -> Option<Element> {
        let matched = target.closest(MAGNETIC_SELECTOR).ok().flatten()?;
        root.contains(Some(&matched)).then_some(matched)
    }

    /// `element` を `(0.0, 0.0)`（中立位置）へリセットする。`HtmlElement`
    /// へのダウンキャストに失敗する場合は何もしない（`write_offset` の
    /// 引数型が `&HtmlElement` のため）。
    fn reset_offset(element: &Element) {
        if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
            write_offset(html_element, 0.0, 0.0);
        }
    }

    /// `pointermove`: `event.target()` から opt-in 対象を解決し、対象が
    /// 変わった場合は前の対象をリセットしてから、新しい対象への進入時に
    /// 一度だけ静止位置の中心を計測してキャッシュする（モジュール doc
    /// 「静止位置の中心のキャッシュ」節）。同じ対象への `pointermove` は
    /// キャッシュした中心を再利用し、`getBoundingClientRect()`/
    /// `rendered_offset` を再計測しない。対象が解決できない（opt-in 要素の
    /// 外）場合は、追従中の要素があればリセットして追跡を解除する。タッチ
    /// 由来のポインタは除外する（モジュール doc「タッチ除外」節）。
    fn handle_pointermove(root: &Element, event: &Event, active: &ActiveMagnetic) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if is_touch_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(magnetic_target) = resolve_magnetic_target(root, &target) else {
            // opt-in 要素の外（対象を持たない座標）へ移動した場合、
            // 追従中の要素が残っていればリセットして追跡を解除する。
            if let Some(previous) = active.borrow_mut().take() {
                reset_offset(&previous.element);
            }
            return;
        };

        let Some(html_element) = magnetic_target.dyn_ref::<HtmlElement>() else {
            return;
        };

        let rest_center = {
            let mut active_ref = active.borrow_mut();
            let is_same_target = active_ref
                .as_ref()
                .is_some_and(|entry| entry.element == magnetic_target);
            if is_same_target {
                // 同じ対象への継続移動: キャッシュした静止位置の中心を
                // そのまま使う（矩形・rendered_offset の再計測はしない）。
                active_ref
                    .as_ref()
                    .expect("checked by is_same_target")
                    .rest_center
            } else {
                // 新しい対象への進入: 前の対象をリセットしてから、この
                // 瞬間の実際の描画位置を基準に静止位置の中心を 1 回だけ
                // 計測してキャッシュする。
                if let Some(previous) = active_ref.take() {
                    reset_offset(&previous.element);
                }
                let rest_center = measure_rest_center(&magnetic_target, html_element);
                *active_ref = Some(ActiveMagneticEntry {
                    element: magnetic_target.clone(),
                    rest_center,
                });
                rest_center
            }
        };

        let (dx, dy) = compute_pull(
            pointer_event.client_x().into(),
            pointer_event.client_y().into(),
            rest_center.0,
            rest_center.1,
            MAGNETIC_MAX_PULL_PX,
            MAGNETIC_STRENGTH,
        );
        write_offset(html_element, dx, dy);
    }

    /// `pointerout`: 追従中の要素からの真の離脱（`related_target` が対象
    /// 外、`gesture.rs::related_within` と同型の判定）で、当該要素の
    /// オフセットをリセットし追跡を解除する。タッチ由来のポインタは除外
    /// する（モジュール doc「タッチ除外」節。除外しない場合、マウスで
    /// 追従中の要素へタッチが重なると無関係なマウス追従までリセットして
    /// しまう——`gesture.rs::handle_pointerout` が同種の理由で行う判定と
    /// 同型）。
    fn handle_pointerout(event: &Event, active: &ActiveMagnetic) {
        if event
            .dyn_ref::<PointerEvent>()
            .is_some_and(|pointer_event| is_touch_pointer(&pointer_event.pointer_type()))
        {
            return;
        }
        let current_element = active.borrow().as_ref().map(|entry| entry.element.clone());
        let Some(current_element) = current_element else {
            return;
        };
        if !related_within(event, &current_element) {
            reset_offset(&current_element);
            *active.borrow_mut() = None;
        }
    }

    /// `root` へ magnetic pull のポインタイベント委譲を登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`reduced_motion` が `true`
    /// （`fandhe_frontend_animation::magnetic::detect_reduced_motion()`
    /// の結果、または呼び出し側の明示注入）なら何も登録せず返す
    /// （モジュール doc「`prefers-reduced-motion: reduce` 時は配線自体を
    /// 行わない」節）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_magnetic_with_reduced_motion(
        root: Element,
        reduced_motion: bool,
    ) -> Result<(), JsValue> {
        if reduced_motion {
            return Ok(());
        }

        let active: ActiveMagnetic = Rc::new(RefCell::new(None));

        let pointermove_root = root.clone();
        let pointermove_active = Rc::clone(&active);
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&pointermove_root, &event, &pointermove_active);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointermove",
            pointermove_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointermove_closure.forget();

        let pointerout_active = Rc::clone(&active);
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerout(&event, &pointerout_active);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerout_closure.forget();

        Ok(())
    }

    /// [`wire_magnetic_with_reduced_motion`] を実際の
    /// `prefers-reduced-motion` 検出結果で呼ぶ（`Runtime::mount`/
    /// `hydrate` から呼ばれる本番経路）。
    ///
    /// # Errors
    ///
    /// [`wire_magnetic_with_reduced_motion`] の失敗を伝播する。
    pub fn wire_magnetic(root: Element) -> Result<(), JsValue> {
        let reduced_motion = fandhe_frontend_animation::magnetic::detect_reduced_motion();
        wire_magnetic_with_reduced_motion(root, reduced_motion)
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_magnetic, wire_magnetic_with_reduced_motion};

#[cfg(test)]
mod tests {
    use super::{MAGNETIC_ATTR, MAGNETIC_SELECTOR};

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(MAGNETIC_ATTR, "data-fandhe-magnetic");
        assert_eq!(MAGNETIC_SELECTOR, "[data-fandhe-magnetic]");
    }
}
