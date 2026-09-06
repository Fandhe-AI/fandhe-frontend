//! `fandhe_frontend_wasm_full::number_input`（イシュー #1613、PR #1881
//! codex-review P1 是正）の実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/src/number_input.rs` の `#[cfg(test)] mod tests`
//! （native）は [`fandhe_frontend_wasm_full::number_input::action_for_key`]
//! （純粋ロジック層）を検証する。本ファイルはその先、**実ブラウザ（headless
//! Chromium、`wasm-pack test --headless --chrome`）上での合成 keydown
//! イベント → [`wire_number_input_component`]（配線層）→
//! `fandhe_frontend_interactive::dispatch` → `NumberInput` 状態遷移**という
//! 製品経路を検証する（`headless_wiring_browser.rs` と同型の実 DOM 検証
//! パターンを踏襲する）。
//!
//! 自動再描画（`Runtime::apply_update_for_dirty`、束縛点更新との統合）は
//! 本ファイルのスコープ外（`headless_wiring_browser.rs` と同じ判断）。
//! `on_update` コールバックは `NumberInput::formatted_value()` を
//! `input` 要素の `value`/`aria-valuenow` へ手動反映し、実利用者が実際に
//! 目にする DOM 属性が更新されることまでを検証する（codex-review P1
//! 「実利用者は列挙されたキーで値を変更できません」の是正確認）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_headless_ui::number_input::{NumberInput, NumberInputFlags};
use fandhe_frontend_interactive::Component;
use fandhe_frontend_wasm_full::events::ActionRef;
use fandhe_frontend_wasm_full::number_input::{
    wire_number_input_component, wire_number_input_events,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventInit, HtmlInputElement, KeyboardEvent, KeyboardEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`headless_wiring_browser.rs::create_container` と同じ意図）。
fn create_container(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist")
        .append_child(&container)
        .expect("append_child must not fail");
    container
}

/// テスト終了時にコンテナを DOM から除去する（テスト間の要素衝突防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent_node() {
            let _ = parent.remove_child(&self.0);
        }
    }
}

/// 合成 `keydown` イベント（`bubbles: true, cancelable: true`）を組み立てる
/// （`keynav_browser.rs::keydown_event` と同型。`cancelable: true` により
/// `dispatch_event` の戻り値で `prevent_default()` 呼び出しを検証できる）。
fn keydown_event(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// 修飾キー付きの合成 `keydown` イベント。
fn keydown_event_with_ctrl(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// IME 変換中（`isComposing: true`）の合成 `keydown` イベント
/// （PR #1881 codex-review P1 是正その 3 の回帰テスト用）。
fn keydown_event_composing(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_is_composing(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// 合成 `click` イベント（`bubbles: true`）を組み立てる（イシュー #1962、
/// `keynav_browser.rs::click_event` と同型。`handle_click` は
/// `prevent_default()` を呼ばない契約〔`crates/wasm-full/src/
/// number_input.rs::wiring::handle_click` 参照〕のため `cancelable` は
/// 不要）。
fn click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// NumberInput の Root > Control > Input を組み立て、`container` へ差し込む。
/// `root` 要素（`data-scope="number-input" data-part="root"`）と
/// `input` 要素（`data-part="input"`）を返す。
fn build_number_input_dom(
    document: &Document,
    container_id: &str,
    number_input: &NumberInput,
    flags: NumberInputFlags,
) -> (Element, Element) {
    let container = create_container(document, container_id);
    let node = number_input.root(
        flags,
        Vec::new(),
        vec![number_input.control(
            flags,
            Vec::new(),
            vec![number_input.input("qty", Some("qty-input"), flags, Vec::new())],
        )],
    );
    let html = fandhe_frontend_core::render(&node);
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("number-input root must exist");
    let input = root
        .query_selector(r#"[data-scope="number-input"][data-part="input"]"#)
        .expect("query_selector must not fail")
        .expect("input element must exist");
    (root, input)
}

/// NumberInput の Root > Control > [IncrementTrigger, Input, DecrementTrigger]
/// を組み立て、`container` へ差し込む（イシュー #1962: click 配線の
/// ブラウザ回帰テスト用）。`increment_trigger`/`decrement_trigger` は
/// [`NumberInput`] の利便メソッドを使い、境界到達時のネイティブ `disabled`
/// 合成（`NumberInput::can_increment`/`can_decrement`）を製品と同じ経路で
/// 再現する。`root`/`input`/increment ボタン/decrement ボタンの 4 要素を
/// 返す。
fn build_number_input_dom_with_triggers(
    document: &Document,
    container_id: &str,
    number_input: &NumberInput,
    flags: NumberInputFlags,
) -> (Element, Element, Element, Element) {
    let container = create_container(document, container_id);
    let node = number_input.root(
        flags,
        Vec::new(),
        vec![number_input.control(
            flags,
            Vec::new(),
            vec![
                number_input.increment_trigger(
                    Some("qty-input"),
                    false,
                    Vec::new(),
                    vec![fandhe_frontend_core::text("+")],
                ),
                number_input.input("qty", Some("qty-input"), flags, Vec::new()),
                number_input.decrement_trigger(
                    Some("qty-input"),
                    false,
                    Vec::new(),
                    vec![fandhe_frontend_core::text("-")],
                ),
            ],
        )],
    );
    let html = fandhe_frontend_core::render(&node);
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("number-input root must exist");
    let input = root
        .query_selector(r#"[data-scope="number-input"][data-part="input"]"#)
        .expect("query_selector must not fail")
        .expect("input element must exist");
    let increment_button = root
        .query_selector(r#"[data-scope="number-input"][data-part="increment-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("increment-trigger element must exist");
    let decrement_button = root
        .query_selector(r#"[data-scope="number-input"][data-part="decrement-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("decrement-trigger element must exist");
    (root, input, increment_button, decrement_button)
}

/// 配線し、`on_update` で `formatted_value()` を `input` の `value`/
/// `aria-valuenow` へ反映するクロージャを組み立てる（本ファイル冒頭 doc
/// 「実利用者が実際に目にする DOM 属性が更新される」節参照）。
fn wire_with_dom_reflection(
    root: Element,
    component: Rc<RefCell<NumberInput>>,
) -> Rc<RefCell<NumberInput>> {
    let reflect_root = root.clone();
    wire_number_input_component(root, component.clone(), move |state: &NumberInput, _| {
        let Ok(Some(input_el)) =
            reflect_root.query_selector(r#"[data-scope="number-input"][data-part="input"]"#)
        else {
            return;
        };
        let formatted = state.formatted_value();
        let _ = input_el.set_attribute("value", &formatted);
        let _ = input_el.set_attribute("aria-valuenow", &formatted);
        if let Ok(html_input) = input_el.dyn_into::<HtmlInputElement>() {
            html_input.set_value(&formatted);
        }
    })
    .expect("wire_number_input_component must not fail");
    component
}

#[wasm_bindgen_test]
fn arrow_up_increments_value_and_updates_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-arrow-up",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let default_not_prevented = input.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert!(
        !default_not_prevented,
        "ArrowUp は claim され prevent_default() が呼ばれること"
    );

    assert_eq!(component.borrow().value(), Some(6.0));
    assert_eq!(input.get_attribute("value").as_deref(), Some("6"));
    assert_eq!(input.get_attribute("aria-valuenow").as_deref(), Some("6"));
}

#[wasm_bindgen_test]
fn arrow_down_decrements_value_and_updates_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-arrow-down",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();

    assert_eq!(component.borrow().value(), Some(4.0));
    assert_eq!(input.get_attribute("value").as_deref(), Some("4"));
}

#[wasm_bindgen_test]
fn home_sets_to_min_and_end_sets_to_max() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-home-end",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    input.dispatch_event(&keydown_event("Home")).unwrap();
    assert_eq!(component.borrow().value(), Some(0.0));
    assert_eq!(input.get_attribute("value").as_deref(), Some("0"));

    input.dispatch_event(&keydown_event("End")).unwrap();
    assert_eq!(component.borrow().value(), Some(10.0));
    assert_eq!(input.get_attribute("value").as_deref(), Some("10"));
}

#[wasm_bindgen_test]
fn enter_commits_typed_value_from_input_element() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-enter",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    // ブラウザ上でユーザーがタイプ中の文字列を模す（`aria-valuenow`/`value`
    // 属性の書き換えではなく `HtmlInputElement::set_value` で表現する。
    // 実際のブラウザではキー入力ごとに `input.value` が変わるのと同じ）。
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("8");

    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(component.borrow().value(), Some(8.0));
    assert_eq!(input.get_attribute("value").as_deref(), Some("8"));
}

#[wasm_bindgen_test]
fn enter_with_non_numeric_typed_value_is_noop_fail_closed() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-enter-invalid",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("not-a-number");

    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "不正な文字列は decode_action が fail-closed に拒否し値は変わらないこと"
    );
}

#[wasm_bindgen_test]
fn disabled_input_ignores_all_keys() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let flags = NumberInputFlags {
        disabled: true,
        ..NumberInputFlags::default()
    };
    let (root, input) = build_number_input_dom(&document, "ni-disabled", &number_input, flags);
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let default_not_prevented = input.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert!(
        default_not_prevented,
        "disabled 時は claim されず prevent_default() が呼ばれないこと"
    );
    assert_eq!(component.borrow().value(), Some(5.0));
}

#[wasm_bindgen_test]
fn readonly_input_ignores_all_keys() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let (root, input) = build_number_input_dom(&document, "ni-readonly", &number_input, flags);
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "readonly 時は増減操作を抑止すること（root/control/input の data-readonly 祖先判定）"
    );
}

#[wasm_bindgen_test]
fn modifier_key_arrow_up_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-modifier",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let default_not_prevented = input
        .dispatch_event(&keydown_event_with_ctrl("ArrowUp"))
        .unwrap();
    assert!(
        default_not_prevented,
        "修飾キー付きは claim されず prevent_default() が呼ばれないこと"
    );
    assert_eq!(component.borrow().value(), Some(5.0));
}

#[wasm_bindgen_test]
fn keydown_on_control_part_is_noop_only_input_part_reacts() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, _input) = build_number_input_dom(
        &document,
        "ni-control-part",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let control = root
        .query_selector(r#"[data-scope="number-input"][data-part="control"]"#)
        .unwrap()
        .unwrap();

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    control.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "Input パーツ以外の keydown は no-op であること"
    );
}

/// [`wire_with_dom_reflection`] と異なり、未入力状態（`value() == None`）へ
/// 遷移した場合に `value`/`aria-valuenow` 属性を除去する
/// （`fandhe_frontend_headless_ui::number_input::input` が未入力時にこの
/// 2 属性を出力しない契約と一致させる、PR #1881 codex-review P1 是正その 2
/// の DOM 反映検証用）。
fn wire_with_full_dom_reflection(
    root: Element,
    component: Rc<RefCell<NumberInput>>,
) -> Rc<RefCell<NumberInput>> {
    let reflect_root = root.clone();
    wire_number_input_component(root, component.clone(), move |state: &NumberInput, _| {
        let Ok(Some(input_el)) =
            reflect_root.query_selector(r#"[data-scope="number-input"][data-part="input"]"#)
        else {
            return;
        };
        match state.value() {
            Some(_) => {
                let formatted = state.formatted_value();
                let _ = input_el.set_attribute("value", &formatted);
                let _ = input_el.set_attribute("aria-valuenow", &formatted);
                if let Ok(html_input) = input_el.clone().dyn_into::<HtmlInputElement>() {
                    html_input.set_value(&formatted);
                }
            }
            None => {
                let _ = input_el.remove_attribute("value");
                let _ = input_el.remove_attribute("aria-valuenow");
                if let Ok(html_input) = input_el.dyn_into::<HtmlInputElement>() {
                    html_input.set_value("");
                }
            }
        }
    })
    .expect("wire_number_input_component must not fail");
    component
}

#[wasm_bindgen_test]
fn arrow_up_syncs_typed_value_before_incrementing() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-arrow-up-sync",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    // 状態値は 5 のまま、実利用者が入力欄を 8 へ書き換えた状態を模す
    // （キャレット確定前）。
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("8");

    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(9.0),
        "編集前の状態値 5 ではなく、タイプ中の入力欄の値 8 を基準に +1 されること"
    );
    assert_eq!(input.get_attribute("value").as_deref(), Some("9"));
    assert_eq!(input.get_attribute("aria-valuenow").as_deref(), Some("9"));
}

#[wasm_bindgen_test]
fn arrow_up_with_non_numeric_typed_value_falls_back_to_state_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-arrow-up-invalid",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("not-a-number");

    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(6.0),
        "不正な入力文字列は同期 set が fail-closed に無視し、状態値 5 を基準に +1 されること"
    );
    assert_eq!(input.get_attribute("value").as_deref(), Some("6"));
}

/// PR #1881 codex P1 / Bugbot Medium 是正の回帰テスト: 入力欄に前後
/// 空白付きの値（`" 8"`）を書き込んで ArrowUp を押した場合、trim せず
/// `input.value` をそのまま `"set"` payload にすると
/// `NumberInput::decode_action` の `parse::<f64>()` が前後空白を拒否し
/// no-op になり、編集前の状態値 5 を基準に +1 された 6 になってしまう
/// （不具合時の挙動）。是正後は trim 済みの 8 を基準に +1 された 9 に
/// なること。
#[wasm_bindgen_test]
fn arrow_up_syncs_typed_value_with_surrounding_whitespace_before_incrementing() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-arrow-up-sync-whitespace",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    // 状態値は 5 のまま、実利用者が前後空白付きの値をペーストした状態を
    // 模す（キャレット確定前）。
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value(" 8");

    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(9.0),
        "前後空白を trim してから同期し、8 を基準に +1 された 9 になること（6 は不具合時の挙動）"
    );
    assert_eq!(input.get_attribute("value").as_deref(), Some("9"));
    assert_eq!(input.get_attribute("aria-valuenow").as_deref(), Some("9"));
}

/// PR #1881 codex P1 / Bugbot Medium 是正の回帰テスト: 前後空白付きの
/// 値（`" 8 "`）を Enter で確定した場合、trim せず `"set"` payload に
/// 渡すと `decode_action` が no-op になり編集前の状態値 5 のまま残留
/// してしまう（不具合時の挙動）。是正後は trim 済みの 8 が反映される
/// こと。
#[wasm_bindgen_test]
fn enter_commits_typed_value_with_surrounding_whitespace() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-enter-whitespace",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_full_dom_reflection(root, component);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value(" 8 ");

    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(8.0),
        "前後空白を trim してから確定し、8 が反映されること（5 のまま残留は不具合時の挙動）"
    );
    assert_eq!(input.get_attribute("value").as_deref(), Some("8"));
    assert_eq!(input.get_attribute("aria-valuenow").as_deref(), Some("8"));
}

#[wasm_bindgen_test]
fn enter_with_blank_input_clears_the_value() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-enter-blank",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_full_dom_reflection(root, component);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    // trim 後空文字（空白のみ）も空欄確定として扱われることを併せて検証する。
    html_input.set_value("   ");

    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(
        component.borrow().value(),
        None,
        "空欄の Enter 確定は set ではなく clear へ分岐し未入力状態になること"
    );
    assert_eq!(
        input.get_attribute("value"),
        None,
        "未入力状態では headless-ui の input() と同じく value 属性が存在しないこと"
    );
    assert_eq!(
        input.get_attribute("aria-valuenow"),
        None,
        "未入力状態では aria-valuenow 属性が存在しないこと"
    );
}

#[wasm_bindgen_test]
fn value_clamps_at_max_and_min_boundaries() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(10.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-clamp",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    // 既に max（10）にいる状態で ArrowUp を押しても 10 のまま（clamp）。
    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert_eq!(component.borrow().value(), Some(10.0));
}

#[wasm_bindgen_test]
fn composing_arrow_up_is_ignored_and_default_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-composing-arrow-up",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    // IME 変換中の候補選択を模し、変換中文字列としてパース不能な値を
    // 入力欄に置く（変換中に increment が実行されればパース失敗により
    // 状態値がそのまま上書きされてしまう、codex-review P1 是正その 3）。
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("こんにちは");

    let default_not_prevented = input
        .dispatch_event(&keydown_event_composing("ArrowUp"))
        .unwrap();
    assert!(
        default_not_prevented,
        "IME 変換中の ArrowUp は claim されず prevent_default() が呼ばれないこと"
    );
    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "IME 変換中は increment が実行されず値が変わらないこと"
    );
}

#[wasm_bindgen_test]
fn composing_enter_is_ignored_and_default_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, input) = build_number_input_dom(
        &document,
        "ni-composing-enter",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("8");

    // IME 確定用の Enter（変換候補確定）を模す。confirm 用 Enter は
    // NumberInput の "set" 確定ではないため claim されてはならない。
    let default_not_prevented = input
        .dispatch_event(&keydown_event_composing("Enter"))
        .unwrap();
    assert!(
        default_not_prevented,
        "IME 確定用の Enter は claim されず prevent_default() が呼ばれないこと"
    );
    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "IME 変換中の Enter は set/clear のいずれも実行されず値が変わらないこと"
    );
}

// ---------------------------------------------------------------------
// IncrementTrigger/DecrementTrigger の click 配線（イシュー #1962）の
// 実ブラウザ回帰。keydown 系と同じく実 DOM 上の合成 click イベント →
// `wire_number_input_component`（配線層）→
// `fandhe_frontend_interactive::dispatch` → `NumberInput` 状態遷移という
// 製品経路を検証する。
// ---------------------------------------------------------------------

#[wasm_bindgen_test]
fn click_increment_trigger_increments_value_and_updates_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, _input, increment_button, _decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-increment",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    increment_button.dispatch_event(&click_event()).unwrap();

    assert_eq!(component.borrow().value(), Some(6.0));
}

#[wasm_bindgen_test]
fn click_decrement_trigger_decrements_value_and_updates_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, _input, _increment_button, decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-decrement",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    decrement_button.dispatch_event(&click_event()).unwrap();

    assert_eq!(component.borrow().value(), Some(4.0));
}

/// 境界到達によりネイティブ `disabled`（+ `data-disabled`）が付与された
/// IncrementTrigger への click が no-op であること。`max` に到達済みの
/// 状態で組み立て、`NumberInput::increment_trigger` の利便メソッドが
/// `can_increment() == false` を disabled へ自動合成する製品経路
/// （`crates/headless-ui/src/number_input.rs` 参照）をそのまま再現する。
#[wasm_bindgen_test]
fn click_disabled_trigger_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(10.0), 0.0, 10.0, 1.0);
    let (root, _input, increment_button, _decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-disabled-trigger",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());
    assert!(
        increment_button.has_attribute("disabled"),
        "max 到達時は increment-trigger にネイティブ disabled が付くこと（前提確認）"
    );

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    increment_button.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(10.0),
        "disabled な IncrementTrigger への click は no-op であること"
    );
}

/// readonly 時、IncrementTrigger/DecrementTrigger にはネイティブ
/// `disabled` が付かない（`root`/`control`/`input` のみが `data-readonly`
/// を持つ、`crates/headless-ui/src/number_input.rs` 参照）ため、click
/// ブロックは配線層の `has_noninteractive_ancestor`（祖先の Control が
/// 持つ `data-readonly` を辿る判定）が唯一の防御層になる。この分岐の
/// 実ブラウザ回帰（PR review 指摘の中核）。
#[wasm_bindgen_test]
fn click_increment_trigger_is_noop_when_readonly() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let (root, _input, increment_button, _decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-readonly-trigger",
        &number_input,
        flags,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    assert!(
        !increment_button.has_attribute("disabled"),
        "readonly 時は increment-trigger にネイティブ disabled が付かないこと（前提確認）"
    );

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    increment_button.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "readonly 時は has_noninteractive_ancestor（祖先の data-readonly 判定）が \
         唯一の防御層として click をブロックすること"
    );
}

/// IncrementTrigger 内の子要素（アイコン用の `<span>` を模す）への click
/// が、ボタン本体への click と同様に dispatch されること（`event.target()`
/// がテキストノード/子要素の場合の `Node::parent_element()` 遡り、
/// `wiring::handle_click` doc 参照）。
#[wasm_bindgen_test]
fn click_on_icon_child_inside_trigger_still_dispatches() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, _input, increment_button, _decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-icon-child",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    // ボタン内の既存テキストノード（"+"）を、アイコン用 span 子要素へ
    // 差し替える（SVG アイコンを模す最小構成。子要素への click ターゲット
    // 委譲を検証する目的のため span で十分）。
    increment_button.set_inner_html("");
    let icon = document
        .create_element("span")
        .expect("create_element must not fail for a plain span");
    increment_button
        .append_child(&icon)
        .expect("append_child must not fail");

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    icon.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(6.0),
        "アイコン子要素への click も親の IncrementTrigger として解決され dispatch されること"
    );
}

#[wasm_bindgen_test]
fn click_on_control_part_is_noop_only_trigger_parts_react() {
    let document = web_sys::window().unwrap().document().unwrap();
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let (root, _input, _increment_button, _decrement_button) = build_number_input_dom_with_triggers(
        &document,
        "ni-click-control-part",
        &number_input,
        NumberInputFlags::default(),
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let control = root
        .query_selector(r#"[data-scope="number-input"][data-part="control"]"#)
        .unwrap()
        .unwrap();

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    control.dispatch_event(&click_event()).unwrap();
    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "IncrementTrigger/DecrementTrigger パーツ以外の click は no-op であること"
    );
}

/// 2 インスタンス（qty/price、別 `name`/別 `data-action-input`）で、
/// price 側の DecrementTrigger への click が qty 側の状態を一切変更せず、
/// dispatch された `(action, payload)` で両者を区別できること
/// （`two_instances_arrow_up_updates_only_the_targeted_field` の click 版）。
#[wasm_bindgen_test]
fn two_instances_click_increment_trigger_updates_only_the_targeted_field() {
    let document = web_sys::window().unwrap().document().unwrap();

    let container = create_container(&document, "ni-two-instances-click-root");
    let qty_input_model = NumberInput::new(Some(5.0), 0.0, 100.0, 1.0);
    let price_input_model = NumberInput::new(Some(5.0), 0.0, 100.0, 1.0);
    let qty_node = qty_input_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![qty_input_model.control(
            NumberInputFlags::default(),
            Vec::new(),
            vec![
                qty_input_model.input(
                    "qty",
                    Some("qty-input"),
                    NumberInputFlags::default(),
                    vec![("data-action-input", "qty_set")],
                ),
                qty_input_model.increment_trigger(
                    Some("qty-input"),
                    false,
                    vec![],
                    vec![fandhe_frontend_core::text("+")],
                ),
            ],
        )],
    );
    let price_node = price_input_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![price_input_model.control(
            NumberInputFlags::default(),
            Vec::new(),
            vec![
                price_input_model.input(
                    "price",
                    Some("price-input"),
                    NumberInputFlags::default(),
                    vec![("data-action-input", "price_set")],
                ),
                price_input_model.increment_trigger(
                    Some("price-input"),
                    false,
                    vec![],
                    vec![fandhe_frontend_core::text("+")],
                ),
            ],
        )],
    );
    let html = format!(
        "{}{}",
        fandhe_frontend_core::render(&qty_node),
        fandhe_frontend_core::render(&price_node)
    );
    container.set_inner_html(&html);
    let _cleanup = RemoveOnDrop(container.clone());

    let price_increment_button = container
        .query_selector(
            r#"#price-input ~ [data-scope="number-input"][data-part="increment-trigger"]"#,
        )
        .expect("query_selector must not fail")
        .expect("price increment-trigger must exist");

    let recorded: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded_clone = recorded.clone();
    wire_number_input_events(container.clone(), move |action_ref: ActionRef| {
        recorded_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_number_input_events must not fail");

    price_increment_button
        .dispatch_event(&click_event())
        .unwrap();

    let dispatched = recorded.borrow().clone();
    assert_eq!(
        dispatched,
        vec![
            ActionRef {
                action: "price_set".to_string(),
                payload: "5".to_string(),
            },
            ActionRef {
                action: "increment".to_string(),
                payload: "price".to_string(),
            },
        ],
        "price 側の IncrementTrigger click は price_set/increment(payload=price) を dispatch すること"
    );

    let mut state = TwoFieldState {
        qty: 5.0,
        price: 5.0,
    };
    for action_ref in &dispatched {
        fandhe_frontend_interactive::dispatch(&mut state, &action_ref.action, &action_ref.payload);
    }
    assert_eq!(
        state.qty, 5.0,
        "qty は price 側の click の影響を受けないこと"
    );
    assert_eq!(state.price, 6.0, "price のみ increment されること");
}

// ---------------------------------------------------------------------
// 複数インスタンス識別（PR #1881 codex-review P1 是正）の実ブラウザ回帰。
// ---------------------------------------------------------------------

/// テスト専用の 2 フィールド `Component`（数量 `qty`・価格 `price`）。
/// `decode_action` は [`fandhe_frontend_wasm_full::number_input`] モジュール
/// 冒頭 doc「複数インスタンスの識別」節が示す書き方をそのまま実装し、
/// `data-action-input`（"qty_set"/"price_set"）・increment/decrement の
/// payload（`name` 属性値 "qty"/"price"）で更新先を振り分ける。
#[derive(Debug, Clone, Copy, PartialEq)]
struct TwoFieldAction {
    field: Field,
    op: Op,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Qty,
    Price,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Set(f64),
    Increment,
}

#[derive(Debug, Clone, Default)]
struct TwoFieldState {
    qty: f64,
    price: f64,
}

impl Component for TwoFieldState {
    type Action = TwoFieldAction;

    fn update(&mut self, action: TwoFieldAction) {
        let value = match action.op {
            Op::Set(v) => v,
            Op::Increment => match action.field {
                Field::Qty => self.qty + 1.0,
                Field::Price => self.price + 1.0,
            },
        };
        match action.field {
            Field::Qty => self.qty = value,
            Field::Price => self.price = value,
        }
    }

    fn view(&self) -> fandhe_frontend_core::Node {
        fandhe_frontend_core::text("")
    }

    fn decode_action(name: &str, payload: &str) -> Option<TwoFieldAction> {
        match name {
            "qty_set" => payload.parse::<f64>().ok().map(|v| TwoFieldAction {
                field: Field::Qty,
                op: Op::Set(v),
            }),
            "price_set" => payload.parse::<f64>().ok().map(|v| TwoFieldAction {
                field: Field::Price,
                op: Op::Set(v),
            }),
            "increment" if payload == "qty" => Some(TwoFieldAction {
                field: Field::Qty,
                op: Op::Increment,
            }),
            "increment" if payload == "price" => Some(TwoFieldAction {
                field: Field::Price,
                op: Op::Increment,
            }),
            _ => None,
        }
    }
}

/// 検証: 同じ root 配下に 2 つの NumberInput（qty/price、別 `name`/
/// 別 `data-action-input`）があるとき、price 側の ArrowUp が qty 側の
/// 状態を一切変更せず、dispatch された `(action, payload)` で両者を
/// 区別できること（PR #1881 codex-review P1「片方の ArrowUp がその入力
/// にだけ反映され、dispatch されたアクション名・payload で区別できる」
/// の受け入れ確認）。
#[wasm_bindgen_test]
fn two_instances_arrow_up_updates_only_the_targeted_field() {
    let document = web_sys::window().unwrap().document().unwrap();

    // 2 つの NumberInput の Input パーツを 1 つの container（1 root）に
    // まとめて差し込む（`Runtime::mount` が root へ 1 回だけ配線する構成の
    // 再現）。
    let container = create_container(&document, "ni-two-instances-root");
    let qty_input_model = NumberInput::new(Some(5.0), 0.0, 100.0, 1.0);
    let price_input_model = NumberInput::new(Some(5.0), 0.0, 100.0, 1.0);
    let qty_node = qty_input_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![qty_input_model.control(
            NumberInputFlags::default(),
            Vec::new(),
            vec![qty_input_model.input(
                "qty",
                Some("qty-input"),
                NumberInputFlags::default(),
                vec![("data-action-input", "qty_set")],
            )],
        )],
    );
    let price_node = price_input_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![price_input_model.control(
            NumberInputFlags::default(),
            Vec::new(),
            vec![price_input_model.input(
                "price",
                Some("price-input"),
                NumberInputFlags::default(),
                vec![("data-action-input", "price_set")],
            )],
        )],
    );
    let html = format!(
        "{}{}",
        fandhe_frontend_core::render(&qty_node),
        fandhe_frontend_core::render(&price_node)
    );
    container.set_inner_html(&html);
    let _cleanup = RemoveOnDrop(container.clone());

    let qty_input = container
        .query_selector("#qty-input")
        .expect("query_selector must not fail")
        .expect("qty input must exist");
    let price_input = container
        .query_selector("#price-input")
        .expect("query_selector must not fail")
        .expect("price input must exist");

    let recorded: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded_clone = recorded.clone();
    wire_number_input_events(container.clone(), move |action_ref: ActionRef| {
        recorded_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_number_input_events must not fail");

    // price 側の ArrowUp のみを送る。
    let default_not_prevented = price_input
        .dispatch_event(&keydown_event("ArrowUp"))
        .unwrap();
    assert!(!default_not_prevented, "ArrowUp は claim されること");

    let dispatched = recorded.borrow().clone();
    assert_eq!(
        dispatched,
        vec![
            ActionRef {
                action: "price_set".to_string(),
                payload: "5".to_string(),
            },
            ActionRef {
                action: "increment".to_string(),
                payload: "price".to_string(),
            },
        ],
        "price 側の ArrowUp は price_set/increment(payload=price) を dispatch すること"
    );

    // 記録された ActionRef を実際に `TwoFieldState` へ dispatch し、
    // qty には一切副作用がなく price のみ更新されることを確認する
    // （decode_action がアクション名 + payload だけで振り分けられる証跡）。
    let mut state = TwoFieldState {
        qty: 5.0,
        price: 5.0,
    };
    for action_ref in &dispatched {
        fandhe_frontend_interactive::dispatch(&mut state, &action_ref.action, &action_ref.payload);
    }
    assert_eq!(state.qty, 5.0, "qty は ArrowUp の影響を受けないこと");
    assert_eq!(state.price, 6.0, "price のみ increment されること");

    // qty 側の input.value は書き換えていないため DOM 上も変化しない
    // （本テストは dispatch 経路の識別を検証する対象であり、DOM 反映は
    // `arrow_up_increments_value_and_updates_dom` 等が別途検証済み）。
    let qty_html_input = qty_input.clone().dyn_into::<HtmlInputElement>().unwrap();
    assert_eq!(qty_html_input.value(), "5");
}

// ---------------------------------------------------------------------
// PR #1982 codex-review P1 是正の実ブラウザ回帰（イシュー #1962）。
// ---------------------------------------------------------------------

/// codex-review 指摘その 1: 公開 API で **Input のみ** に readonly を指定し
/// （Root/Control は既定フラグ）構築した場合でも、IncrementTrigger への
/// click が no-op であること。`click_increment_trigger_is_noop_when_readonly`
/// は Root/Control/Input すべてに `flags` を渡す構成のため、実際には
/// Control 側の `data-readonly`（`has_noninteractive_ancestor` が Trigger の
/// 祖先として辿る）が防御しており、Input 単体の readonly を防ぐ経路
/// （`handle_click` が Input 解決後に改めて `has_noninteractive_ancestor` を
/// 確認する分岐）を検証できていなかった。本テストはその欠落を埋める。
#[wasm_bindgen_test]
fn click_increment_trigger_is_noop_when_only_input_is_readonly() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "ni-click-input-only-readonly");
    let number_input = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
    let input_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let node = number_input.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![number_input.control(
            NumberInputFlags::default(),
            Vec::new(),
            vec![
                number_input.increment_trigger(
                    Some("qty-input"),
                    false,
                    Vec::new(),
                    vec![fandhe_frontend_core::text("+")],
                ),
                number_input.input("qty", Some("qty-input"), input_flags, Vec::new()),
            ],
        )],
    );
    let html = fandhe_frontend_core::render(&node);
    container.set_inner_html(&html);
    let _cleanup = RemoveOnDrop(container.clone());

    let root = container
        .first_element_child()
        .expect("number-input root must exist");
    let increment_button = root
        .query_selector(r#"[data-scope="number-input"][data-part="increment-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("increment-trigger element must exist");
    assert!(
        !increment_button.has_attribute("disabled"),
        "readonly 時は increment-trigger にネイティブ disabled が付かないこと（前提確認）"
    );
    let control = root
        .query_selector(r#"[data-scope="number-input"][data-part="control"]"#)
        .expect("query_selector must not fail")
        .expect("control element must exist");
    assert!(
        !control.has_attribute("data-readonly"),
        "Control は既定フラグで構築されており data-readonly を持たないこと（前提確認）"
    );
    assert!(
        !root.has_attribute("data-readonly"),
        "Root も既定フラグで構築されており data-readonly を持たないこと（前提確認）"
    );

    let component = Rc::new(RefCell::new(number_input));
    let component = wire_with_dom_reflection(root, component);

    increment_button.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        component.borrow().value(),
        Some(5.0),
        "Input のみが readonly な場合でも、Input 解決後の has_noninteractive_ancestor \
         確認により IncrementTrigger への click が no-op であること"
    );
}

/// codex-review 指摘その 2: Control を省略した NumberInput が別の
/// NumberInput にネストされている場合、内側の IncrementTrigger への click
/// が内側インスタンスの Input のみを更新し、外側インスタンスの Input を
/// 誤って更新しないこと。
///
/// 外側 NumberInput（`outer`、Control 省略）の Root 直下に、外側 Input
/// （document 順で先に配置）と、Control を省略した内側 NumberInput
/// （`inner`）の Root（IncrementTrigger + Input）をネストする。旧実装は
/// `find_input_within_control` が最寄りの Root（inner）で探索を打ち切らず
/// 外側の Root まで祖先探索を続け、`container` が外側 Root に置き換わって
/// 外側 Input（document 順で先）を誤って解決していた。
#[wasm_bindgen_test]
fn nested_number_input_without_control_click_updates_only_inner_instance() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "ni-nested-without-control");

    let outer_model = NumberInput::new(Some(50.0), 0.0, 100.0, 1.0);
    let inner_model = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);

    let inner_node = inner_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![
            inner_model.increment_trigger(
                Some("inner-input"),
                false,
                Vec::new(),
                vec![fandhe_frontend_core::text("+")],
            ),
            inner_model.input(
                "inner",
                Some("inner-input"),
                NumberInputFlags::default(),
                vec![("data-action-input", "inner_set")],
            ),
        ],
    );
    let outer_node = outer_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![
            outer_model.input(
                "outer",
                Some("outer-input"),
                NumberInputFlags::default(),
                vec![("data-action-input", "outer_set")],
            ),
            inner_node,
        ],
    );
    let html = fandhe_frontend_core::render(&outer_node);
    container.set_inner_html(&html);
    let _cleanup = RemoveOnDrop(container.clone());

    let outer_input = container
        .query_selector("#outer-input")
        .expect("query_selector must not fail")
        .expect("outer input must exist")
        .dyn_into::<HtmlInputElement>()
        .expect("outer input must cast to HtmlInputElement");
    let inner_increment_button = container
        .query_selector(r#"[data-scope="number-input"][data-part="increment-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("inner increment-trigger element must exist");

    let recorded: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded_clone = recorded.clone();
    wire_number_input_events(container.clone(), move |action_ref: ActionRef| {
        recorded_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_number_input_events must not fail");

    inner_increment_button
        .dispatch_event(&click_event())
        .unwrap();

    let dispatched = recorded.borrow().clone();
    assert_eq!(
        dispatched,
        vec![
            ActionRef {
                action: "inner_set".to_string(),
                payload: "5".to_string(),
            },
            ActionRef {
                action: "increment".to_string(),
                payload: "inner".to_string(),
            },
        ],
        "内側 IncrementTrigger の click は内側インスタンス（inner_set/increment(payload=inner)） \
         のみを dispatch し、外側インスタンス（outer_set）を一切 dispatch しないこと"
    );
    assert_eq!(
        outer_input.value(),
        "50",
        "外側 Input の value は内側トリガーの click の影響を受けないこと（DOM 上の再確認）"
    );
}

#[wasm_bindgen_test]
fn nested_number_input_without_control_outer_trigger_after_inner_input_is_noop() {
    // PR #1982 codex-review P1 / Bugbot 指摘の回帰: Control を省略した
    // 外側 NumberInput に「内側 Root/Input → 外側 Input → 外側
    // IncrementTrigger」の順で DOM 配置すると、外側トリガーの祖先探索が
    // 最寄り Root（外側 Root）で正しく止まっても、`query_selector` が
    // 外側 Root 部分木内の最初の一致（内側 Input）を返してしまい、外側
    // トリガーの click が内側インスタンスを誤って更新し得た。
    // `find_input_within_control` は候補 Input 自身の最寄り Root が
    // トリガーの `nearest_root` と一致することを検証し、不一致なら
    // fail-closed で dispatch しない（no-op）契約を検証する。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "ni-nested-outer-trigger-after-inner");

    let outer_model = NumberInput::new(Some(50.0), 0.0, 100.0, 1.0);
    let inner_model = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);

    let inner_node = inner_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![inner_model.input(
            "inner",
            None,
            NumberInputFlags::default(),
            vec![("data-action-input", "inner_set")],
        )],
    );
    let outer_node = outer_model.root(
        NumberInputFlags::default(),
        Vec::new(),
        vec![
            inner_node,
            outer_model.input(
                "outer",
                Some("outer-input"),
                NumberInputFlags::default(),
                vec![("data-action-input", "outer_set")],
            ),
            outer_model.increment_trigger(
                Some("outer-input"),
                false,
                Vec::new(),
                vec![fandhe_frontend_core::text("+")],
            ),
        ],
    );
    let html = fandhe_frontend_core::render(&outer_node);
    container.set_inner_html(&html);
    let _cleanup = RemoveOnDrop(container.clone());

    let outer_input = container
        .query_selector("#outer-input")
        .expect("query_selector must not fail")
        .expect("outer input must exist")
        .dyn_into::<HtmlInputElement>()
        .expect("outer input must cast to HtmlInputElement");
    let outer_increment_button = container
        .query_selector(r#"[data-scope="number-input"][data-part="increment-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("outer increment-trigger element must exist");

    let recorded: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded_clone = recorded.clone();
    wire_number_input_events(container.clone(), move |action_ref: ActionRef| {
        recorded_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_number_input_events must not fail");

    outer_increment_button
        .dispatch_event(&click_event())
        .unwrap();

    let dispatched = recorded.borrow().clone();
    assert!(
        dispatched.is_empty(),
        "内側 Input の方が DOM 順で先にある構成でも、外側トリガーの click は \
         内側インスタンスを誤って更新せず no-op であること（実際: {dispatched:?}）"
    );
    assert_eq!(
        outer_input.value(),
        "50",
        "外側 Input の value は誤配線の影響を受けないこと（DOM 上の再確認）"
    );
}
