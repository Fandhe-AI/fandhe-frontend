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
    KeyboardEventInit, MouseEvent, MouseEventInit,
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

/// [`keydown_event_with`] に `repeat: true` を加えたもの（Cursor Bugbot 是正
/// の回帰テストで使う、押しっぱなしによるキーリピート keydown の模擬）。
fn keydown_event_with_repeat(key: &str, ctrl: bool, meta: bool, alt: bool) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(ctrl);
    init.set_meta_key(meta);
    init.set_alt_key(alt);
    init.set_repeat(true);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// Shift 押下のみを付けた合成 `"keydown"` イベント（codex-review P1 是正の
/// 回帰テストで使う。検索欄でのテキスト範囲選択操作、例えば
/// Shift+Home/Shift+End/Shift+ArrowDown が候補選択へ奪われないことを
/// 検証する）。純粋層（[`command::command_key_action`] 相当）は `Modifiers`
/// （Ctrl/Alt/Meta のみ）しか知らないため Shift 単体は native テストでは
/// 検証できず、本ブラウザテストでのみ固定できる。
fn keydown_event_with_shift(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_shift_key(true);
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

/// IME 変換中（`isComposing: true`）の `"input"` イベント。
/// `crate::events::wiring::wire_events` の入力 dispatch ガード（codex-review
/// P1 是正、イシュー #2069）の回帰テストで使う。
fn composing_input_event() -> Event {
    let init = web_sys::InputEventInit::new();
    init.set_bubbles(true);
    init.set_is_composing(true);
    web_sys::InputEvent::new_with_event_init_dict("input", &init)
        .expect("InputEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("InputEvent must cast to Event")
}

/// IME 変換確定（`"compositionend"`）イベント。`handle_compositionend`
/// （codex-review P1 是正、イシュー #2069）の回帰テストで使う。
fn compositionend_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("compositionend", &init).expect("Event::new must not fail")
}

/// `bubbles: true`・`cancelable: true` の合成 `"mousedown"` イベント
/// （`handle_mousedown` の回帰テストで使う、`focus_visible_browser.rs::
/// dispatch_mouse_event` と同型）。
fn mousedown_event() -> Event {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    MouseEvent::new_with_mouse_event_init_dict("mousedown", &init)
        .expect("MouseEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("MouseEvent must cast to Event")
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

/// Shift 付きの ArrowDown/Home/End は no-op（`prevent_default()` されず、
/// 選択状態も変化せず、`on_action` へも何も dispatch されない）ことを
/// 検証する（codex-review P1 是正、イシュー #2069）。`Modifiers`
/// （Ctrl/Alt/Meta のみ）を経由する既存の
/// `arrow_key_with_modifier_or_during_ime_composition_is_noop` では
/// Shift 単体を再現できないため独立したテストとして追加する。省略すると
/// 検索欄で Shift+Home/Shift+End によるテキスト範囲選択（ブラウザ既定
/// 動作）が候補選択へ奪われてしまう。
#[wasm_bindgen_test]
fn shift_arrow_home_end_is_noop_and_does_not_prevent_default() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false), ("b", "Beta", false)];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-shift-arrow-noop", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, log) = wire(root, command);

    for key in ["ArrowDown", "ArrowUp", "Home", "End"] {
        let event = keydown_event_with_shift(key);
        input.dispatch_event(&event).unwrap();
        assert!(
            !event.default_prevented(),
            "Shift+{key} は claim されず prevent_default() が呼ばれない"
        );
    }
    assert!(!item_elements[0].has_attribute("data-selected"));
    assert!(!item_elements[1].has_attribute("data-selected"));
    assert!(
        log.borrow().is_empty(),
        "Shift 付きキー操作は on_action へ何も dispatch しない"
    );
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

/// `event.target()` が item ラベルのテキストノード（`fandhe_frontend_core::
/// text` で描画される、`vec![text(*label)]` の子ノード）であっても
/// `select`/`command:execute` が dispatch されることを検証する（Cursor
/// Bugbot High 是正、イシュー #2069）。従来は `target` の `Element` への
/// キャストのみを試み、失敗時に即 return していたため、item のラベル
/// 文字列を直接クリックすると `closest(ITEM_SELECTOR)` に到達できず、
/// 通常のクリック操作でコマンドが一切実行されなかった
/// （`handle_mousedown` 側は同じ状況で `parent_element` へフォールバック
/// しており、`handle_click` 側のみ取りこぼしていた不整合、
/// `mousedown_on_text_node_target_prevents_default` と対）。
#[wasm_bindgen_test]
fn clicking_item_text_label_selects_then_executes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, item_elements) =
        build_command_dom(&document, "cmd-click-text-node", &command, &items);
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

    let text_node = item_elements[0]
        .first_child()
        .expect("item element must contain a text node child (label)");
    assert_eq!(
        text_node.node_type(),
        web_sys::Node::TEXT_NODE,
        "item ラベルはテキストノードとして描画される前提"
    );

    text_node.dispatch_event(&click_event()).unwrap();

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

/// item 内に配置された独立インタラクティブ要素（`button`）をクリックしても
/// 祖先 item を解決した `select`/`command:execute` が dispatch されないこと
/// を検証する（Cursor Bugbot Medium 是正、イシュー #2069）。従来は
/// `handle_mousedown` が既定動作を尊重する一方 `handle_click` のみが
/// `INDEPENDENT_INTERACTIVE_SELECTOR` を見ずに祖先 item まで遡って
/// dispatch し `stop_propagation()` まで行っていた
/// （`mousedown_on_independent_select_inside_dialog_is_not_prevented` と
/// 対で click 経路を固定する）。item 本体（テキストラベル部分）への
/// クリックは従来どおり 2 アクション dispatch されることも併せて確認する。
#[wasm_bindgen_test]
fn clicking_independent_control_inside_item_does_not_dispatch() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, item_elements) =
        build_command_dom(&document, "cmd-click-independent-in-item", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    // item 内に利用者が併設した独立コントロール（例: お気に入り登録
    // ボタン）を追加する。Command のパーツではない。
    let button = document
        .create_element("button")
        .expect("create_element must not fail");
    item_elements[0]
        .append_child(&button)
        .expect("append_child must not fail");

    let (_component, log) = wire(root, command);

    button.dispatch_event(&click_event()).unwrap();
    assert!(
        log.borrow().is_empty(),
        "item 内の独立コントロールのクリックは select/command:execute を \
         dispatch しない"
    );

    // item 本体（テキストラベル）への通常クリックは従来どおり機能する。
    let text_node = item_elements[0]
        .first_child()
        .expect("item element must contain a text node child (label)");
    text_node.dispatch_event(&click_event()).unwrap();
    assert_eq!(
        log.borrow().as_slice(),
        [
            ("select".to_string(), "a".to_string()),
            (
                fandhe_frontend_wasm_full::command::ACTION_EXECUTE.to_string(),
                "a".to_string()
            ),
        ]
    );
}

/// Command を Tabs content や ScrollArea viewport 等、`tabindex="0"` を
/// 固定で持つ外側パネルの内側へ合成配置した場合でも、通常の item クリック
/// が `select`/`command:execute` を dispatch し続けることを検証する
/// （codex-review P1 是正、イシュー #2069）。`INDEPENDENT_INTERACTIVE_
/// SELECTOR` の除外判定は `Element::closest` が祖先方向へ無制限に探索
/// する性質上、`root` の外側にある外側パネルの `tabindex="0"` まで誤って
/// 一致させてしまい、通常の item クリックまで無効化する不具合があった。
/// 一致した要素が `root` 配下に実在する場合のみ除外対象とする修正の回帰
/// テスト。
#[wasm_bindgen_test]
fn clicking_item_inside_ancestor_tabindex_panel_still_dispatches() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, item_elements) = build_command_dom(
        &document,
        "cmd-click-ancestor-tabindex-panel",
        &command,
        &items,
    );

    // Command の外側（例: Tabs content パネルや ScrollArea viewport）を
    // 模した `tabindex="0"` 固定の祖先パネルへ container ごと包む。
    let container = root
        .parent_element()
        .expect("build_command_dom container must exist as root's parent");
    let panel = document
        .create_element("div")
        .expect("create_element must not fail");
    panel
        .set_attribute("tabindex", "0")
        .expect("set_attribute must not fail");
    document
        .body()
        .expect("document body must exist")
        .append_child(&panel)
        .expect("append_child must not fail");
    panel
        .append_child(&container)
        .expect("append_child must not fail");
    let _cleanup = RemoveOnDrop(panel);

    let (_component, log) = wire(root, command);

    let text_node = item_elements[0]
        .first_child()
        .expect("item element must contain a text node child (label)");
    text_node.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        log.borrow().as_slice(),
        [
            ("select".to_string(), "a".to_string()),
            (
                fandhe_frontend_wasm_full::command::ACTION_EXECUTE.to_string(),
                "a".to_string()
            ),
        ],
        "外側パネルの tabindex=\"0\" に誤って一致し、通常の item クリック \
         まで無効化されてはならない"
    );
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

/// Cursor Bugbot 是正の回帰テスト（イシュー #2069）: Cmd/Ctrl+K の
/// 押しっぱなしによるキーリピート（`repeat: true`）の keydown でも
/// `prevent_default()` が呼ばれる（＝ `window.dispatch_event` の戻り値が
/// `false`）ことを検証する。修正前はキーリピート guard が
/// `prevent_default()` より前に return していたため、2 回目以降の
/// keydown でブラウザ既定のショートカット（Chrome の検索/アドレスバー等）
/// を止められなかった。dispatch（toggle の再発火）自体は repeat 中は
/// 引き続き無視されることも合わせて確認する（チラつき防止の既存契約を
/// 弱めない）。
#[wasm_bindgen_test]
fn ctrl_k_repeat_still_prevents_default_but_does_not_redispatch_toggle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let command = Command::default();
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, _item_elements) =
        build_command_dom(&document, "cmd-ctrl-k-repeat", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (component, log) = wire(root, command);

    let window = web_sys::window().unwrap();
    let default_not_prevented = window
        .dispatch_event(&keydown_event_with("k", true, false, false))
        .unwrap();
    assert!(!default_not_prevented, "初回 Ctrl+K は claim される");
    assert!(component.borrow().is_open(), "初回 Ctrl+K で dialog が開く");
    assert!(
        log.borrow()
            .iter()
            .any(|(action, _)| action == fandhe_frontend_wasm_full::command::ACTION_TOGGLE),
        "初回 Ctrl+K は toggle を dispatch する"
    );
    // 初回 dispatch 後（`reflect_filter` による自動選択の再 dispatch を
    // 含み得る）の件数をベースラインとして記録し、キーリピート中は
    // これ以上増えないことのみを確認する（toggle 自体が正確に 1 回か
    // どうかは本テストの検証観点ではない）。
    let dispatch_count_after_first_press = log.borrow().len();

    let repeat_default_not_prevented = window
        .dispatch_event(&keydown_event_with_repeat("k", true, false, false))
        .unwrap();
    assert!(
        !repeat_default_not_prevented,
        "キーリピート中の Ctrl+K も prevent_default() され、ブラウザ既定のショートカットへ漏れない"
    );
    assert_eq!(
        log.borrow().len(),
        dispatch_count_after_first_press,
        "キーリピート中は toggle を再 dispatch しない（チラつき防止の既存契約）"
    );
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

/// codex-review P1 是正の回帰テスト（イシュー #2069）: `data-action-input`
/// を持つ input の `"input"` dispatch は本モジュール（`wire_command_events`）
/// より先に `crate::events::wire_events` が担う。IME 変換中
/// （`isComposing: true`）は `wire_events` 側の dispatch 自体を延期しないと、
/// `wire_command_events` 側で `is_composing()` を確認する前に状態更新・
/// 再描画（構造フォールバック含む）が起きて変換対象の input 要素ごと
/// 差し替わり、変換が中断され得る。修正前は `wire_events` の `"input"`
/// リスナーが IME 判定を持たず、`seen` に dispatch が記録され絞り込みも
/// 反映されてしまっていた。
#[wasm_bindgen_test]
fn composing_input_with_data_action_input_is_not_dispatched_by_wire_events() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false), ("search", "Search", false)];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-action-input-ime", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    input
        .set_attribute("data-action-input", "cmd_query")
        .unwrap();

    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::events::wire_events(root.clone(), move |action_ref| {
        recorder.borrow_mut().push(action_ref.action.clone());
    })
    .expect("wire_events must not fail");
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |_action_ref| {})
        .expect("wire_command_events must not fail");

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("cal");
    input.dispatch_event(&composing_input_event()).unwrap();

    assert!(
        seen.borrow().is_empty(),
        "IME 変換中は data-action-input の dispatch を wire_events 側で延期する"
    );
    assert!(
        !item_elements[1].has_attribute("hidden"),
        "IME 変換中は絞り込み再描画も走らない（構造フォールバックによる input detach 回避）"
    );
}

/// codex-review P1 是正の回帰テスト（イシュー #2069）: 上記
/// `composing_input_with_data_action_input_is_not_dispatched_by_wire_events`
/// が示すとおり `crate::events::wire_events` は変換中の "input" dispatch を
/// 延期するが、確定後にブラウザが必ず追加の "input" を発火するとは限らない
/// （Chrome の一部確定操作で確定後の "input" が発火されない既知挙動）。
/// 追加の "input" を一切送らず `"compositionend"` のみを送る構成で、
/// `data-action-input` の確定値 dispatch と絞り込み反映の双方が行われる
/// ことを検証する（`crate::events::wiring::dispatch_input_action` の回帰）。
#[wasm_bindgen_test]
fn compositionend_dispatches_data_action_input_without_a_trailing_input_event() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false), ("search", "Search", false)];
    let (root, _dialog, input, _list, item_elements) = build_command_dom(
        &document,
        "cmd-action-input-compositionend",
        &command,
        &items,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    input
        .set_attribute("data-action-input", "cmd_query")
        .unwrap();

    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::events::wire_events(root.clone(), move |action_ref| {
        recorder.borrow_mut().push(action_ref.action.clone());
    })
    .expect("wire_events must not fail");
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |_action_ref| {})
        .expect("wire_command_events must not fail");

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("cal");
    input.dispatch_event(&composing_input_event()).unwrap();
    // 追加の "input" を一切送らず、"compositionend" のみで確定させる。
    input.dispatch_event(&compositionend_event()).unwrap();

    assert!(
        !seen.borrow().is_empty(),
        "compositionend 確定後は追加の \"input\" が無くても data-action-input の値が dispatch される"
    );
    assert!(
        item_elements[1].has_attribute("hidden"),
        "compositionend 確定後は絞り込みも反映される"
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

// --- (k) group 祖先の data-disabled（codex-review P1、イシュー #2069） ---

/// codex-review P1 是正の回帰テスト: `group`（`role="group"`）に
/// `data-disabled` を付けた構成で、矢印キーの自動選択候補判定
/// （`selection_sync_plan`）が item 自身の属性だけを見る
/// `keynav::wiring::disabled_flags` ではなく、祖先込みで判定する
/// `item_disabled_flags`（内部で `has_disabled_ancestor` を使う）を使う
/// ことを検証する。修正前は group 配下の item 自身に `data-disabled` が
/// 無いため誤って選択候補になっていた（Enter 実行・クリックは
/// `has_disabled_ancestor` で祖先無効化を確認するため、矢印キー選択だけ
/// 契約が不一致だった）。
#[wasm_bindgen_test]
fn arrow_down_skips_item_disabled_via_group_ancestor() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let container_id = "cmd-group-disabled";
    let list_id = format!("{container_id}-list");
    let container = create_container(&document, container_id);
    let entries: [(&str, &str); 2] = [("b", "Beta"), ("a", "Alpha")];
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
                    vec![
                        command::group(
                            None,
                            vec![("data-disabled", "")],
                            vec![command.item(
                                "b",
                                false,
                                Some(&format!("{container_id}-item-b")),
                                Vec::new(),
                                vec![text("Beta")],
                            )],
                        ),
                        command.item(
                            "a",
                            false,
                            Some(&format!("{container_id}-item-a")),
                            Vec::new(),
                            vec![text("Alpha")],
                        ),
                    ],
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
    let item_b = root
        .query_selector(&format!("#{container_id}-item-b"))
        .unwrap()
        .unwrap();
    let item_a = root
        .query_selector(&format!("#{container_id}-item-a"))
        .unwrap()
        .unwrap();
    let (_component, _log) = wire(root, command);

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();

    assert!(
        !item_b.has_attribute("data-selected"),
        "group の data-disabled 配下の item は選択候補から除外される"
    );
    assert!(
        item_a.has_attribute("data-selected"),
        "group 配下でない次の enabled item が選択される"
    );
    assert_eq!(
        input.get_attribute("aria-activedescendant").as_deref(),
        Some(format!("{container_id}-item-a").as_str())
    );
}

// --- (l) 配線 root 自体の data-disabled と detach（codex-review P1、
//     イシュー #2069） ---

/// codex-review P1 是正の回帰テスト: `data-action-input` を持つ input の
/// 先行 dispatch（`crate::events::wire_events`）が構造フォールバックで
/// 配線 root の子だけを差し替えた後、古い（detach 済みの） input 要素上で
/// 発火した `"input"` イベントに対し、`handle_input` 冒頭の disabled 判定
/// （`has_disabled_ancestor(root, target_element)`）は detach 済みの
/// `target_element` から `root` へ祖先を辿れず `root` 自身の
/// `data-disabled` を見逃してしまう（修正前の実装）。`root` 自身の属性を
/// 直接確認する経路を追加したことで、detach 後もこの状況が
/// no-op になることを検証する。
#[wasm_bindgen_test]
fn data_action_input_dispatch_is_noop_when_wiring_root_disabled_survives_child_detach() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false), ("search", "Search", false)];
    let (root, _dialog, stale_input, _list, _item_elements) =
        build_command_dom(&document, "cmd-root-disabled-detach", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    // 配線 root 自体を無効化する（root は以後も document に残り続ける、
    // wire_events 相当の構造フォールバックが子だけを差し替える想定）。
    root.set_attribute("data-disabled", "").unwrap();
    stale_input
        .set_attribute("data-action-input", "cmd_query")
        .unwrap();

    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        recorder.borrow_mut().push(action_ref.action.clone());
    })
    .expect("wire_command_events must not fail");

    // wire_events による構造フォールバック（root の子だけを新しい DOM で
    // 差し替え、root 自身は維持する）を模倣する: root の innerHTML を
    // 同一構成で再構築し、以後 `stale_input`（旧参照）は root から
    // detach された状態になる。
    let list_id = "cmd-root-disabled-detach-list".to_string();
    let entries: [(&str, &str); 2] = [("calendar", "Calendar"), ("search", "Search")];
    let is_empty = command.is_empty(&entries);
    let refreshed = command.root(
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
                    vec![
                        command.item(
                            "calendar",
                            false,
                            Some("cmd-root-disabled-detach-item-calendar"),
                            Vec::new(),
                            vec![text("Calendar")],
                        ),
                        command.item(
                            "search",
                            false,
                            Some("cmd-root-disabled-detach-item-search"),
                            Vec::new(),
                            vec![text("Search")],
                        ),
                    ],
                ),
            ],
        )],
    );
    // `refreshed` は `command.root` が生成する外殻ごとの Node であり、
    // その子（dialog 以下）だけを root の innerHTML として差し替える
    // （root 自身の attributes、ここでは `data-disabled`、は維持される）。
    let refreshed_root_html = render(&refreshed);
    let refreshed_container = document.create_element("div").unwrap();
    refreshed_container.set_inner_html(&refreshed_root_html);
    let refreshed_root = refreshed_container.first_element_child().unwrap();
    root.set_inner_html(&refreshed_root.inner_html());
    let fresh_item_calendar = root
        .query_selector("#cmd-root-disabled-detach-item-calendar")
        .unwrap()
        .unwrap();

    assert!(
        !root.contains(Some(&stale_input)),
        "テスト前提: 旧 input 参照は root から detach 済み"
    );

    let html_stale_input = stale_input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_stale_input.set_value("cal");
    stale_input.dispatch_event(&input_event()).unwrap();

    assert!(
        seen.borrow().is_empty(),
        "root 自体が data-disabled のとき、detach 済み input からの \"input\" は no-op になる"
    );
    assert!(
        !fresh_item_calendar.has_attribute("hidden"),
        "root 自体が data-disabled のとき、新しい DOM への絞り込み反映も行われない"
    );
}

// --- (m) IME 確定（compositionend）時の補完 dispatch（codex-review P1、
// イシュー #2069） ---

/// `handle_input` は変換中（`isComposing`）の "input" を延期し、確定後に
/// 追加で発火する "input" で反映される前提を置いていたが、この前提は
/// 実ブラウザで常に成立するとは限らない（Chrome の一部確定操作で確定後の
/// "input" が発火されない既知挙動）。本テストは「延期後に追加の "input"
/// を一切送らず、`"compositionend"` のみを送る」構成で確定値が反映される
/// ことを検証する（[`fandhe_frontend_wasm_full::command::wiring::
/// handle_compositionend`] の回帰）。
#[wasm_bindgen_test]
fn compositionend_reflects_deferred_ime_filter_without_a_trailing_input_event() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [
        ("calendar", "Calendar", false),
        ("search", "Search Emoji", false),
    ];
    let (root, _dialog, input, _list, item_elements) =
        build_command_dom(&document, "cmd-compositionend", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();

    // 変換中: ブラウザは isComposing=true の "input" を発火するが、
    // このイベントは反映されず延期される。
    html_input.set_value("cal");
    input.dispatch_event(&composing_input_event()).unwrap();
    assert!(
        !item_elements[1].has_attribute("hidden"),
        "変換中は反映されず search は可視のまま"
    );

    // 変換確定: 追加の "input" を一切送らず "compositionend" のみで
    // 確定値が反映されることを確認する。
    input.dispatch_event(&compositionend_event()).unwrap();
    assert!(
        !item_elements[0].has_attribute("hidden"),
        "calendar は可視のまま"
    );
    assert!(
        item_elements[1].has_attribute("hidden"),
        "compositionend 確定後は追加の \"input\" が無くても search が非表示になる"
    );
}

/// `compositionend` 確定後、ブラウザによっては同一値の追加 "input"
/// （`isComposing: false`）がもう一度発火することがある（Safari/Firefox
/// 等）。この追加 "input" で同じ絞り込み反映を二重 dispatch しない
/// （`ACTION_INPUT` を 2 回 dispatch しない）ことを検証する。
#[wasm_bindgen_test]
fn trailing_input_event_after_compositionend_with_same_value_does_not_double_dispatch() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("calendar", "Calendar", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-compositionend-dedupe", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let inputs_seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = inputs_seen.clone();
    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        if action_ref.action == fandhe_frontend_wasm_full::command::ACTION_INPUT {
            recorder.borrow_mut().push(action_ref.payload.clone());
        }
    })
    .expect("wire_command_events must not fail");

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.set_value("cal");
    input.dispatch_event(&composing_input_event()).unwrap();
    input.dispatch_event(&compositionend_event()).unwrap();
    // ブラウザが確定直後に追加で発火し得る、同一値・非 composing の
    // "input"。
    input.dispatch_event(&input_event()).unwrap();

    assert_eq!(
        inputs_seen.borrow().as_slice(),
        ["cal".to_string()],
        "compositionend で確定済みの値と同一の追加 input は二重 dispatch されない"
    );
}

// --- (n) command:execute の実行フックが移したフォーカスの尊重
// （codex-review P1、イシュー #2069） ---

/// `command:execute`（[`fandhe_frontend_wasm_full::command::ACTION_EXECUTE`]）
/// のアプリ側実行フックが、実行先の別要素（エディタ相当、Command の外側に
/// あり接続済み）へ明示的に `focus()` している場合、直後の `reflect_filter`
/// がそのフォーカスを `input` へ無条件に奪い返してはならないことを検証する
/// （`crates/wasm-full/src/command.rs::wiring::focus_available_for_restore`
/// の回帰）。
#[wasm_bindgen_test]
fn command_execute_focus_move_to_connected_element_is_respected() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-execute-focus", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());

    let editor = document
        .create_element("input")
        .expect("create_element must not fail");
    editor.set_id("cmd-execute-focus-editor");
    document
        .body()
        .expect("document body must exist")
        .append_child(&editor)
        .expect("append_child must not fail");
    let _cleanup_editor = RemoveOnDrop(editor.clone());
    let execute_editor = html_element(&editor);

    fandhe_frontend_wasm_full::command::wire_command_events(root.clone(), move |action_ref| {
        if action_ref.action == fandhe_frontend_wasm_full::command::ACTION_EXECUTE {
            let _ = execute_editor.focus();
        }
    })
    .expect("wire_command_events must not fail");

    let html_input = input.clone().dyn_into::<HtmlInputElement>().unwrap();
    html_input.focus().expect("focus must not fail");
    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    input.dispatch_event(&keydown_event("Enter")).unwrap();

    assert_eq!(
        document.active_element().as_ref(),
        Some(&editor),
        "実行フックが移した先のフォーカスを reflect_filter が奪い返してはならない"
    );
}

// --- (o) item 以外の mousedown も input のフォーカスを維持する
// （Cursor Bugbot Medium、イシュー #2069） ---

/// item 以外（`list`/`dialog`/`empty` 等）の mousedown でブラウザ既定動作
/// （フォーカス移動）が起きると、`input` から blur し、以降
/// `handle_keydown`/`handle_input`（いずれも `event.target()` が
/// `INPUT_PART` であることを要求する）が矢印キー・Enter・入力・Escape を
/// no-op として無視してしまう（`input` を再クリックするまで操作不能）。
/// `handle_mousedown` が item に限らず Command インスタンス配下（`input`
/// 自身を除く）で `prevent_default()` することを検証する。
#[wasm_bindgen_test]
fn mousedown_on_list_background_prevents_default_to_keep_input_focused() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, list, _item_elements) =
        build_command_dom(&document, "cmd-mousedown-list", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let event = mousedown_event();
    list.dispatch_event(&event).unwrap();
    assert!(
        event.default_prevented(),
        "item 以外（list 背景）の mousedown も input のフォーカス維持のため prevent_default される"
    );

    let _ = input;
}

/// `input` パーツ自身の mousedown はブラウザ既定動作（フォーカス・
/// キャレット位置決定・テキスト選択）を妨げないことを確認する
/// （`handle_mousedown` の非退行）。
#[wasm_bindgen_test]
fn mousedown_on_input_itself_is_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-mousedown-input", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let event = mousedown_event();
    input.dispatch_event(&event).unwrap();
    assert!(
        !event.default_prevented(),
        "input パーツ自身の mousedown は既定動作を妨げない"
    );
}

/// `event.target()` がテキストノード（`fandhe_frontend_core::text` で
/// 描画される空メッセージ等）であっても `prevent_default()` が呼ばれる
/// ことを検証する（Cursor Bugbot Medium 是正、イシュー #2069）。従来は
/// `target` の `Element` へのキャストのみを試み、失敗時に即 return して
/// いたため、テキストノード直接クリックで `input` から blur し、以降の
/// 矢印キー・入力・Escape が `input` を再クリックするまで無反応になって
/// いた。
#[wasm_bindgen_test]
fn mousedown_on_text_node_target_prevents_default() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    // items を空にして `empty`（空メッセージ）パーツを可視化させ、その
    // 直下のテキストノードを mousedown の target にする。
    let items: [(&str, &str, bool); 0] = [];
    let (root, _dialog, input, _list, _item_elements) =
        build_command_dom(&document, "cmd-mousedown-text-node", &command, &items);
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let empty_el = root
        .query_selector(r#"[data-scope="command"][data-part="empty"]"#)
        .expect("query_selector must not fail")
        .expect("empty element must exist");
    let text_node = empty_el
        .first_child()
        .expect("empty element must contain a text node child");
    assert_eq!(
        text_node.node_type(),
        web_sys::Node::TEXT_NODE,
        "empty メッセージはテキストノードとして描画される前提"
    );

    let event = mousedown_event();
    text_node.dispatch_event(&event).unwrap();
    assert!(
        event.default_prevented(),
        "テキストノードへの mousedown も input のフォーカス維持のため prevent_default される"
    );

    let _ = input;
}

/// `command::dialog` は任意の children を受け取れるため、検索対象切替用の
/// `select` のような、Command のパーツではない独立したインタラクティブ
/// 要素が併設され得る。この `select` への mousedown は `prevent_default()`
/// されず、ブラウザ既定のフォーカス移動・選択操作が妨げられないことを
/// 検証する（codex-review P1 是正、イシュー #2069）。
#[wasm_bindgen_test]
fn mousedown_on_independent_select_inside_dialog_is_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, dialog, _input, _list, _item_elements) = build_command_dom(
        &document,
        "cmd-mousedown-independent-select",
        &command,
        &items,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    // Command のパーツではない、利用者が併設した検索対象切替用 select を
    // dialog 直下へ追加する。
    let select = document
        .create_element("select")
        .expect("create_element must not fail");
    dialog
        .append_child(&select)
        .expect("append_child must not fail");

    let event = mousedown_event();
    select.dispatch_event(&event).unwrap();
    assert!(
        !event.default_prevented(),
        "Command のパーツではない独立したインタラクティブ要素（select）の \
         mousedown は既定動作を妨げない"
    );
}

/// 上記 `select` の子孫（`option`）への mousedown も同様に
/// `prevent_default()` されないことを検証する（`Element::closest` の
/// self-or-ancestor 一致により子孫まで除外される契約の回帰）。
#[wasm_bindgen_test]
fn mousedown_on_descendant_of_independent_interactive_element_is_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, dialog, _input, _list, _item_elements) = build_command_dom(
        &document,
        "cmd-mousedown-independent-select-descendant",
        &command,
        &items,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let (_component, _log) = wire(root.clone(), command);

    let select = document
        .create_element("select")
        .expect("create_element must not fail");
    let option = document
        .create_element("option")
        .expect("create_element must not fail");
    select
        .append_child(&option)
        .expect("append_child must not fail");
    dialog
        .append_child(&select)
        .expect("append_child must not fail");

    let event = mousedown_event();
    option.dispatch_event(&event).unwrap();
    assert!(
        !event.default_prevented(),
        "独立したインタラクティブ要素の子孫（option）への mousedown も \
         既定動作を妨げない"
    );
}

/// Command を Tabs content や ScrollArea viewport 等、`tabindex="0"` を
/// 固定で持つ外側パネルの内側へ合成配置した場合でも、通常の item への
/// mousedown が `input` のフォーカス維持のため引き続き `prevent_default()`
/// されることを検証する（codex-review P1 是正、イシュー #2069）。
/// `INDEPENDENT_INTERACTIVE_SELECTOR` の除外判定は `Element::closest` が
/// 祖先方向へ無制限に探索する性質上、`root` の外側にある外側パネルの
/// `tabindex="0"` まで誤って一致させてしまい、item への mousedown まで
/// 無効化する不具合があった。一致した要素が `root` 配下に実在する場合の
/// み除外対象とする修正の回帰テスト（click 経路の
/// `clicking_item_inside_ancestor_tabindex_panel_still_dispatches` と対）。
#[wasm_bindgen_test]
fn mousedown_on_item_inside_ancestor_tabindex_panel_prevents_default() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mut command = Command::default();
    command.update(CommandAction::Open);
    let items = [("a", "Alpha", false)];
    let (root, _dialog, _input, _list, item_elements) = build_command_dom(
        &document,
        "cmd-mousedown-ancestor-tabindex-panel",
        &command,
        &items,
    );

    // Command の外側（例: Tabs content パネルや ScrollArea viewport）を
    // 模した `tabindex="0"` 固定の祖先パネルへ container ごと包む。
    let container = root
        .parent_element()
        .expect("build_command_dom container must exist as root's parent");
    let panel = document
        .create_element("div")
        .expect("create_element must not fail");
    panel
        .set_attribute("tabindex", "0")
        .expect("set_attribute must not fail");
    document
        .body()
        .expect("document body must exist")
        .append_child(&panel)
        .expect("append_child must not fail");
    panel
        .append_child(&container)
        .expect("append_child must not fail");
    let _cleanup = RemoveOnDrop(panel);

    let (_component, _log) = wire(root, command);

    let event = mousedown_event();
    item_elements[0].dispatch_event(&event).unwrap();
    assert!(
        event.default_prevented(),
        "外側パネルの tabindex=\"0\" に誤って一致し、item への mousedown \
         による input のフォーカス維持が妨げられてはならない"
    );
}
