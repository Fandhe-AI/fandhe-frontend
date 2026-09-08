//! Command（`fandhe-frontend-headless-ui` `command` モジュール）の入力絞り込み・
//! 矢印キー選択・Enter 実行・dialog 開閉（Cmd/Ctrl+K）配線（イシュー #2069、
//! 親 #2067、祖父トラッキング参照軸 #2001）。
//!
//! # 背景
//!
//! `crates/headless-ui/src/command.rs`（イシュー #2068）は Root/Input/List/
//! Empty/Group/GroupHeading/Item/Shortcut/Separator/Dialog の 10 anatomy
//! パーツと `Command` 状態機械（`Disclosure` + `TextInput` + `SingleSelect`）
//! までを提供する一方、実 DOM 上のイベント配線（入力→絞り込み反映・矢印
//! キーによる行選択・Enter による実行フック・Cmd/Ctrl+K での dialog 開閉・
//! `focus_trap`/`overlay` への `"command"` scope 対応）を本クレートへ
//! 申し送っていた（同モジュール冒頭 doc「out-of-scope」節）。本モジュールが
//! その配線を実装する。
//!
//! # 設計（`number_input.rs`/`angle_slider.rs`/`headless_signature_pad.rs` と
//! 同型の 2 層構成）
//!
//! - 純粋ロジック層（本モジュール上部）は web-sys に依存せず、native の
//!   `cargo test` で決定的に検証できる。
//! - 配線層（[`mod@wiring`]）のみ `#[cfg(target_arch = "wasm32")]` でゲート
//!   する。
//!
//! # アクション対応表
//!
//! | 契機 | dispatch | DOM 反映 |
//! |---|---|---|
//! | `input` 上の `input` | [`ACTION_INPUT`]（`data-action-input` があれば `crate::events::wire_events` が担い本モジュールは反映のみ） | 絞り込み反映（`hidden`/`data-empty`）+ 選択整合 |
//! | `input` 上の ArrowDown/ArrowUp/Home/End | [`ACTION_SELECT`] | `data-selected`/`aria-selected`/`aria-activedescendant` 同期 |
//! | `input` 上の Enter | [`ACTION_EXECUTE`]（`"command:execute"`） | なし |
//! | item クリック（非 disabled） | [`ACTION_SELECT`] → [`ACTION_EXECUTE`] | 選択同期 |
//! | `input` 上の Escape（open な `dialog` パーツ内のみ） | [`ACTION_CLOSE`] | なし |
//! | document 上の Cmd/Ctrl+K | [`ACTION_TOGGLE`] | 再描画後 open なら `input` へ `focus()` |
//!
//! `MAPPING_TABLE`（[`crate::headless`]）へ `(command, item) → "select"` 行は
//! **意図的に追加しない**（`docs/design/wasm-full-architecture.md` §12.3
//! 直下参照）。理由: item クリックは「行選択（`"select"`）」と「実行
//! （[`ACTION_EXECUTE`]）」の 2 アクションを要し、`MAPPING_TABLE` は
//! (scope, part) → 単一アクションの同期写像であるため単一行では表現
//! できず、行を足すと `crate::headless::wire_headless_component` を併用する
//! アプリで `"select"` が二重 dispatch される（`clipboard`/`angle_slider` と
//! 同型の「乗せない理由」）。
//!
//! # `"input"` dispatch と `crate::events::wire_events` の二重 dispatch 回避
//!
//! `crate::events::wire_events` は `input` イベントで `[data-action-input]`
//! を持つ要素を既に dispatch する。規則: **`input` パーツに
//! `data-action-input` 属性があれば dispatch は `wire_events` が担い、本
//! モジュールは DOM 反映のみ行う。無ければ本モジュールが [`ACTION_INPUT`]
//! を dispatch する**。両者は同一 `root` へ bubble 登録され、`wire_events`
//! が先に登録される（`crate::lib::Runtime::mount`/`hydrate` が
//! `events::wire_events` を先に呼ぶ）ため、本モジュールの DOM 反映は常に
//! アプリ再描画の後に走る。
//!
//! # 絞り込みの DOM 反映
//!
//! 判定は [`fandhe_frontend_headless_ui::command::filter_items`]（label のみ
//! 大文字小文字非区別 contains、空クエリは全件）を **そのまま呼ぶ**
//! （新規の絞り込みアルゴリズムを持ち込まない、SSR 側 `Command::filtered_items`
//! と同一結果を保証する）。各 item のラベルは `text_content()` から
//! `shortcut` パーツ子孫を除外して読む（除外しないと「Ctrl」「⌘」等の入力が
//! shortcut を持つ全 item に一致してしまう）。反映先は item（`hidden`）・
//! item を 1 件以上含みすべて hidden な group（`hidden`）・separator
//! （クエリ非空のとき `hidden`）の 3 種のみで、`dialog` の `hidden` には
//! 触れない。可視 item が 0 件のとき `data-empty` を root/list/empty へ
//! 付与する。絞り込み後、選択中 item が hidden/未選択なら先頭の可視・非
//! disabled item を自動選択し（cmdk の自動先頭選択）、可視 item が 0 件なら
//! `"deselect"`（`crate::events::wire_events` 経由ではなく本モジュールの
//! [`ACTION_DESELECT`]）を dispatch して `aria-activedescendant` を除去する。
//!
//! # 行選択（矢印キー）
//!
//! 候補列は同一インスタンス配下の非 `hidden` item（[`crate::keynav`] の
//! `disabled_flags`/`highlight_next_index`/`menu_loop_focus_from_attr` を
//! そのまま再利用する）。cmdk は既定で非循環（`loop_focus` 既定 `false`、
//! `data-loop-focus="true"` で opt-in）。`data-highlighted` は書かない
//! （`fandhe-frontend-headless-ui` の command は `data-highlighted` を出力
//! しない契約、`crates/headless-ui/src/command.rs` モジュール doc「`item`
//! の選択表現」節参照）。
//!
//! # `OverlayKind::Command` と Escape の収束
//!
//! [`crate::overlay::OverlayKind::Command`]（`from_scope("command")`）を
//! 追加した。既定値は Dialog と同じ（`close_on_escape`/
//! `close_on_interact_outside`/`outside_dismiss_blocks_propagation_by_default`
//! いずれも `true`）。呼び出し側（#580 統合層）が
//! `OverlayCloseController` からの閉鎖要求を受けて dispatch すべき名前は
//! [`ACTION_CLOSE`]（`"close"`、`CommandAction::Close` は冪等）。
//!
//! 本モジュールの Escape（`input` 上 bubble）→ [`ACTION_CLOSE`] dispatch と、
//! `crate::overlay::wiring::OverlayCloseController`（document 上）→
//! アプリの `"close"` dispatch は、どちらが先でも `Disclosure::Close` の
//! 冪等性により同一 closed 状態へ収束する（`crate::overlay` モジュール doc
//! 「keynav との二重処理の収束」節と同型の分析）。
//!
//! # `focus_trap::should_trap`
//!
//! `data-scope` が `"dialog"` **または** `"command"` かつ `aria-modal="true"`
//! で `true` を返すよう [`crate::focus_trap::should_trap`] を拡張した
//! （`command::dialog` は `aria-modal="true"` + `tabindex="-1"` を固定
//! 出力するため）。
//!
//! # 既知の制限
//!
//! - [`ACTION_TOGGLE`]/[`ACTION_CLOSE`]/[`ACTION_SELECT`]/[`ACTION_INPUT`]/
//!   [`ACTION_DESELECT`] は headless `Command::decode_action` の語彙に合わせた
//!   裸のアクション名であり、同一 root に他の Disclosure 系部品を合成する
//!   アプリでは `decode_action` 側での衝突があり得る（[`ACTION_EXECUTE`]の
//!   みが名前空間付き）。複数の command インスタンスが同一 root にある構成の
//!   識別は本イシューの対象外。
//! - Cmd/Ctrl+K は複数 `dialog` パーツがあっても常に 1 回・payload 空文字列
//!   の `"toggle"` を dispatch する（単一インスタンス前提）。
//! - cmdk の Alt+Arrow（group 単位ジャンプ）・Meta+Arrow（先頭/末尾）等の
//!   修飾キー付き操作は未対応（修飾キー付きは no-op、既存方針どおり）。
//! - `empty` パーツの live region（`aria-live`）通知は実装しない（後続課題
//!   として起票を提案する、`.claude/rules/out-of-scope-tracking.md`）。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列を一切組み立てない。DOM 反映は `set_attribute`/
//!   `remove_attribute`/`text_content` 読み取り/`focus()` のみ。dispatch
//!   payload（`input.value`、`data-value`）は文字列のまま渡し、再描画時の
//!   エスケープは `fandhe_frontend_core::render` が担う。
//! - `aria-controls`/`closest` 経由で解決した要素はすべて配線 `root` の
//!   `contains` を検査してから採用する（改ざんされた `aria-controls` で
//!   root 外の要素を操作しない、`crate::keynav` の「Stale root」教訓と
//!   同型）。document 上の Cmd/Ctrl+K も `root` 配下に `dialog` パーツが
//!   存在するときのみ発火する。
//! - `Closure::forget` はマウント時の定数回（root 3 + document 1）に限定
//!   する（A04 対策、無制限リークの構造的回避）。
//! - 未知キー・修飾キー付き・IME 変換中・`disabled`/`data-disabled`・
//!   `data-value` 欠落・未選択 Enter・hidden な選択・`dialog` 不在の
//!   Escape/Cmd+K はすべて no-op（fail-closed）。`Command::decode_action` が
//!   [`ACTION_EXECUTE`] を未知アクションとして無視する二重の安全網も働く。

use crate::keynav::Modifiers;

/// `"input"` dispatch のアクション名（`CommandAction::Input` が受理する
/// 固定語彙、`crates/headless-ui/src/command.rs::Command::decode_action`）。
pub const ACTION_INPUT: &str = "input";
/// `"select"` dispatch のアクション名（`CommandAction::Select`）。
pub const ACTION_SELECT: &str = "select";
/// `"deselect"` dispatch のアクション名（`CommandAction::Deselect`）。
pub const ACTION_DESELECT: &str = "deselect";
/// 実行フック dispatch のアクション名。裸の `"select"`/`"input"` 等と異なり
/// 名前空間を付ける（`crate::headless_clipboard` の `"clipboard:"` と同じ
/// 理由: Runtime が無条件配線するため、裸の名前はアプリ独自のアクションと
/// 衝突しうる）。`CommandAction::decode_action` は未知アクションとして無視
/// するため（fail-closed）、アプリの `decode_action` がこの名前を捕捉して
/// 実行・閉鎖する想定。
pub const ACTION_EXECUTE: &str = "command:execute";
/// `"close"` dispatch のアクション名（`CommandAction::Close`、冪等）。
pub const ACTION_CLOSE: &str = "close";
/// `"toggle"` dispatch のアクション名（`CommandAction::Toggle`）。
pub const ACTION_TOGGLE: &str = "toggle";

/// [`command_key_action`] が決定するキー操作の種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKeyAction {
    /// ArrowDown/ArrowUp/Home/End による行選択の再計算
    /// （[`crate::keynav::highlight_next_index`] へ委譲）。
    MoveSelection,
    /// Enter による実行フック dispatch（[`ACTION_EXECUTE`]）。
    Execute,
    /// Escape による dialog 閉鎖要求（open な dialog 内でのみ発生）。
    Close,
}

/// `input` パーツ上の keydown から [`CommandKeyAction`] を決定する純粋関数
/// （web-sys 非依存、native `cargo test` で検証可能）。
///
/// 修飾キー付きは常に `None`（no-op）。`in_open_dialog` は呼び出し側
/// （配線層）が「`input` が open な `dialog` パーツの子孫かどうか」を
/// 解決してから渡す（dialog を持たない構成・closed な dialog の Escape は
/// no-op、`crates/headless-ui/src/command.rs` モジュール doc「`dialog`
/// パーツを `crate::dialog` へ委譲しない理由」節参照）。
#[must_use]
pub fn command_key_action(
    key: &str,
    modifiers: Modifiers,
    in_open_dialog: bool,
) -> Option<CommandKeyAction> {
    if modifiers.any() {
        return None;
    }
    match key {
        "ArrowDown" | "ArrowUp" | "Home" | "End" => Some(CommandKeyAction::MoveSelection),
        "Enter" => Some(CommandKeyAction::Execute),
        "Escape" if in_open_dialog => Some(CommandKeyAction::Close),
        _ => None,
    }
}

/// document 上の keydown が Cmd/Ctrl+K（dialog 開閉ショートカット）かどうか
/// を判定する純粋関数。
///
/// `key` が `"k"`/`"K"`（ASCII 大文字小文字無視）かつ `ctrl` と `meta` の
/// ちょうど一方のみが押されている（XOR）かつ `alt` が押されていないときに
/// `true`。Windows/Linux（Ctrl+K）・macOS（Cmd+K）の双方を受理しつつ、
/// `Ctrl+Alt+K`/`Ctrl+Meta+K` のような OS ショートカットとの衝突が疑わしい
/// 組み合わせは対象外とする。
#[must_use]
pub fn is_toggle_shortcut(key: &str, modifiers: Modifiers) -> bool {
    key.eq_ignore_ascii_case("k") && (modifiers.ctrl ^ modifiers.meta) && !modifiers.alt
}

/// `visible` 列の中で 1 件でも可視な item を含むグループかどうかを判定する
/// 純粋関数（group の `hidden` 反映用）。`item_count` はそのグループが
/// 含む item の総数（0 件のグループ、例えば見出しのみのプレースホルダは
/// 対象外とし `hidden` を付与しない）。
#[must_use]
pub fn group_should_hide(item_count: usize, visible_count: usize) -> bool {
    item_count > 0 && visible_count == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods() -> Modifiers {
        Modifiers::default()
    }

    // --- command_key_action ---

    #[test]
    fn arrow_and_home_end_map_to_move_selection() {
        for key in ["ArrowDown", "ArrowUp", "Home", "End"] {
            assert_eq!(
                command_key_action(key, mods(), false),
                Some(CommandKeyAction::MoveSelection),
                "key={key}"
            );
        }
    }

    #[test]
    fn enter_maps_to_execute_regardless_of_dialog_state() {
        assert_eq!(
            command_key_action("Enter", mods(), false),
            Some(CommandKeyAction::Execute)
        );
        assert_eq!(
            command_key_action("Enter", mods(), true),
            Some(CommandKeyAction::Execute)
        );
    }

    #[test]
    fn escape_maps_to_close_only_within_open_dialog() {
        assert_eq!(
            command_key_action("Escape", mods(), true),
            Some(CommandKeyAction::Close)
        );
        assert_eq!(command_key_action("Escape", mods(), false), None);
    }

    #[test]
    fn modifiers_disable_all_key_actions() {
        let with_ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        for key in ["ArrowDown", "Enter", "Escape"] {
            assert_eq!(command_key_action(key, with_ctrl, true), None, "key={key}");
        }
    }

    #[test]
    fn unknown_key_is_none() {
        assert_eq!(command_key_action("a", mods(), true), None);
    }

    // --- is_toggle_shortcut ---

    #[test]
    fn ctrl_k_and_meta_k_are_toggle_shortcuts() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        let meta = Modifiers {
            meta: true,
            ..Modifiers::default()
        };
        assert!(is_toggle_shortcut("k", ctrl));
        assert!(is_toggle_shortcut("K", ctrl));
        assert!(is_toggle_shortcut("k", meta));
    }

    #[test]
    fn ctrl_and_meta_together_is_not_toggle_shortcut() {
        let both = Modifiers {
            ctrl: true,
            meta: true,
            ..Modifiers::default()
        };
        assert!(!is_toggle_shortcut("k", both));
    }

    #[test]
    fn alt_disqualifies_toggle_shortcut() {
        let ctrl_alt = Modifiers {
            ctrl: true,
            alt: true,
            ..Modifiers::default()
        };
        assert!(!is_toggle_shortcut("k", ctrl_alt));
    }

    #[test]
    fn no_modifier_is_not_toggle_shortcut() {
        assert!(!is_toggle_shortcut("k", mods()));
    }

    #[test]
    fn non_k_key_is_not_toggle_shortcut() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        assert!(!is_toggle_shortcut("j", ctrl));
    }

    // --- group_should_hide ---

    #[test]
    fn group_with_items_all_hidden_should_hide() {
        assert!(group_should_hide(3, 0));
    }

    #[test]
    fn group_with_some_visible_should_not_hide() {
        assert!(!group_should_hide(3, 1));
    }

    #[test]
    fn empty_group_never_hides() {
        assert!(!group_should_hide(0, 0));
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`number_input.rs`/`angle_slider.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        command_key_action, is_toggle_shortcut, ACTION_CLOSE, ACTION_DESELECT, ACTION_EXECUTE,
        ACTION_INPUT, ACTION_SELECT, ACTION_TOGGLE,
    };
    use crate::events::ActionRef;
    use crate::keynav::wiring::{
        closest, collect_parts, disabled_flags, modifiers_of, set_dom_attribute,
    };
    use crate::keynav::{highlight_next_index, menu_loop_focus_from_attr};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, HtmlElement, HtmlInputElement, KeyboardEvent};

    /// Command の `data-scope` 属性値（`fandhe_frontend_headless_ui::command`
    /// の `ANATOMY` と一致）。
    const SCOPE: &str = "command";
    /// Input パーツの `data-part` 属性値（[`matches_part`] の比較対象）。
    const INPUT_PART: &str = "input";
    /// Empty パーツの `data-part` 属性値（[`empty_reflect_targets`] の
    /// セレクタ組み立てに使う）。
    const EMPTY_PART: &str = "empty";

    /// `[data-scope="command"][data-part="root"]` セレクタ。
    const ROOT_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"root\"]";
    /// `[data-scope="command"][data-part="input"]` セレクタ。
    const INPUT_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"input\"]";
    /// `[data-scope="command"][data-part="item"]` セレクタ。
    const ITEM_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"item\"]";
    /// `[data-scope="command"][data-part="group"]` セレクタ。
    const GROUP_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"group\"]";
    /// `[data-scope="command"][data-part="separator"]` セレクタ。
    const SEPARATOR_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"separator\"]";
    /// `[data-scope="command"][data-part="shortcut"]` セレクタ。
    const SHORTCUT_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"shortcut\"]";
    /// `[data-scope="command"][data-part="dialog"]` セレクタ。
    const DIALOG_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"dialog\"]";
    /// `data-action-input` 属性セレクタ断片
    /// （[`crate::events::ACTION_INPUT_ATTR`] と同じ属性契約。`input` パーツが
    /// これを持つ場合は `crate::events::wire_events` が dispatch を担う、
    /// モジュール冒頭 doc「二重 dispatch 回避」節参照）。
    const ACTION_INPUT_ATTR: &str = crate::events::ACTION_INPUT_ATTR;

    /// `root` 配下の Command へ input/keydown/click（計 3 回）、`document` へ
    /// keydown（1 回、Cmd/Ctrl+K）を配線する（マウント時 1 回契約、
    /// `Closure::forget` は本関数呼び出しにつき定数 4 回に限定する、
    /// モジュール冒頭 doc「セキュリティ不変条件」節参照）。
    ///
    /// `on_action` は dispatch 依頼を呼び出し側へ渡すのみで、状態更新・DOM
    /// 反映は行わない（`number_input::wire_number_input_events` と同じ責務
    /// 分離）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`（いずれかのリスナー登録）の失敗を
    /// 伝播する。
    pub fn wire_command_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        let input_root = root.clone();
        let input_on_action = on_action.clone();
        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_input(&input_root, &event, &input_on_action);
        });
        root.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
        input_closure.forget();

        let keydown_root = root.clone();
        let keydown_on_action = on_action.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &keydown_on_action);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let click_root = root.clone();
        let click_on_action = on_action.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &click_on_action);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        if let Some(window) = web_sys::window() {
            let document = window.document();
            if let Some(document) = document {
                let doc_root = root;
                let doc_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                    handle_document_keydown(&document, &doc_root, &event, &on_action);
                });
                web_sys::EventTarget::from(window).add_event_listener_with_callback(
                    "keydown",
                    doc_closure.as_ref().unchecked_ref(),
                )?;
                doc_closure.forget();
            }
        }

        Ok(())
    }

    /// `element` が `data-scope="command"` かつ `data-part == part` に一致
    /// するかどうか。
    fn matches_part(element: &Element, part: &str) -> bool {
        element.get_attribute("data-scope").as_deref() == Some(SCOPE)
            && element.get_attribute("data-part").as_deref() == Some(part)
    }

    /// `start` から祖先方向へ `ROOT_SELECTOR` を辿って Command インスタンス
    /// の root を解決し、`wired_root`（配線登録時の root、Runtime root まで
    /// 及び得る）に含まれることを確認する（改ざんされた `aria-controls`/
    /// 越境操作を避ける fail-closed、モジュール冒頭 doc「セキュリティ
    /// 不変条件」節参照）。
    fn resolve_instance_root(wired_root: &Element, start: &Element) -> Option<Element> {
        let instance = closest(start, ROOT_SELECTOR)?;
        wired_root.contains(Some(&instance)).then_some(instance)
    }

    /// `input` の `aria-controls` から List パーツを解決し、`wired_root` に
    /// 含まれることを確認する（fail-closed）。
    fn resolve_list(wired_root: &Element, input: &Element) -> Option<Element> {
        let list_id = input.get_attribute("aria-controls")?;
        let document = input.owner_document()?;
        let list = document.get_element_by_id(&list_id)?;
        wired_root.contains(Some(&list)).then_some(list)
    }

    /// `list` 配下の Item パーツを document 順に収集し、「最近接 root 祖先が
    /// `instance_root` と一致する」ものだけへ絞り込む（`crate::keynav` の
    /// `collect_tree_items` と同型の越境防止。ネスト別インスタンス・別
    /// Command への越境操作を防ぐ）。
    fn collect_own_items(list: &Element, instance_root: &Element) -> Vec<Element> {
        collect_parts(list, ITEM_SELECTOR)
            .into_iter()
            .filter(|item| {
                closest(item, ROOT_SELECTOR).is_some_and(|nearest| nearest == *instance_root)
            })
            .collect()
    }

    /// `item` の表示ラベルを `shortcut` パーツ子孫を除いた `text_content()`
    /// として読み取る（`crate::keynav::item_label` と同型の「クローン上から
    /// 除去してから読む」方針、モジュール冒頭 doc「絞り込みの DOM 反映」
    /// 節参照）。クローン失敗時は空文字列（安全側、絞り込み対象から除外
    /// される＝非表示側へ倒れる fail-closed）。
    fn item_label_excluding_shortcut(item: &Element) -> String {
        let Ok(clone) = item.clone_node_with_deep(true) else {
            return String::new();
        };
        let Ok(clone) = clone.dyn_into::<Element>() else {
            return String::new();
        };
        if let Ok(shortcuts) = clone.query_selector_all(SHORTCUT_SELECTOR) {
            for i in 0..shortcuts.length() {
                if let Some(node) = shortcuts.item(i) {
                    if let Ok(el) = node.dyn_into::<Element>() {
                        el.remove();
                    }
                }
            }
        }
        clone.text_content().unwrap_or_default().trim().to_string()
    }

    /// `items` を [`fandhe_frontend_headless_ui::command::filter_items`]
    /// （SSR 側 `Command::filtered_items` と同一の絞り込みアルゴリズム）へ
    /// 委譲して可視フラグ列を返す（モジュール冒頭 doc「絞り込みの DOM
    /// 反映」節参照。新規の絞り込みアルゴリズムを持ち込まない）。
    fn visible_flags(items: &[Element], query: &str) -> Vec<bool> {
        let labels: Vec<String> = items.iter().map(item_label_excluding_shortcut).collect();
        let index_keys: Vec<String> = (0..labels.len()).map(|i| i.to_string()).collect();
        let pairs: Vec<(&str, &str)> = index_keys
            .iter()
            .zip(labels.iter())
            .map(|(k, l)| (k.as_str(), l.as_str()))
            .collect();
        let matched: std::collections::HashSet<usize> =
            fandhe_frontend_headless_ui::command::filter_items(&pairs, query)
                .into_iter()
                .filter_map(|(k, _)| k.parse::<usize>().ok())
                .collect();
        (0..items.len()).map(|i| matched.contains(&i)).collect()
    }

    /// 入力イベント（[`ACTION_INPUT`] dispatch の有無に関わらず）ごとに
    /// 呼ばれる絞り込み DOM 反映本体。`instance_root`/`list`/`input` は
    /// 呼び出し側が解決済みのものを渡す。`on_action` は絞り込み後の選択
    /// 整合（[`sync_selection_after_filter`]）が [`ACTION_SELECT`]/
    /// [`ACTION_DESELECT`] を dispatch するために使う（アプリ状態
    /// （`SingleSelect`）を実際に更新しないと、次回の再描画で DOM 直書き
    /// 分が巻き戻ってしまうため）。
    fn reflect_filter(
        instance_root: &Element,
        list: &Element,
        input: &Element,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let query = input
            .clone()
            .dyn_into::<HtmlInputElement>()
            .map(|el| el.value())
            .unwrap_or_default();
        let items = collect_own_items(list, instance_root);
        let visible = visible_flags(&items, &query);

        for (item, &is_visible) in items.iter().zip(visible.iter()) {
            if is_visible {
                let _ = item.remove_attribute("hidden");
            } else {
                set_dom_attribute(item, "hidden", "");
            }
        }

        // group: 自身が包含する item（`ITEM_SELECTOR`）の可視状態を集計する。
        for group in collect_parts(list, GROUP_SELECTOR)
            .into_iter()
            .filter(|g| closest(g, ROOT_SELECTOR).is_some_and(|r| r == *instance_root))
        {
            let own_items = collect_parts(&group, ITEM_SELECTOR);
            let item_count = own_items.len();
            let visible_count = own_items
                .iter()
                .filter(|it| !it.has_attribute("hidden"))
                .count();
            if super::group_should_hide(item_count, visible_count) {
                set_dom_attribute(&group, "hidden", "");
            } else {
                let _ = group.remove_attribute("hidden");
            }
        }

        // separator: クエリ非空のとき hidden（cmdk 準拠）。
        let query_is_empty = query.is_empty();
        for separator in collect_parts(list, SEPARATOR_SELECTOR)
            .into_iter()
            .filter(|s| closest(s, ROOT_SELECTOR).is_some_and(|r| r == *instance_root))
        {
            if query_is_empty {
                let _ = separator.remove_attribute("hidden");
            } else {
                set_dom_attribute(&separator, "hidden", "");
            }
        }

        let visible_count = visible.iter().filter(|&&v| v).count();
        let is_empty = visible_count == 0;
        for target in empty_reflect_targets(instance_root, list) {
            if is_empty {
                set_dom_attribute(&target, "data-empty", "");
            } else {
                let _ = target.remove_attribute("data-empty");
            }
        }

        sync_selection_after_filter(input, &items, &visible, on_action);
    }

    /// `data-empty` を反映する 3 要素（root/list/empty）を集める。`empty`
    /// パーツは `root` の直接の子として置かれる契約（`crates/headless-ui/
    /// src/command.rs` モジュール doc「`empty`/`separator` の配置制約」
    /// 節）だが、越境防止のため `instance_root` 配下から `EMPTY_PART` を
    /// 探索する（`collect_parts` は子孫全体を対象にするため直接の子孫関係を
    /// 前提にしない）。
    fn empty_reflect_targets(instance_root: &Element, list: &Element) -> Vec<Element> {
        let mut targets = vec![instance_root.clone(), list.clone()];
        targets.extend(
            collect_parts(
                instance_root,
                format!("[data-scope=\"{SCOPE}\"][data-part=\"{EMPTY_PART}\"]").as_str(),
            )
            .into_iter()
            .filter(|e| closest(e, ROOT_SELECTOR).is_some_and(|r| r == *instance_root)),
        );
        targets
    }

    /// 絞り込み後の選択整合（モジュール冒頭 doc「絞り込みの DOM 反映」
    /// 節）。選択中 item が hidden/未選択なら先頭の可視・非 disabled item を
    /// [`ACTION_SELECT`] dispatch + DOM 同期する（cmdk の自動先頭選択）。
    /// 可視 item が 0 件なら [`ACTION_DESELECT`] を dispatch し
    /// `aria-activedescendant` を除去する（アプリ状態
    /// （`SingleSelect`）を実際に更新しないと、次回の再描画で DOM 直書き
    /// 分が巻き戻ってしまうため、DOM 直書きに留めず dispatch する）。
    fn sync_selection_after_filter(
        input: &Element,
        items: &[Element],
        visible: &[bool],
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let disabled = disabled_flags(items);
        let already_selected_visible = items.iter().zip(visible.iter()).zip(disabled.iter()).any(
            |((item, &is_visible), &is_disabled)| {
                is_visible && !is_disabled && item.has_attribute("data-selected")
            },
        );
        if already_selected_visible {
            return;
        }
        let first_visible_non_disabled = items
            .iter()
            .zip(visible.iter())
            .zip(disabled.iter())
            .position(|((_, &is_visible), &is_disabled)| is_visible && !is_disabled);

        for item in items {
            let _ = item.remove_attribute("data-selected");
            set_dom_attribute(item, "aria-selected", "false");
        }

        match first_visible_non_disabled {
            Some(idx) => {
                let target = &items[idx];
                set_dom_attribute(target, "data-selected", "");
                set_dom_attribute(target, "aria-selected", "true");
                if let Some(id) = target.get_attribute("id") {
                    set_dom_attribute(input, "aria-activedescendant", &id);
                } else {
                    let _ = input.remove_attribute("aria-activedescendant");
                }
                if let Some(value) = target.get_attribute("data-value") {
                    if let Ok(mut cb) = on_action.try_borrow_mut() {
                        (cb)(ActionRef {
                            action: ACTION_SELECT.to_string(),
                            payload: value,
                        });
                    }
                }
            }
            None => {
                let _ = input.remove_attribute("aria-activedescendant");
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: ACTION_DESELECT.to_string(),
                        payload: String::new(),
                    });
                }
            }
        }
    }

    /// `visible_items[next_index]` を選択状態へ同期する（`data-selected`
    /// presence・`aria-selected`・`input` の `aria-activedescendant`）。
    fn sync_selection(input: &Element, visible_items: &[Element], next_index: usize) {
        for item in visible_items {
            let _ = item.remove_attribute("data-selected");
            set_dom_attribute(item, "aria-selected", "false");
        }
        let Some(target) = visible_items.get(next_index) else {
            return;
        };
        set_dom_attribute(target, "data-selected", "");
        set_dom_attribute(target, "aria-selected", "true");
        if let Some(id) = target.get_attribute("id") {
            set_dom_attribute(input, "aria-activedescendant", &id);
        } else {
            let _ = input.remove_attribute("aria-activedescendant");
        }
    }

    /// `input` イベント: `INPUT_PART` 上でのみ反応する。`data-action-input`
    /// があれば dispatch は `crate::events::wire_events` に委ねて DOM 反映
    /// のみ行い、無ければ [`ACTION_INPUT`] を dispatch する（モジュール
    /// 冒頭 doc「二重 dispatch 回避」節参照）。
    fn handle_input(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !root.contains(Some(target_element)) {
            return;
        }
        if !matches_part(target_element, INPUT_PART) {
            return;
        }
        let Some(instance_root) = resolve_instance_root(root, target_element) else {
            return;
        };
        let Some(list) = resolve_list(root, target_element) else {
            return;
        };

        if !target_element.has_attribute(ACTION_INPUT_ATTR) {
            let value = target_element
                .clone()
                .dyn_into::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(ActionRef {
                    action: ACTION_INPUT.to_string(),
                    payload: value,
                });
            }
        }

        reflect_filter(&instance_root, &list, target_element, on_action);
    }

    /// `input`/`root` まで祖先方向を辿り `data-disabled` の有無を判定する
    /// （`number_input.rs::has_disabled_ancestor` と同型）。
    fn has_disabled_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("disabled") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// keydown: Input パーツ上でのみ反応する（IME 変換中・disabled 祖先は
    /// no-op、モジュール冒頭 doc「セキュリティ不変条件」節参照）。
    fn handle_keydown(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !root.contains(Some(target_element)) {
            return;
        }
        if keyboard_event.is_composing() || keyboard_event.key_code() == 229 {
            return;
        }
        if !matches_part(target_element, INPUT_PART) {
            return;
        }
        if has_disabled_ancestor(root, target_element) {
            return;
        }
        let Some(instance_root) = resolve_instance_root(root, target_element) else {
            return;
        };

        // Escape のスコープ判定: input が open な dialog パーツの子孫か。
        let in_open_dialog = closest(target_element, DIALOG_SELECTOR)
            .is_some_and(|dialog| root.contains(Some(&dialog)) && !dialog.has_attribute("hidden"));

        let modifiers = modifiers_of(keyboard_event);
        let Some(key_action) = command_key_action(&keyboard_event.key(), modifiers, in_open_dialog)
        else {
            return;
        };
        keyboard_event.prevent_default();

        match key_action {
            super::CommandKeyAction::MoveSelection => {
                let Some(list) = resolve_list(root, target_element) else {
                    return;
                };
                let items = collect_own_items(&list, &instance_root);
                let visible_items: Vec<Element> = items
                    .into_iter()
                    .filter(|it| !it.has_attribute("hidden"))
                    .collect();
                let disabled = disabled_flags(&visible_items);
                let current = visible_items
                    .iter()
                    .position(|it| it.has_attribute("data-selected"));
                let loop_focus =
                    menu_loop_focus_from_attr(list.get_attribute("data-loop-focus").as_deref());
                let Some(next_index) = highlight_next_index(
                    current,
                    &keyboard_event.key(),
                    loop_focus,
                    modifiers,
                    &disabled,
                ) else {
                    return;
                };
                sync_selection(target_element, &visible_items, next_index);
                if let Some(value) = visible_items
                    .get(next_index)
                    .and_then(|el| el.get_attribute("data-value"))
                {
                    if let Ok(mut cb) = on_action.try_borrow_mut() {
                        (cb)(ActionRef {
                            action: ACTION_SELECT.to_string(),
                            payload: value,
                        });
                    }
                }
            }
            super::CommandKeyAction::Execute => {
                let Some(list) = resolve_list(root, target_element) else {
                    return;
                };
                let items = collect_own_items(&list, &instance_root);
                let Some(selected) = items.iter().find(|it| {
                    it.has_attribute("data-selected")
                        && !it.has_attribute("hidden")
                        && !it.has_attribute("data-disabled")
                }) else {
                    return;
                };
                let Some(value) = selected.get_attribute("data-value") else {
                    return;
                };
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: ACTION_EXECUTE.to_string(),
                        payload: value,
                    });
                }
            }
            super::CommandKeyAction::Close => {
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: ACTION_CLOSE.to_string(),
                        payload: String::new(),
                    });
                }
            }
        }
    }

    /// click: `ITEM_SELECTOR` 祖先を解決し、非 disabled なら
    /// [`ACTION_SELECT`] → [`ACTION_EXECUTE`] の順に dispatch し、選択 DOM
    /// を同期する（モジュール冒頭 doc「アクション対応表」節）。
    fn handle_click(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
            return;
        };
        if !root.contains(Some(&target_element)) {
            return;
        }
        let Some(item) = closest(&target_element, ITEM_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&item)) {
            return;
        }
        if item.has_attribute("data-disabled") {
            return;
        }
        let Some(value) = item.get_attribute("data-value") else {
            return;
        };
        let Some(instance_root) = resolve_instance_root(root, &item) else {
            return;
        };
        let Some(input) = instance_root.query_selector(INPUT_SELECTOR).ok().flatten() else {
            return;
        };

        event.stop_propagation();

        let items = {
            if let Some(list) = resolve_list(root, &input) {
                collect_own_items(&list, &instance_root)
            } else {
                Vec::new()
            }
        };
        if items.iter().any(|it| it.is_same_node(Some(&item))) {
            let visible_items: Vec<Element> = items
                .iter()
                .filter(|it| !it.has_attribute("hidden"))
                .cloned()
                .collect();
            if let Some(visible_idx) = visible_items
                .iter()
                .position(|it| it.is_same_node(Some(&item)))
            {
                sync_selection(&input, &visible_items, visible_idx);
            }
        }

        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: ACTION_SELECT.to_string(),
                payload: value.clone(),
            });
        }
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: ACTION_EXECUTE.to_string(),
                payload: value,
            });
        }
    }

    /// document 上の keydown: Cmd/Ctrl+K を判定し、`root` 配下に `dialog`
    /// パーツが存在するときのみ [`ACTION_TOGGLE`] を dispatch する。発火後
    /// は生きた DOM で `dialog` を再解決し、open なら配下の `input` へ
    /// `focus()` する（失敗は無視、fail-closed）。
    fn handle_document_keydown(
        _document: &Document,
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if keyboard_event.is_composing() || keyboard_event.key_code() == 229 {
            return;
        }
        let modifiers = modifiers_of(keyboard_event);
        if !is_toggle_shortcut(&keyboard_event.key(), modifiers) {
            return;
        }
        let Some(_dialog) = root.query_selector(DIALOG_SELECTOR).ok().flatten() else {
            return;
        };
        keyboard_event.prevent_default();
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: ACTION_TOGGLE.to_string(),
                payload: String::new(),
            });
        }
        // 再描画後の生きた DOM を再解決してから focus する。
        if let Some(dialog) = root.query_selector(DIALOG_SELECTOR).ok().flatten() {
            if !dialog.has_attribute("hidden") {
                if let Some(input) = dialog.query_selector(INPUT_SELECTOR).ok().flatten() {
                    if let Ok(html) = input.dyn_into::<HtmlElement>() {
                        let _ = html.focus();
                    }
                }
            }
        }
    }

    /// [`wire_command_events`] の dispatch を
    /// `fandhe_frontend_interactive::dispatch` へ接続し、成功時のみ
    /// `on_update` を呼ぶ利便関数（`number_input::wire_number_input_component`
    /// と同型）。
    ///
    /// # Errors
    ///
    /// [`wire_command_events`]（`add_event_listener_with_callback`）の失敗を
    /// 伝播する。
    pub fn wire_command_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: fandhe_frontend_interactive::Component + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
        let wired_root = root.clone();

        wire_command_events(root, move |action_ref: ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            if let Ok(mut cb) = on_update.try_borrow_mut() {
                (cb)(&state, &wired_root);
            }
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_command_component, wire_command_events};
