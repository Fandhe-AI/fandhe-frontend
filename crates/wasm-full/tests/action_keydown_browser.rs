//! 汎用 keydown 配線（`Runtime::mount`/`hydrate` の `Self::wire_keydown`、イシュー #3754）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! 純粋ロジック（属性解釈・キー照合・IME 除外）は `events.rs` の native テスト（#3753）が検証
//! 済みである。本ファイルは配線層が実 DOM 上で次を満たすことを固定する。
//!
//! 1. 一致したキーが action になり、束縛点・keyed list が差分更新され要素の同一性が保たれる
//! 2. 不一致キー・修飾キー不一致・IME 変換中（`isComposing` / `keyCode == 229`）は action にならない
//! 3. `data-keydown-prevent-default` の opt-in だけが `preventDefault()` を呼ぶ
//! 4. 先に動く部品専用リスナーが `preventDefault` したキーには譲る（優先順位）
//! 5. root 外の要素は解釈しない / hydrate 経路でも同じ配線が成立する / XSS 文字列は文字列のまま
//!
//! `AppState::view()` は keydown 属性を出力しないため、mount/hydrate の後に既存要素へ
//! `set_attribute` で属性を付ける（`closest` はイベント発生時に評価される）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "action-keydown")]

use fandhe_frontend_interactive::{AppState, Hydrate};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document.create_element("div").expect("create_element");
    container.set_id(id);
    document
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append_child");
    container
}

/// テスト末尾でプレースホルダを確実に除去する（テスト間の DOM 汚染防止、`runtime_browser.rs` と同じ）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn same(a: &Element, b: Option<Element>) -> bool {
    b.is_some_and(|b| a.is_same_node(Some(&b)))
}

#[derive(Default, Clone, Copy)]
struct Mods {
    ctrl: bool,
    shift: bool,
    composing: bool,
    key_code: u32,
}

/// 合成 keydown を `target` 上で発火し、`defaultPrevented` を返す。`bubbles`/`cancelable` は真。
fn press(target: &Element, key: &str, mods: Mods) -> bool {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(mods.ctrl);
    init.set_shift_key(mods.shift);
    init.set_is_composing(mods.composing);
    init.set_key_code(mods.key_code);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).expect("event");
    // `dispatch_event` は `preventDefault` されたとき false を返す。
    !target.dispatch_event(&event).expect("dispatch_event")
}

fn plain() -> Mods {
    Mods::default()
}

struct Fixture {
    placeholder: Element,
    _cleanup: RemoveOnDrop,
    runtime: Runtime<AppState>,
    input: Element,
}

fn mount_fixture(id: &str) -> Fixture {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, id);
    let cleanup = RemoveOnDrop(placeholder.clone());
    let runtime = Runtime::mount(id, AppState::new()).expect("mount must succeed");
    let input = placeholder.query_selector("#draft-input").unwrap().unwrap();
    Fixture {
        placeholder,
        _cleanup: cleanup,
        runtime,
        input,
    }
}

fn declare(el: &Element, action: &str, keys: &str) {
    el.set_attribute("data-action-keydown", action).unwrap();
    el.set_attribute("data-keys", keys).unwrap();
}

fn counter_el(f: &Fixture) -> Element {
    f.placeholder
        .query_selector("[data-testid='counter-value']")
        .unwrap()
        .unwrap()
}

#[wasm_bindgen_test]
fn matching_key_dispatches_and_preserves_identity() {
    let f = mount_fixture("keydown-match-root");
    declare(&f.input, "set_draft", "Control+Enter");
    let list = f
        .placeholder
        .query_selector("[data-testid='item-list']")
        .unwrap()
        .unwrap();
    let first_item = list.children().item(0).unwrap();
    let counter = counter_el(&f);

    press(
        &f.input,
        "Enter",
        Mods {
            ctrl: true,
            ..plain()
        },
    );

    // payload は照合成功時の正規トークン文字列。
    assert_eq!(f.runtime.component().draft, "Control+Enter");
    assert_eq!(
        f.input.dyn_ref::<HtmlInputElement>().unwrap().value(),
        "Control+Enter"
    );
    assert!(same(&counter, Some(counter_el(&f))));
    assert!(same(
        &f.input,
        f.placeholder.query_selector("#draft-input").unwrap()
    ));
    assert!(same(&first_item, list.children().item(0)));
}

#[wasm_bindgen_test]
fn matching_key_updates_keyed_list_preserving_existing_rows() {
    let f = mount_fixture("keydown-list-root");
    let list = f
        .placeholder
        .query_selector("[data-testid='item-list']")
        .unwrap()
        .unwrap();
    let first_item = list.children().item(0).unwrap();
    let before = list.children().length();
    f.runtime.dispatch_action("set_draft", "新しい項目");
    // 属性は祖先（root 自身）に置いても `closest` で解決される。
    declare(&f.placeholder, "add_item", "Enter");

    press(&f.input, "Enter", plain());

    assert_eq!(list.children().length(), before + 1);
    assert!(same(&first_item, list.children().item(0)));
}

#[wasm_bindgen_test]
fn non_matching_keys_are_ignored() {
    let f = mount_fixture("keydown-nomatch-root");
    declare(&f.input, "increment", "Escape");
    let before = f.runtime.component().counter;

    press(&f.input, "a", plain());
    press(
        &f.input,
        "Escape",
        Mods {
            shift: true,
            ..plain()
        },
    );
    press(
        &f.input,
        "Escape",
        Mods {
            ctrl: true,
            ..plain()
        },
    );
    assert_eq!(f.runtime.component().counter, before);

    press(&f.input, "Escape", plain());
    assert_eq!(f.runtime.component().counter, before + 1);
    assert_eq!(
        counter_el(&f).text_content().unwrap_or_default(),
        (before + 1).to_string()
    );
}

#[wasm_bindgen_test]
fn ime_composition_is_excluded() {
    let f = mount_fixture("keydown-ime-root");
    declare(&f.input, "increment", "Escape Enter");
    let before = f.runtime.component().counter;

    for key in ["Escape", "Enter"] {
        press(
            &f.input,
            key,
            Mods {
                composing: true,
                ..plain()
            },
        );
        press(
            &f.input,
            key,
            Mods {
                key_code: 229,
                ..plain()
            },
        );
    }
    assert_eq!(
        f.runtime.component().counter,
        before,
        "IME 変換中の Escape/Enter は action にならない"
    );

    press(&f.input, "Escape", plain());
    press(&f.input, "Enter", plain());
    assert_eq!(f.runtime.component().counter, before + 2);
}

#[wasm_bindgen_test]
fn prevent_default_is_opt_in() {
    let f = mount_fixture("keydown-prevent-root");
    declare(&f.input, "increment", "Escape");

    assert!(
        !press(&f.input, "Escape", plain()),
        "opt-in 無しでは preventDefault しない"
    );
    f.input
        .set_attribute("data-keydown-prevent-default", "false")
        .unwrap();
    assert!(!press(&f.input, "Escape", plain()));

    f.input
        .set_attribute("data-keydown-prevent-default", "")
        .unwrap();
    assert!(press(&f.input, "Escape", plain()));
    f.input
        .set_attribute("data-keydown-prevent-default", "true")
        .unwrap();
    assert!(press(&f.input, "Escape", plain()));
    assert!(
        !press(&f.input, "a", plain()),
        "不一致のキーは opt-in があっても preventDefault しない"
    );
}

#[wasm_bindgen_test]
fn earlier_listener_that_prevented_default_takes_priority() {
    let f = mount_fixture("keydown-priority-root");
    declare(&f.input, "increment", "Escape");
    let before = f.runtime.component().counter;

    // 部品専用配線（root の bubble より先に動き、処理したキーを preventDefault する）の代役。
    let closure = Closure::<dyn FnMut(Event)>::new(|event: Event| event.prevent_default());
    f.input
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    press(&f.input, "Escape", plain());
    f.input
        .remove_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();

    assert_eq!(f.runtime.component().counter, before);
    press(&f.input, "Escape", plain());
    assert_eq!(f.runtime.component().counter, before + 1);
}

#[wasm_bindgen_test]
fn elements_outside_root_are_ignored() {
    let f = mount_fixture("keydown-outside-root");
    let document = web_sys::window().unwrap().document().unwrap();
    let outside = create_placeholder(&document, "keydown-outside-other");
    let _cleanup = RemoveOnDrop(outside.clone());
    declare(&outside, "increment", "Escape");
    let before = f.runtime.component().counter;

    press(&outside, "Escape", plain());

    assert_eq!(f.runtime.component().counter, before);
}

#[wasm_bindgen_test]
fn hydrate_path_wires_keydown() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut seed = AppState::new();
    seed.counter = 5;
    let placeholder = create_placeholder(&document, "interactive-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    placeholder.set_inner_html(&fandhe_frontend_wasm_full::render_component_html(&seed));
    for (name, value) in seed.hydration_attrs() {
        placeholder.set_attribute(&name, &value).unwrap();
    }
    let runtime = Runtime::hydrate("interactive-root", AppState::new()).expect("hydrate");
    let input = placeholder.query_selector("#draft-input").unwrap().unwrap();
    declare(&input, "increment", "Control+Enter");

    press(
        &input,
        "Enter",
        Mods {
            ctrl: true,
            ..plain()
        },
    );

    assert_eq!(runtime.component().counter, 6);
}

#[wasm_bindgen_test]
fn script_like_key_name_stays_text() {
    let f = mount_fixture("keydown-xss-root");
    let key = "<script>alert(1)</script>";
    declare(&f.input, "set_draft", key);

    press(&f.input, key, plain());

    assert_eq!(f.runtime.component().draft, key);
    assert_eq!(f.input.dyn_ref::<HtmlInputElement>().unwrap().value(), key);
    assert!(f.placeholder.query_selector("script").unwrap().is_none());
}
