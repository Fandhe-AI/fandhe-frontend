//! `list_motion` モジュール（イシュー #2544）の golden・決定性・
//! FLIP 非干渉・XSS 回帰契約テスト。`tests/motion_forms_css.rs`
//! （`docs/internal/pre-styled-ui-golden-test-update-guide.md` 参照）と
//! 同型。

#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::list_motion::{
    enter_css, exit_css, list_motion_css, ENTER_KEYFRAMES_NAME, EXIT_KEYFRAMES_NAME,
    PRESENCE_AUTO_ATTR,
};

#[test]
fn list_motion_css_is_deterministic() {
    assert_eq!(list_motion_css(), list_motion_css());
}

#[test]
fn list_motion_css_never_contains_style_breakout_sequences() {
    let css = list_motion_css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn list_motion_css_never_touches_transform_property() {
    // FLIP（`fandhe-frontend-wasm-full::layout_flip`）は `transform` を
    // 毎フレーム inline `!important` で書き込むため、本モジュールの CSS が
    // `transform` に触れると追従が鈍る（モジュール doc「`transition` では
    // なく `@keyframes` アニメーションを使う理由」節参照）。
    let css = list_motion_css();
    assert!(!css.contains("transform"));
}

#[test]
fn enter_css_is_scoped_under_presence_auto_attr() {
    let css = enter_css();
    assert!(css.contains(&format!("[{PRESENCE_AUTO_ATTR}]")));
    assert!(css.contains("[data-scope=\"list\"][data-part=\"root\"]"));
    assert!(css.contains("> [data-scope=\"list\"][data-part=\"item\"]"));
}

#[test]
fn exit_css_targets_exiting_state_with_zero_delay() {
    let css = exit_css();
    assert!(css.contains(r#"[data-state="exiting"]"#));
    assert!(css.contains("animation-delay: 0s;"));
}

#[test]
fn keyframes_names_are_distinct_from_each_other() {
    assert_ne!(ENTER_KEYFRAMES_NAME, EXIT_KEYFRAMES_NAME);
}

#[test]
fn list_motion_css_passes_stylesheet_push_css() {
    let mut sheet = fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet::new();
    assert!(sheet.push_css(&list_motion_css()).is_ok());
}

#[test]
fn to_css_with_list_motion_is_pure_append() {
    let theme = fandhe_frontend_pre_styled_ui::theme::Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_list_motion();
    let extra = list_motion_css();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + extra.len());
    assert_eq!(&extended[base.len()..], extra);
}
