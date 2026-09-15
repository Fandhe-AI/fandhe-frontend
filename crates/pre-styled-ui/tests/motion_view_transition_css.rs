//! イシュー #2516「named view transition CSS プリセット」の golden・契約
//! テスト。`motion` feature 配下のみコンパイルする（feature off では空
//! テストバイナリ、`tests/motion_zero_cost.rs` の「無効時ゼロコスト」契約
//! と両立する）。
#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::view_transition::{
    self, VIEW_TRANSITION_PRESETS_CSS, VIEW_TRANSITION_PRESET_ATTR,
};

/// `view_transition::VIEW_TRANSITION_PRESETS_CSS` の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_view_transition_css` の実出力を貼り付ける。
const EXPECTED_CSS: &str = "@keyframes fd-motion-fade-in {\n  from {\n    opacity: 0;\n  }\n  to {\n    opacity: 1;\n  }\n}\n@keyframes fd-motion-fade-out {\n  from {\n    opacity: 1;\n  }\n  to {\n    opacity: 0;\n  }\n}\n@keyframes fd-view-transition-slide-out-to-left {\n  to {\n    transform: translateX(-100%);\n  }\n}\n@keyframes fd-view-transition-slide-in-from-right {\n  from {\n    transform: translateX(100%);\n  }\n}\n@keyframes fd-view-transition-wipe-reveal {\n  from {\n    clip-path: inset(0 100% 0 0);\n  }\n  to {\n    clip-path: inset(0 0 0 0);\n  }\n}\n:root[data-fandhe-view-transition=\"fade\"]::view-transition-old(root) {\n  animation: fd-motion-fade-out var(--fandhe-motion-duration-slow, 300ms) both;\n}\n:root[data-fandhe-view-transition=\"fade\"]::view-transition-new(root) {\n  animation: fd-motion-fade-in var(--fandhe-motion-duration-slow, 300ms) both;\n}\n:root[data-fandhe-view-transition=\"slide\"]::view-transition-old(root) {\n  animation: fd-view-transition-slide-out-to-left var(--fandhe-motion-duration-slow, 300ms) both;\n  mix-blend-mode: normal;\n}\n:root[data-fandhe-view-transition=\"slide\"]::view-transition-new(root) {\n  animation: fd-view-transition-slide-in-from-right var(--fandhe-motion-duration-slow, 300ms) both;\n  mix-blend-mode: normal;\n}\n:root[data-fandhe-view-transition=\"wipe\"]::view-transition-old(root) {\n  animation: none;\n  mix-blend-mode: normal;\n}\n:root[data-fandhe-view-transition=\"wipe\"]::view-transition-new(root) {\n  animation: fd-view-transition-wipe-reveal var(--fandhe-motion-duration-slow, 300ms) both;\n  mix-blend-mode: normal;\n}\n@media (prefers-reduced-motion: reduce) {\n  :root[data-fandhe-view-transition=\"fade\"]::view-transition-old(root),\n  :root[data-fandhe-view-transition=\"fade\"]::view-transition-new(root),\n  :root[data-fandhe-view-transition=\"slide\"]::view-transition-old(root),\n  :root[data-fandhe-view-transition=\"slide\"]::view-transition-new(root),\n  :root[data-fandhe-view-transition=\"wipe\"]::view-transition-old(root),\n  :root[data-fandhe-view-transition=\"wipe\"]::view-transition-new(root) {\n    animation: revert;\n  }\n}\n";

#[test]
fn view_transition_presets_css_matches_golden() {
    assert_eq!(VIEW_TRANSITION_PRESETS_CSS, EXPECTED_CSS);
}

#[test]
fn attr_name_matches_literal_contract() {
    assert_eq!(VIEW_TRANSITION_PRESET_ATTR, "data-fandhe-view-transition");
}

#[test]
fn every_preset_has_old_and_new_root_rules() {
    for preset in ["fade", "slide", "wipe"] {
        let old = format!(
            ":root[{VIEW_TRANSITION_PRESET_ATTR}=\"{preset}\"]::view-transition-old(root) {{"
        );
        let new = format!(
            ":root[{VIEW_TRANSITION_PRESET_ATTR}=\"{preset}\"]::view-transition-new(root) {{"
        );
        assert!(
            VIEW_TRANSITION_PRESETS_CSS.contains(&old),
            "preset={preset} の old(root) 規則が見つからない"
        );
        assert!(
            VIEW_TRANSITION_PRESETS_CSS.contains(&new),
            "preset={preset} の new(root) 規則が見つからない"
        );
    }
}

#[test]
fn presets_css_has_no_forbidden_angle_bracket() {
    assert!(!VIEW_TRANSITION_PRESETS_CSS.contains('<'));
}

#[test]
fn presets_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(VIEW_TRANSITION_PRESETS_CSS).is_ok());
}

#[test]
fn to_css_with_view_transition_presets_is_to_css_plus_library() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_view_transition_presets();
    assert!(extended.starts_with(&base));
    assert_eq!(
        extended.len(),
        base.len() + view_transition::VIEW_TRANSITION_PRESETS_CSS.len()
    );
    assert_eq!(
        &extended[base.len()..],
        view_transition::VIEW_TRANSITION_PRESETS_CSS
    );
}

/// reduced-motion ブロックが 3 プリセット全て（fade/slide/wipe の
/// old/new、計 6 セレクタ）を `animation: revert;` へ再宣言することを
/// 固定する（モジュール doc「reduced-motion」節の受け入れ条件）。
#[test]
fn reduced_motion_block_reverts_all_presets() {
    let (_before, after) = VIEW_TRANSITION_PRESETS_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    for preset in ["fade", "slide", "wipe"] {
        assert!(
            after.contains(&format!(
                ":root[{VIEW_TRANSITION_PRESET_ATTR}=\"{preset}\"]::view-transition-old(root)"
            )),
            "preset={preset} の old(root) が reduced-motion ブロックに無い"
        );
        assert!(
            after.contains(&format!(
                ":root[{VIEW_TRANSITION_PRESET_ATTR}=\"{preset}\"]::view-transition-new(root)"
            )),
            "preset={preset} の new(root) が reduced-motion ブロックに無い"
        );
    }
    // 6 セレクタはグループ化された単一規則（`sel1,\n sel2, ... { animation:
    // revert; }`）として `animation: revert;` を 1 回だけ持つ（CSS の
    // グループ化セレクタは同一宣言ブロックを共有するため、6 回の重複
    // 宣言にはならない）。
    assert_eq!(after.matches("animation: revert;").count(), 1);
}
