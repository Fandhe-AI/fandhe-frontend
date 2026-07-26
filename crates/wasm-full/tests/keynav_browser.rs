//! `fandhe_frontend_wasm_full::keynav::wire_keynav`（Tabs/Accordion/Menu/
//! Select/RadioGroup/Combobox のキーボード操作・イシュー #582・#583・
//! #1071（Combobox）、親 #581）の実ブラウザ統合テスト
//! （`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/keynav_native.rs`（native）は純粋層（`tabs_next_index`/
//! `accordion_next_index`/`highlight_next_index`/`radio_next_index`）までを
//! 検証済みである。本ファイルはその先、`wire_keynav` が実 DOM
//! （headless Chromium）上でキーボード委譲・roving tabindex 更新・フォーカス
//! 移動・活性化（automatic/manual）・highlight/選択・radio チェック同期を
//! 正しく反映することを検証する。
//!
//! DOM 構造は `crates/headless-ui/src/tabs.rs`/`accordion.rs`/`menu.rs`/
//! `select.rs`/`radio_group.rs`/`combobox.rs` の SSR 出力契約
//! （`data-scope`/`data-part`/`aria-*`/`data-state`/`tabindex` 等）を
//! 手組みで再現する（`fandhe-frontend-wasm-full` は `[dependencies]` に
//! `fandhe-frontend-headless-ui` を持つが、`wasm32` ターゲットの
//! `wasm-bindgen-test` 実行環境では実 DOM 構築に `web_sys::Element` API を
//! 直接使う必要があり、`headless-ui` の各パーツ関数〔`fandhe_frontend_core::Node`
//! を返す〕をそのまま呼んで DOM 化する経路を持たないため、引き続き手組みで
//! 再現する。属性契約の記述はそれぞれのモジュール doc・スナップショット
//! テスト（native の `headless_wiring.rs` 等）と一致させている）。
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
//! 11. Listbox（イシュー #1070）: content 自身が keydown ターゲットであり
//!     Arrow（既定 Vertical、`data-orientation="horizontal"` で軸切替）/
//!     Home/End で highlight・`aria-activedescendant` を更新、既定非循環・
//!     `data-loop-focus="true"` で循環、disabled スキップ、typeahead は
//!     Menu/Select と同じ実装を再利用、Enter/Space（バッファ非活性時）は
//!     highlight 中の非 disabled 項目へ click 合成、Escape は Menu/Select と
//!     非対称に highlight を維持したまま typeahead バッファのみリセット
//!     （`prevent_default` しない）、攻撃者制御ラベルでの XSS 回帰

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

/// `parent_content` 直下へ `trigger-item`（サブメニューを開く menu item、
/// `crates/headless-ui/src/menu.rs::trigger_item` の SSR 契約を再現）と、
/// その子孫にネストした子 Menu（`root`/`content`、`sub_items`）を追加する
/// （イシュー #662）。`menu_open_arrow_and_end_do_not_reach_into_nested_submenu_items`
/// と同じ「trigger-item の子孫に子 Menu インスタンスの root/content を入れ子
/// 配置する」構造契約を、ArrowRight/ArrowLeft サブメニューナビゲーション
/// テスト向けに再利用可能な形へ切り出したもの。`id_prefix` から
/// `{id_prefix}-item-{trigger_item_value}`（trigger-item 自身）・
/// `{id_prefix}-sub-content`（子 content）・`{id_prefix}-sub-item-{value}`
/// （子 item）の id を組み立てる。戻り値は `(trigger_item, sub_content)`。
#[allow(clippy::too_many_arguments)]
fn append_trigger_item_with_submenu(
    document: &Document,
    parent_content: &Element,
    id_prefix: &str,
    trigger_item_value: &str,
    trigger_item_label: &str,
    trigger_item_disabled: bool,
    sub_items: &[(&str, &str, bool)],
    sub_open: bool,
) -> (Element, Element) {
    let trigger_item = document.create_element("div").unwrap();
    trigger_item.set_attribute("data-scope", "menu").unwrap();
    trigger_item
        .set_attribute("data-part", "trigger-item")
        .unwrap();
    trigger_item.set_attribute("role", "menuitem").unwrap();
    trigger_item.set_attribute("aria-haspopup", "menu").unwrap();
    let trigger_item_id = format!("{id_prefix}-item-{trigger_item_value}");
    trigger_item.set_attribute("id", &trigger_item_id).unwrap();
    let sub_content_id = format!("{id_prefix}-sub-content");
    trigger_item
        .set_attribute("aria-controls", &sub_content_id)
        .unwrap();
    trigger_item
        .set_attribute("aria-expanded", if sub_open { "true" } else { "false" })
        .unwrap();
    trigger_item
        .set_attribute("data-state", if sub_open { "open" } else { "closed" })
        .unwrap();
    if trigger_item_disabled {
        trigger_item.set_attribute("aria-disabled", "true").unwrap();
        trigger_item.set_attribute("data-disabled", "").unwrap();
    }
    trigger_item.set_text_content(Some(trigger_item_label));
    parent_content.append_child(&trigger_item).unwrap();

    let sub_root = document.create_element("div").unwrap();
    sub_root.set_attribute("data-scope", "menu").unwrap();
    sub_root.set_attribute("data-part", "root").unwrap();
    let sub_content = document.create_element("div").unwrap();
    sub_content.set_attribute("data-scope", "menu").unwrap();
    sub_content.set_attribute("data-part", "content").unwrap();
    sub_content.set_attribute("id", &sub_content_id).unwrap();
    sub_content.set_attribute("role", "menu").unwrap();
    if !sub_open {
        sub_content.set_attribute("hidden", "").unwrap();
    }
    for (value, label, disabled) in sub_items {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "menu").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("role", "menuitem").unwrap();
        item.set_attribute("data-value", value).unwrap();
        item.set_attribute("id", &format!("{id_prefix}-sub-item-{value}"))
            .unwrap();
        if *disabled {
            item.set_attribute("aria-disabled", "true").unwrap();
            item.set_attribute("data-disabled", "").unwrap();
        }
        item.set_text_content(Some(label));
        sub_content.append_child(&item).unwrap();
    }
    sub_root.append_child(&sub_content).unwrap();
    trigger_item.append_child(&sub_root).unwrap();

    (trigger_item, sub_content)
}

/// `trigger_like`（`trigger`/`trigger-item`）への合成 `click` を受けて
/// `content` の `hidden`/`data-state`（+ `trigger_like` 自身の
/// `aria-expanded`/`data-state`）をトグルする模擬リスナーを配線する
/// （イシュー #662）。実アプリでは click → `data-action` →
/// `fandhe_frontend_interactive::dispatch("toggle")` → 再描画がこの開閉を
/// 担うが（モジュール doc §設計）、本ファイルは `fandhe-frontend-interactive`
/// を配線しない手組み DOM テストのため、既存の
/// `menu_closed_arrow_down_opens_via_synthesized_click_and_sets_initial_highlight`
/// 等が使う open 専用の模擬リスナーをトグル可能に拡張した共通版として使う。
fn wire_toggle_listener(trigger_like: &Element, content: &Element) {
    let closure = Closure::<dyn FnMut(Event)>::new({
        let trigger_like = trigger_like.clone();
        let content = content.clone();
        move |event: Event| {
            // `trigger_like` の子孫（サブメニュー項目・ネストした子
            // trigger-item）への合成 click は素の DOM bubble に乗って
            // `trigger_like` 自身にも届いてしまう。実アプリでは 1 個の
            // 委譲リスナーが `closest("[data-action]")` で「click された
            // 要素から見て最も近い一致」のみを処理する（子孫に別の
            // data-action があればそちらが優先される）のに対し、本テストの
            // 模擬リスナーは trigger-like ごとに個別配線するため、
            // `event.target()` が自分自身と一致する場合のみ処理する
            // ことで同じ「最近傍のみ反応する」性質を再現する。これを
            // 怠ると「子孫項目の Enter/click 合成が祖先 trigger-item の
            // サブメニューまで意図せず閉じてしまう」誤動作を起こす
            // （イシュー #662 のサブメニュー内 Enter/ネスト 2 段テストで
            // 顕在化）。
            let is_self_click = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.is_same_node(Some(&trigger_like)));
            if !is_self_click {
                return;
            }
            if content.has_attribute("hidden") {
                let _ = content.remove_attribute("hidden");
                let _ = content.set_attribute("data-state", "open");
                let _ = trigger_like.set_attribute("aria-expanded", "true");
                let _ = trigger_like.set_attribute("data-state", "open");
            } else {
                let _ = content.set_attribute("hidden", "");
                let _ = content.set_attribute("data-state", "closed");
                let _ = trigger_like.set_attribute("aria-expanded", "false");
                let _ = trigger_like.set_attribute("data-state", "closed");
            }
        }
    });
    trigger_like
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

/// [`build_menu_dom`] の親 content 直下へ 1 段のサブメニュー
/// （`trigger-item` + 子 Menu content）を追加した DOM を構築する
/// （イシュー #662）。[`append_trigger_item_with_submenu`]・
/// [`wire_toggle_listener`] を組み合わせ、トリガー・trigger-item いずれの
/// クリックでも模擬トグルが効くようにする。戻り値は
/// `(root, trigger_item, sub_content)`。
#[allow(clippy::too_many_arguments)]
fn build_submenu_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool)],
    trigger_item_value: &str,
    trigger_item_label: &str,
    trigger_item_disabled: bool,
    sub_items: &[(&str, &str, bool)],
    open: bool,
    sub_open: bool,
) -> (Element, Element, Element) {
    let root = build_menu_dom(document, root_id, items, open, false);
    let content = document
        .get_element_by_id(&format!("{root_id}-content"))
        .unwrap();
    let (trigger_item, sub_content) = append_trigger_item_with_submenu(
        document,
        &content,
        root_id,
        trigger_item_value,
        trigger_item_label,
        trigger_item_disabled,
        sub_items,
        sub_open,
    );
    wire_toggle_listener(&trigger_item, &sub_content);
    (root, trigger_item, sub_content)
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
    // 実際の SSR 既定（`crates/headless-ui/src/menu.rs::content` は closed の
    // とき `hidden` 存在属性を付与する契約）に合わせ closed 状態で配置する
    // （イシュー #662: `hidden` なしにすると「サブメニューが開いている」と
    // 誤認され、本テストの意図（highlight/収集のスコープ外判定）とは別に
    // アクティブ content 解決が子孫へ降下してしまい、以降の Home/End/
    // ArrowDown アサーションが変わってしまう）。
    let nested_root = document.create_element("div").unwrap();
    nested_root.set_attribute("data-scope", "menu").unwrap();
    nested_root.set_attribute("data-part", "root").unwrap();
    let nested_content = document.create_element("div").unwrap();
    nested_content.set_attribute("data-scope", "menu").unwrap();
    nested_content
        .set_attribute("data-part", "content")
        .unwrap();
    nested_content.set_attribute("hidden", "").unwrap();
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

/// 検証（Bugbot 指摘、イシュー #641）: Arrow ナビゲーションでも highlight
/// クリアに加えて typeahead バッファがリセットされる。リセットされないと
/// `TYPEAHEAD_TIMEOUT_MS` 以内のナビゲーション後の単独文字入力が古い
/// バッファへ追記され、誤ったクエリで検索してしまう。
#[wasm_bindgen_test]
fn menu_open_arrow_navigation_resets_typeahead_buffer() {
    let document = web_sys::window().unwrap().document().unwrap();
    // "Avocado" は "a" 単独に一致するが "ap" には一致しない
    // （"Apple" のように 2 文字目が "p" のラベルを先頭項目に選ぶと、旧
    // バッファ "a" と新規入力 "p" が連結した "ap" が先頭項目自身にも
    // 前方一致してしまい、リセット有無を区別できなくなるため、意図的に
    // "Av" 始まりを選ぶ）。"Apricot" のみが "ap" に一致する。
    let root = build_menu_dom(
        &document,
        "kn-ta-menu7",
        &[
            ("a", "Avocado", false),
            ("b", "Banana", false),
            ("c", "Apricot", false),
        ],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-ta-menu7-trigger").unwrap();
    let item_a = document.get_element_by_id("kn-ta-menu7-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu7-item-b").unwrap();
    let item_c = document.get_element_by_id("kn-ta-menu7-item-c").unwrap();
    html_element(&trigger).focus().unwrap();

    // "a" で Avocado（先頭一致）を highlight。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // ArrowDown で Banana へ highlight を移動。バッファがリセットされる
    // べきタイミング。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));

    // ナビゲーション後に "p" を単独入力 → リセットされていれば新規バッファ
    // "p" はどの項目にも前方一致せずマッチ無し（highlight は Banana の
    // まま）。もし旧バッファ "a" が残っていれば "ap" として Apricot に
    // マッチし、ナビゲーション後の highlight（Banana）から外れて Apricot
    // へ移動してしまう（バグ再現時の誤動作）。
    trigger.dispatch_event(&keydown_event("p")).unwrap();
    assert!(
        item_b.has_attribute("data-highlighted"),
        "ナビゲーション後の単独入力は新規バッファとして扱われマッチ無しのはず"
    );
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(!item_c.has_attribute("data-highlighted"));
}

/// 検証（Bugbot 指摘、イシュー #641）: Enter による選択確定後も typeahead
/// バッファがリセットされる。
#[wasm_bindgen_test]
fn menu_open_enter_activation_resets_typeahead_buffer() {
    let document = web_sys::window().unwrap().document().unwrap();
    // "Avocado" は "a" 単独に一致するが "ap" には一致しない（理由は
    // `menu_open_arrow_navigation_resets_typeahead_buffer` のコメント参照）。
    // "Apricot" のみが "ap" に一致する。
    let root = build_menu_dom(
        &document,
        "kn-ta-menu8",
        &[("a", "Avocado", false), ("b", "Apricot", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let item_a = document.get_element_by_id("kn-ta-menu8-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu8-item-b").unwrap();
    let item_a_for_closure = item_a.clone();
    let click_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |_e| {
        let _ = item_a_for_closure.set_attribute("data-clicked", "");
    });
    item_a
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    click_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document.get_element_by_id("kn-ta-menu8-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    // "a" で Avocado（先頭一致）を highlight。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // Enter で確定（highlight 自体は activate_highlighted_item では
    // 変更されないが、typeahead バッファはリセットされるべき）。
    trigger.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_a.has_attribute("data-clicked"));

    // 確定後に "p" を単独入力 → もし旧バッファ "a" が残っていれば "ap" と
    // なり Apricot（item_b）にマッチしてしまうが、リセットされていれば
    // 新規バッファ "p" はどの項目にも前方一致せずマッチ無し（highlight は
    // Avocado のまま変化しない）。
    trigger.dispatch_event(&keydown_event("p")).unwrap();
    assert!(
        item_a.has_attribute("data-highlighted"),
        "Enter 確定後の単独入力は新規バッファとして扱われ Avocado のままのはず"
    );
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// 検証（Bugbot 指摘、イシュー #641）: Space による選択確定後も typeahead
/// バッファがリセットされる（Enter と同様の契約）。
///
/// 注: Space の決定分岐（`" " if !buffer_active`）はバッファが既に
/// タイムアウトで非アクティブな場合のみ到達するため（アクティブ中は
/// `_ if is_typeahead_key(...)` 分岐で継続 typeahead として扱われる）、
/// この分岐に到達した時点で自然経過時間により以降の入力は既に新規
/// バッファ扱いになる。つまり本テストは `typeahead.reset()` の有無を
/// 単独では区別できない（Enter と異なり、この経路のリセットは常に
/// no-op 上の防御的な契約統一であり、Arrow/Enter 分と異なり回帰を単独
/// 検出するテストにはならない）。ここでは Space 確定後も正しく動作し
/// 続けること（後続の typeahead 検索が壊れないこと）のみを確認する。
#[wasm_bindgen_test]
async fn menu_open_space_activation_keeps_typeahead_usable_afterward() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-ta-menu9",
        &[("a", "Avocado", false), ("b", "Apricot", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let item_a = document.get_element_by_id("kn-ta-menu9-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-ta-menu9-item-b").unwrap();
    let item_a_for_closure = item_a.clone();
    let click_closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(move |_e| {
        let _ = item_a_for_closure.set_attribute("data-clicked", "");
    });
    item_a
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    click_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document.get_element_by_id("kn-ta-menu9-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    // "a" で Avocado（先頭一致）を highlight。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // バッファのタイムアウトを待ってから Space → 決定分岐
    // （`!buffer_active`）を通り Avocado が確定する。
    sleep_ms((TYPEAHEAD_TIMEOUT_MS as i32) + 200).await;
    trigger.dispatch_event(&keydown_event(" ")).unwrap();
    assert!(item_a.has_attribute("data-clicked"));

    // 確定後の新規入力 "p" はどの項目にも前方一致しない（"Avocado"/
    // "Apricot" いずれも先頭は "a"）ため no-op のまま（highlight は
    // Avocado のまま変化しない）。バッファが壊れた異常な状態のまま残って
    // いないことの確認。
    trigger.dispatch_event(&keydown_event("p")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
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

// ---------------------------------------------------------------------
// サブメニュー（`trigger-item`）の ArrowRight/ArrowLeft 開閉ナビゲーション
// （イシュー #662）。[`build_submenu_dom`]/[`append_trigger_item_with_submenu`]/
// [`wire_toggle_listener`] を使う。
// ---------------------------------------------------------------------

/// ArrowRight: highlight 中の trigger-item のサブメニューが展開され、先頭の
/// 非 disabled 項目が highlight される（受け入れ条件 1）。
#[wasm_bindgen_test]
fn menu_open_arrow_right_expands_submenu_and_highlights_first_enabled_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-open1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-open1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープは [a, sub] の 2 件。End で末尾（trigger-item "sub"）へ。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    let not_default_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        !not_default_prevented,
        "展開する ArrowRight は prevent_default されるべき"
    );

    assert!(
        !sub_content.has_attribute("hidden"),
        "サブメニューが展開されるべき"
    );
    let item_x = document
        .get_element_by_id("kn-sub-open1-sub-item-x")
        .unwrap();
    assert!(
        item_x.has_attribute("data-highlighted"),
        "展開直後は先頭の非 disabled 項目が highlight されるべき"
    );
    assert_eq!(
        sub_content
            .get_attribute("aria-activedescendant")
            .as_deref(),
        Some("kn-sub-open1-sub-item-x")
    );
    // 親の highlight（trigger-item 上）は展開後も維持される（アクティブ
    // content チェーンが DOM のみから再構成できる設計、モジュール doc 参照）。
    assert!(trigger_item.has_attribute("data-highlighted"));
}

/// ArrowRight: `trigger-item` に `id` が無い場合でも、サブメニューが開いた
/// うえで先頭の非 disabled 項目へ highlight が移るべき（Bugbot 指摘
/// "Missing id skips submenu entry"、イシュー #662 PR #674）。`headless-ui`
/// は `trigger_item` の `id` を anatomy 上 optional としており、`id` が
/// 無いことを理由に `open_submenu_and_focus_first_item` が highlight 移動を
/// 一切行わず return してしまうと、サブメニューは開くのにハイライトが入ら
/// ないという以前の不具合（"Enter opens submenu without entering" 相当）が
/// id-less な trigger-item についてのみ再発してしまう。
#[wasm_bindgen_test]
fn menu_open_arrow_right_expands_submenu_and_highlights_first_item_without_trigger_item_id() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-noid1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    // headless-ui の anatomy 上 `trigger_item` の `id` は optional。ここで
    // 意図的に取り除き、`document.get_element_by_id` による再解決手段が
    // 無い状態を再現する。
    trigger_item.remove_attribute("id").unwrap();
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-noid1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープは [a, sub] の 2 件。End で末尾（trigger-item "sub"）へ。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(
        !sub_content.has_attribute("hidden"),
        "id が無くてもサブメニューは展開されるべき"
    );
    let item_x = document
        .get_element_by_id("kn-sub-noid1-sub-item-x")
        .unwrap();
    assert!(
        item_x.has_attribute("data-highlighted"),
        "id-less な trigger-item でも展開直後は先頭の非 disabled 項目が \
         highlight されるべき"
    );
    assert_eq!(
        sub_content
            .get_attribute("aria-activedescendant")
            .as_deref(),
        Some("kn-sub-noid1-sub-item-x")
    );
}

/// ArrowRight: 先頭項目が disabled のときは次の非 disabled 項目へ
/// フォールバックする。
#[wasm_bindgen_test]
fn menu_open_arrow_right_skips_disabled_first_submenu_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-open2",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", true), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-open2-trigger").unwrap();
    html_element(&trigger).focus().unwrap();
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(!sub_content.has_attribute("hidden"));
    let item_x = document
        .get_element_by_id("kn-sub-open2-sub-item-x")
        .unwrap();
    let item_y = document
        .get_element_by_id("kn-sub-open2-sub-item-y")
        .unwrap();
    assert!(!item_x.has_attribute("data-highlighted"));
    assert!(item_y.has_attribute("data-highlighted"));
}

/// ArrowRight: highlight 中の項目が通常 item（`trigger-item` ではない）
/// のときは no-op（`prevent_default` しない、受け入れ条件 2 系の一部）。
#[wasm_bindgen_test]
fn menu_open_arrow_right_on_regular_item_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, _trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-noop1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-noop1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();
    // Home で通常 item "a" を highlight する（trigger-item ではない）。
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    let item_a = document.get_element_by_id("kn-sub-noop1-item-a").unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    let not_default_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        not_default_prevented,
        "trigger-item でない項目上の ArrowRight は prevent_default されるべきでない"
    );
    assert!(
        sub_content.has_attribute("hidden"),
        "サブメニューは閉じたまま"
    );
}

/// ArrowRight: highlight 中の trigger-item が disabled のときは no-op。
#[wasm_bindgen_test]
fn menu_open_arrow_right_on_disabled_trigger_item_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-noop2",
        &[("a", "A", false)],
        "sub",
        "Sub",
        true,
        &[("x", "X", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-noop2-trigger").unwrap();
    html_element(&trigger).focus().unwrap();
    // disabled trigger-item は End 等の通常ナビゲーションでは（disabled
    // スキップ設計により）そもそも highlight されない。本テストは
    // 「highlight 中の項目が disabled だった場合」の fail-closed no-op を
    // 検証する防御的なケースのため、highlight を直接 DOM へ設定する。
    let content = document.get_element_by_id("kn-sub-noop2-content").unwrap();
    trigger_item.set_attribute("data-highlighted", "").unwrap();
    content
        .set_attribute(
            "aria-activedescendant",
            &trigger_item.get_attribute("id").unwrap(),
        )
        .unwrap();

    let not_default_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        not_default_prevented,
        "disabled trigger-item 上の ArrowRight は prevent_default されるべきでない"
    );
    assert!(sub_content.has_attribute("hidden"));
}

/// ArrowRight: Select（`data-scope="select"`）には `trigger-item` が存在
/// せず、highlight 中の item がセレクタ不一致となるため自然に no-op となる。
#[wasm_bindgen_test]
fn select_open_arrow_right_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_select_dom(
        &document,
        "kn-select-noop1",
        &[("a", "A", false), ("b", "B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-select-noop1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    let item_a = document
        .get_element_by_id("kn-select-noop1-item-a")
        .unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    let not_default_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(not_default_prevented);
    assert!(item_a.has_attribute("data-highlighted"));
}

/// ArrowRight: click 駆動の再レンダーで親 `trigger-item` 自身の
/// `data-highlighted`/親 content の `aria-activedescendant` が失われても、
/// id ベースの再解決で親チェーンの highlight が復帰する（Bugbot 指摘
/// "ArrowRight drops parent chain highlight"、イシュー #662 PR #674）。
/// `resolve_active_content` は open chain 再構築にこの親 highlight を必要と
/// するため、これが失われたままだと以降 `ArrowLeft` で閉じられなくなる。
#[wasm_bindgen_test]
fn menu_open_arrow_right_restores_parent_highlight_after_click_driven_rerender_clears_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-rerender1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    // click 駆動の再レンダーをシミュレートする: `wire_toggle_listener` の
    // hidden トグルに続けて、実運用の再レンダーが親 trigger-item の
    // `data-highlighted`/親 content の `aria-activedescendant` を洗い流す
    // ケースを模したリスナーを追加登録する。
    let content = document
        .get_element_by_id("kn-sub-rerender1-content")
        .unwrap();
    let rerender_closure = Closure::<dyn FnMut(Event)>::new({
        let trigger_item = trigger_item.clone();
        let content = content.clone();
        move |_e: Event| {
            let _ = trigger_item.remove_attribute("data-highlighted");
            let _ = content.remove_attribute("aria-activedescendant");
        }
    });
    trigger_item
        .add_event_listener_with_callback("click", rerender_closure.as_ref().unchecked_ref())
        .unwrap();
    rerender_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document
        .get_element_by_id("kn-sub-rerender1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(
        !sub_content.has_attribute("hidden"),
        "サブメニューが展開されるべき"
    );
    assert!(
        trigger_item.has_attribute("data-highlighted"),
        "click 駆動の再レンダーで失われても親 trigger-item の highlight は復帰するべき"
    );
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        trigger_item.get_attribute("id").as_deref()
    );
}

/// Enter: highlight 中の `trigger-item` へ Enter を押すとサブメニューが
/// 展開され、`ArrowRight` と同様にハイライトが子メニューの先頭非 disabled
/// 項目へ移る（Bugbot 指摘 "Enter opens submenu without entering"、イシュー
/// #662 PR #674）。従来は `click()` のみを合成しハイライトを親アイテムに
/// 残したままだったため、次のキー操作が APG のサブメニュー活性化挙動と
/// 一致しなかった。
#[wasm_bindgen_test]
fn menu_open_enter_on_trigger_item_expands_submenu_and_moves_highlight_into_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-enter1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-enter1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    trigger.dispatch_event(&keydown_event("Enter")).unwrap();

    assert!(
        !sub_content.has_attribute("hidden"),
        "Enter でサブメニューが展開されるべき"
    );
    let item_x = document
        .get_element_by_id("kn-sub-enter1-sub-item-x")
        .unwrap();
    assert!(
        item_x.has_attribute("data-highlighted"),
        "Enter は展開後、サブメニュー先頭項目へ highlight を移すべき"
    );
    assert_eq!(
        sub_content
            .get_attribute("aria-activedescendant")
            .as_deref(),
        Some("kn-sub-enter1-sub-item-x")
    );
    assert!(
        trigger_item.has_attribute("data-highlighted"),
        "親 trigger-item の highlight も維持されるべき"
    );
}

/// Space（バッファ無効時）: Enter と同様にサブメニューを展開しハイライトを
/// 子メニューの先頭項目へ移す（イシュー #662 PR #674、Enter との対称性）。
#[wasm_bindgen_test]
fn menu_open_space_on_trigger_item_expands_submenu_and_moves_highlight_into_it() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-space1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-space1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    trigger.dispatch_event(&keydown_event(" ")).unwrap();

    assert!(
        !sub_content.has_attribute("hidden"),
        "Space でサブメニューが展開されるべき"
    );
    let item_x = document
        .get_element_by_id("kn-sub-space1-sub-item-x")
        .unwrap();
    assert!(
        item_x.has_attribute("data-highlighted"),
        "Space は展開後、サブメニュー先頭項目へ highlight を移すべき"
    );
    assert!(
        trigger_item.has_attribute("data-highlighted"),
        "親 trigger-item の highlight も維持されるべき"
    );
}

/// 展開後、ArrowDown/ArrowUp/Home/End はサブメニュー content 内で
/// highlight を移動し、親 content の highlight（trigger-item 上）は不変。
/// Enter はサブメニュー項目へ click 合成する。
#[wasm_bindgen_test]
fn menu_open_submenu_navigation_operates_within_active_content_and_preserves_parent_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-nav1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let item_x = document
        .get_element_by_id("kn-sub-nav1-sub-item-x")
        .unwrap();
    let item_y = document
        .get_element_by_id("kn-sub-nav1-sub-item-y")
        .unwrap();
    let click_closure = Closure::<dyn FnMut(Event)>::new({
        let item_y = item_y.clone();
        move |_e: Event| {
            let _ = item_y.set_attribute("data-clicked", "");
        }
    });
    item_y
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    click_closure.forget();

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document.get_element_by_id("kn-sub-nav1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(item_x.has_attribute("data-highlighted"));

    // ArrowDown はサブメニュー内で次（y）へ移動し、親の trigger-item
    // highlight は変わらない。
    trigger.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(!item_x.has_attribute("data-highlighted"));
    assert!(item_y.has_attribute("data-highlighted"));
    assert!(trigger_item.has_attribute("data-highlighted"));

    // Enter でサブメニュー項目（y）へ click が合成される。
    trigger.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_y.has_attribute("data-clicked"));

    // Home でサブメニュー先頭（x）へ戻る。
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(item_x.has_attribute("data-highlighted"));
    assert!(!item_y.has_attribute("data-highlighted"));

    assert!(!sub_content.has_attribute("hidden"));
}

/// ArrowLeft: サブメニューを閉じ、highlight を親 trigger-item へ復帰させる
/// （受け入れ条件 2）。
#[wasm_bindgen_test]
fn menu_open_arrow_left_closes_submenu_and_restores_parent_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-close1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-close1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    let item_x = document
        .get_element_by_id("kn-sub-close1-sub-item-x")
        .unwrap();
    assert!(!sub_content.has_attribute("hidden"));
    assert!(item_x.has_attribute("data-highlighted"));

    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(
        !not_default_prevented,
        "サブメニュー内での ArrowLeft は prevent_default されるべき"
    );

    assert!(
        sub_content.has_attribute("hidden"),
        "サブメニューが閉じるべき"
    );
    assert!(
        !item_x.has_attribute("data-highlighted"),
        "閉じた後サブメニュー項目の highlight は残らないべき"
    );
    assert!(
        trigger_item.has_attribute("data-highlighted"),
        "閉じた後 highlight は親 trigger-item へ復帰するべき"
    );
    let content = document.get_element_by_id("kn-sub-close1-content").unwrap();
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-sub-close1-item-sub")
    );
}

/// ArrowLeft: `trigger_item` に `id` が無い場合でも、click 前に保持していた
/// ノードを `is_same_node` で照合してサブメニューを閉じたうえで親
/// trigger-item へ highlight を復帰させるべき（Bugbot 指摘 "ArrowLeft still
/// requires trigger id"、イシュー #662 PR #674）。`open_submenu_and_focus_
/// first_item`（ArrowRight/Enter/Space 側）は id なし trigger-item を
/// `is_same_node` ベースの fallback で救済済みだが、ArrowLeft 側のみ `id`
/// 欠落を理由に親 highlight 復帰をスキップしていた不整合の回帰テスト。
#[wasm_bindgen_test]
fn menu_open_arrow_left_closes_submenu_and_restores_parent_highlight_without_trigger_item_id() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-close-noid1",
        &[("a", "A", false)],
        "sub",
        "Sub",
        false,
        &[("x", "X", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    // headless-ui の anatomy 上 `trigger_item` の `id` は optional。ここで
    // 意図的に取り除き、`document.get_element_by_id` による再解決手段が
    // 無い状態を再現する。
    trigger_item.remove_attribute("id").unwrap();
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-sub-close-noid1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    let item_x = document
        .get_element_by_id("kn-sub-close-noid1-sub-item-x")
        .unwrap();
    assert!(!sub_content.has_attribute("hidden"));
    assert!(item_x.has_attribute("data-highlighted"));

    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(
        !not_default_prevented,
        "サブメニュー内での ArrowLeft は prevent_default されるべき"
    );

    assert!(
        sub_content.has_attribute("hidden"),
        "サブメニューが閉じるべき"
    );
    assert!(
        !item_x.has_attribute("data-highlighted"),
        "閉じた後サブメニュー項目の highlight は残らないべき"
    );
    assert!(
        trigger_item.has_attribute("data-highlighted"),
        "id-less な trigger-item でも閉じた後 highlight は親へ復帰するべき"
    );
}

/// ArrowLeft: トップレベル（サブメニュー内でない）では no-op
/// （`prevent_default` しない、受け入れ条件 2 後段）。
#[wasm_bindgen_test]
fn menu_open_arrow_left_at_top_level_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-toplevel-noop1",
        &[("a", "A", false), ("b", "B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-toplevel-noop1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    let item_a = document
        .get_element_by_id("kn-toplevel-noop1-item-a")
        .unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(
        not_default_prevented,
        "トップレベルの ArrowLeft は prevent_default されるべきでない"
    );
    assert!(item_a.has_attribute("data-highlighted"));
}

/// ネスト 2 段: ArrowRight を 2 回で孫 content まで降下し、ArrowLeft で
/// 1 段ずつ復帰する（アクティブ content チェーン解決の検証）。
#[wasm_bindgen_test]
fn menu_open_arrow_right_and_left_navigate_two_level_nested_submenu() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item_1, sub_content_1) = build_submenu_dom(
        &document,
        "kn-sub-nest1",
        &[("a", "A", false)],
        "sub1",
        "Sub1",
        false,
        &[("x", "X", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    // sub_content_1 の子孫にもう 1 段（trigger-item "sub2" + content [p]）を
    // 追加する（`id_prefix` を変え、孫 id と衝突しないようにする）。
    let (trigger_item_2, sub_content_2) = append_trigger_item_with_submenu(
        &document,
        &sub_content_1,
        "kn-sub-nest1-sub1",
        "sub2",
        "Sub2",
        false,
        &[("p", "P", false)],
        false,
    );
    wire_toggle_listener(&trigger_item_2, &sub_content_2);

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    let trigger = document.get_element_by_id("kn-sub-nest1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープ [a, sub1] の末尾（sub1）を highlight → 展開。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(!sub_content_1.has_attribute("hidden"));

    // sub_content_1 のスコープは [x, sub2] の 2 件。End で sub2 を
    // highlight → 展開。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item_2.has_attribute("data-highlighted"));
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(!sub_content_2.has_attribute("hidden"));
    let item_p = document
        .get_element_by_id("kn-sub-nest1-sub1-sub-item-p")
        .unwrap();
    assert!(item_p.has_attribute("data-highlighted"));

    // ArrowLeft で 1 段戻る（sub_content_2 が閉じ、highlight は sub2 へ）。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(sub_content_2.has_attribute("hidden"));
    assert!(trigger_item_2.has_attribute("data-highlighted"));
    assert!(
        !sub_content_1.has_attribute("hidden"),
        "sub1 はまだ開いたまま"
    );

    // もう一段 ArrowLeft で sub1 も閉じ、highlight は親 trigger-item(sub1) へ。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(sub_content_1.has_attribute("hidden"));
    assert!(trigger_item_1.has_attribute("data-highlighted"));

    // トップレベルまで戻ったので、もう一度 ArrowLeft しても no-op。
    let not_default_prevented = trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(not_default_prevented);
}

/// 展開後の typeahead はサブメニュー項目のラベルを対象にし、親スコープの
/// 打鍵バッファを引きずらない（`TypeaheadState::is_active_for` は対象
/// content が変わると無条件で無効になり、ArrowRight/ArrowLeft の明示的な
/// `typeahead.reset()` 呼び出しと合わせた多重防御になる、モジュール doc
/// §サブメニュー参照）。
#[wasm_bindgen_test]
fn submenu_arrow_navigation_resets_typeahead_buffer_and_targets_active_content_labels() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-typeahead1",
        &[("a", "Apple", false)],
        "sub",
        "Sub",
        false,
        &[("p", "Pear", false), ("q", "Quince", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-sub-typeahead1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープで "a" を入力（Apple にマッチ）。
    trigger.dispatch_event(&keydown_event("a")).unwrap();
    let item_a = document
        .get_element_by_id("kn-sub-typeahead1-item-a")
        .unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // End → trigger-item highlight → ArrowRight で展開（typeahead バッファは
    // リセットされる）。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(!sub_content.has_attribute("hidden"));

    // サブメニュー内で "q" を入力すると Quince にマッチする（親の "a"
    // バッファが引き継がれていないことの確認、旧バッファのままなら
    // "aq" 等になり Quince にマッチしなくなる）。
    trigger.dispatch_event(&keydown_event("q")).unwrap();
    let item_p = document
        .get_element_by_id("kn-sub-typeahead1-sub-item-p")
        .unwrap();
    let item_q = document
        .get_element_by_id("kn-sub-typeahead1-sub-item-q")
        .unwrap();
    assert!(item_q.has_attribute("data-highlighted"));
    assert!(!item_p.has_attribute("data-highlighted"));

    // ArrowLeft で閉鎖してもバッファがリセットされる。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(sub_content.has_attribute("hidden"));
}

/// XSS 回帰（REQ-1、イシュー #662）: 攻撃者制御文字列を含む trigger-item/
/// サブメニュー項目のラベルに対し ArrowRight/ArrowLeft/typeahead を操作
/// しても `script`/`img` 要素が DOM に生成されないこと。
#[wasm_bindgen_test]
fn submenu_arrow_navigation_with_attacker_controlled_labels_does_not_inject_script() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-xss1",
        &[("a", "A", false)],
        "sub",
        "<img src=x onerror=alert(1)>",
        false,
        &[("x", "\"><script>alert(2)</script>", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document.get_element_by_id("kn-sub-xss1-trigger").unwrap();
    html_element(&trigger).focus().unwrap();

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));
    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(!sub_content.has_attribute("hidden"));
    assert!(root.query_selector("script, img").unwrap().is_none());

    // 攻撃者制御ラベルの先頭文字（`"`）で typeahead しても注入されない。
    trigger.dispatch_event(&keydown_event("\"")).unwrap();
    assert!(root.query_selector("script, img").unwrap().is_none());

    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(sub_content.has_attribute("hidden"));
    assert!(root.query_selector("script, img").unwrap().is_none());
}

/// 回帰テスト（Bugbot 指摘、イシュー #662 PR #674）: `trigger-item` 自身の
/// 表示テキストが空（アイコンのみ等、実 UI でありうる形）で、かつサブ
/// メニューが `hidden`（未展開）のとき、親レベルの typeahead が
/// `trigger-item` 自身のラベルではなく子孫（隠れたサブメニュー項目）の
/// テキストへ誤マッチしないことを検証する。修正前は `item_label` が単純に
/// `text_content()` を使っており、`trigger-item` 自身が空文字でも隠れた
/// サブメニュー項目 "Zeta" のテキストを拾って "z" 入力にマッチしてしまって
/// いた。
#[wasm_bindgen_test]
fn menu_open_typeahead_does_not_match_hidden_nested_submenu_item_text() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, trigger_item, sub_content) = build_submenu_dom(
        &document,
        "kn-sub-typeahead2",
        &[("m", "Mango", false)],
        "sub",
        "",
        false,
        &[("z", "Zeta", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-sub-typeahead2-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    assert!(sub_content.has_attribute("hidden"));
    assert!(!trigger_item.has_attribute("data-highlighted"));

    // 隠れたサブメニュー項目 "Zeta" の先頭文字 "z" を入力しても、
    // 表示テキストを持たない trigger-item 自身はマッチしないため
    // highlight されない（修正前は誤って highlight されていた）。
    trigger.dispatch_event(&keydown_event("z")).unwrap();
    assert!(!trigger_item.has_attribute("data-highlighted"));
}

// ---------------------------------------------------------------------
// サブメニュー兄弟配置回帰テスト（Cursor Bugbot Medium 指摘、イシュー #662
// PR #674 追補）。`crates/headless-ui/src/menu.rs` モジュール doc の
// 「親 Menu インスタンスの content 内に子 Menu インスタンス由来の
// trigger_item/positioner/content を入れ子で配置する」契約では、子の
// positioner/content は trigger-item の**兄弟**として親 content 直下に
// 並ぶ配置が正当（かつ `aria-controls` は anatomy 上 optional）。
// `resolve_submenu_content`（`crates/wasm-full/src/keynav.rs`）の兄弟方向
// フォールバックがこの一般的な配置を解決できることを検証する。
// ---------------------------------------------------------------------

/// `parent_content` 直下へ、サブメニューを `trigger_item` の**兄弟**として
/// 配置する構成（イシュー #662 Bugbot 指摘・PR #674 追補）で `trigger-item`
/// + 子 Menu（`positioner`/`root`/`content`）を追加する。
/// [`append_trigger_item_with_submenu`]（子孫配置・`aria-controls` 経路の
/// 既存フィクスチャ）とは独立に保ち、既存の子孫配置テストへ影響を与えない。
/// `trigger_item` には `aria-controls` を意図的に設定せず、
/// `resolve_submenu_content` の兄弟方向フォールバックのみで解決できることを
/// 検証できるようにする。`id_prefix`・`trigger_item_value` から
/// `{id_prefix}-item-{trigger_item_value}`（trigger-item 自身）・
/// `{id_prefix}-sub-content-{trigger_item_value}`（子 content）・
/// `{id_prefix}-sub-item-{trigger_item_value}-{value}`（子 item）の id を
/// 組み立てる（同一 `parent_content` 直下に複数のサブメニューを並べても id
/// が衝突しないようにするため）。戻り値は `(trigger_item, sub_content)`。
#[allow(clippy::too_many_arguments)]
fn append_sibling_trigger_item_with_submenu(
    document: &Document,
    parent_content: &Element,
    id_prefix: &str,
    trigger_item_value: &str,
    trigger_item_label: &str,
    trigger_item_disabled: bool,
    sub_items: &[(&str, &str, bool)],
    sub_open: bool,
) -> (Element, Element) {
    let trigger_item = document.create_element("div").unwrap();
    trigger_item.set_attribute("data-scope", "menu").unwrap();
    trigger_item
        .set_attribute("data-part", "trigger-item")
        .unwrap();
    trigger_item.set_attribute("role", "menuitem").unwrap();
    trigger_item.set_attribute("aria-haspopup", "menu").unwrap();
    let trigger_item_id = format!("{id_prefix}-item-{trigger_item_value}");
    trigger_item.set_attribute("id", &trigger_item_id).unwrap();
    trigger_item
        .set_attribute("aria-expanded", if sub_open { "true" } else { "false" })
        .unwrap();
    trigger_item
        .set_attribute("data-state", if sub_open { "open" } else { "closed" })
        .unwrap();
    if trigger_item_disabled {
        trigger_item.set_attribute("aria-disabled", "true").unwrap();
        trigger_item.set_attribute("data-disabled", "").unwrap();
    }
    trigger_item.set_text_content(Some(trigger_item_label));
    parent_content.append_child(&trigger_item).unwrap();

    // positioner: trigger-item の兄弟として、子 root/content をラップする
    // 中間要素（`crates/headless-ui/src/menu.rs::positioner` 相当）。
    let positioner = document.create_element("div").unwrap();
    positioner.set_attribute("data-scope", "menu").unwrap();
    positioner.set_attribute("data-part", "positioner").unwrap();

    let sub_root = document.create_element("div").unwrap();
    sub_root.set_attribute("data-scope", "menu").unwrap();
    sub_root.set_attribute("data-part", "root").unwrap();
    let sub_content = document.create_element("div").unwrap();
    sub_content.set_attribute("data-scope", "menu").unwrap();
    sub_content.set_attribute("data-part", "content").unwrap();
    let sub_content_id = format!("{id_prefix}-sub-content-{trigger_item_value}");
    sub_content.set_attribute("id", &sub_content_id).unwrap();
    sub_content.set_attribute("role", "menu").unwrap();
    if !sub_open {
        sub_content.set_attribute("hidden", "").unwrap();
    }
    for (value, label, disabled) in sub_items {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "menu").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("role", "menuitem").unwrap();
        item.set_attribute("data-value", value).unwrap();
        item.set_attribute(
            "id",
            &format!("{id_prefix}-sub-item-{trigger_item_value}-{value}"),
        )
        .unwrap();
        if *disabled {
            item.set_attribute("aria-disabled", "true").unwrap();
            item.set_attribute("data-disabled", "").unwrap();
        }
        item.set_text_content(Some(label));
        sub_content.append_child(&item).unwrap();
    }
    sub_root.append_child(&sub_content).unwrap();
    positioner.append_child(&sub_root).unwrap();
    // positioner（子 root/content 一式）を trigger_item の**兄弟**として
    // parent_content 直下へ追加する（trigger_item の子孫にはしない）。
    parent_content.append_child(&positioner).unwrap();

    (trigger_item, sub_content)
}

/// ArrowRight: `aria-controls` を持たず、かつ子 content が `trigger-item`
/// の兄弟として親 content 直下に配置される構成（`headless-ui` の一般的な
/// anatomy 配置）でも、サブメニューが展開され先頭の非 disabled 項目へ
/// highlight が移るべき（Cursor Bugbot Medium 指摘、イシュー #662 PR #674
/// 追補。修正前は `resolve_submenu_content` が子孫方向フォールバックしか
/// 持たず、この配置ではサイレント no-op になっていた）。
#[wasm_bindgen_test]
fn menu_open_arrow_right_expands_sibling_submenu_without_aria_controls() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(
        &document,
        "kn-sub-sibling1",
        &[("a", "A", false)],
        true,
        false,
    );
    let content = document
        .get_element_by_id("kn-sub-sibling1-content")
        .unwrap();
    let (trigger_item, sub_content) = append_sibling_trigger_item_with_submenu(
        &document,
        &content,
        "kn-sub-sibling1",
        "sub",
        "Sub",
        false,
        &[("x", "X", false), ("y", "Y", false)],
        false,
    );
    wire_toggle_listener(&trigger_item, &sub_content);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-sub-sibling1-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープは [a, sub] の 2 件。End で末尾（trigger-item "sub"）へ。
    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item.has_attribute("data-highlighted"));

    let not_default_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        !not_default_prevented,
        "展開する ArrowRight は prevent_default されるべき"
    );

    assert!(
        !sub_content.has_attribute("hidden"),
        "aria-controls が無く兄弟配置でもサブメニューが展開されるべき"
    );
    let item_x = document
        .get_element_by_id("kn-sub-sibling1-sub-item-sub-x")
        .unwrap();
    assert!(
        item_x.has_attribute("data-highlighted"),
        "展開直後は先頭の非 disabled 項目が highlight されるべき"
    );
    assert_eq!(
        sub_content
            .get_attribute("aria-activedescendant")
            .as_deref(),
        Some("kn-sub-sibling1-sub-item-sub-x")
    );
    // 親の highlight（trigger-item 上）は展開後も維持される。
    assert!(trigger_item.has_attribute("data-highlighted"));
}

/// ArrowRight: 同一 content 直下に兄弟配置のサブメニューを持つ trigger-item
/// が 2 つ並ぶ場合、各 trigger-item は自分のサブメニューだけを解決し、
/// 隣の trigger-item のサブメニューを誤って展開しない（Cursor Bugbot
/// Medium 指摘、イシュー #662 PR #674 追補。`resolve_submenu_content_via_sibling`
/// の「次の trigger-item に到達したら打ち切る」ガードの検証）。
#[wasm_bindgen_test]
fn menu_open_arrow_right_resolves_only_own_sibling_submenu_when_two_adjacent() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_menu_dom(&document, "kn-sub-sibling2", &[], true, false);
    let content = document
        .get_element_by_id("kn-sub-sibling2-content")
        .unwrap();
    let (trigger_item_1, sub_content_1) = append_sibling_trigger_item_with_submenu(
        &document,
        &content,
        "kn-sub-sibling2",
        "sub1",
        "Sub1",
        false,
        &[("x", "X", false)],
        false,
    );
    wire_toggle_listener(&trigger_item_1, &sub_content_1);
    let (trigger_item_2, sub_content_2) = append_sibling_trigger_item_with_submenu(
        &document,
        &content,
        "kn-sub-sibling2",
        "sub2",
        "Sub2",
        false,
        &[("y", "Y", false)],
        false,
    );
    wire_toggle_listener(&trigger_item_2, &sub_content_2);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let trigger = document
        .get_element_by_id("kn-sub-sibling2-trigger")
        .unwrap();
    html_element(&trigger).focus().unwrap();

    // 親スコープは [sub1, sub2] の 2 件。Home で先頭（sub1）を highlight。
    trigger.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(trigger_item_1.has_attribute("data-highlighted"));

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(
        !sub_content_1.has_attribute("hidden"),
        "sub1 自身のサブメニューが展開されるべき"
    );
    assert!(
        sub_content_2.has_attribute("hidden"),
        "隣接する sub2 のサブメニューを誤って展開してはいけない"
    );
    let item_x = document
        .get_element_by_id("kn-sub-sibling2-sub-item-sub1-x")
        .unwrap();
    assert!(item_x.has_attribute("data-highlighted"));

    // 閉じて sub2 へ移り、対称に検証する。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(sub_content_1.has_attribute("hidden"));

    trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(trigger_item_2.has_attribute("data-highlighted"));

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert!(
        !sub_content_2.has_attribute("hidden"),
        "sub2 自身のサブメニューが展開されるべき"
    );
    assert!(
        sub_content_1.has_attribute("hidden"),
        "sub1 のサブメニューは閉じたままであるべき"
    );
    let item_y = document
        .get_element_by_id("kn-sub-sibling2-sub-item-sub2-y")
        .unwrap();
    assert!(item_y.has_attribute("data-highlighted"));
}

// ---------------------------------------------------------------------
// Combobox（イシュー #1071）
// ---------------------------------------------------------------------

/// `crates/headless-ui/src/combobox.rs` の SSR 出力契約を手組みで再現した
/// Combobox DOM を生成する。Menu/Select と異なりフォーカスは `input`
/// （`role="combobox"`）が保持し、`trigger` は `tabindex="-1"` 固定で
/// フォーカスを受けない（`combobox::trigger` doc 契約）。`aria-expanded` は
/// input・trigger の両方へ出力する（`combobox::input`/`combobox::trigger`
/// 契約）。返り値は `(root, input, trigger, content)`。
fn build_combobox_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool)],
    open: bool,
    loop_focus: bool,
) -> (Element, Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "combobox").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let input_id = format!("{root_id}-input");
    let trigger_id = format!("{root_id}-trigger");
    let content_id = format!("{root_id}-content");

    let input = document.create_element("input").unwrap();
    input.set_attribute("data-scope", "combobox").unwrap();
    input.set_attribute("data-part", "input").unwrap();
    input.set_attribute("role", "combobox").unwrap();
    input.set_attribute("id", &input_id).unwrap();
    input
        .set_attribute("aria-expanded", if open { "true" } else { "false" })
        .unwrap();
    input.set_attribute("aria-controls", &content_id).unwrap();
    root.append_child(&input).unwrap();

    let trigger = document.create_element("button").unwrap();
    trigger.set_attribute("data-scope", "combobox").unwrap();
    trigger.set_attribute("data-part", "trigger").unwrap();
    trigger.set_attribute("type", "button").unwrap();
    trigger.set_attribute("tabindex", "-1").unwrap();
    trigger.set_attribute("id", &trigger_id).unwrap();
    trigger.set_attribute("aria-haspopup", "listbox").unwrap();
    trigger
        .set_attribute("aria-expanded", if open { "true" } else { "false" })
        .unwrap();
    trigger.set_attribute("aria-controls", &content_id).unwrap();
    root.append_child(&trigger).unwrap();

    let content = document.create_element("div").unwrap();
    content.set_attribute("data-scope", "combobox").unwrap();
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
        item.set_attribute("data-scope", "combobox").unwrap();
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
    (root, input, trigger, content)
}

/// Combobox の trigger click を open/closed トグルとして模擬する
/// （[`wire_toggle_listener`] の Combobox 版）。input・trigger 双方の
/// `aria-expanded`/`data-state` を更新する点が異なる
/// （`crates/headless-ui/src/combobox.rs` の「`aria-expanded` は input と
/// trigger の両方が出力する」契約に対応、`keynav.rs` モジュール doc
/// §Combobox 参照）。
fn wire_combobox_toggle_listener(trigger: &Element, input: &Element, content: &Element) {
    let closure = Closure::<dyn FnMut(Event)>::new({
        let trigger = trigger.clone();
        let input = input.clone();
        let content = content.clone();
        move |event: Event| {
            let is_self_click = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.is_same_node(Some(&trigger)));
            if !is_self_click {
                return;
            }
            if content.has_attribute("hidden") {
                let _ = content.remove_attribute("hidden");
                let _ = content.set_attribute("data-state", "open");
                let _ = trigger.set_attribute("aria-expanded", "true");
                let _ = trigger.set_attribute("data-state", "open");
                let _ = input.set_attribute("aria-expanded", "true");
                let _ = input.set_attribute("data-state", "open");
            } else {
                let _ = content.set_attribute("hidden", "");
                let _ = content.set_attribute("data-state", "closed");
                let _ = trigger.set_attribute("aria-expanded", "false");
                let _ = trigger.set_attribute("data-state", "closed");
                let _ = input.set_attribute("aria-expanded", "false");
                let _ = input.set_attribute("data-state", "closed");
            }
        }
    });
    trigger
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

/// Combobox の item click を「選択 + close」として模擬する
/// （`ComboboxAction::Select` の `closeOnSelect` 既定を模す）。クリックされた
/// item に `data-clicked` を付与し、確定の検知に使う。
fn wire_combobox_item_select_listeners(
    items: &[Element],
    trigger: &Element,
    input: &Element,
    content: &Element,
) {
    for item in items {
        let closure = Closure::<dyn FnMut(Event)>::new({
            let item = item.clone();
            let trigger = trigger.clone();
            let input = input.clone();
            let content = content.clone();
            move |event: Event| {
                let is_self_click = event
                    .target()
                    .and_then(|target| target.dyn_into::<Element>().ok())
                    .is_some_and(|target| target.is_same_node(Some(&item)));
                if !is_self_click {
                    return;
                }
                let _ = item.set_attribute("data-clicked", "");
                let _ = content.set_attribute("hidden", "");
                let _ = content.set_attribute("data-state", "closed");
                let _ = trigger.set_attribute("aria-expanded", "false");
                let _ = trigger.set_attribute("data-state", "closed");
                let _ = input.set_attribute("aria-expanded", "false");
                let _ = input.set_attribute("data-state", "closed");
            }
        });
        item.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

/// 検証 1（実装計画 §5.3-1）: closed の input 上で ArrowDown → trigger へ
/// click 合成され open、先頭の非 disabled item に `data-highlighted`、
/// input の `aria-activedescendant` がその item の `id`、input/trigger の
/// `aria-expanded="true"`。`prevent_default` されている。
#[wasm_bindgen_test]
fn combobox_closed_arrow_down_opens_via_synthesized_click_and_sets_initial_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-open1",
        &[("a", "A", false), ("b", "B", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_combobox_toggle_listener(&trigger, &input, &content);

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    let not_default_prevented = input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(!not_default_prevented, "open は prevent_default されるべき");

    assert!(!content.has_attribute("hidden"));
    assert_eq!(
        input.get_attribute("aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(
        trigger.get_attribute("aria-expanded").as_deref(),
        Some("true")
    );
    let item_a = document.get_element_by_id("kn-cb-open1-item-a").unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert_eq!(
        input.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-cb-open1-item-a")
    );
    // content 側には aria-activedescendant を書かない（input 側配線、
    // モジュール doc §Combobox 参照）。
    assert!(content.get_attribute("aria-activedescendant").is_none());
}

/// 検証 2（実装計画 §5.3-2）: closed の input 上で ArrowUp → open + 末尾の
/// 非 disabled item が初期 highlight。
#[wasm_bindgen_test]
fn combobox_closed_arrow_up_opens_with_initial_highlight_on_last_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-open2",
        &[("a", "A", false), ("b", "B", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_combobox_toggle_listener(&trigger, &input, &content);

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowUp")).unwrap();

    assert!(!content.has_attribute("hidden"));
    let item_b = document.get_element_by_id("kn-cb-open2-item-b").unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert_eq!(
        input.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-cb-open2-item-b")
    );
}

/// 検証 3・6・7（実装計画 §5.3-3/6/7）: open で ArrowDown/ArrowUp/Home/End が
/// highlight を移動し disabled をスキップ、既定は非循環、`data-loop-focus`
/// で循環する。あわせて closed の Home/End/Enter は `prevent_default` され
/// ない（キャレット移動・submit の既定を奪わない）ことも確認する。
#[wasm_bindgen_test]
fn combobox_open_arrow_home_end_move_highlight_and_closed_caret_keys_are_not_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-move",
        &[("a", "A", false), ("b", "B", true), ("c", "C", false)],
        true,
        true,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_combobox_toggle_listener(&trigger, &input, &content);

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("Home")).unwrap();
    let item_a = document.get_element_by_id("kn-cb-move-item-a").unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    // disabled（b）をスキップして c へ。
    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    let item_c = document.get_element_by_id("kn-cb-move-item-c").unwrap();
    assert!(item_c.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));

    // `data-loop-focus="true"` のため末尾から ArrowDown で先頭へ循環する。
    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    input.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_c.has_attribute("data-highlighted"));

    // closed 側の回帰: Home/End/Enter はキャレット移動・submit の既定動作を
    // 奪わない（モジュール doc §Combobox・fail-closed 判定表参照）。
    let (root2, input2, trigger2, content2) = build_combobox_dom(
        &document,
        "kn-cb-closed-noop",
        &[("a", "A", false)],
        false,
        false,
    );
    let _cleanup2 = RemoveOnDrop(root2.clone());
    wire_combobox_toggle_listener(&trigger2, &input2, &content2);
    wire_keynav(root2.clone()).expect("wire_keynav must succeed");
    html_element(&input2).focus().unwrap();
    for key in ["Home", "End", "Enter"] {
        let not_default_prevented = input2.dispatch_event(&keydown_event(key)).unwrap();
        assert!(
            not_default_prevented,
            "key={key}: closed 時は prevent_default されないべき"
        );
    }
    assert!(content2.has_attribute("hidden"), "closed のままであるべき");
}

/// 検証 4（実装計画 §5.3-4）: open で Enter → highlight 中 item へ click
/// 合成（選択と同時に close）。highlight 不在・disabled の item は no-op。
#[wasm_bindgen_test]
fn combobox_open_enter_clicks_highlighted_item_and_closes() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-confirm",
        &[("a", "A", false), ("b", "B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let item_a = document.get_element_by_id("kn-cb-confirm-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-cb-confirm-item-b").unwrap();
    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_combobox_item_select_listeners(
        &[item_a.clone(), item_b.clone()],
        &trigger,
        &input,
        &content,
    );

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    // highlight 不在での Enter は no-op（click 未検知のまま）。
    input.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(!item_a.has_attribute("data-clicked"));

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    input.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_a.has_attribute("data-clicked"));
    assert!(
        content.has_attribute("hidden"),
        "選択と同時に close するべき"
    );
    assert_eq!(
        input.get_attribute("aria-expanded").as_deref(),
        Some("false")
    );
}

/// Bugbot 指摘 "Confirm leaves activedescendant set"（PR #1094 レビュー、
/// イシュー #1071）の回帰: Enter による確定（選択 + close）でも、Escape と
/// 同様に `data-highlighted`/`aria-activedescendant` がクリアされること。
/// クリアされないと collapsed 後も hidden な option を `aria-activedescendant`
/// が指し続け、ARIA 1.2 の collapsed-combobox ルール違反として次に open
/// するまで支援技術を混乱させる。
#[wasm_bindgen_test]
fn combobox_open_enter_confirm_clears_highlight_and_activedescendant() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-confirm-clear",
        &[("a", "A", false), ("b", "B", false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let item_a = document
        .get_element_by_id("kn-cb-confirm-clear-item-a")
        .unwrap();
    let item_b = document
        .get_element_by_id("kn-cb-confirm-clear-item-b")
        .unwrap();
    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_combobox_item_select_listeners(
        &[item_a.clone(), item_b.clone()],
        &trigger,
        &input,
        &content,
    );

    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert_eq!(
        input.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-cb-confirm-clear-item-a")
    );

    input.dispatch_event(&keydown_event("Enter")).unwrap();
    assert!(item_a.has_attribute("data-clicked"), "選択が確定するべき");
    assert!(
        !item_a.has_attribute("data-highlighted"),
        "確定後は data-highlighted がクリアされているべき"
    );
    assert!(
        input.get_attribute("aria-activedescendant").is_none(),
        "確定後は input の aria-activedescendant がクリアされているべき\
         （collapsed-combobox ルール、Bugbot 指摘）"
    );
}

/// 検証 5・6（実装計画 §5.3-5/6）: open の Escape で highlight クリア +
/// trigger への click 合成で閉じる。closed の Escape は no-op（fail-open
/// 回帰: closed で claim すると誤って open してしまう）。
#[wasm_bindgen_test]
fn combobox_open_escape_clears_highlight_and_closes_but_closed_escape_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) =
        build_combobox_dom(&document, "kn-cb-escape", &[("a", "A", false)], true, false);
    let _cleanup = RemoveOnDrop(root.clone());
    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    let item_a = document.get_element_by_id("kn-cb-escape-item-a").unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    input.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(!item_a.has_attribute("data-highlighted"));
    assert!(input.get_attribute("aria-activedescendant").is_none());
    assert!(content.has_attribute("hidden"));
    assert_eq!(
        input.get_attribute("aria-expanded").as_deref(),
        Some("false")
    );

    // closed の Escape は fail-closed に no-op（誤って open しないこと）。
    let not_default_prevented = input.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(
        not_default_prevented,
        "closed の Escape は prevent_default されないべき"
    );
    assert!(content.has_attribute("hidden"), "closed のままであるべき");
}

/// 検証 8（実装計画 §5.3-8）: printable 文字キー・ArrowLeft/ArrowRight/Tab は
/// open/closed いずれも no-op かつ `prevent_default` されない（typeahead 非
/// 適用・キャレット移動維持の回帰、モジュール doc §Combobox 参照）。
#[wasm_bindgen_test]
fn combobox_typeahead_and_caret_keys_are_never_prevented() {
    let document = web_sys::window().unwrap().document().unwrap();
    for open in [false, true] {
        let (root, input, trigger, content) = build_combobox_dom(
            &document,
            &format!("kn-cb-caret-{open}"),
            &[("a", "A", false)],
            open,
            false,
        );
        let _cleanup = RemoveOnDrop(root.clone());
        wire_combobox_toggle_listener(&trigger, &input, &content);
        wire_keynav(root.clone()).expect("wire_keynav must succeed");
        html_element(&input).focus().unwrap();

        for key in ["a", "ArrowLeft", "ArrowRight", "Tab"] {
            let not_default_prevented = input.dispatch_event(&keydown_event(key)).unwrap();
            assert!(
                not_default_prevented,
                "open={open} key={key}: prevent_default されないべき"
            );
        }
        let item_a = document
            .get_element_by_id(&format!("kn-cb-caret-{open}-item-a"))
            .unwrap();
        assert!(!item_a.has_attribute("data-highlighted"));
    }
}

/// 検証 9（実装計画 §5.3-9）: 修飾キー（Ctrl/Alt/Meta）付きは no-op。
#[wasm_bindgen_test]
fn combobox_modifier_keys_are_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-modifiers",
        &[("a", "A", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key("ArrowDown");
    init.set_ctrl_key(true);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .unwrap()
        .dyn_into::<Event>()
        .unwrap();
    let not_default_prevented = input.dispatch_event(&event).unwrap();
    assert!(not_default_prevented, "修飾キー付きは no-op であるべき");
    assert!(content.has_attribute("hidden"), "closed のままであるべき");
}

/// 検証 10（実装計画 §5.3-10）: `trigger` が存在しない anatomy では
/// ArrowDown が no-op（fail-closed）かつ panic しない。
#[wasm_bindgen_test]
fn combobox_missing_trigger_arrow_down_is_noop_and_does_not_panic() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-no-trigger",
        &[("a", "A", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    trigger.remove();
    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(content.has_attribute("hidden"), "trigger 不在では開かない");
}

/// 検証 11（実装計画 §5.3-11）: 改ざん `aria-controls`（root 外の id を指す）
/// は封じ込め検査で no-op（A01 対策）。
#[wasm_bindgen_test]
fn combobox_tampered_aria_controls_pointing_outside_root_is_noop() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-tampered",
        &[("a", "A", false)],
        false,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());

    let outside = document.create_element("div").unwrap();
    outside.set_id("kn-cb-tampered-outside-content");
    outside.set_attribute("data-scope", "combobox").unwrap();
    outside.set_attribute("data-part", "content").unwrap();
    document.body().unwrap().append_child(&outside).unwrap();
    let _cleanup_outside = RemoveOnDrop(outside.clone());

    input
        .set_attribute("aria-controls", "kn-cb-tampered-outside-content")
        .unwrap();

    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    // root 内の正当な content は変化せず、root 外の改ざん先も操作されない。
    assert!(content.has_attribute("hidden"));
    assert!(!outside.has_attribute("data-highlighted"));
}

/// 検証 12（実装計画 §5.3-12）: 攻撃者制御文字列（`<script>` を含むラベル・
/// `data-value`）を持つ combobox に対して全キー操作・確定を行っても
/// `document.querySelector("script")` が増えないこと（XSS 回帰、REQ-1）。
#[wasm_bindgen_test]
fn combobox_keyboard_navigation_with_attacker_controlled_strings_does_not_inject_script() {
    let document = web_sys::window().unwrap().document().unwrap();
    // `id` は build_combobox_dom が `value` からそのまま導出するため、
    // 攻撃者制御文字列は `label`（item のテキスト内容）と、id 生成後に
    // 追記する `data-value` の両方へ別途仕込む（id 自体を攻撃者制御にしない、
    // テストの自己整合性のため）。
    let attacker_label = "<script>window.__cb_xss = true</script>";
    let (root, input, trigger, content) = build_combobox_dom(
        &document,
        "kn-cb-xss",
        &[("opt-1", attacker_label, false)],
        true,
        false,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let item = document.get_element_by_id("kn-cb-xss-item-opt-1").unwrap();
    item.set_attribute("data-value", "\"><script>window.__cb_xss2 = true</script>")
        .unwrap();
    wire_combobox_toggle_listener(&trigger, &input, &content);
    wire_combobox_item_select_listeners(&[item.clone()], &trigger, &input, &content);
    wire_keynav(root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    let before = document.query_selector_all("script").unwrap().length();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    input.dispatch_event(&keydown_event("Home")).unwrap();
    input.dispatch_event(&keydown_event("End")).unwrap();
    input.dispatch_event(&keydown_event("Enter")).unwrap();
    input.dispatch_event(&keydown_event("Escape")).unwrap();

    let after = document.query_selector_all("script").unwrap().length();
    assert_eq!(
        before, after,
        "攻撃者制御文字列を含む combobox のキー操作で <script> が増えてはならない"
    );
}

/// Bugbot 指摘 "Stale root blocks open highlight"（PR #1094 レビュー、
/// イシュー #1071）の回帰: keynav のマウント境界（`wire_keynav` に渡す
/// root）は Combobox 本体の `[data-part="root"]` より外側の安定した祖先
/// （実アプリでは再描画のたびに置き換わらないコンテナ）であり、trigger
/// click 駆動の再描画で Combobox の `[data-part="root"]` 配下（内側の
/// root/content/input/trigger）全体が新しい要素へ丸ごと差し替わっても、
/// open 直後の初期 highlight・`aria-activedescendant` が detached になった
/// 旧ツリーではなく生きた（新しい）DOM 上へ設定されること。
#[wasm_bindgen_test]
fn combobox_closed_arrow_down_opens_after_full_subtree_replacement_still_sets_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root_id = "kn-cb-replace";
    let items: &[(&str, &str, bool)] = &[("a", "A", false), ("b", "B", false)];

    // keynav のマウント境界は Combobox の `[data-part="root"]` より外側の
    // 安定コンテナとする（実アプリでの「Combobox サブツリーだけが再描画で
    // 差し替わり、mount root 自体は永続する」構成を模す）。
    let mount_root = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&mount_root).unwrap();
    let _cleanup = RemoveOnDrop(mount_root.clone());

    let (combobox_root, input, trigger, _content) =
        build_combobox_dom(&document, root_id, items, false, false);
    // `build_combobox_dom` は body 直下へ追加するため、mount_root 配下へ
    // 付け替える（`append_child` は既存ノードを再親化する）。
    mount_root.append_child(&combobox_root).unwrap();

    // trigger click のたびに、Combobox の `[data-part="root"]` 配下全体を
    // detach し、同じ id を持つ新しい要素へ丸ごと差し替える（click 駆動の
    // 再描画を模す）。mount_root 自体は差し替えない。
    let closure = Closure::<dyn FnMut(Event)>::new({
        let trigger = trigger.clone();
        let document = document.clone();
        let mount_root = mount_root.clone();
        let root_id = root_id.to_string();
        let items: Vec<(String, String, bool)> = items
            .iter()
            .map(|(v, l, d)| (v.to_string(), l.to_string(), *d))
            .collect();
        move |event: Event| {
            let is_self_click = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.is_same_node(Some(&trigger)));
            if !is_self_click {
                return;
            }
            let old_root = document.get_element_by_id(&root_id).unwrap();
            old_root.remove();
            let owned_items: Vec<(&str, &str, bool)> = items
                .iter()
                .map(|(v, l, d)| (v.as_str(), l.as_str(), *d))
                .collect();
            let (new_root, _new_input, _new_trigger, _new_content) =
                build_combobox_dom(&document, &root_id, &owned_items, true, false);
            mount_root.append_child(&new_root).unwrap();
        }
    });
    trigger
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    wire_keynav(mount_root.clone()).expect("wire_keynav must succeed");
    html_element(&input).focus().unwrap();

    input.dispatch_event(&keydown_event("ArrowDown")).unwrap();

    // 生きた（新しい）content/input/item を id 経由で解決して検証する
    // （旧 root/content/input は detached のまま残っている）。
    let live_content = document
        .get_element_by_id(&format!("{root_id}-content"))
        .unwrap();
    let live_input = document
        .get_element_by_id(&format!("{root_id}-input"))
        .unwrap();
    let live_item_a = document
        .get_element_by_id(&format!("{root_id}-item-a"))
        .unwrap();
    assert!(
        !live_content.has_attribute("hidden"),
        "生きた content は open のままであるべき"
    );
    assert!(
        live_item_a.has_attribute("data-highlighted"),
        "detached になった旧 root からの再クエリではなく、生きた DOM 上へ \
         初期 highlight が設定されるべき（Bugbot 指摘）"
    );
    assert_eq!(
        live_input.get_attribute("aria-activedescendant").as_deref(),
        Some(format!("{root_id}-item-a").as_str()),
        "生きた input の aria-activedescendant が設定されるべき（Bugbot 指摘）"
    );
}

// ---------------------------------------------------------------------
// Listbox（常時展開のリスト選択、`crates/headless-ui/src/listbox.rs`）の
// Arrow/Home/End/typeahead/Enter・Space/Escape キーボード配線
// （イシュー #1070）。
// ---------------------------------------------------------------------

/// `crates/headless-ui/src/listbox.rs` の SSR 出力契約を手組みで再現した
/// Listbox DOM を生成する。`items`: `(value, label, disabled)` のリスト。
/// `orientation`/`loop_focus` は `Some` のときのみ `data-orientation`/
/// `data-loop-focus` を content へ付与する（headless-ui はいずれも出力
/// しない呼び出し側オプトイン属性であるため、`None` は「欠落」を表す）。
/// `listbox::content()`（`role="listbox"`/`tabindex="0"`）とは異なり本クレート
/// は `fandhe-frontend-headless-ui` に依存しないため、実際の `listbox::content()`/
/// `listbox::item()` 関数は呼べず属性契約を手組みで再現する。
fn build_listbox_dom(
    document: &Document,
    root_id: &str,
    items: &[(&str, &str, bool)],
    orientation: Option<&str>,
    loop_focus: Option<&str>,
) -> Element {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "listbox").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let content = document.create_element("div").unwrap();
    content.set_attribute("data-scope", "listbox").unwrap();
    content.set_attribute("data-part", "content").unwrap();
    let content_id = format!("{root_id}-content");
    content.set_attribute("id", &content_id).unwrap();
    content.set_attribute("role", "listbox").unwrap();
    content.set_attribute("tabindex", "0").unwrap();
    if let Some(orientation) = orientation {
        content
            .set_attribute("data-orientation", orientation)
            .unwrap();
    }
    if let Some(loop_focus) = loop_focus {
        content
            .set_attribute("data-loop-focus", loop_focus)
            .unwrap();
    }

    for (value, label, disabled) in items {
        let item = document.create_element("div").unwrap();
        item.set_attribute("data-scope", "listbox").unwrap();
        item.set_attribute("data-part", "item").unwrap();
        item.set_attribute("role", "option").unwrap();
        item.set_attribute("data-value", value).unwrap();
        item.set_attribute("id", &format!("{root_id}-item-{value}"))
            .unwrap();
        item.set_attribute("aria-selected", "false").unwrap();
        if *disabled {
            item.set_attribute("aria-disabled", "true").unwrap();
            item.set_attribute("data-disabled", "").unwrap();
        }
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

/// 検証 1（受け入れ条件 2）: 既定 Vertical で ArrowDown/ArrowUp が highlight
/// を移動し、`content` の `aria-activedescendant` が highlight 対象の `id`
/// へ追随する。`data-highlighted` は常に 1 個のみ付与される。
#[wasm_bindgen_test]
fn listbox_arrow_down_up_move_highlight_and_update_aria_activedescendant() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-arrow1",
        &[("a", "A", false), ("b", "B", false), ("c", "C", false)],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-arrow1-content").unwrap();
    let item_a = document.get_element_by_id("kn-lb-arrow1-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-lb-arrow1-item-b").unwrap();

    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-lb-arrow1-item-a")
    );

    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-lb-arrow1-item-b")
    );

    content.dispatch_event(&keydown_event("ArrowUp")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));

    // Horizontal 方向のキーは既定 Vertical では no-op（受け入れ条件）。
    let not_default_prevented = content
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        not_default_prevented,
        "既定 Vertical では ArrowRight は prevent_default されるべきではない"
    );
    assert!(item_a.has_attribute("data-highlighted"));
}

/// 検証 2: disabled 項目を Arrow ナビゲーションがスキップする。
#[wasm_bindgen_test]
fn listbox_arrow_navigation_skips_disabled_items() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-disabled1",
        &[("a", "A", false), ("b", "B", true), ("c", "C", false)],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document
        .get_element_by_id("kn-lb-disabled1-content")
        .unwrap();
    let item_a = document
        .get_element_by_id("kn-lb-disabled1-item-a")
        .unwrap();
    let item_c = document
        .get_element_by_id("kn-lb-disabled1-item-c")
        .unwrap();

    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        item_c.has_attribute("data-highlighted"),
        "disabled な B をスキップして C へ移動すべき"
    );
}

/// 検証 3: Home/End で先頭/末尾の非 disabled 項目へ移動する。
#[wasm_bindgen_test]
fn listbox_home_end_move_to_first_last_enabled() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-homeend1",
        &[
            ("a", "A", true),
            ("b", "B", false),
            ("c", "C", false),
            ("d", "D", true),
        ],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document
        .get_element_by_id("kn-lb-homeend1-content")
        .unwrap();
    let item_b = document.get_element_by_id("kn-lb-homeend1-item-b").unwrap();
    let item_c = document.get_element_by_id("kn-lb-homeend1-item-c").unwrap();

    content.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    content.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_c.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// 検証 4: `data-loop-focus` 欠落（既定）では末尾で ArrowDown が no-op
/// （`aria-activedescendant` 不変、`prevent_default` もされない）。
#[wasm_bindgen_test]
fn listbox_default_does_not_loop_at_ends() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-noloop1",
        &[("a", "A", false), ("b", "B", false)],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-noloop1-content").unwrap();
    let item_b = document.get_element_by_id("kn-lb-noloop1-item-b").unwrap();

    content.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));

    let not_default_prevented = content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        not_default_prevented,
        "端での既定非循環 ArrowDown は prevent_default されるべきではない"
    );
    assert!(item_b.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-lb-noloop1-item-b")
    );
}

/// 検証 5: content に `data-loop-focus="true"` を明示すると端で循環する。
#[wasm_bindgen_test]
fn listbox_explicit_loop_focus_true_wraps_at_ends() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-loop1",
        &[("a", "A", false), ("b", "B", false)],
        None,
        Some("true"),
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-loop1-content").unwrap();
    let item_a = document.get_element_by_id("kn-lb-loop1-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-lb-loop1-item-b").unwrap();

    content.dispatch_event(&keydown_event("End")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        item_a.has_attribute("data-highlighted"),
        "data-loop-focus=\"true\" では末尾から先頭へ循環すべき"
    );
}

/// 検証 6: `data-orientation="horizontal"` のときのみ ArrowLeft/ArrowRight を
/// 受理し、ArrowUp/ArrowDown は no-op になる。
#[wasm_bindgen_test]
fn listbox_horizontal_orientation_responds_to_left_right_and_ignores_up_down() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-horiz1",
        &[("a", "A", false), ("b", "B", false)],
        Some("horizontal"),
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-horiz1-content").unwrap();
    let item_a = document.get_element_by_id("kn-lb-horiz1-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-lb-horiz1-item-b").unwrap();

    content
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    content
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(item_b.has_attribute("data-highlighted"));

    let not_default_prevented = content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(
        not_default_prevented,
        "horizontal では ArrowDown は prevent_default されるべきではない"
    );
    assert!(item_b.has_attribute("data-highlighted"));

    content.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
}

/// 検証 7: typeahead で `item-text` 子から解決したラベルへ highlight が
/// 移動する（既存の Menu/Select typeahead 実装の再利用確認）。
#[wasm_bindgen_test]
fn listbox_typeahead_moves_highlight_to_matching_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-typeahead1",
        &[
            ("a", "Apple", false),
            ("b", "Banana", false),
            ("c", "Cherry", false),
        ],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document
        .get_element_by_id("kn-lb-typeahead1-content")
        .unwrap();
    let item_b = document
        .get_element_by_id("kn-lb-typeahead1-item-b")
        .unwrap();

    content.dispatch_event(&keydown_event("b")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-lb-typeahead1-item-b")
    );
}

/// 検証 8: タイムアウト内の連続入力でバッファが絞り込まれ、タイムアウト
/// （[`TYPEAHEAD_TIMEOUT_MS`]）超過後は新規バッファとして再探索する
/// （`menu_open_typeahead_buffers_within_timeout_and_resets_after` と同型）。
#[wasm_bindgen_test]
async fn listbox_typeahead_buffers_within_timeout_and_resets_after() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-typeahead2",
        &[
            ("a", "Almond", false),
            ("b", "Apricot", false),
            ("c", "Banana", false),
        ],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document
        .get_element_by_id("kn-lb-typeahead2-content")
        .unwrap();
    let item_a = document
        .get_element_by_id("kn-lb-typeahead2-item-a")
        .unwrap();
    let item_b = document
        .get_element_by_id("kn-lb-typeahead2-item-b")
        .unwrap();
    let item_c = document
        .get_element_by_id("kn-lb-typeahead2-item-c")
        .unwrap();

    content.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    content.dispatch_event(&keydown_event("p")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));

    sleep_ms((TYPEAHEAD_TIMEOUT_MS as i32) + 200).await;
    content.dispatch_event(&keydown_event("b")).unwrap();
    assert!(item_c.has_attribute("data-highlighted"));
    assert!(!item_b.has_attribute("data-highlighted"));
}

/// 検証 9（計画書 §3.4）: Enter/Space は highlight 中の非 disabled 項目へ
/// `click()` を合成する。合成 click の発火をテストローカルのリスナーで検証
/// し（`crate::headless::MAPPING_TABLE` が `listbox` 行を持たないため
/// `aria-selected` の変化は assert しない）、disabled 項目が highlight 中の
/// ときは click が発火しないことも確認する。
#[wasm_bindgen_test]
fn listbox_enter_and_space_synthesize_click_on_highlighted_item() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-activate1",
        &[("a", "A", false), ("b", "B", true)],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document
        .get_element_by_id("kn-lb-activate1-content")
        .unwrap();
    let item_a = document
        .get_element_by_id("kn-lb-activate1-item-a")
        .unwrap();
    let item_b = document
        .get_element_by_id("kn-lb-activate1-item-b")
        .unwrap();

    let click_count = std::rc::Rc::new(std::cell::Cell::new(0));
    let click_closure = Closure::<dyn FnMut(Event)>::new({
        let click_count = click_count.clone();
        move |_event: Event| {
            click_count.set(click_count.get() + 1);
        }
    });
    item_a
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    item_b
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .unwrap();
    click_closure.forget();

    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));
    content.dispatch_event(&keydown_event("Enter")).unwrap();
    assert_eq!(
        click_count.get(),
        1,
        "Enter は highlight 中の項目へ click を合成すべき"
    );

    content.dispatch_event(&keydown_event(" ")).unwrap();
    assert_eq!(
        click_count.get(),
        2,
        "Space（バッファ非活性時）も highlight 中の項目へ click を合成すべき"
    );

    // B（disabled）が highlight 中の状態を検証する。ArrowDown は disabled
    // 項目をスキップする既存仕様（`listbox_arrow_navigation_skips_disabled_items`
    // 参照）のため、通常のキーボード操作では B へ到達できない。本テストは
    // 「highlight 中の項目が disabled だった場合」の fail-closed no-op を
    // 検証する防御的なケースのため、`menu_open_arrow_right_on_disabled_trigger_item_is_noop`
    // と同じパターンで highlight を直接 DOM へ設定する。
    item_a.remove_attribute("data-highlighted").unwrap();
    item_b.set_attribute("data-highlighted", "").unwrap();
    content
        .set_attribute(
            "aria-activedescendant",
            &item_b.get_attribute("id").unwrap(),
        )
        .unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    content.dispatch_event(&keydown_event("Enter")).unwrap();
    assert_eq!(
        click_count.get(),
        2,
        "disabled 項目が highlight 中のときは click が発火しないべき（fail-closed）"
    );
}

/// 検証 10（計画書 §3.3）: Escape は Menu/Select と非対称に、typeahead
/// バッファのみをリセットし highlight（`data-highlighted`/
/// `aria-activedescendant`）は維持する。直後の文字入力は新規バッファとして
/// 扱われる（Escape 前の入力と連結されない）。
#[wasm_bindgen_test]
fn listbox_escape_resets_typeahead_buffer_without_clearing_highlight() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-escape1",
        &[("a", "Apple", false), ("b", "Banana", false)],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-escape1-content").unwrap();
    let item_a = document.get_element_by_id("kn-lb-escape1-item-a").unwrap();
    let item_b = document.get_element_by_id("kn-lb-escape1-item-b").unwrap();

    content.dispatch_event(&keydown_event("a")).unwrap();
    assert!(item_a.has_attribute("data-highlighted"));

    let not_default_prevented = content.dispatch_event(&keydown_event("Escape")).unwrap();
    assert!(
        not_default_prevented,
        "Escape は prevent_default されるべきではない（親ダイアログ閉鎖を奪わない）"
    );
    assert!(
        item_a.has_attribute("data-highlighted"),
        "Escape で highlight はクリアされないべき（reopen 契約が無い Listbox）"
    );
    assert_eq!(
        content.get_attribute("aria-activedescendant").as_deref(),
        Some("kn-lb-escape1-item-a")
    );

    // Escape 直後の "b" は新規バッファとして Banana にマッチすべき
    // （Escape 前の "a" と連結され "ab" として扱われてはいけない）。
    content.dispatch_event(&keydown_event("b")).unwrap();
    assert!(item_b.has_attribute("data-highlighted"));
    assert!(!item_a.has_attribute("data-highlighted"));
}

/// 検証 11（XSS 回帰、REQ-1）: 攻撃者制御文字列を持つラベルに対し矢印移動・
/// typeahead・Enter を行っても `script` 要素が DOM に生成されない
/// （`menu_typeahead_with_attacker_controlled_label_does_not_inject_script`
/// と同型）。
#[wasm_bindgen_test]
fn listbox_keyboard_navigation_with_attacker_controlled_label_does_not_inject_script() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = build_listbox_dom(
        &document,
        "kn-lb-xss1",
        &[
            ("a", "<script>document.title='pwned'</script>", false),
            ("b", "B", false),
        ],
        None,
        None,
    );
    let _cleanup = RemoveOnDrop(root.clone());
    let original_title = document.title();
    wire_keynav(root.clone()).expect("wire_keynav must succeed");

    let content = document.get_element_by_id("kn-lb-xss1-content").unwrap();

    content.dispatch_event(&keydown_event("ArrowDown")).unwrap();
    content.dispatch_event(&keydown_event("<")).unwrap();
    content.dispatch_event(&keydown_event("Enter")).unwrap();

    assert!(root.query_selector("script").unwrap().is_none());
    assert_eq!(
        document.title(),
        original_title,
        "攻撃者制御ラベルの操作で document.title が変化してはいけない"
    );
}
