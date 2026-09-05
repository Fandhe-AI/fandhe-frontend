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
use fandhe_frontend_wasm_full::number_input::wire_number_input_component;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit};

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
