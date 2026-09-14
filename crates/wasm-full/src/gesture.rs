//! hover / press ジェスチャー配線（イシュー #2520、親 #2508/#2491）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 は motion.dev の
//! `hover()`/`press()` を「A 群: 既存 CSS `:hover`/`:active`/
//! `:focus-visible`（`docs/design/pre-styled-ui-interaction-visual-language.md`）
//! で足りる」に大半を分類する。本モジュールはその外側にある、CSS 疑似
//! クラスだけでは表現できない 2 点の残余ギャップのみを埋める:
//!
//! 1. タッチ端末で `:hover` がタップ後に貼り付く疑似 hover を除去した
//!    「非タッチ限定 hover」の判定を `data-*` として書き戻す。
//! 2. `role="button"` 等のカスタムインタラクティブ要素は keyboard
//!    （Enter/Space）活性化で `:active` が付かないため、press 視覚状態を
//!    `data-*` として供給する。
//!
//! `IntersectionObserver`/Pointer Events の購読と `data-*` 書き戻しのみで
//! 完結する群 B 相当であり、`fandhe_animation`/`fandhe_frontend_animation`
//! の `Driver`/`Target` 連携は不要（3 層構成、#2491）。`events.rs`/
//! `keynav.rs`/`sidebar.rs` と同じ 2 層構成（DOM 非依存の純粋ロジック層 +
//! `#[cfg(target_arch = "wasm32")]` 配線層）を踏襲する。
//!
//! # `data-*` 属性の命名（既存語彙との意図的な非採用の踏襲）
//!
//! `data-hover`/`data-active`/`data-state="hover"` は採用しない。
//! `crates/headless-ui/`（`switch.rs` 他）は ark-ui 由来の `data-hover`/
//! `data-active` を「CSS 疑似クラスで足りる」として意図的に不採用にした
//! 設計判断を明記済みであり、ここで同名を新規採用すると読み手を混乱
//! させる。`data-active`（chart 系ハイライト等）・`data-pressed`
//! （Toggle の永続選択状態）・`data-state`（各部品の状態機械）はいずれも
//! 既存の別意味論で広く使用済みのため避け、内部実装用の
//! `data-fandhe-*` プレフィックス（`stagger_index.rs`/`content_height.rs`
//! と同型）を用いる。opt-in を hover/press で分離するのは、コストが
//! 異なる（hover は 2 リスナー共有、press は 5 リスナー共有）ため、
//! 利用者が必要な方だけ opt-in できるようにするため。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_in_view` の直後で `wire_gesture` を呼ぶ（feature
//! `gesture`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない
//! （`Self::wire_sidebar`/`Self::wire_in_view` と同型）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! DOM へ書き込む値は [`HOVER_STATE_ATTR`]/[`PRESS_STATE_ATTR`] いずれも
//! 常に空文字列 `""` の存在属性のみであり、ポインタ座標・キー文字列・
//! 利用者制御の入力を属性値・セレクタへ混ぜない。

/// opt-in（著者が SSR 出力に静的に付与）: hover 検知を有効化する。
pub const GESTURE_HOVER_ATTR: &str = "data-fandhe-gesture-hover";
/// opt-in（著者が SSR 出力に静的に付与）: press 検知を有効化する。
pub const GESTURE_PRESS_ATTR: &str = "data-fandhe-gesture-press";
/// 状態（wasm-full が動的に付け外す、存在で真を表す）: 非タッチ hover 中。
pub const HOVER_STATE_ATTR: &str = "data-fandhe-hover";
/// 状態（同上）: press 中（pointer または keyboard 活性化）。
pub const PRESS_STATE_ATTR: &str = "data-fandhe-press";

/// `PointerEvent::pointer_type()` がタッチ由来か（タッチ疑似 hover 除去用）。
///
/// `crate::chart::is_sticky_pointer` と同一ロジックだが意味的文脈が異なる
/// （sticky tooltip 判定 vs pseudo-hover 除去）ため独立定義する。
#[must_use]
pub fn is_touch_pointer(pointer_type: &str) -> bool {
    pointer_type == "touch"
}

/// `KeyboardEvent::key()` が press 活性化キー（Enter/Space）か。
///
/// `keynav.rs` の活性化キー判定と同じ 2 語のみを見る（"Spacebar" 等の
/// レガシー別名は既存コードに前例がないため追加しない）。
#[must_use]
pub fn is_press_activation_key(key: &str) -> bool {
    key == "Enter" || key == " "
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        is_press_activation_key, is_touch_pointer, GESTURE_HOVER_ATTR, GESTURE_PRESS_ATTR,
        HOVER_STATE_ATTR, PRESS_STATE_ATTR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, KeyboardEvent, MouseEvent, PointerEvent};

    /// `event.target()` を `Element` として取得する（`Text` ノード等は
    /// `None`）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `related`（`pointerover`/`pointerout` の `relatedTarget`）が
    /// `boundary`（含む）配下に留まっているか。子要素間の内部移動を
    /// 「離脱」と誤判定しないための境界判定（`sidebar::wiring` の
    /// `pointerover`/`pointerout` 判定と同型）。
    fn related_within(event: &Event, boundary: &Element) -> bool {
        event
            .dyn_ref::<MouseEvent>()
            .and_then(web_sys::MouseEvent::related_target)
            .and_then(|target| target.dyn_into::<Element>().ok())
            .is_some_and(|related| boundary.contains(Some(&related)))
    }

    /// `root` 配下・`opt_in_attr` を持つ最も近い祖先（自身含む）を返す。
    fn closest_opted_in(root: &Element, target: &Element, opt_in_attr: &str) -> Option<Element> {
        let selector = format!("[{opt_in_attr}]");
        let matched = target.closest(&selector).ok().flatten()?;
        root.contains(Some(&matched)).then_some(matched)
    }

    /// `pointerover`: 非タッチかつ真の進入（`related_target` が
    /// `hover_target` 外）のときのみ [`HOVER_STATE_ATTR`] を付与する。
    fn handle_pointerover(root: &Element, event: &Event) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if is_touch_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(hover_target) = closest_opted_in(root, &target, GESTURE_HOVER_ATTR) else {
            return;
        };
        if related_within(event, &hover_target) {
            return;
        }
        let _ = set_dom_attribute(&hover_target, HOVER_STATE_ATTR, "");
    }

    /// `pointerout`: 真の離脱（`related_target` が対象外）で
    /// [`HOVER_STATE_ATTR`] を外す。press 中に要素外へ抜けた場合の
    /// [`PRESS_STATE_ATTR`] 解除（ネイティブ `:active` が要素外離脱で
    /// 解除される挙動の再現）も同じハンドラで併せて行う（冪等な
    /// remove のため press 状態でなくても無害）。
    fn handle_pointerout(root: &Element, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        if let Some(hover_target) = closest_opted_in(root, &target, GESTURE_HOVER_ATTR) {
            if !related_within(event, &hover_target) {
                let _ = hover_target.remove_attribute(HOVER_STATE_ATTR);
            }
        }
        if let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) {
            if !related_within(event, &press_target) {
                let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
            }
        }
    }

    /// `pointerdown`: opt-in 要素へ [`PRESS_STATE_ATTR`] を付与する
    /// （pointer/touch 両方対象、タッチ除外は行わない）。
    fn handle_pointerdown(root: &Element, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
    }

    /// `pointerup`/`pointercancel`: opt-in 要素から [`PRESS_STATE_ATTR`]
    /// を外す（同一 [`Closure`] を両イベント名で登録しリスナー数を節約）。
    fn handle_pointerup_or_cancel(root: &Element, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
    }

    /// `keydown`: 活性化キー（Enter/Space）・非リピート・opt-in 要素の
    /// ときのみ [`PRESS_STATE_ATTR`] を付与する。`prevent_default()` は
    /// 呼ばない（ネイティブ要素への意図しない誤爆を避け、純粋な視覚状態
    /// シグナルに留める）。
    fn handle_keydown(root: &Element, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if keyboard_event.repeat() || !is_press_activation_key(&keyboard_event.key()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
    }

    /// `keyup`: 活性化キーの解放で [`PRESS_STATE_ATTR`] を外す。
    fn handle_keyup(root: &Element, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if !is_press_activation_key(&keyboard_event.key()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
    }

    /// `root` へ hover/press 検知の 7 リスナー（pointerover/pointerout/
    /// pointerdown/pointerup/pointercancel/keydown/keyup）を委譲登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`events::wire_events` と同じく登録回数を定数個に
    /// 抑える方針（A04 対策）で、要素ごとの動的登録・解除は行わない
    /// （動的挿入要素にも root 委譲で自動対応する）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_gesture(root: Element) -> Result<(), JsValue> {
        let pointerover_root = root.clone();
        let pointerover_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerover(&pointerover_root, &event);
        });
        root.add_event_listener_with_callback(
            "pointerover",
            pointerover_closure.as_ref().unchecked_ref(),
        )?;
        pointerover_closure.forget();

        let pointerout_root = root.clone();
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerout(&pointerout_root, &event);
        });
        root.add_event_listener_with_callback(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
        )?;
        pointerout_closure.forget();

        let pointerdown_root = root.clone();
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&pointerdown_root, &event);
        });
        root.add_event_listener_with_callback(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
        )?;
        pointerdown_closure.forget();

        let pointerup_root = root.clone();
        let pointerup_or_cancel_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerup_or_cancel(&pointerup_root, &event);
        });
        root.add_event_listener_with_callback(
            "pointerup",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
        )?;
        root.add_event_listener_with_callback(
            "pointercancel",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
        )?;
        pointerup_or_cancel_closure.forget();

        let keydown_root = root.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let keyup_root = root.clone();
        let keyup_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keyup(&keyup_root, &event);
        });
        root.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_gesture;

#[cfg(test)]
mod tests {
    use super::{is_press_activation_key, is_touch_pointer};

    #[test]
    fn is_touch_pointer_matches_only_touch() {
        assert!(is_touch_pointer("touch"));
        assert!(!is_touch_pointer("mouse"));
        assert!(!is_touch_pointer("pen"));
        assert!(!is_touch_pointer(""));
    }

    #[test]
    fn is_press_activation_key_matches_enter_and_space() {
        assert!(is_press_activation_key("Enter"));
        assert!(is_press_activation_key(" "));
        assert!(!is_press_activation_key("Escape"));
        assert!(!is_press_activation_key("Spacebar"));
        assert!(!is_press_activation_key(""));
    }
}
