//! `fandhe_frontend_wasm_full::tabs_indicator`（イシュー #2211、
//! `crates/wasm-full/src/tabs_indicator.rs`）の実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/src/tabs_indicator.rs` 内の native 単体テストは純粋層
//! （[`indicator_rect`]/[`format_px`]）の計算契約までを検証する。本ファイル
//! はその先、**実ブラウザ（headless Chromium、`wasm-pack test --headless
//! --chrome`）上での `wire_keynav`/`wire_headless_component` 経由の実測・
//! CSS 変数書き込み**という製品経路を検証する
//! （`content_height_browser.rs`/`keynav_browser.rs` と同型の実 DOM
//! 検証パターンを踏襲する）。
//!
//! DOM 構造は `keynav_browser.rs::build_tabs_dom` と同じく
//! `crates/headless-ui/src/tabs.rs` の SSR 出力契約
//! （`data-scope`/`data-part`/`aria-*`/`data-state`）を手組みで再現する
//! （本クレートは `fandhe-frontend-headless-ui` に依存しないため）。
//! `indicator` パーツは `INDICATOR_STYLE_INITIAL` と同じ `0px` 初期値を
//! 持たせ、`list`/`trigger` へは幅を持たせるための最小限のインライン
//! `style` を与える（headless Chromium はデフォルトの `<button>` 描画で
//! 既に非ゼロの幅・高さを持つため必須ではないが、実測値のばらつきを
//! 抑えて assertion を安定させる）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::keynav::wire_keynav;
use fandhe_frontend_wasm_full::tabs_indicator::{
    sync_tabs_indicator_in_list, INDICATOR_HEIGHT_VAR, INDICATOR_LEFT_VAR, INDICATOR_TOP_VAR,
    INDICATOR_WIDTH_VAR,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`content_height_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
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

/// `crates/headless-ui/src/tabs.rs` の SSR 出力契約を手組みで再現した Tabs
/// DOM を、`indicator` パーツ（opt-in）付きで生成する。`triggers`:
/// `(value, label, disabled)` のリスト。`with_indicator` が `false` の
/// ときは `indicator: false`（headless `TabsProps::indicator`）相当を
/// 再現し、indicator 要素自体を生成しない。
#[allow(clippy::too_many_arguments)]
fn build_tabs_dom_with_indicator(
    document: &Document,
    root_id: &str,
    triggers: &[(&str, &str, bool)],
    selected: Option<&str>,
    activation_mode: &str,
    with_indicator: bool,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "tabs").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let list = document.create_element("div").unwrap();
    list.set_attribute("data-scope", "tabs").unwrap();
    list.set_attribute("data-part", "list").unwrap();
    list.set_attribute("role", "tablist").unwrap();
    list.set_attribute("data-orientation", "horizontal")
        .unwrap();
    list.set_attribute("data-activation-mode", activation_mode)
        .unwrap();
    list.set_attribute("data-loop-focus", "true").unwrap();
    // `getBoundingClientRect` の実測値を安定させるための最小限のレイアウト
    // 指定（`position: relative` は `crates/pre-styled-ui/src/tabs.rs`
    // recipe が付与する契約の再現、モジュール doc「実測の数式」節参照）。
    list.set_attribute("style", "display: flex; position: relative;")
        .unwrap();

    let mut first_tabbable_set = false;
    for (value, label, disabled) in triggers {
        let is_active = selected == Some(*value);
        let trigger = document.create_element("button").unwrap();
        trigger.set_attribute("data-scope", "tabs").unwrap();
        trigger.set_attribute("data-part", "trigger").unwrap();
        trigger.set_attribute("type", "button").unwrap();
        let trigger_id = format!("{root_id}-trigger-{value}");
        let content_id = format!("{root_id}-content-{value}");
        trigger.set_attribute("id", &trigger_id).unwrap();
        trigger.set_attribute("role", "tab").unwrap();
        trigger
            .set_attribute("aria-selected", if is_active { "true" } else { "false" })
            .unwrap();
        trigger.set_attribute("aria-controls", &content_id).unwrap();
        trigger
            .set_attribute("data-state", if is_active { "active" } else { "inactive" })
            .unwrap();
        trigger
            .set_attribute("style", "width: 40px; height: 20px;")
            .unwrap();
        let is_tabbable = is_active || (!first_tabbable_set && !disabled && selected.is_none());
        if is_tabbable {
            first_tabbable_set = true;
        }
        trigger
            .set_attribute("tabindex", if is_tabbable { "0" } else { "-1" })
            .unwrap();
        if *disabled {
            trigger.set_attribute("disabled", "").unwrap();
            trigger.set_attribute("data-disabled", "").unwrap();
        }
        trigger.set_text_content(Some(label));
        list.append_child(&trigger).unwrap();

        let content = document.create_element("div").unwrap();
        content.set_attribute("data-scope", "tabs").unwrap();
        content.set_attribute("data-part", "content").unwrap();
        content.set_attribute("id", &content_id).unwrap();
        content.set_attribute("role", "tabpanel").unwrap();
        content
            .set_attribute("data-state", if is_active { "active" } else { "inactive" })
            .unwrap();
        if !is_active {
            content.set_attribute("hidden", "").unwrap();
        }
        content.set_text_content(Some(&format!("panel-{value}")));
        root.append_child(&content).unwrap();
    }

    if with_indicator {
        let indicator = document.create_element("span").unwrap();
        indicator.set_attribute("data-scope", "tabs").unwrap();
        indicator.set_attribute("data-part", "indicator").unwrap();
        indicator.set_attribute("data-state", "inactive").unwrap();
        indicator.set_attribute("aria-hidden", "true").unwrap();
        indicator.set_attribute("hidden", "").unwrap();
        // headless `INDICATOR_STYLE_INITIAL` と同じ `0px` 初期値。
        indicator
            .set_attribute(
                "style",
                "--left: 0px; --top: 0px; --width: 0px; --height: 0px",
            )
            .unwrap();
        list.append_child(&indicator).unwrap();
    }

    root.insert_before(&list, root.first_child().as_ref())
        .unwrap();
    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// `element` の CSSOM 経由の CSS custom property 値を読む（文字列比較では
/// なく `CssStyleDeclaration` 経由で読み、書き込み手段〔CSSOM〕と対称に
/// する。`content_height_browser.rs::content_height_var` と同型）。
fn style_var(element: &Element, name: &str) -> String {
    element
        .dyn_ref::<HtmlElement>()
        .expect("element must be an HtmlElement")
        .style()
        .get_property_value(name)
        .expect("get_property_value must not fail")
}

fn indicator_of(list: &Element) -> Element {
    list.query_selector(r#"[data-scope="tabs"][data-part="indicator"]"#)
        .expect("query_selector must not fail")
        .expect("indicator element must exist")
}

/// 検証: `wire_keynav` のマウント時初期同期（`sync_tabs_indicator`）が、
/// SSR 初期状態で選択中の trigger（`selected: Some(...)`）に対して
/// indicator の 4 変数を実測値へ更新し、`data-state="active"`・`hidden`
/// 除去を行う。
#[wasm_bindgen_test]
fn mount_time_sync_updates_indicator_for_initially_selected_trigger() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-mount1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);

    // wire_keynav 前は SSR 初期値のまま。
    assert_eq!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
    assert!(indicator.has_attribute("hidden"));

    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));
    assert_ne!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
    assert_ne!(style_var(&indicator, INDICATOR_HEIGHT_VAR), "0px");
    // 先頭 trigger は `list` の padding box 起点と一致するため、`left`/`top`
    // はいずれも `0px` 近傍になる（`list` に padding を与えていないため）。
    assert_eq!(style_var(&indicator, INDICATOR_TOP_VAR), "0px");
}

/// 検証: click による活性化切替（automatic/manual を問わず click は必ず
/// 活性化を伴う経路、`keynav.rs::handle_trigger_click` doc 参照）で
/// indicator の `left` が新しい選択 trigger の位置へ追従する。
#[wasm_bindgen_test]
fn click_activation_moves_indicator_left() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-click1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);
    let trigger_b = document.get_element_by_id("ti-click1-trigger-b").unwrap();

    let left_before = style_var(&indicator, INDICATOR_LEFT_VAR);

    trigger_b.dispatch_event(&click_event()).unwrap();

    let left_after = style_var(&indicator, INDICATOR_LEFT_VAR);
    assert_ne!(
        left_before, left_after,
        "trigger b への click 後は indicator の left が trigger a 選択時から \
         変化していること"
    );
    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
}

/// 検証: automatic activation の keydown（ArrowRight でフォーカス移動と
/// 同時に活性化）でも indicator が追従する。
#[wasm_bindgen_test]
fn automatic_activation_keydown_moves_indicator() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-auto1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);
    let trigger_a = document.get_element_by_id("ti-auto1-trigger-a").unwrap();

    let left_before = style_var(&indicator, INDICATOR_LEFT_VAR);

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    let left_after = style_var(&indicator, INDICATOR_LEFT_VAR);
    assert_ne!(
        left_before, left_after,
        "automatic activation の ArrowRight 後は indicator が trigger b の \
         位置へ追従していること"
    );
}

/// 検証: manual activation の keydown はフォーカス移動のみで
/// `activate_tab` を呼ばないため、indicator は追従しない
/// （`crates/wasm-full/src/tabs_indicator.rs` モジュール doc
/// 「`crate::keynav`/`crate::headless::wire_headless_component` との統合」
/// 節参照）。
#[wasm_bindgen_test]
fn manual_activation_keydown_does_not_move_indicator() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-manual1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "manual",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);
    let trigger_a = document.get_element_by_id("ti-manual1-trigger-a").unwrap();

    let left_before = style_var(&indicator, INDICATOR_LEFT_VAR);
    let width_before = style_var(&indicator, INDICATOR_WIDTH_VAR);

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    // フォーカス（roving tabindex）は移動するが、indicator は trigger a の
    // 位置のまま変化しない（選択自体が変わっていないため）。
    let trigger_b = document.get_element_by_id("ti-manual1-trigger-b").unwrap();
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(style_var(&indicator, INDICATOR_LEFT_VAR), left_before);
    assert_eq!(style_var(&indicator, INDICATOR_WIDTH_VAR), width_before);

    // click（Enter/Space 相当）で初めて追従する。
    trigger_b.dispatch_event(&click_event()).unwrap();
    assert_ne!(style_var(&indicator, INDICATOR_LEFT_VAR), left_before);
}

/// 検証: `indicator: false`（headless `TabsProps::indicator` が `false`
/// のとき SSR は indicator 要素自体を出力しない）相当の DOM では、click
/// 活性化を行っても panic せず完全な no-op のまま（`sync_tabs_indicator_
/// in_list` の早期 return、モジュール doc「書き込み順序」節 1.）。
#[wasm_bindgen_test]
fn tabs_without_indicator_part_is_a_complete_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-noindicator1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_b = document
        .get_element_by_id("ti-noindicator1-trigger-b")
        .unwrap();
    // panic しないことそのものが検証対象（indicator 要素が存在しない DOM
    // での click 活性化）。
    trigger_b.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
}

/// [`sync_tabs_indicator_in_list`] を `list` に対して直接呼んでも、
/// 選択中 trigger が存在すれば同じ実測結果になる（`wire_keynav` 経由の
/// 呼び出しと同一の公開関数を使うことの確認。`crate::headless::
/// wire_headless_component` からの呼び出し経路の代替検証）。
#[wasm_bindgen_test]
fn direct_sync_call_matches_wire_keynav_result() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-direct1",
        &[("a", "A", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);

    sync_tabs_indicator_in_list(&list);

    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));
    assert_ne!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
}
