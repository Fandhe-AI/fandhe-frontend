//! `fandhe_frontend_wasm_full::chart_range`（イシュー #2134、親 #2132）の
//! 実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/chart_range.rs` の native `#[cfg(test)] mod tests`
//! は純粋ロジック層（[`parse_range_bound`]/[`resolve_range`]/
//! [`category_hidden_by_range`]/[`is_indexed_element_hidden`]）を検証
//!済みである。本ファイルはその先、配線層（`wiring`、
//! `#[cfg(target_arch = "wasm32")]`）が実 DOM（headless Chromium）上で
//! 凡例 trigger クリック・期間切替 item クリック・構造再描画後の
//! 再同期を正しく反映することを検証する。
//!
//! マークアップは `fandhe_frontend_core::el` で
//! `crates/pre-styled-ui/src/charts/legend.rs`（イシュー #2133）・
//! `crates/pre-styled-ui/src/charts/{bar_chart,tooltip}.rs`
//! （イシュー #2129 の `data-range`/hit-area/tooltip 契約）の SSR 出力
//! 契約を手組みする（`fandhe-frontend-wasm-full` は
//! `fandhe-frontend-pre-styled-ui` に依存しないため。`chart_tooltip_browser.rs`
//! と同型。実マークアップとのドリフトリスクは PR 本文に記録し、追跡
//! Issue の起票を提案する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, render};
use fandhe_frontend_wasm_full::chart_range::wire_chart_range_events;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`chart_tooltip_browser.rs::create_container` と同型）。
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
/// （`chart_tooltip_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.set_inner_html("");
        self.0.remove();
    }
}

/// 2 系列（`a`/`b`）× 3 カテゴリの bar chart 相当マークアップに、
/// 凡例（系列トグル）と期間切替 toggle-group を添えて `container_id`
/// 配下へ構築する。チャート root（`svg[data-part="root"]`）の id は
/// `"chart-root"`、`aria-controls` はその id を指す。
///
/// - hit-area 3 件（`data-index` 0/1/2、`data-series` なし）
/// - 描画要素 6 件（`data-series="a"|"b"` × `data-index` 0/1/2）
/// - tooltip 3 件（カテゴリ単位、`data-index` 0/1/2）、各 tooltip 内に
///   `tooltip-item` 2 件（`data-series="a"|"b"`）
/// - 凡例 trigger 2 件（`data-series="a"|"b"`、`aria-controls="chart-root"`）
/// - toggle-group item 2 件（`data-value="30d"|"90d"`、`aria-controls`
///   は toggle-group root 側、`data-range-from`/`data-range-to` は
///   `"30d"` のみ `from=1`/`to=3`（カテゴリ 0 を範囲外にする）を持つ）
fn full_chart_markup(container_id: &str) {
    /// カテゴリ `index`（`"0"`/`"1"`/`"2"`）× 系列 `name`（`"a"`/`"b"`）の
    /// bar 要素を 1 件組み立てる（`el` の `tag`/`attrs` はいずれも
    /// `&'static str` のみを取るため、動的に生成した文字列（`format!`）
    /// を渡せない。カテゴリ数がテスト全体で固定 3 のため、呼び出し側で
    /// リテラルの `index` 文字列を直接渡す）。
    fn bar(index: &'static str, name: &'static str) -> fandhe_frontend_core::Node {
        el(
            "rect",
            vec![
                ("data-scope", "bar-chart"),
                ("data-part", "bar"),
                ("data-index", index),
                ("data-series", name),
            ],
            vec![],
        )
    }
    fn hit_area(index: &'static str) -> fandhe_frontend_core::Node {
        el(
            "rect",
            vec![
                ("data-scope", "chart"),
                ("data-part", "hit-area"),
                ("data-index", index),
                ("fill", "none"),
                ("pointer-events", "none"),
                ("tabindex", "-1"),
            ],
            vec![],
        )
    }
    fn tooltip_item(name: &'static str) -> fandhe_frontend_core::Node {
        el(
            "div",
            vec![
                ("data-scope", "chart"),
                ("data-part", "tooltip-item"),
                ("data-series", name),
            ],
            vec![],
        )
    }
    fn tooltip(
        index: &'static str,
        items: Vec<fandhe_frontend_core::Node>,
    ) -> fandhe_frontend_core::Node {
        el(
            "div",
            vec![
                ("data-scope", "chart"),
                ("data-part", "tooltip"),
                ("data-index", index),
                ("hidden", ""),
            ],
            items,
        )
    }

    let chart_children = vec![
        bar("0", "a"),
        bar("0", "b"),
        bar("1", "a"),
        bar("1", "b"),
        bar("2", "a"),
        bar("2", "b"),
        hit_area("0"),
        hit_area("1"),
        hit_area("2"),
    ];
    let svg = el(
        "svg",
        vec![("data-part", "root"), ("id", "chart-root"), ("role", "img")],
        chart_children,
    );

    let tooltip_layer = el(
        "div",
        vec![("data-scope", "chart"), ("data-part", "tooltip-layer")],
        vec![
            tooltip("0", vec![tooltip_item("a"), tooltip_item("b")]),
            tooltip("1", vec![tooltip_item("a"), tooltip_item("b")]),
            tooltip("2", vec![tooltip_item("a"), tooltip_item("b")]),
        ],
    );

    let legend = el(
        "ul",
        vec![],
        vec![
            el(
                "li",
                vec![],
                vec![el(
                    "button",
                    vec![
                        ("data-scope", "chart-legend"),
                        ("data-part", "trigger"),
                        ("data-series", "a"),
                        ("aria-pressed", "true"),
                        ("aria-controls", "chart-root"),
                    ],
                    vec![],
                )],
            ),
            el(
                "li",
                vec![],
                vec![el(
                    "button",
                    vec![
                        ("data-scope", "chart-legend"),
                        ("data-part", "trigger"),
                        ("data-series", "b"),
                        ("aria-pressed", "true"),
                        ("aria-controls", "chart-root"),
                    ],
                    vec![],
                )],
            ),
        ],
    );

    let range_group = el(
        "div",
        vec![
            ("data-scope", "toggle-group"),
            ("data-part", "root"),
            ("aria-controls", "chart-root"),
        ],
        vec![
            el(
                "button",
                vec![
                    ("data-scope", "toggle-group"),
                    ("data-part", "item"),
                    ("data-value", "30d"),
                    ("data-range-from", "1"),
                    ("data-range-to", "3"),
                ],
                vec![],
            ),
            el(
                "button",
                vec![
                    ("data-scope", "toggle-group"),
                    ("data-part", "item"),
                    ("data-value", "90d"),
                ],
                vec![],
            ),
        ],
    );

    let html = render(&el(
        "div",
        vec![],
        vec![svg, tooltip_layer, legend, range_group],
    ));
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
        .get_element_by_id(container_id)
        .expect("container must exist")
        .set_inner_html(&html);
}

fn query(root: &Element, selector: &str) -> Option<Element> {
    root.query_selector(selector).ok().flatten()
}

fn query_all(root: &Element, selector: &str) -> Vec<Element> {
    let Ok(list) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    let len = list.length();
    (0..len)
        .filter_map(|i| list.get(i))
        .filter_map(|node| node.dyn_into::<Element>().ok())
        .collect()
}

/// bar/hit-area/tooltip-item を `data-index`/`data-series` の組で 1 件
/// 選ぶ（`series` が `None` のときは `data-series` を持たない hit-area
/// を選ぶ）。
fn indexed(root: &Element, part_selector: &str, index: usize, series: Option<&str>) -> Element {
    query_all(root, part_selector)
        .into_iter()
        .find(|element| {
            let index_matches =
                element.get_attribute("data-index").as_deref() == Some(index.to_string().as_str());
            let series_matches = element.get_attribute("data-series").as_deref() == series;
            index_matches && series_matches
        })
        .unwrap_or_else(|| panic!("element index={index} series={series:?} must exist"))
}

/// カテゴリ `index` の tooltip（`[data-scope="chart"][data-part="tooltip"]
/// [data-index="<index>"]`）配下で系列 `series` の `tooltip-item` を選ぶ。
/// `tooltip-item` 自体は `data-index` を持たない（`data-series` のみ、
/// `charts/tooltip.rs` の SSR 契約）ため [`indexed`] は使えず、まずカテゴリ
/// で tooltip を特定してから系列で子を絞る 2 段階の探索が必要。
fn tooltip_item_in(root: &Element, index: usize, series: &str) -> Element {
    let tooltip = query_all(root, "[data-scope=\"chart\"][data-part=\"tooltip\"]")
        .into_iter()
        .find(|element| {
            element.get_attribute("data-index").as_deref() == Some(index.to_string().as_str())
        })
        .unwrap_or_else(|| panic!("tooltip index={index} must exist"));
    query_all(
        &tooltip,
        "[data-scope=\"chart\"][data-part=\"tooltip-item\"]",
    )
    .into_iter()
    .find(|element| element.get_attribute("data-series").as_deref() == Some(series))
    .unwrap_or_else(|| panic!("tooltip-item index={index} series={series} must exist"))
}

/// 合成 `click`（bubbles: true）を生成して `target` へ発火する
/// （`headless_clipboard_browser.rs::synthetic_click` と同型）。
fn click(target: &Element) {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail");
    target
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
}

#[wasm_bindgen_test]
fn legend_trigger_click_flips_aria_pressed_and_hides_matching_series() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-1");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-1");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let trigger_a = query(
        &container,
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"][data-series=\"a\"]",
    )
    .expect("trigger a must exist");
    assert_eq!(
        trigger_a.get_attribute("aria-pressed").as_deref(),
        Some("true")
    );

    click(&trigger_a);

    assert_eq!(
        trigger_a.get_attribute("aria-pressed").as_deref(),
        Some("false"),
        "click must flip aria-pressed to false"
    );
    let bar_a0 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(
        bar_a0.has_attribute("data-hidden"),
        "series a bars must gain data-hidden once toggled off"
    );
    let bar_b0 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("b"),
    );
    assert!(
        !bar_b0.has_attribute("data-hidden"),
        "series b bars must stay visible (independent toggle)"
    );
    let tooltip_item_a = tooltip_item_in(&container, 0, "a");
    assert!(
        tooltip_item_a.has_attribute("data-hidden"),
        "tooltip-item for the hidden series must also gain data-hidden"
    );

    // 再クリックで元に戻る（冪等な同期、モジュール doc「同期」節）。
    click(&trigger_a);
    assert_eq!(
        trigger_a.get_attribute("aria-pressed").as_deref(),
        Some("true")
    );
    let bar_a0_after = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(!bar_a0_after.has_attribute("data-hidden"));
}

#[wasm_bindgen_test]
fn range_item_click_writes_data_range_and_hides_out_of_range_categories() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-2");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-2");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let chart_root = document
        .get_element_by_id("chart-root")
        .expect("chart root must exist");
    assert!(chart_root.get_attribute("data-range").is_none());

    let item_30d = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"30d\"]",
    )
    .expect("30d item must exist");
    click(&item_30d);

    assert_eq!(
        chart_root.get_attribute("data-range").as_deref(),
        Some("30d"),
        "click must write the item's data-value onto the chart root's data-range"
    );

    // category_hidden_by_range: from=1,to=3 のためカテゴリ 0 のみ範囲外。
    let hit0 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        0,
        None,
    );
    assert!(hit0.has_attribute("data-hidden"));
    assert_eq!(hit0.get_attribute("display").as_deref(), Some("none"));
    let hit1 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        1,
        None,
    );
    assert!(!hit1.has_attribute("data-hidden"));
    assert!(hit1.get_attribute("display").is_none());

    let bar_a0 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(bar_a0.has_attribute("data-hidden"));
    let bar_a1 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        1,
        Some("a"),
    );
    assert!(!bar_a1.has_attribute("data-hidden"));

    // 範囲外カテゴリの tooltip 本体は多層防御として hidden を強制する。
    let tooltip0 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"tooltip\"]",
        0,
        None,
    );
    assert!(tooltip0.has_attribute("hidden"));

    // 90d（範囲宣言なし item）へ戻すと全カテゴリが可視へ戻る。
    let item_90d = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"90d\"]",
    )
    .expect("90d item must exist");
    click(&item_90d);
    assert_eq!(
        chart_root.get_attribute("data-range").as_deref(),
        Some("90d")
    );
    let hit0_after = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        0,
        None,
    );
    assert!(!hit0_after.has_attribute("data-hidden"));
}

#[wasm_bindgen_test]
fn roving_tabindex_follows_visible_hit_areas_after_range_change() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-3");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-3");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    // 初期同期（マウント直後の `sync_all`）で先頭の可視 hit-area が
    // roving tabindex の起点になる。
    let hit0 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        0,
        None,
    );
    assert_eq!(hit0.get_attribute("tabindex").as_deref(), Some("0"));

    let item_30d = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"30d\"]",
    )
    .expect("30d item must exist");
    click(&item_30d);

    // カテゴリ 0 が範囲外になった後は、可視集合の先頭（カテゴリ 1）が
    // roving tabindex を持つ。
    let hit0_after = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        0,
        None,
    );
    assert_eq!(hit0_after.get_attribute("tabindex").as_deref(), Some("-1"));
    let hit1_after = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        1,
        None,
    );
    assert_eq!(hit1_after.get_attribute("tabindex").as_deref(), Some("0"));
}

#[wasm_bindgen_test]
fn unresolved_aria_controls_is_a_no_op() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-4");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-4");
    // アプリ側が誤って未解決の id を指した凡例 trigger を追加する。
    let orphan = el(
        "button",
        vec![
            ("data-scope", "chart-legend"),
            ("data-part", "trigger"),
            ("data-series", "z"),
            ("aria-pressed", "true"),
            ("aria-controls", "does-not-exist"),
        ],
        vec![],
    );
    let html = render(&orphan);
    container
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let orphan_trigger = query(
        &container,
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"][data-series=\"z\"]",
    )
    .expect("orphan trigger must exist");
    click(&orphan_trigger);

    // aria-pressed 自体は反転するが（`sync_chart` に到達しないだけ）、
    // 他チャートの状態には一切影響しない（`resolve_chart_root` が
    // `root.contains` を必須にしている fail-closed 経路）。
    assert_eq!(
        orphan_trigger.get_attribute("aria-pressed").as_deref(),
        Some("false")
    );
    let bar_a0 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(!bar_a0.has_attribute("data-hidden"));
}

#[wasm_bindgen_test]
async fn structural_rerender_reapplies_sync() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-5");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-5");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let trigger_a = query(
        &container,
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"][data-series=\"a\"]",
    )
    .expect("trigger a must exist");
    click(&trigger_a);
    let bar_a0 = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(bar_a0.has_attribute("data-hidden"));

    // `Runtime::rerender_subtree` 相当の構造フォールバック（root 配下の
    // 丸ごと再構築）を模す。凡例 trigger の `aria-pressed="false"` は
    // アプリが保持している状態としてそのまま書き戻される想定。
    full_chart_markup("chart-range-test-5");
    let trigger_a_after_rebuild = query(
        &container,
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"][data-series=\"a\"]",
    )
    .expect("trigger a must exist after rebuild");
    trigger_a_after_rebuild
        .set_attribute("aria-pressed", "false")
        .unwrap();
    let bar_a0_rebuilt = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(
        !bar_a0_rebuilt.has_attribute("data-hidden"),
        "rebuilt subtree starts from fresh SSR markup without data-hidden"
    );

    // 次のマイクロタスクまで待って MutationObserver のコールバックが
    // 走るのを待つ（`chart_tooltip_browser.rs::await_next_tick` と同型）。
    await_next_tick().await;

    let bar_a0_resynced = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(
        bar_a0_resynced.has_attribute("data-hidden"),
        "MutationObserver must resync data-hidden from the rebuilt aria-pressed state"
    );
}

/// 次のマイクロタスク（`Promise::resolve` 経由）まで待つ。
/// `MutationObserver` のコールバックはマイクロタスクキューで実行される
/// ため、同期的な `dispatch_event` 直後には観測できない
/// （`chart_tooltip_browser.rs::await_next_tick` と同型）。
async fn await_next_tick() {
    let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("Promise::resolve must not reject");
}
