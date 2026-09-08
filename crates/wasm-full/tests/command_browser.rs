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
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventInit, HtmlElement, HtmlInputElement, KeyboardEvent,
    KeyboardEventInit,
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

/// `Element` を `HtmlElement` へキャストする（スクロール回帰テスト向けの
/// `style()`/`scroll_top()` 操作用、`keynav_browser.rs::html_element` と
/// 同型）。
fn html_element(element: &Element) -> HtmlElement {
    element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must be an HtmlElement")
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
///
/// `wire_command_component` を直接使わず `wire_command_events` +
/// `fandhe_frontend_interactive::dispatch` を手動で組み合わせる
/// （`wire_command_component` 内部と同型の呼び出し順）。理由は 2 点:
///
/// 1. dispatch 依頼された `ActionRef` を [`ActionLog`] へ記録する
///    （dispatch されなかった＝no-op を主張するテスト
///    （`clicking_disabled_item_is_noop`/`escape_without_open_dialog_is_noop`）
///    が実際に空ログのままであることを検証できるようにする。以前は
///    `log` 変数が一切書き込まれずに宣言だけされており、これらのテストは
///    常に空のまま無条件通過していた、Bugbot Medium 是正）。
/// 2. dispatch 後、`dialog` パーツの `hidden` を `state.is_open()` へ反映
///    する（`number_input_browser.rs::wire_with_dom_reflection` と同型の
///    最小限の DOM 反映。`crate::command::wiring::handle_document_keydown`
///    は dispatch 後に生きた DOM の `dialog` を再解決して `hidden` の有無
///    で input へ `focus()` するかを判定するため、この反映が無いと
///    Cmd/Ctrl+K 後の focus 検証が実利用者の再描画を経ない分だけ成立しない、
///    Bugbot Medium 是正）。
fn wire(root: Element, command: Command) -> (Rc<RefCell<Command>>, ActionLog) {
    let component = Rc::new(RefCell::new(command));
    let log: ActionLog = Rc::new(RefCell::new(Vec::new()));
    let dispatch_component = component.clone();
    let reflect_root = root.clone();
    let record_log = log.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root, move |action_ref| {
        record_log
            .borrow_mut()
            .push((action_ref.action.clone(), action_ref.payload.clone()));
        let Ok(mut state) = dispatch_component.try_borrow_mut() else {
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
        if let Ok(Some(dialog)) =
            reflect_root.query_selector(r#"[data-scope="command"][data-part="dialog"]"#)
        {
            if state.is_open() {
                let _ = dialog.remove_attribute("hidden");
            } else {
                let _ = dialog.set_attribute("hidden", "");
            }
        }
    })
    .expect("wire_command_events must not fail");
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

/// `container_id` を `id` に持つコンテナ要素を `parent` 配下へ組み立てる、
/// [`build_command_dom`] の「任意の親へ配置できる」版。`typing_query_
/// after_full_subtree_replacement_still_reflects_filter` が、構造フォール
/// バックによる `[data-part="root"]` 丸ごと差し替えを模すために使う
/// （`keynav_browser.rs::build_combobox_dom` と同型の意図）。
fn build_command_into(
    document: &Document,
    parent: &Element,
    container_id: &str,
    command: &Command,
    items: &[(&str, &str, bool)],
) -> (Element, Element, Vec<Element>) {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(container_id);
    let list_id = format!("{container_id}-list");
    let entries: Vec<(&str, &str)> = items.iter().map(|(v, l, _)| (*v, *l)).collect();
    let is_empty = command.is_empty(&entries);
    let item_nodes: Vec<_> = items
        .iter()
        .map(|(value, label, disabled)| {
            command.item(value, *disabled, None, Vec::new(), vec![text(*label)])
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
            ],
        )],
    );
    container.set_inner_html(&render(&node));
    parent
        .append_child(&container)
        .expect("append_child must not fail");
    let root = container
        .first_element_child()
        .expect("command root must exist");
    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .expect("query_selector must not fail")
        .expect("input element must exist");
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
    (container, input, item_elements)
}

/// [`build_command_into`] と同型だが item 要素に安定 id
/// （`{container_id}-item-{value}`）を付与する。`aria-activedescendant` の
/// 検証（id 経由でしか参照できない）に使う構造フォールバック回帰テスト
/// 専用の亜種。
fn build_command_into_with_ids(
    document: &Document,
    parent: &Element,
    container_id: &str,
    command: &Command,
    items: &[(&str, &str, bool)],
) -> (Element, Element, Vec<Element>) {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(container_id);
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
            ],
        )],
    );
    container.set_inner_html(&render(&node));
    parent
        .append_child(&container)
        .expect("append_child must not fail");
    let root = container
        .first_element_child()
        .expect("command root must exist");
    let input = root
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .expect("query_selector must not fail")
        .expect("input element must exist");
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
    (container, input, item_elements)
}

/// codex-review P1・Bugbot High 是正の回帰テスト: 絞り込みの dispatch
/// （[`fandhe_frontend_wasm_full::command::ACTION_INPUT`]）が構造フォール
/// バック（`[data-part="root"]` を含むコンテナ丸ごとの差し替え）を誘発
/// しても、`hidden` 反映が失われず**生きた（新しい）DOM**へ正しく届くこと
/// を検証する（`crates/wasm-full/src/command.rs` モジュール冒頭 doc
/// 「DOM 同期契約」節、`keynav_browser.rs::
/// combobox_closed_arrow_down_opens_after_full_subtree_replacement_still_sets_highlight`
/// と同型のシナリオ）。修正前の実装（dispatch 前に解決した要素へ直接
/// 書き込む）ではこのテストは古い（detach 済みの）item へ `hidden` を
/// 書いてしまい、生きた DOM 側の `search` item が可視のまま残って失敗する。
#[wasm_bindgen_test]
fn typing_query_after_full_subtree_replacement_still_reflects_filter() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = "cmd-replace";
    let items: [(&str, &str, bool); 2] = [
        ("calendar", "Calendar", false),
        ("search", "Search Emoji", false),
    ];
    let mut command = Command::default();
    command.update(CommandAction::Open);

    // Command のマウント境界（`wire_command_events` へ渡す root）は
    // コンテナより外側の安定コンテナとする（mount root 自体は差し替え
    // ない。実アプリでの「Command サブツリーだけが再描画で差し替わり、
    // mount root 自体は永続する」構成を模す）。
    let mount_root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&mount_root).unwrap();
    let _cleanup = RemoveOnDrop(mount_root.clone());

    let (_container, input, _item_elements) =
        build_command_into(&document, &mount_root, container_id, &command, &items);
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();

    // dispatch のたびに、`container_id` のコンテナ配下全体を detach し、
    // 同じ id を持つ新しいコンテナへ丸ごと差し替える（アプリの再描画を
    // 模す）。新しい DOM は絞り込み未反映＝全 item 可視の状態で構築する
    // （絞り込みは本モジュールの DOM-only な関心事であり、アプリの
    // `view()` はこれを知らない、モジュール冒頭 doc「絞り込みの DOM
    // 反映」節参照）。
    let state: Rc<RefCell<Command>> = Rc::new(RefCell::new(command));
    let rebuild_document = document.clone();
    let rebuild_mount_root = mount_root.clone();
    let rebuild_container_id = container_id.to_string();
    let rebuild_state = state.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(
        mount_root.clone(),
        move |action_ref| {
            let Ok(mut command_state) = rebuild_state.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *command_state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Some(old_container) = rebuild_document.get_element_by_id(&rebuild_container_id) {
                old_container.remove();
            }
            build_command_into(
                &rebuild_document,
                &rebuild_mount_root,
                &rebuild_container_id,
                &command_state,
                &items,
            );
        },
    )
    .expect("wire_command_events must not fail");

    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();

    // 生きた（新しい）DOM を id 経由で再解決して検証する（`input`/
    // `_item_elements` は差し替え前の detach 済み要素のまま残っている）。
    let live_container = document
        .get_element_by_id(container_id)
        .expect("replaced container must exist");
    let live_calendar = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="calendar"]"#)
        .unwrap()
        .expect("live calendar item must exist");
    let live_search = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="search"]"#)
        .unwrap()
        .expect("live search item must exist");
    assert!(
        !live_calendar.has_attribute("hidden"),
        "構造フォールバック後も生きた DOM の calendar は可視のまま"
    );
    assert!(
        live_search.has_attribute("hidden"),
        "構造フォールバック後も生きた DOM の search へ hidden が反映される \
         （codex-review P1・Bugbot High 是正の回帰）"
    );
}

/// codex-review P1（選択 dispatch 後にも絞り込み状態を再反映する）・
/// Bugbot High（command.rs#L1084-L1096）是正の回帰テスト: 絞り込み後に
/// `ArrowDown` を押すと（可視候補が複数あり選択位置が実際に動くため）
/// [`fandhe_frontend_wasm_full::command::ACTION_SELECT`] が dispatch され、
/// それが構造フォールバック（コンテナ丸ごとの差し替え）を誘発する。この
/// 再描画はフィルタ関心（DOM-only）を知らないため `search` item は可視の
/// まま出力される。`handle_keydown` の `MoveSelection` 分岐が dispatch 後に
/// `reflect_filter` で再同期しない修正前の実装では、この再描画後も
/// `search` が可視のままになり、続くキー操作で非一致候補を選択・実行
/// できてしまう（本テストの意図）。
#[wasm_bindgen_test]
fn arrow_key_after_dispatch_re_syncs_filter_across_full_subtree_replacement() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = "cmd-arrow-replace";
    // "cal" は calendar/calzone の 2 件に一致し search には一致しない。
    // 可視候補が 2 件あることで ArrowDown が実際に選択位置を動かし
    // （dispatch が起きない single-item 構成を避ける）、`search` を
    // hidden のまま維持できるかを検証できる。
    let items: [(&str, &str, bool); 3] = [
        ("calendar", "Calendar", false),
        ("calzone", "Calzone Recipe", false),
        ("search", "Search Emoji", false),
    ];
    let mut command = Command::default();
    command.update(CommandAction::Open);

    let mount_root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&mount_root).unwrap();
    let _cleanup = RemoveOnDrop(mount_root.clone());

    let (_container, input, _item_elements) =
        build_command_into(&document, &mount_root, container_id, &command, &items);
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();

    let state: Rc<RefCell<Command>> = Rc::new(RefCell::new(command));
    let rebuild_document = document.clone();
    let rebuild_mount_root = mount_root.clone();
    let rebuild_container_id = container_id.to_string();
    let rebuild_state = state.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(
        mount_root.clone(),
        move |action_ref| {
            let Ok(mut command_state) = rebuild_state.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *command_state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Some(old_container) = rebuild_document.get_element_by_id(&rebuild_container_id) {
                old_container.remove();
            }
            build_command_into(
                &rebuild_document,
                &rebuild_mount_root,
                &rebuild_container_id,
                &command_state,
                &items,
            );
        },
    )
    .expect("wire_command_events must not fail");

    // "cal" は calendar/calzone の 2 件に一致（search は非一致）。
    // ACTION_INPUT の dispatch が再描画を誘発するが、直後の
    // `reflect_filter` が hidden を生きた DOM へ反映し、先頭一致
    // （calendar）を自動選択する（既存テストと同型の前提状態）。
    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();

    let live_input = document
        .get_element_by_id(container_id)
        .and_then(|c| {
            c.query_selector(r#"[data-scope="command"][data-part="input"]"#)
                .ok()
        })
        .flatten()
        .expect("live input must exist after ACTION_INPUT rebuild");
    let live_input = live_input.dyn_into::<HtmlInputElement>().unwrap();

    // ArrowDown を押す: 可視候補は calendar/calzone の 2 件のため選択位置が
    // calendar → calzone へ実際に動き、[`fandhe_frontend_wasm_full::command::
    // ACTION_SELECT`] が dispatch され、構造フォールバック再描画を誘発する
    // （本テストの wiring は dispatch のたびに必ず再構築する）。
    live_input
        .dyn_ref::<Element>()
        .unwrap()
        .dispatch_event(&keydown_event("ArrowDown"))
        .unwrap();

    let live_container = document
        .get_element_by_id(container_id)
        .expect("replaced container must exist after ArrowDown dispatch");
    let live_calendar = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="calendar"]"#)
        .unwrap()
        .expect("live calendar item must exist");
    let live_calzone = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="calzone"]"#)
        .unwrap()
        .expect("live calzone item must exist");
    let live_search = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="search"]"#)
        .unwrap()
        .expect("live search item must exist");

    assert!(
        !live_calendar.has_attribute("hidden"),
        "ArrowDown 後も一致候補 calendar は可視のまま"
    );
    assert!(
        !live_calzone.has_attribute("hidden"),
        "ArrowDown 後も一致候補 calzone は可視のまま"
    );
    assert!(
        live_search.has_attribute("hidden"),
        "ArrowDown が誘発した構造フォールバック再描画後も、非一致候補 \
         search は hidden のまま復元される（codex-review P1・Bugbot High \
         是正の回帰。修正前は search が再表示され、次のキー操作で \
         選択・実行できてしまっていた）"
    );
    assert!(
        !live_calendar.has_attribute("data-selected"),
        "ArrowDown で選択は calendar から離れる"
    );
    assert!(
        live_calzone.has_attribute("data-selected"),
        "ArrowDown 後は calzone が選択状態になる"
    );
}

/// codex-review P1（選択 dispatch 後にも絞り込み状態を再反映する）・
/// Bugbot High（command.rs#L1185-L1201）是正の回帰テスト: 絞り込み後に
/// 可視候補（絞り込み前と別の item）をクリックすると
/// [`fandhe_frontend_wasm_full::command::ACTION_SELECT`]/
/// [`fandhe_frontend_wasm_full::command::ACTION_EXECUTE`] が dispatch され、
/// それが構造フォールバック再描画を誘発する。`handle_click` が dispatch 後に
/// `reflect_filter` で再同期しない修正前の実装では、この再描画後も
/// 非一致候補が可視のまま残る（`arrow_key_after_dispatch_...` と同型の
/// シナリオ、契機がクリックである点のみ異なる）。
#[wasm_bindgen_test]
fn clicking_item_after_filter_re_syncs_filter_across_full_subtree_replacement() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = "cmd-click-replace";
    let items: [(&str, &str, bool); 3] = [
        ("calendar", "Calendar", false),
        ("calzone", "Calzone Recipe", false),
        ("search", "Search Emoji", false),
    ];
    let mut command = Command::default();
    command.update(CommandAction::Open);

    let mount_root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&mount_root).unwrap();
    let _cleanup = RemoveOnDrop(mount_root.clone());

    let (_container, input, _item_elements) =
        build_command_into(&document, &mount_root, container_id, &command, &items);
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();

    let state: Rc<RefCell<Command>> = Rc::new(RefCell::new(command));
    let rebuild_document = document.clone();
    let rebuild_mount_root = mount_root.clone();
    let rebuild_container_id = container_id.to_string();
    let rebuild_state = state.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(
        mount_root.clone(),
        move |action_ref| {
            let Ok(mut command_state) = rebuild_state.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *command_state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Some(old_container) = rebuild_document.get_element_by_id(&rebuild_container_id) {
                old_container.remove();
            }
            build_command_into(
                &rebuild_document,
                &rebuild_mount_root,
                &rebuild_container_id,
                &command_state,
                &items,
            );
        },
    )
    .expect("wire_command_events must not fail");

    // "cal" は calendar/calzone の 2 件に一致（search は非一致）。
    html_input.set_value("cal");
    input.dispatch_event(&input_event()).unwrap();

    let live_calzone_before_click = document
        .get_element_by_id(container_id)
        .and_then(|c| {
            c.query_selector(r#"[data-scope="command"][data-part="item"][data-value="calzone"]"#)
                .ok()
        })
        .flatten()
        .expect("live calzone item must exist after ACTION_INPUT rebuild");

    // calzone（先頭自動選択された calendar とは別の可視候補）をクリック:
    // ACTION_SELECT → ACTION_EXECUTE が dispatch され、構造フォールバック
    // 再描画を誘発する。
    live_calzone_before_click
        .dispatch_event(&click_event())
        .unwrap();

    let live_container = document
        .get_element_by_id(container_id)
        .expect("replaced container must exist after click dispatch");
    let live_calendar = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="calendar"]"#)
        .unwrap()
        .expect("live calendar item must exist");
    let live_search = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="search"]"#)
        .unwrap()
        .expect("live search item must exist");

    assert!(
        !live_calendar.has_attribute("hidden"),
        "クリック後も一致候補 calendar は可視のまま"
    );
    assert!(
        live_search.has_attribute("hidden"),
        "クリックが誘発した構造フォールバック再描画後も、非一致候補 \
         search は hidden のまま復元される（codex-review P1・Bugbot High \
         是正の回帰。修正前は search が再表示されていた）"
    );
}

/// codex-review P1（選択維持時も `aria-activedescendant` を同期する）・
/// Bugbot Medium（command.rs#L667-L670）是正の回帰テスト: 絞り込みを
/// 2 段階で進め、2 段階目で選択中候補が可視のまま維持される
/// （[`SelectionPlan::Keep`]）状況を作る。構造フォールバック再描画（呼び
/// 出し側は `Command::input` へ `activedescendant: None` を渡す構成、
/// 実アプリで選択中 item の id を把握せず再描画するケースを模す）を経ても、
/// `input` の `aria-activedescendant` が選択中 item の id を指したまま
/// 復元されることを検証する。修正前の `SelectionPlan::Keep => {}` は
/// 完全な no-op であり、再描画後の新しい `input` に
/// `aria-activedescendant` が一切設定されないままになっていた。
#[wasm_bindgen_test]
fn typing_further_while_selection_unchanged_restores_aria_activedescendant_after_replacement() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = "cmd-keep-replace";
    let items: [(&str, &str, bool); 2] = [
        ("calendar", "Calendar", false),
        ("search", "Search Emoji", false),
    ];
    let mut command = Command::default();
    command.update(CommandAction::Open);

    let mount_root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&mount_root).unwrap();
    let _cleanup = RemoveOnDrop(mount_root.clone());

    let (_container, input, _item_elements) =
        build_command_into_with_ids(&document, &mount_root, container_id, &command, &items);
    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();

    let state: Rc<RefCell<Command>> = Rc::new(RefCell::new(command));
    let rebuild_document = document.clone();
    let rebuild_mount_root = mount_root.clone();
    let rebuild_container_id = container_id.to_string();
    let rebuild_state = state.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(
        mount_root.clone(),
        move |action_ref| {
            let Ok(mut command_state) = rebuild_state.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *command_state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Some(old_container) = rebuild_document.get_element_by_id(&rebuild_container_id) {
                old_container.remove();
            }
            // 実アプリの `view()` は選択中 item の id を個別に把握せず
            // `Command::input` へ `None` を渡すことがある（本テストの
            // 意図する構成、上記 doc 参照）。
            build_command_into_with_ids(
                &rebuild_document,
                &rebuild_mount_root,
                &rebuild_container_id,
                &command_state,
                &items,
            );
        },
    )
    .expect("wire_command_events must not fail");

    // 1 段階目: "ca" は calendar のみ一致 → 自動選択（Select 分岐）。
    html_input.set_value("ca");
    input.dispatch_event(&input_event()).unwrap();

    // 2 段階目: "cal" も calendar のみ一致のまま → 選択は変わらず
    // （Keep 分岐）だが、直前の ACTION_INPUT による構造フォールバック
    // 再描画で `aria-activedescendant` は失われた状態から始まる。
    let live_input_after_first = document
        .get_element_by_id(container_id)
        .and_then(|c| {
            c.query_selector(r#"[data-scope="command"][data-part="input"]"#)
                .ok()
        })
        .flatten()
        .expect("live input must exist after first ACTION_INPUT rebuild")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    live_input_after_first.set_value("cal");
    live_input_after_first
        .dyn_ref::<Element>()
        .unwrap()
        .dispatch_event(&input_event())
        .unwrap();

    let live_container = document
        .get_element_by_id(container_id)
        .expect("replaced container must exist after second ACTION_INPUT dispatch");
    let live_input = live_container
        .query_selector(r#"[data-scope="command"][data-part="input"]"#)
        .unwrap()
        .expect("live input must exist");
    let live_calendar = live_container
        .query_selector(r#"[data-scope="command"][data-part="item"][data-value="calendar"]"#)
        .unwrap()
        .expect("live calendar item must exist");

    assert!(
        live_calendar.has_attribute("data-selected"),
        "2 段階目でも calendar は選択状態のまま（Keep 分岐）"
    );
    assert_eq!(
        live_input.get_attribute("aria-activedescendant").as_deref(),
        Some(format!("{container_id}-item-calendar").as_str()),
        "Keep 分岐でも input の aria-activedescendant が選択中 item を \
         指すよう復元される（codex-review P1・Bugbot Medium 是正の回帰。 \
         修正前は Keep 分岐が no-op のため、再描画で失われた \
         aria-activedescendant が復元されなかった）"
    );
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

/// codex-review P1 是正の回帰テスト: `dialog` パーツに
/// `data-close-on-escape="false"`（`crate::overlay::close_on_escape_for`
/// の opt-out 属性）が付いている場合、open な dialog 内の Escape でも
/// [`fandhe_frontend_wasm_full::command::ACTION_CLOSE`] を dispatch しない
/// こと、および `keydown_event` の既定動作を妨げない（`prevent_default()`
/// を呼ばない）ことを検証する（`crates/wasm-full/src/command.rs` モジュール
/// 冒頭 doc「`OverlayKind::Command` と Escape の収束」節）。修正前の実装は
/// この属性を無視して無条件に dispatch していた。
#[wasm_bindgen_test]
fn escape_with_data_close_on_escape_false_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let container_id = "cmd-escape-optout";
    let list_id = format!("{container_id}-list");
    let container = create_container(&document, container_id);
    let entries: [(&str, &str); 1] = [("a", "Alpha")];
    let is_empty = command.is_empty(&entries);
    let node = command.root(
        is_empty,
        Vec::new(),
        vec![command.dialog(
            "Command Menu",
            vec![("data-close-on-escape", "false")],
            vec![
                command.input(&list_id, None, Vec::new()),
                command::list(
                    &list_id,
                    "Suggestions",
                    is_empty,
                    Vec::new(),
                    vec![command.item(
                        "a",
                        false,
                        Some("cmd-escape-optout-item-a"),
                        Vec::new(),
                        vec![text("Alpha")],
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
    let (_component, log) = wire(root, command);

    let default_not_prevented = input.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(
        default_not_prevented,
        "data-close-on-escape=\"false\" のとき prevent_default() は呼ばれない"
    );
    assert!(
        log.borrow().is_empty(),
        "data-close-on-escape=\"false\" のとき close は dispatch されない"
    );
    let _ = items;
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
    // `document` 全体ではなく `root`（本テストが描画した部分木）に限定
    // する。`document` 全体を対象にすると wasm-bindgen-test ハーネス自身が
    // ページに埋め込む `<script>`（テストバンドル読み込み用）や他テストの
    // 残置要素まで拾ってしまい、本コンポーネントの既定エスケープが正しく
    // 機能していても常に失敗する（他 `*_browser.rs` の同種 XSS 回帰テストは
    // いずれも `root`/`container` 限定、本テストのみ `document` 限定だった
    // ことが原因の flaky FAIL、Bugbot/CI 指摘是正）。
    assert!(
        root.query_selector("script").unwrap().is_none(),
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

// --- (i) 無効化された Command での開閉ショートカット（codex-review P1、イシュー #2069） ---

/// Command root に `data-disabled` を付けたとき、`document` 上の
/// Cmd/Ctrl+K が [`fandhe_frontend_headless_ui::command::CommandAction::
/// Toggle`] を dispatch せず、`input` へのフォーカス移動も起きないことを
/// 検証する（codex-review P1 是正: 従来は dialog の存在だけで toggle が
/// dispatch され、`handle_keydown`/`handle_input`/`handle_click` が採用する
/// `has_disabled_ancestor` の無効化契約と不整合だった）。
#[wasm_bindgen_test]
fn disabled_root_ignores_toggle_shortcut_and_does_not_focus_input() {
    let document = web_sys::window().unwrap().document().unwrap();
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-disabled-toggle", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    root.set_attribute("data-disabled", "").unwrap();
    let (component, _log) = wire(root, command);

    let window = web_sys::window().unwrap();
    window
        .dispatch_event(&keydown_event_with("k", true, false, false))
        .unwrap();

    assert!(
        !component.borrow().is_open(),
        "data-disabled な Command は Ctrl+K で開かない"
    );
    assert_ne!(
        document.active_element().as_ref(),
        Some(&input),
        "無効化時は input へフォーカスが移らない"
    );
}

// --- (j) 選択項目のスクロール追随（codex-review P1、イシュー #2069） ---

/// `list` パーツに `overflow-y: auto` + 固定 `height`（アプリ側のスクロール
/// 可能な候補リスト構成、`keynav_browser.rs::
/// select_open_arrow_down_scrolls_highlighted_item_into_view_when_content_overflows`
/// と同型の可視領域制約）を与え、末尾項目（初期スクロール位置では不可視）
/// まで ArrowDown で選択を移動させると `list.scroll_top` が 0 から動く
/// （スクロール追随が発生したことの直接証拠）ことを検証する（codex-review
/// P1 是正: `crate::keynav::wiring::scroll_item_into_view_if_needed` を
/// Command の `sync_selection`/`write_selection_plan` から再利用した回帰
/// 固定）。
#[wasm_bindgen_test]
fn arrow_down_scrolls_selected_item_into_view_when_list_overflows() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items: Vec<(&str, &str, bool)> = (0..20)
        .map(|i| {
            let leaked: &'static str = Box::leak(format!("item{i}").into_boxed_str());
            (leaked, leaked, false)
        })
        .collect();
    let (root, _dialog, input, list, item_elements) =
        build_command_dom(&document, "cmd-scroll", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let list_html = html_element(&list);
    list_html
        .style()
        .set_property("overflow-y", "auto")
        .unwrap();
    list_html.style().set_property("height", "80px").unwrap();
    list_html.style().set_property("display", "block").unwrap();
    for item in &item_elements {
        let item_html = html_element(item);
        item_html.style().set_property("height", "30px").unwrap();
        item_html.style().set_property("display", "block").unwrap();
    }

    let (_component, _log) = wire(root, command);
    assert_eq!(list_html.scroll_top(), 0);

    for _ in 0..20 {
        input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    }

    let last_item = &item_elements[19];
    assert!(last_item.has_attribute("data-selected"));
    assert!(
        list_html.scroll_top() > 0,
        "選択項目がスクロール可能な list 外へ出たら list.scroll_top が 0 から動く"
    );
}
