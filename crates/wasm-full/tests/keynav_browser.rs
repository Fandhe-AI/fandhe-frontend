//! `fandhe_frontend_wasm_full::keynav::wire_keynav`（Tabs/Accordion/Menu/
//! Select/RadioGroup のキーボード操作・イシュー #582・#583、親 #581）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/keynav_native.rs`（native）は純粋層（`tabs_next_index`/
//! `accordion_next_index`/`highlight_next_index`/`radio_next_index`）までを
//! 検証済みである。本ファイルはその先、`wire_keynav` が実 DOM
//! （headless Chromium）上でキーボード委譲・roving tabindex 更新・フォーカス
//! 移動・活性化（automatic/manual）・highlight/選択・radio チェック同期を
//! 正しく反映することを検証する。
//!
//! DOM 構造は `crates/headless-ui/src/tabs.rs`/`accordion.rs`/`menu.rs`/
//! `select.rs`/`radio_group.rs` の SSR 出力契約（`data-scope`/`data-part`/
//! `aria-*`/`data-state`/`tabindex` 等）を手組みで再現する（本クレートは
//! `fandhe-frontend-headless-ui` に依存しないため、実際の `tabs()`/
//! `accordion`/`menu`/`select`/`radio_group` 関数は呼べない。属性契約の記述は
//! それぞれのモジュール doc・スナップショットテストと一致させている）。
//!
//! # 検証内容（実装計画 §6 の検証項目 1〜9 に対応）
//!
//! 1. horizontal: ArrowRight/ArrowLeft でフォーカス移動 + roving tabindex 更新
//! 2. vertical: ArrowDown/ArrowUp で同上（horizontal では no-op）
//! 3. Home/End で先頭/末尾の非 disabled trigger へ移動・disabled スキップ
//! 4. `data-loop-focus="false"` で端 no-op
//! 5. automatic: フォーカス移動と同時に `aria-selected`/`data-state`/`hidden` 反映
//! 6. manual: Arrow ではパネル不変・クリック（Enter/Space 相当）で活性化
//! 7. Accordion: ArrowDown/ArrowUp/Home/End のフォーカス移動（非循環・disabled スキップ）
//! 8. Menu/Select: closed 時 ArrowDown で open + 初期 highlight、open 時
//!    Arrow/Home/End で highlight 移動・`aria-activedescendant` 追随・
//!    disabled スキップ・既定非循環・`data-loop-focus="true"` で循環、
//!    Enter/Space で highlight 項目へ click 合成
//! 9. RadioGroup: Arrow 移動 + 同時 check + `data-state` 4 パーツ同期・循環・
//!    disabled スキップ・orientation 制限・Home/End・`change` 同期
//!
//! XSS 回帰（REQ-1）: 攻撃者制御文字列を持つラベルに対しキー操作・活性化・
//! highlight/選択を行っても `script` 要素が DOM に生成されないことを固定する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::keynav::wire_keynav;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventInit, HtmlElement, HtmlInputElement, KeyboardEvent,
    KeyboardEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`runtime_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `crates/headless-ui/src/tabs.rs` の SSR 出力契約を手組みで再現した Tabs
/// DOM を生成する。`triggers`: `(value, label, disabled)` のリスト。
/// `selected` は初期活性 value（`None` なら全 inactive）。
#[allow(clippy::too_many_arguments)]
fn build_tabs_dom(
    document: &Document,
    root_id: &str,
    triggers: &[(&str, &str, bool)],
    selected: Option<&str>,
    orientation: &str,
    activation_mode: &str,
    loop_focus: bool,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "tabs").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let list = document.create_element("div").unwrap();
    list.set_attribute("data-scope", "tabs").unwrap();
    list.set_attribute("data-part", "list").unwrap();
    list.set_attribute("role", "tablist").unwrap();
    list.set_attribute("data-orientation", orientation).unwrap();
    list.set_attribute("data-activation-mode", activation_mode)
        .unwrap();
    list.set_attribute("data-loop-focus", if loop_focus { "true" } else { "false" })
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
        // roving tabindex: active があればそれが 0、なければ最初の非 disabled。
        let is_tabbable = if is_active {
            true
        } else if !first_tabbable_set && !disabled && selected.is_none() {
            true
        } else {
            false
        };
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

    root.insert_before(&list, root.first_child().as_ref())
        .unwrap();
    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// `crates/headless-ui/src/accordion.rs` の SSR 出力契約を手組みで再現した
/// Accordion DOM（item-trigger のみが本モジュールの関心事）を生成する。
fn build_accordion_dom(document: &Document, root_id: &str, triggers: &[(&str, bool)]) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "accordion").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    for (label, disabled) in triggers {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "accordion").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        let trigger = document.create_element("button").unwrap();
        trigger.set_attribute("data-scope", "accordion").unwrap();
        trigger.set_attribute("data-part", "item-trigger").unwrap();
        trigger.set_attribute("type", "button").unwrap();
        if *disabled {
            trigger.set_attribute("disabled", "").unwrap();
            trigger.set_attribute("data-disabled", "").unwrap();
        }
        trigger.set_text_content(Some(label));
        item.append_child(&trigger).unwrap();
        root.append_child(&item).unwrap();
    }

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// `crates/headless-ui/src/menu.rs` の SSR 出力契約を手組みで再現した Menu
/// DOM を生成する。`items`: `(value, label, disabled)` のリスト。`open` が
/// `true` のとき content から `hidden` を外す。`loop_focus` が `true` のとき
/// content へ `data-loop-focus="true"` を付与する（既定 false、モジュール doc
/// §Menu/Select 参照）。各 item は `id` を `{root_id}-item-{value}` として
/// 出力し、`aria-activedescendant` の参照先として使えるようにする。
fn build_menu_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool)],
    open: bool,
    loop_focus: bool,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "menu").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let trigger = document.create_element("button").unwrap();
    trigger.set_attribute("data-scope", "menu").unwrap();
    trigger.set_attribute("data-part", "trigger").unwrap();
    trigger.set_attribute("type", "button").unwrap();
    let trigger_id = format!("{root_id}-trigger");
    let content_id = format!("{root_id}-content");
    trigger.set_attribute("id", &trigger_id).unwrap();
    trigger.set_attribute("aria-haspopup", "menu").unwrap();
    trigger
        .set_attribute("aria-expanded", if open { "true" } else { "false" })
        .unwrap();
    trigger.set_attribute("aria-controls", &content_id).unwrap();
    trigger.set_text_content(Some("Menu"));
    root.append_child(&trigger).unwrap();

    let content = document.create_element("div").unwrap();
    content.set_attribute("data-scope", "menu").unwrap();
    content.set_attribute("data-part", "content").unwrap();
    content.set_attribute("id", &content_id).unwrap();
    content.set_attribute("role", "menu").unwrap();
    if loop_focus {
        content.set_attribute("data-loop-focus", "true").unwrap();
    }
    if !open {
        content.set_attribute("hidden", "").unwrap();
    }
    for (value, label, disabled) in items {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "menu").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("role", "menuitem").unwrap();
        item.set_attribute("data-value", value).unwrap();
        item.set_attribute("id", &format!("{root_id}-item-{value}"))
            .unwrap();
        if *disabled {
            item.set_attribute("aria-disabled", "true").unwrap();
            item.set_attribute("data-disabled", "").unwrap();
        }
        item.set_text_content(Some(label));
        content.append_child(&item).unwrap();
    }
    root.append_child(&content).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// `crates/headless-ui/src/select.rs` の SSR 出力契約を手組みで再現した
/// Select DOM を生成する。[`build_menu_dom`] とほぼ同型だが `role="listbox"`/
/// `role="option"` を使う。
fn build_select_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool)],
    open: bool,
    loop_focus: bool,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "select").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let trigger = document.create_element("button").unwrap();
    trigger.set_attribute("data-scope", "select").unwrap();
    trigger.set_attribute("data-part", "trigger").unwrap();
    trigger.set_attribute("type", "button").unwrap();
    let trigger_id = format!("{root_id}-trigger");
    let content_id = format!("{root_id}-content");
    trigger.set_attribute("id", &trigger_id).unwrap();
    trigger.set_attribute("aria-haspopup", "listbox").unwrap();
    trigger
        .set_attribute("aria-expanded", if open { "true" } else { "false" })
        .unwrap();
    trigger.set_attribute("aria-controls", &content_id).unwrap();
    trigger.set_text_content(Some("Select"));
    root.append_child(&trigger).unwrap();

    let content = document.create_element("div").unwrap();
    content.set_attribute("data-scope", "select").unwrap();
    content.set_attribute("data-part", "content").unwrap();
    content.set_attribute("id", &content_id).unwrap();
    content.set_attribute("role", "listbox").unwrap();
    if loop_focus {
        content.set_attribute("data-loop-focus", "true").unwrap();
    }
    if !open {
        content.set_attribute("hidden", "").unwrap();
    }
    for (value, label, disabled) in items {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "select").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("role", "option").unwrap();
        item.set_attribute("data-value", value).unwrap();
        item.set_attribute("id", &format!("{root_id}-item-{value}"))
            .unwrap();
        if *disabled {
            item.set_attribute("aria-disabled", "true").unwrap();
            item.set_attribute("data-disabled", "").unwrap();
        }
        item.set_text_content(Some(label));
        content.append_child(&item).unwrap();
    }
    root.append_child(&content).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// `crates/headless-ui/src/radio_group.rs` の SSR 出力契約を手組みで再現した
/// RadioGroup DOM を生成する。`items`: `(value, label, checked, disabled)`
/// のリスト。`orientation` が `Some` のとき root へ `data-orientation` を
/// 付与する。
fn build_radio_group_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool, bool)],
    orientation: Option<&str>,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "radio-group").unwrap();
    root.set_attribute("data-part", "root").unwrap();
    root.set_attribute("role", "radiogroup").unwrap();
    if let Some(orientation) = orientation {
        root.set_attribute("data-orientation", orientation).unwrap();
    }

    for (value, label, checked, disabled) in items {
        let state = if *checked { "checked" } else { "unchecked" };
        let item = document.create_element("label").unwrap();
        item.set_attribute("data-scope", "radio-group").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("data-state", state).unwrap();
        if *disabled {
            item.set_attribute("data-disabled", "").unwrap();
        }

        let control = document.create_element("span").unwrap();
        control.set_attribute("data-scope", "radio-group").unwrap();
        control.set_attribute("data-part", "item-control").unwrap();
        control.set_attribute("data-state", state).unwrap();
        item.append_child(&control).unwrap();

        let text = document.create_element("span").unwrap();
        text.set_attribute("data-scope", "radio-group").unwrap();
        text.set_attribute("data-part", "item-text").unwrap();
        text.set_attribute("data-state", state).unwrap();
        text.set_text_content(Some(label));
        item.append_child(&text).unwrap();

        let input = document.create_element("input").unwrap();
        input.set_attribute("data-scope", "radio-group").unwrap();
        input
            .set_attribute("data-part", "item-hidden-input")
            .unwrap();
        input.set_attribute("type", "radio").unwrap();
        input.set_attribute("value", value).unwrap();
        input.set_attribute("data-state", state).unwrap();
        input.set_id(&format!("{root_id}-input-{value}"));
        if *checked {
            input.set_attribute("checked", "").unwrap();
        }
        if *disabled {
            input.set_attribute("disabled", "").unwrap();
        }
        item.append_child(&input).unwrap();
        // `set_attribute("checked", "")` は HTML パース時の初期値のみを表す
        // ため、`HtmlInputElement::checked()` が読む「現在の」チェック状態
        // （IDL 属性）も一致させておく（ブラウザの実 DOM 動作を模す）。
        if let Ok(html_input) = input.clone().dyn_into::<HtmlInputElement>() {
            html_input.set_checked(*checked);
        }

        root.append_child(&item).unwrap();
    }

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    root
}

/// 合成 `keydown` イベント（`bubbles: true`）を組み立てる。
fn keydown_event(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// 合成 `click` イベント（`bubbles: true`）を組み立てる
/// （`runtime_browser.rs::bubbling_event` と同じ意図。Enter/Space 相当の
/// activation もネイティブ button の click として届く前提を模する）。
fn click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

fn html_element(element: &Element) -> HtmlElement {
    element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must be an HtmlElement")
}

/// 合成 `change` イベント（`bubbles: true`）を組み立てる（ネイティブ
/// `<input type="radio">` のチェック変更を模す。`click_event` と同じ意図）。
fn change_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("change", &init).expect("Event::new must not fail")
}

/// 検証 1: horizontal で ArrowRight/ArrowLeft がフォーカス移動 + roving
/// tabindex（`0`/`-1`）を更新する。
#[wasm_bindgen_test]
fn horizontal_arrow_keys_move_focus_and_update_roving_tabindex() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-h1",
        &[("a", "A", false), ("b", "B", false), ("c", "C", false)],
        Some("a"),
        "horizontal",
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document.get_element_by_id("kn-h1-trigger-a").unwrap();
    let trigger_b = document.get_element_by_id("kn-h1-trigger-b").unwrap();

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(trigger_a.get_attribute("tabindex").as_deref(), Some("-1"));
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-h1-trigger-b".to_string())
    );

    trigger_b
        .dispatch_event(&keydown_event("ArrowLeft"))
        .unwrap();
    assert_eq!(trigger_a.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("-1"));
}

/// 検証 2: vertical では ArrowDown/ArrowUp のみが動き、horizontal 方向の
/// キー（ArrowRight）は no-op のまま。
#[wasm_bindgen_test]
fn vertical_orientation_only_responds_to_vertical_arrows() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-v1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "vertical",
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document.get_element_by_id("kn-v1-trigger-a").unwrap();
    let trigger_b = document.get_element_by_id("kn-v1-trigger-b").unwrap();

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(trigger_a.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("-1"));

    trigger_a
        .dispatch_event(&keydown_event("ArrowDown"))
        .unwrap();
    assert_eq!(trigger_a.get_attribute("tabindex").as_deref(), Some("-1"));
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("0"));
}

/// 検証 3・4: Home/End が disabled をスキップし、`data-loop-focus="false"`
/// では端で no-op になる。
#[wasm_bindgen_test]
fn home_end_skip_disabled_and_loop_focus_false_is_noop_at_edges() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-he1",
        &[
            ("a", "A", true),
            ("b", "B", false),
            ("c", "C", false),
            ("d", "D", true),
        ],
        Some("b"),
        "horizontal",
        "automatic",
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_b = document.get_element_by_id("kn-he1-trigger-b").unwrap();
    let trigger_c = document.get_element_by_id("kn-he1-trigger-c").unwrap();

    trigger_b.dispatch_event(&keydown_event("End")).unwrap();
    assert_eq!(trigger_c.get_attribute("tabindex").as_deref(), Some("0"));

    // loop_focus=false のため、末尾（c）から ArrowRight しても移動しない。
    trigger_c
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(trigger_c.get_attribute("tabindex").as_deref(), Some("0"));

    trigger_c.dispatch_event(&keydown_event("Home")).unwrap();
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("0"));
}

/// 検証 5: automatic activationMode ではフォーカス移動と同時に
/// `aria-selected`/`data-state`/`hidden` が切り替わる。
#[wasm_bindgen_test]
fn automatic_activation_mode_activates_on_focus_move() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-auto1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "horizontal",
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document.get_element_by_id("kn-auto1-trigger-a").unwrap();
    let trigger_b = document.get_element_by_id("kn-auto1-trigger-b").unwrap();
    let content_a = document.get_element_by_id("kn-auto1-content-a").unwrap();
    let content_b = document.get_element_by_id("kn-auto1-content-b").unwrap();

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(
        trigger_a.get_attribute("aria-selected").as_deref(),
        Some("false")
    );
    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        trigger_b.get_attribute("data-state").as_deref(),
        Some("active")
    );
    assert!(content_a.has_attribute("hidden"));
    assert!(!content_b.has_attribute("hidden"));
    assert_eq!(
        content_b.get_attribute("data-state").as_deref(),
        Some("active")
    );
}

/// 検証 6: manual activationMode では Arrow キーはフォーカス移動のみで
/// パネルは不変。クリック（Enter/Space 相当）で初めて活性化する。
#[wasm_bindgen_test]
fn manual_activation_mode_requires_explicit_click_to_activate() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-manual1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "horizontal",
        "manual",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document.get_element_by_id("kn-manual1-trigger-a").unwrap();
    let trigger_b = document.get_element_by_id("kn-manual1-trigger-b").unwrap();
    let content_a = document.get_element_by_id("kn-manual1-content-a").unwrap();
    let content_b = document.get_element_by_id("kn-manual1-content-b").unwrap();

    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    // フォーカス（roving tabindex）は移動するが、選択状態は不変。
    assert_eq!(trigger_b.get_attribute("tabindex").as_deref(), Some("0"));
    assert_eq!(
        trigger_a.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("false")
    );
    assert!(!content_a.has_attribute("hidden"));
    assert!(content_b.has_attribute("hidden"));

    // クリック（Enter/Space 相当）で初めて活性化する。
    trigger_b.dispatch_event(&click_event()).unwrap();
    assert_eq!(
        trigger_a.get_attribute("aria-selected").as_deref(),
        Some("false")
    );
    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert!(content_a.has_attribute("hidden"));
    assert!(!content_b.has_attribute("hidden"));
}

/// disabled trigger はクリックしても活性化されない（fail-closed）。
#[wasm_bindgen_test]
fn click_on_disabled_trigger_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-disabled-click1",
        &[("a", "A", false), ("b", "B", true)],
        Some("a"),
        "horizontal",
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document
        .get_element_by_id("kn-disabled-click1-trigger-a")
        .unwrap();
    let trigger_b = document
        .get_element_by_id("kn-disabled-click1-trigger-b")
        .unwrap();

    trigger_b.dispatch_event(&click_event()).unwrap();

    assert_eq!(
        trigger_a.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("false")
    );
}

/// クリックが trigger のテキストラベル（子テキストノード）を `event.target()`
/// として届く場合でも活性化される（`events::wire_events` と同方針で
/// `Node::parent_element()` を経由した祖先探索を要求する回帰、Cursor Bugbot
/// 指摘・PR #612）。ブラウザは実際のマウスクリックでテキストノードを
/// `target` に含めるため、trigger 要素ではなくその最初の子ノード（テキスト
/// ノード）へ直接 `dispatch_event` することで実クリックを模する。
#[wasm_bindgen_test]
fn click_on_trigger_text_label_activates_manual_tab() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-textlabel1",
        &[("a", "A", false), ("b", "B", false)],
        Some("a"),
        "horizontal",
        "manual",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_b = document
        .get_element_by_id("kn-textlabel1-trigger-b")
        .unwrap();
    let content_a = document
        .get_element_by_id("kn-textlabel1-content-a")
        .unwrap();
    let content_b = document
        .get_element_by_id("kn-textlabel1-content-b")
        .unwrap();
    let label_text_node = trigger_b
        .first_child()
        .expect("trigger must contain its label text node");

    label_text_node
        .dispatch_event(&click_event())
        .expect("dispatch_event on text node must not fail");

    assert_eq!(
        trigger_b.get_attribute("aria-selected").as_deref(),
        Some("true")
    );
    assert!(content_a.has_attribute("hidden"));
    assert!(!content_b.has_attribute("hidden"));
}

/// 検証 7: Accordion は ArrowDown/ArrowUp/Home/End でフォーカス移動する
/// （非循環・disabled スキップ、roving tabindex・活性化は変更しない）。
#[wasm_bindgen_test]
fn accordion_arrow_and_home_end_move_focus_without_looping() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_accordion_dom(
        &document,
        "kn-acc1",
        &[("one", false), ("two", false), ("three", false)],
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let triggers: Vec<Element> = {
        let list = root
            .query_selector_all("[data-part=\"item-trigger\"]")
            .unwrap();
        (0..list.length())
            .map(|i| list.get(i).unwrap().dyn_into::<Element>().unwrap())
            .collect()
    };
    html_element(&triggers[0]).focus().unwrap();

    triggers[0]
        .dispatch_event(&keydown_event("ArrowDown"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.text_content()),
        Some(Some("two".to_string()))
    );

    triggers[1]
        .dispatch_event(&keydown_event("ArrowUp"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.text_content()),
        Some(Some("one".to_string()))
    );

    triggers[0].dispatch_event(&keydown_event("End")).unwrap();
    assert_eq!(
        document.active_element().map(|el| el.text_content()),
        Some(Some("three".to_string()))
    );

    // 非循環: 末尾から ArrowDown は no-op。
    triggers[2]
        .dispatch_event(&keydown_event("ArrowDown"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.text_content()),
        Some(Some("three".to_string()))
    );
}

/// XSS 回帰（REQ-1）: 攻撃者制御文字列を含むラベルを持つ trigger に対し
/// キー操作・活性化を行っても `script` 要素が DOM に生成されないこと。
/// `keynav` は `set_attribute`/`focus()` のみで `set_inner_html` を使わない
/// ため、ラベル自体は本モジュールの書き込み対象外だが、隣接ノードの属性
/// 更新が攻撃者制御ラベルの存在下でも安全であることを固定する。
#[wasm_bindgen_test]
fn keyboard_navigation_with_attacker_controlled_label_does_not_inject_script() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_tabs_dom(
        &document,
        "kn-xss1",
        &[("a", "<script>alert(1)</script>", false), ("b", "B", false)],
        Some("a"),
        "horizontal",
        "automatic",
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger_a = document.get_element_by_id("kn-xss1-trigger-a").unwrap();
    trigger_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(root.query_selector("script").unwrap().is_none());
    assert_eq!(
        trigger_a.text_content().as_deref(),
        Some("<script>alert(1)</script>")
    );
}

// ---------------------------------------------------------------------
// Menu/Select（イシュー #583）
//
// `keynav::wire_keynav` 自体は Menu/Select の開閉 dispatch を行わない
// （`events::wire_events` + `Component::decode_action` の責務、モジュール doc
// §Menu/Select 参照）。そのため各テストは trigger へ「クリックされたら
// content の `hidden` を外す」だけの薄いネイティブ click リスナーを事前登録
// し、`events::wire_events` が担う開閉 dispatch を模擬する。同様に item への
// 決定（Enter/Space）合成 click は、item へ「クリックされたら
// `data-clicked` を立てる」薄いリスナーで検知する。
// ---------------------------------------------------------------------

/// closed の trigger 上で ArrowDown を押すと trigger へ `click()` が合成され
/// （模擬リスナーが `hidden` を外す）、開いた直後に先頭の非 disabled 項目が
/// 初期 highlight される。
#[wasm_bindgen_test]
fn menu_closed_arrow_down_opens_via_synthesized_click_and_sets_initial_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-open1",
        &[("a", "A", false), ("b", "B", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let content = document.get_element_by_id("kn-menu-open1-content").unwrap();
    let open_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new({
        let content = content.clone();
        move |_event: Event| {
            let _ = content.remove_attribute("hidden");
        }
    });
    let trigger = document.get_element_by_id("kn-menu-open1-trigger").unwrap();
    trigger
        .add_event_listener_with_callback("click", open_closure.as_ref().unchecked_ref())
        .unwrap();
    open_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();

    assert!(!content.has_attribute("hidden"));
    let item_a = document.get_element_by_id("kn-menu-open1-item-a").unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-menu-open1-item-a")
    );
}

/// open の Menu で ArrowDown/ArrowUp/Home/End が highlight を移動し、
/// `aria-activedescendant` が追随し、disabled をスキップし、既定では端で
/// 循環しない。
#[wasm_bindgen_test]
fn menu_open_arrow_and_home_end_move_highlight_and_skip_disabled_without_looping() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-nav1",
        &[
            ("a", "A", false),
            ("b", "B", true),
            ("c", "C", false),
            ("d", "D", false),
        ],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-menu-nav1-trigger").unwrap();
    let content = document.get_element_by_id("kn-menu-nav1-content").unwrap();
    let item_a = document.get_element_by_id("kn-menu-nav1-item-a").unwrap();
    let item_c = document.get_element_by_id("kn-menu-nav1-item-c").unwrap();
    let item_d = document.get_element_by_id("kn-menu-nav1-item-d").unwrap();
    html_element(&trigger).focus().unwrap();

    // 先頭 highlight なしから ArrowDown → 先頭（a）。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // b は disabled のためスキップして c へ。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(item_c.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-menu-nav1-item-c")
    );

    // End → 末尾（d）。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_d.has_attribute("data-highlighted"));

    // 既定非循環: 末尾から ArrowDown は no-op。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_d.has_attribute("data-highlighted"));

    // Home → 先頭（a）。
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert!(!item_d.has_attribute("data-highlighted"));
}

/// `data-loop-focus="true"` が明示された Menu content は端で循環する。
#[wasm_bindgen_test]
fn menu_open_with_explicit_loop_focus_true_wraps_at_ends() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-loop1",
        &[("a", "A", false), ("b", "B", false)],
        true,
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-menu-loop1-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-menu-loop1-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-menu-loop1-item-b").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));

    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// open の Menu で Enter/Space を押すと highlight 中の項目へ `click()` が
/// 合成される（`data-action` 相当の模擬リスナーで検知）。disabled highlight
/// は no-op。
#[wasm_bindgen_test]
fn menu_open_enter_and_space_click_highlighted_item_and_skip_disabled() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-select1",
        &[("a", "A", false), ("b", "B", true)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let item_a = document
        .get_element_by_id("kn-menu-select1-item-a")
        .unwrap();
    let item_b = document
        .get_element_by_id("kn-menu-select1-item-b")
        .unwrap();
    for item in [&item_a, &item_b] {
        let item_for_closure = item.clone();
        let click_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |_e| {
            let _ = item_for_closure.set_attribute("data-clicked", "");
        });
        item.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
            .unwrap();
        click_closure.forget();
    }

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document
        .get_element_by_id("kn-menu-select1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    // highlight を a へ移動してから Enter。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    trigger.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_a.has_attribute("data-clicked"));
    assert!(!item_b.has_attribute("data-clicked"));

    // b（disabled）へ highlight を移動しようとしても a のまま留まる
    // （非循環・disabled スキップ、b は候補から除外される）。Space を押しても
    // 引き続き a がクリックされる（highlight は a のまま）。
    let _ = item_a.remove_attribute("data-clicked");
    trigger.dispatch_event(&keydown_event(" ")).unwrap();
    assert!(item_a.has_attribute("data-clicked"));
}

/// Select も Menu と同じ highlight/決定契約を共有する（`role="listbox"`/
/// `[data-part="item"]`、PR #617 の SSR 契約と一致確認）。
#[wasm_bindgen_test]
fn select_open_arrow_moves_highlight_and_enter_clicks_highlighted_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_select_dom(
        &document,
        "kn-select1",
        &[("apple", "Apple", false), ("banana", "Banana", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let item_apple = document.get_element_by_id("kn-select1-item-apple").unwrap();
    let item_banana = document
        .get_element_by_id("kn-select1-item-banana")
        .unwrap();
    let click_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new({
        let item_banana = item_banana.clone();
        move |_e| {
            let _ = item_banana.set_attribute("data-clicked", "");
        }
    });
    item_banana
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    click_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document.get_element_by_id("kn-select1-trigger").unwrap();
    let content = document.get_element_by_id("kn-select1-content").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_apple.has_attribute("data-highlighted"));
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_banana.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-select1-item-banana")
    );

    trigger.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_banana.has_attribute("data-clicked"));
}

// ---------------------------------------------------------------------
// RadioGroup（イシュー #583）
// ---------------------------------------------------------------------

/// ArrowRight/ArrowDown で次、ArrowLeft/ArrowUp で前の非 disabled 項目へ
/// 移動し、フォーカス移動と同時にチェックされ、4 パーツの `data-state` が
/// 同期し、端で循環する。
#[wasm_bindgen_test]
fn radio_group_arrow_moves_focus_checks_and_syncs_state_with_looping() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_radio_group_dom(
        &document,
        "kn-radio1",
        &[
            ("a", "A", true, false),
            ("b", "B", false, false),
            ("c", "C", false, false),
        ],
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let input_a = document.get_element_by_id("kn-radio1-input-a").unwrap();
    let input_b = document.get_element_by_id("kn-radio1-input-b").unwrap();
    let input_c = document.get_element_by_id("kn-radio1-input-c").unwrap();
    html_element(&input_a).focus().unwrap();

    input_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio1-input-b".to_string())
    );
    assert!(input_b
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .checked());
    assert!(!input_a
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .checked());
    assert_eq!(
        input_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    assert_eq!(
        input_a.get_attribute("data-state").as_deref(),
        Some("unchecked")
    );
    // item/item-control/item-text も同期する。
    let item_b = input_b.parent_element().unwrap();
    assert_eq!(
        item_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    let control_b = item_b
        .query_selector("[data-part=\"item-control\"]")
        .unwrap()
        .unwrap();
    assert_eq!(
        control_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    let text_b = item_b
        .query_selector("[data-part=\"item-text\"]")
        .unwrap()
        .unwrap();
    assert_eq!(
        text_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );

    // 末尾（c）から ArrowRight で循環して先頭（a）へ。
    input_b
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio1-input-c".to_string())
    );
    input_c
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio1-input-a".to_string())
    );
    assert!(input_a
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .checked());
}

/// disabled 項目をスキップして移動する。
#[wasm_bindgen_test]
fn radio_group_skips_disabled_items() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_radio_group_dom(
        &document,
        "kn-radio-disabled1",
        &[
            ("a", "A", true, false),
            ("b", "B", false, true),
            ("c", "C", false, false),
        ],
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let input_a = document
        .get_element_by_id("kn-radio-disabled1-input-a")
        .unwrap();
    html_element(&input_a).focus().unwrap();
    input_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio-disabled1-input-c".to_string())
    );
}

/// `data-orientation="horizontal"` の RadioGroup は左右キーのみを受理し、
/// 上下キーは no-op。
#[wasm_bindgen_test]
fn radio_group_horizontal_orientation_ignores_vertical_keys() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_radio_group_dom(
        &document,
        "kn-radio-horiz1",
        &[("a", "A", true, false), ("b", "B", false, false)],
        Some("horizontal"),
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let input_a = document
        .get_element_by_id("kn-radio-horiz1-input-a")
        .unwrap();
    html_element(&input_a).focus().unwrap();

    input_a.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio-horiz1-input-a".to_string())
    );

    input_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio-horiz1-input-b".to_string())
    );
}

/// Home/End は orientation に関わらず先頭/末尾の非 disabled 項目へ移動する。
#[wasm_bindgen_test]
fn radio_group_home_end_move_to_first_last_enabled() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_radio_group_dom(
        &document,
        "kn-radio-he1",
        &[
            ("a", "A", false, true),
            ("b", "B", true, false),
            ("c", "C", false, false),
            ("d", "D", false, true),
        ],
        Some("vertical"),
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let input_b = document.get_element_by_id("kn-radio-he1-input-b").unwrap();
    html_element(&input_b).focus().unwrap();

    input_b.dispatch_event(&keydown_event("End")).unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio-he1-input-c".to_string())
    );

    let input_c = document.get_element_by_id("kn-radio-he1-input-c").unwrap();
    input_c.dispatch_event(&keydown_event("Home")).unwrap();
    assert_eq!(
        document.active_element().map(|el| el.id()),
        Some("kn-radio-he1-input-b".to_string())
    );
}

/// ネイティブ `change`（マウスクリック・ネイティブ Space 決定を模す）が
/// 発火すると、グループ内全項目の `data-state` がネイティブ `checked` の
/// 実態へ同期する（`wire_keynav` の change 委譲リスナー）。
#[wasm_bindgen_test]
fn radio_group_native_change_event_syncs_data_state_across_group() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_radio_group_dom(
        &document,
        "kn-radio-change1",
        &[("a", "A", true, false), ("b", "B", false, false)],
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let input_a = document
        .get_element_by_id("kn-radio-change1-input-a")
        .unwrap();
    let input_b = document
        .get_element_by_id("kn-radio-change1-input-b")
        .unwrap();

    // ブラウザのネイティブクリック相当: b を checked にしてから change を発火する
    // （本テストでは name 未設定のため排他選択はブラウザ任せにならず、
    // 手動で a を false にすることでネイティブ挙動を模す）。
    input_b
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .set_checked(true);
    input_a
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .set_checked(false);
    input_b.dispatch_event(&change_event()).unwrap();

    assert_eq!(
        input_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );
    assert_eq!(
        input_a.get_attribute("data-state").as_deref(),
        Some("unchecked")
    );
    let item_b = input_b.parent_element().unwrap();
    assert_eq!(
        item_b.get_attribute("data-state").as_deref(),
        Some("checked")
    );
}

/// XSS 回帰（REQ-1）: 攻撃者制御文字列を含むラベル・`data-value`・`id` を
/// 持つ Menu/RadioGroup に対し highlight 移動・決定・radio 移動を行っても
/// `script` 要素が DOM に生成されないこと。
#[wasm_bindgen_test]
fn menu_and_radio_group_keyboard_navigation_with_attacker_controlled_strings_does_not_inject_script(
) {
    let document = web_sys::window().unwrap().document().unwrap();

    let menu_root = build_menu_dom(
        &document,
        "kn-xss-menu1",
        &[(
            "<script>alert(2)</script>",
            "<script>alert(3)</script>",
            false,
        )],
        true,
        false,
    );
    let _menu_cleanup = RemoveOnDrop(menu_root.clone());
    wire_keynav(menu_root.clone()).expect("wire_keynav must succeed");
    let menu_trigger = document.get_element_by_id("kn-xss-menu1-trigger").unwrap();
    html_element(&menu_trigger).focus().unwrap();
    menu_trigger
        .dispatch_event(&keydown_event("ArrowDown"))
        .unwrap();
    assert!(menu_root.query_selector("script").unwrap().is_none());

    let radio_root = build_radio_group_dom(
        &document,
        "kn-xss-radio1",
        &[
            ("a", "<script>alert(4)</script>", true, false),
            ("b", "B", false, false),
        ],
        None,
    );
    let _radio_cleanup = RemoveOnDrop(radio_root.clone());
    wire_keynav(radio_root.clone()).expect("wire_keynav must succeed");
    let radio_input_a = document.get_element_by_id("kn-xss-radio1-input-a").unwrap();
    html_element(&radio_input_a).focus().unwrap();
    radio_input_a
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(radio_root.query_selector("script").unwrap().is_none());
}
