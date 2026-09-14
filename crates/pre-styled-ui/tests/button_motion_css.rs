//! イシュー #2538「button の Motion+ 由来 variant」の golden・契約テスト。
//! `motion` feature 配下のみコンパイルする（feature off では空テスト
//! バイナリ、`tests/motion_zero_cost.rs` の「無効時ゼロコスト」契約と
//! 両立する。`tests/motion_border_beam_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_core::{render, text};
use fandhe_frontend_pre_styled_ui::button::ButtonProps;
use fandhe_frontend_pre_styled_ui::button_motion::{
    add_to_basket_button, hold_to_confirm_button, rolling_text_button, rolling_text_stagger_button,
    BUTTON_MOTION_CSS,
};
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `button_motion::BUTTON_MOTION_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test button_motion_css` の実出力を貼り付ける。
const EXPECTED_BUTTON_MOTION_CSS: &str = "[data-scope=\"button\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n}\n[data-scope=\"button\"][data-part=\"rolling-text-current\"],\n[data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] {\n  display: block;\n  transition: transform 0.3s ease;\n}\n[data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] {\n  position: absolute;\n  inset: 0;\n  transform: translateY(100%);\n}\n@media (hover: hover) {\n  [data-scope=\"button\"][data-part=\"root\"]:hover [data-part=\"rolling-text-current\"] {\n    transform: translateY(-100%);\n  }\n  [data-scope=\"button\"][data-part=\"root\"]:hover [data-part=\"rolling-text-duplicate\"] {\n    transform: translateY(0);\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"button\"][data-part=\"rolling-text-current\"],\n  [data-scope=\"button\"][data-part=\"rolling-text-duplicate\"] {\n    transition: none;\n  }\n}\n[data-scope=\"button\"][data-part=\"hold-fill\"] {\n  position: absolute;\n  inset: 0;\n  width: calc(var(--fandhe-motion-hold-progress, 0) * 100%);\n  background: currentColor;\n  opacity: 0.2;\n  pointer-events: none;\n  transition: none;\n}\n[data-scope=\"button\"][data-part=\"root\"][data-state=\"confirmed\"] {\n  transition: background-color 0.2s ease;\n}\n[data-scope=\"button\"][data-part=\"basket-icon-adding\"],\n[data-scope=\"button\"][data-part=\"basket-icon-added\"] {\n  display: none;\n}\n[data-scope=\"button\"][data-part=\"root\"][data-state=\"adding\"] [data-part=\"basket-icon-idle\"],\n[data-scope=\"button\"][data-part=\"root\"][data-state=\"added\"] [data-part=\"basket-icon-idle\"] {\n  display: none;\n}\n[data-scope=\"button\"][data-part=\"root\"][data-state=\"adding\"] [data-part=\"basket-icon-adding\"] {\n  display: inline-flex;\n}\n[data-scope=\"button\"][data-part=\"root\"][data-state=\"added\"] [data-part=\"basket-icon-added\"] {\n  display: inline-flex;\n}\n";

#[test]
fn button_motion_css_matches_golden() {
    assert_eq!(BUTTON_MOTION_CSS, EXPECTED_BUTTON_MOTION_CSS);
}

#[test]
fn css_has_no_forbidden_angle_bracket() {
    assert!(!BUTTON_MOTION_CSS.contains('<'));
}

#[test]
fn to_css_with_button_motion_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_button_motion();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + BUTTON_MOTION_CSS.len());
    assert_eq!(&extended[base.len()..], BUTTON_MOTION_CSS);
}

#[test]
fn reduced_motion_block_redefines_rolling_text_transition() {
    let (before, after) = BUTTON_MOTION_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    assert!(before.contains(r#"[data-part="rolling-text-current"],"#));
    assert!(after.contains("transition: none;"));
}

#[test]
fn rolling_text_button_renders_two_layers() {
    let node = rolling_text_button(&ButtonProps::default(), vec![], "Save");
    let html = render(&node);
    assert!(html.contains(r#"data-part="rolling-text-current""#));
    assert!(html.contains(r#"data-part="rolling-text-duplicate""#));
}

#[test]
fn rolling_text_stagger_button_renders_char_spans() {
    let node = rolling_text_stagger_button(&ButtonProps::default(), vec![], "Go");
    let html = render(&node);
    assert!(html.contains("--fandhe-motion-stagger-index: 0"));
    assert!(html.contains("--fandhe-motion-stagger-index: 1"));
}

#[test]
fn hold_to_confirm_button_renders_fill_layer_and_opt_in_attr() {
    let node = hold_to_confirm_button(&ButtonProps::default(), vec![], None, vec![text("Hold")]);
    let html = render(&node);
    assert!(html.contains("data-fandhe-hold-to-confirm"));
    assert!(html.contains(r#"data-part="hold-fill""#));
}

#[test]
fn add_to_basket_button_renders_three_icon_layers_and_opt_in_attr() {
    let node = add_to_basket_button(
        &ButtonProps::default(),
        vec![],
        "Add to basket",
        text("+"),
        text("..."),
        text("v"),
    );
    let html = render(&node);
    assert!(html.contains("data-fandhe-add-to-basket"));
    assert!(html.contains(r#"data-part="basket-icon-idle""#));
    assert!(html.contains(r#"data-part="basket-icon-adding""#));
    assert!(html.contains(r#"data-part="basket-icon-added""#));
}
