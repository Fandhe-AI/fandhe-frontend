//! add-to-basket ボタンの DOM 配線層（イシュー #2538、親 button の Motion+
//! 由来 variant）。
//!
//! # 責務境界
//!
//! `docs/design/motion-reference-adoption-policy.md` §4「B 群: `data-state`
//! 状態機械 + タイマー」に分類される。rAF ループ・`fandhe-frontend-animation`
//! への依存は不要——3 状態（`idle` → `adding` → `added` → `idle`）の遷移と
//! 自動リセットタイマーのみで完結する（`headless_clipboard.rs` の
//! click → `data-state` 遷移 → `PendingTimer` パターンと同型）。
//!
//! # スコープ外（実装計画 §1 の判断）
//!
//! ラベル文言の動的差し替え（"Added!" 等のテキスト変更）は行わない。
//! `data-state` に応じたアイコンのクロスフェード（`data-part=`
//! `"basket-icon-idle"`/`"basket-icon-adding"`/`"basket-icon-added"` の
//! CSS 切替、`crate::button_motion` 側の責務）のみを扱う。理由: (1) ラベル
//! 文言の意味変更はカタログ/EC ロジック（アプリ責務）に踏み込み
//! `docs/policy/intentional-non-adoption.md` §3.25 の責務境界に抵触
//! しかねない、(2) 動的テキスト書き換えは新たな文字列注入経路を増やす
//! （REQ-1 既定エスケープの外側で `textContent` を書く操作が必要になり
//! 得る）。実際のカート追加ロジック自体は `events.rs` の `data-action`
//! 委譲を通じて利用者側ハンドラが担う（本モジュールは視覚状態のみ）。
//!
//! # `data-*` 属性の命名
//!
//! `data-fandhe-*` 内部プレフィックス。状態は各部品共通の `data-state`
//! （値 `"idle"`/`"adding"`/`"added"`）を用いる。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_hold_to_confirm` の直後で `Self::wire_add_to_basket` を呼ぶ
//! （feature `add-to-basket`、既定 on）。`dispatch` チャネルを持たない
//! 属性専用配線のため（`Self::wire_sidebar`/`Self::wire_gesture` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! `events.rs` の `data-action` 委譲は変更しない・妨げない（本モジュールの
//! click リスナーは bubble フェーズで登録し、視覚状態のみを扱う。利用者の
//! `data-action` ハンドラは同じ click イベントを通常どおり受け取る）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! `data-state`/`data-fandhe-*` はいずれも固定リテラルのみを書き込み、
//! 利用者制御の文字列を属性値・セレクタへ混ぜない。

/// opt-in（著者が SSR 出力に静的に付与）: add-to-basket を有効化する
/// ルート要素マーカー。
pub const ADD_TO_BASKET_ATTR: &str = "data-fandhe-add-to-basket";
/// 候補走査セレクタ。
pub const ADD_TO_BASKET_SELECTOR: &str = "[data-fandhe-add-to-basket]";
/// idle 状態（初期値）。
pub const STATE_IDLE: &str = "idle";
/// 追加中（一時的な視覚フィードバック）状態。
pub const STATE_ADDING: &str = "adding";
/// 追加完了状態。
pub const STATE_ADDED: &str = "added";
/// `adding` を表示し続ける既定時間（ミリ秒）。
pub const DEFAULT_ADDING_DURATION_MS: i32 = 400;
/// `added` を表示し続けてから `idle` へ戻す既定時間（ミリ秒）。
pub const DEFAULT_ADDED_RESET_TIMEOUT_MS: i32 = 1500;

/// 現在の `data-state` 値から、click イベントを受理すべきかどうかを判定
/// する（純粋関数、native `cargo test` 可能）。`idle`（または属性未設定＝
/// `None`）のときのみ新しい遷移を受理する——`adding`/`added` 中の再クリック
/// は二重トリガー防止のため無視する（`headless_clipboard.rs` の pending
/// timer 存在チェックと同型のガード）。
#[must_use]
pub fn should_accept_click(current_state: Option<&str>) -> bool {
    match current_state {
        None => true,
        Some(state) => state != STATE_ADDING && state != STATE_ADDED,
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        should_accept_click, ADD_TO_BASKET_SELECTOR, DEFAULT_ADDED_RESET_TIMEOUT_MS,
        DEFAULT_ADDING_DURATION_MS, STATE_ADDED, STATE_ADDING, STATE_IDLE,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// 保留中の自動遷移タイマー（`headless_clipboard.rs::PendingTimer` と
    /// 同型のパターン）。
    struct PendingTimer {
        handle: i32,
        _closure: Closure<dyn FnMut()>,
    }

    /// 1 トリガー要素専用の保留タイマー保持スロット。要素ごとに
    /// [`wire_candidate`] が 1 個ずつ `Rc::new(RefCell::new(None))` を
    /// 生成し、その要素の `click` クロージャへ move する（複数要素間で
    /// 共有しない・マップで id 管理もしない、最も単純な 1 要素 1 スロット
    /// 設計）。
    type PendingTimerSlot = Rc<RefCell<Option<PendingTimer>>>;

    /// `element` の `data-state` を書き込む。
    fn set_state(element: &Element, state: &str) {
        let _ = set_dom_attribute(element, "data-state", state);
    }

    /// `element` の現在の `data-state` から click を受理するか判定する。
    fn handle_click(element: &Element, pending: &PendingTimerSlot) {
        let current = element.get_attribute("data-state");
        if !should_accept_click(current.as_deref()) {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };

        // 既存の保留タイマーがあれば先に解除する（二重トリガー防止の
        // 追加防御。`should_accept_click` が `adding`/`added` 中の再入を
        // 既に弾くため通常到達しないが、状態と実タイマーの不整合が
        // 生じた場合の fail-closed な後始末として残す）。
        if let Some(timer) = pending.borrow_mut().take() {
            window.clear_timeout_with_handle(timer.handle);
        }

        set_state(element, STATE_ADDING);

        let element_for_added = element.clone();
        let pending_for_added = pending.clone();
        let to_added = Closure::<dyn FnMut()>::new(move || {
            set_state(&element_for_added, STATE_ADDED);
            schedule_reset_to_idle(&element_for_added, &pending_for_added);
        });
        let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            to_added.as_ref().unchecked_ref(),
            DEFAULT_ADDING_DURATION_MS,
        ) else {
            return;
        };
        *pending.borrow_mut() = Some(PendingTimer {
            handle,
            _closure: to_added,
        });
    }

    /// `added` 表示後、[`DEFAULT_ADDED_RESET_TIMEOUT_MS`] 経過で `idle` へ
    /// 戻すタイマーを予約する。
    fn schedule_reset_to_idle(element: &Element, pending: &PendingTimerSlot) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let element_for_idle = element.clone();
        let to_idle = Closure::<dyn FnMut()>::new(move || {
            set_state(&element_for_idle, STATE_IDLE);
        });
        let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            to_idle.as_ref().unchecked_ref(),
            DEFAULT_ADDED_RESET_TIMEOUT_MS,
        ) else {
            return;
        };
        *pending.borrow_mut() = Some(PendingTimer {
            handle,
            _closure: to_idle,
        });
    }

    /// `root` 配下の `[data-fandhe-add-to-basket]` 要素（複数可）を
    /// 出現順に集める（`scroll_driver.rs::collect_scroll_progress_candidates`
    /// と同型）。
    fn collect_candidates(root: &Element) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(ADD_TO_BASKET_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// 1 要素へ `click`（bubble フェーズ、`events.rs` の `data-action`
    /// 委譲を妨げない）リスナーを登録する。
    fn wire_candidate(element: &Element) -> Result<(), JsValue> {
        let pending: PendingTimerSlot = Rc::new(RefCell::new(None));
        let click_element = element.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            handle_click(&click_element, &pending);
        });
        element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    /// `root` 配下の add-to-basket 候補へ配線する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`window` が取得できない環境では配線をスキップし
    /// `Ok(())` を返す（fail-closed、`headless_clipboard.rs` と同型）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_add_to_basket(root: Element) -> Result<(), JsValue> {
        if web_sys::window().is_none() {
            return Ok(());
        }
        for candidate in collect_candidates(&root) {
            wire_candidate(&candidate)?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_add_to_basket;

#[cfg(test)]
mod tests {
    use super::{should_accept_click, STATE_ADDED, STATE_ADDING, STATE_IDLE};

    #[test]
    fn accepts_click_when_idle_or_unset() {
        assert!(should_accept_click(None));
        assert!(should_accept_click(Some(STATE_IDLE)));
    }

    #[test]
    fn rejects_click_while_adding_or_added() {
        assert!(!should_accept_click(Some(STATE_ADDING)));
        assert!(!should_accept_click(Some(STATE_ADDED)));
    }

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(super::ADD_TO_BASKET_ATTR, "data-fandhe-add-to-basket");
        assert_eq!(super::ADD_TO_BASKET_SELECTOR, "[data-fandhe-add-to-basket]");
    }
}
