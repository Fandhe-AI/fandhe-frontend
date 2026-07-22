//! `fandhe_frontend_wasm_full::keynav::wire_keynav`（Tabs/Accordion のキーボード
//! 操作・イシュー #582、親 #581）の実ブラウザ統合テスト
//! （`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/keynav_native.rs`（native）は純粋層（`tabs_next_index`/
//! `accordion_next_index`）までを検証済みである。本ファイルはその先、
//! `wire_keynav` が実 DOM（headless Chromium）上でキーボード委譲・
//! roving tabindex 更新・フォーカス移動・活性化（automatic/manual）を
//! 正しく反映することを検証する。
//!
//! DOM 構造は `crates/headless-ui/src/tabs.rs`/`accordion.rs` の SSR 出力
//! 契約（`data-scope`/`data-part`/`aria-*`/`data-state`/`tabindex` 等）を
//! 手組みで再現する（本クレートは `fandhe-frontend-headless-ui` に依存しない
//! ため、実際の `tabs()`/`accordion` 関数は呼べない。属性契約の記述は
//! それぞれのモジュール doc・スナップショットテストと一致させている）。
//!
//! # 検証内容（実装計画 §6 の検証項目 1〜7 に対応）
//!
//! 1. horizontal: ArrowRight/ArrowLeft でフォーカス移動 + roving tabindex 更新
//! 2. vertical: ArrowDown/ArrowUp で同上（horizontal では no-op）
//! 3. Home/End で先頭/末尾の非 disabled trigger へ移動・disabled スキップ
//! 4. `data-loop-focus="false"` で端 no-op
//! 5. automatic: フォーカス移動と同時に `aria-selected`/`data-state`/`hidden` 反映
//! 6. manual: Arrow ではパネル不変・クリック（Enter/Space 相当）で活性化
//! 7. Accordion: ArrowDown/ArrowUp/Home/End のフォーカス移動（非循環・disabled スキップ）
//!
//! XSS 回帰（REQ-1）: 攻撃者制御文字列を持つラベルに対しキー操作・活性化を
//! 行っても `script` 要素が DOM に生成されないことを固定する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::keynav::wire_keynav;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement, KeyboardEvent, KeyboardEventInit};

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
