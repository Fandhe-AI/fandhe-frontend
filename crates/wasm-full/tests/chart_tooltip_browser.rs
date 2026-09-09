//! `fandhe_frontend_wasm_full::chart`（イシュー #2130）の実ブラウザ回帰
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/chart.rs` の native `#[cfg(test)] mod tests` は
//! 純粋ロジック層（[`hit_area_next_index`]/[`is_sticky_pointer`]/
//! [`anchor_relative`]/[`hit_area_anchor`]/[`matches_key`]）を検証済み
//! である。本ファイルはその先、配線層（`wiring`、
//! `#[cfg(target_arch = "wasm32")]`）が実 DOM（headless Chromium）上で
//! hover/タッチ/キーボード操作に応じて `hidden`/`data-active`/
//! `--fandhe-chart-tooltip-x/y` を正しく切り替えることを検証する。
//!
//! マークアップは `fandhe_frontend_core::el` で
//! `crates/pre-styled-ui/src/charts/tooltip.rs` の SSR 出力契約（イシュー
//! #2129）を手組みする（`fandhe-frontend-wasm-full` は
//! `fandhe-frontend-pre-styled-ui` に依存しないため。実マークアップとの
//! ドリフトリスクは PR 本文に記録し、追跡 Issue の起票を提案する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, render};
use fandhe_frontend_wasm_full::chart::wire_chart_events;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, EventTarget, FocusEvent, FocusEventInit, KeyboardEvent, KeyboardEventInit,
    PointerEvent, PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`sidebar_browser.rs::create_container` と同型）。
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
/// （`sidebar_browser.rs::RemoveOnDrop` と同型。本モジュールの document
/// pointerdown リスナーも `Closure::forget` されるため、後続テストへの
/// 汚染を子要素除去で断つ）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.set_inner_html("");
        self.0.remove();
    }
}

/// 2 カテゴリ × 1 系列（bar 相当。帯 hit-area は `data-series` を持たない）
/// のマークアップを組み立てる。カテゴリ 0 の bar には静的 `data-active`
/// を付け、SSR の `active_index` 相当の既定強調を再現する。
fn bar_like_markup(container_id: &str) {
    let bar0 = el(
        "rect",
        vec![
            ("data-scope", "bar-chart"),
            ("data-part", "bar"),
            ("data-index", "0"),
            ("data-series", "visits"),
            ("data-active", ""),
        ],
        vec![],
    );
    let bar1 = el(
        "rect",
        vec![
            ("data-scope", "bar-chart"),
            ("data-part", "bar"),
            ("data-index", "1"),
            ("data-series", "visits"),
        ],
        vec![],
    );
    let hit0 = el(
        "rect",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "0"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let hit1 = el(
        "rect",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "1"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let axis_label = el("text", vec![("data-part", "axis-label")], vec![]);
    let svg = el(
        "svg",
        vec![("role", "img")],
        vec![bar0, bar1, axis_label, hit0, hit1],
    );
    let tooltip0 = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip"),
            ("data-index", "0"),
            ("hidden", ""),
        ],
        vec![],
    );
    let tooltip1 = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip"),
            ("data-index", "1"),
            ("hidden", ""),
        ],
        vec![],
    );
    let layer = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip-layer"),
            ("aria-hidden", "true"),
        ],
        vec![tooltip0, tooltip1],
    );
    let frame = el(
        "div",
        vec![("data-scope", "chart"), ("data-part", "frame")],
        vec![svg, layer],
    );
    let html = render(&frame);
    let container_selector = format!("#{container_id}");
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(&container_selector).ok().flatten())
        .expect("container must exist")
        .set_inner_html(&html);
}

/// scatter 相当（2 系列 × 同一 `data-index`）のマークアップ。
fn scatter_like_markup(container_id: &str) {
    let point_a = el(
        "circle",
        vec![
            ("data-scope", "scatter-chart"),
            ("data-part", "point"),
            ("data-index", "0"),
            ("data-series", "a"),
        ],
        vec![],
    );
    let point_b = el(
        "circle",
        vec![
            ("data-scope", "scatter-chart"),
            ("data-part", "point"),
            ("data-index", "0"),
            ("data-series", "b"),
        ],
        vec![],
    );
    let hit_a = el(
        "circle",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "0"),
            ("data-series", "a"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let hit_b = el(
        "circle",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "0"),
            ("data-series", "b"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let svg = el(
        "svg",
        vec![("role", "img")],
        vec![point_a, point_b, hit_a, hit_b],
    );
    let tooltip_a = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip"),
            ("data-index", "0"),
            ("data-series", "a"),
            ("hidden", ""),
        ],
        vec![],
    );
    let tooltip_b = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip"),
            ("data-index", "0"),
            ("data-series", "b"),
            ("hidden", ""),
        ],
        vec![],
    );
    let layer = el(
        "div",
        vec![
            ("data-scope", "chart"),
            ("data-part", "tooltip-layer"),
            ("aria-hidden", "true"),
        ],
        vec![tooltip_a, tooltip_b],
    );
    let frame = el(
        "div",
        vec![("data-scope", "chart"), ("data-part", "frame")],
        vec![svg, layer],
    );
    let html = render(&frame);
    let container_selector = format!("#{container_id}");
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector(&container_selector).ok().flatten())
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

fn pointer_event(kind: &str, pointer_type: &str, client_x: f64, client_y: f64) -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_type(pointer_type);
    init.set_client_x(client_x as i32);
    init.set_client_y(client_y as i32);
    PointerEvent::new_with_event_init_dict(kind, &init).expect("PointerEvent::new must not fail")
}

fn dispatch_pointer(target: &EventTarget, kind: &str, pointer_type: &str, x: f64, y: f64) {
    target
        .dispatch_event(pointer_event(kind, pointer_type, x, y).as_ref())
        .expect("dispatch_event must not fail");
}

fn keydown_event(key: &str) -> KeyboardEvent {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
}

fn focus_event(kind: &str, related: Option<&Element>) -> FocusEvent {
    let init = FocusEventInit::new();
    init.set_bubbles(true);
    if let Some(target) = related {
        init.set_related_target(Some(target.unchecked_ref::<EventTarget>()));
    }
    FocusEvent::new_with_focus_event_init_dict(kind, &init).expect("FocusEvent::new must not fail")
}

#[wasm_bindgen_test]
fn wire_enhances_hit_areas_and_sets_roving_tabindex() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-enhance");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-enhance");

    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit_areas = query_all(&container, "[data-scope=\"chart\"][data-part=\"hit-area\"]");
    assert_eq!(hit_areas.len(), 2);
    for hit_area in &hit_areas {
        assert_eq!(
            hit_area.get_attribute("pointer-events").as_deref(),
            Some("all")
        );
    }
    assert_eq!(hit_areas[0].get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(
        hit_areas[1].get_attribute("tabindex").as_deref(),
        Some("-1")
    );
}

#[wasm_bindgen_test]
fn hover_shows_matching_tooltip_and_toggles_data_active() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-hover");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-hover");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let hit1 = query(&container, "[data-part=\"hit-area\"][data-index=\"1\"]").expect("hit-area 1");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");
    let tooltip1 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"1\"]").expect("tooltip 1");
    let bar0 = query(&container, "[data-scope=\"bar-chart\"][data-index=\"0\"]").expect("bar 0");
    let bar1 = query(&container, "[data-scope=\"bar-chart\"][data-index=\"1\"]").expect("bar 1");

    dispatch_pointer(hit0.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(!tooltip0.has_attribute("hidden"));
    assert!(tooltip1.has_attribute("hidden"));
    assert!(hit0.has_attribute("data-active"));
    assert!(bar0.has_attribute("data-active"));
    assert!(!hit1.has_attribute("data-active"));
    assert!(!bar1.has_attribute("data-active"));

    dispatch_pointer(hit1.unchecked_ref(), "pointermove", "mouse", 20.0, 5.0);
    assert!(tooltip0.has_attribute("hidden"));
    assert!(!tooltip1.has_attribute("hidden"));
    assert!(!hit0.has_attribute("data-active"));
    assert!(hit1.has_attribute("data-active"));
    assert!(bar1.has_attribute("data-active"));
    assert!(!bar0.has_attribute("data-active"));
}

#[wasm_bindgen_test]
fn pointerout_outside_svg_closes_and_restores_initial_active() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-pointerout");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-pointerout");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let svg = query(&container, "svg").expect("svg");
    let hit1 = query(&container, "[data-part=\"hit-area\"][data-index=\"1\"]").expect("hit-area 1");
    let bar0 = query(&container, "[data-scope=\"bar-chart\"][data-index=\"0\"]").expect("bar 0");
    let bar1 = query(&container, "[data-scope=\"bar-chart\"][data-index=\"1\"]").expect("bar 1");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");

    dispatch_pointer(hit1.unchecked_ref(), "pointermove", "mouse", 20.0, 5.0);
    assert!(bar1.has_attribute("data-active"));

    // `related_target` を container（svg 外）にした pointerout で閉じる。
    let init = web_sys::MouseEventInit::new();
    init.set_bubbles(true);
    init.set_related_target(Some(container.unchecked_ref::<EventTarget>()));
    let leave = web_sys::MouseEvent::new_with_mouse_event_init_dict("pointerout", &init)
        .expect("MouseEvent::new must not fail");
    svg.dispatch_event(leave.as_ref())
        .expect("dispatch_event must not fail");

    // SSR 既定（bar0 のみ data-active、全 tooltip hidden）へ復元される。
    assert!(bar0.has_attribute("data-active"));
    assert!(!bar1.has_attribute("data-active"));
    assert!(tooltip0.has_attribute("hidden"));
}

#[wasm_bindgen_test]
fn pointermove_to_non_hit_area_closes_session() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-axis");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-axis");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let axis_label = query(&container, "[data-part=\"axis-label\"]").expect("axis label");
    let bar1 = query(&container, "[data-scope=\"bar-chart\"][data-index=\"1\"]").expect("bar 1");

    dispatch_pointer(hit0.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(hit0.has_attribute("data-active"));

    dispatch_pointer(axis_label.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(!hit0.has_attribute("data-active"));
    assert!(!bar1.has_attribute("data-active"));
}

#[wasm_bindgen_test]
fn keyboard_arrow_moves_focus_and_tooltip_and_is_non_circular() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-keyboard");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-keyboard");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let hit1 = query(&container, "[data-part=\"hit-area\"][data-index=\"1\"]").expect("hit-area 1");
    let tooltip1 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"1\"]").expect("tooltip 1");

    hit0.dispatch_event(focus_event("focusin", None).as_ref())
        .expect("dispatch_event must not fail");
    assert!(hit0.has_attribute("data-active"));

    let event = keydown_event("ArrowRight");
    hit0.dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
    assert!(event.default_prevented());
    assert!(!tooltip1.has_attribute("hidden"));
    assert!(hit1.has_attribute("data-active"));
    assert_eq!(hit1.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(hit0.get_attribute("tabindex").as_deref(), Some("-1"));

    // 末尾での ArrowRight は no-op（非循環）かつ prevent_default されない。
    let boundary_event = keydown_event("ArrowRight");
    hit1.dispatch_event(boundary_event.as_ref())
        .expect("dispatch_event must not fail");
    assert!(!boundary_event.default_prevented());

    // Escape でツールチップを閉じる（フォーカスは維持）。
    let escape = keydown_event("Escape");
    hit1.dispatch_event(escape.as_ref())
        .expect("dispatch_event must not fail");
    assert!(tooltip1.has_attribute("hidden"));
}

#[wasm_bindgen_test]
fn focusout_to_unrelated_element_closes_session() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-focusout");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-focusout");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");

    hit0.dispatch_event(focus_event("focusin", None).as_ref())
        .expect("dispatch_event must not fail");
    assert!(!tooltip0.has_attribute("hidden"));

    hit0.dispatch_event(focus_event("focusout", Some(&container)).as_ref())
        .expect("dispatch_event must not fail");
    assert!(tooltip0.has_attribute("hidden"));
}

#[wasm_bindgen_test]
fn touch_pointerdown_opens_sticky_session_and_outside_tap_closes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-touch");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-touch");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");
    let frame = query(&container, "[data-part=\"frame\"]").expect("frame");

    dispatch_pointer(hit0.unchecked_ref(), "pointerdown", "touch", 5.0, 5.0);
    assert!(!tooltip0.has_attribute("hidden"));

    // sticky セッション中の pointerout は無視される（frame 内側）。
    let init = web_sys::MouseEventInit::new();
    init.set_bubbles(true);
    let leave = web_sys::MouseEvent::new_with_mouse_event_init_dict("pointerout", &init)
        .expect("MouseEvent::new must not fail");
    hit0.dispatch_event(leave.as_ref())
        .expect("dispatch_event must not fail");
    assert!(!tooltip0.has_attribute("hidden"));

    // frame 外側（body）への pointerdown が document までバブリングし、
    // sticky セッションを閉じる。
    let outside: Element = document
        .body()
        .expect("document body must exist")
        .dyn_into()
        .expect("body is an Element");
    dispatch_pointer(outside.unchecked_ref(), "pointerdown", "touch", 0.0, 0.0);
    assert!(tooltip0.has_attribute("hidden"));
    let _ = frame;
}

#[wasm_bindgen_test]
fn touch_sticky_session_ignores_focusout_and_only_outside_tap_closes() {
    // handle_focusout の sticky ガード回帰テスト。タッチ由来の sticky
    // セッション中にフォーカス不可能な要素（軸ラベル相当・related_target
    // なし）へ focusout が発火しても、ドキュメント契約
    // （「タッチ由来で開始したセッションはチャート外タップまで開いた
    // ままにする」）どおり閉じないことを検証する。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-touch-focusout");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-touch-focusout");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");

    dispatch_pointer(hit0.unchecked_ref(), "pointerdown", "touch", 5.0, 5.0);
    assert!(!tooltip0.has_attribute("hidden"));

    // sticky セッション中の focusout（related_target なし）は無視される。
    hit0.dispatch_event(focus_event("focusout", None).as_ref())
        .expect("dispatch_event must not fail");
    assert!(!tooltip0.has_attribute("hidden"));

    // frame 外側（body）への pointerdown のみが sticky セッションを閉じる。
    let outside: Element = document
        .body()
        .expect("document body must exist")
        .dyn_into()
        .expect("body is an Element");
    dispatch_pointer(outside.unchecked_ref(), "pointerdown", "touch", 0.0, 0.0);
    assert!(tooltip0.has_attribute("hidden"));
}

#[wasm_bindgen_test]
fn scatter_series_matching_shows_only_matching_series() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-scatter");
    let _guard = RemoveOnDrop(container.clone());
    scatter_like_markup("chart-test-scatter");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit_a =
        query(&container, "[data-part=\"hit-area\"][data-series=\"a\"]").expect("hit-area a");
    let tooltip_a =
        query(&container, "[data-part=\"tooltip\"][data-series=\"a\"]").expect("tooltip a");
    let tooltip_b =
        query(&container, "[data-part=\"tooltip\"][data-series=\"b\"]").expect("tooltip b");
    let point_a = query(
        &container,
        "[data-scope=\"scatter-chart\"][data-series=\"a\"]",
    )
    .expect("point a");
    let point_b = query(
        &container,
        "[data-scope=\"scatter-chart\"][data-series=\"b\"]",
    )
    .expect("point b");

    dispatch_pointer(hit_a.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(!tooltip_a.has_attribute("hidden"));
    assert!(tooltip_b.has_attribute("hidden"));
    assert!(point_a.has_attribute("data-active"));
    assert!(!point_b.has_attribute("data-active"));
}

#[wasm_bindgen_test]
fn layer_missing_is_a_no_op() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-no-layer");
    let _guard = RemoveOnDrop(container.clone());

    let hit0 = el(
        "rect",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "0"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let svg = el("svg", vec![("role", "img")], vec![hit0]);
    let html = render(&svg);
    container.set_inner_html(&html);

    wire_chart_events(container.clone())
        .expect("wire_chart_events must succeed even without a layer");
    let hit_area = query(&container, "[data-part=\"hit-area\"]").expect("hit-area");
    // enhance は layer の有無に関係なく pointer-events を切り替える。
    assert_eq!(
        hit_area.get_attribute("pointer-events").as_deref(),
        Some("all")
    );
    // pointermove は panic せず、data-active も一切変化しない
    // （layer 欠落は fail-closed no-op、モジュール doc「ロケータ契約」節）。
    dispatch_pointer(hit_area.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(!hit_area.has_attribute("data-active"));
}

#[wasm_bindgen_test]
async fn structural_rerender_reapplies_enhance() {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-rerender");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-rerender");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let svg = query(&container, "svg").expect("svg");
    // SSR 相当（`pointer-events="none" tabindex="-1"`）へ丸ごと差し替える
    // 構造フォールバック再描画を模擬する。
    let hit_reset = el(
        "rect",
        vec![
            ("data-scope", "chart"),
            ("data-part", "hit-area"),
            ("data-index", "0"),
            ("fill", "none"),
            ("pointer-events", "none"),
            ("tabindex", "-1"),
        ],
        vec![],
    );
    let svg_html = render(&hit_reset);
    svg.set_inner_html(&svg_html);

    // MutationObserver のマイクロタスクキュー完了を待つ。
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let closure = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        });
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(closure.unchecked_ref(), 0)
            .expect("setTimeout must not fail");
    });
    JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");

    let hit_area = query(&container, "[data-part=\"hit-area\"]").expect("hit-area");
    assert_eq!(
        hit_area.get_attribute("pointer-events").as_deref(),
        Some("all")
    );
    assert_eq!(hit_area.get_attribute("tabindex").as_deref(), Some("0"));
}

/// `MutationObserver` のマイクロタスクキュー完了を待つ共通ヘルパー
/// （`structural_rerender_reapplies_enhance` の待機コードを抽出、以下 3 件
/// の回帰テストと共有する）。
async fn await_next_tick() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let closure = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        });
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(closure.unchecked_ref(), 0)
            .expect("setTimeout must not fail");
    });
    JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");
}

#[wasm_bindgen_test]
async fn wire_chart_events_without_initial_charts_still_wires_later_inserted_chart() {
    // codex レビュー指摘（イシュー #2130 PR #2267）の回帰テスト: 初期表示に
    // チャート（hit-area）が 1 つも無い状態で `wire_chart_events` を呼んだ
    // 場合でも、`MutationObserver` は常に登録され、後から
    // `rerender_subtree` 相当でチャートが追加されたときに配線されることを
    // 検証する（旧実装は hit-area が空なら早期リターンし、リスナー自体を
    // 一切登録していなかった）。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-late-mount");
    let _guard = RemoveOnDrop(container.clone());
    // マウント時点では hit-area を含む子要素が 1 つも無い。
    assert!(query(&container, "[data-part=\"hit-area\"]").is_none());
    wire_chart_events(container.clone())
        .expect("wire_chart_events must succeed even without any chart at mount time");

    // 後から構造再描画相当でチャートを追加する。
    bar_like_markup("chart-test-late-mount");
    await_next_tick().await;

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]")
        .expect("hit-area 0 must exist after late insertion");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");

    // enhance() が再適用され、pointermove がツールチップを開けることを
    // 確認する（配線が実際に効いていることの直接証拠）。
    dispatch_pointer(hit0.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(!tooltip0.has_attribute("hidden"));
    assert!(hit0.has_attribute("data-active"));
}

#[wasm_bindgen_test]
fn focus_session_survives_unrelated_pointer_movement_and_closes_on_focusout() {
    // codex/Bugbot レビュー指摘（イシュー #2130 PR #2267）の回帰テスト:
    // キーボードフォーカスで開いたツールチップは、hit-area 以外
    // （軸ラベル相当）への無関係なポインタ移動だけでは閉じない
    // （hover/focus は独立した「開いたままにする理由」であり、両方非活性
    // になって初めて閉じる、`should_close_session` 契約）。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-focus-pointer");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-focus-pointer");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0 = query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0");
    let axis_label = query(&container, "[data-part=\"axis-label\"]").expect("axis label");
    let tooltip0 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0");

    hit0.dispatch_event(focus_event("focusin", None).as_ref())
        .expect("dispatch_event must not fail");
    assert!(!tooltip0.has_attribute("hidden"));

    // hit-area 以外へのポインタ移動は hover を非活性化するのみで、
    // focus が活性のためセッションは閉じない。
    dispatch_pointer(axis_label.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(
        !tooltip0.has_attribute("hidden"),
        "focus セッションは無関係なポインタ移動だけでは閉じないはずである"
    );

    // フォーカスが root 配下の hit-area 以外へ移ると（hover も非活性の
    // ため）セッションが閉じる。
    hit0.dispatch_event(focus_event("focusout", Some(&container)).as_ref())
        .expect("dispatch_event must not fail");
    assert!(tooltip0.has_attribute("hidden"));
}

#[wasm_bindgen_test]
async fn touch_sticky_session_discarded_after_svg_replacement_then_pointermove_recovers() {
    // codex レビュー指摘（イシュー #2130 PR #2267）の回帰テスト: タッチで
    // 開いた sticky セッション中に構造再描画（`rerender_subtree` 相当、
    // 新しい `<svg>` 要素ごとの丸ごと差し替え）が起きると、セッションが
    // 削除済み `<svg>` を保持したままになり、以降 `pointermove` が
    // （古いセッションの `sticky == true` に阻まれて）機能しなくなって
    // いた不具合の是正を検証する。`MutationObserver` が古い `svg` の消失を
    // 検知してセッションを破棄するため、新しい `<svg>` への通常の
    // `pointermove`（mouse）だけでツールチップが開けるようになる。
    let document = web_sys::window().unwrap().document().unwrap();
    let container = create_container(&document, "chart-test-touch-rerender");
    let _guard = RemoveOnDrop(container.clone());
    bar_like_markup("chart-test-touch-rerender");
    wire_chart_events(container.clone()).expect("wire_chart_events must succeed");

    let hit0_v1 =
        query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0 (v1)");
    dispatch_pointer(hit0_v1.unchecked_ref(), "pointerdown", "touch", 5.0, 5.0);
    let tooltip0_v1 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0 (v1)");
    assert!(!tooltip0_v1.has_attribute("hidden"));

    // container 配下の frame/svg/layer を丸ごと新規要素へ差し替える
    // （`rerender_subtree` の丸ごと差し替えを模擬。既存 `<svg>` の
    // `inner_html` 書き換えとは異なり `<svg>` 要素自体が別インスタンスに
    // なる点が本回帰の再現に必須）。
    bar_like_markup("chart-test-touch-rerender");
    await_next_tick().await;

    let hit0_v2 =
        query(&container, "[data-part=\"hit-area\"][data-index=\"0\"]").expect("hit-area 0 (v2)");
    assert_ne!(
        hit0_v1, hit0_v2,
        "再描画後の hit-area は別要素インスタンスであるはずである"
    );

    let tooltip0_v2 =
        query(&container, "[data-part=\"tooltip\"][data-index=\"0\"]").expect("tooltip 0 (v2)");
    dispatch_pointer(hit0_v2.unchecked_ref(), "pointermove", "mouse", 5.0, 5.0);
    assert!(
        !tooltip0_v2.has_attribute("hidden"),
        "古い svg の破棄済みセッションに阻まれず pointermove が機能するはずである"
    );
    assert!(hit0_v2.has_attribute("data-active"));
}
