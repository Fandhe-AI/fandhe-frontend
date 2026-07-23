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
//!    Enter/Space で highlight 項目へ click 合成、Escape で highlight の
//!    みクリア（close 自体は行わない、Bugbot 指摘・イシュー #583 回帰）
//! 9. RadioGroup: Arrow 移動 + 同時 check + `data-state` 4 パーツ同期・循環・
//!    disabled スキップ・orientation 制限・Home/End・`change` 同期
//!
//! XSS 回帰（REQ-1）: 攻撃者制御文字列を持つラベルに対しキー操作・活性化・
//! highlight/選択・typeahead を行っても `script` 要素が DOM に生成されない
//! ことを固定する。
//!
//! 10. typeahead（イシュー #641）: Menu open 時の文字キーでマッチ項目へ
//!     highlight 移動、タイムアウト内の連続入力で絞り込み・タイムアウト後は
//!     新規バッファ、同一文字連打での巡回・disabled スキップ、Select でも
//!     同動作（ラベルは `item-text` 子から解決され indicator 文字が混入
//!     しない）、closed 時の文字キーで open + マッチ項目の初期 highlight、
//!     バッファ有効時の Space はバッファへ追記され決定にならない、Escape
//!     後の再入力は新規バッファから始まる、攻撃者制御ラベルでの XSS 回帰

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::keynav::{wire_keynav, TYPEAHEAD_TIMEOUT_MS};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, EventInit, HtmlElement, HtmlInputElement, KeyboardEvent,
    KeyboardEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// `ms` ミリ秒だけ実時間で待つ（`tooltip_delay_browser.rs::sleep_ms` と
/// 同実装。typeahead バッファのタイムアウト境界（350ms）を実タイマーで
/// 決定的に検証するために使う）。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms)
            .expect("setTimeout must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("setTimeout promise must not reject");
}

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

/// [`build_select_dom`] の亜種。各項目に `[data-part="item-text"]` 子
/// （ラベル）と、その手前に `[data-part="item-indicator"]`（別テキストを
/// 持つ兄弟パーツ）を追加した Select DOM を生成する。typeahead のラベル
/// 解決（`item_label`、イシュー #641）が `item-text` 子を優先し
/// item-indicator のテキストを混入させないことを検証するために使う
/// （`crates/headless-ui/src/select.rs` の anatomy 契約に対応）。
fn build_select_dom_with_item_text(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, &str, bool)],
    open: bool,
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
    if !open {
        content.set_attribute("hidden", "").unwrap();
    }
    for (value, indicator_text, label, disabled) in items {
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
        // indicator を先に置く: `item_label` が item 自身の `text_content()`
        // へ誤ってフォールバックした場合、indicator の文字列も連結されて
        // しまいラベル一致判定を汚染する（本テストが検出したい回帰）。
        let indicator = document.create_element("span").unwrap();
        indicator
            .set_attribute("data-part", "item-indicator")
            .unwrap();
        indicator.set_text_content(Some(indicator_text));
        item.append_child(&indicator).unwrap();

        let text = document.create_element("span").unwrap();
        text.set_attribute("data-part", "item-text").unwrap();
        text.set_text_content(Some(label));
        item.append_child(&text).unwrap();

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
        // 実マークアップのフォーム統合（同一 `name` によるネイティブ排他選択・
        // ブラウザ既定のキーボードグループ化）を再現する（Bugbot 指摘、
        // イシュー #583。`name` 省略下ではブラウザ既定の移動が発生せず
        // orientation 制限の非有効化を検出できなかった）。
        input.set_attribute("name", root_id).unwrap();
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

/// 合成 `keydown` イベント（`bubbles: true, cancelable: true`）を組み立てる。
/// `cancelable: true` により `dispatch_event` の戻り値（`false` ==
/// `prevent_default()` が呼ばれた）でハンドラの prevent_default 呼び出しを
/// 検証できる（イシュー #583 Bugbot 指摘の回帰テスト用）。
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

/// closed の trigger 上で Enter/Space を押した場合も ArrowDown と同じく
/// `click()` が合成され初期 highlight（先頭の非 disabled 項目）が設定される。
/// ネイティブ `<button>` の既定 click 発火に任せた場合、本ハンドラが戻った
/// 後で非同期に click が発火し初期 highlight を設定する機会がないまま open
/// してしまう回帰（Bugbot 指摘、イシュー #583）のテスト。
#[wasm_bindgen_test]
fn menu_closed_enter_and_space_open_via_synthesized_click_and_set_initial_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    for (root_id, key) in [("kn-menu-open-enter", "Enter"), ("kn-menu-open-space", " ")] {
        let root = build_menu_dom(
            &document,
            root_id,
            &[("a", "A", false), ("b", "B", false)],
            false,
            false,
        );
        let _cleanup = RemoveOnDrop(root.clone());

        let content = document
            .get_element_by_id(&format!("{root_id}-content"))
            .unwrap();
        let open_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new({
            let content = content.clone();
            move |_event: Event| {
                let _ = content.remove_attribute("hidden");
            }
        });
        let trigger = document
            .get_element_by_id(&format!("{root_id}-trigger"))
            .unwrap();
        trigger
            .add_event_listener_with_callback("click", open_closure.as_ref().unchecked_ref())
            .unwrap();
        open_closure.forget();

        wire_keynav(root.clone()).expect("wire_keynav must succeed");
        html_element(&trigger).focus().unwrap();

        let not_default_prevented = trigger.dispatch_event(&keydown_event(key)).unwrap();
        assert!(
            !not_default_prevented,
            "closed trigger 上の Enter/Space は prevent_default されるべき"
        );

        assert!(!content.has_attribute("hidden"));
        let item_a = document
            .get_element_by_id(&format!("{root_id}-item-a"))
            .unwrap();
        assert!(
            item_a.has_attribute("data-highlighted"),
            "key={key}: Enter/Space で開いた直後も先頭項目が highlight されるべき"
        );
        assert_eq!(
            content.get_attribute("aria-activedescendant").as_deref(),
            Some(format!("{root_id}-item-a").as_str())
        );
    }
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

    // 既定非循環: 末尾から ArrowDown は highlight 移動としては no-op だが、
    // 開いている間はページスクロール抑止のため prevent_default は呼ばれる
    // （`dispatch_event` は cancelable なイベントで prevent_default される
    // と false を返す。Bugbot 指摘、イシュー #583 の回帰）。
    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        !not_default_prevented,
        "非ループ既定動作で端に到達しても prevent_default が呼ばれるべき"
    );
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

/// 親 Menu の item 収集がネストしたサブメニュー（`trigger-item` が開く子
/// Menu の content）配下の item/trigger-item まで拾ってしまい、親の
/// Arrow/Home/End 操作がスコープ外のサブメニュー項目を移動・highlight して
/// しまう回帰（Bugbot 指摘、イシュー #583）のテスト。
///
/// 親 content 直下に item "a"・trigger-item "sub" を置き、"sub" の子孫に
/// （open 状態の）ネストした子 Menu content（item "x"・"y"）を配置する。
/// `query_selector_all` は subtree 全体を対象にするため、フィルタが無ければ
/// `[a, sub, x, y]` の 4 件が親の highlight 候補に混入する。
#[wasm_bindgen_test]
fn menu_open_arrow_and_end_do_not_reach_into_nested_submenu_items() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-nested1",
        &[("a", "A", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let content = document
        .get_element_by_id("kn-menu-nested1-content")
        .unwrap();

    // 親 content 直下に trigger-item "sub" を追加する。
    let trigger_item = document.create_element("div").unwrap();
    trigger_item.set_attribute("data-scope", "menu").unwrap();
    trigger_item
        .set_attribute("data-part", "trigger-item")
        .unwrap();
    trigger_item
        .set_attribute("id", "kn-menu-nested1-item-sub")
        .unwrap();
    trigger_item.set_text_content(Some("Sub"));
    content.append_child(&trigger_item).unwrap();

    // "sub" の子孫にネストした子 Menu（root/content/item x, y）を配置する
    // （open 状態、`hidden` 属性なし）。
    let nested_root = document.create_element("div").unwrap();
    nested_root.set_attribute("data-scope", "menu").unwrap();
    nested_root.set_attribute("data-part", "root").unwrap();
    let nested_content = document.create_element("div").unwrap();
    nested_content.set_attribute("data-scope", "menu").unwrap();
    nested_content
        .set_attribute("data-part", "content")
        .unwrap();
    nested_content
        .set_attribute("id", "kn-menu-nested1-nested-content")
        .unwrap();
    for value in ["x", "y"] {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "menu").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("id", &format!("kn-menu-nested1-item-{value}"))
            .unwrap();
        nested_content.append_child(&item).unwrap();
    }
    nested_root.append_child(&nested_content).unwrap();
    trigger_item.append_child(&nested_root).unwrap();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-menu-nested1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    let item_a = document
        .get_element_by_id("kn-menu-nested1-item-a")
        .unwrap();
    let item_sub = document
        .get_element_by_id("kn-menu-nested1-item-sub")
        .unwrap();
    let item_x = document
        .get_element_by_id("kn-menu-nested1-item-x")
        .unwrap();
    let item_y = document
        .get_element_by_id("kn-menu-nested1-item-y")
        .unwrap();

    // End は親スコープの末尾（sub）へ。ネスト内の y へは到達しない。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_sub.has_attribute("data-highlighted"));
    assert!(!item_y.has_attribute("data-highlighted"));
    assert!(!item_x.has_attribute("data-highlighted"));

    // Home で親スコープの先頭（a）へ。
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert!(!item_sub.has_attribute("data-highlighted"));

    // ArrowDown で次（sub）。末尾到達のため、更に ArrowDown してもネスト内の
    // x/y へは進まない（親スコープは a, sub の 2 件のみ）。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_sub.has_attribute("data-highlighted"));
    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(!not_default_prevented);
    assert!(item_sub.has_attribute("data-highlighted"));
    assert!(!item_x.has_attribute("data-highlighted"));
    assert!(!item_y.has_attribute("data-highlighted"));
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

/// Menu が open のまま Escape を受けると、`data-highlighted`/
/// `aria-activedescendant` がクリアされる（本モジュールが書き込んだ
/// highlight 表現の後始末のみで、`hidden`/`data-state` の実際の close は
/// 依然として overlay モジュール（#580 統合層）の責務、モジュール doc
/// §Menu/Select 参照）。Bugbot 指摘（イシュー #583）の回帰固定:
/// Escape 後にマウス等で reopen した際、最初の Arrow キーが古い highlight
/// から続かず先頭から開始することを、highlight クリア済みの状態で検証する。
#[wasm_bindgen_test]
fn menu_open_escape_clears_highlight_without_closing() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-menu-esc1",
        &[("a", "A", false), ("b", "B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-menu-esc1-trigger").unwrap();
    let content = document.get_element_by_id("kn-menu-esc1-content").unwrap();
    let item_a = document.get_element_by_id("kn-menu-esc1-item-a").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-menu-esc1-item-a")
    );

    trigger.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(
        !item_a.has_attribute("data-highlighted"),
        "Escape で古い highlight がクリアされるべき"
    );
    assert!(
        content.get_attribute("aria-activedescendant").is_none(),
        "Escape で aria-activedescendant もクリアされるべき"
    );
    // 本モジュールは close 自体（`hidden`/`data-state`）を行わない
    // （overlay モジュールの責務、モジュール doc 参照）。
    assert!(
        !content.has_attribute("hidden"),
        "本モジュールは Escape で content を close しない"
    );

    // 再オープン相当の次の ArrowDown は、古い highlight が残っていないため
    // 先頭（a）から開始する。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
}

/// Select も Menu と同じ Escape highlight クリア契約を共有する
/// （`select_open_arrow_moves_highlight_and_enter_clicks_highlighted_item`
/// と同じ SSR 契約、Bugbot 指摘、イシュー #583）。
#[wasm_bindgen_test]
fn select_open_escape_clears_highlight_without_closing() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_select_dom(
        &document,
        "kn-select-esc1",
        &[("apple", "Apple", false), ("banana", "Banana", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-select-esc1-trigger")
        .unwrap();
    let content = document
        .get_element_by_id("kn-select-esc1-content")
        .unwrap();
    let item_apple = document
        .get_element_by_id("kn-select-esc1-item-apple")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_apple.has_attribute("data-highlighted"));

    trigger.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(!item_apple.has_attribute("data-highlighted"));
    assert!(content.get_attribute("aria-activedescendant").is_none());
    assert!(!content.has_attribute("hidden"));
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

    // orientation により却下される ArrowDown も、同一 `name` によるネイティブ
    // radio グループ化のブラウザ既定移動を抑止するため prevent_default
    // される（`dispatch_event` は cancelable なイベントで prevent_default
    // されると false を返す。Bugbot 指摘、イシュー #583 の回帰）。
    let not_default_prevented = input_a.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        !not_default_prevented,
        "orientation で却下される矢印キーでも prevent_default が呼ばれるべき"
    );
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

// ---------------------------------------------------------------------
// typeahead（Menu/Select 共用、イシュー #641）
// ---------------------------------------------------------------------

/// 検証: open の Menu で文字キーがマッチ項目へ highlight を移動する
/// （`data-highlighted`/`aria-activedescendant` 追随）。
#[wasm_bindgen_test]
fn menu_open_typeahead_moves_highlight_to_matching_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu1",
        &[("a", "Apple", false), ("b", "Banana", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu1-trigger").unwrap();
    let content = document.get_element_by_id("kn-ta-menu1-content").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu1-item-b").unwrap();
    html_element(&trigger).focus().unwrap();

    let not_default_prevented = trigger.dispatch_event(&keydown_event("b")).unwrap();
    assert!(
        !not_default_prevented,
        "typeahead でハンドリングした文字キーは prevent_default されるべき"
    );
    assert!(item_b.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-ta-menu1-item-b")
    );
}

/// 検証: タイムアウト内の連続入力でバッファが絞り込まれ、タイムアウト
/// （[`TYPEAHEAD_TIMEOUT_MS`]）超過後は新規バッファとして再探索する。
#[wasm_bindgen_test]
async fn menu_open_typeahead_buffers_within_timeout_and_resets_after() {
    let document = web_sys::window().unwrap().document().unwrap();
    // "Almond" は "a" にのみ一致し "ap" には一致しない（"Apple" のように
    // 2 文字目が "p" のラベルを選ぶと "a"/"ap" の絞り込み挙動を区別できない
    // ため、意図的に "Al" 始まりを選ぶ）。
    let root = build_menu_dom(
        &document,
        "kn-ta-menu2",
        &[
            ("a", "Almond", false),
            ("b", "Apricot", false),
            ("c", "Banana", false),
        ],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu2-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-ta-menu2-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu2-item-b").unwrap();
    let item_c = document.get_element_by_id("kn-ta-menu2-item-c").unwrap();
    html_element(&trigger).focus().unwrap();

    // "a" → Almond（先頭一致）。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // タイムアウト内に "p" を追記 → バッファ "ap" → Apricot に絞り込む。
    trigger.dispatch_event(&keydown_event("p")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));

    // タイムアウト超過後に "b" → 新規バッファとして Banana を検索する
    // （"apb" ではなく "b" 単独として扱われることの確認）。
    sleep_ms((TYPEAHEAD_TIMEOUT_MS as i32) + 200).await;
    trigger.dispatch_event(&keydown_event("b")).unwrap();
    assert!(item_c.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// 検証: 同一文字の連打で同じ頭文字の項目を巡回し、disabled はスキップする。
#[wasm_bindgen_test]
fn menu_open_typeahead_repeated_char_cycles_and_skips_disabled() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu3",
        &[
            ("a", "Apple", false),
            ("b", "Avocado", true),
            ("c", "Apricot", false),
        ],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu3-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-ta-menu3-item-a").unwrap();
    let item_c = document.get_element_by_id("kn-ta-menu3-item-c").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // "a" を連打 → 同一バッファ "aa" として扱われ、次の "a" 始まり項目
    // （disabled の b をスキップして c）へ巡回する。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_c.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));
}

/// 検証: Select でもラベルは `[data-part="item-text"]` 子から解決され、
/// その手前に置かれた item-indicator のテキストが typeahead マッチを
/// 汚染しないこと（イシュー #641 の受け入れ条件）。
#[wasm_bindgen_test]
fn select_open_typeahead_uses_item_text_label_not_indicator_text() {
    let document = web_sys::window().unwrap().document().unwrap();
    // indicator に "Zephyr"（Z 始まり）を仕込む。`item_label` が誤って
    // indicator のテキストまで拾ってしまうと "b" 入力でも item-a が
    // マッチしてしまう（indicator "Zephyr" は "b" と無関係だが、item 自身の
    // `text_content()` は子孫全体（indicator + item-text）を連結するため
    // "ZephyrApple" のような文字列になり、本来ヒットしないはずの挙動を
    // 誘発しうる）。
    let root = build_select_dom_with_item_text(
        &document,
        "kn-ta-select1",
        &[
            ("a", "Zephyr", "Apple", false),
            ("b", "Zephyr", "Banana", false),
        ],
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-select1-trigger").unwrap();
    let item_b = document.get_element_by_id("kn-ta-select1-item-b").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("b")).unwrap();
    assert!(
        item_b.has_attribute("data-highlighted"),
        "item-text 子のラベル（Banana）でマッチし、indicator（Zephyr）は無視されるべき"
    );
}

/// 検証: closed の trigger 上で文字キーを押すと open + マッチ項目の初期
/// highlight が設定される。
#[wasm_bindgen_test]
fn menu_closed_typeahead_opens_and_sets_matching_initial_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu4",
        &[("a", "Apple", false), ("b", "Banana", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let content = document.get_element_by_id("kn-ta-menu4-content").unwrap();
    let open_closure = Closure::<dyn FnMut(Event)>::new({
        let content = content.clone();
        move |_event: Event| {
            let _ = content.remove_attribute("hidden");
        }
    });
    let trigger = document.get_element_by_id("kn-ta-menu4-trigger").unwrap();
    trigger
        .add_event_listener_with_callback("click", open_closure.as_ref().unchecked_ref())
        .unwrap();
    open_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("b")).unwrap();

    assert!(!content.has_attribute("hidden"));
    let item_b = document.get_element_by_id("kn-ta-menu4-item-b").unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-ta-menu4-item-b")
    );
}

/// 検証: バッファ有効時の Space はバッファへ追記され決定（click 合成）に
/// ならない。バッファが無効（空）の Space は従来通り決定として扱われる。
#[wasm_bindgen_test]
fn menu_open_space_buffers_when_typeahead_active_but_activates_when_empty() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu5",
        &[("a", "Apple", false), ("b", "A B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu5-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-ta-menu5-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu5-item-b").unwrap();
    html_element(&trigger).focus().unwrap();

    // バッファ空の Space は従来通り決定（highlight 不在のため no-op のまま、
    // click は合成されない）。副作用が無いことのみ確認する。
    let not_default_prevented = trigger.dispatch_event(&keydown_event(" ")).unwrap();
    assert!(
        !not_default_prevented,
        "Space は開いている間常に prevent_default される"
    );
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));

    // "a" で Apple を highlight。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // タイムアウト内の Space はバッファへ追記され（"a "）、"A B" にマッチして
    // highlight が移動する（決定にはならない）。
    trigger.dispatch_event(&keydown_event(" ")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));
}

/// 検証: Escape で highlight クリアに加えて typeahead バッファもリセット
/// される（再入力は新規バッファから始まる）。
#[wasm_bindgen_test]
fn menu_open_escape_resets_typeahead_buffer() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu6",
        &[("a", "Apple", false), ("b", "Apricot", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu6-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-ta-menu6-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu6-item-b").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    trigger.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));

    // Escape 後に "p" を単独入力 → もし旧バッファ "a" が残っていれば "ap"
    // として Apricot にマッチしてしまうが、リセットされていれば新規バッファ
    // "p" は "p" 始まりの項目が無いためマッチ無し（no-op）のはず。
    trigger.dispatch_event(&keydown_event("p")).unwrap();
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// XSS 回帰（REQ-1、イシュー #641）: 攻撃者制御文字列を含むラベルに対し
/// typeahead（open/closed 双方）を行っても `script` 要素が DOM に生成
/// されないこと。
#[wasm_bindgen_test]
fn menu_typeahead_with_attacker_controlled_label_does_not_inject_script() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-xss1",
        &[("a", "<script>alert(5)</script>", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-xss1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("<")).unwrap();
    assert!(root.query_selector("script").unwrap().is_none());

    // closed 経路の typeahead も同様に検証する。
    let closed_root = build_menu_dom(
        &document,
        "kn-ta-xss2",
        &[("a", "<script>alert(6)</script>", false)],
        false,
        false,
    );
    let _closed_cleanup = RemoveOnDrop(closed_root.clone());
    let closed_content = document.get_element_by_id("kn-ta-xss2-content").unwrap();
    let open_closure = Closure::<dyn FnMut(Event)>::new({
        let content = closed_content.clone();
        move |_event: Event| {
            let _ = content.remove_attribute("hidden");
        }
    });
    let closed_trigger = document.get_element_by_id("kn-ta-xss2-trigger").unwrap();
    closed_trigger
        .add_event_listener_with_callback("click", open_closure.as_ref().unchecked_ref())
        .unwrap();
    open_closure.forget();
    wire_keynav(closed_root.clone()).expect("wire_keynav must succeed");
    html_element(&closed_trigger).focus().unwrap();
    closed_trigger.dispatch_event(&keydown_event("<")).unwrap();
    assert!(closed_root.query_selector("script").unwrap().is_none());
}
