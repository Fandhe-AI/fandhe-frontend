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
use fandhe_frontend_wasm_full::chart::wire_chart_events;
use fandhe_frontend_wasm_full::chart_range::wire_chart_range_events;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, KeyboardEvent, KeyboardEventInit, MouseEvent, MouseEventInit};

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

/// `keydown`（`bubbles: true, cancelable: true`）を合成する
/// （`chart_tooltip_browser.rs::keydown_event` と同型）。
fn keydown_event(key: &str) -> KeyboardEvent {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
}

/// line-chart 相当の `series-line`（`data-series` のみ・`data-index` を
/// 持たない）1 本 + 凡例 trigger 1 件を `container_id` 配下へ構築する
/// （イシュー #2134 codex-review 指摘: `[data-index]` のみを対象にして
/// いた `sync_chart` がこれらの要素を取りこぼしていた回帰の検証用）。
fn series_only_markup(container_id: &str) {
    let series_line = el(
        "path",
        vec![
            ("data-scope", "line-chart"),
            ("data-part", "series-line"),
            ("data-series", "a"),
            ("d", "M0,0L1,1"),
        ],
        vec![],
    );
    let svg = el(
        "svg",
        vec![("data-part", "root"), ("id", "series-only-chart-root")],
        vec![series_line],
    );
    let legend = el(
        "button",
        vec![
            ("data-scope", "chart-legend"),
            ("data-part", "trigger"),
            ("data-series", "a"),
            ("aria-pressed", "true"),
            ("aria-controls", "series-only-chart-root"),
        ],
        vec![],
    );
    let html = render(&el("div", vec![], vec![svg, legend]));
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
        .get_element_by_id(container_id)
        .expect("container must exist")
        .set_inner_html(&html);
}

#[wasm_bindgen_test]
fn series_only_marks_without_data_index_sync_with_legend_toggle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-series-only");
    let _guard = RemoveOnDrop(container.clone());
    series_only_markup("chart-range-test-series-only");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let series_line = query(
        &container,
        "[data-scope=\"line-chart\"][data-part=\"series-line\"]",
    )
    .expect("series-line must exist");
    assert!(
        !series_line.has_attribute("data-hidden"),
        "series-line starts visible (aria-pressed=true)"
    );

    let trigger = query(
        &container,
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"]",
    )
    .expect("trigger must exist");
    click(&trigger);

    assert!(
        series_line.has_attribute("data-hidden"),
        "series-only marks (data-series only, no data-index) must sync with legend toggle"
    );

    click(&trigger);
    assert!(
        !series_line.has_attribute("data-hidden"),
        "re-click must restore visibility (idempotent sync)"
    );
}

/// bar 1 件（`data-index="0" data-series="b"`）が SSR 時点で既に
/// `data-hidden`（`BarChartProps::hidden_series` 相当）を持ち、`toggle-group`
/// による期間切替コントロールのみが配線され、凡例 trigger は一切存在
/// しない構成を `container_id` 配下へ構築する（イシュー #2134
/// codex-review 指摘: 凡例なしで期間コントロールだけを配線すると、
/// マウント直後の `sync_all` が SSR の非表示設定を「凡例なし＝全件表示」
/// へ誤って解除してしまっていた回帰の検証用）。
fn declared_hidden_without_legend_markup(container_id: &str) {
    fn bar(series: &'static str, hidden: bool) -> fandhe_frontend_core::Node {
        let mut attrs = vec![
            ("data-scope", "bar-chart"),
            ("data-part", "bar"),
            ("data-index", "0"),
            ("data-series", series),
        ];
        if hidden {
            attrs.push(("data-hidden", ""));
        }
        el("rect", attrs, vec![])
    }
    let svg = el(
        "svg",
        vec![("data-part", "root"), ("id", "declared-hidden-chart-root")],
        vec![bar("a", false), bar("b", true)],
    );
    let range_group = el(
        "div",
        vec![
            ("data-scope", "toggle-group"),
            ("data-part", "root"),
            ("aria-controls", "declared-hidden-chart-root"),
        ],
        vec![el(
            "button",
            vec![
                ("data-scope", "toggle-group"),
                ("data-part", "item"),
                ("data-value", "90d"),
            ],
            vec![],
        )],
    );
    let html = render(&el("div", vec![], vec![svg, range_group]));
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
        .get_element_by_id(container_id)
        .expect("container must exist")
        .set_inner_html(&html);
}

#[wasm_bindgen_test]
fn mount_time_sync_preserves_ssr_declared_hidden_when_no_legend_governs_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-declared");
    let _guard = RemoveOnDrop(container.clone());
    declared_hidden_without_legend_markup("chart-range-test-declared");

    // マウント直後の `sync_all`（`wire_chart_range_events` 内部）が
    // 走った後の状態を検証する。
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let bar_a = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("a"),
    );
    assert!(
        !bar_a.has_attribute("data-hidden"),
        "series a stays visible (SSR declared it visible, no legend involved)"
    );
    let bar_b = indexed(
        &container,
        "[data-scope=\"bar-chart\"][data-part=\"bar\"]",
        0,
        Some("b"),
    );
    assert!(
        bar_b.has_attribute("data-hidden"),
        "series b's SSR-declared hidden_series state must survive mount-time sync \
         even though no legend governs it (period control alone must not clear it)"
    );
}

#[wasm_bindgen_test]
fn disabled_range_item_click_is_rejected() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-disabled-item");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-disabled-item");

    // 30d item を無効化する（`toggle_group::item`/`select::item` の
    // `data-disabled` 契約と同じ語彙）。
    let item_30d = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"30d\"]",
    )
    .expect("30d item must exist");
    item_30d
        .set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");

    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let chart_root = document
        .get_element_by_id("chart-root")
        .expect("chart root must exist");
    assert!(chart_root.get_attribute("data-range").is_none());

    click(&item_30d);

    assert!(
        chart_root.get_attribute("data-range").is_none(),
        "clicking a data-disabled range item must not write data-range \
         (headless dispatch's data-disabled contract, イシュー #2134 codex-review 指摘)"
    );
}

/// hit-area 3 件（`data-index` 0/1/2）+ 期間切替 toggle-group（`data-value`
/// `"invalid"` の item が `data-range-from="not-a-number"`
/// `data-range-to="2"` を持つ）を `container_id` 配下へ構築する（イシュー
/// #2134 codex-review 指摘: 存在する境界属性がパース不能な場合の
/// fail-closed 挙動の検証用。属性欠落〔既定値適用〕とは区別する）。
fn invalid_range_bound_markup(container_id: &str) {
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
    let svg = el(
        "svg",
        vec![("data-part", "root"), ("id", "invalid-range-chart-root")],
        vec![hit_area("0"), hit_area("1"), hit_area("2")],
    );
    let range_group = el(
        "div",
        vec![
            ("data-scope", "toggle-group"),
            ("data-part", "root"),
            ("aria-controls", "invalid-range-chart-root"),
        ],
        vec![el(
            "button",
            vec![
                ("data-scope", "toggle-group"),
                ("data-part", "item"),
                ("data-value", "invalid"),
                ("data-range-from", "not-a-number"),
                ("data-range-to", "2"),
            ],
            vec![],
        )],
    );
    let html = render(&el("div", vec![], vec![svg, range_group]));
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
        .get_element_by_id(container_id)
        .expect("container must exist")
        .set_inner_html(&html);
}

#[wasm_bindgen_test]
fn unparseable_range_bound_disables_range_hiding_instead_of_falling_back_to_defaults() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-invalid-bound");
    let _guard = RemoveOnDrop(container.clone());
    invalid_range_bound_markup("chart-range-test-invalid-bound");
    wire_chart_range_events(container.clone()).expect("wiring must not fail");

    let item = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"invalid\"]",
    )
    .expect("item must exist");
    click(&item);

    // `data-range-from="not-a-number"` はパース不能: `data-range-to="2"`
    // が有効でも、属性欠落時の既定値（`from=0`）へフォールバックせず
    // 範囲全体を無効化しなければならない（fail-closed、カテゴリを一切
    // 隠さない）。
    for index in 0..3 {
        let hit = indexed(
            &container,
            "[data-scope=\"chart\"][data-part=\"hit-area\"]",
            index,
            None,
        );
        assert!(
            !hit.has_attribute("data-hidden"),
            "category {index} must stay visible when a present range bound fails to parse"
        );
    }
}

#[wasm_bindgen_test]
fn keyboard_home_skips_hidden_hit_areas_after_range_change() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-range-test-keyboard-nav");
    let _guard = RemoveOnDrop(container.clone());
    full_chart_markup("chart-range-test-keyboard-nav");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");
    wire_chart_range_events(container.clone()).expect("wire_chart_range_events must succeed");

    let item_30d = query(
        &container,
        "[data-scope=\"toggle-group\"][data-part=\"item\"][data-value=\"30d\"]",
    )
    .expect("30d item must exist");
    click(&item_30d);

    // カテゴリ 0 が範囲外（`display: none`/`data-hidden`）になった後、
    // 可視集合はカテゴリ 1/2 のみ。
    let hit0 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        0,
        None,
    );
    let hit1 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        1,
        None,
    );
    let hit2 = indexed(
        &container,
        "[data-scope=\"chart\"][data-part=\"hit-area\"]",
        2,
        None,
    );
    assert!(hit0.has_attribute("data-hidden"));

    // カテゴリ 2（hit2）から "Home" を押す。非表示 hit-area を除外して
    // いなければ全 hit-area 中の先頭（カテゴリ 0、非表示）へ移動して
    // しまう（イシュー #2134 codex-review 指摘）。修正後は可視集合の
    // 先頭（カテゴリ 1）へ移動する。
    let event = keydown_event("Home");
    hit2.dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");

    assert!(
        event.default_prevented(),
        "Home must move focus within the visible set and prevent default"
    );
    assert_eq!(hit1.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(hit2.get_attribute("tabindex").as_deref(), Some("-1"));
    assert_eq!(
        hit0.get_attribute("tabindex").as_deref(),
        Some("-1"),
        "the hidden hit-area must never become the roving tabindex entry point"
    );
}
