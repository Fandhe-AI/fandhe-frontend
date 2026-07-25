//! styled Navigation Menu（イシュー #993）の CSS 契約テスト。
//!
//! `crates/pre-styled-ui/tests/menubar_css.rs` と同型の判断（golden fixture
//! 方式は宣言 1 個の増減でも無関係な diff が広範囲に生じる brittle さの方が
//! 実害が大きい）に従い、「6 スロット分のセレクタが存在する」「主要な状態
//! セレクタが揃っている」「CSS breakout を含まない」という不変条件を契約
//! アサーションとして固定する。

use fandhe_frontend_pre_styled_ui::navigation_menu;

const EXPECTED_SLOTS: &[&str] = &["root", "list", "item", "trigger", "content", "link"];

#[test]
fn stylesheet_is_deterministic() {
    assert_eq!(navigation_menu::stylesheet(), navigation_menu::stylesheet());
}

#[test]
fn stylesheet_declares_selectors_for_all_six_anatomy_slots() {
    let css = navigation_menu::stylesheet();
    for part in EXPECTED_SLOTS {
        let needle = format!(r#"[data-scope="navigation-menu"][data-part="{part}"]"#);
        assert!(
            css.contains(&needle),
            "missing base selector for part={part}"
        );
    }
}

#[test]
fn stylesheet_declares_open_state_selector_for_trigger() {
    let css = navigation_menu::stylesheet();
    assert!(
        css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"][data-state="open"]"#)
    );
}

#[test]
fn stylesheet_declares_current_state_selector_for_link() {
    let css = navigation_menu::stylesheet();
    assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="link"][data-current]"#));
}

#[test]
fn stylesheet_declares_disabled_state_selectors() {
    let css = navigation_menu::stylesheet();
    assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"][data-disabled]"#));
    assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="item"][data-disabled]"#));
}

#[test]
fn stylesheet_declares_list_align_items_flex_start_not_center() {
    // showcase の content 中和時に 1 項目だけ縦に伸びて他項目が縦ずれする
    // 回帰（PR #1000 の反省、`crates/pre-styled-ui/src/navigation_menu.rs`
    // モジュール doc「レイアウト」節参照）を構造的に防ぐための固定。
    let css = navigation_menu::stylesheet();
    assert!(css.contains(
        "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: flex-start;\n  "
    ));
    assert!(!css.contains(
        "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: center;\n"
    ));
}

#[test]
fn stylesheet_declares_focus_visible_ring_only_on_trigger() {
    let css = navigation_menu::stylesheet();
    assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"]:focus-visible {"#));
    assert!(!css.contains(r#"[data-scope="navigation-menu"][data-part="link"]:focus-visible {"#));
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = navigation_menu::stylesheet();
    assert!(!css.contains('<'));
    assert!(!css.contains("</style"));
}
