//! イシュー #2539「stat の数値カウントアップ（AnimateNumber 相当）」の
//! golden・契約テスト。`motion` feature 配下のみコンパイルする（feature
//! off では空テストバイナリ、`tests/motion_zero_cost.rs` の「無効時
//! ゼロコスト」契約と両立する。`tests/button_motion_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_core::{render, text};
use fandhe_frontend_pre_styled_ui::stat_motion::{count_up_value_text, STAT_MOTION_CSS};
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `stat_motion::STAT_MOTION_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test stat_motion_css` の実出力を貼り付ける。
const EXPECTED_STAT_MOTION_CSS: &str = "[data-scope=\"stat\"][data-part=\"value-text\"][data-fandhe-count-up] {\n  font-variant-numeric: tabular-nums;\n}\n";

#[test]
fn stat_motion_css_matches_golden() {
    assert_eq!(STAT_MOTION_CSS, EXPECTED_STAT_MOTION_CSS);
}

#[test]
fn css_has_no_forbidden_angle_bracket() {
    assert!(!STAT_MOTION_CSS.contains('<'));
}

#[test]
fn to_css_with_stat_motion_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_stat_motion();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + STAT_MOTION_CSS.len());
    assert_eq!(&extended[base.len()..], STAT_MOTION_CSS);
}

#[test]
fn count_up_value_text_renders_opt_in_attr_and_anatomy() {
    let node = count_up_value_text(vec![], vec![text("1,234")]);
    let html = render(&node);
    assert!(html.contains(r#"data-scope="stat""#));
    assert!(html.contains(r#"data-part="value-text""#));
    assert!(html.contains("data-fandhe-count-up"));
    assert!(html.contains("1,234"));
}

#[test]
fn count_up_value_text_passes_through_extra_attrs() {
    let node = count_up_value_text(
        vec![("data-fandhe-count-up-trigger", "in-view")],
        vec![text("0")],
    );
    let html = render(&node);
    assert!(html.contains(r#"data-fandhe-count-up-trigger="in-view""#));
}

#[test]
fn count_up_value_text_escapes_child_text_xss() {
    let node = count_up_value_text(vec![], vec![text("<script>alert(1)</script>")]);
    let html = render(&node);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}
