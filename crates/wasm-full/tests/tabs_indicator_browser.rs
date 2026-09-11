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
    sync_tabs_indicator, sync_tabs_indicator_in_list, INDICATOR_HEIGHT_VAR, INDICATOR_LEFT_VAR,
    INDICATOR_TOP_VAR, INDICATOR_WIDTH_VAR,
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
        // `selected` が有効な（`disabled` でない）trigger と一致するかどうかで
        // 初期 `data-state`/`hidden` を決める。実際の headless SSR
        // （`crates/headless-ui/src/tabs.rs`
        // `indicator_true_with_active_tab_full_html_snapshot` 等の golden）は
        // 一致する場合 `data-state="active"`・`hidden` 属性なしで出力し、
        // 4 変数（`--left`/`--top`/`--width`/`--height`）のみ `0px` 固定の
        // ままとする（実測配線前は `wire_keynav`/`wire_headless_component` が
        // 呼ばれるまで測定値を持たない）。このフィクスチャは SSR 契約を
        // 忠実に再現するため、`data-state`/`hidden` を無条件固定にしない。
        let is_initially_active = selected.is_some_and(|value| {
            triggers
                .iter()
                .any(|(v, _, disabled)| *v == value && !disabled)
        });
        let indicator = document.create_element("span").unwrap();
        indicator.set_attribute("data-scope", "tabs").unwrap();
        indicator.set_attribute("data-part", "indicator").unwrap();
        indicator
            .set_attribute(
                "data-state",
                if is_initially_active {
                    "active"
                } else {
                    "inactive"
                },
            )
            .unwrap();
        indicator.set_attribute("aria-hidden", "true").unwrap();
        if !is_initially_active {
            indicator.set_attribute("hidden", "").unwrap();
        }
        // headless `INDICATOR_STYLE_INITIAL` と同じ `0px` 初期値（`data-state`
        // が `active` の場合でも、headless は実測を行わないため 4 変数は
        // `0px` 固定のまま出力する）。
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
/// indicator の 4 変数（headless は測定を行わないため SSR 出力は `0px`
/// 固定のまま）を実測値へ更新する。`data-state="active"`・`hidden` 除去
/// 自体は headless SSR が選択一致時点で既に行う
/// （`crates/headless-ui/src/tabs.rs`
/// `indicator_true_with_active_tab_full_html_snapshot` 参照）ため、
/// wire 前後で変化しない。
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

    // wire_keynav 前は SSR 初期値のまま: 選択中 trigger と一致するため
    // `data-state="active"`・`hidden` なしは SSR 時点で既に成立しているが、
    // 4 変数は headless が測定しないため `0px` 固定のまま。
    assert_eq!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));

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

/// レビュー指摘是正: 選択中（`data-state="active"`）の trigger が `list`
/// 内に見つからない分岐（モジュール doc「書き込み順序」節 2.）を検証する。
/// `data-state="inactive"`・`hidden` は設定されるが、4 変数は SSR 初期値
/// のまま一切書き込まれない（この分岐が「触らない」ことを、初期値ではなく
/// 事前に別値を設定した状態から検証し、コードが実際に無変更のままである
/// ことを固定する）。
#[wasm_bindgen_test]
fn missing_active_trigger_leaves_four_vars_untouched() {
    let document = web_sys::window().unwrap().document().unwrap();
    // `selected: None` のため、いずれの trigger にも `data-state="active"`
    // が付かない（`build_tabs_dom_with_indicator` 参照）。
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-noactive1",
        &[("a", "A", false), ("b", "B", false)],
        None,
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);

    // 「触らない」ことをこの呼び出し前後の差分で確認するため、SSR 初期値
    // （0px）とは異なる値を事前に設定しておく。
    indicator
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .style()
        .set_property(INDICATOR_LEFT_VAR, "42px")
        .unwrap();

    sync_tabs_indicator_in_list(&list);

    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("inactive")
    );
    assert!(indicator.has_attribute("hidden"));
    assert_eq!(
        style_var(&indicator, INDICATOR_LEFT_VAR),
        "42px",
        "選択中 trigger が無いとき --left は一切書き換わらないこと"
    );
}

/// レビュー指摘是正: 実測 `width`/`height` が 0 以下（`display: none` 下等
/// でレイアウト未確定）の分岐（モジュール doc「書き込み順序」節 3.）を
/// 検証する。選択中 trigger 自体は見つかる（`data-state="active"`）が
/// 矩形が 0 のため、4 変数への書き込み・`data-state`/`hidden` の更新の
/// いずれも行われない。
#[wasm_bindgen_test]
fn zero_size_active_trigger_skips_write() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-zero1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let trigger_a = document.get_element_by_id("ti-zero1-trigger-a").unwrap();
    // `display: none` でレイアウトを未確定にする
    // （`getBoundingClientRect()` が 0 矩形を返す）。
    trigger_a
        .set_attribute("style", "width: 40px; height: 20px; display: none;")
        .unwrap();

    let list = document
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let indicator = indicator_of(&list);

    // SSR 初期状態: `selected` と一致するため `data-state="active"`・
    // `hidden` なし・4 変数は `0px`（`build_tabs_dom_with_indicator` 参照）。
    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));

    sync_tabs_indicator_in_list(&list);

    // 0 矩形のため書き込みは一切発生せず、SSR 初期状態のまま。
    assert_eq!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
    assert_eq!(style_var(&indicator, INDICATOR_HEIGHT_VAR), "0px");
    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));
}

/// レビュー指摘是正: `crate::headless::wire_headless_component`
/// が実際に呼ぶ入口 [`sync_tabs_indicator`]（`list` を直接受け取る
/// [`sync_tabs_indicator_in_list`] とは異なり、`root` 配下を
/// `query_selector_all` で走査して所属 `list` を都度解決する経路）を
/// `wire_keynav` を経由せず直接契約テストする。
#[wasm_bindgen_test]
fn sync_tabs_indicator_root_entry_point_updates_descendant_indicator() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-rootentry1",
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

    assert_eq!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");

    sync_tabs_indicator(&root).expect("sync_tabs_indicator must succeed");

    assert_eq!(
        indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!indicator.has_attribute("hidden"));
    assert_ne!(style_var(&indicator, INDICATOR_WIDTH_VAR), "0px");
    assert_ne!(style_var(&indicator, INDICATOR_HEIGHT_VAR), "0px");
}

/// レビュー指摘是正（イシュー #2211 PR #2342 codex-review P1）: 初期非表示
/// のタブパネル内にネストした tabs がある場合、マウント時は内部 trigger の
/// 矩形が 0 で indicator 実測がスキップされる
/// （`crate::tabs_indicator` モジュール doc「書き込み順序」節の 3.
/// 「`width`/`height` が 0 以下なら何も書き込まない」参照）。その後、
/// 親タブをクリックしてパネルを表示したときに `keynav::activate_tab` が
/// 表示対象の `content` 配下も再同期し、内部 indicator が 0px のまま
/// 欠落しないことを検証する。
#[wasm_bindgen_test]
fn nested_tabs_indicator_syncs_when_parent_panel_becomes_visible() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom_with_indicator(
        &document,
        "ti-nested1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    // 初期非表示（`data-state="inactive"` + `hidden`）の content "b" の中に
    // ネストした tabs（indicator 付き）を組み込む。
    let content_b = document.get_element_by_id("ti-nested1-content-b").unwrap();
    let nested_root = build_tabs_dom_with_indicator(
        &document,
        "ti-nested1-inner",
        &[("x", "X", false), ("y", "Y", false)],
        Some("x"),
        "automatic",
        true,
    );
    // `build_tabs_dom_with_indicator` は `document.body()` 直下へ append する
    // ため、外側 content 配下へ付け替える（ネスト構造の再現）。
    nested_root.remove();
    content_b.append_child(&nested_root).unwrap();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let nested_list = nested_root
        .query_selector(r#"[data-scope="tabs"][data-part="list"]"#)
        .unwrap()
        .unwrap();
    let nested_indicator = indicator_of(&nested_list);

    // マウント時点では外側 content "b" が `hidden` のため、内側 trigger の
    // 矩形は 0 であり実測がスキップされている（既存挙動）。
    assert_eq!(style_var(&nested_indicator, INDICATOR_WIDTH_VAR), "0px");

    let trigger_b = document.get_element_by_id("ti-nested1-trigger-b").unwrap();
    trigger_b.dispatch_event(&click_event()).unwrap();

    assert!(
        !content_b.has_attribute("hidden"),
        "親タブ b のクリック後は content が可視化されていること"
    );
    assert_ne!(
        style_var(&nested_indicator, INDICATOR_WIDTH_VAR),
        "0px",
        "親パネル表示後はネストした tabs の indicator も実測値へ同期される \
         こと（欠落したままにならないこと）"
    );
    assert_eq!(
        nested_indicator.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(!nested_indicator.has_attribute("hidden"));
}
