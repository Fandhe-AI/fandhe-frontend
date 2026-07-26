//! `fandhe_frontend_wasm_full::keynav`（Tabs/Accordion/Menu/Select/
//! RadioGroup/Combobox のキーボード操作・イシュー #582・#583・#641
//! （typeahead）・#1071（Combobox）、親 #581）の native テスト。
//!
//! `keynav` モジュールの純粋層（[`tabs_next_index`]/[`accordion_next_index`]/
//! [`highlight_next_index`]/[`radio_next_index`]/[`is_typeahead_key`]/
//! [`typeahead_push`]/[`typeahead_next_index`]）は web-sys に依存しないため、
//! `wasm32` ターゲット・実 DOM を介さず native の `cargo test --workspace` から
//! 公開 API 経由で直接検証できる（`wasm-full/tests/nav_native.rs` と同じ
//! 2 層構成方針）。詳細な網羅ケース（orientation 別 Arrow・disabled スキップ・
//! loopFocus 有無・typeahead のバッファ/循環等）はモジュール内単体テスト
//! （`crates/wasm-full/src/keynav.rs`）に既に持つため、本ファイルは「公開 API
//! 経由で壊れていないか」の統合確認に絞る。配線層（`wire_keynav`、
//! `#[cfg(target_arch = "wasm32")]`）の検証は `wasm-full/tests/keynav_browser.rs`
//! （実ブラウザ）が担う。

use fandhe_frontend_wasm_full::keynav::{
    accordion_next_index, combobox_key_action, highlight_next_index, is_typeahead_key,
    loop_focus_from_attr, menu_loop_focus_from_attr, radio_next_index, submenu_nav,
    tabs_next_index, typeahead_next_index, typeahead_push, ComboboxKeyAction, Modifiers,
    Orientation, SubmenuNav, TYPEAHEAD_TIMEOUT_MS,
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
