//! イシュー #2542「cursor 部品（カスタムカーソル・追従）」の golden・
//! 契約テスト。`motion` feature 配下のみコンパイルする（feature off では
//! 空テストバイナリ、`tests/motion_zero_cost.rs` の「無効時ゼロコスト」
//! 契約と両立する。`tests/motion_border_beam_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::cursor::{self, CURSOR_ATTR};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `cursor::CURSOR_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_cursor_css` の実出力を貼り付ける。
const EXPECTED_CURSOR_CSS: &str = "[data-fandhe-cursor] {\n  position: fixed;\n  top: 0;\n  left: 0;\n  width: 20px;\n  height: 20px;\n  margin: 0;\n  border-radius: var(--fandhe-radius-full, 9999px);\n  background: var(--fandhe-color-accent);\n  z-index: var(--fandhe-z-index-max, 2147483647);\n  pointer-events: none;\n  transform: translate(var(--fandhe-motion-cursor-x, -100px), var(--fandhe-motion-cursor-y, -100px)) translate(-50%, -50%);\n  transition-property: width, height, opacity, background-color, border-radius;\n  transition-duration: var(--fandhe-motion-duration-fast);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n  opacity: 1;\n}\n[data-fandhe-cursor]:not([data-fandhe-cursor-state]) {\n  display: none;\n}\n[data-fandhe-cursor][data-fandhe-cursor-state=\"hidden\"] {\n  opacity: 0;\n}\n[data-fandhe-cursor][data-fandhe-cursor-variant=\"ring\"] {\n  width: 40px;\n  height: 40px;\n  background: transparent;\n  border: 2px solid var(--fandhe-color-accent);\n}\n[data-fandhe-cursor][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"]) {\n  width: auto;\n  height: auto;\n  padding: 4px 10px;\n  border-radius: var(--fandhe-radius-full, 9999px);\n  color: var(--fandhe-color-on-accent, #fff);\n  font-size: var(--fandhe-font-size-xs);\n  white-space: nowrap;\n}\n[data-fandhe-cursor][data-fandhe-cursor-variant=\"ring\"][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"]) {\n  background: var(--fandhe-color-accent);\n  border: none;\n}\n[data-fandhe-cursor][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"])::after {\n  content: attr(data-fandhe-cursor-label);\n}\n[data-fandhe-cursor-active],\n[data-fandhe-cursor-active] * {\n  cursor: none;\n}\n@media (prefers-reduced-motion: reduce), (pointer: coarse), (hover: none) {\n  [data-fandhe-cursor] {\n    display: none;\n  }\n  [data-fandhe-cursor-active],\n  [data-fandhe-cursor-active] * {\n    cursor: auto;\n  }\n}\n";

#[test]
fn cursor_css_matches_golden() {
    assert_eq!(cursor::CURSOR_CSS, EXPECTED_CURSOR_CSS);
}

#[test]
fn cursor_attr_is_expected_literal() {
    assert_eq!(CURSOR_ATTR, "data-fandhe-cursor");
}

#[test]
fn does_not_use_angle_bracket_literal() {
    assert!(!cursor::CURSOR_CSS.contains('<'));
}

#[test]
fn reduced_motion_block_appears_after_normal_block_and_disables_native_cursor_hiding() {
    let (before, after) = cursor::CURSOR_CSS
        .split_once("@media (prefers-reduced-motion: reduce), (pointer: coarse), (hover: none) {")
        .expect("フェイルセーフ用 @media ブロックが見つからない");
    assert!(before.contains("[data-fandhe-cursor-active],"));
    assert!(after.contains("display: none;"));
    assert!(after.contains("cursor: auto;"));
}

#[test]
fn to_css_with_cursor_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_cursor();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + cursor::CURSOR_CSS.len());
    assert_eq!(&extended[base.len()..], cursor::CURSOR_CSS);
}

#[test]
fn cursor_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(cursor::CURSOR_CSS).is_ok());
}

#[test]
fn cursor_node_renders_marker_and_aria_hidden() {
    let node = cursor::cursor(vec![]);
    let rendered = fandhe_frontend_pre_styled_ui::fandhe_frontend_core::render(&node);
    assert!(rendered.contains("data-fandhe-cursor=\"\""));
    assert!(rendered.contains("aria-hidden=\"true\""));
}
