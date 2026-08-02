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
use fandhe_frontend_headless_ui::{
    collapsible, dialog, radio_group, select, Dialog, Menu, RadioGroup,
};
use fandhe_frontend_wasm_full::headless::wire_headless_component;
use fandhe_frontend_wasm_full::headless_select::wire_select_value_text;
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
                None,
                vec![],
                vec![
                    select::item(open_state, false, false, "opt-1", None, vec![], vec![]),
                    select::item(open_state, false, false, "opt-2", None, vec![], vec![]),
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
                        None,
                        vec![],
                        vec![select::item(
                            select_open,
                            false,
                            false,
                            "opt-1",
                            None,
                            vec![],
                            vec![],
                        )],
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

// --- イシュー #642: Select value-text のクライアント側同期 ---
//
// `select::value_text` の SSR 出力に付与された `data-bind-text` マーカー
// （`fandhe_frontend_headless_ui::select::VALUE_TEXT_FIELD`）を頼りに、
// `wire_select_value_text` が select/deselect dispatch 後にラベルを再同期
// することを実ブラウザ上で固定する。

const SELECT_PLACEHOLDER: &str = "Select a framework";

/// value-text 同期テスト共通の Select マークアップを組み立てる
/// （trigger + value_text(placeholder) + clear-trigger、content 配下に
/// item-text 付き item を 2 個）。
fn build_select_with_value_text_html(items: &[(&str, &str)]) -> String {
    let open_state = fandhe_frontend_headless_ui::state::OpenState::Closed;
    let item_nodes = items
        .iter()
        .map(|(value, label)| {
            select::item(
                open_state,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    None,
                    vec![],
                    vec![fandhe_frontend_core::text(*label)],
                )],
            )
        })
        .collect::<Vec<_>>();

    fandhe_frontend_core::render(&select::root(
        open_state,
        vec![],
        vec![
            select::control(
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
                    select::value_text(
                        true,
                        vec![],
                        vec![fandhe_frontend_core::text(SELECT_PLACEHOLDER)],
                    ),
                    select::clear_trigger(vec![], vec![fandhe_frontend_core::text("Clear")]),
                ],
            ),
            select::positioner(
                open_state,
                vec![],
                vec![select::content(
                    open_state,
                    None,
                    None,
                    None,
                    vec![],
                    item_nodes,
                )],
            ),
        ],
    ))
}

#[wasm_bindgen_test]
fn select_item_click_updates_value_text_label() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-value-text-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = build_select_with_value_text_html(&[("vue", "Vue"), ("react", "React")]);
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("select root must exist");

    let value_text_el = root
        .query_selector(r#"[data-part="value-text"]"#)
        .expect("query_selector must not fail")
        .expect("value-text element must exist");
    assert!(value_text_el.has_attribute("data-placeholder-shown"));

    let component = Rc::new(RefCell::new(Select::default()));
    wire_select_value_text(
        root.clone(),
        component.clone(),
        SELECT_PLACEHOLDER.to_string(),
    )
    .expect("wire_select_value_text must not fail");

    let trigger_el = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");
    dispatch_click(&trigger_el);

    let item_react = root
        .query_selector(r#"[data-value="react"]"#)
        .expect("query_selector must not fail")
        .expect("react item element must exist");
    dispatch_click(&item_react);

    assert_eq!(component.borrow().selected(), Some("react"));
    assert_eq!(value_text_el.text_content(), Some("React".to_string()));
    assert!(
        !value_text_el.has_attribute("data-placeholder-shown"),
        "選択が確定したら data-placeholder-shown は除去されること"
    );

    // マーカー自体は束縛点走査のため維持されたままであること。
    assert!(value_text_el.has_attribute("data-bind-text"));
}

#[wasm_bindgen_test]
fn select_clear_trigger_restores_placeholder_value_text() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-clear-value-text-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = build_select_with_value_text_html(&[("vue", "Vue")]);
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("select root must exist");

    let component = Rc::new(RefCell::new(Select::default()));
    wire_select_value_text(
        root.clone(),
        component.clone(),
        SELECT_PLACEHOLDER.to_string(),
    )
    .expect("wire_select_value_text must not fail");

    let value_text_el = root
        .query_selector(r#"[data-part="value-text"]"#)
        .expect("query_selector must not fail")
        .expect("value-text element must exist");

    let item_vue = root
        .query_selector(r#"[data-value="vue"]"#)
        .expect("query_selector must not fail")
        .expect("vue item element must exist");
    dispatch_click(&item_vue);
    assert_eq!(value_text_el.text_content(), Some("Vue".to_string()));

    let clear_el = root
        .query_selector(r#"[data-part="clear-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("clear-trigger element must exist");
    dispatch_click(&clear_el);

    assert_eq!(component.borrow().selected(), None);
    assert_eq!(
        value_text_el.text_content(),
        Some(SELECT_PLACEHOLDER.to_string()),
        "clear-trigger による deselect 後は placeholder 文言へ復帰すること"
    );
    assert!(
        value_text_el.has_attribute("data-placeholder-shown"),
        "deselect 後は data-placeholder-shown が再付与されること"
    );
}

#[wasm_bindgen_test]
fn select_stale_selected_value_without_matching_item_is_noop_for_value_text() {
    // 改ざん・欠損入力（選択値に対応する item が root 配下に存在しない）は
    // 同期を行わない no-op とする（fail-closed）。
    //
    // PR #649 review comment 3634998607（Bugbot 指摘）: 本テストは当初
    // 「SSR 初期状態（placeholder 表示済み）に対してステイルな選択値を渡す」
    // 構成だったため、`sync_select_value_text` がプレースホルダー表示を
    // 誤って再構築してしまうバグ（deselect 時の描画と区別できていなかった）
    // があっても、value-text の見た目は変化せず検知できなかった（no-op と
    // 「プレースホルダーへ書き戻す誤動作」が偶然同じ結果になっていたため）。
    // 先に正当な選択（"vue"）を確定させてラベル表示状態を作った上で
    // ステイルな選択値へ遷移させ、「ラベル表示が保持され、
    // `data-placeholder-shown` が付与されないこと」を検証することで、
    // 誤ったプレースホルダー再描画が発生していないことを確実に検知する。
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-stale-value-text-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = build_select_with_value_text_html(&[("vue", "Vue")]);
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("select root must exist");

    let value_text_el = root
        .query_selector(r#"[data-part="value-text"]"#)
        .expect("query_selector must not fail")
        .expect("value-text element must exist");

    let mut select_state = Select::default();

    // まず正当な選択を確定させ、value-text をラベル表示状態にしておく。
    assert!(fandhe_frontend_interactive::dispatch(
        &mut select_state,
        "select",
        "vue"
    ));
    fandhe_frontend_wasm_full::headless_select::sync_select_value_text(
        &select_state,
        &root,
        SELECT_PLACEHOLDER,
    );
    assert_eq!(value_text_el.text_content(), Some("Vue".to_string()));
    assert!(!value_text_el.has_attribute("data-placeholder-shown"));

    // "svelte" は markup 上のどの item にも存在しない選択値（改ざん想定）。
    assert!(fandhe_frontend_interactive::dispatch(
        &mut select_state,
        "select",
        "svelte"
    ));
    fandhe_frontend_wasm_full::headless_select::sync_select_value_text(
        &select_state,
        &root,
        SELECT_PLACEHOLDER,
    );

    assert_eq!(
        value_text_el.text_content(),
        Some("Vue".to_string()),
        "一致する item が無い選択値では value-text を書き換えず、直前のラベル表示を維持すること"
    );
    assert!(
        !value_text_el.has_attribute("data-placeholder-shown"),
        "ステイルな選択値では data-placeholder-shown への誤ったフォールバックが発生しないこと"
    );
}

#[wasm_bindgen_test]
fn select_value_text_xss_label_click_does_not_produce_script_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-select-value-text-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>alert(1)</script>";
    let html = build_select_with_value_text_html(&[("evil", payload)]);
    container.set_inner_html(&html);
    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "item-text に XSS ペイロードを含む item の展開時点で script 要素が生成されてはならない"
    );

    let root = container
        .first_element_child()
        .expect("select root must exist");

    let component = Rc::new(RefCell::new(Select::default()));
    wire_select_value_text(
        root.clone(),
        component.clone(),
        SELECT_PLACEHOLDER.to_string(),
    )
    .expect("wire_select_value_text must not fail");

    let item_el = root
        .query_selector(r#"[data-value="evil"]"#)
        .expect("query_selector must not fail")
        .expect("evil item element must exist");
    dispatch_click(&item_el);

    assert_eq!(component.borrow().selected(), Some("evil"));

    let value_text_el = root
        .query_selector(r#"[data-part="value-text"]"#)
        .expect("query_selector must not fail")
        .expect("value-text element must exist");
    // `set_text_content` はテキストノードとして書き込むため、選択後も script
    // 要素が生成されず、テキストとしてリテラルのまま保持されること。
    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "value-text 更新後も script 要素が生成されてはならない"
    );
    assert_eq!(value_text_el.text_content(), Some(payload.to_string()));
}

// --- サブメニュー（`trigger-item`）の実 headless wiring 回帰
// （イシュー #662 PR #674 Bugbot 指摘: `menu`/`trigger-item` のマッピング行が
// 欠落しており `keynav.rs` の ArrowRight/ArrowLeft が合成する `click()` も
// マウスでの実クリックも no-op になっていた） ---

/// `trigger-item` への実クリックが `menu_trigger_click_toggles_open_closed`
/// と同じ `"toggle"` を経由して**子 `Menu` インスタンス自身**を開閉し、かつ
/// `nested_select_inside_dialog_click_does_not_cross_dispatch_to_outer_dialog`
/// と同型の「内側を先に解決したら `stop_propagation` で外側へ伝播しない」
/// 契約により、bubble して親 `Menu` インスタンスへ誤ってクロス
/// ディスパッチされないことを検証する。`child_wrapper`（trigger-item と
/// その子孫 content をまとめる境界要素）を子 `Menu` インスタンス専用に
/// `wire_headless_component` する構成は、ネストした headless インスタンス
/// を個別配線する既存パターン（Dialog 内 Select の回帰テスト）をサブ
/// メニューへ適用したものであり、アプリ側が実際にサブメニューを組む際に
/// 従うべき配線契約を確認するものでもある。
///
/// `child_content`（さらにその子 `child_item`）は `trigger_item` の
/// **子孫**として配置する（`keynav.rs::wiring::resolve_submenu_content` の
/// フォールバック経路が「`aria-controls` 欠落時は子孫
/// `[data-part="content"]` を辿る」ことを前提にしている実際の DOM 構成、
/// イシュー #662 PR #674 Bugbot 指摘）。修正前はこの入れ子配置において
/// `child_item` クリックが `action_from_parts` の祖先探索で `content` を
/// 素通りし、外側の `trigger_item`（`menu`/`trigger-item` → `"toggle"`）に
/// 誤って解決されていた（`wire_headless_events` が `stop_propagation` する
/// ため、アイテム自身のクリック処理は握り潰される）。
#[wasm_bindgen_test]
fn submenu_trigger_item_click_toggles_child_menu_and_does_not_cross_dispatch_to_parent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-nested-menu-submenu-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let parent_root = document
        .create_element("div")
        .expect("create_element must not fail");
    parent_root.set_attribute("data-scope", "menu").unwrap();
    parent_root.set_attribute("data-part", "root").unwrap();

    let parent_content = document
        .create_element("div")
        .expect("create_element must not fail");
    parent_content.set_attribute("data-scope", "menu").unwrap();
    parent_content
        .set_attribute("data-part", "content")
        .unwrap();

    // 子 `Menu` インスタンス専用の配線境界要素（実 headless-ui 出力の
    // `data-part` 語彙には存在しない、テスト用のラッパー）。アプリ側が
    // サブメニューを組む際は、この境界に相当する要素へ子インスタンスの
    // `wire_headless_component` を個別に呼ぶ必要がある。
    let child_wrapper = document
        .create_element("div")
        .expect("create_element must not fail");

    let trigger_item = document
        .create_element("div")
        .expect("create_element must not fail");
    trigger_item.set_attribute("data-scope", "menu").unwrap();
    trigger_item
        .set_attribute("data-part", "trigger-item")
        .unwrap();
    trigger_item.set_attribute("role", "menuitem").unwrap();

    // `child_content` は `trigger_item` の子（＝子孫）として配置する
    // （実際の DOM 契約に合わせるための構成、上記関数 doc 参照）。
    let child_content = document
        .create_element("div")
        .expect("create_element must not fail");
    child_content.set_attribute("data-scope", "menu").unwrap();
    child_content.set_attribute("data-part", "content").unwrap();

    // `child_content` 配下の子アイテム（`menu`/`item` はマッピング表に
    // 無く常に `None` — クリックしても何のアクションも解決しないことが
    // 期待値。祖先の `trigger_item` の `toggle` を誤って奪ってはならない）。
    let child_item = document
        .create_element("div")
        .expect("create_element must not fail");
    child_item.set_attribute("data-scope", "menu").unwrap();
    child_item.set_attribute("data-part", "item").unwrap();
    child_item
        .set_attribute("data-value", "child-item-1")
        .unwrap();

    child_content.append_child(&child_item).unwrap();
    trigger_item.append_child(&child_content).unwrap();
    child_wrapper.append_child(&trigger_item).unwrap();
    parent_content.append_child(&child_wrapper).unwrap();
    parent_root.append_child(&parent_content).unwrap();
    container.append_child(&parent_root).unwrap();

    let parent_component = Rc::new(RefCell::new(Menu::default()));
    let child_component = Rc::new(RefCell::new(Menu::default()));
    // 外側（親 Menu）を先に配線し、内側（子 Menu インスタンスの境界）を
    // 後から配線する（`nested_select_inside_dialog_...` と同じく配線順に
    // 依存しないことも合わせて確認する）。
    wire_headless_component(parent_root.clone(), parent_component.clone(), |_, _| {})
        .expect("outer wire_headless_component must not fail");
    wire_headless_component(child_wrapper.clone(), child_component.clone(), |_, _| {})
        .expect("inner wire_headless_component must not fail");

    dispatch_click(&trigger_item);

    assert!(
        child_component.borrow().is_open(),
        "trigger-item への実クリックは子 Menu インスタンス自身を開くこと \
         （修正前は menu/trigger-item のマッピング行欠落により no-op だった、\
         イシュー #662 PR #674 Bugbot 指摘の回帰）"
    );
    assert!(
        !parent_component.borrow().is_open(),
        "trigger-item クリックが bubble して親 Menu の toggle へ誤って \
         クロスディスパッチされてはならない"
    );

    dispatch_click(&child_item);

    assert!(
        child_component.borrow().is_open(),
        "content 配下の子アイテムクリックが親（同一子インスタンスの）\
         trigger-item の toggle として誤って解決され、開いたサブメニューが \
         意図せず閉じてはならない（イシュー #662 PR #674 Bugbot 指摘の回帰、\
         action_from_parts の content 境界修正を検証する）"
    );
    assert!(
        !parent_component.borrow().is_open(),
        "content 配下の子アイテムクリックが親 Menu インスタンスへクロス \
         ディスパッチされてはならない"
    );
}

// --- イシュー #1161: NavigationMenu/Calendar/Menubar の trigger クリックが
// 実ブラウザ上で `data-value` payload を伴って dispatch へ到達すること ---

#[wasm_bindgen_test]
fn navigation_menu_trigger_click_toggles_open_state_in_real_dom() {
    use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};
    use fandhe_frontend_headless_ui::state::OpenState;

    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-navigation-menu-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&navigation_menu::root(
        "Main",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![navigation_menu::item(
                OpenState::Closed,
                false,
                vec![],
                vec![navigation_menu::trigger(
                    OpenState::Closed,
                    false,
                    "products",
                    None,
                    None,
                    vec![],
                    vec![fandhe_frontend_core::text("Products")],
                )],
            )],
        )],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("navigation-menu root must exist");
    let trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");

    let component = Rc::new(RefCell::new(NavigationMenu::default()));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger);
    assert!(
        component.borrow().is_open("products"),
        "trigger クリック後は該当項目が開いていること"
    );

    dispatch_click(&trigger);
    assert!(
        !component.borrow().is_open("products"),
        "trigger 再クリックで toggle により閉じること"
    );
}

#[wasm_bindgen_test]
fn navigation_menu_trigger_data_value_xss_payload_click_does_not_produce_script_element() {
    use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};
    use fandhe_frontend_headless_ui::state::OpenState;

    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-navigation-menu-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>alert(1)</script>";
    let html = fandhe_frontend_core::render(&navigation_menu::trigger(
        OpenState::Closed,
        false,
        payload,
        None,
        None,
        vec![],
        vec![],
    ));
    container.set_inner_html(&html);
    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "data-value に XSS ペイロードを含む trigger の展開時点で script 要素が生成されてはならない"
    );

    let trigger_el = container
        .first_element_child()
        .expect("navigation-menu trigger element must exist");

    let component = Rc::new(RefCell::new(NavigationMenu::default()));
    wire_headless_component(trigger_el.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger_el);
    assert!(component.borrow().is_open(payload));

    let rendered = fandhe_frontend_core::render(
        &fandhe_frontend_interactive::render_for_hydration(&*component.borrow()),
    );
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(rendered.contains("&lt;script&gt;"));
}

#[wasm_bindgen_test]
fn calendar_day_trigger_click_selects_date_in_real_dom() {
    use fandhe_frontend_headless_ui::calendar::{self, Calendar};
    use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-calendar-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let today = PlainDate::new(2026, 7, 15).expect("valid date");
    let day = PlainDate::new(2026, 7, 20).expect("valid date");
    let html = fandhe_frontend_core::render(&calendar::day_trigger(
        day,
        false,
        false,
        false,
        false,
        None,
        vec![],
        vec![],
    ));
    container.set_inner_html(&html);
    let trigger = container
        .first_element_child()
        .expect("day-trigger element must exist");

    let cal = Calendar::new(2026, 7, today, None, None, None, Weekday::Monday)
        .expect("valid calendar construction");
    let component = Rc::new(RefCell::new(cal));
    wire_headless_component(trigger.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger);
    assert_eq!(
        component.borrow().selected(),
        Some(day),
        "day-trigger クリック後は該当日付が選択されていること"
    );
}

#[wasm_bindgen_test]
fn menubar_trigger_click_toggles_open_menu_in_real_dom() {
    use fandhe_frontend_headless_ui::menubar::{self, Menubar};
    use fandhe_frontend_headless_ui::state::OpenState;
    use fandhe_frontend_headless_ui::Orientation;

    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "headless-menubar-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&menubar::root(
        Orientation::Horizontal,
        "App menu",
        vec![],
        vec![menubar::menu(
            OpenState::Closed,
            vec![],
            vec![menubar::trigger(
                true,
                OpenState::Closed,
                false,
                false,
                0,
                None,
                vec![],
                vec![fandhe_frontend_core::text("File")],
            )],
        )],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("menubar root must exist");
    let trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");

    let component = Rc::new(RefCell::new(Menubar::new(
        0,
        1,
        None,
        false,
        Orientation::Horizontal,
    )));
    wire_headless_component(root.clone(), component.clone(), |_state, _root| {})
        .expect("wire_headless_component must not fail");

    dispatch_click(&trigger);
    assert_eq!(
        component.borrow().open(),
        Some(0),
        "trigger クリック後は該当 Menu が開いていること"
    );

    dispatch_click(&trigger);
    assert_eq!(
        component.borrow().open(),
        None,
        "trigger 再クリックで toggle により閉じること"
    );
}
