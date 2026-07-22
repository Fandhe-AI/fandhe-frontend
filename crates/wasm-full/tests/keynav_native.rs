//! `fandhe_frontend_wasm_full::keynav`（Tabs/Accordion のキーボード操作・
//! イシュー #582、親 #581）の native テスト。
//!
//! `keynav` モジュールの純粋層（[`tabs_next_index`]/[`accordion_next_index`]）
//! は web-sys に依存しないため、`wasm32` ターゲット・実 DOM を介さず native の
//! `cargo test --workspace` から公開 API 経由で直接検証できる
//! （`wasm-full/tests/nav_native.rs` と同じ 2 層構成方針）。詳細な網羅ケース
//! （orientation 別 Arrow・disabled スキップ・loopFocus 有無等）はモジュール内
//! 単体テスト（`crates/wasm-full/src/keynav.rs`）に既に持つため、本ファイルは
//! 「公開 API 経由で壊れていないか」の統合確認に絞る。配線層
//! （`wire_keynav`、`#[cfg(target_arch = "wasm32")]`）の検証は
//! `wasm-full/tests/keynav_browser.rs`（実ブラウザ）が担う。

use fandhe_frontend_wasm_full::keynav::{
    accordion_next_index, loop_focus_from_attr, tabs_next_index, Modifiers, Orientation,
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
