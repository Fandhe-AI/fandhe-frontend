//! PinInput（`fandhe-frontend-headless-ui` `pin_input` モジュール）の
//! `input` / `focusin` → dispatch 配線（イシュー #3834、親 #3831）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/pin_input.rs` は桁ごとの `input` パーツ
//! （`data-index`・`maxlength="1"`）と `"input"`/`"focus"` 等の dispatch 契約
//! （`PinInput::decode_action`）を提供する一方、実 DOM のイベントを dispatch へ
//! つなぐ配線を「wasm 層の後続責務」として残していた。本モジュールは
//! そのうち **1 文字入力と次桁へのフォーカス移動** だけを実装する。
//! 貼り付け・Backspace/Delete/矢印キー・実ブラウザ検証は後続の #3835/#3836 の担当で、
//! 本モジュールは `paste`/`keydown` に一切触れない。
//!
//! # 設計（`number_input.rs` と同型の 2 層構成）
//!
//! - 純粋ロジック層（本ファイル直下の `pub` 関数）は web-sys に依存せず、
//!   native の `cargo test` で検証できる。
//! - 配線層（`wiring`）のみ `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! 本モジュールは `lib.rs` 側で `#[cfg(feature = "pin-input")]` によりモジュール
//! ごとゲートされる（feature off のビルドへ配線コードを含めない、イシュー #3834
//! の受け入れ条件。`number_input` がモジュールをゲートしないのとは意図的に異なる）。
//!
//! # dispatch とアクションの対応
//!
//! | DOM イベント | 条件 | アクション | payload |
//! |---|---|---|---|
//! | `input`（桁 input） | 値がちょうど 1 文字で kind に適合 | `"focus"` → `"input"` | `data-index` → その 1 文字 |
//! | `focusin`（桁 input） | `data-index` が桁数未満 | `"focus"` | `data-index` |
//!
//! `"focus"` を `"input"` の直前に送るのは、アプリ状態の focused が古くても正しい桁へ
//! 書き込ませるため（`number_input` が増減前に `"set"` で同期するのと同じ考え方）。
//! 受理された入力のあと、次の桁（`min(index + 1, count - 1)`）へ DOM フォーカスを移す。
//! 0 文字（削除系）と 2 文字以上（autofill・`insertFromPaste`）は何もしない
//! （DOM の値も消さない）。これらは #3835 が `"paste"` 等へ接続する。
//!
//! # `MAPPING_TABLE` 境界行との役割分担
//!
//! `crate::headless::MAPPING_TABLE` の `(pin-input, input)` 行は `data-value` を持たない
//! 現行マークアップでは常に解決されない「境界行」であり、pin-input を他のクリック可能な
//! 部品の内側に置いたとき祖先探索が外側の部品へ抜けるのを防ぐためだけに存在する。
//! 桁番号の同期は本モジュールの `focusin` リスナーが担う。
//!
//! # 二重 dispatch の回避
//!
//! 桁 input に `data-action-input` がある場合は `crate::events::wire_events` が `input`
//! の dispatch を担うため、本モジュールは dispatch せずフォーカス移動だけを行う。
//! 同一 root に複数の PinInput を置く場合のインスタンス区別も、この上書き名で回避する
//! （payload にインスタンス ID が無いため、本モジュール単独では 1 root : 1 状態機械の
//! 簡略化が前提）。
//!
//! # DOM 同期契約
//!
//! 桁の値・`data-filled`・`data-complete`・hidden-input の反映は、アプリの `C` が dirty を
//! 立てて `apply_dirty_if_any` の再描画経路で行う。本モジュールが DOM を直接書くのは
//! 大文字そろえ kind の表示補正と、kind に適合しない文字の旧値復元（描画済み value 属性、無ければ空）の 2 点だけで、
//! いずれも `HtmlInputElement::set_value`（ASCII 英数字 1 文字または空文字）に限る。
//! HTML 文字列は組み立てない（REQ-1）。`fandhe_frontend_interactive::dispatch` は
//! 「復号できたか」しか返さず状態機械の受理・拒否は分からないため、拒否の検出は
//! 事前フィルタ（`prefilter_char`）で担い、`data-hydrate-kind` が無く kind を知れない場合は
//! 再描画後のライブ `value` で受理を確かめる（`input_accepted`。入力で更新されるのは `value` プロパティで属性ではない）。受理を確かめてからだけ次桁へ
//! フォーカスを移し、`data-action-input` 桁は bubble 側の dispatch 完了後（`setTimeout(0)`）に移す。dispatch 後に要素が差し替わりうるので、
//! 要素参照は保持せず「何番目の pin-input root か」と `data-index` から引き直す。
//!
//! # `docs/policy/intentional-non-adoption.md` §3.25 との整理
//!
//! kind に応じた文字種の事前判定と大文字そろえは入力補助であり、データ検証ではない。
//! 値の正しさの検証はアプリ側の責務のまま。規則は `PinInputKind::{normalize_char,
//! is_valid_char}` を直接呼び、複製しない。
//!
//! # セキュリティ不変条件
//!
//! - `input.value` は先頭 2 文字までしか読まない。`data-index` は `usize` へパースし桁数未満を
//!   検査する（fail-closed）。`data-hydrate-kind` が解釈不能なら改ざんとみなして何もしない。
//! - リスナーは配線 1 回につき 2 個（`Closure::forget` 2 回）で固定する。
//! - PIN/OTP は秘密値のため、値・payload を console やエラーに出さない。

use crate::events::ActionRef;
use fandhe_frontend_headless_ui::pin_input::PinInputKind;

/// PinInput の `data-scope` 属性値。
pub const PIN_INPUT_SCOPE: &str = "pin-input";
/// 桁 input パーツの `data-part` 属性値。
pub const INPUT_PART: &str = "input";
/// Root パーツの `data-part` 属性値。
pub const ROOT_PART: &str = "root";
/// 1 文字入力を状態機械へ渡すアクション名（`PinInput::decode_action`）。
pub const ACTION_INPUT: &str = "input";
/// 桁へのフォーカス同期アクション名（`PinInput::decode_action`）。
pub const ACTION_FOCUS: &str = "focus";

/// `input` イベント後の `input.value` の分類（先頭 2 文字だけを読んだ結果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValue {
    /// 空文字（削除系。#3835 の担当のため何もしない）。
    Empty,
    /// ちょうど 1 文字。
    Single(char),
    /// 2 文字以上（autofill・貼り付け。#3835 の担当のため何もしない）。
    Multi,
}

/// `data-hydrate-kind` の解決結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindResolution {
    /// 解釈できた kind（規則は headless-ui を正とする）。
    Known(PinInputKind),
    /// 属性が無く kind が分からない（ASCII 英数字の上位集合で事前判定する）。
    Unknown,
    /// 属性はあるが解釈できない（改ざんとみなして何もしない）。
    Tampered,
}

/// `data-index` の生文字列を桁数未満の `usize` へパースする（それ以外は `None`）。
#[must_use]
pub fn parse_digit_index(raw: Option<&str>, count: usize) -> Option<usize> {
    let index = raw?.parse::<usize>().ok()?;
    (index < count).then_some(index)
}

/// `input.value` を先頭 2 文字だけ読んで分類する（巨大な値でも確保量は増えない）。
#[must_use]
pub fn classify_input_value(value: &str) -> InputValue {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (None, _) => InputValue::Empty,
        (Some(c), None) => InputValue::Single(c),
        (Some(_), Some(_)) => InputValue::Multi,
    }
}

/// Root の `data-hydrate-kind` 属性値を [`KindResolution`] へ解決する。
#[must_use]
pub fn resolve_kind(hydrate_kind: Option<&str>) -> KindResolution {
    match hydrate_kind {
        None => KindResolution::Unknown,
        Some(raw) => {
            PinInputKind::parse(raw).map_or(KindResolution::Tampered, KindResolution::Known)
        }
    }
}

/// 事前フィルタ。dispatch してよい場合は正規化後の文字、そうでなければ `None`。
///
/// 状態機械が受理しうる文字はどの kind でも ASCII 英数字だけなので、kind 不明のときも
/// ASCII 英数字以外は弾く（多層防御。最終判定は状態機械）。
#[must_use]
pub fn prefilter_char(kind: KindResolution, c: char) -> Option<char> {
    match kind {
        KindResolution::Known(k) => {
            let normalized = k.normalize_char(c);
            k.is_valid_char(normalized).then_some(normalized)
        }
        KindResolution::Unknown => c.is_ascii_alphanumeric().then_some(c),
        KindResolution::Tampered => None,
    }
}

/// 状態機械が入力を受理したかの事後判定。再描画後の桁 input のライブ `value` が
/// 入力文字と（大文字小文字を除いて）一致すれば受理とみなす。
///
/// `data-hydrate-kind` が無い通常描画（`KindResolution::Unknown`）では事前フィルタが
/// 状態機械の kind を知れないため、dispatch の戻り値（復号できたか）ではなく、
/// 状態機械の結果が反映された DOM のライブ値で受理を確かめる。
#[must_use]
pub fn input_accepted(rendered_value: Option<&str>, c: char) -> bool {
    let mut chars = rendered_value.unwrap_or("").chars();
    matches!((chars.next(), chars.next()), (Some(r), None) if r.eq_ignore_ascii_case(&c))
}

/// 次にフォーカスする桁。最終桁では留まる。`count == 0` なら `None`。
#[must_use]
pub fn next_focus_index(index: usize, count: usize) -> Option<usize> {
    count
        .checked_sub(1)
        .map(|last| index.saturating_add(1).min(last))
}

/// 1 文字入力で dispatch する列（`focus` → `input` の順）。
#[must_use]
pub fn input_dispatches(index: usize, c: char) -> [ActionRef; 2] {
    [
        ActionRef {
            action: ACTION_FOCUS.to_string(),
            payload: index.to_string(),
        },
        ActionRef {
            action: ACTION_INPUT.to_string(),
            payload: c.to_string(),
        },
    ]
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        classify_input_value, input_accepted, input_dispatches, next_focus_index,
        parse_digit_index, prefilter_char, resolve_kind, InputValue, KindResolution, ACTION_FOCUS,
        INPUT_PART, PIN_INPUT_SCOPE, ROOT_PART,
    };
    use crate::dom::has_disabled_ancestor;
    use crate::events::{ActionRef, ACTION_INPUT_ATTR};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, InputEvent};

    const INPUT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="input"]"#;
    const ROOT_SELECTOR: &str = r#"[data-scope="pin-input"][data-part="root"]"#;

    thread_local! {
        /// フォーカス復元中の `focusin` 再入を抑止するフラグ（wasm は単一スレッド）。
        static REFOCUSING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    type SharedOnAction<F> = std::rc::Rc<std::cell::RefCell<F>>;

    fn is_part(element: &Element, part: &str) -> bool {
        element.get_attribute("data-scope").as_deref() == Some(PIN_INPUT_SCOPE)
            && element.get_attribute("data-part").as_deref() == Some(part)
    }

    /// 最寄りの PinInput Root を探す。
    fn find_instance_root(start: &Element) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if is_part(&element, ROOT_PART) {
                return Some(element);
            }
            current = element.parent_element();
        }
        None
    }

    /// `data-readonly` は最寄りの PinInput Root で打ち切る（インスタンス内に限る、PR #1982）。
    fn has_readonly_ancestor(start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-readonly") {
                return true;
            }
            if is_part(&element, ROOT_PART) {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    fn instance_inputs(instance_root: &Element) -> Vec<Element> {
        let Ok(list) = instance_root.query_selector_all(INPUT_SELECTOR) else {
            return Vec::new();
        };
        (0..list.length())
            .filter_map(|i| list.item(i)?.dyn_into::<Element>().ok())
            .collect()
    }

    /// 配線 root 配下（root 自身を含む）の PinInput Root を文書順に列挙する。
    ///
    /// `query_selector_all` は検索元自身を含まないため、配線 root 自身が PinInput Root の
    /// 場合は先頭へ加える（`Runtime` が Root 要素そのものを渡す構成への対応）。
    fn instance_roots(root: &Element) -> Vec<Element> {
        let mut roots = Vec::new();
        if is_part(root, ROOT_PART) {
            roots.push(root.clone());
        }
        if let Ok(list) = root.query_selector_all(ROOT_SELECTOR) {
            roots.extend(
                (0..list.length()).filter_map(|i| list.item(i)?.dyn_into::<Element>().ok()),
            );
        }
        roots
    }

    /// 配線 root 配下で `instance_root` が何番目の PinInput Root か。
    fn root_ordinal(root: &Element, instance_root: &Element) -> Option<u32> {
        instance_roots(root)
            .iter()
            .position(|e| e == instance_root)
            .and_then(|i| u32::try_from(i).ok())
    }

    /// 再描画で要素が差し替わっても追えるよう、順序と `data-index` から桁 input を引き直す。
    fn resolve_input(root: &Element, ordinal: u32, index: usize) -> Option<HtmlInputElement> {
        let instance_root = instance_roots(root).into_iter().nth(ordinal as usize)?;
        instance_inputs(&instance_root)
            .into_iter()
            .find(|e| {
                parse_digit_index(e.get_attribute("data-index").as_deref(), usize::MAX)
                    == Some(index)
            })?
            .dyn_into::<HtmlInputElement>()
            .ok()
    }

    /// 桁 input 以外・操作不能な要素なら `None`。戻り値は (桁 input, 最寄りの Root)。
    fn target_digit(root: &Element, event: &Event) -> Option<(Element, Element)> {
        let target = event.target()?.dyn_into::<Element>().ok()?;
        if !is_part(&target, INPUT_PART) {
            return None;
        }
        if has_disabled_ancestor(root, &target) || has_readonly_ancestor(&target) {
            return None;
        }
        let instance_root = find_instance_root(&target)?;
        Some((target, instance_root))
    }

    /// `input`（capture）/ `focusin` を `root` へ 1 回ずつ登録する（`Closure::forget` は計 2 回）。
    ///
    /// `on_action` は dispatch できたとき `true` を返す。フォーカス移動は `on_action` から
    /// 戻った後（`component` の借用解放後）に行う。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_pin_input_events(
        root: Element,
        on_action: impl FnMut(ActionRef) -> bool + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        let input_root = root.clone();
        let input_on_action = on_action.clone();
        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_input(&input_root, &event, &input_on_action);
        });
        // capture で登録し、`events::wire_events` の bubble `input` dispatch より前に
        // 大文字そろえ・値の復元を済ませる（正規化前の値が dispatch されるのを防ぐ）。
        root.add_event_listener_with_callback_and_bool(
            "input",
            input_closure.as_ref().unchecked_ref(),
            true,
        )?;
        input_closure.forget();

        let focus_root = root.clone();
        let focusin_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focusin(&focus_root, &event, &on_action);
        });
        root.add_event_listener_with_callback("focusin", focusin_closure.as_ref().unchecked_ref())?;
        focusin_closure.forget();
        Ok(())
    }

    fn call<F: FnMut(ActionRef) -> bool>(on_action: &SharedOnAction<F>, action: ActionRef) -> bool {
        on_action
            .try_borrow_mut()
            .map(|mut f| (f)(action))
            .unwrap_or(false)
    }

    fn handle_input<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
    ) {
        // IME 変換中は確定前の文字列を扱わない（fail-closed）。
        if event
            .dyn_ref::<InputEvent>()
            .is_some_and(InputEvent::is_composing)
        {
            return;
        }
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
            return;
        };
        let InputValue::Single(c) = classify_input_value(&input.value()) else {
            return;
        };
        let count = instance_inputs(&instance_root).len();
        let Some(index) = parse_digit_index(target.get_attribute("data-index").as_deref(), count)
        else {
            return;
        };
        let kind = resolve_kind(instance_root.get_attribute("data-hydrate-kind").as_deref());
        if kind == KindResolution::Tampered {
            return;
        }
        let Some(ordinal) = root_ordinal(root, &instance_root) else {
            return;
        };
        let Some(normalized) = prefilter_char(kind, c) else {
            // 状態機械へは何も送らないため、DOM を状態側の旧値（描画済み value 属性）へ戻す。
            let restored = input.default_value();
            if restored.chars().count() <= 1 {
                input.set_value(&restored);
            } else {
                input.set_value("");
            }
            return;
        };
        if normalized != c {
            input.set_value(&normalized.to_string());
        }
        // `data-action-input` がある桁は `events::wire_events` が dispatch を担う。
        // 所有者判定は `events::attribute_input_action` と同じ `closest` 探索・root 境界で行う
        // （祖先の Root やラッパーに属性があっても bubble 側が dispatch するため）。
        if has_input_action_owner(root, &target) {
            // bubble 側の dispatch と再描画が済んでから確認・フォーカス移動する
            // （先に focus() すると focusin の "focus" dispatch で状態機械の focused が
            // 次桁へ移り、後続の "input" が次桁へ書かれる）。
            let deferred_root = root.clone();
            let cb = Closure::once_into_js(move || {
                finish_input(&deferred_root, ordinal, index, count, normalized);
            });
            let scheduled = web_sys::window().is_some_and(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 0)
                    .is_ok()
            });
            if !scheduled {
                // スケジュールできないときは fail-closed（旧値へ戻して終える）。
                restore_input(root, ordinal, index);
            }
            return;
        }
        let [focus, value] = input_dispatches(index, normalized);
        call(on_action, focus);
        if !call(on_action, value) {
            restore_input(root, ordinal, index);
            return;
        }
        finish_input(root, ordinal, index, count, normalized);
    }

    /// `data-action-input` を持つ最寄りの祖先（自身を含む）が配線 root 内にあるか。
    fn has_input_action_owner(root: &Element, target: &Element) -> bool {
        let selector = format!("[{ACTION_INPUT_ATTR}]");
        target
            .closest(&selector)
            .ok()
            .flatten()
            .is_some_and(|matched| root.contains(Some(&matched)))
    }

    /// 桁 input の DOM 値を描画済み value 属性（無ければ空）へ戻す。
    fn restore_input(root: &Element, ordinal: u32, index: usize) {
        if let Some(el) = resolve_input(root, ordinal, index) {
            let restored = el.default_value();
            if restored.chars().count() <= 1 {
                el.set_value(&restored);
            } else {
                el.set_value("");
            }
        }
    }

    /// dispatch と再描画の完了後に、ライブの `value`（再描画で要素が差し替わっていれば
    /// 状態から描画し直された値）が入力文字と一致するかを確かめてから次桁へフォーカスを移す。
    /// 一致しなければ（再描画で旧値へ戻された）旧値へ戻しフォーカスは動かさない。
    /// 入力で更新されるのは `value` プロパティで `value` 属性ではないため、属性は見ない。
    fn finish_input(root: &Element, ordinal: u32, index: usize, count: usize, c: char) {
        let Some(el) = resolve_input(root, ordinal, index) else {
            return;
        };
        if !input_accepted(Some(el.value().as_str()), c) {
            restore_input(root, ordinal, index);
            return;
        }
        if let Some(next) = next_focus_index(index, count) {
            if let Some(n) = resolve_input(root, ordinal, next) {
                let _ = n.focus();
            }
        }
    }

    fn handle_focusin<F: FnMut(ActionRef) -> bool>(
        root: &Element,
        event: &Event,
        on_action: &SharedOnAction<F>,
    ) {
        let Some((target, instance_root)) = target_digit(root, event) else {
            return;
        };
        let count = instance_inputs(&instance_root).len();
        let Some(index) = parse_digit_index(target.get_attribute("data-index").as_deref(), count)
        else {
            return;
        };
        if REFOCUSING.with(std::cell::Cell::get) {
            return;
        }
        let Some(ordinal) = root_ordinal(root, &instance_root) else {
            return;
        };
        let dispatched = call(
            on_action,
            ActionRef {
                action: ACTION_FOCUS.to_string(),
                payload: index.to_string(),
            },
        );
        if !dispatched {
            return;
        }
        // 再描画（構造フォールバック）で桁 input が差し替わるとフォーカスを失うため、
        // 引き直した要素へフォーカスを戻す。`focus()` の `focusin` 再入は抑止する。
        let Some(el) = resolve_input(root, ordinal, index) else {
            return;
        };
        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());
        if active.as_ref() != Some(el.unchecked_ref::<Element>()) {
            REFOCUSING.with(|f| f.set(true));
            let _ = el.focus();
            REFOCUSING.with(|f| f.set(false));
        }
    }

    /// [`wire_pin_input_events`] に `Component` の dispatch と DOM 反映を接続する。
    ///
    /// dispatch できたら `on_update` で再描画し `true` を返す。`Runtime::mount`/`hydrate`
    /// から `Runtime::wire_pin_input` 経由で呼ばれる。
    ///
    /// # Errors
    ///
    /// [`wire_pin_input_events`] の失敗を伝播する。
    pub fn wire_pin_input_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: fandhe_frontend_interactive::Component + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
        let wired_root = root.clone();
        wire_pin_input_events(root, move |action_ref: ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return false;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return false;
            }
            if let Ok(mut cb) = on_update.try_borrow_mut() {
                (cb)(&state, &wired_root);
            }
            true
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_pin_input_component, wire_pin_input_events};
