//! イシュー #2382「共通 `@keyframes` ライブラリ（opt-in）」の golden・契約
//! テスト。`motion` feature 配下のみコンパイルする（feature off では空
//! テストバイナリ、`tests/motion_zero_cost.rs` の「無効時ゼロコスト」契約
//! と両立する）。
#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::motion::{
    self, BOUNCE_KEYFRAMES_NAME, FADE_IN_KEYFRAMES_NAME, FADE_OUT_KEYFRAMES_NAME,
    SHAKE_KEYFRAMES_NAME, SLIDE_FROM_BOTTOM_KEYFRAMES_NAME, SLIDE_FROM_LEFT_KEYFRAMES_NAME,
    SLIDE_FROM_RIGHT_KEYFRAMES_NAME, SLIDE_FROM_TOP_KEYFRAMES_NAME, ZOOM_IN_KEYFRAMES_NAME,
    ZOOM_OUT_KEYFRAMES_NAME,
};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// `motion::KEYFRAMES_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_keyframes_css` の実出力を貼り付ける。
const EXPECTED_KEYFRAMES_CSS: &str = "@keyframes fd-motion-fade-in {\n  from {\n    opacity: 0;\n  }\n  to {\n    opacity: 1;\n  }\n}\n@keyframes fd-motion-fade-out {\n  from {\n    opacity: 1;\n  }\n  to {\n    opacity: 0;\n  }\n}\n@keyframes fd-motion-zoom-in {\n  from {\n    opacity: 0;\n    scale: 0.95;\n  }\n  to {\n    opacity: 1;\n    scale: 1;\n  }\n}\n@keyframes fd-motion-zoom-out {\n  from {\n    opacity: 1;\n    scale: 1;\n  }\n  to {\n    opacity: 0;\n    scale: 0.95;\n  }\n}\n@keyframes fd-motion-slide-from-top {\n  from {\n    opacity: 0;\n    translate: 0 calc(-1 * var(--fandhe-motion-slide-offset, 0.5rem));\n  }\n  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n}\n@keyframes fd-motion-slide-from-bottom {\n  from {\n    opacity: 0;\n    translate: 0 var(--fandhe-motion-slide-offset, 0.5rem);\n  }\n  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n}\n@keyframes fd-motion-slide-from-left {\n  from {\n    opacity: 0;\n    translate: calc(-1 * var(--fandhe-motion-slide-offset, 0.5rem)) 0;\n  }\n  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n}\n@keyframes fd-motion-slide-from-right {\n  from {\n    opacity: 0;\n    translate: var(--fandhe-motion-slide-offset, 0.5rem) 0;\n  }\n  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n}\n@keyframes fd-motion-bounce {\n  0%, 100% {\n    translate: 0 0;\n  }\n  50% {\n    translate: 0 calc(-1 * var(--fandhe-motion-bounce-height, 0.5rem));\n  }\n}\n@keyframes fd-motion-shake {\n  0%, 100% {\n    translate: 0 0;\n  }\n  25% {\n    translate: calc(-1 * var(--fandhe-motion-shake-distance, 0.25rem)) 0;\n  }\n  75% {\n    translate: var(--fandhe-motion-shake-distance, 0.25rem) 0;\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  @keyframes fd-motion-zoom-in {\n    from {\n      opacity: 0;\n    }\n    to {\n      opacity: 1;\n    }\n  }\n  @keyframes fd-motion-zoom-out {\n    from {\n      opacity: 1;\n    }\n    to {\n      opacity: 0;\n    }\n  }\n  @keyframes fd-motion-slide-from-top {\n    from {\n      opacity: 0;\n    }\n    to {\n      opacity: 1;\n    }\n  }\n  @keyframes fd-motion-slide-from-bottom {\n    from {\n      opacity: 0;\n    }\n    to {\n      opacity: 1;\n    }\n  }\n  @keyframes fd-motion-slide-from-left {\n    from {\n      opacity: 0;\n    }\n    to {\n      opacity: 1;\n    }\n  }\n  @keyframes fd-motion-slide-from-right {\n    from {\n      opacity: 0;\n    }\n    to {\n      opacity: 1;\n    }\n  }\n  @keyframes fd-motion-bounce {\n    from, to {\n      translate: 0 0;\n    }\n  }\n  @keyframes fd-motion-shake {\n    from, to {\n      translate: 0 0;\n    }\n  }\n}\n";

#[test]
fn keyframes_css_matches_golden() {
    assert_eq!(motion::KEYFRAMES_CSS, EXPECTED_KEYFRAMES_CSS);
}

#[test]
fn every_keyframes_name_const_is_defined_in_css() {
    let names = [
        FADE_IN_KEYFRAMES_NAME,
        FADE_OUT_KEYFRAMES_NAME,
        ZOOM_IN_KEYFRAMES_NAME,
        ZOOM_OUT_KEYFRAMES_NAME,
        SLIDE_FROM_TOP_KEYFRAMES_NAME,
        SLIDE_FROM_BOTTOM_KEYFRAMES_NAME,
        SLIDE_FROM_LEFT_KEYFRAMES_NAME,
        SLIDE_FROM_RIGHT_KEYFRAMES_NAME,
        BOUNCE_KEYFRAMES_NAME,
        SHAKE_KEYFRAMES_NAME,
    ];
    for name in names {
        assert!(name.starts_with("fd-motion-"), "name={name}");
        assert!(
            motion::KEYFRAMES_CSS.contains(&format!("@keyframes {name} {{")),
            "@keyframes {name} が KEYFRAMES_CSS に見つからない"
        );
    }
}

#[test]
fn reduced_motion_block_redefines_motion_bearing_keyframes() {
    let (before, after) = motion::KEYFRAMES_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    // fade-in/out は動きを含まないため reduced-motion ブロックで再定義しない。
    assert_eq!(after.matches(FADE_IN_KEYFRAMES_NAME).count(), 0);
    assert_eq!(after.matches(FADE_OUT_KEYFRAMES_NAME).count(), 0);

    // 残り 8 名はいずれも再定義され、`translate`/`scale` の非ゼロ値
    // （動きの元）を持たない（静止 or opacity のみへの縮退）。
    let motion_bearing = [
        ZOOM_IN_KEYFRAMES_NAME,
        ZOOM_OUT_KEYFRAMES_NAME,
        SLIDE_FROM_TOP_KEYFRAMES_NAME,
        SLIDE_FROM_BOTTOM_KEYFRAMES_NAME,
        SLIDE_FROM_LEFT_KEYFRAMES_NAME,
        SLIDE_FROM_RIGHT_KEYFRAMES_NAME,
        BOUNCE_KEYFRAMES_NAME,
        SHAKE_KEYFRAMES_NAME,
    ];
    for name in motion_bearing {
        let marker = format!("@keyframes {name} {{");
        assert_eq!(
            after.matches(&marker).count(),
            1,
            "reduced-motion ブロック内に @keyframes {name} の再定義が無い"
        );
    }
    assert!(!after.contains("var(--fandhe-motion-"));
    assert!(!after.contains("0.95"));
    assert!(!before.is_empty());
}

#[test]
fn to_css_with_keyframes_is_to_css_plus_library() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_keyframes();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + motion::KEYFRAMES_CSS.len());
    assert_eq!(&extended[base.len()..], motion::KEYFRAMES_CSS);
}

#[test]
fn keyframes_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(motion::KEYFRAMES_CSS).is_ok());
    assert!(!motion::KEYFRAMES_CSS.contains('<'));
}
