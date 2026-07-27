//! `fandhe_frontend_wasm_full::keynav`（Tabs/Accordion/Menu/Select/
//! RadioGroup/Menubar/Combobox/Listbox/NavigationMenu/ToggleGroup の
//! キーボード操作・イシュー #582・#583・#641（typeahead）・#1073
//! （Menubar）・#1071（Combobox）・#1070（Listbox）・#1075
//! （NavigationMenu/ToggleGroup）、親 #581）の native テスト。
//!
//! `keynav` モジュールの純粋層（[`tabs_next_index`]/[`accordion_next_index`]/
//! [`highlight_next_index`]/[`radio_next_index`]/[`listbox_next_index`]/
//! [`is_typeahead_key`]/[`typeahead_push`]/[`typeahead_next_index`]）は
//! web-sys に依存しないため、`wasm32` ターゲット・実 DOM を介さず native の
//! `cargo test --workspace` から公開 API 経由で直接検証できる
//! （`wasm-full/tests/nav_native.rs` と同じ 2 層構成方針）。詳細な網羅ケース
//! （orientation 別 Arrow・disabled スキップ・loopFocus 有無・typeahead の
//! バッファ/循環等）はモジュール内単体テスト（`crates/wasm-full/src/keynav.rs`）
//! に既に持つため、本ファイルは「公開 API 経由で壊れていないか」の統合確認に
//! 絞る。配線層（`wire_keynav`、`#[cfg(target_arch = "wasm32")]`）の検証は
//! `wasm-full/tests/keynav_browser.rs`（実ブラウザ）が担う。

use fandhe_frontend_wasm_full::keynav::{
    accordion_next_index, calendar_next_index, combobox_key_action, highlight_next_index,
    is_typeahead_key, listbox_next_index, loop_focus_from_attr, menu_loop_focus_from_attr,
    navigation_menu_link_next_index, navigation_menu_trigger_key_action, radio_next_index,
    splitter_key_action, submenu_nav, tabs_next_index, toggle_group_next_index,
    typeahead_next_index, typeahead_push, ComboboxKeyAction, Modifiers, NavigationMenuKeyAction,
    Orientation, SplitterKeyAction, SubmenuNav, TYPEAHEAD_TIMEOUT_MS,
};

/// 検証 1: Tabs horizontal の ArrowRight/ArrowLeft がフォーカスを移動する。
#[test]
fn tabs_horizontal_arrow_keys_move_focus() {
    let disabled = [false, false, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        tabs_next_index(
            1,
            "ArrowLeft",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
}

/// 検証 2: Tabs vertical では ArrowDown/ArrowUp のみが動き、horizontal 方向の
/// キーは no-op（`data-orientation` による分岐、モジュール doc §Tabs 参照）。
#[test]
fn tabs_vertical_orientation_only_responds_to_vertical_arrows() {
    let disabled = [false, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowDown",
            Orientation::Vertical,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Vertical,
            true,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 3: Home/End は disabled をスキップして先頭/末尾の非 disabled trigger
/// へ移動する。
#[test]
fn home_end_skip_disabled_items() {
    let disabled = [true, false, false, true];
    assert_eq!(
        tabs_next_index(
            1,
            "Home",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        tabs_next_index(
            1,
            "End",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(2)
    );
}

/// 検証 4: `data-loop-focus="false"` 相当の入力では端で no-op、それ以外
/// （欠落・`"true"`・未知値）は循環する（[`loop_focus_from_attr`] の契約）。
#[test]
fn loop_focus_flag_controls_wraparound_at_boundaries() {
    let disabled = [false, false, false];
    assert!(loop_focus_from_attr(None));
    assert!(!loop_focus_from_attr(Some("false")));

    assert_eq!(
        tabs_next_index(
            2,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
    assert_eq!(
        tabs_next_index(
            2,
            "ArrowRight",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 5: 修飾キー（Ctrl/Alt/Meta）付きは既知キーでも no-op
/// （ブラウザ標準ショートカットとの衝突回避、モジュール doc 参照）。
#[test]
fn modifier_keys_disable_navigation_even_for_known_keys() {
    let disabled = [false, false];
    let modifiers = Modifiers {
        ctrl: true,
        alt: false,
        meta: false,
    };
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            modifiers,
            &disabled
        ),
        None
    );
}

/// 検証 6: Accordion は ArrowDown/ArrowUp/Home/End のみを扱い、
/// **循環しない**（モジュール doc §Accordion の判断を固定する回帰テスト）。
#[test]
fn accordion_moves_focus_without_looping_and_skips_disabled() {
    let disabled = [true, false, false, true];
    assert_eq!(
        accordion_next_index(1, "ArrowDown", Modifiers::default(), &disabled),
        Some(2)
    );
    assert_eq!(
        accordion_next_index(2, "ArrowDown", Modifiers::default(), &disabled),
        None
    );
    assert_eq!(
        accordion_next_index(2, "Home", Modifiers::default(), &disabled),
        Some(1)
    );
    assert_eq!(
        accordion_next_index(1, "End", Modifiers::default(), &disabled),
        Some(2)
    );
}

/// 検証 7: 空・全 disabled・範囲外インデックスは panic せず `None`
/// （fail-closed、`.claude/rules/coding-rust.md` の panic 回避方針）。
#[test]
fn empty_all_disabled_and_out_of_range_are_noop_not_panic() {
    let empty: [bool; 0] = [];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &empty
        ),
        None
    );

    let all_disabled = [true, true];
    assert_eq!(
        accordion_next_index(0, "ArrowDown", Modifiers::default(), &all_disabled),
        None
    );

    let disabled = [false, false];
    assert_eq!(
        tabs_next_index(
            99,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 8: Menu/Select 共用の [`highlight_next_index`] は、highlight 位置
/// なし（`None`）からの ArrowDown で先頭、ArrowUp で末尾へ移動し、既定では
/// 循環しない（`menu_loop_focus_from_attr` の既定 false、Tabs の
/// `loop_focus_from_attr`・既定 true とは逆）。
#[test]
fn highlight_next_index_defaults_to_no_loop_and_menu_loop_focus_defaults_false() {
    let disabled = [false, false, false];
    assert!(!menu_loop_focus_from_attr(None));
    assert!(menu_loop_focus_from_attr(Some("true")));

    assert_eq!(
        highlight_next_index(None, "ArrowDown", false, Modifiers::default(), &disabled),
        Some(0)
    );
    assert_eq!(
        highlight_next_index(None, "ArrowUp", false, Modifiers::default(), &disabled),
        Some(2)
    );
    assert_eq!(
        highlight_next_index(Some(2), "ArrowDown", false, Modifiers::default(), &disabled),
        None
    );
    assert_eq!(
        highlight_next_index(Some(2), "ArrowDown", true, Modifiers::default(), &disabled),
        Some(0)
    );
}

/// 検証 9: RadioGroup 専用の [`radio_next_index`] は常に循環し、
/// `orientation` が `Some` のときその軸のキーのみを受理する
/// （`None` のときは両軸を受理する）。
#[test]
fn radio_next_index_always_loops_and_orientation_restricts_axis() {
    let disabled = [false, false, false];
    assert_eq!(
        radio_next_index(2, "ArrowRight", None, Modifiers::default(), &disabled),
        Some(0)
    );
    assert_eq!(
        radio_next_index(
            0,
            "ArrowDown",
            Some(Orientation::Horizontal),
            Modifiers::default(),
            &disabled
        ),
        None
    );
    assert_eq!(
        radio_next_index(
            0,
            "ArrowRight",
            Some(Orientation::Vertical),
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 10（イシュー #641）: typeahead の公開 API（[`is_typeahead_key`]/
/// [`typeahead_push`]/[`typeahead_next_index`]）が単体テストと同じ挙動で
/// 公開 API 経由でも壊れていないことを確認する統合確認。
#[test]
fn typeahead_public_api_matches_and_cycles_through_labels() {
    // 単一文字は修飾キーなし・非制御文字のみ typeahead 対象。
    assert!(is_typeahead_key("m", false, Modifiers::default()));
    assert!(!is_typeahead_key("Enter", false, Modifiers::default()));
    assert!(!is_typeahead_key(
        "m",
        false,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        }
    ));
    // Space はバッファ有効時のみ typeahead 対象。
    assert!(!is_typeahead_key(" ", false, Modifiers::default()));
    assert!(is_typeahead_key(" ", true, Modifiers::default()));

    // バッファはタイムアウト内で継続し、超過後は新規開始する。
    let buffer = typeahead_push("", "a", f64::INFINITY);
    assert_eq!(buffer, "a");
    let buffer = typeahead_push(&buffer, "p", 10.0);
    assert_eq!(buffer, "ap");
    let buffer = typeahead_push(&buffer, "z", TYPEAHEAD_TIMEOUT_MS + 1.0);
    assert_eq!(buffer, "z");

    // ラベルマッチ: 同一文字の繰り返しは current の次から循環探索する。
    let labels = ["Apple", "Banana", "Avocado"];
    assert_eq!(
        typeahead_next_index(Some(0), "a", &labels, &[false, false, false]),
        Some(2)
    );
    assert_eq!(
        typeahead_next_index(Some(2), "aa", &labels, &[false, false, false]),
        Some(0)
    );
    // disabled はスキップし、全 disabled・空バッファは None（fail-closed）。
    // current（Apple）自身は「次から」探索のため対象外、Banana は disabled
    // のためスキップされ Avocado がマッチする。
    assert_eq!(
        typeahead_next_index(Some(0), "a", &labels, &[false, true, false]),
        Some(2)
    );
    assert_eq!(
        typeahead_next_index(None, "", &labels, &[false, false, false]),
        None
    );
    let empty_labels: [&str; 0] = [];
    let empty_disabled: [bool; 0] = [];
    assert_eq!(
        typeahead_next_index(None, "a", &empty_labels, &empty_disabled),
        None
    );
}

/// 検証 11（イシュー #662）: サブメニュー（`trigger-item`）開閉判定の公開 API
/// （[`submenu_nav`]）が単体テストと同じ挙動で公開 API 経由でも壊れていない
/// ことを確認する統合確認。実際に展開・閉鎖できるかどうかの DOM 判断
/// （trigger-item か・disabled か・チェーン深さ 0 か等）は配線層の責務であり
/// 実ブラウザテスト（`keynav_browser.rs`）が担う。
#[test]
fn submenu_nav_public_api_matches_arrow_right_left_and_rejects_others() {
    assert_eq!(
        submenu_nav("ArrowRight", Modifiers::default()),
        Some(SubmenuNav::Open)
    );
    assert_eq!(
        submenu_nav("ArrowLeft", Modifiers::default()),
        Some(SubmenuNav::Close)
    );
    // 修飾キー付きは対象外。
    assert_eq!(
        submenu_nav(
            "ArrowRight",
            Modifiers {
                alt: true,
                ..Modifiers::default()
            }
        ),
        None
    );
    // ArrowRight/ArrowLeft 以外は対象外。
    assert_eq!(submenu_nav("ArrowDown", Modifiers::default()), None);
    assert_eq!(submenu_nav("Enter", Modifiers::default()), None);
}

// ---------------------------------------------------------------------
// Menubar（イシュー #1073）: `crates/wasm-full/src/keynav.rs::wiring`
// （`handle_menubar_trigger_keydown`/`move_menubar_focus`、wasm32 専用）が
// menubar のトリガー間移動・loop 既定値・サブメニュー開閉に再利用する既存
// 純粋関数を、その menubar セマンティクスの観点から検証する（受け入れ条件
// 1「既存 menu 配線の再利用判断の記録」の証跡）。配線層自身の DOM 挙動
// （実 click 合成・stale element 再解決等）は実ブラウザテスト
// （`keynav_browser.rs`）が担う。
// ---------------------------------------------------------------------

/// 検証 12（Menubar §closed トリガー間移動 1）: horizontal menubar の
/// ArrowRight/ArrowLeft によるトリガー間移動は [`tabs_next_index`] を
/// そのまま再利用する（`handle_menubar_trigger_keydown` の closed 分岐）。
#[test]
fn menubar_horizontal_trigger_navigation_reuses_tabs_next_index() {
    let disabled = [false, false, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        tabs_next_index(
            1,
            "ArrowLeft",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
}

/// 検証 13（Menubar §closed トリガー間移動 2）: horizontal menubar で
/// ArrowDown/ArrowUp は `None`（トリガー間移動の対象外）を返し、
/// `handle_menubar_trigger_keydown` が open 系キーへフォールスルーする
/// 条件になる（モジュール doc「キー順序規則」参照）。
#[test]
fn menubar_horizontal_vertical_keys_are_none_and_fall_through_to_open_keys() {
    let disabled = [false, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowDown",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowUp",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 14（Menubar §vertical menubar）: vertical menubar では
/// ArrowDown/ArrowUp がトリガー間移動、ArrowLeft/ArrowRight は `None`
/// （サブメニュー展開/復帰・未消費水平キーの扱いへ回る）ことを確認する。
#[test]
fn menubar_vertical_orientation_moves_on_vertical_keys_only() {
    let disabled = [false, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowDown",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowLeft",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 15（Menubar §`data-loop-focus` 契約）: `loop_focus=false`（既定）
/// では端で no-op、`true` では循環する（`Menubar` root の
/// `data-loop-focus` 属性がそのまま `tabs_next_index` の `loop_focus`
/// 引数として使われる）。
#[test]
fn menubar_loop_focus_flag_controls_wraparound_at_trigger_boundaries() {
    let disabled = [false, false, false];
    assert_eq!(
        tabs_next_index(
            2,
            "ArrowRight",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
    assert_eq!(
        tabs_next_index(
            2,
            "ArrowRight",
            Orientation::Horizontal,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
}

/// 検証 16（Menubar §disabled トリガーのスキップ）: disabled トリガーは
/// [`tabs_next_index`] の探索でスキップされる（`Menubar::trigger` の
/// `data-disabled`/`disabled` が `disabled_flags` へそのまま反映される
/// 契約、`handle_menubar_trigger_keydown` の DOM 側実装は
/// `keynav_browser.rs` が検証する）。
#[test]
fn menubar_disabled_triggers_are_skipped_during_navigation() {
    let disabled = [false, true, false];
    assert_eq!(
        tabs_next_index(
            0,
            "ArrowRight",
            Orientation::Horizontal,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(2)
    );
}

/// 検証 17（Menubar §loop 既定値、`Menubar::default()` との整合）:
/// `crates/headless-ui/src/menubar.rs` の `Menubar::default()` は
/// `loop_focus: false` を既定とする。DOM 側の既定値パーサ
/// （[`menu_loop_focus_from_attr`]）が同じ既定（`"true"` のときのみ
/// 循環、欠落/未知値は非循環）であることを固定し、状態機械と DOM の
/// 既定が乖離しないことを保証する（`handle_menubar_trigger_keydown` は
/// menu と同じくこの関数をそのまま再利用する、Tabs 用の
/// `loop_focus_from_attr`〔既定 true〕とは意図的に異なる関数を使う）。
#[test]
fn menubar_loop_focus_from_attr_default_matches_menubar_state_machine_default() {
    assert!(!menu_loop_focus_from_attr(None));
    assert!(!menu_loop_focus_from_attr(Some("false")));
    assert!(!menu_loop_focus_from_attr(Some("unknown")));
    assert!(menu_loop_focus_from_attr(Some("true")));
}

/// 検証 18（Menubar §サブメニュー開閉、`submenu_nav` の再利用確認）:
/// menubar の `sub-trigger`/`sub-content` に対する ArrowRight（展開）/
/// ArrowLeft（復帰）判定は menu の `trigger-item` と同じ
/// [`submenu_nav`] をそのまま再利用する（DOM 側の trigger-item/
/// sub-trigger 判定・disabled・チェーン深さ 0 の扱いは配線層
/// `ScopeSelectors::trigger_item` のスコープ切り替えが担い、
/// `keynav_browser.rs` が検証する）。
#[test]
fn menubar_submenu_open_close_reuses_submenu_nav() {
    assert_eq!(
        submenu_nav("ArrowRight", Modifiers::default()),
        Some(SubmenuNav::Open)
    );
    assert_eq!(
        submenu_nav("ArrowLeft", Modifiers::default()),
        Some(SubmenuNav::Close)
    );
}

/// 検証 19（Menubar §highlight 移動・初期 highlight）: サブメニュー
/// content 内の項目 highlight 移動・初期 highlight（先頭/末尾の非
/// disabled 項目）は menu と同じ [`highlight_next_index`] を再利用する
/// （`ScopeSelectors::item`/`trigger_item` によるスコープ切り替えのみが
/// menubar 固有、探索アルゴリズム自体は共通）。
#[test]
fn menubar_submenu_highlight_navigation_reuses_highlight_next_index() {
    let disabled = [false, false, false];
    assert_eq!(
        highlight_next_index(None, "ArrowDown", false, Modifiers::default(), &disabled),
        Some(0)
    );
    assert_eq!(
        highlight_next_index(Some(0), "ArrowDown", false, Modifiers::default(), &disabled),
        Some(1)
    );
    assert_eq!(
        highlight_next_index(Some(2), "ArrowDown", false, Modifiers::default(), &disabled),
        None
    );
}

/// 検証 12（イシュー #1071）: Combobox のキーボード判定の公開 API
/// （[`combobox_key_action`]）が単体テストと同じ挙動で公開 API 経由でも
/// 壊れていないことを確認する統合確認。Menu/Select と異なり closed の
/// Home/End/Enter/Escape・ArrowLeft/ArrowRight・printable 文字はすべて
/// no-op（テキスト `<input>` の既定動作を奪わない、モジュール doc
/// §Combobox 参照）。実際の DOM 解決・click 合成の検証は配線層の責務であり
/// 実ブラウザテスト（`keynav_browser.rs`）が担う。
#[test]
fn combobox_key_action_public_api_matches_open_move_confirm_close_contract() {
    assert_eq!(
        combobox_key_action("ArrowDown", Modifiers::default(), false),
        Some(ComboboxKeyAction::Open { from_end: false })
    );
    assert_eq!(
        combobox_key_action("ArrowUp", Modifiers::default(), false),
        Some(ComboboxKeyAction::Open { from_end: true })
    );
    // テキストフィールドの既定動作（キャレット移動・フォーム submit）を
    // 奪わない closed 時の no-op 群。
    assert_eq!(
        combobox_key_action("Home", Modifiers::default(), false),
        None
    );
    assert_eq!(
        combobox_key_action("End", Modifiers::default(), false),
        None
    );
    assert_eq!(
        combobox_key_action("Enter", Modifiers::default(), false),
        None
    );
    // fail-closed 回帰: closed で Escape を claim すると誤って open して
    // しまう。
    assert_eq!(
        combobox_key_action("Escape", Modifiers::default(), false),
        None
    );
    assert_eq!(combobox_key_action("a", Modifiers::default(), false), None);

    assert_eq!(
        combobox_key_action("ArrowDown", Modifiers::default(), true),
        Some(ComboboxKeyAction::MoveHighlight)
    );
    assert_eq!(
        combobox_key_action("Home", Modifiers::default(), true),
        Some(ComboboxKeyAction::MoveHighlight)
    );
    assert_eq!(
        combobox_key_action("Enter", Modifiers::default(), true),
        Some(ComboboxKeyAction::Confirm)
    );
    assert_eq!(
        combobox_key_action("Escape", Modifiers::default(), true),
        Some(ComboboxKeyAction::Close)
    );
    // typeahead 非適用（printable 文字は input の既定動作へ委ねる）。
    assert_eq!(combobox_key_action("a", Modifiers::default(), true), None);
    // 修飾キー付きは open/closed いずれも no-op。
    assert_eq!(
        combobox_key_action(
            "ArrowDown",
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            true
        ),
        None
    );
}

// ---------------------------------------------------------------------
// Listbox（イシュー #1070）: `crates/wasm-full/src/keynav.rs` の
// [`listbox_next_index`] を、Listbox の常時展開・trigger 非保持セマンティクス
// の観点から検証する。配線層自身の DOM 挙動は実ブラウザテスト
// （`keynav_browser.rs`）が担う。
// ---------------------------------------------------------------------

/// 検証 12（イシュー #1070）: Listbox 専用の [`listbox_next_index`] は
/// 既定 Vertical で ArrowDown/ArrowUp のみが動き、Horizontal 方向のキーは
/// no-op（`data-orientation` オプトインで軸を切り替える設計、モジュール doc
/// §Listbox 参照）。
#[test]
fn listbox_vertical_arrow_keys_move_highlight() {
    let disabled = [false, false, false];
    assert_eq!(
        listbox_next_index(
            Some(0),
            "ArrowDown",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        listbox_next_index(
            Some(1),
            "ArrowUp",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
    assert_eq!(
        listbox_next_index(
            Some(0),
            "ArrowRight",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        None
    );
}

/// 検証 13（イシュー #1070）: Home/End は disabled をスキップし、既定
/// （`data-loop-focus` 欠落）では端で循環しない（[`menu_loop_focus_from_attr`]
/// と loopFocus 既定を共有する契約の固定）。
#[test]
fn listbox_home_end_skip_disabled_and_default_does_not_loop() {
    let disabled = [true, false, false, true];
    assert_eq!(
        listbox_next_index(
            Some(2),
            "Home",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(1)
    );
    assert_eq!(
        listbox_next_index(
            Some(1),
            "End",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &disabled
        ),
        Some(2)
    );

    let all_enabled = [false, false, false];
    assert_eq!(
        listbox_next_index(
            Some(2),
            "ArrowDown",
            Orientation::Vertical,
            false,
            Modifiers::default(),
            &all_enabled
        ),
        None
    );
}

/// 検証 14（イシュー #1070）: `data-loop-focus="true"` 明示時のみ端で循環し、
/// 修飾キー付きは既知キーでも no-op（`"extended"` selection mode との衝突
/// 回避、モジュール doc §Listbox 参照）。
#[test]
fn listbox_loop_focus_true_wraps_and_modifiers_are_noop() {
    let disabled = [false, false, false];
    assert_eq!(
        listbox_next_index(
            Some(2),
            "ArrowDown",
            Orientation::Vertical,
            true,
            Modifiers::default(),
            &disabled
        ),
        Some(0)
    );
    assert_eq!(
        listbox_next_index(
            Some(0),
            "ArrowDown",
            Orientation::Vertical,
            true,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            &disabled
        ),
        None
    );
}

/// 検証: NavigationMenu trigger の代表ケース（イシュー #1075）。
#[test]
fn navigation_menu_trigger_key_action_representative_cases() {
    assert_eq!(
        navigation_menu_trigger_key_action(
            "ArrowDown",
            Modifiers::default(),
            Orientation::Horizontal,
            false
        ),
        Some(NavigationMenuKeyAction::OpenToLink { from_end: false })
    );
    assert_eq!(
        navigation_menu_trigger_key_action(
            "ArrowDown",
            Modifiers::default(),
            Orientation::Horizontal,
            true
        ),
        Some(NavigationMenuKeyAction::FocusLink { from_end: false })
    );
    assert_eq!(
        navigation_menu_trigger_key_action(
            "Escape",
            Modifiers::default(),
            Orientation::Horizontal,
            false
        ),
        None
    );
}

/// 検証: NavigationMenu content 内リンクの代表ケース（イシュー #1075）。
#[test]
fn navigation_menu_link_next_index_representative_cases() {
    let disabled = [false, false, false];
    assert_eq!(
        navigation_menu_link_next_index(0, "ArrowDown", Modifiers::default(), &disabled),
        Some(1)
    );
    // 非循環: 末尾で ArrowDown は None。
    assert_eq!(
        navigation_menu_link_next_index(2, "ArrowDown", Modifiers::default(), &disabled),
        None
    );
}

/// 検証: ToggleGroup と RadioGroup がインデックス計算を共有していること
/// （共通化判断の機械的な固定、イシュー #1075、モジュール doc
/// §ToggleGroup 参照）。
#[test]
fn toggle_group_next_index_matches_radio_next_index() {
    let disabled = [false, false, true, false];
    assert_eq!(
        toggle_group_next_index(0, "ArrowRight", None, Modifiers::default(), &disabled),
        radio_next_index(0, "ArrowRight", None, Modifiers::default(), &disabled)
    );
}

// --- 検証 15（イシュー #1074）: Calendar/Splitter の公開 API 経由の統合確認
// （詳細な網羅ケースは `crates/wasm-full/src/keynav.rs` の単体テストに既に
// 持つため、本ファイルは「公開 API 経由で壊れていないか」に絞る）。---

#[test]
fn calendar_next_index_is_reachable_via_public_api() {
    let disabled = vec![false; 14];
    assert_eq!(
        calendar_next_index(3, "ArrowRight", 7, Modifiers::default(), &disabled),
        Some(4)
    );
    assert_eq!(
        calendar_next_index(3, "ArrowDown", 7, Modifiers::default(), &disabled),
        Some(10)
    );
}

#[test]
fn splitter_key_action_is_reachable_via_public_api() {
    assert_eq!(
        splitter_key_action("ArrowRight", Orientation::Horizontal, Modifiers::default()),
        Some(SplitterKeyAction::Increment)
    );
    assert_eq!(
        splitter_key_action("End", Orientation::Vertical, Modifiers::default()),
        Some(SplitterKeyAction::SetToMax)
    );
}

/// 検証 16（イシュー #1074、AC「`aria-valuenow` 連動更新」の本体）:
/// `fandhe_frontend_headless_ui::splitter::Splitter` へ `SplitterKeyAction`
/// 相当の dispatch 名（`"increment"`/`"home"`/`"end"`）を適用すると、
/// `Splitter::resize_trigger` が再生成する SSR 出力の `aria-valuenow` が
/// 追随することを固定する。keynav モジュール doc §1.1a（「`aria-valuenow`
/// のみを直接書き換えない」設計判断）の裏付けであり、DOM 反映が dispatch →
/// 再描画（`crate::lib::Runtime::wire` 経路）でのみ成立することを headless-ui
/// を直接操作して native から実証する。
#[test]
fn splitter_dispatch_increment_updates_aria_valuenow_on_rerender() {
    use fandhe_frontend_headless_ui::splitter::Splitter;

    let mut s = Splitter::default();
    let before =
        fandhe_frontend_core::render(&s.resize_trigger(0, "panel-0", false, vec![], vec![]));
    assert!(before.contains(r#"aria-valuenow="50""#));

    assert!(fandhe_frontend_interactive::dispatch(
        &mut s,
        "increment",
        "0"
    ));
    let after =
        fandhe_frontend_core::render(&s.resize_trigger(0, "panel-0", false, vec![], vec![]));
    assert!(after.contains(r#"aria-valuenow="51""#));
}

#[test]
fn splitter_dispatch_home_and_end_update_aria_valuenow_to_min_and_max() {
    use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter};

    let mut s = Splitter::new(
        &[
            PanelSpec::new(50.0, 20.0, 80.0),
            PanelSpec::new(50.0, 20.0, 80.0),
        ],
        fandhe_frontend_headless_ui::data_attrs::Orientation::Horizontal,
    );

    assert!(fandhe_frontend_interactive::dispatch(&mut s, "home", "0"));
    let home_html =
        fandhe_frontend_core::render(&s.resize_trigger(0, "panel-0", false, vec![], vec![]));
    assert!(home_html.contains(r#"aria-valuenow="20""#));

    assert!(fandhe_frontend_interactive::dispatch(&mut s, "end", "0"));
    let end_html =
        fandhe_frontend_core::render(&s.resize_trigger(0, "panel-0", false, vec![], vec![]));
    assert!(end_html.contains(r#"aria-valuenow="80""#));
}

/// 検証: 範囲外 trigger（存在しない index）の `"set"`/`"increment"` は
/// no-op（fail-closed、panic しない）。
#[test]
fn splitter_dispatch_out_of_range_trigger_is_noop_without_panic() {
    use fandhe_frontend_headless_ui::splitter::Splitter;

    let mut s = Splitter::default();
    assert!(fandhe_frontend_interactive::dispatch(
        &mut s,
        "increment",
        "9"
    ));
    assert_eq!(s.size(0), Some(50.0));
    assert_eq!(s.size(1), Some(50.0));
}
