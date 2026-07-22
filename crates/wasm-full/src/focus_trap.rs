//! Dialog のフォーカストラップと trigger 復帰（イシュー #586、親 #584）。
//!
//! `fandhe-frontend-headless-ui` の Dialog（#531）は SSR マークアップ（`data-scope="dialog"`/
//! `data-part="content"`/`aria-modal`/`role`）と開閉状態機械までを提供し、
//! 「フォーカストラップ・トリガーへのフォーカス復帰は JS ランタイム側
//! （本クレート）の責務」として明示的にスコープ外としていた
//! （`crates/headless-ui/src/dialog.rs` 冒頭 doc の「スコープ外」節参照）。
//! 本モジュールはその欠落を埋め、`aria-modal="true"` の content 内へ Tab
//! フォーカスを循環させ、閉鎖時にトリガー（または直前のフォーカス位置）へ
//! フォーカスを復帰させる。
//!
//! [`crate::overlay`]・[`crate::keynav`] と同じ 2 層構成を踏襲する: web-sys
//! に依存しない純粋ロジック層（[`should_trap`]・[`is_tabbable`]・
//! [`initial_focus_index`]・[`next_trap_index`]、native の `cargo test` で
//! 検証可能）と、`#[cfg(target_arch = "wasm32")]` でゲートした配線層
//! （[`wiring::FocusTrapController`]）に分離する。
//!
//! # 他モジュール・他クレートとの契約
//!
//! - [`should_trap`] は `data-scope="dialog"` かつ `aria-modal="true"` の
//!   ときのみ `true` を返す（fail-closed。欠落・`"false"`・不正値・非 dialog
//!   scope はいずれも `false`）。`fandhe-frontend-headless-ui` 側に専用 API を
//!   追加せず、`crates/headless-ui/src/dialog.rs::content` が出力する既存
//!   属性のみを読む（本イシューでは headless-ui クレートを変更しない）。
//! - `data-autofocus` は初期フォーカス先を明示するオプトイン属性。呼び出し側が
//!   `dialog::content`/子パーツの `attrs: Vec<(&str, &str)>` 引数経由で付与する
//!   前提とし（[`crate::overlay`] の opt-out 属性と同じ配線方針）、専用 API は
//!   追加しない。
//! - 本モジュールは `"close"` dispatch・再描画・DOM の open/close 属性更新を
//!   一切行わない。[`wiring::FocusTrapController::push_trap`]/[`wiring::FocusTrapController::pop_trap`]
//!   を Dialog の open/close タイミングで呼ぶのはイシュー #580（DOM イベント
//!   配線統合層）の責務とする（[`crate::overlay`] と同じ責務分離方針）。
//! - スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応済み、兄弟
//!   イシューで追跡中）: 背景コンテンツの `inert`/`aria-hidden` 化、focusin
//!   による強制引き戻し、Tooltip の `openDelay`/`closeDelay`（イシュー #587）。

use crate::events::AttrSource;

/// `content` がフォーカストラップの対象かどうかを判定する。
///
/// `data-scope="dialog"` かつ `aria-modal="true"` のときのみ `true`。
/// それ以外（`data-scope` 欠落・非 dialog scope・`aria-modal` 欠落・
/// `"false"`・不正値）はすべて `false` とする（クライアントで改ざんされ
/// うる `data-*`/`aria-*` 入力に対する fail-closed。トラップを誤って
/// 有効化するより、無効のまま panic しない安全側を優先する）。
#[must_use]
pub fn should_trap<T: AttrSource>(content: &T) -> bool {
    content.attr("data-scope").as_deref() == Some("dialog")
        && content.attr("aria-modal").as_deref() == Some("true")
}

/// 要素が Tab 循環の対象（tabbable）かどうかを判定する。
///
/// 次のいずれかを満たす要素は非 tabbable として除外する:
/// - `disabled` 属性を持つ（`disabled=""`/`disabled="disabled"` 等、値によらず
///   属性の**存在**で判定。HTML の真偽属性の慣例に合わせる）
/// - `data-disabled` 属性を持つ（headless-ui コンポーネントの無効化表現、
///   [`crate::keynav`] の `is_disabled` と同じ判定方針）
/// - `tabindex="-1"`（プログラム的フォーカスのみを許可する明示的除外）
/// - `hidden` 属性を持つ
/// - `type="hidden"`（`<input type="hidden">` 等）
///
/// 属性値の解釈は固定語彙のみとし、未知の値は「非該当（tabbable のまま）」
/// にフォールバックする（fail-closed の逆側で「トラップから漏らさない」を
/// 優先する必要がある場合は呼び出し側で別途フィルタする）。
#[must_use]
pub fn is_tabbable<T: AttrSource>(el: &T) -> bool {
    if el.attr("disabled").is_some() {
        return false;
    }
    if el.attr("data-disabled").is_some() {
        return false;
    }
    if el.attr("tabindex").as_deref() == Some("-1") {
        return false;
    }
    if el.attr("hidden").is_some() {
        return false;
    }
    if el.attr("type").as_deref() == Some("hidden") {
        return false;
    }
    true
}

/// tabbable 候補一覧から、Dialog 開時に初期フォーカスを当てる要素の index を
/// 選ぶ。
///
/// 選定順序: `data-autofocus` 属性を持つ最初の tabbable 要素 → 先頭の
/// tabbable 要素 → 候補が 1 件も tabbable でなければ `None`
/// （呼び出し側は `None` の場合 content 自身へフォールバックする、
/// [`wiring::FocusTrapController::push_trap`] 参照）。
#[must_use]
pub fn initial_focus_index<T: AttrSource>(candidates: &[T]) -> Option<usize> {
    let tabbable_indices: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, el)| is_tabbable(*el))
        .map(|(index, _)| index)
        .collect();

    if let Some(index) = tabbable_indices
        .iter()
        .find(|&&index| candidates[index].attr("data-autofocus").is_some())
    {
        return Some(*index);
    }

    tabbable_indices.first().copied()
}

/// Tab キー押下時の次のフォーカス先 index を計算する（トラップ活性時のみ
/// 呼ばれる想定。Tab キーは常に `prevent_default()` してこの関数の結果へ
/// 手動でフォーカスを移す、[`wiring::FocusTrapController`] doc 参照）。
///
/// - `len == 0`: `None`（フォーカス対象なし）
/// - `current == None`（トラップ外・不明な現在位置からの入場）: 通常 Tab は
///   先頭（`0`）、Shift+Tab は末尾（`len - 1`）
/// - `current == Some(len - 1)`（末尾）かつ通常 Tab: 先頭（`0`）へ循環
/// - `current == Some(0)`（先頭）かつ Shift+Tab: 末尾（`len - 1`）へ循環
/// - それ以外: 通常 Tab は `current + 1`、Shift+Tab は `current - 1`
#[must_use]
pub fn next_trap_index(current: Option<usize>, len: usize, backward: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len - 1;
    match current {
        None => Some(if backward { last } else { 0 }),
        Some(index) if index >= last => Some(if backward { index.saturating_sub(1) } else { 0 }),
        Some(0) => Some(if backward { last } else { 1 }),
        Some(index) => Some(if backward { index - 1 } else { index + 1 }),
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`overlay.rs`/`keynav.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{initial_focus_index, is_tabbable, next_trap_index, should_trap};
    use crate::events::AttrSource;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent};

    /// tabbable 候補を収集する `querySelectorAll` セレクタ。WAI-ARIA dialog
    /// パターンで一般的にフォーカス可能とされる要素のみを対象にする。
    ///
    /// 裸の `[data-autofocus]` は含めない（イシュー #586 レビュー指摘）:
    /// `data-autofocus` は「tabbable 候補の中から初期フォーカス先を選ぶ」
    /// 優先度マーカーであり、それ自体が要素をフォーカス可能にする属性
    /// ではない。裸の `[data-autofocus]` をセレクタに含めると、`<div
    /// data-autofocus>` のようなネイティブに非フォーカス可能な要素まで
    /// `is_tabbable` を素通りして tabbable 候補に混入し、Tab 押下時に
    /// `prevent_default()` した上で no-op な `focus()` を呼ぶ（実際には
    /// フォーカスが移動しない）ため、以降 `document.active_element()` が
    /// 候補内のどれとも一致せず Tab 循環が固着する。`data-autofocus` を
    /// ネイティブに非フォーカス可能な要素へ付与したい呼び出し側は、
    /// `tabindex` も併せて付与する必要がある（`[tabindex]` 節で拾われる）。
    const TABBABLE_SELECTOR: &str = "a[href],button,input,select,textarea,[tabindex]";

    /// `web_sys::Element` を [`AttrSource`] へ橋渡しする薄いラッパー
    /// （`events.rs::wiring::ElementAttrSource`/`overlay.rs::wiring::ElementAttrSource`
    /// と同じ意図の配線層専用アダプタ）。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// `content` 配下の tabbable 要素を、DOM 順（`querySelectorAll` の順序）で
    /// 都度収集する。DOM 変化（要素の追加・削除・`data-disabled` の切り替え等）
    /// へ追随するため、呼び出しごとに毎回収集し直す
    /// （`keynav.rs` の「DOM 属性を単一情報源とする」方針を継承。キャッシュしない）。
    fn collect_tabbable(content: &Element) -> Vec<Element> {
        let Ok(node_list) = content.query_selector_all(TABBABLE_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut result = Vec::with_capacity(len as usize);
        for i in 0..len {
            let Some(node) = node_list.item(i) else {
                continue;
            };
            let Ok(el) = node.dyn_into::<Element>() else {
                continue;
            };
            if is_tabbable(&ElementAttrSource(&el)) {
                result.push(el);
            }
        }
        result
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`tabindex`）は `&'static str` リテラルで固定された非 URL・
    /// 非イベントハンドラ属性であり実害はないが、
    /// `fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する
    /// （`keynav.rs::wiring::set_dom_attribute` と同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return;
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    /// `content` 自身をプログラム的フォーカスのみ可能にする（Tab 到達順は
    /// 汚さず、WAI-ARIA dialog パターンの「tabbable な子が無い場合は content
    /// 自身へフォーカスする」慣行に合わせる）。`tabindex` は固定リテラル
    /// `"-1"` のみ書き込み、動的値は属性値スロットへ混入しない（REQ-1 の
    /// 精神を DOM 属性書き込みにも適用する）。
    fn focus_content_itself(content: &Element) {
        set_dom_attribute(content, "tabindex", "-1");
        if let Ok(html_el) = content.clone().dyn_into::<HtmlElement>() {
            let _ = html_el.focus();
        }
    }

    /// [`FocusTrapController`] が管理する 1 トラップの実体。
    struct MountedTrap {
        /// トラップ対象の content 要素。Tab 循環のたびに配下を再収集する
        /// 基点として保持する。
        content: Element,
        /// push 時点でスナップショットした「トラップ開始前にフォーカスが
        /// 当たっていた要素」（`document.active_element()` 優先、取得
        /// 不能時は呼び出し側が渡した `trigger` 引数）。[`FocusTrapController::pop_trap`]
        /// の復帰先として使う。
        restore_to: Option<Element>,
    }

    /// document へ keydown（Tab 循環）リスナーを **1 回だけ** 登録し、開いている
    /// Dialog のフォーカストラップをスタックで管理する配線層の中核型。
    ///
    /// [`crate::overlay::OverlayCloseController`] と対称の設計:
    /// `Closure::forget` を使わず [`Drop`] でリスナーを対称的に解除する
    /// （コントローラを繰り返し生成・破棄しても document 上のリスナー数が
    /// 無制限に増加しない、A04 安全でない設計（リスナーリーク/DoS）対策）。
    /// `Drop` はリスナー解除のみを行い、フォーカス復帰は行わない
    /// （アンマウント時の暗黙復帰は呼び出し側の意図と競合しうるため、復帰は
    /// [`Self::pop_trap`] の明示経路に限定する）。
    pub struct FocusTrapController {
        document: Document,
        stack: std::rc::Rc<std::cell::RefCell<Vec<MountedTrap>>>,
        keydown_closure: Closure<dyn FnMut(Event)>,
    }

    impl FocusTrapController {
        /// `document` へ keydown リスナーを登録したコントローラを組み立てる。
        ///
        /// # Errors
        ///
        /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
        pub fn new(document: &Document) -> Result<Self, JsValue> {
            let stack: std::rc::Rc<std::cell::RefCell<Vec<MountedTrap>>> =
                std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));

            let keydown_stack = stack.clone();
            // `Closure` は `'static` を要求するため、参照ではなく所有権を
            // 持つ複製を渡す（`web_sys::Document` は内部 `JsValue` の複製で
            // 実 DOM ドキュメントへの参照を共有する、`overlay.rs::wiring` と
            // 同じ配線パターン）。
            let keydown_document = document.clone();
            let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Ok(keyboard_event) = event.clone().dyn_into::<KeyboardEvent>() else {
                    return;
                };
                if keyboard_event.key() != "Tab" {
                    return;
                }
                // 修飾キー付き Tab（ブラウザ/OS ショートカットとの衝突回避）は
                // no-op とする（`keynav.rs` と同方針）。
                if keyboard_event.ctrl_key()
                    || keyboard_event.alt_key()
                    || keyboard_event.meta_key()
                {
                    return;
                }

                let stack_ref = keydown_stack.borrow();
                // 最上位のトラップのみが対象（入れ子 Dialog はレイヤー方式、
                // `overlay.rs` の Escape と同方針）。
                let Some(topmost) = stack_ref.last() else {
                    return;
                };
                let content = topmost.content.clone();
                drop(stack_ref);

                let candidates = collect_tabbable(&content);
                if candidates.is_empty() {
                    // tabbable な子が無い「空ダイアログ」は稀なフォールバックでは
                    // なく通常発生しうるケース（例: 確認メッセージのみで
                    // ボタンが無いダイアログ）。この経路で `prevent_default()`
                    // を呼ばずに早期 return すると、ブラウザの既定動作で Tab が
                    // `aria-modal` content の外へフォーカスを漏らし、トラップが
                    // 破られる（イシュー #586 レビュー指摘、CVE-XSS 相当の
                    // ではないが WAI-ARIA dialog パターン違反）。
                    // `push_trap`/[`focus_content_itself`] が既に `content` へ
                    // `tabindex="-1"` を付与済みのため、ここでは Tab を消費し
                    // フォーカスを `content` へ固定し直す。
                    event.prevent_default();
                    if let Ok(html_el) = content.clone().dyn_into::<HtmlElement>() {
                        let _ = html_el.focus();
                    }
                    return;
                }

                let active = keydown_document
                    .active_element()
                    .and_then(|el| el.dyn_into::<Element>().ok());
                let current_index = active.as_ref().and_then(|active_el| {
                    candidates
                        .iter()
                        .position(|candidate| candidate.is_same_node(Some(active_el)))
                });

                let backward = keyboard_event.shift_key();
                let Some(next_index) = next_trap_index(current_index, candidates.len(), backward)
                else {
                    return;
                };

                event.prevent_default();
                if let Ok(html_el) = candidates[next_index].clone().dyn_into::<HtmlElement>() {
                    let _ = html_el.focus();
                }
            });
            document.add_event_listener_with_callback(
                "keydown",
                keydown_closure.as_ref().unchecked_ref(),
            )?;

            Ok(Self {
                document: document.clone(),
                stack,
                keydown_closure,
            })
        }

        /// Dialog 開時に呼ぶ。`content` が [`should_trap`] を満たさない
        /// （非 modal・非 dialog scope）場合は登録せず `None` を返す
        /// （fail-closed。呼び出し側は戻り値 `None` の場合、後続の
        /// [`Self::pop_trap`] を呼ぶ必要がない）。
        ///
        /// 復帰先として、呼び出し時点の `document.active_element()`
        /// （取得不能時は `trigger` 引数）をスナップショットし、初期
        /// フォーカスを [`initial_focus_index`] が選んだ要素（候補が
        /// 空なら `content` 自身、`tabindex="-1"` を付与した上でフォーカス）
        /// へ移す。
        ///
        /// 戻り値の index は [`Self::pop_trap`] の引数として使う
        /// （push 時点のスタック末尾位置。`overlay.rs::OverlayCloseController::push_overlay`
        /// と同じく非最上位 remove でシフトしうる。単一 Dialog の通常運用
        /// では常に LIFO で push/pop されるため実害は無いが、契約は明示する）。
        #[must_use]
        pub fn push_trap(&self, content: &Element, trigger: Option<&Element>) -> Option<usize> {
            let source = ElementAttrSource(content);
            if !should_trap(&source) {
                return None;
            }

            let restore_to = self
                .document
                .active_element()
                .and_then(|el| el.dyn_into::<Element>().ok())
                .or_else(|| trigger.cloned());

            let candidates = collect_tabbable(content);
            let candidate_sources: Vec<ElementAttrSource<'_>> =
                candidates.iter().map(ElementAttrSource).collect();
            match initial_focus_index(&candidate_sources) {
                Some(index) => {
                    if let Ok(html_el) = candidates[index].clone().dyn_into::<HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
                None => focus_content_itself(content),
            }

            let mut stack = self.stack.borrow_mut();
            stack.push(MountedTrap {
                content: content.clone(),
                restore_to,
            });
            Some(stack.len() - 1)
        }

        /// Dialog 閉時に呼ぶ（[`Self::push_trap`] と対称の呼び出しを呼び出し側
        /// の契約とする）。スタックから該当エントリを除去し、push 時に
        /// スナップショットした復帰先要素へフォーカスを戻す。
        ///
        /// 復帰先が document から切断済み（`is_connected` が `false`）の場合は
        /// no-op とする（fail-closed。切断済み要素へ `focus()` してもブラウザは
        /// 何もしないため実害は無いが、意図を明示するため事前判定する）。
        /// `index` が範囲外の場合も panic せず no-op とする（呼び出し側の
        /// 二重 pop・契約違反に対する安全側フォールバック、[`crate::overlay::OverlayCloseController::remove_overlay`]
        /// と同方針）。
        pub fn pop_trap(&self, index: usize) {
            let removed = {
                let mut stack = self.stack.borrow_mut();
                if index >= stack.len() {
                    return;
                }
                stack.remove(index)
            };

            let Some(restore_to) = removed.restore_to else {
                return;
            };
            if !restore_to.is_connected() {
                return;
            }
            if let Ok(html_el) = restore_to.dyn_into::<HtmlElement>() {
                let _ = html_el.focus();
            }
        }

        /// 現在スタックに登録されているトラップの件数（テスト・デバッグ用途）。
        #[must_use]
        pub fn stack_len(&self) -> usize {
            self.stack.borrow().len()
        }
    }

    impl Drop for FocusTrapController {
        /// keydown リスナーを解除する（登録は [`Self::new`] の 1 回のみ、
        /// 解除もここでの 1 回のみで完結し、`Closure::forget` を使わない。
        /// フォーカス復帰は行わない、本型の doc 冒頭参照）。
        fn drop(&mut self) {
            let _ = self.document.remove_event_listener_with_callback(
                "keydown",
                self.keydown_closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::FocusTrapController;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// native `cargo test` 用のテストダブル（`events.rs::tests::FakeElement`/
    /// `overlay.rs::tests::FakeElement` と同じ意図）。
    struct FakeElement {
        attrs: HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeElement {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn element(attrs: &[(&'static str, &'static str)]) -> FakeElement {
        FakeElement {
            attrs: attrs.iter().copied().collect(),
        }
    }

    // --- should_trap ---

    #[test]
    fn should_trap_true_for_modal_dialog() {
        let content = element(&[("data-scope", "dialog"), ("aria-modal", "true")]);
        assert!(should_trap(&content));
    }

    #[test]
    fn should_trap_false_for_non_modal_dialog() {
        let content = element(&[("data-scope", "dialog"), ("aria-modal", "false")]);
        assert!(!should_trap(&content));
    }

    #[test]
    fn should_trap_false_when_aria_modal_missing() {
        let content = element(&[("data-scope", "dialog")]);
        assert!(!should_trap(&content));
    }

    #[test]
    fn should_trap_false_for_non_dialog_scope() {
        let content = element(&[("data-scope", "popover"), ("aria-modal", "true")]);
        assert!(!should_trap(&content));
    }

    #[test]
    fn should_trap_false_for_bogus_aria_modal_value() {
        for bogus in ["TRUE", "1", ""] {
            let content = element(&[("data-scope", "dialog"), ("aria-modal", bogus)]);
            assert!(!should_trap(&content), "aria-modal={bogus:?}");
        }
    }

    // --- is_tabbable ---

    #[test]
    fn is_tabbable_true_for_plain_button() {
        let el = element(&[]);
        assert!(is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_false_when_disabled() {
        let el = element(&[("disabled", "")]);
        assert!(!is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_false_when_data_disabled() {
        let el = element(&[("data-disabled", "")]);
        assert!(!is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_false_when_tabindex_negative_one() {
        let el = element(&[("tabindex", "-1")]);
        assert!(!is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_true_when_tabindex_zero() {
        let el = element(&[("tabindex", "0")]);
        assert!(is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_false_when_hidden() {
        let el = element(&[("hidden", "")]);
        assert!(!is_tabbable(&el));
    }

    #[test]
    fn is_tabbable_false_when_type_hidden() {
        let el = element(&[("type", "hidden")]);
        assert!(!is_tabbable(&el));
    }

    // --- initial_focus_index ---

    #[test]
    fn initial_focus_index_prefers_data_autofocus() {
        let candidates = vec![
            element(&[]),
            element(&[("data-autofocus", "")]),
            element(&[]),
        ];
        assert_eq!(initial_focus_index(&candidates), Some(1));
    }

    #[test]
    fn initial_focus_index_falls_back_to_first_tabbable() {
        let candidates = vec![element(&[("disabled", "")]), element(&[]), element(&[])];
        assert_eq!(initial_focus_index(&candidates), Some(1));
    }

    #[test]
    fn initial_focus_index_none_when_no_candidates() {
        let candidates: Vec<FakeElement> = vec![];
        assert_eq!(initial_focus_index(&candidates), None);
    }

    #[test]
    fn initial_focus_index_none_when_all_non_tabbable() {
        let candidates = vec![element(&[("disabled", "")]), element(&[("hidden", "")])];
        assert_eq!(initial_focus_index(&candidates), None);
    }

    #[test]
    fn initial_focus_index_ignores_data_autofocus_on_non_tabbable() {
        // data-autofocus が付いていても disabled なら除外し、次の tabbable へ
        // フォールバックする。
        let candidates = vec![
            element(&[("data-autofocus", ""), ("disabled", "")]),
            element(&[]),
        ];
        assert_eq!(initial_focus_index(&candidates), Some(1));
    }

    // --- next_trap_index ---

    #[test]
    fn next_trap_index_empty_is_none() {
        assert_eq!(next_trap_index(Some(0), 0, false), None);
        assert_eq!(next_trap_index(None, 0, false), None);
    }

    #[test]
    fn next_trap_index_forward_from_last_wraps_to_first() {
        assert_eq!(next_trap_index(Some(2), 3, false), Some(0));
    }

    #[test]
    fn next_trap_index_backward_from_first_wraps_to_last() {
        assert_eq!(next_trap_index(Some(0), 3, true), Some(2));
    }

    #[test]
    fn next_trap_index_forward_middle_increments() {
        assert_eq!(next_trap_index(Some(1), 3, false), Some(2));
    }

    #[test]
    fn next_trap_index_backward_middle_decrements() {
        assert_eq!(next_trap_index(Some(1), 3, true), Some(0));
    }

    #[test]
    fn next_trap_index_no_current_forward_enters_at_first() {
        assert_eq!(next_trap_index(None, 3, false), Some(0));
    }

    #[test]
    fn next_trap_index_no_current_backward_enters_at_last() {
        assert_eq!(next_trap_index(None, 3, true), Some(2));
    }

    #[test]
    fn next_trap_index_single_candidate_forward_stays() {
        assert_eq!(next_trap_index(Some(0), 1, false), Some(0));
    }

    #[test]
    fn next_trap_index_single_candidate_backward_stays() {
        assert_eq!(next_trap_index(Some(0), 1, true), Some(0));
    }
}
