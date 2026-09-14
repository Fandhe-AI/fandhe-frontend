//! イシュー #2531「border-beam 装飾オプション（opt-in）」の golden・契約
//! テスト。`motion` feature 配下のみコンパイルする（feature off では空
//! テストバイナリ、`tests/motion_zero_cost.rs` の「無効時ゼロコスト」契約
//! と両立する。`tests/motion_keyframes_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::border_beam::{self, BORDER_BEAM_CLASS};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `border_beam::BORDER_BEAM_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_border_beam_css` の実出力を貼り付ける。
const EXPECTED_BORDER_BEAM_CSS: &str = ".fd-border-beam {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  padding: var(--fandhe-border-beam-width, 1px);\n}\n.fd-border-beam::before {\n  content: \"\";\n  position: absolute;\n  top: 50%;\n  left: 50%;\n  min-width: 200%;\n  min-height: 200%;\n  aspect-ratio: 1;\n  z-index: -1;\n  pointer-events: none;\n  transform: translate(-50%, -50%) rotate(0deg);\n  background: conic-gradient(from 0deg, transparent, var(--fandhe-border-beam-color, var(--fandhe-color-accent)) var(--fandhe-border-beam-spread, 10%), transparent calc(var(--fandhe-border-beam-spread, 10%) * 2));\n  animation: fd-border-beam-rotate var(--fandhe-border-beam-duration, 6s) linear infinite;\n}\n@keyframes fd-border-beam-rotate {\n  to {\n    transform: translate(-50%, -50%) rotate(360deg);\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  .fd-border-beam {\n    overflow: visible;\n    padding: 0;\n    border: var(--fandhe-border-beam-width, 1px) solid var(--fandhe-border-beam-color, var(--fandhe-color-border-emphasized));\n  }\n  .fd-border-beam::before {\n    content: none;\n  }\n}\n";

#[test]
fn border_beam_css_matches_golden() {
    assert_eq!(border_beam::BORDER_BEAM_CSS, EXPECTED_BORDER_BEAM_CSS);
}

#[test]
fn class_name_is_expected_literal() {
    assert_eq!(BORDER_BEAM_CLASS, "fd-border-beam");
}

#[test]
fn does_not_use_property_registration_or_angle_bracket_literal() {
    // モジュール doc「技術選定」節: `@property … syntax: "<angle>"` は
    // `<` を含むため使えない（`recipe.rs`/`scroll_area.rs` と同じ不変
    // 条件）。`transform: rotate()` 方式で代替している。
    assert!(!border_beam::BORDER_BEAM_CSS.contains("@property"));
    assert!(!border_beam::BORDER_BEAM_CSS.contains('<'));
}

#[test]
fn reduced_motion_block_appears_after_normal_block_and_disables_animation() {
    let (before, after) = border_beam::BORDER_BEAM_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    assert!(before.contains(".fd-border-beam::before {"));
    // 回転レイヤーを `content: none` で非生成化し、`overflow: hidden`/
    // `padding` を解除して静的な `border` へ縮退する（Issue #2531）。
    assert!(after.contains("content: none;"));
    assert!(after.contains("overflow: visible;"));
    assert!(after.contains("border: var(--fandhe-border-beam-width, 1px) solid"));
}

#[test]
fn to_css_with_border_beam_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_border_beam();
    assert!(extended.starts_with(&base));
    assert_eq!(
        extended.len(),
        base.len() + border_beam::BORDER_BEAM_CSS.len()
    );
    assert_eq!(&extended[base.len()..], border_beam::BORDER_BEAM_CSS);
}

#[test]
fn border_beam_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(border_beam::BORDER_BEAM_CSS).is_ok());
    assert!(!border_beam::BORDER_BEAM_CSS.contains('<'));
}
