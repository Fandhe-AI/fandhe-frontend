//! styled Menubar（イシュー #992）の CSS 契約テスト。
//!
//! `crates/pre-styled-ui/tests/menu_css.rs`/`toggle_tip_css.rs` の golden
//! fixture 方式（CSS 全文のバイト単位固定）は、11 パーツを持つ Menubar では
//! 宣言 1 個の増減でも無関係な diff が広範囲に生じる brittle さの方が実害が
//! 大きいと判断し、本ファイルでは採らない（イシュー #992 実装計画で確定
//! 済みの判断）。代わりに「11 スロット分のセレクタが存在する」「主要な
//! 状態セレクタが揃っている」「CSS breakout を含まない」という不変条件を
//! 契約アサーションとして固定する。

use fandhe_frontend_pre_styled_ui::menubar;

const EXPECTED_SLOTS: &[&str] = &[
    "root",
    "menu",
    "trigger",
    "positioner",
    "content",
    "item",
    "item-group",
    "item-group-label",
    "separator",
    "sub-trigger",
    "sub-content",
];

#[test]
fn stylesheet_is_deterministic() {
    assert_eq!(menubar::stylesheet(), menubar::stylesheet());
}

#[test]
fn stylesheet_declares_selectors_for_all_eleven_anatomy_slots() {
    let css = menubar::stylesheet();
    for part in EXPECTED_SLOTS {
        let needle = format!(r#"[data-scope="menubar"][data-part="{part}"]"#);
        assert!(
            css.contains(&needle),
            "missing base selector for part={part}"
        );
    }
}

#[test]
fn stylesheet_declares_open_state_selectors_for_trigger_and_sub_trigger() {
    let css = menubar::stylesheet();
    assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-state="open"]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="sub-trigger"][data-state="open"]"#));
}

#[test]
fn stylesheet_declares_highlighted_state_selectors_for_item_and_sub_trigger() {
    let css = menubar::stylesheet();
    assert!(css.contains(r#"[data-scope="menubar"][data-part="item"][data-highlighted]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="sub-trigger"][data-highlighted]"#));
}

#[test]
fn stylesheet_declares_disabled_state_selectors() {
    let css = menubar::stylesheet();
    assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-disabled]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="item"][data-disabled]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="sub-trigger"][data-disabled]"#));
}

#[test]
fn stylesheet_declares_vertical_orientation_selector_on_root() {
    let css = menubar::stylesheet();
    assert!(
        css.contains(r#"[data-scope="menubar"][data-part="root"][data-orientation="vertical"]"#)
    );
    assert!(css.contains("flex-direction: column;"));
}

#[test]
fn stylesheet_declares_focus_visible_ring_only_on_trigger() {
    let css = menubar::stylesheet();
    assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"]:focus-visible {"#));
    // virtual focus パーツ（item/sub-trigger）は実 DOM フォーカスを受けない
    // ため `:focus-visible` を付けない（モジュール doc「focus-visible
    // リング」節参照）。
    assert!(!css.contains(r#"[data-scope="menubar"][data-part="item"]:focus-visible {"#));
    assert!(!css.contains(r#"[data-scope="menubar"][data-part="sub-trigger"]:focus-visible {"#));
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = menubar::stylesheet();
    assert!(!css.contains('<'));
    assert!(!css.contains("</style"));
}
