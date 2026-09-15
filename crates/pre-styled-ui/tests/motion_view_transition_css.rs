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
const EXPECTED_CSS: &str = "@keyframes fd-motion-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@keyframes fd-motion-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
@keyframes fd-view-transition-slide-out-to-left {
  to {
    transform: translateX(-100%);
  }
}
@keyframes fd-view-transition-slide-in-from-right {
  from {
    transform: translateX(100%);
  }
}
@keyframes fd-view-transition-wipe-reveal {
  from {
    clip-path: inset(0 100% 0 0);
  }
  to {
    clip-path: inset(0 0 0 0);
  }
}
@keyframes fd-view-transition-iris-reveal {
  from {
    clip-path: circle(0% at 50% 50%);
  }
  to {
    clip-path: circle(75% at 50% 50%);
  }
}
@keyframes fd-view-transition-doors-reveal {
  from {
    clip-path: inset(0 50%);
  }
  to {
    clip-path: inset(0 0);
  }
}
@keyframes fd-view-transition-shutter-reveal {
  from {
    clip-path: inset(50% 0);
  }
  to {
    clip-path: inset(0 0);
  }
}
@keyframes fd-view-transition-blinds-reveal {
  from {
    mask-size: 100% 0vh;
  }
  to {
    mask-size: 100% 13vh;
  }
}
@keyframes fd-view-transition-strips-reveal {
  from {
    mask-size: 0vw 12.5vh;
  }
  to {
    mask-size: 100vw 12.5vh;
  }
}
@keyframes fd-view-transition-pixels-reveal {
  from {
    mask-size: 0vw 0vh;
  }
  to {
    mask-size: 25vw 25vh;
  }
}
@keyframes fd-view-transition-mask-wipe-reveal {
  from {
    mask-position: 200% 0;
  }
  to {
    mask-position: 0 0;
  }
}
@keyframes fd-view-transition-mask-radial-reveal {
  from {
    mask-size: 0 0;
  }
  to {
    mask-size: 300vmax 300vmax;
  }
}
:root[data-fandhe-view-transition=\"fade\"]::view-transition-old(root) {
  animation: fd-motion-fade-out var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: plus-lighter;
}
:root[data-fandhe-view-transition=\"fade\"]::view-transition-new(root) {
  animation: fd-motion-fade-in var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: plus-lighter;
}
:root[data-fandhe-view-transition=\"slide\"]::view-transition-old(root) {
  animation: fd-view-transition-slide-out-to-left var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"slide\"]::view-transition-new(root) {
  animation: fd-view-transition-slide-in-from-right var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"wipe\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"wipe\"]::view-transition-new(root) {
  animation: fd-view-transition-wipe-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"iris\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"iris\"]::view-transition-new(root) {
  animation: fd-view-transition-iris-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"doors\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"doors\"]::view-transition-new(root) {
  animation: fd-view-transition-doors-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"shutter\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"shutter\"]::view-transition-new(root) {
  animation: fd-view-transition-shutter-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"blinds\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"blinds\"]::view-transition-new(root) {
  animation: fd-view-transition-blinds-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
  mask-image: linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0);
  mask-repeat: no-repeat;
  mask-position: 0 0vh, 0 12.5vh, 0 25vh, 0 37.5vh, 0 50vh, 0 62.5vh, 0 75vh, 0 87.5vh;
  mask-size: 100% 0vh;
}
:root[data-fandhe-view-transition=\"strips\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"strips\"]::view-transition-new(root) {
  animation: fd-view-transition-strips-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
  mask-image: linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0);
  mask-repeat: no-repeat;
  mask-position: 0 0vh, 100% 12.5vh, 0 25vh, 100% 37.5vh, 0 50vh, 100% 62.5vh, 0 75vh, 100% 87.5vh;
  mask-size: 0vw 12.5vh;
}
:root[data-fandhe-view-transition=\"pixels\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"pixels\"]::view-transition-new(root) {
  animation: fd-view-transition-pixels-reveal var(--fandhe-motion-duration-slow, 300ms) steps(6, end) both;
  mix-blend-mode: normal;
  mask-image: linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0), linear-gradient(#000 0 0);
  mask-repeat: no-repeat;
  mask-position: 0vw 0vh, 25vw 0vh, 50vw 0vh, 75vw 0vh, 0vw 25vh, 25vw 25vh, 50vw 25vh, 75vw 25vh, 0vw 50vh, 25vw 50vh, 50vw 50vh, 75vw 50vh, 0vw 75vh, 25vw 75vh, 50vw 75vh, 75vw 75vh;
  mask-size: 0vw 0vh;
}
:root[data-fandhe-view-transition=\"mask-wipe\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"mask-wipe\"]::view-transition-new(root) {
  animation: fd-view-transition-mask-wipe-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
  mask-image: linear-gradient(to right, #000 75%, transparent);
  mask-repeat: no-repeat;
  mask-size: 200% 100%;
}
:root[data-fandhe-view-transition=\"mask-radial\"]::view-transition-old(root) {
  animation: none;
  mix-blend-mode: normal;
}
:root[data-fandhe-view-transition=\"mask-radial\"]::view-transition-new(root) {
  animation: fd-view-transition-mask-radial-reveal var(--fandhe-motion-duration-slow, 300ms) both;
  mix-blend-mode: normal;
  mask-image: radial-gradient(circle, #000 40%, transparent 70%);
  mask-position: center;
  mask-repeat: no-repeat;
}
@media (prefers-reduced-motion: reduce) {
  :root[data-fandhe-view-transition=\"fade\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"fade\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"slide\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"slide\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"wipe\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"wipe\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"iris\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"iris\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"doors\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"doors\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"shutter\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"shutter\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"blinds\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"blinds\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"strips\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"strips\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"pixels\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"pixels\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"mask-wipe\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"mask-wipe\"]::view-transition-new(root),
  :root[data-fandhe-view-transition=\"mask-radial\"]::view-transition-old(root),
  :root[data-fandhe-view-transition=\"mask-radial\"]::view-transition-new(root) {
    animation: revert;
    mask-image: none;
  }
}
";

#[test]
fn view_transition_presets_css_matches_golden() {
    assert_eq!(VIEW_TRANSITION_PRESETS_CSS, EXPECTED_CSS);
}

#[test]
fn attr_name_matches_literal_contract() {
    assert_eq!(VIEW_TRANSITION_PRESET_ATTR, "data-fandhe-view-transition");
}

/// 全プリセット名（イシュー #2516 の fade/slide/wipe + イシュー #2537 の
/// iris/doors/shutter/blinds/strips/pixels/mask-wipe/mask-radial、計 11）。
const ALL_PRESETS: [&str; 11] = [
    "fade",
    "slide",
    "wipe",
    "iris",
    "doors",
    "shutter",
    "blinds",
    "strips",
    "pixels",
    "mask-wipe",
    "mask-radial",
];

#[test]
fn every_preset_has_old_and_new_root_rules() {
    for preset in ALL_PRESETS {
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

/// reduced-motion ブロックが全プリセット（[`ALL_PRESETS`] の old/new、計 22
/// セレクタ）を `animation: revert;` へ再宣言することを固定する（モジュール
/// doc「reduced-motion」節の受け入れ条件）。mask 系プリセットの静的
/// `mask-*` 宣言を確定的にクロスフェードへ戻すための `mask-image: none;`
/// も同一グループ規則へ 1 回だけ追加されていることをあわせて固定する。
#[test]
fn reduced_motion_block_reverts_all_presets() {
    let (_before, after) = VIEW_TRANSITION_PRESETS_CSS
        .split_once("@media (prefers-reduced-motion: reduce) {")
        .expect("reduced-motion ブロックが見つからない");
    for preset in ALL_PRESETS {
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
    // 22 セレクタはグループ化された単一規則（`sel1,\n sel2, ... { animation:
    // revert; mask-image: none; }`）として両宣言をそれぞれ 1 回だけ持つ
    // （CSS のグループ化セレクタは同一宣言ブロックを共有するため、22 回の
    // 重複宣言にはならない）。
    assert_eq!(after.matches("animation: revert;").count(), 1);
    assert_eq!(after.matches("mask-image: none;").count(), 1);
}

/// mask 系 4 プリセット（blinds/strips/pixels/mask-wipe/mask-radial のうち
/// `mask-repeat` を持つもの）の new(root) 規則が `mask-repeat: no-repeat`
/// を持つことを固定する（複数 `mask-image` 層・単層ソフトエッジの双方で
/// UA 既定の repeat による意図しない繰り返しを防ぐ）。
#[test]
fn mask_presets_new_root_has_mask_repeat_no_repeat() {
    for preset in ["blinds", "strips", "pixels", "mask-wipe", "mask-radial"] {
        let selector = format!(
            ":root[{VIEW_TRANSITION_PRESET_ATTR}=\"{preset}\"]::view-transition-new(root) {{"
        );
        let (_before, rule_and_after) = VIEW_TRANSITION_PRESETS_CSS
            .split_once(&selector)
            .unwrap_or_else(|| panic!("preset={preset} の new(root) 規則が見つからない"));
        let rule_body = rule_and_after
            .split_once('}')
            .map(|(body, _)| body)
            .unwrap_or(rule_and_after);
        assert!(
            rule_body.contains("mask-repeat: no-repeat;"),
            "preset={preset} の new(root) に mask-repeat: no-repeat; が無い"
        );
    }
}

/// `pixels` プリセットのみ `steps(` タイミング関数（格子リビールの段階的な
/// 育ち方）を持つことを固定する。
#[test]
fn pixels_preset_uses_steps_timing_function() {
    assert!(VIEW_TRANSITION_PRESETS_CSS.contains("steps(6, end)"));
}

/// `@property`（山括弧を含む構文）を一切含まないことを固定する
/// （モジュール doc「`@property` を使わない理由」節、`push_css` の `<` 拒否
/// と対をなす受け入れ条件）。
#[test]
fn presets_css_has_no_at_property() {
    assert!(!VIEW_TRANSITION_PRESETS_CSS.contains("@property"));
}
