//! イシュー #2532「text アニメーション部品（split-text reveal / typewriter
//! / scramble）」の golden・契約テスト。`motion` feature 配下のみコンパイル
//! する（`tests/motion_border_beam_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::text_reveal::{
    self, chars, scramble, typewriter, words, SCRAMBLE_ATTR, TYPEWRITER_ATTR,
};
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `text_reveal::TEXT_REVEAL_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_text_reveal_css` の実出力を貼り付ける。
const EXPECTED_TEXT_REVEAL_CSS: &str = ".fd-text-reveal__sr {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n  border: 0;\n}\n.fd-text-reveal__word {\n  white-space: nowrap;\n  display: inline-block;\n}\n.fd-text-reveal__unit {\n  display: inline-block;\n  animation: fd-text-reveal-in var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard) both;\n  animation-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-text-reveal-step, 40ms));\n}\n@keyframes fd-text-reveal-in {\n  from {\n    opacity: 0;\n    translate: 0 0.4em;\n  }\n  to {\n    opacity: 1;\n    translate: none;\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  .fd-text-reveal__unit {\n    animation: none;\n  }\n}\n";

#[test]
fn text_reveal_css_matches_golden() {
    assert_eq!(text_reveal::TEXT_REVEAL_CSS, EXPECTED_TEXT_REVEAL_CSS);
}

#[test]
fn text_reveal_css_has_no_forbidden_angle_bracket() {
    assert!(!text_reveal::TEXT_REVEAL_CSS.contains('<'));
}

#[test]
fn to_css_with_text_reveal_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_text_reveal();
    assert!(extended.starts_with(&base));
    assert_eq!(
        extended.len(),
        base.len() + text_reveal::TEXT_REVEAL_CSS.len()
    );
    assert_eq!(&extended[base.len()..], text_reveal::TEXT_REVEAL_CSS);
}

#[test]
fn text_reveal_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(text_reveal::TEXT_REVEAL_CSS).is_ok());
}

#[test]
fn reduced_motion_block_appears_after_normal_block_and_disables_animation() {
    let (before, after) = text_reveal::TEXT_REVEAL_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    assert!(before.contains(".fd-text-reveal__unit {"));
    assert!(after.contains("animation: none;"));
}

/// SSR 出力の XSS 回帰: `<script>` を含むテキストが分割・エスケープされ、
/// `aria-hidden`/visually-hidden 構造が保たれることを確認する。
#[test]
fn chars_render_escapes_untrusted_text_and_keeps_structure() {
    let html = render(&chars("<script>alert(1)</script>"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains(r#"class="fd-text-reveal__sr""#));
    assert!(html.contains(r#"class="fd-text-reveal__units" aria-hidden="true""#));
}

#[test]
fn words_render_escapes_untrusted_text() {
    let html = render(&words("<img src=x>"));
    assert!(!html.contains("<img"));
    assert!(html.contains("&lt;img"));
}

#[test]
fn typewriter_render_escapes_untrusted_text_and_carries_attr() {
    let html = render(&typewriter("<b>hi</b>", None));
    assert!(!html.contains("<b>hi</b>"));
    assert!(html.contains(TYPEWRITER_ATTR));
    assert!(html.contains(r#"class="fd-text-reveal__display" aria-hidden="true""#));
}

#[test]
fn scramble_render_carries_duration_attr_value() {
    let html = render(&scramble("hi", Some(500)));
    assert!(html.contains(&format!(r#"{SCRAMBLE_ATTR}="500""#)));
}
