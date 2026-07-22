//! `fandhe_frontend_wasm_full::headless`（イシュー #580）の実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/tests/headless_wiring.rs`（native）は
//! [`fandhe_frontend_wasm_full::headless::action_for_part`]/`action_from_parts`
//! （純粋ロジック層）とマッピング表のドリフト検知を担う。本ファイルは
//! その先、**実ブラウザ（headless Chromium、`wasm-pack test --headless
//! --chrome`）上での合成 click イベント → [`wire_headless_events`]/
//! [`wire_headless_component`]（配線層）→ `fandhe_frontend_interactive::dispatch`**
//! という製品経路を検証する（受け入れ条件 1〜3、`xss_escape_wasm.rs` と同型の
//! 実 DOM 検証パターンを踏襲する）。
//!
//! headless コンポーネントの自動再描画（束縛点更新との統合）は本イシューの
//! スコープ外（`headless.rs` モジュール doc §out-of-scope 参照）のため、
//! `on_update` コールバックは状態変化の観測にのみ使う（DOM への `data-state`
//! 反映は行わない）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_headless_ui::collapsible::Collapsible;
use fandhe_frontend_headless_ui::select::Select;
use fandhe_frontend_headless_ui::state::SingleSelect;
use fandhe_frontend_headless_ui::{collapsible, dialog, radio_group, select, Dialog, RadioGroup};
use fandhe_frontend_wasm_full::headless::wire_headless_component;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`xss_escape_wasm.rs::create_container` と同じ意図: 一意な id でテスト間の
/// 要素衝突を避ける）。
fn create_container(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`xss_escape_wasm.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click` イベントを生成する（`bubbles: true`）。
///
/// [`wire_headless_events`]/[`wire_headless_component`] はリスナーを root
/// 要素へ登録するため、子要素上で発火したイベントがバブリングで root まで
/// 届く必要がある。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

fn dispatch_click(target: &Element) {
    target
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
}

// --- 受け入れ条件 1: Disclosure 系（Collapsible/Dialog）の trigger クリックで open/close/toggle ---

#[wasm_bindgen_test]
fn collapsible_trigger_click_toggles_open_state_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-collapsible-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&collapsible::root(
        fandhe_frontend_headless_ui::state::OpenState::Closed,
        false,
        vec![],
        vec![collapsible::trigger(
            fandhe_frontend_headless_ui::state::OpenState::Closed,
            false,
            None,
            vec![],
            vec![fandhe_frontend_core::text("Toggle")],
        )],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("collapsible root must exist");
    let trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");

    let component = Rc::new(RefCell::new(Collapsible::default()));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger);
    assert!(
        component.borrow().is_open(),
        "trigger クリック後は Collapsible が開いていること"
    );

    dispatch_click(&trigger);
    assert!(
        !component.borrow().is_open(),
        "trigger 再クリックで toggle により閉じること"
    );
}

#[wasm_bindgen_test]
fn dialog_trigger_and_close_trigger_click_open_and_close_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-dialog-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let open_state = fandhe_frontend_headless_ui::state::OpenState::Closed;
    let html = fandhe_frontend_core::render(&dialog::root(
        open_state,
        vec![],
        vec![
            dialog::trigger(
                open_state,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Open")],
            ),
            dialog::close_trigger(vec![], vec![fandhe_frontend_core::text("Close")]),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("dialog root must exist");
    let trigger_el = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");
    let close_el = root
        .query_selector(r#"[data-part="close-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("close-trigger element must exist");

    let component = Rc::new(RefCell::new(Dialog::default()));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger_el);
    assert!(component.borrow().is_open());

    dispatch_click(&close_el);
    assert!(!component.borrow().is_open());
}

// --- 受け入れ条件 2: Tabs/RadioGroup/Select の select dispatch ---

#[wasm_bindgen_test]
fn radio_group_item_click_selects_value_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-radio-group-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&radio_group::root(
        false,
        None,
        None,
        vec![],
        vec![
            radio_group::item(
                false,
                false,
                "red",
                vec![],
                vec![radio_group::item_text(
                    false,
                    false,
                    vec![],
                    vec![fandhe_frontend_core::text("Red")],
                )],
            ),
            radio_group::item(
                false,
                false,
                "blue",
                vec![],
                vec![fandhe_frontend_core::text("Blue")],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("radio-group root must exist");

    let component = Rc::new(RefCell::new(RadioGroup::default()));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    // item-text（内側、表にない part）をクリックしても祖先の item（表内）で
    // 解決できること（受け入れ条件のカバレッジ、`item_text` は red 項目内側）。
    let item_text = root
        .query_selector(r#"[data-part="item-text"]"#)
        .expect("query_selector must not fail")
        .expect("item-text element must exist");
    dispatch_click(&item_text);
    assert!(component.borrow().is_checked("red"));

    let blue_item = root
        .query_selector(r#"[data-value="blue"]"#)
        .expect("query_selector must not fail")
        .expect("blue item element must exist");
    dispatch_click(&blue_item);
    assert!(component.borrow().is_checked("blue"));
    assert!(!component.borrow().is_checked("red"));
}

#[wasm_bindgen_test]
fn select_full_cycle_open_select_and_clear_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let open_state = fandhe_frontend_headless_ui::state::OpenState::Closed;
    let html = fandhe_frontend_core::render(&select::root(
        open_state,
        vec![],
        vec![
            select::trigger(
                open_state,
                false,
                None,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Open")],
            ),
            select::clear_trigger(vec![], vec![fandhe_frontend_core::text("Clear")]),
            select::content(
                open_state,
                None,
                None,
                vec![],
                vec![
                    select::item(open_state, false, "opt-1", vec![], vec![]),
                    select::item(open_state, false, "opt-2", vec![], vec![]),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("select root must exist");

    let component = Rc::new(RefCell::new(Select::default()));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    let trigger_el = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");
    dispatch_click(&trigger_el);
    assert!(component.borrow().is_open());

    let opt1 = root
        .query_selector(r#"[data-value="opt-1"]"#)
        .expect("query_selector must not fail")
        .expect("opt-1 item element must exist");
    dispatch_click(&opt1);
    assert_eq!(component.borrow().selected(), Some("opt-1"));
    // ark-ui の closeOnSelect 既定 true に準拠し、選択と同時に listbox が閉じる。
    assert!(!component.borrow().is_open());

    let clear_el = root
        .query_selector(r#"[data-part="clear-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("clear-trigger element must exist");
    dispatch_click(&clear_el);
    assert_eq!(component.borrow().selected(), None);
}

// --- 受け入れ条件 3: fail-closed（未知アクション・改ざん data-* 入力で panic せず no-op） ---

#[wasm_bindgen_test]
fn click_outside_mapping_table_and_disabled_part_are_noop_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-fail-closed-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&collapsible::root(
        fandhe_frontend_headless_ui::state::OpenState::Closed,
        false,
        vec![],
        vec![
            collapsible::trigger(
                fandhe_frontend_headless_ui::state::OpenState::Closed,
                true, // disabled
                None,
                vec![],
                vec![fandhe_frontend_core::text("Disabled trigger")],
            ),
            // マッピング表にない (scope, part) を模す装飾要素。
            fandhe_frontend_core::el(
                "div",
                vec![("data-scope", "unknown-widget"), ("data-part", "trigger")],
                vec![fandhe_frontend_core::text("Unmapped")],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("collapsible root must exist");

    let component = Rc::new(RefCell::new(Collapsible::default()));
    let update_calls = Rc::new(RefCell::new(0u32));
    let update_calls_in_closure = update_calls.clone();
    wire_headless_component(root.clone(), component.clone(), move |_state, _root| {
        *update_calls_in_closure.borrow_mut() += 1;
    })
    .expect("wire_headless_component must not fail");

    let disabled_trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("disabled trigger element must exist");
    dispatch_click(&disabled_trigger);
    assert!(
        !component.borrow().is_open(),
        "data-disabled が付与された trigger のクリックは no-op であること"
    );

    let unmapped = root
        .query_selector(r#"[data-scope="unknown-widget"]"#)
        .expect("query_selector must not fail")
        .expect("unmapped element must exist");
    dispatch_click(&unmapped);
    assert!(
        !component.borrow().is_open(),
        "マッピング表にない (scope, part) のクリックは no-op であること"
    );

    assert_eq!(
        *update_calls.borrow(),
        0,
        "no-op のクリックでは on_update が一切呼ばれないこと（dispatch 不成立）"
    );
}

#[wasm_bindgen_test]
fn select_item_without_data_value_is_noop_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-missing-value-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // select item の data-value を除去した改ざん入力を模す
    // （`select::item` の出力から data-value を持たない同型要素を手組みする）。
    let tampered_item = fandhe_frontend_core::el(
        "div",
        vec![("data-scope", "select"), ("data-part", "item")],
        vec![fandhe_frontend_core::text("Tampered")],
    );
    let html = fandhe_frontend_core::render(&tampered_item);
    container.set_inner_html(&html);
    let item_el = container
        .first_element_child()
        .expect("tampered item element must exist");

    let component = Rc::new(RefCell::new(Select::default()));
    wire_headless_component(item_el.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&item_el);
    assert_eq!(
        component.borrow().selected(),
        None,
        "data-value を欠いた select item のクリックは選択を成立させないこと"
    );
}

// --- ネストした headless root 間のクロスディスパッチ防止回帰
// （イシュー #580 PR #611 Bugbot 指摘: Dialog 内に Select をネストして双方を
// wire_headless_component すると、内側 Select の trigger/item クリックが
// bubble して外側 Dialog の共有語彙（"toggle"）へ誤って二重ディスパッチ
// されていた） ---

#[wasm_bindgen_test]
fn nested_select_inside_dialog_click_does_not_cross_dispatch_to_outer_dialog() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-nested-dialog-select-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let dialog_open = fandhe_frontend_headless_ui::state::OpenState::Closed;
    let select_open = fandhe_frontend_headless_ui::state::OpenState::Closed;
    let html = fandhe_frontend_core::render(&dialog::root(
        dialog_open,
        vec![],
        vec![
            dialog::trigger(
                dialog_open,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Open dialog")],
            ),
            select::root(
                select_open,
                vec![],
                vec![
                    select::trigger(
                        select_open,
                        false,
                        None,
                        None,
                        vec![],
                        vec![fandhe_frontend_core::text("Open select")],
                    ),
                    select::content(
                        select_open,
                        None,
                        None,
                        vec![],
                        vec![select::item(select_open, false, "opt-1", vec![], vec![])],
                    ),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let dialog_root = container
        .first_element_child()
        .expect("dialog root must exist");
    let select_root = dialog_root
        .query_selector(r#"[data-scope="select"]"#)
        .expect("query_selector must not fail")
        .expect("nested select root must exist");
    let select_trigger = select_root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("select trigger element must exist");

    let dialog_component = Rc::new(RefCell::new(Dialog::default()));
    let select_component = Rc::new(RefCell::new(Select::default()));
    // 外側（Dialog）を先に配線し、内側（Select）を後から配線する
    // （実装上の配線順に依存しないことも合わせて確認する）。
    wire_headless_component(
        dialog_root.clone(),
        dialog_component.clone(),
        |_state, _root| {},
    )
    .expect("outer wire_headless_component must not fail");
    wire_headless_component(
        select_root.clone(),
        select_component.clone(),
        |_state, _root| {},
    )
    .expect("inner wire_headless_component must not fail");

    dispatch_click(&select_trigger);

    assert!(
        select_component.borrow().is_open(),
        "内側 Select の trigger クリックで Select 自身は開くこと"
    );
    assert!(
        !dialog_component.borrow().is_open(),
        "内側 Select の trigger クリックが bubble して外側 Dialog の \
         共有語彙（toggle）へ誤ってクロスディスパッチされてはならない \
         （イシュー #580 PR #611 Bugbot 指摘の回帰）"
    );
}

// --- REQ-1 回帰: マッピング結果の payload は実 DOM 再描画時も既定エスケープを経由する ---

#[wasm_bindgen_test]
fn radio_group_item_data_value_xss_payload_click_does_not_produce_script_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-radio-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>alert(1)</script>";
    let html =
        fandhe_frontend_core::render(&radio_group::item(false, false, payload, vec![], vec![]));
    container.set_inner_html(&html);
    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "data-value に XSS ペイロードを含む item の展開時点で script 要素が生成されてはならない"
    );

    let item_el = container
        .first_element_child()
        .expect("radio-group item element must exist");

    let component = Rc::new(RefCell::new(SingleSelect::default()));
    wire_headless_component(item_el.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&item_el);
    assert_eq!(component.borrow().selected(), Some(payload));

    // 選択値を再描画（render_for_hydration 経由）してもエスケープが効くこと
    // （REQ-1、`radio_group::item` の value 契約と同じ経路）。
    let rendered = fandhe_frontend_core::render(
        &fandhe_frontend_interactive::render_for_hydration(&*component.borrow()),
    );
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(rendered.contains("&lt;script&gt;"));
}
