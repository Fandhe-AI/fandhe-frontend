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
    repeat: bool,
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
    init.set_repeat(mods.repeat);
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

// ---- data-payload の合成（イシュー #3764）----

#[wasm_bindgen_test]
fn data_payload_overrides_key_token_for_set_draft() {
    let f = mount_fixture("keydown-payload-draft-root");
    declare(&f.input, "set_draft", "Control+Enter");
    f.input.set_attribute("data-payload", "hello").unwrap();

    press(
        &f.input,
        "Enter",
        Mods {
            ctrl: true,
            ..plain()
        },
    );

    assert_eq!(f.runtime.component().draft, "hello");
    assert_eq!(
        f.input.dyn_ref::<HtmlInputElement>().unwrap().value(),
        "hello"
    );
}

#[wasm_bindgen_test]
fn data_payload_lets_remove_item_work_on_keydown() {
    let f = mount_fixture("keydown-payload-remove-root");
    f.runtime.dispatch_action("set_draft", "second");
    f.runtime.dispatch_action("add_item", "");
    let list = f
        .placeholder
        .query_selector("[data-testid='item-list']")
        .unwrap()
        .unwrap();
    let before = list.children().length();
    let first_item = list.children().item(0).unwrap();
    let id = f.runtime.component().item_ids[1].to_string();
    declare(&f.input, "remove_item", "Delete");
    f.input.set_attribute("data-payload", &id).unwrap();

    press(&f.input, "Delete", plain());

    assert_eq!(list.children().length(), before - 1);
    assert!(same(&first_item, list.children().item(0)));
}

#[wasm_bindgen_test]
fn data_payload_script_stays_text() {
    let f = mount_fixture("keydown-payload-xss-root");
    let p = "<script>alert(1)</script>";
    declare(&f.input, "set_draft", "Enter");
    f.input.set_attribute("data-payload", p).unwrap();

    press(&f.input, "Enter", plain());

    assert_eq!(f.runtime.component().draft, p);
    assert!(f.placeholder.query_selector("script").unwrap().is_none());
}

fn repeating() -> Mods {
    Mods {
        repeat: true,
        ..plain()
    }
}

#[wasm_bindgen_test]
fn repeat_dispatches_without_ignore_repeat_attr() {
    let f = mount_fixture("keydown-repeat-default-root");
    declare(&f.input, "increment", "Enter");
    let before = f.runtime.component().counter;

    press(&f.input, "Enter", plain());
    press(&f.input, "Enter", repeating());

    assert_eq!(f.runtime.component().counter, before + 2);
}

#[wasm_bindgen_test]
fn ignore_repeat_attr_suppresses_only_repeating_keydown() {
    for (i, value) in ["", "true"].into_iter().enumerate() {
        let f = mount_fixture(&format!("keydown-repeat-ignore-root-{i}"));
        declare(&f.input, "increment", "Enter");
        f.input
            .set_attribute("data-keydown-ignore-repeat", value)
            .unwrap();
        let before = f.runtime.component().counter;

        press(&f.input, "Enter", plain());
        assert_eq!(f.runtime.component().counter, before + 1);
        press(&f.input, "Enter", repeating());
        press(&f.input, "Enter", repeating());
        assert_eq!(f.runtime.component().counter, before + 1);

        // 再押下（repeat == false）は通常どおり dispatch される。
        press(&f.input, "Enter", plain());
        assert_eq!(f.runtime.component().counter, before + 2);
    }
}

#[wasm_bindgen_test]
fn ignore_repeat_attr_false_does_not_suppress() {
    let f = mount_fixture("keydown-repeat-false-root");
    declare(&f.input, "increment", "Enter");
    f.input
        .set_attribute("data-keydown-ignore-repeat", "false")
        .unwrap();
    let before = f.runtime.component().counter;

    press(&f.input, "Enter", repeating());

    assert_eq!(f.runtime.component().counter, before + 1);
}

#[wasm_bindgen_test]
fn ignore_repeat_keeps_prevent_default_but_skips_dispatch() {
    let f = mount_fixture("keydown-repeat-prevent-root");
    declare(&f.input, "increment", "Enter");
    f.input
        .set_attribute("data-keydown-ignore-repeat", "")
        .unwrap();
    f.input
        .set_attribute("data-keydown-prevent-default", "")
        .unwrap();
    let before = f.runtime.component().counter;

    let prevented = press(&f.input, "Enter", repeating());

    assert!(prevented);
    assert_eq!(f.runtime.component().counter, before);
}
// ---- document/window 部品との排他（イシュー #3768、設計記録 §41.5 方式 2b）----
//
// overlay の Escape・sidebar の Cmd/Ctrl+B と Escape・command の Ctrl/Cmd+K は root の bubble
// より後に動くため、部品が消費する keydown では汎用 action を見送り、部品が単独で処理する。
// 各ケースは「部品だけが動く」と「汎用 action が動かない」を同時に表明し、部品が消費しない
// 状況（閉じた overlay・無効化・dialog なし）では従来どおり汎用 action だけが動くことも固定する。

use fandhe_frontend_wasm_full::overlay::OverlayCloseController;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn element_with(document: &Document, tag: &str, attrs: &[(&str, &str)]) -> Element {
    let element = document.create_element(tag).expect("create_element");
    for (name, value) in attrs {
        element.set_attribute(name, value).expect("set_attribute");
    }
    element
}

fn current_document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

/// `target` へクリック回数を数えるリスナーを付けて回数のセルを返す。
fn count_clicks(target: &Element) -> Rc<Cell<u32>> {
    let count = Rc::new(Cell::new(0));
    let counter = count.clone();
    let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        counter.set(counter.get() + 1);
    });
    target
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
    count
}

type CloseLog = Rc<RefCell<Vec<usize>>>;

fn overlay_controller(document: &Document) -> (OverlayCloseController, CloseLog) {
    let log: CloseLog = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    let controller = OverlayCloseController::new(document, move |request| {
        sink.borrow_mut().push(request.index);
    })
    .expect("OverlayCloseController::new");
    (controller, log)
}

#[wasm_bindgen_test]
fn overlay_escape_is_exclusive_with_generic_action() {
    let f = mount_fixture("keydown-claim-overlay-root");
    declare(&f.input, "set_draft", "Escape");
    let document = current_document();
    let (controller, log) = overlay_controller(&document);
    let content = element_with(&document, "div", &[("data-scope", "dialog")]);
    f.placeholder.append_child(&content).unwrap();
    let index = controller
        .push_overlay(&content, None)
        .expect("known scope");

    press(&f.input, "Escape", plain());
    assert_eq!(log.borrow().as_slice(), &[0], "overlay closes");
    assert_eq!(f.runtime.component().draft, "", "generic action is skipped");

    controller.remove_overlay(index);
    press(&f.input, "Escape", plain());
    assert_eq!(log.borrow().len(), 1, "overlay no longer registered");
    assert_eq!(f.runtime.component().draft, "Escape", "generic takes over");
}

#[wasm_bindgen_test]
fn overlay_topmost_opt_out_leaves_escape_to_generic_action() {
    let f = mount_fixture("keydown-claim-overlay-optout-root");
    declare(&f.input, "set_draft", "Escape");
    let document = current_document();
    let (controller, log) = overlay_controller(&document);
    let content = element_with(
        &document,
        "div",
        &[("data-scope", "dialog"), ("data-close-on-escape", "false")],
    );
    f.placeholder.append_child(&content).unwrap();
    controller
        .push_overlay(&content, None)
        .expect("known scope");

    press(&f.input, "Escape", plain());
    assert!(log.borrow().is_empty(), "opt-out overlay does not close");
    assert_eq!(f.runtime.component().draft, "Escape");
}

#[wasm_bindgen_test]
fn dropped_overlay_controller_no_longer_claims_escape() {
    let f = mount_fixture("keydown-claim-overlay-drop-root");
    declare(&f.input, "set_draft", "Escape");
    let document = current_document();
    let (controller, _log) = overlay_controller(&document);
    let content = element_with(&document, "div", &[("data-scope", "dialog")]);
    f.placeholder.append_child(&content).unwrap();
    controller
        .push_overlay(&content, None)
        .expect("known scope");
    drop(controller);

    press(&f.input, "Escape", plain());
    assert_eq!(f.runtime.component().draft, "Escape");
}

/// root 内へ sidebar（provider > trigger）を足して配線し、trigger のクリック回数を返す。
fn mount_sidebar_in(f: &Fixture, query: &str, trigger_disabled: bool) -> (Element, Rc<Cell<u32>>) {
    let document = current_document();
    let provider = element_with(
        &document,
        "div",
        &[
            ("data-scope", "sidebar"),
            ("data-part", "provider"),
            ("data-state", "collapsed"),
        ],
    );
    let trigger = element_with(
        &document,
        "button",
        &[("data-scope", "sidebar"), ("data-part", "trigger")],
    );
    if trigger_disabled {
        trigger.set_attribute("data-disabled", "").unwrap();
    }
    provider.append_child(&trigger).unwrap();
    f.placeholder.append_child(&provider).unwrap();
    let clicks = count_clicks(&trigger);
    fandhe_frontend_wasm_full::sidebar::wire_sidebar_events_with_query(
        f.placeholder.clone(),
        query,
    )
    .expect("wire_sidebar_events_with_query");
    (provider, clicks)
}

fn ctrl() -> Mods {
    Mods {
        ctrl: true,
        ..plain()
    }
}

#[wasm_bindgen_test]
fn sidebar_ctrl_b_is_exclusive_with_generic_action() {
    let f = mount_fixture("keydown-claim-sidebar-toggle-root");
    declare(&f.input, "set_draft", "Control+b");
    let (_provider, clicks) = mount_sidebar_in(&f, "(max-width: 0px)", false);

    press(&f.input, "b", ctrl());
    assert_eq!(clicks.get(), 1, "sidebar toggles via trigger click");
    assert_eq!(f.runtime.component().draft, "", "generic action is skipped");
}

#[wasm_bindgen_test]
fn sidebar_ctrl_b_with_disabled_trigger_falls_back_to_generic_action() {
    let f = mount_fixture("keydown-claim-sidebar-disabled-root");
    declare(&f.input, "set_draft", "Control+b");
    let (_provider, clicks) = mount_sidebar_in(&f, "(max-width: 0px)", true);

    press(&f.input, "b", ctrl());
    assert_eq!(clicks.get(), 0);
    assert_eq!(f.runtime.component().draft, "Control+b");
}

#[wasm_bindgen_test]
fn sidebar_escape_on_open_mobile_drawer_is_exclusive_but_closed_drawer_is_not() {
    let f = mount_fixture("keydown-claim-sidebar-escape-root");
    declare(&f.input, "set_draft", "Escape");
    // 常時モバイルのクエリで配線してから、drawer を開いた状態へ属性を直接設定する。
    let (provider, clicks) = mount_sidebar_in(&f, "(min-width: 1px)", false);

    provider.set_attribute("data-mobile", "").unwrap();
    provider.set_attribute("data-state", "expanded").unwrap();
    press(&f.input, "Escape", plain());
    assert_eq!(clicks.get(), 1, "sidebar dismisses the drawer");
    assert_eq!(f.runtime.component().draft, "", "generic action is skipped");

    provider.set_attribute("data-state", "collapsed").unwrap();
    press(&f.input, "Escape", plain());
    assert_eq!(clicks.get(), 1, "closed drawer is not dismissed again");
    assert_eq!(f.runtime.component().draft, "Escape", "generic takes over");
}

/// root 内へ command（root > dialog）を足して配線し、dispatch された action 名の記録を返す。
fn mount_command_in(
    f: &Fixture,
    root_disabled: bool,
    with_dialog: bool,
) -> Rc<RefCell<Vec<String>>> {
    let document = current_document();
    let command_root = element_with(
        &document,
        "div",
        &[("data-scope", "command"), ("data-part", "root")],
    );
    if root_disabled {
        command_root.set_attribute("data-disabled", "").unwrap();
    }
    if with_dialog {
        let dialog = element_with(
            &document,
            "div",
            &[("data-scope", "command"), ("data-part", "dialog")],
        );
        command_root.append_child(&dialog).unwrap();
    }
    f.placeholder.append_child(&command_root).unwrap();
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(f.placeholder.clone(), move |action| {
        sink.borrow_mut().push(action.action);
    })
    .expect("wire_command_events");
    log
}

#[wasm_bindgen_test]
fn command_ctrl_k_is_exclusive_with_generic_action() {
    let f = mount_fixture("keydown-claim-command-root");
    declare(&f.input, "set_draft", "Control+k");
    let log = mount_command_in(&f, false, true);

    press(&f.input, "k", ctrl());
    assert_eq!(log.borrow().as_slice(), ["toggle"], "command toggles");
    assert_eq!(f.runtime.component().draft, "", "generic action is skipped");
}

#[wasm_bindgen_test]
fn command_ctrl_k_without_dialog_or_when_disabled_falls_back_to_generic_action() {
    let f = mount_fixture("keydown-claim-command-nodialog-root");
    declare(&f.input, "set_draft", "Control+k");
    let log = mount_command_in(&f, false, false);
    press(&f.input, "k", ctrl());
    assert!(log.borrow().is_empty());
    assert_eq!(f.runtime.component().draft, "Control+k");

    let g = mount_fixture("keydown-claim-command-disabled-root");
    declare(&g.input, "set_draft", "Control+k");
    let log = mount_command_in(&g, true, true);
    press(&g.input, "k", ctrl());
    assert!(log.borrow().is_empty());
    assert_eq!(g.runtime.component().draft, "Control+k");
}

#[wasm_bindgen_test]
fn unclaimed_keys_still_reach_generic_action_with_components_wired() {
    let f = mount_fixture("keydown-claim-regression-root");
    declare(&f.input, "set_draft", "Control+j Escape");
    let document = current_document();
    let (_controller, log) = overlay_controller(&document);
    let log_cmd = mount_command_in(&f, false, true);
    let (_provider, clicks) = mount_sidebar_in(&f, "(max-width: 0px)", false);

    // 無関係なキーは claim されず従来どおり汎用 action が動く。
    press(&f.input, "j", ctrl());
    assert_eq!(f.runtime.component().draft, "Control+j");
    // overlay が空なら Escape も汎用が受ける（sidebar は閉じた drawer、overlay は未登録）。
    press(&f.input, "Escape", plain());
    assert_eq!(f.runtime.component().draft, "Escape");
    assert!(log.borrow().is_empty());
    assert!(log_cmd.borrow().is_empty());
    assert_eq!(clicks.get(), 0);
}
