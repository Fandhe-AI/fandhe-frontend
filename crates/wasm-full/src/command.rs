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
//! 候補列は同一インスタンス配下の非 `hidden` item（`highlight_next_index`
//! と `menu_loop_focus_from_attr` は [`crate::keynav`] をそのまま再利用
//! するが、disabled 判定は item 自身の属性しか見ない `keynav::wiring::
//! disabled_flags` ではなく、祖先の `data-disabled`/`disabled` も辿る
//! 本モジュール独自の `item_disabled_flags`（内部で `has_disabled_
//! ancestor` を使う）を使う。Enter 実行・クリックと同じ無効化契約に
//! 揃えるため、codex-review P1 是正）。
//!
//! cmdk は既定で非循環（`loop_focus` 既定 `false`、
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
//!   Shift も同様に no-op とする（[`crate::keynav::Modifiers`] は Ctrl/Alt/
//!   Meta の 3 フィールドのみを持つ公開型で拡張しないため、`handle_keydown`
//!   が `KeyboardEvent::shift_key()` を直接判定する。省略すると検索欄で
//!   Shift+Home/Shift+End/Shift+ArrowDown を押したときブラウザ既定の
//!   テキスト範囲選択を奪って候補選択に化けてしまう、codex-review P1
//!   是正）。
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
//! - `Closure::forget` はマウント時の定数回（root 6〔input の capture/
//!   bubble 各 1・keydown・click・mousedown（item のフォーカス維持用、
//!   codex-review P1 是正）・compositionend（IME 確定時の補完 dispatch、
//!   イシュー #2069 codex-review P1 是正）〕+ document 1）に限定する
//!   （A04 対策、無制限リークの構造的回避）。
//! - 未知キー・修飾キー付き（Shift 含む）・IME 変換中・`disabled`/
//!   `data-disabled`・`data-value` 欠落・未選択 Enter・hidden な選択・
//!   `dialog` 不在の Escape/Cmd+K はすべて no-op（fail-closed）。
//!   `Command::decode_action` が [`ACTION_EXECUTE`] を未知アクションとして
//!   無視する二重の安全網も働く。
//! - item 内に利用者が併設した独立インタラクティブ要素（`button`/
//!   `a[href]`/`input`/`select`/`textarea`）のクリックは、その要素自身の
//!   既定動作に委ね、祖先 item を解決した `select`/`command:execute` を
//!   dispatch しない（`handle_click`/`handle_mousedown` の 2 経路で
//!   `INDEPENDENT_INTERACTIVE_SELECTOR` 判定を共有し無効化契約を一致
//!   させる、Cursor Bugbot Medium 是正）。

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
        closest, collect_parts, modifiers_of, scroll_item_into_view_if_needed, set_dom_attribute,
    };
    use crate::keynav::{highlight_next_index, menu_loop_focus_from_attr};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Document, Element, Event, HtmlElement, HtmlInputElement, InputEvent, KeyboardEvent, Node,
    };

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
    /// `handle_mousedown` のフォーカス維持処理から除外する「独立した
    /// インタラクティブ要素」のセレクタ（codex-review P1 是正、イシュー
    /// #2069）。`command::dialog` は任意の children を受け取れるため、
    /// 検索対象切替用の `select` や別の `input` 等が併設され得るが、
    /// これらとその子孫（`Element::closest` は self-or-ancestor で一致
    /// する）へのクリックはブラウザ既定のフォーカス移動・選択操作を
    /// 妨げてはならない（[`handle_mousedown`] doc 参照）。`tabindex="-1"`
    /// は除外する: `command::dialog` パーツ自身がプログラム的フォーカス
    /// 専用に `tabindex="-1"` を持つ（headless-ui `command::dialog`）ため
    /// 含めると、dialog 背景への mousedown までフォーカス維持処理の対象
    /// 外になってしまう（既存の item クリック挙動は `role="option"` のみで
    /// tabindex を持たないため、本セレクタには一致せず影響なし）。
    const INDEPENDENT_INTERACTIVE_SELECTOR: &str = "a[href], button, input, select, textarea, \
        [contenteditable=\"true\"], [tabindex]:not([tabindex=\"-1\"])";
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

    /// `root` 配下の Command へ input/keydown/click/mousedown（計 4 回）、
    /// `document` へ keydown（1 回、Cmd/Ctrl+K）を配線する（マウント時 1 回
    /// 契約、`Closure::forget` は本関数呼び出しにつき定数 6 回に限定する
    /// （codex-review P1 再々是正で capture-phase の 1 回を追加、続けて
    /// item mousedown のフォーカス維持用に 1 回追加、モジュール冒頭 doc
    /// 「セキュリティ不変条件」節参照）。
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

        // capture-phase での dispatch 前フォーカス記録（codex-review P1
        // 再々是正）: `data-action-input` を持つ input は、同一 `root` へ
        // 本関数より前に登録される `crate::events::wire_events` の
        // bubble-phase "input" リスナーが、本モジュールの bubble-phase
        // [`handle_input`] より必ず先に発火する（`crate::lib::Runtime::
        // mount`/`hydrate` が `events::wire_events` を先に呼ぶ、モジュール
        // 冒頭 doc「`"input"` dispatch と `crate::events::wire_events` の
        // 二重 dispatch 回避」節）。その dispatch が構造フォールバック
        // 再描画を誘発すると、[`handle_input`] 冒頭で改めて
        // `capture_focus_state` してももう手遅れ（`target_element` は既に
        // detach 済み）になる（P1 是正の再発、data-action-input 経路限定の
        // 既知の穴）。DOM のイベント capturing phase はどの target の
        // bubbling phase リスナーよりも必ず先に完了するという契約を利用し、
        // ここで `wire_events` の dispatch より確実に前にフォーカス状態を
        // 記録しておく（`add_event_listener_with_callback_and_bool` の
        // `use_capture: true`）。
        let pre_input_focus: std::rc::Rc<std::cell::RefCell<Option<FocusState>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let capture_pre_input_focus = pre_input_focus.clone();
        let capture_focus_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>() else {
                return;
            };
            // Input パーツ以外（capture phase は root 配下の全 "input"
            // イベントを受け取る）は無視する（[`handle_input`] と同じ判定）。
            if !matches_part(target_element, INPUT_PART) {
                return;
            }
            *capture_pre_input_focus.borrow_mut() = Some(capture_focus_state(target_element));
        });
        root.add_event_listener_with_callback_and_bool(
            "input",
            capture_focus_closure.as_ref().unchecked_ref(),
            true,
        )?;
        capture_focus_closure.forget();

        // IME 確定（`compositionend`）で反映済みの値（イシュー #2069
        // codex-review P1 是正）。[`handle_compositionend`] が確定値を
        // 直接反映した際に記録し、[`handle_input`] がその直後にブラウザが
        // 追加で発火する非 composing な "input"（同一値）を二重 dispatch
        // しないためのガードに使う（[`handle_input`] doc参照）。
        let composed_input_value: std::rc::Rc<std::cell::RefCell<Option<String>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        let input_root = root.clone();
        let input_on_action = on_action.clone();
        let input_pre_focus = pre_input_focus.clone();
        let input_composed_guard = composed_input_value.clone();
        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_input(
                &input_root,
                &event,
                &input_on_action,
                &input_pre_focus,
                &input_composed_guard,
            );
        });
        root.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
        input_closure.forget();

        let compositionend_root = root.clone();
        let compositionend_on_action = on_action.clone();
        let compositionend_pre_focus = pre_input_focus.clone();
        let compositionend_guard = composed_input_value;
        let compositionend_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_compositionend(
                &compositionend_root,
                &event,
                &compositionend_on_action,
                &compositionend_pre_focus,
                &compositionend_guard,
            );
        });
        root.add_event_listener_with_callback(
            "compositionend",
            compositionend_closure.as_ref().unchecked_ref(),
        )?;
        compositionend_closure.forget();

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

        let mousedown_root = root.clone();
        let mousedown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_mousedown(&mousedown_root, &event);
        });
        root.add_event_listener_with_callback(
            "mousedown",
            mousedown_closure.as_ref().unchecked_ref(),
        )?;
        mousedown_closure.forget();

        // 配線完了時点での初期絞り込み反映（codex-review P1 是正、イシュー
        // #2069）: ここまでは入力/keydown 等のリスナー登録のみで、mount/
        // hydrate 時に検索クエリを保持したまま描画された Command（全候補が
        // 未絞り込みのまま DOM に出力されている）の List・選択状態は未同期
        // のままだった。`root` 配下の全 Command インスタンスについて、
        // 生きた input から [`resolve_list`] で List パーツを解決し、
        // [`reflect_filter`] を 1 回ずつ呼んで絞り込み・選択整合を配線直後
        // に同期する（`outer_focus_state: None` — 配線直後はまだ利用者
        // 操作によるフォーカス変化がないため、[`reflect_filter`] 内部で
        // 現在のフォーカス状態をそのまま採用させる）。ダイアログが初期
        // 非表示（`hidden`）の構成でも安全に呼べる（[`reflect_filter`] は
        // 生きた DOM の再解決のみに依存し、可視性を前提にしない）。
        for input in collect_parts(&root, INPUT_SELECTOR) {
            if let Some(list) = resolve_list(&root, &input) {
                let list_id = list.id();
                reflect_filter(&root, &list_id, &on_action, None);
            }
        }

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
    ///
    /// `Keep`/`Select` いずれの分岐でも、選択中 item を
    /// [`scroll_item_into_view_if_needed`]（`crate::keynav::wiring` から
    /// 再利用、境界は Command の [`LIST_SELECTOR`]）で最寄りのスクロール
    /// 可能な祖先内へ表示させる（codex-review P1 是正: 高さ制限された
    /// 候補リストで ArrowDown/End を繰り返しても選択項目が可視領域外へ
    /// 出たまま追随しなかった不具合、イシュー #2069）。既に可視領域内なら
    /// no-op であり、スクロール可能な祖先を持たない構成（`list` に
    /// `overflow-y`/`max-height` を設定しない既定 CSS）でも副作用がない。
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
                scroll_item_into_view_if_needed(target, LIST_SELECTOR);
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
                scroll_item_into_view_if_needed(target, LIST_SELECTOR);
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

    /// `input` の祖先に `dialog` パーツが無い、またはあっても `hidden`
    /// でないかどうか（Cursor Bugbot High 是正）: `ACTION_EXECUTE` で
    /// dialog が閉じる構成では [`reflect_filter`] 呼び出し前後で dialog
    /// 要素に `hidden` 属性が付くだけで DOM から取り除かれるわけではない
    /// ため、[`resolve_command_parts_by_list_id`] は閉じた後も引き続き
    /// 同じ `input` を解決できてしまう。この状態のまま
    /// [`restore_focus_state`] を呼ぶと、非表示の command dialog 内へ
    /// フォーカスを強制的に戻してしまう（実行後のフォーカスがユーザーへ
    /// 見えないパレットへ消える不具合）。dialog パーツを持たない
    /// Command（非ダイアログ構成）は常に `true` を返す。
    fn input_focus_target_is_visible(input: &Element) -> bool {
        match closest(input, DIALOG_SELECTOR) {
            Some(dialog) => !dialog.has_attribute("hidden"),
            None => true,
        }
    }

    /// dispatch の結果、フォーカスが `input` 以外の要素へ意図的に移されて
    /// いないかどうか（codex-review P1 是正、イシュー #2069）:
    /// `command:execute`（[`ACTION_EXECUTE`]）のアプリ側実行フックが、実行
    /// 先の別要素（例: エディタ）へ明示的に `focus()` している場合、
    /// [`restore_focus_state`] を無条件に呼ぶと `reflect_filter` がその
    /// フォーカスを `input` へ無条件に奪い返してしまい、アプリの実行フック
    /// が意図したフォーカス移動を妨げる（dialog を持たない Command で
    /// 顕著、Enter・item クリックの双方で発生し得る）。
    ///
    /// 構造フォールバック再描画による detach 由来のフォーカス喪失では、
    /// 削除された要素にフォーカスが残ることはなくブラウザが自動的に
    /// `document.body()` へフォーカスを戻す（`document.active_element()`
    /// はフォーカスされている要素が無いとき `<body>` を返す DOM の仕様）。
    /// このため「`active_element` が `None` または `<body>`」であれば
    /// 再描画由来の喪失（安全に `input` へ復元してよい）、それ以外の
    /// 接続済み要素であればアプリ側の意図的なフォーカス移動（尊重し復元
    /// しない）と区別できる。
    fn focus_available_for_restore(document: &Document) -> bool {
        match document.active_element() {
            None => true,
            Some(active) => match document.body() {
                Some(body) => active.is_same_node(Some(&body)),
                None => false,
            },
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
        outer_focus_state: Option<FocusState>,
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
        // フォーカス状態の記録（codex-review P1 再是正）: `outer_focus_state`
        // が渡された場合は、呼び出し側（[`handle_input`]/[`handle_keydown`]
        // の `MoveSelection`/`Execute` 分岐）が**自身の dispatch より前**に
        // 記録したスナップショットである。呼び出し側の dispatch が構造
        // フォールバック再描画を誘発していれば、この時点で再解決した
        // `input` は既にフォーカスを失っている（新規生成された要素は
        // `document.active_element()` と一致しない）ため、ここで内部的に
        // 改めて `capture_focus_state` すると `focused=false` を確定させて
        // しまい連続入力・矢印操作が途切れる（元の P1 指摘）。
        // `outer_focus_state.focused` が真なら、まずそれを再解決した
        // `input` へ即座に復元してから `focus_state` として採用すること
        // で、呼び出し側の dispatch・本関数自身の dispatch のどちらが
        // 再描画を誘発しても正しく収束する（`restore_focus_state` は
        // 既にフォーカス済みの要素に対しても副作用なく安全に呼べる）。
        // `input_focus_target_is_visible` は Cursor Bugbot High 是正:
        // `ACTION_EXECUTE` で dialog が閉じた（`hidden` が付いた）構成では
        // 復元先が非表示のままになるため、そのときは復元しない
        // （fail-closed、同関数 doc 参照）。`focus_available_for_restore`
        // は codex-review P1 是正（イシュー #2069）: 呼び出し側（`handle_
        // input`/`handle_keydown`）の dispatch が `command:execute` の実行
        // フックで別要素へ意図的にフォーカスを移していれば尊重し、`input`
        // を奪い返さない（同関数 doc 参照）。
        if let Some(state) = outer_focus_state.as_ref() {
            if state.focused
                && input_focus_target_is_visible(&input)
                && input
                    .owner_document()
                    .is_some_and(|doc| focus_available_for_restore(&doc))
            {
                restore_focus_state(&input, state);
            }
        }
        let focus_state = outer_focus_state.unwrap_or_else(|| capture_focus_state(&input));
        let items = collect_own_items(&list, &instance_root);
        let visible = visible_flags(&items, &query);
        let plan = {
            let disabled = item_disabled_flags(root, &items);
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
                let fresh_disabled = item_disabled_flags(root, &fresh_items);
                selection_sync_plan(&fresh_items, &fresh_visible, &fresh_disabled)
            };
            // dispatch が構造フォールバック再描画を誘発し `input` が
            // detach された場合、再解決した生きた `input` へフォーカス・
            // 選択範囲を復元する（codex-review P1 是正）。ただし復元先が
            // 非表示の command dialog 内なら復元しない（Cursor Bugbot High
            // 是正、`input_focus_target_is_visible` doc 参照）。フォーカス
            // が既に別の接続済み要素へ意図的に移されていれば復元しない
            // （codex-review P1 是正、イシュー #2069、
            // `focus_available_for_restore` doc 参照）。
            if input_focus_target_is_visible(&fresh_input)
                && fresh_input
                    .owner_document()
                    .is_some_and(|doc| focus_available_for_restore(&doc))
            {
                restore_focus_state(&fresh_input, &focus_state);
            }
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
    ///
    /// 新規選択項目を [`scroll_item_into_view_if_needed`]（境界は Command
    /// の [`LIST_SELECTOR`]）で可視領域へ追随させる（codex-review P1 是正:
    /// ArrowDown/ArrowUp/Home/End の矢印キー選択経路、イシュー #2069。
    /// [`write_selection_plan`] の doc コメント参照）。
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
        scroll_item_into_view_if_needed(target, LIST_SELECTOR);
    }

    /// `input` イベント: `INPUT_PART` 上でのみ反応する。`data-action-input`
    /// があれば dispatch は `crate::events::wire_events` に委ねて DOM 反映
    /// のみ行い、無ければ [`ACTION_INPUT`] を dispatch する（モジュール
    /// 冒頭 doc「二重 dispatch 回避」節参照）。
    ///
    /// IME 変換確定時の補完 dispatch は [`handle_compositionend`] が担う
    /// （イシュー #2069 codex-review P1 是正）。`composed_guard` はその
    /// 補完 dispatch との二重 dispatch 回避に使う 1 ショットガード
    /// （[`handle_compositionend`] doc参照）。
    fn handle_input(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
        pre_focus: &std::rc::Rc<std::cell::RefCell<Option<FocusState>>>,
        composed_guard: &std::rc::Rc<std::cell::RefCell<Option<String>>>,
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
        // IME 変換中は入力 dispatch・`reflect_filter` 双方を延期する
        // （codex-review P1 是正）。`data-action-input` を持たない通常の
        // Command では本ハンドラが [`ACTION_INPUT`] dispatch と
        // `reflect_filter`（絞り込みに応じた構造再描画を誘発し得る）を
        // 直接担うため、変換途中の "input" イベント（`InputEvent::
        // is_composing()` が真）で反映すると変換対象の input 要素ごと
        // 再描画で削除され、日本語などの IME 入力が中断される
        // （[`handle_keydown`]/[`handle_document_keydown`] の
        // `is_composing()`/`key_code() == 229` 判定と同型の安全網、
        // モジュール冒頭 doc「セキュリティ不変条件」節）。変換確定
        // （compositionend）時の確定値反映は [`handle_compositionend`] が
        // 別途担う（イシュー #2069 codex-review P1 是正: ブラウザが確定後に
        // 必ず追加の "input" を発火するとは限らないため、この no-op のみに
        // 依存しない）。
        if event
            .dyn_ref::<InputEvent>()
            .is_some_and(InputEvent::is_composing)
        {
            return;
        }
        // 二重 dispatch 回避（イシュー #2069 codex-review P1 是正）: 直前の
        // `compositionend` で既に確定値を反映済みの場合、ブラウザがその
        // 直後に追加で発火する非 composing な "input"（現在値が確定値と
        // 同一）はスキップする。値が異なれば通常どおり反映する。1 ショット
        // のみ有効（`take()` で消費、次の無関係な input まで誤ってスキップ
        // し続けない）。
        if let Some(dispatched_value) = composed_guard.borrow_mut().take() {
            let current_value = target_element
                .clone()
                .dyn_into::<HtmlInputElement>()
                .ok()
                .map(|el| el.value());
            if current_value.as_deref() == Some(dispatched_value.as_str()) {
                return;
            }
        }
        dispatch_input_effect(root, target_element, on_action, pre_focus);
    }

    /// `compositionend` イベント: `INPUT_PART` 上でのみ反応する（イシュー
    /// #2069 codex-review P1 是正）。
    ///
    /// [`handle_input`] は変換中（`isComposing`）の "input" を延期し、
    /// 確定後に発火する追加の "input" で反映される前提を置いていたが、
    /// この前提は実ブラウザで常に成立するとは限らない（Chrome の一部確定
    /// 操作で確定後の "input" が発火されない既知挙動）。放置すると確定
    /// した最終値（絞り込みクエリ）が反映されず、日本語などの IME 入力
    /// 確定でアプリ状態・検索結果が更新されない。`compositionend` は
    /// 仕様上必ず発火するため、ここで確定値を直接反映することで取りこぼし
    /// を防ぐ。
    ///
    /// 反映後、使用した値を `composed_guard` へ記録する
    /// （[`handle_input`] 側がブラウザの追加 "input" による二重 dispatch を
    /// 避けるために消費する）。
    ///
    /// このガードは `set_timeout` の 0ms 遅延で自ら解除するタイマーを
    /// 併せて仕掛け、直後の 1 マクロタスクに限定する（イシュー #2069
    /// codex-review P1 再指摘 是正）。`compositionend` 直後に対応する
    /// 追加 "input" が発火しない場合、`take()` されずに残った記録値が
    /// 次に非 composing な "input" が発火するまで無期限に残留してしまう。
    /// この間に（例えば `command:execute` の実行フックで検索クエリが
    /// リセットされてパレットが再表示される等）別の状態変化が起き、その後
    /// 無関係な入力（同じ文字列の独立した貼り付け等）が行われると、値の
    /// 一致だけで誤って二重 dispatch 抑止が発動し、入力値とアプリ状態が
    /// 不整合になる（実ブラウザの `compositionend`→追加 "input" は発火
    /// する場合でも同一マクロタスク内で起きるため、0ms タイマーで
    /// 「同じ確定操作に対する直後の重複」だけを救い、それ以降のユーザー
    /// 操作〔必ず新しいタスクで発生する〕には影響しない）。
    fn handle_compositionend(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
        pre_focus: &std::rc::Rc<std::cell::RefCell<Option<FocusState>>>,
        composed_guard: &std::rc::Rc<std::cell::RefCell<Option<String>>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !matches_part(target_element, INPUT_PART) {
            return;
        }
        let value = target_element
            .clone()
            .dyn_into::<HtmlInputElement>()
            .ok()
            .map(|el| el.value());
        dispatch_input_effect(root, target_element, on_action, pre_focus);
        *composed_guard.borrow_mut() = value.clone();
        if let Some(value) = value {
            schedule_composed_guard_reset(composed_guard, value);
        }
    }

    /// [`handle_compositionend`] が仕掛ける二重 dispatch ガードの自己解除
    /// タイマー本体。`expected` と現在のガード値が一致する場合に限り
    /// `take()` する（このタイマーが仕掛けられた後、別の `compositionend`
    /// が新しい値でガードを上書きしていた場合に、古いタイマーが新しい
    /// ガードを誤って消し飛ばさないための一致確認。doc は
    /// [`handle_compositionend`] 参照）。
    fn schedule_composed_guard_reset(
        composed_guard: &std::rc::Rc<std::cell::RefCell<Option<String>>>,
        expected: String,
    ) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let guard_for_timer = composed_guard.clone();
        let reset = Closure::once_into_js(move || {
            let mut guard = guard_for_timer.borrow_mut();
            if guard.as_deref() == Some(expected.as_str()) {
                guard.take();
            }
        });
        let _ =
            window.set_timeout_with_callback_and_timeout_and_arguments_0(reset.unchecked_ref(), 0);
    }

    /// [`handle_input`]（変換中でない "input"）と [`handle_compositionend`]
    /// （IME 確定）の双方から共有する反映本体（イシュー #2069 codex-review
    /// P1 是正）。disabled 判定・List パーツ解決・`ACTION_INPUT` dispatch・
    /// [`reflect_filter`] 呼び出しをまとめる。
    fn dispatch_input_effect(
        root: &Element,
        target_element: &Element,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
        pre_focus: &std::rc::Rc<std::cell::RefCell<Option<FocusState>>>,
    ) {
        // disabled/`data-disabled` な祖先（Command root 自体を含む）配下
        // では入力による状態更新も no-op とする（codex-review P1 是正:
        // 従来 [`handle_keydown`] のみが `has_disabled_ancestor` を確認して
        // おり、`data-action-input` を持たない input の絞り込み dispatch は
        // 無効化状態を無視して発生していた）。`root` 自身の属性は
        // `has_disabled_ancestor` の走査に頼らず直接確認する（codex-review
        // P1 再是正）: `data-action-input` を持つ input では、本チェックの
        // 直前に既に `crate::events::wire_events` の "input" dispatch が
        // 走り、`root` の子だけを構造フォールバックで差し替えている場合が
        // ある（`root` 自体は document から切り離されない）。その場合
        // `target_element` は旧 DOM に属したまま `root` から detach 済み
        // であり、`parent_element()` を辿っても新しい `root` の部分木には
        // 到達しないため、`has_disabled_ancestor(root, target_element)` は
        // 祖先列に `root` が現れず `false` を返してしまい、`root` 自身の
        // `data-disabled` を見逃す。`root` 自身の属性はこの detach の影響を
        // 受けないため別途直接確認する。
        if root.has_attribute("data-disabled")
            || root.has_attribute("disabled")
            || has_disabled_ancestor(root, target_element)
        {
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

        // dispatch（`ACTION_INPUT` の `on_action` 呼び出し）より前の
        // フォーカス状態（codex-review P1 再々是正）: `pre_focus` は
        // [`wire_command_events`] が capture phase で記録したスナップショット
        // を優先的に使う。`data-action-input` を持つ input では、同一
        // `root` へ先に登録された `crate::events::wire_events` の
        // bubble-phase dispatch が本ハンドラより先に走り、構造フォール
        // バック再描画を誘発し得るため、この時点で改めて
        // `capture_focus_state(target_element)` すると `target_element` は
        // 既に detach 済みで `focused=false` が確定してしまう
        // （P1 是正の再発）。capture phase での記録は `wire_events` の
        // dispatch より確実に前に完了しているため正しい。`pre_focus` が
        // 空（capture phase のリスナー登録に失敗した等）の場合のみ、
        // フォールバックとして現在の `target_element` から直接記録する。
        let focus_state = pre_focus
            .borrow_mut()
            .take()
            .unwrap_or_else(|| capture_focus_state(target_element));

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
        // そのまま使い回さないため正しく動作する。上で記録した
        // `focus_state` を渡すことで、この直前の dispatch による再描画で
        // 失われたフォーカスも正しく復元できる（codex-review P1 再是正）。
        reflect_filter(root, &list_id, on_action, Some(focus_state));
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
    /// （`number_input.rs::has_disabled_ancestor` と類似だが、`root.contains`
    /// による早期打ち切りは持たない）。
    ///
    /// `start` は再描画による構造フォールバックで既に `root` から detach
    /// 済みの要素であり得る（[`handle_input`] の `data-action-input` 経路の
    /// 既知の事情、モジュール冒頭 doc「`"input"` dispatch と
    /// `crate::events::wire_events` の二重 dispatch 回避」節参照）。detach
    /// 済み要素は `root.contains(element)` が常に偽になるため、これを走査
    /// 打ち切り条件に使うと `start` 自身の属性しか確認できず祖先の
    /// disabled を見逃す（Cursor Bugbot 是正）。`element == *root` に到達
    /// したとき、または `parent_element()` が尽きたときにのみ走査を止める
    /// （detach 済みの stale な部分木を辿っても `root` には到達しないため
    /// 安全に完走する）。
    fn has_disabled_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("disabled") {
                return true;
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// 各 item の「実効 disabled」（[`has_disabled_ancestor`] による祖先
    /// 込みの判定）を列挙する（codex-review P1 是正: `keynav::wiring::
    /// disabled_flags` は item 自身の属性しか見ないため、group（`role=
    /// "group"`）に `data-disabled` を付けた構成で、矢印キー選択・検索後
    /// 自動選択の候補判定（[`selection_sync_plan`]）が無効化された配下の
    /// item を選択候補にしてしまっていた。Enter 実行・クリックは
    /// `has_disabled_ancestor` で祖先無効化を確認するため、候補判定でも
    /// 同じ関数を使い操作経路間の無効化契約を一致させる）。
    fn item_disabled_flags(root: &Element, items: &[Element]) -> Vec<bool> {
        items
            .iter()
            .map(|it| has_disabled_ancestor(root, it))
            .collect()
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

        // Shift 押下は no-op（codex-review P1 是正、イシュー #2069）。
        // `Modifiers`（[`crate::keynav::Modifiers`]）は Ctrl/Alt/Meta の
        // 3 フィールドのみを持ち Shift を含まない公開型であり、破壊的
        // 変更を避けるため本モジュールでは拡張せず `KeyboardEvent` から
        // 直接判定する。これを省略すると
        // 検索欄で Shift+Home/Shift+End/Shift+ArrowDown を押したとき
        // `command_key_action` が `MoveSelection` を返し
        // `prevent_default()` されてしまい、ブラウザ既定のテキスト範囲
        // 選択（Shift によるキャレット選択拡張）を奪ってしまう
        // （「プレーン HTML/JS 尊重」に反する、修飾キー付きは常に no-op
        // という本関数冒頭 doc の契約にも反する）。
        if keyboard_event.shift_key() {
            return;
        }

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
                let disabled = item_disabled_flags(root, &visible_items);
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
                // dispatch（`ACTION_SELECT` の `on_action` 呼び出し）より
                // 前にフォーカス状態を記録する（codex-review P1 再是正、
                // [`handle_input`] と同じ理由）。`target_element` は本
                // keydown イベントの `event.target()` であり、かつ関数冒頭
                // で `matches_part(target_element, INPUT_PART)` を確認済み
                // なので Input パーツそのものである。
                let focus_state = capture_focus_state(target_element);
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
                // に現在の絞り込み・選択状態の再反映を委ねる。上で記録した
                // `focus_state` を渡し、この直前の dispatch による再描画で
                // 失われたフォーカスも正しく復元する（codex-review P1
                // 再是正）。
                reflect_filter(root, &list_id, on_action, Some(focus_state));
            }
            super::CommandKeyAction::Execute => {
                // 押しっぱなしによるキーリピート（`repeat: true`）は無視
                // する（Cursor Bugbot 是正）。無視しないと Enter を長押し
                // しただけで [`ACTION_EXECUTE`] が繰り返し dispatch され、
                // 冪等でない実行（送信・削除等）が意図せず多重発火する。
                if keyboard_event.repeat() {
                    return;
                }
                let Some(list) = resolve_list(root, target_element) else {
                    return;
                };
                let items = collect_own_items(&list, &instance_root);
                // `item` 自身の `data-disabled` だけでなく、祖先（Command
                // root 自体を含む）の disabled/`data-disabled` も確認する
                // （codex-review P1 是正: `handle_click`/`handle_keydown`
                // の `MoveSelection` 分岐は矢印操作・クリックで
                // `has_disabled_ancestor` を経由した無効化判定を行うのに、
                // Enter 実行だけが item 自身の `data-disabled` しか見ておらず
                // 操作経路間で無効化契約が不一致だった。`has_disabled_
                // ancestor` は item 自身の `data-disabled` も先頭で確認する
                // ため、単純な `it.has_attribute("data-disabled")` の上位
                // 互換になる）。
                let Some(selected) = items.iter().find(|it| {
                    it.has_attribute("data-selected")
                        && !it.has_attribute("hidden")
                        && !has_disabled_ancestor(root, it)
                }) else {
                    return;
                };
                let Some(value) = selected.get_attribute("data-value") else {
                    return;
                };
                // 再描画をまたいで List パーツを再解決するための識別子
                // （[`handle_input`]/`MoveSelection` 分岐と同じ契約）。
                let list_id = list.id();
                // dispatch（`ACTION_EXECUTE` の `on_action` 呼び出し）より
                // 前にフォーカス状態を記録する（codex-review P1 再是正、
                // `MoveSelection` 分岐と同じ理由）。
                let focus_state = capture_focus_state(target_element);
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
                // なるだけで安全に収束する。上で記録した `focus_state` を
                // 渡し、この直前の dispatch による再描画で失われたフォー
                // カスも正しく復元する（codex-review P1 再是正。ダイアログ
                // が閉じる構成では `outer_focus_state.focused` が真でも
                // `resolve_command_parts_by_list_id` が `None` を返し
                // no-op となるだけで安全）。
                reflect_filter(root, &list_id, on_action, Some(focus_state));
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

    /// mousedown: Command インスタンス配下でのブラウザ既定動作（フォーカス
    /// 移動、結果として `input` が `blur` する）を抑止し、`input` の
    /// フォーカスを維持する（codex-review P1 是正、item 以外への拡張は
    /// Cursor Bugbot Medium 是正・イシュー #2069）。実ブラウザでは `click`
    /// イベントより前に `mousedown` が発火し、その既定動作でフォーカス
    /// 可能要素以外をクリックすると現在のフォーカス（`input`）が失われる。
    /// `handle_click` 側で `capture_focus_state` しても、その時点で既に
    /// `focused = false` が確定してしまっており、`ACTION_EXECUTE` 実行後
    /// （パレットを開いたままにする構成）にフォーカスが復元されず、続く
    /// 検索入力・矢印・Enter が処理できなくなる。`ACTION_SELECT`/
    /// `ACTION_EXECUTE` が意図的にフォーカスを移動させるまでは `input` の
    /// フォーカスを保つため、`input` パーツ自身を除く Command インスタンス
    /// 配下（`dialog`/`list`/`group`/`separator`/`empty`/`item` いずれも
    /// 含む）の mousedown は `prevent_default()` で blur 自体を起こさせ
    /// ない。
    ///
    /// item 上のみに限定していた従来実装は、`dialog`（`tabindex="-1"` で
    /// フォーカス可能）・`list`/`group`/`separator`/`empty` 上の mousedown
    /// で `input` から blur し、以降 `handle_keydown`/`handle_input`
    /// （いずれも `event.target()` が `INPUT_PART` であることを要求する）
    /// が矢印キー・Enter・入力・Escape を no-op として無視してしまう
    /// 不具合があった（`input` を再クリックするまで操作不能になる。
    /// Escape が処理されないことで `prevent_default()`/`stop_propagation()`
    /// も呼ばれず、親オーバーレイの Escape ハンドラへ伝播して意図せず
    /// 閉じてしまう副作用も含む、Cursor Bugbot Medium 指摘）。
    ///
    /// `input` パーツ自身の mousedown はブラウザ既定動作（フォーカス・
    /// キャレット位置決定・テキスト選択）を妨げない。disabled な item は
    /// 従来どおり除外する（クリックしても実行されないため blur してよい、
    /// 既存の `handle_click` の disabled 判定と同型）。
    ///
    /// `command::dialog` は任意の children を受け取れるため、検索対象切替
    /// 用の `select` や別の `input` 等、Command のパーツではない独立した
    /// インタラクティブ要素が併設され得る（codex-review P1 是正、イシュー
    /// #2069）。これらとその子孫への mousedown まで `prevent_default()`
    /// してしまうと、AGENTS.md の「HTML/JS/CSS のプレーン尊重」に反し、
    /// そのクリックによるフォーカス移動・選択操作を抑止してしまう。
    /// `INDEPENDENT_INTERACTIVE_SELECTOR` に一致する要素（またはその子孫）
    /// はフォーカス維持処理の対象から除外し、item や非インタラクティブな
    /// 背景（`dialog`/`list`/`group`/`separator`/`empty` の素の領域）に
    /// 限定して適用する。
    ///
    /// `closest()` は self-or-ancestor を祖先方向へ無制限に探索するため
    /// （native `Element.closest` の仕様）、`root` の外側（Command を
    /// `command::dialog` ではなく Tabs content や ScrollArea viewport 等の
    /// 内側に合成配置した場合、それら親パネルが固定で持つ
    /// `tabindex="0"`）まで一致してしまい、通常の item クリックまで無効化
    /// されてしまう不具合があった（codex-review P1 再是正、イシュー
    /// #2069）。一致した要素が `root` 配下（`root` 自身を含む）に実在する
    /// ことを `root.contains` で確認し、`root` の外側の祖先への一致は
    /// 無視する（item 自身が独立コントロールを兼ねることはない前提で、
    /// item 内部の独立コントロール・`command::dialog` 直下の独立コント
    /// ロールの両方を維持しつつ、外側パネルへの意図しない波及だけを断つ）。
    fn handle_mousedown(root: &Element, event: &Event) {
        let Some(target) = event.target() else {
            return;
        };
        // `event.target()` は `Element` とは限らない（Cursor Bugbot Medium
        // 是正・イシュー #2069）。`fandhe_frontend_core::text` で描画される
        // グループ見出し・空メッセージ等は素のテキストノードであり、直接
        // クリックした場合 `target` はテキストノードになる。従来はここで
        // `Element` へのキャストのみを試みて失敗時に即 return していたため
        // `prevent_default()` が呼ばれず、`input` が blur し、以降の矢印
        // キー・入力・Escape が `input` を再クリックするまで無反応になって
        // いた。テキストノード等 `Element` でない場合は親要素
        // （`Node::parent_element`）へフォールバックし、それでも要素が
        // 得られない場合のみ何もしない。
        let target_element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };
        let target_element = &target_element;
        if !root.contains(Some(target_element)) {
            return;
        }
        if matches_part(target_element, INPUT_PART) {
            return;
        }
        // 改ざんされた `data-scope`/`data-part` で root 外の要素を操作
        // させない fail-closed（モジュール冒頭 doc「セキュリティ不変条件」
        // 節と同型）: `target_element` が実在の Command インスタンス
        // （`ROOT_SELECTOR` 祖先）の配下であることを確認する。
        let Some(instance_root) = resolve_instance_root(root, target_element) else {
            return;
        };
        // 独立したインタラクティブ要素（またはその子孫）はブラウザ既定
        // 動作を妨げない（codex-review P1 是正、イシュー #2069。
        // `INDEPENDENT_INTERACTIVE_SELECTOR` doc 参照）。一致した要素が
        // **この Command インスタンス（`instance_root`）配下**に実在する
        // 場合のみ対象とする。`root`（配線登録時の Runtime root）配下かで
        // 判定すると、Command を Tabs content や ScrollArea viewport の
        // 子孫として配置した構成で、それらパネルが固定で持つ
        // `tabindex="0"` に `closest` が一致し、かつパネル自体は
        // Command インスタンスの外側の祖先でありながら広域の `root` には
        // 含まれてしまうため、通常の item クリックまで誤って無効化されて
        // いた（codex-review P1 再是正・Cursor Bugbot High 是正、イシュー
        // #2069）。`instance_root` は Command 自身の root であり、その外側
        // の祖先パネルは `instance_root.contains` で確実に除外される一方、
        // item 内部の独立コントロールは `instance_root` の子孫のため
        // 引き続き正しく対象になる。
        if closest(target_element, INDEPENDENT_INTERACTIVE_SELECTOR)
            .is_some_and(|independent| instance_root.contains(Some(&independent)))
        {
            return;
        }
        if let Some(item) = closest(target_element, ITEM_SELECTOR) {
            if root.contains(Some(&item)) && has_disabled_ancestor(root, &item) {
                return;
            }
        }
        event.prevent_default();
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
        // `event.target()` は `Element` とは限らない（Cursor Bugbot High
        // 是正・イシュー #2069）。`fandhe_frontend_core::text` で描画される
        // item ラベルは素のテキストノードであり、そのラベル文字列を直接
        // クリックした場合 `target` はテキストノードになる。従来はここで
        // `Element` へのキャストのみを試みて失敗時に即 return していたため
        // `closest(ITEM_SELECTOR)` に到達できず、item のラベル部分への
        // クリックが `select`/`command:execute` を一切 dispatch しなかった
        // （[`handle_mousedown`] は同じ状況で `Node::parent_element` へ
        // フォールバックしており、本ハンドラのみ取りこぼしていた不整合）。
        // テキストノード等 `Element` でない場合は親要素へフォールバックし、
        // それでも要素が得られない場合のみ何もしない
        // （[`handle_mousedown`] と同型のフォールバック）。
        let target_element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };
        if !root.contains(Some(&target_element)) {
            return;
        }
        // 改ざんされた `data-scope`/`data-part` で root 外の要素を操作
        // させない fail-closed（[`handle_mousedown`] と同型）に加え、
        // 直後の独立コントロール除外判定の探索範囲を確定する（下記）ため、
        // 先に Command インスタンスの root を解決しておく。
        let Some(target_instance_root) = resolve_instance_root(root, &target_element) else {
            return;
        };
        // item 内に配置された独立インタラクティブ要素（`button`/
        // `a[href]`/`input`/`select`/`textarea`、`INDEPENDENT_INTERACTIVE_
        // SELECTOR` doc 参照）のクリックは、その要素自身の既定動作
        // （フォーム送信・リンク遷移・チェック切替等）に委ね、祖先の
        // item を解決した `select`/`command:execute` の二重発火をさせない
        // （Cursor Bugbot Medium 是正、イシュー #2069）。[`handle_mousedown`]
        // は既にこの判定を採用しており、click と mousedown の 2 経路で
        // 無効化契約を一致させる（[`handle_keydown`]/[`handle_click`] 間で
        // `has_disabled_ancestor` を共有するのと同じ設計判断）。従来は
        // ここを通過してしまい、item 内の `button` をクリックすると
        // `handle_mousedown` は既定動作を尊重する一方 `handle_click` が
        // 祖先 item まで遡って `select` → `command:execute` を dispatch し
        // `stop_propagation()` まで行っていた。
        //
        // `closest()` は祖先方向へ無制限に探索するため、一致した要素が
        // **この Command インスタンス（`target_instance_root`）配下**に
        // 実在する場合のみ対象とする。`root`（配線登録時の Runtime root）
        // 配下かで判定すると、Command を Tabs content や ScrollArea
        // viewport の子孫として配置した構成で、それらパネルが固定で持つ
        // `tabindex="0"` に `closest` が一致し、かつパネル自体は Command
        // インスタンスの外側の祖先でありながら広域の `root` には含まれて
        // しまうため、通常の item クリックまで誤って無効化されていた
        // （codex-review P1 再是正・Cursor Bugbot High 是正、イシュー
        // #2069。[`handle_mousedown`] doc「祖先探索範囲の限定」節と同型）。
        if closest(&target_element, INDEPENDENT_INTERACTIVE_SELECTOR)
            .is_some_and(|independent| target_instance_root.contains(Some(&independent)))
        {
            return;
        }
        let Some(item) = closest(&target_element, ITEM_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&item)) {
            return;
        }
        // `item` 自身の `data-disabled` だけでなく、祖先（Command root
        // 自体を含む）の disabled/`data-disabled` も確認する（codex-review
        // P1 是正: [`handle_keydown`] は `has_disabled_ancestor` を使う一方
        // 本ハンドラは item 自身しか見ておらず、root へ `data-disabled` を
        // 付けても item クリックによる `select`/`command:execute` が
        // 引き続き dispatch されていた）。`has_disabled_ancestor` は
        // `item` 自身の `data-disabled` も先頭で確認するため、単純な
        // `item.has_attribute("data-disabled")` の上位互換になる。
        if has_disabled_ancestor(root, &item) {
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

        // dispatch（`ACTION_SELECT` の `on_action` 呼び出し）より前に
        // フォーカス状態を記録する（codex-review P1 再是正、
        // [`handle_keydown`] の `MoveSelection` 分岐と同じ理由）。
        let select_focus_state = capture_focus_state(&input);
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
        // 安全に収束する。上で記録した `select_focus_state` を渡し、この
        // 直前の dispatch による再描画で失われたフォーカスも正しく復元
        // する（codex-review P1 再是正）。
        if let Some(list_id) = list_id.as_deref() {
            reflect_filter(root, list_id, on_action, Some(select_focus_state));
        }
        // `ACTION_EXECUTE` の dispatch 直前にもフォーカス状態を再取得する
        // （codex-review P1 再是正）。直前の `reflect_filter` 呼び出しが
        // 構造フォールバック再描画を誘発している可能性があるため、
        // `list_id` から生きた `input` を改めて解決してから記録する
        // （見つからない・List パーツが detach 済み等はダイアログが閉じた
        // ことを意味し得るため fail-closed に `None` へ倒す）。
        let execute_focus_state = list_id
            .as_deref()
            .and_then(|id| resolve_command_parts_by_list_id(root, id))
            .map(|(_, _, live_input)| capture_focus_state(&live_input));
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
        // no-op となるだけで安全に収束する。上で記録した
        // `execute_focus_state` を渡し、この直前の dispatch による再描画で
        // 失われたフォーカスも正しく復元する（codex-review P1 再是正）。
        if let Some(list_id) = list_id.as_deref() {
            reflect_filter(root, list_id, on_action, execute_focus_state);
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
        // Shift 併用（例: Ctrl+Shift+K）は対象外とする（Cursor Bugbot
        // 是正）。[`Modifiers`]（`crate::keynav`）は Shift を追跡しない
        // 共有型（同型 doc「Shift は許容する」節、他コンポーネントの矢印
        // ナビゲーション等では無害だが本ショートカットには波及させない）
        // ため、`is_toggle_shortcut` へは渡さず `KeyboardEvent` から直接
        // 判定する。Shift 付きはブラウザ/OS 側のショートカット
        // （例: ページ内検索の一部実装）と衝突しうる組み合わせであり、
        // `Ctrl+Alt+K`/`Ctrl+Meta+K` を対象外とする既存方針と同じ判断軸
        // で誤発火を避ける。
        if keyboard_event.shift_key() {
            return;
        }
        if !is_toggle_shortcut(&keyboard_event.key(), modifiers) {
            return;
        }
        let Some(dialog) = root.query_selector(DIALOG_SELECTOR).ok().flatten() else {
            return;
        };
        // Command インスタンス（`dialog` の最も近い `ROOT_SELECTOR` 祖先）を
        // 解決し、`has_disabled_ancestor` で無効化契約を確認する
        // （codex-review P1 是正: 従来は dialog の存在だけで toggle を
        // dispatch していたため、`handle_keydown`/`handle_input`/
        // `handle_click` が採用する `data-disabled` 無効化契約と不整合
        // だった。`root` へ `data-disabled` を付けても Cmd/Ctrl+K で開閉・
        // フォーカス移動できてしまう不具合の是正）。`resolve_instance_root`
        // は越境防止のため `wired_root`（`root`）に含まれることも確認する
        // fail-closed 実装であり、解決できない（改ざん・越境等）場合も
        // 他ハンドラ（`handle_keydown`/`handle_click`）と同型に no-op へ
        // 倒す。
        let Some(instance_root) = resolve_instance_root(root, &dialog) else {
            return;
        };
        if has_disabled_ancestor(root, &instance_root) {
            return;
        }
        // ショートカット一致が確定した時点で `prevent_default()` を呼ぶ
        // （Cursor Bugbot 是正）。従来は下記キーリピート guard が
        // `prevent_default()` より前に return していたため、Cmd/Ctrl+K を
        // 押しっぱなしにすると 2 回目以降の keydown で `prevent_default()`
        // されず、ブラウザ既定のショートカット（例: Chrome の検索/
        // アドレスバー）が発火してフォーカスを奪い得た。[`handle_keydown`]
        // の Enter（[`super::CommandKeyAction::Execute`]）分岐が
        // `prevent_default()` を repeat チェックより前に呼んでいるのと
        // 同型に揃える。
        keyboard_event.prevent_default();
        // 押しっぱなしによるキーリピート（`repeat: true`）は dispatch を
        // 無視する（Cursor Bugbot 是正）。無視しないと Cmd/Ctrl+K を長押し
        // しただけで toggle の dispatch が繰り返し発火し、dialog が開閉を
        // 反復する（チラつき）。`prevent_default()` は上で既に呼び済みの
        // ため、ブラウザ既定のショートカット抑止はリピート中も継続する。
        if keyboard_event.repeat() {
            return;
        }
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
        let list_id = root
            .query_selector(DIALOG_SELECTOR)
            .ok()
            .flatten()
            .filter(|dialog| !dialog.has_attribute("hidden"))
            .and_then(|dialog| dialog.query_selector(LIST_SELECTOR).ok().flatten())
            .map(|list| list.id())
            .filter(|list_id| !list_id.is_empty());
        if let Some(list_id) = list_id.as_deref() {
            reflect_filter(root, list_id, on_action, None);
        }
        // `reflect_filter` 自身の内部 dispatch（絞り込み後の選択整合を
        // 取るための `ACTION_SELECT`/`ACTION_DESELECT`）が構造フォール
        // バック再描画を誘発した場合、上の `reflect_filter` 呼び出しより
        // 前に解決した `dialog`/`input` 参照はもう detach 済みで、それへ
        // `focus()` しても表示中の入力欄には作用しない（codex-review P1
        // 是正: Ctrl/Cmd+K で開いた直後に検索入力を開始できない）。
        // 生きた `dialog` を改めて再解決し、開いていることを確認してから
        // その配下の `input` へ focus する。
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
