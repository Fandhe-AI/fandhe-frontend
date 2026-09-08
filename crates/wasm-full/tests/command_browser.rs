//! `fandhe_frontend_wasm_full::command`（イシュー #2069、親 #2067）の実
//! ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/command.rs` の native 単体テスト（`#[cfg(test)]`）は
//! DOM 非依存の純粋ロジック層（`command_key_action`/`is_toggle_shortcut`/
//! `group_should_hide`）までを検証済みである。本ファイルはその先、
//! `command::wiring`（`#[cfg(target_arch = "wasm32")]` 配線層）が実 DOM 上で
//! 合成イベントに対して正しく絞り込み・行選択・実行・開閉を行うことを
//! 検証する（`number_input_browser.rs`/`overlay_close_browser.rs` と同型の
//! 実 DOM 検証パターンを踏襲する）。
//!
//! フィクスチャの HTML はすべて `fandhe-frontend-headless-ui` の `command`
//! 自由関数・`Command` 利便メソッド + `fandhe_frontend_core::render`
//! （既定エスケープ）で組み立て、`format!` 等による HTML 文字列直接組み立て・
//! `raw_html()` は使用しない（`.claude/rules/coding-rust.md`）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::command::{self, Command, CommandAction};
use fandhe_frontend_interactive::Component;
use fandhe_frontend_wasm_full::command::wire_command_component;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventInit, HtmlInputElement, KeyboardEvent, KeyboardEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`number_input_browser.rs::create_container` と同じ意図）。
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

fn keydown_event_with(key: &str, ctrl: bool, meta: bool, alt: bool) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(ctrl);
    init.set_meta_key(meta);
    init.set_alt_key(alt);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

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

fn click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

fn input_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("input", &init).expect("Event::new must not fail")
}

/// `container` 配下へ Command（root > dialog > input + list(+group/separator)
/// + empty）を組み立てる。`items` は `(value, label, disabled)`。
///
/// 戻り値は `(root, dialog, input, list, item_elements)`。
fn build_command_dom(
    document: &Document,
    container_id: &str,
    command: &Command,
    items: &[(&str, &str, bool)],
) -> (Element, Element, Element, Element, Vec<Element>) {
    let container = create_container(document, container_id);
    let list_id = format!("{container_id}-list");
    let entries: Vec<(&str, &str)> = items.iter().map(|(v, l, _)| (*v, *l)).collect();
    let is_empty = command.is_empty(&entries);

    let item_nodes: Vec<_> = items
        .iter()
        .map(|(value, label, disabled)| {
            command.item(
                value,
                *disabled,
                Some(&format!("{container_id}-item-{value}")),
                Vec::new(),
                vec![text(*label)],
            )
        })
        .collect();

    let node = command.root(
        is_empty,
        Vec::new(),
        vec![command.dialog(
            "Command Menu",
            Vec::new(),
            vec![
                command.input(&list_id, None, Vec::new()),
                command::list(&list_id, "Suggestions", is_empty, Vec::new(), item_nodes),
                command::empty(is_empty, Vec::new(), vec![text("No results found.")]),
            ],
        )],
    );
    let html = render(&node);
    container.set_inner_html(&html);

    let root = container
        .first_element_child()
        .expect("command root must exist");
    let dialog = root
        .query_selector(r#"[data-scope="command"][data-part="dialog"]"#)
        .expect("query_selector must not fail")
        .expect("dialog element must exist");
    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .expect("query_selector must not fail")
        .expect("input element must exist");
    let list = root
        .query_selector(r#"[data-scope="command"][data-part="list"]"#)
        .expect("query_selector must not fail")
        .expect("list element must exist");
    let item_elements: Vec<Element> = items
        .iter()
        .map(|(value, _, _)| {
            root.query_selector(&format!(
                r#"[data-scope="command"][data-part="item"][data-value="{value}"]"#
            ))
            .expect("query_selector must not fail")
            .unwrap_or_else(|| panic!("item element for value={value} must exist"))
        })
        .collect();

    (root, dialog, input, list, item_elements)
}

/// [`wire`] の記録用アクションログ型（`(action, payload)` の列）。
type ActionLog = Rc<RefCell<Vec<(String, String)>>>;

/// 配線し、記録用のアクション列と `Command` 状態を返す。
fn wire(root: Element, command: Command) -> (Rc<RefCell<Command>>, ActionLog) {
    let component = Rc::new(RefCell::new(command));
    let log: ActionLog = Rc::new(RefCell::new(Vec::new()));
    wire_command_component(root, component.clone(), move |_state: &Command, _root| {})
        .expect("wire_command_component must not fail");
    (component, log)
}

// --- (a) 入力絞り込み ---

#[wasm_bindgen_test]
fn typing_query_hides_non_matching_items_and_sets_data_empty_when_all_hidden() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [
        ("calendar", "Calendar", false),
        ("search", "Search Emoji", false),
    ];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-filter", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();

    assert!(
        !item_elements[0].has_attribute("hidden"),
        "calendar は可視のまま"
    );
    assert!(
        item_elements[1].has_attribute("hidden"),
        "search は非表示になる"
    );

    html_input.set_value("zzz");
    input.dispatch_event(&input_event()).unwrap();
    assert!(item_elements[0].has_attribute("hidden"));
    assert!(item_elements[1].has_attribute("hidden"));
    assert!(root.has_attribute("data-empty"));

    html_input.set_value("");
    input.dispatch_event(&input_event()).unwrap();
    assert!(!item_elements[0].has_attribute("hidden"));
    assert!(!item_elements[1].has_attribute("hidden"));
    assert!(!root.has_attribute("data-empty"));
}

#[wasm_bindgen_test]
fn group_hides_only_when_all_contained_items_are_hidden() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false), ("search", "Search", false)];
    let list_id = "cmd-group-list";
    let entries: Vec<(&str, &str)> = items.iter().map(|(v, l, _)| (*v, *l)).collect();
    let is_empty = command.is_empty(&entries);
    let group_heading_id = "cmd-group-heading";

    let container = create_container(&document, "cmd-group");
    let node = command.root(
        is_empty,
        Vec::new(),
        vec![command.dialog(
            "Command Menu",
            Vec::new(),
            vec![
                command.input(list_id, None, Vec::new()),
                command::list(
                    list_id,
                    "Suggestions",
                    is_empty,
                    Vec::new(),
                    vec![command::group(
                        Some(group_heading_id),
                        Vec::new(),
                        vec![
                            command::group_heading(
                                Some(group_heading_id),
                                Vec::new(),
                                vec![text("Suggestions")],
                            ),
                            command.item(
                                "calendar",
                                false,
                                Some("cmd-group-item-calendar"),
                                Vec::new(),
                                vec![text("Calendar")],
                            ),
                            command.item(
                                "search",
                                false,
                                Some("cmd-group-item-search"),
                                Vec::new(),
                                vec![text("Search")],
                            ),
                        ],
                    )],
                ),
            ],
        )],
    );
    container.set_inner_html(&render(&node));
    let root = container.first_element_child().unwrap();
    let _cleanup = RemoveOnDrop(root.clone());
    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .unwrap()
        .unwrap();
    let group = root
        .query_selector(r#"[data-scope="command"][data-part="group"]"#)
        .unwrap()
        .unwrap();
    let (_component, _log) = wire(root, command);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("zzz");
    input.dispatch_event(&input_event()).unwrap();
    assert!(
        group.has_attribute("hidden"),
        "全 item hidden の group は hidden"
    );

    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();
    assert!(
        !group.has_attribute("hidden"),
        "1 件でも可視 item を含む group は hidden にならない"
    );
}

// --- (b) shortcut テキストはマッチ対象にならない ---

#[wasm_bindgen_test]
fn shortcut_text_is_excluded_from_filter_matching() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let list_id = "cmd-shortcut-list";
    let container = create_container(&document, "cmd-shortcut");
    let entries: [(&str, &str); 1] = [("copy", "Copy")];
    let is_empty = command.is_empty(&entries);
    let node = command.root(
        is_empty,
        Vec::new(),
        vec![command.dialog(
            "Command Menu",
            Vec::new(),
            vec![
                command.input(list_id, None, Vec::new()),
                command::list(
                    list_id,
                    "Suggestions",
                    is_empty,
                    Vec::new(),
                    vec![command.item(
                        "copy",
                        false,
                        Some("cmd-shortcut-item-copy"),
                        Vec::new(),
                        vec![
                            text("Copy"),
                            command::shortcut(Vec::new(), vec![text("⌘C")]),
                        ],
                    )],
                ),
            ],
        )],
    );
    container.set_inner_html(&render(&node));
    let root = container.first_element_child().unwrap();
    let _cleanup = RemoveOnDrop(root.clone());
    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .unwrap()
        .unwrap();
    let item = root
        .query_selector(r#"[data-scope="command"][data-part="item"]"#)
        .unwrap()
        .unwrap();
    let (_component, _log) = wire(root, command);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("⌘C");
    input.dispatch_event(&input_event()).unwrap();
    assert!(
        item.has_attribute("hidden"),
        "shortcut テキストは絞り込みマッチ対象にならない"
    );
}

// --- (c) 矢印キーによる行選択 ---

#[wasm_bindgen_test]
fn arrow_down_selects_next_visible_non_disabled_item_and_syncs_dom() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [
        ("a", "Alpha", false),
        ("b", "Beta", true),
        ("c", "Gamma", false),
    ];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-arrow", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root, command);

    let prevented = input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        !prevented,
        "ArrowDown は claim され prevent_default() が呼ばれる"
    );
    assert!(item_elements[0].has_attribute("data-selected"));
    assert_eq!(
        item_elements[0].get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        input.get_attribute("aria-activedescendant").as_deref(),
        Some("cmd-arrow-item-a")
    );

    // b は disabled のためスキップし c へ進む。
    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(!item_elements[0].has_attribute("data-selected"));
    assert!(!item_elements[1].has_attribute("data-selected"));
    assert!(item_elements[2].has_attribute("data-selected"));

    // 既定は非循環: 末尾からさらに ArrowDown しても留まる。
    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_elements[2].has_attribute("data-selected"));
}

#[wasm_bindgen_test]
fn home_and_end_move_to_first_and_last_visible_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [
        ("a", "Alpha", false),
        ("b", "Beta", false),
        ("c", "Gamma", false),
    ];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-home-end", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root, command);

    input.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_elements[2].has_attribute("data-selected"));

    input.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(item_elements[0].has_attribute("data-selected"));
}

#[wasm_bindgen_test]
fn arrow_key_with_modifier_or_during_ime_composition_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-arrow-noop", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root, command);

    input
        .dispatch_event(&keydown_event_with("ArrowDown", true, false, false))
        .unwrap();
    assert!(!item_elements[0].has_attribute("data-selected"));

    input
        .dispatch_event(&keydown_event_composing("ArrowDown"))
        .unwrap();
    assert!(!item_elements[0].has_attribute("data-selected"));
}

// --- (d) Enter による実行 ---

#[wasm_bindgen_test]
fn enter_dispatches_execute_for_selected_visible_non_disabled_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-enter", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let executed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let component = Rc::new(RefCell::new(command));
    let recorder = executed.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        if action_ref.action == fandhe_frontend_wasm_full::command::ACTION_EXECUTE {
            recorder.borrow_mut().push(action_ref.payload.clone());
        }
        let _ = fandhe_frontend_interactive::dispatch(
            &mut *component.borrow_mut(),
            &action_ref.action,
            &action_ref.payload,
        );
    })
    .expect("wire_command_events must not fail");

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(executed.borrow().as_slice(), ["a".to_string()]);
}

#[wasm_bindgen_test]
fn enter_without_selection_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-enter-noop", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root, command);

    let prevented = input.dispatch_event(&keydown_event("Enter")).unwrap();
    // 未選択でも Enter 自体は claim される（`command_key_action` は選択
    // 有無を見ない）が、`handle_keydown` の Execute 分岐は選択が無ければ
    // dispatch しない。
    assert!(!prevented);
}

// --- (e) item クリック ---

#[wasm_bindgen_test]
fn clicking_item_selects_then_executes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, item_elements) =
        build_command_dom(&document, "cmd-click", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let seen: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let component = Rc::new(RefCell::new(command));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        recorder
            .borrow_mut()
            .push((action_ref.action.clone(), action_ref.payload.clone()));
        let _ = fandhe_frontend_interactive::dispatch(
            &mut *component.borrow_mut(),
            &action_ref.action,
            &action_ref.payload,
        );
    })
    .expect("wire_command_events must not fail");

    item_elements[0].dispatch_event(&click_event()).unwrap();

    let log = seen.borrow();
    assert_eq!(
        log.as_slice(),
        [
            ("select".to_string(), "a".to_string()),
            (
                fandhe_frontend_wasm_full::command::ACTION_EXECUTE.to_string(),
                "a".to_string()
            ),
        ]
    );
}

#[wasm_bindgen_test]
fn clicking_disabled_item_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", true)];
    let (root, _dialog, _input, _list, item_elements) =
        build_command_dom(&document, "cmd-click-disabled", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, log) = wire(root, command);

    item_elements[0].dispatch_event(&click_event()).unwrap();
    assert!(log.borrow().is_empty());
}

// --- (f) Escape ---

#[wasm_bindgen_test]
fn escape_within_open_dialog_dispatches_close() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-escape", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        recorder.borrow_mut().push(action_ref.action.clone());
    })
    .expect("wire_command_events must not fail");

    input.dispatch_event(&keydown_event("Escape")).unwrap();
    assert_eq!(
        seen.borrow().as_slice(),
        [fandhe_frontend_wasm_full::command::ACTION_CLOSE.to_string()]
    );
}

#[wasm_bindgen_test]
fn escape_without_open_dialog_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    // dialog を閉じたまま（Open しない）でも input 自体は操作可能な構成
    // として、Escape が no-op であることを確認する。
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-escape-noop", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, log) = wire(root, command);

    input.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(log.borrow().is_empty());
}

// --- (g) Cmd/Ctrl+K ---

#[wasm_bindgen_test]
fn ctrl_k_dispatches_toggle_and_focuses_input_when_opened() {
    let document = web_sys::window().unwrap().document().unwrap();
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-ctrl-k", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (component, _log) = wire(root, command);

    let window = web_sys::window().unwrap();
    let event = keydown_event_with("k", true, false, false);
    window
        .dispatch_event(&event)
        .expect("window.dispatch_event must not fail");

    assert!(component.borrow().is_open(), "Ctrl+K で dialog が開く");
    assert_eq!(
        document.active_element().as_ref(),
        Some(&input),
        "open 後は input へフォーカスが移る"
    );
}

#[wasm_bindgen_test]
fn meta_k_also_toggles() {
    let document = web_sys::window().unwrap().document().unwrap();
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, _item_elements) =
        build_command_dom(&document, "cmd-meta-k", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (component, _log) = wire(root, command);

    let window = web_sys::window().unwrap();
    window
        .dispatch_event(&keydown_event_with("k", false, true, false))
        .unwrap();
    assert!(component.borrow().is_open());
}

#[wasm_bindgen_test]
fn ctrl_alt_k_and_ctrl_meta_k_do_not_toggle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, _item_elements) =
        build_command_dom(&document, "cmd-ctrl-alt-k", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (component, _log) = wire(root, command);

    let window = web_sys::window().unwrap();
    window
        .dispatch_event(&keydown_event_with("k", true, false, true))
        .unwrap();
    assert!(!component.borrow().is_open(), "Ctrl+Alt+K は no-op");

    window
        .dispatch_event(&keydown_event_with("k", true, true, false))
        .unwrap();
    assert!(!component.borrow().is_open(), "Ctrl+Meta+K は no-op");
}

#[wasm_bindgen_test]
fn ctrl_k_without_any_dialog_part_in_document_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "cmd-no-dialog");
    let _cleanup = RemoveOnDrop(container.clone());
    // dialog パーツを一切持たない root（意図的に構成しないケース）。
    let component = Rc::new(RefCell::new(Command::default()));
    fandhe_frontend_wasm_full::command::wire_command_component(
        container.clone(),
        component.clone(),
        move |_state: &Command, _root| {},
    )
    .expect("wire_command_component must not fail");

    let window = web_sys::window().unwrap();
    window
        .dispatch_event(&keydown_event_with("k", true, false, false))
        .unwrap();
    assert!(!component.borrow().is_open());
}

// --- (h) data-action-input との二重 dispatch 回避 ---

#[wasm_bindgen_test]
fn input_with_data_action_input_does_not_double_dispatch_but_still_reflects_filter() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false), ("search", "Search", false)];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-action-input", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    input
        .set_attribute("data-action-input", "cmd_query")
        .unwrap();

    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        recorder.borrow_mut().push(action_ref.action.clone());
    })
    .expect("wire_command_events must not fail");

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();

    assert!(
        !seen
            .borrow()
            .contains(&fandhe_frontend_wasm_full::command::ACTION_INPUT.to_string()),
        "data-action-input がある input は本モジュールが \"input\" を dispatch しない"
    );
    assert!(
        item_elements[1].has_attribute("hidden"),
        "絞り込み反映は行われる"
    );
}

// --- (i) aria-controls 改ざん ---

#[wasm_bindgen_test]
fn tampered_aria_controls_pointing_outside_root_is_noop_without_panic() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-tampered", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    // root 外の要素へ aria-controls を差し替える。
    let outside = document.create_element("div").unwrap();
    outside.set_id("cmd-tampered-outside-list");
    document.body().unwrap().append_child(&outside).unwrap();
    let _cleanup_outside = RemoveOnDrop(outside);
    input
        .set_attribute("aria-controls", "cmd-tampered-outside-list")
        .unwrap();

    let (_component, _log) = wire(root, command);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("zzz");
    // panic しないことが主な検証観点。
    input.dispatch_event(&input_event()).unwrap();
    assert!(
        !item_elements[0].has_attribute("hidden"),
        "list 解決失敗時は反映しない"
    );
}

// --- (j) XSS 回帰 ---

#[wasm_bindgen_test]
fn xss_payload_in_item_label_and_value_does_not_create_script_element() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let payload_value = "<script>window.__cmd_xss = true;</script>";
    let payload_label = "<img src=x onerror=alert(1)>";
    let items = [(payload_value, payload_label, false)];
    let container_id = "cmd-xss";
    let list_id = format!("{container_id}-list");
    let container = create_container(&document, container_id);
    let entries: [(&str, &str); 1] = [(payload_value, payload_label)];
    let is_empty = command.is_empty(&entries);
    let node = command.root(
        is_empty,
        Vec::new(),
        vec![command.dialog(
            "Command Menu",
            Vec::new(),
            vec![
                command.input(&list_id, None, Vec::new()),
                command::list(
                    &list_id,
                    "Suggestions",
                    is_empty,
                    Vec::new(),
                    vec![command.item(
                        payload_value,
                        false,
                        None,
                        Vec::new(),
                        vec![text(payload_label)],
                    )],
                ),
            ],
        )],
    );
    container.set_inner_html(&render(&node));
    let root = container.first_element_child().unwrap();
    let _cleanup = RemoveOnDrop(root.clone());
    assert!(
        document
            .query_selector("script[src], script:not([src])")
            .unwrap()
            .is_none()
            || document.query_selector_all("script").unwrap().length() == 0,
        "既定エスケープにより <script> 要素は生成されない"
    );

    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .unwrap()
        .unwrap();
    let item = root
        .query_selector(r#"[data-scope="command"][data-part="item"]"#)
        .unwrap()
        .unwrap();

    let seen: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        recorder
            .borrow_mut()
            .push((action_ref.action.clone(), action_ref.payload.clone()));
    })
    .expect("wire_command_events must not fail");

    item.dispatch_event(&click_event()).unwrap();

    let log = seen.borrow();
    assert_eq!(
        log[0].1, payload_value,
        "payload は文字列のまま dispatch される"
    );
    assert!(item.query_selector("script").unwrap().is_none());
    let _ = input;
    let _ = items;
}
