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
//! | `input` 上の Escape（open な `dialog` パーツ内・`data-close-on-escape="false"` でない場合のみ） | [`ACTION_CLOSE`] | なし |
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
//! # DOM 同期契約（codex-review P1 是正、イシュー #2069 実装後の追加指摘）
//!
//! dispatch（`on_action` 呼び出し）はアプリの再描画（構造フォールバック
//! による配線 `root` 配下の丸ごと差し替え）を誘発し得る。dispatch 前に
//! 解決した `Element` 参照（instance_root/list/input/item 等）は、その
//! dispatch の後には detach されている可能性があり、そのまま DOM へ書き
//! 込んでも表示に反映されない（`crate::angle_slider` の `PartKey`/
//! `resolve_part`〔イシュー #1956〕と同型の事情）。
//!
//! 本モジュールはこれを 2 つの規律で扱う。
//!
//! - **List パーツの `id` を再解決キーにする**: List パーツの `id`
//!   （[`fandhe_frontend_headless_ui::command::list`] の必須引数）は
//!   `view()` が再現する安定値であり、`crate::angle_slider::wiring::
//!   PartKey` と同じ役割を果たす。`handle_input`/[`reflect_filter`] は
//!   dispatch 前に読み取った要素参照を dispatch 後まで使い回さず、
//!   `list_id` から `document.get_element_by_id` で生きた DOM を
//!   再解決してから使う（[`resolve_command_parts_by_list_id`]）。
//! - **DOM への書き込みは、すべての dispatch が完了した後に 1 回だけ**:
//!   [`reflect_filter`] は「絞り込み結果 + 選択整合の判定（dispatch
//!   なし）→ 高々 1 回の dispatch → 再解決 → 書き込み」の順で進める。
//!   dispatch が挟まる中間段階では `hidden`/`data-selected` 等を一切
//!   書かない（先に書いた属性が後続の再描画で失われるのを防ぐ、
//!   `crates/wasm-full/tests/command_browser.rs` の
//!   `typing_query_after_full_subtree_replacement_still_reflects_filter`
//!   が固定する）。
//!
//! **既知の制限**: `data-action-input` を持つ input（下記「二重
//! dispatch 回避」節）の場合、`handle_input` が呼ばれる時点で
//! `crate::events::wire_events` の同一イベントに対する dispatch が
//! 既に走っている可能性がある。それが再描画を誘発していれば
//! `handle_input` 冒頭の `event.target()` は既に detach 済みだが、
//! `data-scope`/`data-part`/`aria-controls` の属性読み取りは detach
//! 済み要素でも成立するため、本モジュールは `target_element` 自体の
//! `root.contains` を要求せず、[`resolve_list`] が解決した List パーツ
//! への `root.contains` のみをセキュリティ境界として使う（詳細は
//! `handle_input` 内コメント）。
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
//! 「keynav との二重処理の収束」節と同型の分析）。**この分析は「dispatch
//! が起きるかどうか」の対称性のみを述べており、opt-out 属性
//! （`data-close-on-escape="false"`）そのものの尊重は別途保証が要る**
//! （codex-review P1 是正）。`OverlayCloseController` は `push_overlay`
//! 時点でこの属性を読んで登録要否を決めるが、本モジュールの Escape
//! ハンドラは独立に動作するため、無条件に [`ACTION_CLOSE`] を dispatch
//! すると opt-out が Command のスコープ内 Escape に対して効かなくなって
//! しまう。このため `wiring::handle_keydown` は Escape を
//! [`CommandKeyAction::Close`] と判定した後、open な `dialog` 要素へ
//! [`crate::overlay::close_on_escape_for`]（`OverlayKind::Command`）を
//! 直接適用し、`false` なら dispatch も `prevent_default()` も行わない
//! （`crates/wasm-full/tests/command_browser.rs::
//! escape_with_data_close_on_escape_false_is_noop` が固定する）。
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
    use crate::events::{ActionRef, AttrSource};
    use crate::keynav::wiring::{
        closest, collect_parts, disabled_flags, modifiers_of, set_dom_attribute,
    };
    use crate::keynav::{highlight_next_index, menu_loop_focus_from_attr};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, HtmlElement, HtmlInputElement, KeyboardEvent};

    /// `web_sys::Element` を [`AttrSource`] へ橋渡しする薄いラッパー
    /// （`overlay.rs::wiring::ElementAttrSource` と同じ意図の配線層専用
    /// アダプタ。Escape の opt-out 属性判定（[`super::super::overlay::
    /// close_on_escape_for`]）に使う）。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

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
    /// `[data-scope="command"][data-part="list"]` セレクタ（codex-review P1
    /// 是正: `document` 上の Cmd/Ctrl+K ハンドラが dispatch 前に `list_id` を
    /// 保持していない経路〔[`handle_document_keydown`]〕から、再描画後の
    /// `dialog` 配下の List パーツを直接解決するために使う）。
    const LIST_SELECTOR: &str = "[data-scope=\"command\"][data-part=\"list\"]";
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

    /// 絞り込み後の選択整合の判定結果（DOM 書き込み・dispatch を含まない
    /// 純粋な判定、[`selection_sync_plan`] が返す）。[`reflect_filter`] が
    /// dispatch 前の判定と、dispatch 後（再描画をまたぐ可能性がある）の
    /// DOM 反映の双方で同じロジックを共有するために切り出した
    /// （モジュール冒頭 doc「絞り込みの DOM 反映」節）。
    #[derive(Clone, Copy)]
    enum SelectionPlan {
        /// `items[idx]`（既に選択中・可視・非 disabled）はそのまま。
        /// 状態更新の dispatch は不要だが、DOM への `aria-activedescendant`
        /// 反映は省略しない（codex-review P1 是正: 呼び出し側が
        /// `aria-activedescendant` に `None` を渡して再描画した場合、
        /// この分岐が no-op だと `input` に `aria-activedescendant` が
        /// 一切設定されず選択中 item が支援技術へ伝わらないため）。
        Keep(usize),
        /// `items[idx]`（可視・非 disabled の先頭）を選択状態にする
        /// （cmdk の自動先頭選択）。
        Select(usize),
        /// 可視 item が 0 件。選択を解除する。
        Deselect,
    }

    /// 絞り込み後の選択整合を判定する純粋関数（DOM 書き込み・dispatch を
    /// 一切行わない）。
    fn selection_sync_plan(
        items: &[Element],
        visible: &[bool],
        disabled: &[bool],
    ) -> SelectionPlan {
        let already_selected_visible_idx = items
            .iter()
            .zip(visible.iter())
            .zip(disabled.iter())
            .position(|((item, &is_visible), &is_disabled)| {
                is_visible && !is_disabled && item.has_attribute("data-selected")
            });
        if let Some(idx) = already_selected_visible_idx {
            return SelectionPlan::Keep(idx);
        }
        match items
            .iter()
            .zip(visible.iter())
            .zip(disabled.iter())
            .position(|((_, &is_visible), &is_disabled)| is_visible && !is_disabled)
        {
            Some(idx) => SelectionPlan::Select(idx),
            None => SelectionPlan::Deselect,
        }
    }

    /// [`SelectionPlan`] から dispatch すべき [`ActionRef`] を決める
    /// （`Select` で対象 item に `data-value` が無い場合は dispatch しない
    /// ＝ `None`。DOM 反映自体は [`write_selection_plan`] が別途担う。
    /// アプリ状態（`SingleSelect`）を実際に更新しないと次回の再描画で
    /// DOM 直書き分が巻き戻ってしまうため、DOM 直書きに留めず dispatch
    /// する）。
    fn selection_plan_dispatch(items: &[Element], plan: SelectionPlan) -> Option<ActionRef> {
        match plan {
            SelectionPlan::Keep(_) => None,
            SelectionPlan::Select(idx) => {
                items
                    .get(idx)?
                    .get_attribute("data-value")
                    .map(|value| ActionRef {
                        action: ACTION_SELECT.to_string(),
                        payload: value,
                    })
            }
            SelectionPlan::Deselect => Some(ActionRef {
                action: ACTION_DESELECT.to_string(),
                payload: String::new(),
            }),
        }
    }

    /// [`SelectionPlan`] を DOM へ反映する（`data-selected`/`aria-selected`/
    /// `input` の `aria-activedescendant`。dispatch は行わない、
    /// [`selection_plan_dispatch`] と責務分離）。
    fn write_selection_plan(input: &Element, items: &[Element], plan: SelectionPlan) {
        match plan {
            SelectionPlan::Keep(idx) => {
                // `data-selected`/`aria-selected` は既に正しい（判定条件
                // そのもの）ため書き換えない。`aria-activedescendant` は
                // 呼び出し側の再描画が独立して決める値のため、再描画後も
                // 選択中 item を指すよう明示的に同期し直す（上記
                // `SelectionPlan::Keep` doc 参照）。
                let Some(target) = items.get(idx) else {
                    return;
                };
                if let Some(id) = target.get_attribute("id") {
                    set_dom_attribute(input, "aria-activedescendant", &id);
                } else {
                    let _ = input.remove_attribute("aria-activedescendant");
                }
            }
            SelectionPlan::Select(idx) => {
                for item in items {
                    let _ = item.remove_attribute("data-selected");
                    set_dom_attribute(item, "aria-selected", "false");
                }
                let Some(target) = items.get(idx) else {
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
            SelectionPlan::Deselect => {
                for item in items {
                    let _ = item.remove_attribute("data-selected");
                    set_dom_attribute(item, "aria-selected", "false");
                }
                let _ = input.remove_attribute("aria-activedescendant");
            }
        }
    }

    /// [`reflect_filter`] が dispatch 前後で `input` の入力フォーカスを
    /// 復元するためのスナップショット（codex-review P1 是正:
    /// `input`/`select` の dispatch が構造フォールバック再描画を誘発すると
    /// フォーカス中の `input` が削除され、`reflect_filter` の属性同期のみ
    /// ではフォーカスが戻らないため、dispatch 前に記録し再解決した
    /// `input` へ復元する）。
    struct FocusState {
        /// dispatch 前の時点で `input` が `document.active_element()` と
        /// 一致していたか。
        focused: bool,
        /// `focused` が `true` のときのみ意味を持つ選択範囲
        /// （`HtmlInputElement::selection_start`/`selection_end`）。
        selection_start: Option<u32>,
        selection_end: Option<u32>,
    }

    /// `input` の現在のフォーカス・選択範囲を記録する（DOM 変更なし）。
    fn capture_focus_state(input: &Element) -> FocusState {
        let focused = input
            .owner_document()
            .and_then(|doc| doc.active_element())
            .is_some_and(|active| active.is_same_node(Some(input)));
        let (selection_start, selection_end) = if focused {
            input
                .clone()
                .dyn_into::<HtmlInputElement>()
                .ok()
                .map(|el| {
                    (
                        el.selection_start().ok().flatten(),
                        el.selection_end().ok().flatten(),
                    )
                })
                .unwrap_or((None, None))
        } else {
            (None, None)
        };
        FocusState {
            focused,
            selection_start,
            selection_end,
        }
    }

    /// [`capture_focus_state`] で記録した状態を、再描画後に再解決した
    /// `input`（生きた DOM）へ復元する。`focused` が `false`（dispatch 前に
    /// フォーカスされていなかった）なら no-op。
    fn restore_focus_state(input: &Element, state: &FocusState) {
        if !state.focused {
            return;
        }
        let Ok(html) = input.clone().dyn_into::<HtmlElement>() else {
            return;
        };
        let _ = html.focus();
        if let (Ok(input_el), Some(start), Some(end)) = (
            input.clone().dyn_into::<HtmlInputElement>(),
            state.selection_start,
            state.selection_end,
        ) {
            let _ = input_el.set_selection_range(start, end);
        }
    }

    /// 入力イベント（[`ACTION_INPUT`] dispatch の有無に関わらず）ごとに
    /// 呼ばれる絞り込み DOM 反映本体。`root` は配線登録時の root、
    /// `list_id` は List パーツの `id`（[`resolve_list`] で確認済みの
    /// ものを呼び出し側が渡す）。
    ///
    /// # DOM 同期契約（codex-review P1 是正、`crate::angle_slider` の
    /// `PartKey`/`resolve_part` と同型）
    ///
    /// dispatch（`on_action` 呼び出し）はアプリの再描画（構造フォール
    /// バックによる `root` 配下の丸ごと差し替え）を誘発し得るため、要素
    /// 参照は dispatch をまたいで使い回さない。本関数は
    ///
    /// 1. `list_id` から生きた DOM（instance_root/list/input）を解決し、
    ///    絞り込み結果（`visible`）と選択整合の判定（[`SelectionPlan`]、
    ///    DOM 書き込みなし）を行う
    /// 2. 判定に応じて高々 1 回、[`ACTION_SELECT`]/[`ACTION_DESELECT`] を
    ///    dispatch する
    /// 3. dispatch が起きた場合は `list_id` から**再度**生きた DOM を
    ///    解決し直し、絞り込み結果・選択整合の判定も再計算する
    /// 4. ここまでに確定した最終状態を 1 回だけ DOM へ書き込む（`hidden`/
    ///    group/separator/`data-empty`/`data-selected`/`aria-selected`/
    ///    `aria-activedescendant`）
    ///
    /// の順で進める。dispatch 後に先に書いた `hidden` 等が再描画で失われる
    /// 問題（codex-review P1・Bugbot High）を避けるため、DOM への書き込みは
    /// 常にステップ 4（すべての dispatch が完了した後）にのみ集約する。
    /// 途中のいずれかの再解決に失敗した場合は、それ以降の反映を諦める
    /// （fail-closed）。
    ///
    /// `data-action-input` を持つ input（[`handle_input`] 冒頭 doc参照）の
    /// 場合、本関数が呼ばれる時点で `crate::events::wire_events` の同一
    /// イベントに対する dispatch が既に走っている可能性があるが、本関数は
    /// ステップ 1 で改めて生きた DOM を解決するため、この既存 dispatch に
    /// よる再描画の前後いずれで呼ばれても正しく動作する。
    fn reflect_filter(
        root: &Element,
        list_id: &str,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some((instance_root, list, input)) = resolve_command_parts_by_list_id(root, list_id)
        else {
            return;
        };
        let query = input
            .clone()
            .dyn_into::<HtmlInputElement>()
            .map(|el| el.value())
            .unwrap_or_default();
        // dispatch 前のフォーカス状態を記録する（codex-review P1 是正、
        // モジュール冒頭 doc「DOM 同期契約」節参照）。
        let focus_state = capture_focus_state(&input);
        let items = collect_own_items(&list, &instance_root);
        let visible = visible_flags(&items, &query);
        let plan = {
            let disabled = disabled_flags(&items);
            selection_sync_plan(&items, &visible, &disabled)
        };

        let dispatch = selection_plan_dispatch(&items, plan);
        let dispatched = dispatch.is_some();
        if let Some(action_ref) = dispatch {
            if let Ok(mut cb) = on_action.try_borrow_mut() {
                (cb)(action_ref);
            }
        }

        let (instance_root, list, input, items, visible, plan) = if dispatched {
            let Some((fresh_instance_root, fresh_list, fresh_input)) =
                resolve_command_parts_by_list_id(root, list_id)
            else {
                return;
            };
            let fresh_items = collect_own_items(&fresh_list, &fresh_instance_root);
            let fresh_visible = visible_flags(&fresh_items, &query);
            let fresh_plan = {
                let fresh_disabled = disabled_flags(&fresh_items);
                selection_sync_plan(&fresh_items, &fresh_visible, &fresh_disabled)
            };
            // dispatch が構造フォールバック再描画を誘発し `input` が
            // detach された場合、再解決した生きた `input` へフォーカス・
            // 選択範囲を復元する（codex-review P1 是正）。
            restore_focus_state(&fresh_input, &focus_state);
            (
                fresh_instance_root,
                fresh_list,
                fresh_input,
                fresh_items,
                fresh_visible,
                fresh_plan,
            )
        } else {
            (instance_root, list, input, items, visible, plan)
        };

        for (item, &is_visible) in items.iter().zip(visible.iter()) {
            if is_visible {
                let _ = item.remove_attribute("hidden");
            } else {
                set_dom_attribute(item, "hidden", "");
            }
        }

        // group: 自身が包含する item（`ITEM_SELECTOR`）の可視状態を集計する。
        for group in collect_parts(&list, GROUP_SELECTOR)
            .into_iter()
            .filter(|g| closest(g, ROOT_SELECTOR).is_some_and(|r| r == instance_root))
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
        for separator in collect_parts(&list, SEPARATOR_SELECTOR)
            .into_iter()
            .filter(|s| closest(s, ROOT_SELECTOR).is_some_and(|r| r == instance_root))
        {
            if query_is_empty {
                let _ = separator.remove_attribute("hidden");
            } else {
                set_dom_attribute(&separator, "hidden", "");
            }
        }

        let visible_count = visible.iter().filter(|&&v| v).count();
        let is_empty = visible_count == 0;
        for target in empty_reflect_targets(&instance_root, &list) {
            if is_empty {
                set_dom_attribute(&target, "data-empty", "");
            } else {
                let _ = target.remove_attribute("data-empty");
            }
        }

        write_selection_plan(&input, &items, plan);
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
        // `root.contains(target_element)` を先頭の早期リターンには使わない
        // （既知の制限、モジュール冒頭 doc「二重 dispatch 回避」節参照）:
        // `data-action-input` を持つ input の場合、本ハンドラが呼ばれる
        // 時点で既に `crate::events::wire_events`（同一 `root` へ先に登録
        // されたリスナー）の dispatch が走っている可能性があり、それが
        // 構造フォールバックの再描画を誘発していれば、この時点で
        // `target_element` は既に detach 済みで `root.contains` が偽になる
        // （detach は本モジュールの正常な制御フローが引き起こすものであり、
        // 攻撃者が操作できるものではない。detach 済みでも `get_attribute`
        // は要素自身のデータをそのまま返すため `matches_part` の判定は
        // 安全に成立する）。ここで containment を要求すると、この分岐の
        // 絞り込み反映が構造的に走らなくなってしまう。セキュリティ境界は
        // 後段（[`resolve_list`]・[`resolve_command_parts_by_list_id`]）が
        // 再解決した List/Root/Input それぞれへの `root.contains` 検査が
        // 担う（改ざんされた `aria-controls` で root 外を操作させない
        // fail-closed、モジュール冒頭 doc「セキュリティ不変条件」節）。
        if !matches_part(target_element, INPUT_PART) {
            return;
        }
        let Some(list) = resolve_list(root, target_element) else {
            return;
        };
        // 再描画をまたいで同じ List パーツを再解決するための識別子（下記
        // 「dispatch 後の再解決」節参照）。List パーツの `id` は
        // `fandhe_frontend_headless_ui::command::list` の必須引数であり
        // 常に安定して出力される。
        let list_id = list.id();

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

        // 絞り込み反映は [`reflect_filter`] に `list_id` だけを渡し、生きた
        // DOM からの再解決・dispatch・書き込みの順序統制を任せる
        // （codex-review P1 是正、[`reflect_filter`] doc「DOM 同期契約」
        // 節）。上の `on_action` 呼び出し・`wire_events` の既存 dispatch の
        // いずれが再描画を誘発していても、`target_element`/`list` を
        // そのまま使い回さないため正しく動作する。
        reflect_filter(root, &list_id, on_action);
    }

    /// [`handle_input`] が dispatch 後の再解決に使う: `list_id`
    /// （[`resolve_list`] が確認済みの List パーツ `id`）から、生きた DOM
    /// 上の instance_root/list/input の 3 点を再解決する
    /// （`crate::angle_slider::wiring::resolve_part` と同型の再解決契約）。
    ///
    /// `list_id` が空文字列（List パーツに `id` が付いていない構成異常）・
    /// `root` 配下に見つからない・instance_root に input が存在しない
    /// のいずれもフェイルクローズドに `None` を返す。
    fn resolve_command_parts_by_list_id(
        root: &Element,
        list_id: &str,
    ) -> Option<(Element, Element, Element)> {
        if list_id.is_empty() {
            return None;
        }
        let document = root.owner_document()?;
        let list = document.get_element_by_id(list_id)?;
        if !root.contains(Some(&list)) {
            return None;
        }
        let instance_root = closest(&list, ROOT_SELECTOR)?;
        if !root.contains(Some(&instance_root)) {
            return None;
        }
        let input = instance_root
            .query_selector(INPUT_SELECTOR)
            .ok()
            .flatten()?;
        if !root.contains(Some(&input)) {
            return None;
        }
        Some((instance_root, list, input))
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
        let open_dialog = closest(target_element, DIALOG_SELECTOR)
            .filter(|dialog| root.contains(Some(dialog)) && !dialog.has_attribute("hidden"));
        let in_open_dialog = open_dialog.is_some();

        let modifiers = modifiers_of(keyboard_event);
        let Some(key_action) = command_key_action(&keyboard_event.key(), modifiers, in_open_dialog)
        else {
            return;
        };

        // Escape による閉鎖要求は `crate::overlay` の opt-out 属性
        // （`data-close-on-escape="false"`）と一元化する
        // （codex-review P1 是正）。本モジュールの Escape ハンドラは
        // `crate::overlay::wiring::OverlayCloseController`（document 上）
        // とは独立に動作するため、同じ opt-out 属性を dialog 要素から
        // 直接読み取って揃える。`OverlayCloseController` 側は
        // push_overlay 時点でこの属性を読んで登録要否を決める設計
        // （overlay.rs モジュール doc参照）であり、本モジュールが無条件に
        // dispatch すると opt-out が Command のスコープ内 Escape に対して
        // 効かなくなってしまう（モジュール冒頭 doc「`OverlayKind::Command`
        // と Escape の収束」節は「どちらが先でも収束する」という dispatch
        // 有無の対称性のみを述べており、opt-out 属性そのものの尊重は別途
        // 本ガードが担う）。
        if key_action == super::CommandKeyAction::Close {
            let allow_close = open_dialog.as_ref().is_some_and(|dialog| {
                crate::overlay::close_on_escape_for(
                    crate::overlay::OverlayKind::Command,
                    &ElementAttrSource(dialog),
                )
            });
            if !allow_close {
                return;
            }
        }

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
                // 再描画をまたいで List パーツを再解決するための識別子
                // （[`handle_input`] と同じ契約、[`reflect_filter`] doc
                // 「DOM 同期契約」節参照）。
                let list_id = list.id();
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
                // dispatch が構造フォールバック再描画を誘発すると、上の
                // `sync_selection` による `hidden`/`data-selected`/
                // `aria-activedescendant` の直書きが失われ得る
                // （codex-review P1・Bugbot High 是正）。`reflect_filter`
                // に現在の絞り込み・選択状態の再反映を委ねる。
                reflect_filter(root, &list_id, on_action);
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
                // 再描画をまたいで List パーツを再解決するための識別子
                // （[`handle_input`]/`MoveSelection` 分岐と同じ契約）。
                let list_id = list.id();
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: ACTION_EXECUTE.to_string(),
                        payload: value,
                    });
                }
                // 実行先がダイアログを開いたまま再描画すると
                // `hidden`/`aria-activedescendant` が再び失われ得る
                // （codex-review P1 是正）。ダイアログが閉じる構成では
                // `reflect_filter` 内の再解決が fail-closed に no-op と
                // なるだけで安全に収束する。
                reflect_filter(root, &list_id, on_action);
            }
            super::CommandKeyAction::Close => {
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: ACTION_CLOSE.to_string(),
                        payload: String::new(),
                    });
                }
                // 処理済み Escape が document 上の
                // `OverlayCloseController`（親 Dialog 等）まで伝播して
                // 二重に閉じ処理を誘発しないよう、ここで消費する
                // （codex-review P1 是正）。`prevent_default()` だけでは
                // イベント伝播そのものは止まらないため、`root` 上の
                // 本リスナーで明示的に `stop_propagation()` する。
                keyboard_event.stop_propagation();
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

        let list = resolve_list(root, &input);
        // 再描画をまたいで List パーツを再解決するための識別子
        // （[`handle_input`]/[`handle_keydown`] と同じ契約、
        // [`reflect_filter`] doc「DOM 同期契約」節参照）。
        let list_id = list.as_ref().map(Element::id);
        let items = list
            .as_ref()
            .map(|list| collect_own_items(list, &instance_root))
            .unwrap_or_default();
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
        // dispatch が構造フォールバック再描画を誘発すると、上の
        // `sync_selection` による `hidden`/`data-selected`/
        // `aria-activedescendant` の直書きが失われ得る（codex-review P1・
        // Bugbot High 是正、[`handle_keydown`] の `MoveSelection` 分岐と
        // 同型）。続く `ACTION_EXECUTE` でダイアログが閉じる構成では
        // `reflect_filter` 内の再解決が fail-closed に no-op となるだけで
        // 安全に収束する。
        if let Some(list_id) = list_id.as_deref() {
            reflect_filter(root, list_id, on_action);
        }
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: ACTION_EXECUTE.to_string(),
                payload: value,
            });
        }
        // `ACTION_EXECUTE` の実行先がダイアログを開いたまま再描画すると
        // `hidden`/`aria-activedescendant` が再び失われ得る（codex-review
        // P1 是正、[`handle_keydown`] の `Execute` 分岐と同型）。ダイアログ
        // が閉じる構成では `reflect_filter` 内の再解決が fail-closed に
        // no-op となるだけで安全に収束する。
        if let Some(list_id) = list_id.as_deref() {
            reflect_filter(root, list_id, on_action);
        }
    }

    /// document 上の keydown: Cmd/Ctrl+K を判定し、`root` 配下に `dialog`
    /// パーツが存在するときのみ [`ACTION_TOGGLE`] を dispatch する。発火後
    /// は生きた DOM で `dialog` を再解決し、open なら絞り込み・選択状態を
    /// [`reflect_filter`] で再同期してから配下の `input` へ `focus()` する
    /// （失敗は無視、fail-closed）。
    fn handle_document_keydown(
        _document: &Document,
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        // `Closure::forget` で document へ登録したリスナーは `root`（マウント
        // 時点の要素）を無期限に保持し続ける。`root` がその後の再描画で
        // document から切り離されても本リスナー自体は生き続けるため、
        // `root.is_connected()` を確認しないと detached subtree 内の
        // `dialog`（`query_selector` はサブツリー内であれば detached でも
        // ヒットする）へ toggle を dispatch し続けてしまう（codex-review P1
        // 是正: 旧コンポーネントへのグローバルショートカット誤発火）。
        if !root.is_connected() {
            return;
        }
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
        // 再描画後の生きた DOM を再解決する。open なら絞り込み・選択状態を
        // 再同期してから `input` へ focus する（codex-review P1・Cursor
        // Bugbot 是正: dialog の再オープンをまたいで保持される `query` に
        // 対し、`view()` は `hidden`/`aria-activedescendant` を再現しない
        // ため、次の `input` イベントまで全項目が可視のまま取り残される。
        // `reflect_filter` は `list_id` から生きた DOM を再解決して
        // `input.value()`（＝保持された query）で絞り込みを再計算するため、
        // 再オープン直後から正しい絞り込み状態を復元できる）。
        if let Some(dialog) = root.query_selector(DIALOG_SELECTOR).ok().flatten() {
            if !dialog.has_attribute("hidden") {
                if let Some(list) = dialog.query_selector(LIST_SELECTOR).ok().flatten() {
                    let list_id = list.id();
                    if !list_id.is_empty() {
                        reflect_filter(root, &list_id, on_action);
                    }
                }
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
