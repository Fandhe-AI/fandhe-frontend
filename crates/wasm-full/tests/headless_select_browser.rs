//! `fandhe_frontend_wasm_full::headless_select`（イシュー #642/#1619）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/headless_select.rs` 相当の native ユニットテストは
//! `crates/wasm-full/src/headless_select.rs` 内の `#[cfg(test)]` モジュールが
//! 担う（純粋ロジック層のみ）。本ファイルはその先、`sync_select_value_text`
//! が実 DOM 上で item-text/item-indicator の `data-state`/`hidden` を
//! 正しく反映し、かつインスタンス境界（`[data-scope="select"]
//! [data-part="root"]`）を越えてネストした別 Select インスタンスの item を
//! 誤って書き換えないことを検証する（codex-review P1 是正・Cursor 指摘、
//! PR #1899）。
//!
//! # 検証内容
//!
//! 1. `sync_select_value_text` 呼び出し後、選択された item の
//!    item-text/item-indicator の `data-state` が `"open"` になり、
//!    indicator の `hidden` が外れること。非選択 item 側は `"closed"`/
//!    `hidden` 付与のままであること（選択切り替え後の再同期も含む）。
//! 2. `root` 配下に別 Select インスタンス（同じ `data-scope="select"`）が
//!    ネストして存在する場合、外側インスタンスの同期がネストした内側
//!    インスタンスの item へ一切波及しないこと。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::select::{
    content, item, item_indicator, item_text, root, trigger, value_text, Select, SelectAction,
    SelectProps,
};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_interactive::Component;
use fandhe_frontend_wasm_full::headless_select::sync_select_value_text;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`headless_clipboard_browser.rs` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `crates/headless-ui/src/select.rs` の SSR 出力契約（root/content/item/
/// item-text/item-indicator）で、`items`（`(value, label)` 列）を持つ
/// Select 1 インスタンス分の `Node` を組み立てる。全 item は未選択状態
/// （`OpenState::Closed`）の SSR 初期表現で出力する。
fn build_select_node(items: &[(&str, &str)]) -> fandhe_frontend_core::Node {
    let props = SelectProps::default();
    root(
        OpenState::Closed,
        &props,
        Vec::new(),
        vec![
            // `sync_select_value_text` は `[data-part="value-text"]` が
            // root 配下に無いと fail-closed に no-op で早期 return する
            // 契約（モジュール doc 参照）のため、テスト対象の同期処理が
            // 実際に走ることを保証するために必須。
            value_text(true, &props, Vec::new(), Vec::new()),
            content(
                OpenState::Closed,
                None,
                None,
                None,
                Vec::new(),
                items
                    .iter()
                    .map(|(value, label)| {
                        item(
                            OpenState::Closed,
                            &props,
                            false,
                            false,
                            value,
                            None,
                            Vec::new(),
                            vec![
                                item_text(
                                    OpenState::Closed,
                                    &props,
                                    false,
                                    false,
                                    None,
                                    Vec::new(),
                                    vec![fandhe_frontend_core::text(*label)],
                                ),
                                item_indicator(OpenState::Closed, Vec::new(), Vec::new()),
                            ],
                        )
                    })
                    .collect(),
            ),
        ],
    )
}

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`headless_clipboard_browser.rs::create_container` と同型）。
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

fn item_text_el(item: &Element) -> Element {
    item.query_selector("[data-part=\"item-text\"]")
        .unwrap()
        .expect("item-text child must exist")
}

fn item_indicator_el(item: &Element) -> Element {
    item.query_selector("[data-part=\"item-indicator\"]")
        .unwrap()
        .expect("item-indicator child must exist")
}

/// 検証 1: 選択変更後、item-text/item-indicator の `data-state`/`hidden` が
/// 選択状態と一致する（codex-review P1 是正・Cursor Medium 指摘、イシュー
/// #1619）。従来は親 item の `aria-selected`/`data-selected` のみ更新され、
/// item-text/item-indicator は SSR 初期値 (`"closed"`/`hidden` 付与) の
/// まま取り残されていた。
#[wasm_bindgen_test]
fn sync_select_value_text_updates_item_text_and_indicator_data_state() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "hs-sync-basic");
    let _cleanup = RemoveOnDrop(container.clone());

    let node = build_select_node(&[("a", "Apple"), ("b", "Banana")]);
    container.set_inner_html(&render(&node));

    let root_el = container
        .query_selector("[data-scope=\"select\"][data-part=\"root\"]")
        .unwrap()
        .unwrap();
    let item_a = container
        .query_selector("[data-part=\"item\"][data-value=\"a\"]")
        .unwrap()
        .unwrap();
    let item_b = container
        .query_selector("[data-part=\"item\"][data-value=\"b\"]")
        .unwrap()
        .unwrap();

    // 初期状態（SSR 契約）: 両方 closed/hidden。
    assert_eq!(
        item_text_el(&item_a).get_attribute("data-state").as_deref(),
        Some("closed")
    );
    assert!(item_indicator_el(&item_a).has_attribute("hidden"));

    let mut select = Select::default();
    select.update(SelectAction::Select("a".to_string()));
    sync_select_value_text(&select, &root_el, "Select a fruit");

    assert_eq!(
        item_text_el(&item_a).get_attribute("data-state").as_deref(),
        Some("open"),
        "選択された item の item-text は data-state=open へ同期されるべき"
    );
    assert!(
        !item_indicator_el(&item_a).has_attribute("hidden"),
        "選択された item の item-indicator は hidden が外れるべき"
    );
    assert_eq!(
        item_text_el(&item_b).get_attribute("data-state").as_deref(),
        Some("closed"),
        "非選択 item の item-text は data-state=closed のままのはず"
    );
    assert!(
        item_indicator_el(&item_b).has_attribute("hidden"),
        "非選択 item の item-indicator は hidden のままのはず"
    );

    // 選択切り替え: 旧選択 item の状態が正しく巻き戻ること。
    select.update(SelectAction::Select("b".to_string()));
    sync_select_value_text(&select, &root_el, "Select a fruit");

    assert_eq!(
        item_text_el(&item_a).get_attribute("data-state").as_deref(),
        Some("closed"),
        "選択解除された旧 item の item-text は data-state=closed へ巻き戻るべき"
    );
    assert!(
        item_indicator_el(&item_a).has_attribute("hidden"),
        "選択解除された旧 item の item-indicator は hidden が復帰するべき"
    );
    assert_eq!(
        item_text_el(&item_b).get_attribute("data-state").as_deref(),
        Some("open")
    );
    assert!(!item_indicator_el(&item_b).has_attribute("hidden"));
}

/// 検証 2: `root` 配下にネストした別 Select インスタンス（同じ
/// `data-scope="select"`）が存在しても、外側インスタンスの同期は内側
/// インスタンスの item へ波及しない（codex-review P1 是正、イシュー
/// #1619。従来は `query_selector_all` が `root` 配下の全 Select item を
/// 無差別に収集していたため、ネストした別インスタンスの item まで
/// 書き換えてしまっていた）。
#[wasm_bindgen_test]
fn sync_select_value_text_does_not_leak_into_nested_select_instance() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "hs-sync-nested");
    let _cleanup = RemoveOnDrop(container.clone());

    // 外側 Select の item "a" の子として、内側 Select（同じ value "a"/"b"
    // を持つ）をまるごと埋め込む（実運用では起こらない構成だが、走査境界の
    // 防御を直接検証するための最小再現）。
    let props = SelectProps::default();
    let inner = build_select_node(&[("a", "Inner A"), ("b", "Inner B")]);
    let outer = root(
        OpenState::Closed,
        &props,
        Vec::new(),
        vec![
            value_text(true, &props, Vec::new(), Vec::new()),
            content(
                OpenState::Closed,
                None,
                None,
                None,
                Vec::new(),
                vec![
                    item(
                        OpenState::Closed,
                        &props,
                        false,
                        false,
                        "a",
                        None,
                        Vec::new(),
                        vec![
                            item_text(
                                OpenState::Closed,
                                &props,
                                false,
                                false,
                                None,
                                Vec::new(),
                                vec![fandhe_frontend_core::text("Outer A")],
                            ),
                            item_indicator(OpenState::Closed, Vec::new(), Vec::new()),
                            inner,
                        ],
                    ),
                    item(
                        OpenState::Closed,
                        &props,
                        false,
                        false,
                        "b",
                        None,
                        Vec::new(),
                        vec![
                            item_text(
                                OpenState::Closed,
                                &props,
                                false,
                                false,
                                None,
                                Vec::new(),
                                vec![fandhe_frontend_core::text("Outer B")],
                            ),
                            item_indicator(OpenState::Closed, Vec::new(), Vec::new()),
                        ],
                    ),
                ],
            ),
        ],
    );
    container.set_inner_html(&render(&outer));

    let outer_root = container
        .query_selector("[data-scope=\"select\"][data-part=\"root\"]")
        .unwrap()
        .unwrap();
    let outer_item_a = container
        .query_selector("[data-scope=\"select\"][data-part=\"item\"][data-value=\"a\"]")
        .unwrap()
        .unwrap();
    let inner_root = outer_item_a
        .query_selector("[data-scope=\"select\"][data-part=\"root\"]")
        .unwrap()
        .unwrap();
    let inner_item_a = inner_root
        .query_selector("[data-part=\"item\"][data-value=\"a\"]")
        .unwrap()
        .unwrap();

    let mut select = Select::default();
    select.update(SelectAction::Select("a".to_string()));
    sync_select_value_text(&select, &outer_root, "placeholder");

    // 外側 item "a" は選択反映される。
    assert_eq!(
        outer_item_a.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        item_text_el(&outer_item_a)
            .get_attribute("data-state")
            .as_deref(),
        Some("open")
    );

    // 内側インスタンスの item "a" は SSR 初期状態のまま（無関係な別
    // インスタンスへ波及していない）。
    assert_eq!(
        inner_item_a.get_attribute("aria-selected").as_deref(),
        Some("false"),
        "ネストした別 Select インスタンスの item は書き換えられないはず"
    );
    assert!(!inner_item_a.has_attribute("data-selected"));
    assert_eq!(
        item_text_el(&inner_item_a)
            .get_attribute("data-state")
            .as_deref(),
        Some("closed")
    );
    assert!(item_indicator_el(&inner_item_a).has_attribute("hidden"));
}

/// 検証 3: `root` 配下にネストした別 Select インスタンス（trigger あり）が
/// 存在し、外側インスタンス自身は trigger を省略する構成でも、外側の
/// `data-placeholder-shown` 同期が内側インスタンスの trigger へ波及しない
/// （codex-review P1 再指摘、イシュー #1619、PR #1899）。従来の
/// `root.query_selector(TRIGGER_SELECTOR)` は素の文書順探索のため、外側が
/// trigger を持たない場合に内側 Select の trigger を誤って掴んで書き換えて
/// しまっていた。
#[wasm_bindgen_test]
fn sync_select_value_text_does_not_leak_trigger_into_nested_select_instance() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "hs-sync-nested-trigger");
    let _cleanup = RemoveOnDrop(container.clone());

    let props = SelectProps::default();

    // 内側 Select は trigger を持つ（未選択 = プレースホルダー表示中の SSR
    // 初期状態）。外側 Select は trigger を持たず value-text のみで
    // 構成する（trigger 省略構成は headless-ui 契約上許容される）。
    let inner = root(
        OpenState::Closed,
        &props,
        Vec::new(),
        vec![
            trigger(
                OpenState::Closed,
                &props,
                true,
                None,
                None,
                Vec::new(),
                Vec::new(),
            ),
            value_text(true, &props, Vec::new(), Vec::new()),
            content(
                OpenState::Closed,
                None,
                None,
                None,
                Vec::new(),
                vec![item(
                    OpenState::Closed,
                    &props,
                    false,
                    false,
                    "a",
                    None,
                    Vec::new(),
                    vec![
                        item_text(
                            OpenState::Closed,
                            &props,
                            false,
                            false,
                            None,
                            Vec::new(),
                            vec![fandhe_frontend_core::text("Inner A")],
                        ),
                        item_indicator(OpenState::Closed, Vec::new(), Vec::new()),
                    ],
                )],
            ),
        ],
    );

    let outer = root(
        OpenState::Closed,
        &props,
        Vec::new(),
        vec![
            value_text(true, &props, Vec::new(), Vec::new()),
            content(
                OpenState::Closed,
                None,
                None,
                None,
                Vec::new(),
                vec![item(
                    OpenState::Closed,
                    &props,
                    false,
                    false,
                    "a",
                    None,
                    Vec::new(),
                    vec![
                        item_text(
                            OpenState::Closed,
                            &props,
                            false,
                            false,
                            None,
                            Vec::new(),
                            vec![fandhe_frontend_core::text("Outer A")],
                        ),
                        item_indicator(OpenState::Closed, Vec::new(), Vec::new()),
                        inner,
                    ],
                )],
            ),
        ],
    );
    container.set_inner_html(&render(&outer));

    let outer_root = container
        .query_selector("[data-scope=\"select\"][data-part=\"root\"]")
        .unwrap()
        .unwrap();
    let outer_item_a = container
        .query_selector("[data-scope=\"select\"][data-part=\"item\"][data-value=\"a\"]")
        .unwrap()
        .unwrap();
    let inner_root = outer_item_a
        .query_selector("[data-scope=\"select\"][data-part=\"root\"]")
        .unwrap()
        .unwrap();
    let inner_trigger = inner_root
        .query_selector("[data-scope=\"select\"][data-part=\"trigger\"]")
        .unwrap()
        .unwrap();

    // 内側 trigger は SSR 初期状態で data-placeholder-shown が付与されて
    // いること（未選択 = プレースホルダー表示中）。
    assert!(inner_trigger.has_attribute("data-placeholder-shown"));

    let mut select = Select::default();
    select.update(SelectAction::Select("a".to_string()));
    sync_select_value_text(&select, &outer_root, "placeholder");

    // 外側は trigger を持たないため同期は no-op（trigger 更新自体は
    // スキップ）だが、value-text の選択反映自体は行われる。
    assert_eq!(
        outer_item_a.get_attribute("aria-selected").as_deref(),
        Some("true")
    );

    // 内側 Select の trigger はネストした別インスタンスであり、外側の
    // 選択操作からは一切書き換えられないため、SSR 初期状態のまま
    // data-placeholder-shown が残るはず（バグ再現時はここで属性が誤って
    // 除去されていた）。
    assert!(
        inner_trigger.has_attribute("data-placeholder-shown"),
        "ネストした別 Select インスタンスの trigger は書き換えられないはず"
    );
}
