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
fn stylesheet_declares_highlighted_state_selectors_for_trigger_item_and_sub_trigger() {
    // イシュー #1702: headless 層が roving tabindex のポインタ移動時に
    // trigger へも `data-highlighted` を出力する契約
    // （`crates/headless-ui/src/menubar.rs::trigger` 参照）ため、item/
    // sub-trigger に加え trigger も検証する。
    let css = menubar::stylesheet();
    assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-highlighted]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="item"][data-highlighted]"#));
    assert!(css.contains(r#"[data-scope="menubar"][data-part="sub-trigger"][data-highlighted]"#));
}

#[test]
fn stylesheet_declares_trigger_hover_rule_excluding_highlighted_and_open() {
    // イシュー #1702: highlight 中・open 中のいずれでも hover の淡い背景が
    // accent / accent-subtle 背景を洗い流さないよう
    // `HoverExceptAttrEq("data-highlighted", "data-state", "open")` を使う
    // （highlighted 分は PR #1745 P1 指摘・menu 3/3 `trigger-item` と同型の
    // 回帰防止、open 分は PR #1803 Bugbot Medium severity 指摘「Hover
    // washes out open trigger」の回帰防止）。`@media (hover: hover)` 配下へ
    // 集約出力される契約（`crates/pre-styled-ui/src/recipe.rs` 参照）。
    let css = menubar::stylesheet();
    assert!(css.contains("@media (hover: hover)"));
    assert!(css.contains(
        r#"[data-scope="menubar"][data-part="trigger"]:hover:not([data-disabled]):not([data-highlighted]):not([data-state="open"]) {"#
    ));
}

#[test]
fn stylesheet_declares_trigger_focus_ring_in_canonical_token_form() {
    // イシュー #1702: 直書き `outline: 2px solid var(--fandhe-color-accent)`
    // から `focus_ring_declarations(FocusRingColor::Token,
    // FocusRingOffset::Outside)`（イシュー #1424 canonical ヘルパ）へ
    // 置換したトークン参照形（フォールバック連鎖込み）を固定する。
    let css = menubar::stylesheet();
    assert!(css.contains(
        "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
    ));
    assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
}

#[test]
fn stylesheet_declares_root_border_and_radius() {
    // イシュー #1702: `border-bottom` 単独から全辺 `border` +
    // `border-radius: var(--fandhe-radius-md)` へ拡張（root shadow は
    // 意図的に追加しない、`crates/pre-styled-ui/src/menubar.rs` モジュール
    // doc「意図的に合わせなかった点」節参照）。
    let css = menubar::stylesheet();
    assert!(css.contains(
        "[data-scope=\"menubar\"][data-part=\"root\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n"
    ));
}

#[test]
fn stylesheet_declares_trigger_transition() {
    let css = menubar::stylesheet();
    assert!(css
        .contains("[data-scope=\"menubar\"][data-part=\"trigger\"] {\n  display: inline-flex;\n"));
    assert!(css.contains("transition-property: background, color;"));
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
