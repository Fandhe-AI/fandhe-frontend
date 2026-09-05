//! NumberInput（`fandhe-frontend-headless-ui` `number_input` モジュール）の
//! keydown → dispatch 配線（イシュー #1613、PR #1881 codex-review P1 是正）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/number_input.rs` は Root/Label/Control/Input/
//! IncrementTrigger/DecrementTrigger/ValueText の 7 anatomy パーツと
//! `"increment"`/`"decrement"`/`"set"`/`"clear"`/`"home"`/`"end"` dispatch
//! 契約を提供する一方、実際に `input`（`role="spinbutton"`）上のキー入力を
//! dispatch へ接続する DOM 配線は同モジュール冒頭 rustdoc が「クライアント
//! ランタイム側の後続責務」と申し送っていた。本モジュールがその配線を
//! 実装する。
//!
//! この申し送り文言（「本イシューでも新設しない」）は、同時に参考サイト
//! 突合された [`crate::angle_slider`]（イシュー #1601、Arrow キーのみ配線
//! 済み）・`crate::keynav`（Combobox、イシュー #1071）が実際には自身の
//! イシュー内で keydown 配線を新設した実績と矛盾しており、PR #1881
//! codex-review（P1）が「`crates/docs-site/src/primitive_specs/forms_b.rs`
//! の keyboard 一覧が対応済みとして表示する ArrowUp/ArrowDown/Home/End/
//! Enter がいずれも実配線を持たず操作不能」と指摘した。本モジュールは
//! その是正として keydown 配線を新設する（headless-ui 側の rustdoc も
//! 追随して更新済み）。
//!
//! # 設計（`angle_slider.rs`/`headless_signature_pad.rs` と同型の 2 層構成）
//!
//! - 純粋ロジック層（[`action_for_key`]）は web-sys に依存せず、native の
//!   `cargo test` で決定的に検証できる。
//! - 配線層（[`wire_number_input_events`]/[`wire_number_input_component`]）
//!   のみ `#[cfg(target_arch = "wasm32")]` でゲートする。
//!
//! `crate::headless::MAPPING_TABLE`（クリック委譲の静的マッピング表）へは
//! 乗せない。keydown はクリック（`click` イベント）とは別種別であり、
//! [`crate::angle_slider`]/[`crate::splitter`] と同じく独立配線モジュールと
//! して切り出す。
//!
//! # dispatch とアクションの対応
//!
//! | キー | アクション | payload |
//! |---|---|---|
//! | `ArrowUp` | （同期）`"set"` → `"increment"` | `input.value` → なし |
//! | `ArrowDown` | （同期）`"set"` → `"decrement"` | `input.value` → なし |
//! | `Home` | `"home"` | なし |
//! | `End` | `"end"` | なし |
//! | `Enter`（`input.value` が空でない） | `"set"` | `input` 要素の現在の `value`（未確定のタイプ中文字列） |
//! | `Enter`（`input.value` が trim 後空文字） | `"clear"` | なし |
//!
//! `ArrowUp`/`ArrowDown` は、キャレット確定前にタイプ中の `input.value` が
//! 状態値と食い違っているケース（例: 状態値 5 のまま入力欄を 8 に書き換えて
//! ArrowUp）で編集前の状態値を基準に増減すると実利用者の目に見える表示値と
//! 矛盾する（PR #1881 codex-review P1 是正その 1）。これを避けるため、
//! 増減アクションの **直前** に `input.value` を `"set"` として同期
//! dispatch してから増減する（1 回のキー操作で 2 アクションを dispatch
//! する）。`input.value` が数値としてパース不能・非有限な場合、`"set"` は
//! [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//! が no-op（`None`）として fail-closed に無視するため、増減は編集前の
//! 状態値のまま行われる（「不正な入力は破棄し状態値基準で増減する」契約）。
//! `Home`/`End` は同期を行わない（`min`/`max` への絶対設定であり、タイプ中
//! の値に依存しないため元々矛盾が生じない）。
//!
//! `Enter` は、`input.value` を trim した結果が空文字列であれば未入力状態
//! （`NumberInputAction::Clear`）へ、それ以外は従来どおり `"set"` へ分岐
//! する（PR #1881 codex-review P1 是正その 2。空欄確定時に旧値が残留する
//! 不具合の是正）。
//!
//! `"set"` の payload はキャレット確定前のテキストそのものであり、
//! [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//! が改めて `str::parse::<f64>()` + 有限性検証で fail-closed に扱う
//! （不正な文字列は no-op、多層防御）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **修飾キー（Shift/Alt/Ctrl+Arrow）による step 倍率**:
//!   `fandhe_frontend_headless_ui::number_input::NumberInput` の状態機械
//!   自体が倍率 API を持たない（`crates/headless-ui/src/number_input.rs`
//!   モジュール doc「非追随」節）ため、本モジュールも対応しない。
//! - **IncrementTrigger/DecrementTrigger ボタンのクリック配線**:
//!   `crate::headless::MAPPING_TABLE` に `("number-input", "increment-trigger")`
//!   相当の行が存在せず、マウスクリックは本 PR の対象外（イシュー #1613 の
//!   スコープは「キーボード操作」であり、ボタンクリックの配線欠落は別事象
//!   として追跡する）。
//!
//! # セキュリティ不変条件
//!
//! - dispatch payload（`"set"` の文字列）は
//!   [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
//!   が改めて厳密パース・有限性検証する（本モジュールは payload 文字列を
//!   組み立てるのみ、多層防御）。
//! - `data-disabled` **または** `data-readonly` を持つ input/祖先パーツ上の
//!   keydown は no-op（[`has_noninteractive_ancestor`]、`crate::angle_slider`
//!   の `has_noninteractive_ancestor` と同型の fail-closed 判定）。
//! - DOM 反映は `set_attribute`/`get_attribute`/`value` プロパティ読み取りの
//!   みで行い、HTML 文字列を一切組み立てない（REQ-1）。属性名・イベント名は
//!   すべて `&'static str` リテラル。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。

use crate::keynav::Modifiers;

/// dispatch アクション名 `"increment"`。
pub const ACTION_INCREMENT: &str = "increment";
/// dispatch アクション名 `"decrement"`。
pub const ACTION_DECREMENT: &str = "decrement";
/// dispatch アクション名 `"home"`。
pub const ACTION_HOME: &str = "home";
/// dispatch アクション名 `"end"`。
pub const ACTION_END: &str = "end";
/// dispatch アクション名 `"set"`。
pub const ACTION_SET: &str = "set";
/// dispatch アクション名 `"clear"`（Enter 確定時に `input.value` が
/// trim 後空文字の場合に使う、PR #1881 codex-review P1 是正）。
pub const ACTION_CLEAR: &str = "clear";

/// keydown から決定される操作種別（純粋層、web-sys 非依存）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// `step` 分の増加（[`ACTION_INCREMENT`]、payload なし）。
    Increment,
    /// `step` 分の減少（[`ACTION_DECREMENT`]、payload なし）。
    Decrement,
    /// `min` へ設定（[`ACTION_HOME`]、payload なし）。
    Home,
    /// `max` へ設定（[`ACTION_END`]、payload なし）。
    End,
    /// タイプ中の値を確定する（[`ACTION_SET`]、payload は `input.value`）。
    Set,
}

impl KeyAction {
    /// dispatch アクション名（`&'static str`）。
    #[must_use]
    pub fn action_name(self) -> &'static str {
        match self {
            KeyAction::Increment => ACTION_INCREMENT,
            KeyAction::Decrement => ACTION_DECREMENT,
            KeyAction::Home => ACTION_HOME,
            KeyAction::End => ACTION_END,
            KeyAction::Set => ACTION_SET,
        }
    }
}

/// キー名 + 修飾キーから [`KeyAction`] を決定する純粋関数（DOM 非依存、
/// native `cargo test` で検証可能）。
///
/// 修飾キー（Ctrl/Alt/Meta）付きは常に `None`（[`Modifiers::any`]、
/// `crate::keynav`/`crate::angle_slider` と同じ安全側判断。Shift+Arrow の
/// step 倍率は状態機械側に API がないため対応しない、モジュール冒頭 doc
/// 「スコープ外」節参照）。それ以外の未知キーも `None`（no-op）。
#[must_use]
pub fn action_for_key(key: &str, modifiers: Modifiers) -> Option<KeyAction> {
    if modifiers.any() {
        return None;
    }
    match key {
        "ArrowUp" => Some(KeyAction::Increment),
        "ArrowDown" => Some(KeyAction::Decrement),
        "Home" => Some(KeyAction::Home),
        "End" => Some(KeyAction::End),
        "Enter" => Some(KeyAction::Set),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`angle_slider.rs`/`headless_signature_pad.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::action_for_key;
    use crate::events::ActionRef;
    use crate::keynav::Modifiers;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, KeyboardEvent};

    /// NumberInput の `data-scope` 属性値（`fandhe_frontend_headless_ui::number_input`
    /// の `ANATOMY` と一致、`crates/headless-ui/src/number_input.rs` 参照）。
    const NUMBER_INPUT_SCOPE: &str = "number-input";
    /// NumberInput Input パーツの `data-part` 属性値。
    const INPUT_PART: &str = "input";

    /// `event` から [`Modifiers`] を抽出する（`crate::keynav::modifiers_of`
    /// と同型の判断だが `pub(crate)` ではないためここで個別定義する）。
    fn modifiers_of(event: &KeyboardEvent) -> Modifiers {
        Modifiers {
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        }
    }

    /// `start` から `root` まで祖先方向を辿り、`data-disabled` **または**
    /// `data-readonly` を持つ要素が 1 つでもあれば `true` を返す
    /// （`crate::angle_slider::wiring::has_noninteractive_ancestor` と同型の
    /// fail-closed 判定。`root`/`control`/`input` はいずれも disabled/
    /// readonly 時に該当 data-* 属性を持つ、`crates/headless-ui/src/
    /// number_input.rs` の `root`/`control`/`input` 参照）。
    fn has_noninteractive_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("data-readonly") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `root` 配下の NumberInput Input パーツへ keydown 配線を 1 回だけ
    /// 登録する（マウント時 1 回契約、`angle_slider.rs`/`splitter.rs` と
    /// 同型）。
    ///
    /// `on_action` は `"increment"`/`"decrement"`/`"home"`/`"end"`/`"set"` の
    /// dispatch 依頼を呼び出し側へ渡すのみで、状態更新・DOM 反映は行わない
    /// （`headless_clipboard::wire_clipboard_events` と同じ責務分離）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_number_input_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        let keydown_root = root.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &on_action);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        Ok(())
    }

    /// keydown: Input（`data-scope="number-input"` `data-part="input"`）上
    /// でのみ反応する（[`action_for_key`]、モジュール冒頭 doc 「dispatch と
    /// アクションの対応」節参照）。
    fn handle_keydown(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !root.contains(Some(target_element)) {
            return;
        }

        let scope = target_element.get_attribute("data-scope");
        let part = target_element.get_attribute("data-part");
        if scope.as_deref() != Some(NUMBER_INPUT_SCOPE) || part.as_deref() != Some(INPUT_PART) {
            return;
        }
        if has_noninteractive_ancestor(root, target_element) {
            return;
        }

        let modifiers = modifiers_of(keyboard_event);
        let Some(key_action) = action_for_key(&keyboard_event.key(), modifiers) else {
            return;
        };

        // `input.value` はキャレット確定前のタイプ中文字列であり、
        // Increment/Decrement/Set のいずれも参照し得るため一度だけ読む
        // （モジュール冒頭 doc「dispatch とアクションの対応」節参照）。
        let raw_value = target_element
            .clone()
            .dyn_into::<HtmlInputElement>()
            .map(|input| input.value())
            .unwrap_or_default();

        keyboard_event.prevent_default();

        let dispatch_one = |action: &'static str, payload: String| {
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(ActionRef {
                    action: action.to_string(),
                    payload,
                });
            }
        };

        match key_action {
            // PR #1881 codex-review P1 是正その 1: 増減の直前にタイプ中の
            // `input.value` を `"set"` として同期 dispatch する。値が
            // パース不能・非有限な場合は `decode_action` が no-op として
            // 無視するため、増減は編集前の状態値のまま安全に行われる
            // （fail-closed、モジュール冒頭 doc 参照）。
            super::KeyAction::Increment => {
                dispatch_one(super::ACTION_SET, raw_value);
                dispatch_one(super::ACTION_INCREMENT, String::new());
            }
            super::KeyAction::Decrement => {
                dispatch_one(super::ACTION_SET, raw_value);
                dispatch_one(super::ACTION_DECREMENT, String::new());
            }
            super::KeyAction::Home => {
                dispatch_one(super::ACTION_HOME, String::new());
            }
            super::KeyAction::End => {
                dispatch_one(super::ACTION_END, String::new());
            }
            // PR #1881 codex-review P1 是正その 2: trim 後空文字は
            // `"set"`（`decode_action` が空文字列パース失敗で no-op にし
            // 旧値が残留する）ではなく `"clear"` へ分岐し、未入力状態へ
            // 正しく同期する。
            super::KeyAction::Set => {
                if raw_value.trim().is_empty() {
                    dispatch_one(super::ACTION_CLEAR, String::new());
                } else {
                    dispatch_one(super::ACTION_SET, raw_value);
                }
            }
        }
    }

    /// [`wire_number_input_events`] の keydown dispatch を
    /// `fandhe_frontend_interactive::dispatch` へ接続し、成功時のみ
    /// `on_update` を呼ぶ利便関数（`crate::headless::wire_headless_component`/
    /// `headless_signature_pad::wire_signature_pad_component` と同型）。
    ///
    /// `on_update` は呼び出し側（`crate::lib::Runtime::wire_number_input`）が
    /// 束縛点更新（`BindingTable::apply_dirty`・keyed list 差し替え）を渡す
    /// 想定であり、本関数自体は DOM 反映を行わない。
    ///
    /// # Errors
    ///
    /// [`wire_number_input_events`]（`add_event_listener_with_callback`）の
    /// 失敗を伝播する。
    pub fn wire_number_input_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: fandhe_frontend_interactive::Component + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
        let wired_root = root.clone();

        wire_number_input_events(root, move |action_ref: ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Ok(mut cb) = on_update.try_borrow_mut() {
                (cb)(&state, &wired_root);
            }
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_number_input_component, wire_number_input_events};

#[cfg(test)]
mod tests {
    use super::*;

    fn mods() -> Modifiers {
        Modifiers::default()
    }

    fn mods_ctrl() -> Modifiers {
        Modifiers {
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    // --- action_for_key ---

    #[test]
    fn arrow_up_is_increment() {
        assert_eq!(
            action_for_key("ArrowUp", mods()),
            Some(KeyAction::Increment)
        );
    }

    #[test]
    fn arrow_down_is_decrement() {
        assert_eq!(
            action_for_key("ArrowDown", mods()),
            Some(KeyAction::Decrement)
        );
    }

    #[test]
    fn home_sets_to_min() {
        assert_eq!(action_for_key("Home", mods()), Some(KeyAction::Home));
    }

    #[test]
    fn end_sets_to_max() {
        assert_eq!(action_for_key("End", mods()), Some(KeyAction::End));
    }

    #[test]
    fn enter_commits_typed_value() {
        assert_eq!(action_for_key("Enter", mods()), Some(KeyAction::Set));
    }

    #[test]
    fn unknown_key_is_noop() {
        assert_eq!(action_for_key("a", mods()), None);
        assert_eq!(action_for_key("PageUp", mods()), None);
        assert_eq!(action_for_key("Tab", mods()), None);
        assert_eq!(action_for_key("ArrowLeft", mods()), None);
        assert_eq!(action_for_key("ArrowRight", mods()), None);
    }

    #[test]
    fn modifier_keys_are_noop() {
        assert_eq!(action_for_key("ArrowUp", mods_ctrl()), None);
        assert_eq!(action_for_key("Enter", mods_ctrl()), None);
        assert_eq!(
            action_for_key(
                "Home",
                Modifiers {
                    ctrl: false,
                    alt: true,
                    meta: false
                }
            ),
            None
        );
        assert_eq!(
            action_for_key(
                "End",
                Modifiers {
                    ctrl: false,
                    alt: false,
                    meta: true
                }
            ),
            None
        );
    }

    // --- KeyAction::action_name ---

    #[test]
    fn action_name_matches_decode_action_contract() {
        assert_eq!(KeyAction::Increment.action_name(), "increment");
        assert_eq!(KeyAction::Decrement.action_name(), "decrement");
        assert_eq!(KeyAction::Home.action_name(), "home");
        assert_eq!(KeyAction::End.action_name(), "end");
        assert_eq!(KeyAction::Set.action_name(), "set");
    }

    // --- ACTION_CLEAR（PR #1881 codex-review P1 是正その 2） ---

    /// [`ACTION_CLEAR`] は
    /// [`fandhe_frontend_headless_ui::number_input::NumberInput::decode_action`]
    /// が受理する `"clear"` と完全一致すること（配線層は文字列リテラルを
    /// 個別に書かず本定数のみを参照する契約の固定）。
    #[test]
    fn action_clear_matches_decode_action_contract() {
        assert_eq!(ACTION_CLEAR, "clear");
    }
}
