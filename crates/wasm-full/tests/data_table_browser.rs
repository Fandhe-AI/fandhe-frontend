//! `fandhe_frontend_wasm_full::data_table`（イシュー #2126、親 #2124）の
//! 実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/data_table.rs` の native `#[cfg(test)] mod tests`
//! は純粋ロジック層とヘッドレス出力のドリフト検知までを検証済みである
//! （`tests/data_table_native.rs` も参照）。本ファイルはその先、配線層
//! （`wiring`、`#[cfg(target_arch = "wasm32")]`）が実 DOM（headless
//! Chromium）上で
//!
//! 1. sort-trigger クリック → `none → ascending → descending → none`
//!    巡回、column-header の `aria-sort`/`data-sort` と sort-trigger の
//!    `data-sort` の同期、`"data-table:sort"` 通知
//! 2. 別列の sort-trigger クリックで旧列が `none` に戻り新列が
//!    `ascending` になること
//! 3. 改ざん DOM（2 列が同時に非 `none`）での sort no-op
//! 4. `data-disabled` 祖先を持つ sort-trigger クリックの no-op
//! 5. 列表示切替（menu `checkbox-item`）クリック → 対象列の `hidden`/
//!    `data-hidden` 付け外し・`data-state`/`aria-checked` 同期・
//!    `"data-table:toggle-column"` 通知
//! 6. `aria-disabled="true"` の checkbox-item クリックの no-op
//! 7. 配線時の select-all `indeterminate` DOM プロパティ同期と、
//!    `data-state` 変更後の `MutationObserver` 経由の再同期
//! 8. pagination next/prev クリック → `item` の `data-selected` 移動、
//!    境界到達時の `prev`/`first`/`next`/`last` トリガーの
//!    `disabled`/`aria-disabled`/`data-disabled` 3 点セット、
//!    `"data-table:page"` 通知
//! 9. link モード（`<a href>`）の pagination item クリックは触らない
//! 10. 同一 container 内の 2 インスタンスの独立性
//! 11. `Runtime::hydrate` 統合（`questionnaire_browser.rs`
//!     `runtime_dirty_rerender` と同型）: `"data-table:sort"` 通知を
//!     `C` が受理して束縛点を再描画すること
//!
//! を検証する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, render, table as table_el, tbody, thead, tr, Node};
use fandhe_frontend_headless_ui::checkbox::{hidden_input, CheckboxProps, CheckedState};
use fandhe_frontend_headless_ui::data_table::{
    column_attrs, column_toggle_item, ColumnProps, DataTable, DataTableProps,
};
use fandhe_frontend_headless_ui::pagination::{ItemMode, Pagination};
use fandhe_frontend_wasm_full::data_table::{
    wire_data_table_events, ACTION_PAGE, ACTION_SORT, ACTION_TOGGLE_COLUMN, COLUMN_TOGGLE_MARKER,
};
use fandhe_frontend_wasm_full::events::ActionRef;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlInputElement, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`message_scroller_browser.rs::create_container` と同型）。
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click`（bubbles: true）を生成する（`questionnaire_browser.rs::synthetic_click`
/// と同型）。
fn synthetic_click() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// `condition` が真になるまでポーリングする（`questionnaire_browser.rs::wait_for`
/// と同型）。
async fn wait_for(desc: &str, mut condition: impl FnMut() -> bool) {
    for _ in 0..500 {
        if condition() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10,
                )
                .expect("setTimeout must not fail");
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("timeout promise must resolve");
    }
    panic!("wait_for timed out after 500 polls (~5s): {desc}");
}

/// `setTimeout(0)` を 1 回発行して次 tick をまたぐ、条件ポーリングを
/// 伴わない単純な猶予待機（no-op click の DOM 不変確認用）。
async fn settle() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                0,
            )
            .expect("setTimeout must not fail");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");
}

fn part(root: &Element, scope: &str, part: &str) -> Element {
    root.query_selector(&format!(r#"[data-scope="{scope}"][data-part="{part}"]"#))
        .expect("query_selector must not fail")
        .unwrap_or_else(|| panic!("{scope}/{part} part must exist"))
}

fn part_by_value(root: &Element, scope: &str, part: &str, value: &str) -> Element {
    root.query_selector(&format!(
        r#"[data-scope="{scope}"][data-part="{part}"][data-value="{value}"]"#
    ))
    .expect("query_selector must not fail")
    .unwrap_or_else(|| panic!("{scope}/{part} data-value={value} must exist"))
}

/// 最小構成の DataTable フィクスチャ（`id` はインスタンス識別用）。
///
/// - column-header: `name`（sortable）・`email`（非 sortable、非表示切替対象）
/// - sort-trigger: `name` 列のみ
/// - menu checkbox-item: `email` 列の表示切替（既定は表示中）
/// - select-all: `checkbox::hidden_input` を子に持つ
/// - footer: `Pagination::new(5, 1, 1, 1, 1)`（5 ページ、1 ページ目選択）
fn build_fixture(id: &str) -> Node {
    let table = DataTable::default();
    let pagination = Pagination::new(5, 1, 1, 1, 1);

    let email_column = ColumnProps {
        id: "email",
        hidden: false,
    };

    // `column_header`/`select_all` は `th`、`select_row`（本フィクスチャ
    // では `column_attrs` を付けた素の `td`）は `td` を生成するため、
    // `table`/`thead`/`tbody`/`tr`（core のノード API）で正規の table
    // 構造を組んでからその中へ配置する。`div`（`DataTable::root`）直下に
    // 直接置くと、ブラウザの HTML フラグメント解析（コンテキストが
    // `div` のため insertion mode が「in body」になる）で `th`/`td`
    // start tag がパースエラーとして無視され、`data-part="column-header"`
    // 等が DOM 上に一切現れない（`crates/docs-site/src/primitive_showcase/data_display_utilities.rs`
    // の codex-review P1 是正、PR #2303 と同じ原因）。
    // `select_all`（`th`）も同じ理由で table 構造内へ置く必要がある
    // （配下の `checkbox::hidden_input` ごと HTML フラグメント解析で
    // 落とされないようにする）。
    let select_all_header = DataTable::select_all(
        CheckedState::Unchecked,
        vec![],
        vec![hidden_input(
            &CheckboxProps::default(),
            "select-all",
            "on",
            vec![],
        )],
    );
    let header_row = tr(
        vec![],
        vec![
            select_all_header,
            table.column_header("name", true, vec![], vec![]),
            table.column_header("email", false, vec![], vec![]),
        ],
    );
    let body_row = tr(vec![], vec![el("td", column_attrs(&email_column), vec![])]);
    let data_table_markup = table_el(
        vec![],
        vec![
            thead(vec![], vec![header_row]),
            tbody(vec![], vec![body_row]),
        ],
    );

    DataTable::root(
        DataTableProps::default(),
        vec![("id", id)],
        vec![
            data_table_markup,
            table.sort_trigger("name", vec![], vec![]),
            column_toggle_item(&email_column, false, false, vec![], vec![]),
            pagination.root(
                "pagination",
                vec![],
                vec![
                    pagination.first_trigger(ItemMode::Button, vec![], vec![]),
                    pagination.prev_trigger(ItemMode::Button, vec![], vec![]),
                    pagination.item(ItemMode::Button, 1, false, vec![], vec![]),
                    pagination.item(ItemMode::Button, 2, false, vec![], vec![]),
                    pagination.item(ItemMode::Button, 3, false, vec![], vec![]),
                    pagination.item(ItemMode::Button, 4, false, vec![], vec![]),
                    pagination.item(ItemMode::Button, 5, false, vec![], vec![]),
                    pagination.next_trigger(ItemMode::Button, vec![], vec![]),
                    pagination.last_trigger(ItemMode::Button, vec![], vec![]),
                ],
            ),
        ],
    )
}

fn mount_fixture(document: &Document, container_id: &str, table_id: &str) -> Element {
    let container = create_container(document, container_id);
    let html = render(&build_fixture(table_id));
    container
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    document
        .get_element_by_id(table_id)
        .expect("rendered data-table root must have the expected id")
}

fn wire(root: &Element) -> Rc<RefCell<Vec<ActionRef>>> {
    let actions: Rc<RefCell<Vec<ActionRef>>> = Rc::new(RefCell::new(Vec::new()));
    let actions_clone = actions.clone();
    wire_data_table_events(root.clone(), move |action_ref: ActionRef| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_data_table_events must not fail");
    actions
}

// --- ソート ---

#[wasm_bindgen_test]
async fn sort_trigger_click_cycles_none_ascending_descending_none() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-sort-cycle-container", "dt-sort-cycle");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let header = part(&root, "data-table", "column-header");
    let trigger = part(&root, "data-table", "sort-trigger");

    for expected in ["ascending", "descending", "none"] {
        trigger
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");
        wait_for(
            "column-header aria-sort reflects the cycled direction",
            || header.get_attribute("aria-sort").as_deref() == Some(expected),
        )
        .await;
        assert_eq!(header.get_attribute("data-sort").as_deref(), Some(expected));
        assert_eq!(
            trigger.get_attribute("data-sort").as_deref(),
            Some(expected)
        );
    }

    assert_eq!(actions.borrow().len(), 3);
    assert_eq!(actions.borrow()[0].action, ACTION_SORT);
}

#[wasm_bindgen_test]
async fn tampered_dom_with_two_sorted_columns_is_a_no_op() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-sort-tamper-container", "dt-sort-tamper");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let header = part(&root, "data-table", "column-header");
    header
        .set_attribute("aria-sort", "descending")
        .expect("set_attribute must not fail");
    // フィクスチャの `email` 列は非 sortable（`aria-sort` を持たない）ため、
    // ここでは name 列自身へ矛盾する値を仕込めない。代わりに 2 つ目の
    // sortable column-header を動的に追加して改ざんを再現する。
    let extra_header = document
        .create_element("th")
        .expect("create_element must not fail");
    extra_header
        .set_attribute("data-scope", "data-table")
        .expect("set_attribute must not fail");
    extra_header
        .set_attribute("data-part", "column-header")
        .expect("set_attribute must not fail");
    extra_header
        .set_attribute("data-column", "extra")
        .expect("set_attribute must not fail");
    extra_header
        .set_attribute("aria-sort", "ascending")
        .expect("set_attribute must not fail");
    root.append_child(&extra_header)
        .expect("append_child must not fail");

    let trigger = part(&root, "data-table", "sort-trigger");
    trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(
        header.get_attribute("aria-sort").as_deref(),
        Some("descending")
    );
    assert_eq!(trigger.get_attribute("data-sort").as_deref(), Some("none"));
    assert!(actions.borrow().is_empty());
}

#[wasm_bindgen_test]
async fn data_disabled_ancestor_suppresses_sort_click() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-sort-disabled-container", "dt-sort-disabled");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    root.set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");
    let header = part(&root, "data-table", "column-header");
    let trigger = part(&root, "data-table", "sort-trigger");
    trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(header.get_attribute("aria-sort").as_deref(), Some("none"));
    assert!(actions.borrow().is_empty());
}

// --- 列表示切替 ---

#[wasm_bindgen_test]
async fn column_toggle_item_click_hides_and_shows_matching_cells() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-toggle-container", "dt-toggle");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let header = part(&root, "data-table", "column-header"); // note: 最初の一致は "name"
    let email_header = root
        .query_selector(
            r#"[data-scope="data-table"][data-part="column-header"][data-column="email"]"#,
        )
        .expect("query_selector must not fail")
        .expect("email column-header must exist");
    let email_cell = root
        .query_selector(r#"td[data-column="email"]"#)
        .expect("query_selector must not fail")
        .expect("email cell must exist");
    let toggle_item = part_by_value(&root, "menu", "checkbox-item", "email");

    // name 列は sort 対象、email 列だけが toggle 対象であることの前提確認。
    assert!(header.get_attribute("data-column").as_deref() != Some("email"));

    toggle_item
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("email column-header gains hidden after toggle", || {
        email_header.has_attribute("hidden")
    })
    .await;
    assert!(email_header.has_attribute("data-hidden"));
    assert!(email_cell.has_attribute("hidden"));
    assert!(email_cell.has_attribute("data-hidden"));
    assert_eq!(
        toggle_item.get_attribute("data-state").as_deref(),
        Some("unchecked")
    );
    assert_eq!(
        toggle_item.get_attribute("aria-checked").as_deref(),
        Some("false")
    );

    toggle_item
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(
        "email column-header loses hidden after second toggle",
        || !email_header.has_attribute("hidden"),
    )
    .await;
    assert!(!email_cell.has_attribute("hidden"));
    assert_eq!(
        toggle_item.get_attribute("data-state").as_deref(),
        Some("checked")
    );

    assert_eq!(actions.borrow().len(), 2);
    assert_eq!(actions.borrow()[0].action, ACTION_TOGGLE_COLUMN);
}

#[wasm_bindgen_test]
async fn aria_disabled_checkbox_item_click_is_a_no_op() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(
        &document,
        "dt-toggle-disabled-container",
        "dt-toggle-disabled",
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let toggle_item = part_by_value(&root, "menu", "checkbox-item", "email");
    toggle_item
        .set_attribute("aria-disabled", "true")
        .expect("set_attribute must not fail");
    toggle_item
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(
        toggle_item.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    assert!(actions.borrow().is_empty());
}

/// codex-review P1 指摘の回帰: data-table 内の全 `menu`/`checkbox-item` を
/// 列表示切替として無条件処理すると、対応する列の存在しない
/// `checkbox-item`（行操作・フィルター用など）のクリックでも誤って
/// 何らかの列を隠してしまう。`data-value` が実在しない列 id（`bogus`）の
/// `checkbox-item` を動的追加し、クリックしても既存列（`email`）が一切
/// 変化せず通知も飛ばないことを固定する。
#[wasm_bindgen_test]
async fn column_toggle_item_ignores_checkbox_item_with_unmatched_column() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(
        &document,
        "dt-toggle-unmatched-container",
        "dt-toggle-unmatched",
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let email_header = root
        .query_selector(
            r#"[data-scope="data-table"][data-part="column-header"][data-column="email"]"#,
        )
        .expect("query_selector must not fail")
        .expect("email column-header must exist");

    let unmatched = document
        .create_element("button")
        .expect("create_element must not fail");
    unmatched
        .set_attribute("data-scope", "menu")
        .expect("set_attribute must not fail");
    unmatched
        .set_attribute("data-part", "checkbox-item")
        .expect("set_attribute must not fail");
    unmatched
        .set_attribute("data-value", "bogus")
        .expect("set_attribute must not fail");
    unmatched
        .set_attribute("data-state", "checked")
        .expect("set_attribute must not fail");
    root.append_child(&unmatched)
        .expect("append_child must not fail");

    unmatched
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert!(!email_header.has_attribute("hidden"));
    assert!(!email_header.has_attribute("data-hidden"));
    // no-op のため `data-state` も書き換えられていないこと。
    assert_eq!(
        unmatched.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    assert!(actions.borrow().is_empty());
}

#[wasm_bindgen_test]
async fn column_toggle_item_ignores_unmarked_checkbox_item_with_coincidentally_matching_value() {
    // codex-review P1 指摘の再現: 同じ data-table 内に列表示切替と無関係
    // な menu checkbox-item（例: 通知方法選択メニュー）があり、その
    // `data-value` が偶然実在する列 id（"email"）と一致する場合でも、
    // `column_toggle_item`（headless-ui）由来ではない（＝
    // `COLUMN_TOGGLE_MARKER` を持たない）要素は no-op でなければならない。
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(
        &document,
        "dt-toggle-unmarked-container",
        "dt-toggle-unmarked",
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let email_header = root
        .query_selector(
            r#"[data-scope="data-table"][data-part="column-header"][data-column="email"]"#,
        )
        .expect("query_selector must not fail")
        .expect("email column-header must exist");

    // マーカーなしの無関係な checkbox-item（例: 通知方法「email」選択）。
    let unrelated = document
        .create_element("button")
        .expect("create_element must not fail");
    unrelated
        .set_attribute("data-scope", "menu")
        .expect("set_attribute must not fail");
    unrelated
        .set_attribute("data-part", "checkbox-item")
        .expect("set_attribute must not fail");
    unrelated
        .set_attribute("data-value", "email")
        .expect("set_attribute must not fail");
    unrelated
        .set_attribute("data-state", "unchecked")
        .expect("set_attribute must not fail");
    assert!(!unrelated.has_attribute(COLUMN_TOGGLE_MARKER));
    root.append_child(&unrelated)
        .expect("append_child must not fail");

    unrelated
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert!(!email_header.has_attribute("hidden"));
    assert!(!email_header.has_attribute("data-hidden"));
    // no-op のため無関係な checkbox-item 自身の `data-state` も
    // 書き換えられていないこと（列表示切替として誤処理されていない）。
    assert_eq!(
        unrelated.get_attribute("data-state").as_deref(),
        Some("unchecked")
    );
    assert!(actions.borrow().is_empty());
}

// --- select-all indeterminate ---

#[wasm_bindgen_test]
async fn select_all_indeterminate_syncs_on_wire_and_via_mutation_observer() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-indeterminate-container", "dt-indeterminate");
    let _cleanup = RemoveOnDrop(root.clone());
    let _actions = wire(&root);

    let input: HtmlInputElement = part(&root, "checkbox", "hidden-input")
        .dyn_into()
        .expect("hidden-input must be an HtmlInputElement");
    // フィクスチャは `CheckedState::Unchecked` で構築しているため、配線時
    // 同期は `indeterminate == false` を維持する。
    assert!(!input.indeterminate());

    input
        .set_attribute("data-state", "indeterminate")
        .expect("set_attribute must not fail");
    wait_for(
        "hidden-input indeterminate becomes true after data-state change",
        || input.indeterminate(),
    )
    .await;

    input
        .set_attribute("data-state", "checked")
        .expect("set_attribute must not fail");
    wait_for("hidden-input indeterminate becomes false again", || {
        !input.indeterminate()
    })
    .await;
}

// --- ページング ---

#[wasm_bindgen_test]
async fn pagination_next_click_moves_selection_and_updates_boundary_triggers() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-page-next-container", "dt-page-next");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    let next_trigger = part(&root, "pagination", "next-trigger");
    let prev_trigger = part(&root, "pagination", "prev-trigger");
    let item1 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="1"]"#)
        .expect("query_selector must not fail")
        .expect("item 1 must exist");
    let item2 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="2"]"#)
        .expect("query_selector must not fail")
        .expect("item 2 must exist");

    assert!(item1.has_attribute("data-selected"));
    // ページ 1（先頭）では `can_prev() == false` のため prev-trigger は
    // 配線前から `disabled`（headless-ui `Pagination::prev_trigger` の
    // 出力契約）。
    assert!(prev_trigger.has_attribute("disabled"));

    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 2 becomes selected after next click", || {
        item2.has_attribute("data-selected")
    })
    .await;
    assert!(!item1.has_attribute("data-selected"));
    assert!(!prev_trigger.has_attribute("disabled"));

    assert_eq!(actions.borrow().len(), 1);
    assert_eq!(actions.borrow()[0].action, ACTION_PAGE);
}

#[wasm_bindgen_test]
async fn pagination_last_click_disables_next_and_last_triggers() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-page-last-container", "dt-page-last");
    let _cleanup = RemoveOnDrop(root.clone());
    let _actions = wire(&root);

    let last_trigger = part(&root, "pagination", "last-trigger");
    let next_trigger = part(&root, "pagination", "next-trigger");
    let item5 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="5"]"#)
        .expect("query_selector must not fail")
        .expect("item 5 must exist");

    last_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 5 becomes selected after last click", || {
        item5.has_attribute("data-selected")
    })
    .await;

    assert!(next_trigger.has_attribute("disabled"));
    assert_eq!(
        next_trigger.get_attribute("aria-disabled").as_deref(),
        Some("true")
    );
    assert!(next_trigger.has_attribute("data-disabled"));
    assert!(last_trigger.has_attribute("disabled"));

    // 末尾ページで再度 next を押しても変化なし（no-op）。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;
    assert!(item5.has_attribute("data-selected"));
}

/// codex-review P1 指摘の回帰: `item` クリックで `data-selected` だけでなく
/// `aria-current="page"` も同期すること（旧ページに `aria-current` が
/// 残らないこと）。
#[wasm_bindgen_test]
async fn pagination_next_click_syncs_aria_current() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(
        &document,
        "dt-page-aria-current-container",
        "dt-page-aria-current",
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let _actions = wire(&root);

    let next_trigger = part(&root, "pagination", "next-trigger");
    let item1 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="1"]"#)
        .expect("query_selector must not fail")
        .expect("item 1 must exist");
    let item2 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="2"]"#)
        .expect("query_selector must not fail")
        .expect("item 2 must exist");

    assert_eq!(item1.get_attribute("aria-current").as_deref(), Some("page"));

    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 2 gains aria-current after next click", || {
        item2.get_attribute("aria-current").as_deref() == Some("page")
    })
    .await;
    assert_eq!(item1.get_attribute("aria-current"), None);
}

/// codex-review P1 指摘の回帰: 省略記号（ellipsis）で遷移先ページの
/// `item` が DOM 上に存在しない場合でも、その後の `prev`/`last` 操作が
/// 機能し続けること（現在ページを見失わない）。フィクスチャの `item 2`
/// を DOM から取り除いて「省略された」状態を模擬してから `next` を 2 回
/// クリックし、`item 4`（遷移先が存在する）へ正しく戻れることを確認する。
#[wasm_bindgen_test]
async fn pagination_next_click_survives_missing_item_for_omitted_page() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-page-omitted-container", "dt-page-omitted");
    let _cleanup = RemoveOnDrop(root.clone());
    let _actions = wire(&root);

    let next_trigger = part(&root, "pagination", "next-trigger");
    let prev_trigger = part(&root, "pagination", "prev-trigger");
    let item2 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="2"]"#)
        .expect("query_selector must not fail")
        .expect("item 2 must exist");
    // 省略記号を模擬: ページ 2 の item を DOM から取り除く。
    item2.remove();
    let item3 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="3"]"#)
        .expect("query_selector must not fail")
        .expect("item 3 must exist");
    let item4 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="4"]"#)
        .expect("query_selector must not fail")
        .expect("item 4 must exist");

    // 1 回目の next: 遷移先（ページ 2）の item が存在しないため、DOM 上は
    // どの item も選択されない（既知の限界）が、現在ページの内部状態は
    // 失われないこと。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;
    assert!(!item3.has_attribute("data-selected"));
    assert!(!prev_trigger.has_attribute("disabled"));

    // 2 回目の next: 現在ページ（2）を見失っていなければページ 3 へ進む。
    // 見失って `read_pagination_state` が `None` を返すようになっていた
    // 場合はここで no-op になり `item3` が選択されないため回帰を検知する。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 3 becomes selected after second next click", || {
        item3.has_attribute("data-selected")
    })
    .await;
    assert_eq!(item3.get_attribute("aria-current").as_deref(), Some("page"));

    // prev で正しくページ 2 相当（item は存在しないため選択反映はされない）
    // へ戻れ、その後さらに next でページ 3 → 4 へ進めること（内部状態が
    // 一貫して追跡され続けることの確認）。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 4 becomes selected after third next click", || {
        item4.has_attribute("data-selected")
    })
    .await;
}

#[wasm_bindgen_test]
async fn pagination_next_click_prefers_dom_selected_item_over_stale_internal_attribute() {
    // codex-review P1 指摘の再現: `pagination_root` を使い回したままアプリが
    // `item` 群を再描画する構成（例: フィルター変更でページ 1 へ戻す）で、
    // 前回の `handle_page` が書き戻した内部属性（`data-current-page`）が
    // 再描画後の DOM 状態より古いまま残っていても、DOM 上に一意に存在する
    // `data-selected` item を現在ページの正として優先しなければならない。
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-page-stale-container", "dt-page-stale");
    let _cleanup = RemoveOnDrop(root.clone());
    let _actions = wire(&root);

    let next_trigger = part(&root, "pagination", "next-trigger");
    let item1 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="1"]"#)
        .expect("query_selector must not fail")
        .expect("item 1 must exist");
    let item2 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="2"]"#)
        .expect("query_selector must not fail")
        .expect("item 2 must exist");
    let item3 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="3"]"#)
        .expect("query_selector must not fail")
        .expect("item 3 must exist");
    let item4 = root
        .query_selector(r#"[data-scope="pagination"][data-part="item"][data-index="4"]"#)
        .expect("query_selector must not fail")
        .expect("item 4 must exist");

    // ページ 3 まで進め、`data-current-page="3"` を書き戻させる。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 2 becomes selected after first next click", || {
        item2.has_attribute("data-selected")
    })
    .await;
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("item 3 becomes selected after second next click", || {
        item3.has_attribute("data-selected")
    })
    .await;

    // アプリがフィルター変更等で `item` 群を再描画し、ページ 1 へ戻した
    // ことを模擬する（`pagination_root` 自身は使い回すため
    // `data-current-page="3"` は書き換えられずそのまま残る）。
    item3.remove_attribute("data-selected").ok();
    item3.remove_attribute("aria-current").ok();
    item1
        .set_attribute("data-selected", "")
        .expect("set_attribute must not fail");
    item1
        .set_attribute("aria-current", "page")
        .expect("set_attribute must not fail");

    // 内部属性が優先されるバグがあれば、ここでのクリックは
    // 「ページ 3」を正として扱いページ 4 へ進んでしまう
    // （`item4` が選択される）。修正後は DOM 上のページ 1 を正として
    // ページ 2 へ進む（`item2` が選択される）。
    next_trigger
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(
        "item 2 becomes selected again after re-render resets to page 1",
        || item2.has_attribute("data-selected"),
    )
    .await;
    assert!(!item4.has_attribute("data-selected"));
    assert_eq!(item2.get_attribute("aria-current").as_deref(), Some("page"));
}

#[wasm_bindgen_test]
async fn pagination_link_mode_item_click_is_untouched() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let root = mount_fixture(&document, "dt-page-link-container", "dt-page-link");
    let _cleanup = RemoveOnDrop(root.clone());
    let actions = wire(&root);

    // フィクスチャの item は button モード。link モード要素を動的に追加し、
    // それを起点に click しても data-table 配線が一切触らないことを確認する。
    let link = document
        .create_element("a")
        .expect("create_element must not fail");
    // フラグメントのみの href にする（`dispatch_event` は合成イベントでも
    // ブラウザのアクティベーション動作〔リンクのナビゲーション〕を実行する
    // ため、クエリ文字列やパスへの遷移だとテストページが実際に再読込され
    // ハーネスが壊れる。フラグメントナビゲーションは無害）。
    link.set_attribute("href", "#dt-page-link-noop")
        .expect("set_attribute must not fail");
    link.set_attribute("data-scope", "pagination")
        .expect("set_attribute must not fail");
    link.set_attribute("data-part", "item")
        .expect("set_attribute must not fail");
    link.set_attribute("data-index", "2")
        .expect("set_attribute must not fail");
    let pagination_root = part(&root, "pagination", "root");
    pagination_root
        .append_child(&link)
        .expect("append_child must not fail");

    link.dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert!(!link.has_attribute("data-selected"));
    assert!(actions.borrow().is_empty());
}

// --- 複数インスタンス ---

#[wasm_bindgen_test]
async fn two_instances_in_the_same_container_are_independent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "dt-multi-container");
    let _cleanup = RemoveOnDrop(container.clone());

    let html_a = render(&build_fixture("dt-multi-a"));
    let html_b = render(&build_fixture("dt-multi-b"));
    container
        .insert_adjacent_html("beforeend", &html_a)
        .expect("insert_adjacent_html must not fail");
    container
        .insert_adjacent_html("beforeend", &html_b)
        .expect("insert_adjacent_html must not fail");

    let actions = wire(&container);

    let root_a = document
        .get_element_by_id("dt-multi-a")
        .expect("instance a must exist");
    let root_b = document
        .get_element_by_id("dt-multi-b")
        .expect("instance b must exist");
    let trigger_a = part(&root_a, "data-table", "sort-trigger");
    let header_b = part(&root_b, "data-table", "column-header");

    trigger_a
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("instance a sort-trigger reflects ascending", || {
        part(&root_a, "data-table", "sort-trigger")
            .get_attribute("data-sort")
            .as_deref()
            == Some("ascending")
    })
    .await;

    assert_eq!(header_b.get_attribute("aria-sort").as_deref(), Some("none"));
    assert_eq!(actions.borrow().len(), 1);
}

// --- `Runtime::hydrate` 統合 ---

mod runtime_dirty_rerender {
    use super::{synthetic_click, wait_for, RemoveOnDrop};
    use fandhe_frontend_core::{bind_text, render, table as table_el, tbody, thead, tr, Node};
    use fandhe_frontend_headless_ui::data_table::{DataTable, DataTableProps};
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::data_table::decode_sort_payload;
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen_test::*;
    use web_sys::{Document, Element};

    /// `Runtime::wire_data_table` の dispatch 後再描画接続を実 DOM で
    /// 固定するための最小ホスト（`questionnaire_browser.rs::QuestionnaireHost`
    /// と同じ設計）。
    struct DataTableHost {
        table: DataTable,
        sort_label: String,
        dirty: Vec<&'static str>,
        root_id: String,
    }

    impl DataTableHost {
        fn new(table: DataTable) -> Self {
            let sort_label = sort_label_of(&table);
            Self {
                table,
                sort_label,
                dirty: Vec::new(),
                root_id: String::new(),
            }
        }

        fn with_root_id(mut self, root_id: &str) -> Self {
            self.root_id = root_id.to_string();
            self
        }
    }

    fn sort_label_of(table: &DataTable) -> String {
        match table.sort() {
            Some((id, dir)) => format!("{id}:{}", dir.as_data_sort()),
            None => "none".to_string(),
        }
    }

    impl Component for DataTableHost {
        // `data-table:sort` 通知は headless-ui の `DataTableAction` の
        // `Sort(String)` へ委譲前提と同じ payload（列 id）を運ぶが、
        // `Runtime::wire_data_table` の橋渡し先はアプリ側 `C::decode_action`
        // であり修飾済みアクション名（`"data-table:sort"`）をそのまま渡す
        // 契約（`data_table.rs` モジュール doc「責務境界」節参照）。ホストは
        // 修飾を剥がしてから `DataTable::decode_action` へ委譲する。
        type Action = fandhe_frontend_headless_ui::data_table::DataTableAction;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            self.table.update(action);
            let new_label = sort_label_of(&self.table);
            if new_label != self.sort_label {
                self.sort_label = new_label;
                self.dirty.push("sort_label");
            }
        }

        fn view(&self) -> Node {
            // `column_header` は `th` を生成するため、`table`/`thead`/`tbody`/
            // `tr`（core のノード API）で正規の table 構造を組んでからその
            // 中へ配置する。`div`（`DataTable::root`）直下に直接置くと、
            // ブラウザの HTML フラグメント解析で `th` start tag がパース
            // エラーとして無視され `column-header` が DOM 上に一切現れず、
            // `handle_sort` の改ざん検知（sortable header 収集）が常に
            // 空集合を見て no-op になる（本ファイル冒頭 `build_fixture`
            // と同じ原因、`crates/docs-site/src/primitive_showcase/data_display_utilities.rs`
            // の codex-review P1 是正、PR #2303 参照）。
            let header_row = tr(
                vec![],
                vec![self.table.column_header("name", true, vec![], vec![])],
            );
            let data_table_markup = table_el(
                vec![],
                vec![thead(vec![], vec![header_row]), tbody(vec![], vec![])],
            );
            DataTable::root(
                DataTableProps::default(),
                vec![("id", self.root_id.as_str())],
                vec![
                    data_table_markup,
                    self.table.sort_trigger("name", vec![], vec![]),
                    bind_text(
                        "span",
                        vec![("data-testid", "sort-label")],
                        "sort_label",
                        self.sort_label.clone(),
                    ),
                ],
            )
        }

        fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
            // `data-table:sort` の payload は
            // `encode_sort_payload(direction, column_id, instance_id)`
            // （`codec::encode_list` による 3 要素エンコード）であり、
            // `DataTable::decode_action("sort", ..)` が期待する「列 id
            // 単体」ではない。ホストが列 id のみを取り出してから委譲する
            // （`data_table.rs` モジュール doc「責務境界」節が明示する、
            // 通知 payload の解釈はアプリ側の責務という契約）。
            match name.strip_prefix("data-table:") {
                Some("sort") => {
                    let (_, column_id, _) = decode_sort_payload(payload)?;
                    DataTable::decode_action("sort", &column_id)
                }
                Some(_) => None,
                None => DataTable::decode_action(name, payload),
            }
        }
    }

    impl DirtyTracked for DataTableHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    impl BindingSource for DataTableHost {
        fn bound_value(&self, field: &str) -> Option<BoundValue> {
            match field {
                "sort_label" => Some(BoundValue::Text(self.sort_label.clone())),
                _ => None,
            }
        }
    }

    impl Hydrate for DataTableHost {
        fn hydration_attrs(&self) -> Vec<(String, String)> {
            self.table.hydration_attrs()
        }

        fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
            DataTable::from_hydration_attrs(attrs).map(Self::new)
        }
    }

    fn mount_host_and_hydrate(
        document: &Document,
        root_id: &str,
        table: DataTable,
    ) -> (Element, Runtime<DataTableHost>) {
        let host = DataTableHost::new(table).with_root_id(root_id);
        let html = render(&host.view());
        document
            .body()
            .expect("document body must exist in browser test environment")
            .insert_adjacent_html("beforeend", &html)
            .expect("insert_adjacent_html must not fail");
        let root_el: Element = document
            .get_element_by_id(root_id)
            .expect("rendered DataTable root must have the expected id");

        for (name, value) in host.hydration_attrs() {
            root_el
                .set_attribute(&name, &value)
                .expect("set_attribute must not fail");
        }

        let table_for_host =
            DataTable::from_hydration_attrs(&host.hydration_attrs()).unwrap_or_default();
        let runtime = Runtime::hydrate(root_id, DataTableHost::new(table_for_host))
            .expect("hydrate must succeed for well-formed attrs");
        assert_eq!(
            runtime.root().id(),
            root_id,
            "hydrate は root_id 要素自身を DataTable root として復元すること"
        );

        (root_el, runtime)
    }

    #[wasm_bindgen_test]
    async fn runtime_hydrate_click_rerenders_host_binding_after_sort_notification() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");

        let (root_el, _runtime) = mount_host_and_hydrate(
            &document,
            "data-table-host-runtime-dirty-rerender-root",
            DataTable::default(),
        );
        let _cleanup = RemoveOnDrop(root_el.clone());

        let sort_label = || {
            root_el
                .query_selector("[data-bind-text='sort_label']")
                .expect("query_selector must not fail")
                .expect("sort_label binding point must exist")
                .text_content()
                .unwrap_or_default()
        };
        assert_eq!(sort_label(), "none");

        let trigger = root_el
            .query_selector(r#"[data-scope="data-table"][data-part="sort-trigger"]"#)
            .expect("query_selector must not fail")
            .expect("sort-trigger must exist");
        trigger
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");

        wait_for(
            "sort_label binding reflects the new sort after dispatch",
            || sort_label() == "name:ascending",
        )
        .await;
    }
}
