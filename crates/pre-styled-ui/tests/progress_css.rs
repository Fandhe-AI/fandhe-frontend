//! styled Progress（linear + circle 対応、イシュー #763/#1564/#1688）の決定的
//! CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/drawer_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → size variants → variant → color-palette → states → `@keyframes` →
//! `@media (prefers-reduced-motion)`）が崩れた場合や意図しない宣言の追加・
//! 欠落があった場合に、この golden テストが即座に検知する。イシュー #1564 で
//! linear（Track/Range）の styled CSS・`ProgressVariant`/`ColorPalette` 軸・
//! indeterminate/vertical の状態別 CSS・reduced-motion 対応を追加した。イシュー
//! #1688 で circle-range の indeterminate 状態に固定弧（`stroke-dasharray`）
//! を追加し、塗り色の完全リングが回転するだけで `complete` と区別できない
//! 問題を是正した（circle-track は引き続き indeterminate 専用規則を持たない）。

use fandhe_frontend_pre_styled_ui::progress;

const PROGRESS_GOLDEN_CSS: &str = r#"[data-scope="progress"][data-part="root"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="progress"][data-part="label"] {
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="progress"][data-part="value-text"] {
  color: var(--fandhe-color-fg-muted);
  font-variant-numeric: tabular-nums;
  margin-left: auto;
}

[data-scope="progress"][data-part="track"] {
  position: relative;
  overflow: hidden;
  flex-basis: 100%;
  width: 100%;
  height: var(--fandhe-progress-track-height, 0.625rem);
  border-radius: var(--fandhe-radius-full, 999px);
  background: var(--fandhe-progress-track-bg, var(--fandhe-color-bg-muted));
  box-shadow: var(--fandhe-progress-track-shadow, inset 0 0 0 1px var(--fandhe-color-border-muted));
}

[data-scope="progress"][data-part="range"] {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: var(--fandhe-progress-percent, 0%);
  border-radius: inherit;
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  transition-property: width, height;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="progress"][data-part="circle"] {
  --size: var(--fandhe-progress-size, 3rem);
  --thickness: var(--fandhe-progress-thickness, 0.25rem);
  transform-origin: center;
}

[data-scope="progress"][data-part="circle-track"] {
  stroke: var(--fandhe-color-border);
}

[data-scope="progress"][data-part="circle-range"] {
  stroke: var(--fandhe-palette, var(--fandhe-color-accent));
  stroke-linecap: round;
  transition-property: stroke-dashoffset;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="progress"][data-part="root"].fd-progress--size-xs {
  --fandhe-progress-size: 1rem;
  --fandhe-progress-thickness: 0.15rem;
  --fandhe-progress-track-height: 0.375rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-sm {
  --fandhe-progress-size: 2rem;
  --fandhe-progress-thickness: 0.2rem;
  --fandhe-progress-track-height: 0.5rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-md {
  --fandhe-progress-size: 3rem;
  --fandhe-progress-thickness: 0.25rem;
  --fandhe-progress-track-height: 0.625rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-lg {
  --fandhe-progress-size: 4rem;
  --fandhe-progress-thickness: 0.3rem;
  --fandhe-progress-track-height: 0.75rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-xl {
  --fandhe-progress-size: 5rem;
  --fandhe-progress-thickness: 0.35rem;
  --fandhe-progress-track-height: 1rem;
}

[data-scope="progress"][data-part="root"].fd-progress--variant-outline {
  --fandhe-progress-track-bg: var(--fandhe-color-bg-muted);
  --fandhe-progress-track-shadow: inset 0 0 0 1px var(--fandhe-color-border-muted);
}

[data-scope="progress"][data-part="root"].fd-progress--variant-subtle {
  --fandhe-progress-track-bg: var(--fandhe-palette-subtle);
  --fandhe-progress-track-shadow: none;
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="progress"][data-part="root"].fd-progress--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {
  animation: fd-progress-circle-spin 1s linear infinite;
}

[data-scope="progress"][data-part="circle-range"][data-state="indeterminate"] {
  --fandhe-progress-circumference: calc(2 * 3.14159265 * var(--radius));
  stroke-dasharray: calc(var(--fandhe-progress-circumference) * 0.25) var(--fandhe-progress-circumference);
}

[data-scope="progress"][data-part="range"][data-state="indeterminate"] {
  width: 40%;
  animation: fd-progress-range-slide 1.5s var(--fandhe-motion-easing-standard) infinite;
}

[data-scope="progress"][data-part="track"][data-orientation="vertical"] {
  width: var(--fandhe-progress-track-height, 0.625rem);
  height: var(--fandhe-progress-track-length, 12rem);
  flex-basis: auto;
}

[data-scope="progress"][data-part="range"][data-orientation="vertical"] {
  top: auto;
  bottom: 0;
  width: 100%;
  height: var(--fandhe-progress-percent, 0%);
}

[data-scope="progress"][data-part="range"][data-state="indeterminate"][data-orientation="vertical"] {
  width: 100%;
  height: 40%;
  animation: fd-progress-range-slide-vertical 1.5s var(--fandhe-motion-easing-standard) infinite;
}

@keyframes fd-progress-circle-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
@keyframes fd-progress-range-slide {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(250%);
  }
}
@keyframes fd-progress-range-slide-vertical {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(-250%);
  }
}
@media (prefers-reduced-motion: reduce) {
  [data-scope="progress"][data-part="circle"][data-state="indeterminate"] {
    animation: none;
  }

  [data-scope="progress"][data-part="range"][data-state="indeterminate"] {
    animation: none;
  }
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(progress::stylesheet(), PROGRESS_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic() {
    assert_eq!(progress::stylesheet(), progress::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = progress::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn circle_selectors_match_headless_anatomy_data_parts() {
    let css = progress::stylesheet();
    for part in [
        "root",
        "label",
        "value-text",
        "track",
        "range",
        "circle",
        "circle-track",
        "circle-range",
    ] {
        assert!(
            css.contains(&format!(r#"[data-scope="progress"][data-part="{part}"]"#)),
            "missing selector for data-part={part}: css={css}"
        );
    }
}

#[test]
fn size_variant_declares_all_five_sizes_on_root() {
    let css = progress::stylesheet();
    for (size, geo_size, thickness, track_height) in [
        ("xs", "1rem", "0.15rem", "0.375rem"),
        ("sm", "2rem", "0.2rem", "0.5rem"),
        ("md", "3rem", "0.25rem", "0.625rem"),
        ("lg", "4rem", "0.3rem", "0.75rem"),
        ("xl", "5rem", "0.35rem", "1rem"),
    ] {
        assert!(css.contains(&format!(
            r#"[data-scope="progress"][data-part="root"].fd-progress--size-{size}"#
        )));
        assert!(css.contains(&format!("--fandhe-progress-size: {geo_size};")));
        assert!(css.contains(&format!("--fandhe-progress-thickness: {thickness};")));
        assert!(css.contains(&format!("--fandhe-progress-track-height: {track_height};")));
    }
}

#[test]
fn variant_axis_declares_track_custom_properties_on_root_only() {
    // PR #1835 Cursor Bugbot 指摘（High）の回帰: headless track は variant
    // クラスを一切受け取らないため `[data-part="track"].fd-progress--variant-*`
    // というセレクタは構造的に一致しない。修正後は root セレクタへ
    // `--fandhe-progress-track-bg`/`--fandhe-progress-track-shadow` を
    // custom property として登録し、track の base 規則（既に別テストで
    // 固定済み）が継承経由で参照する。
    let css = progress::stylesheet();
    assert!(
        css.contains(r#"[data-scope="progress"][data-part="root"].fd-progress--variant-outline"#)
    );
    assert!(
        css.contains(r#"[data-scope="progress"][data-part="root"].fd-progress--variant-subtle"#)
    );
    assert!(css.contains("--fandhe-progress-track-bg: var(--fandhe-color-bg-muted);"));
    assert!(css.contains(
        "--fandhe-progress-track-shadow: inset 0 0 0 1px var(--fandhe-color-border-muted);"
    ));
    assert!(css.contains("--fandhe-progress-track-bg: var(--fandhe-palette-subtle);"));
    assert!(!css.contains(r#"[data-scope="progress"][data-part="track"].fd-progress--variant"#));
    assert!(
        !css.contains(r#"[data-scope="progress"][data-part="circle-track"].fd-progress--variant"#)
    );
}

#[test]
fn subtle_variant_disables_outline_track_shadow() {
    // イシュー #1564/PR #1835 codex-review P1・Cursor Bugbot 指摘の回帰:
    // track の base 規則は `--fandhe-progress-track-shadow` 未定義時に
    // Outline 相当の inset shadow（1px 枠線）へフォールバックする
    // （`stylesheet_matches_golden_css_byte_for_byte` で固定済みの
    // `box-shadow: var(--fandhe-progress-track-shadow, inset 0 0 0 1px
    // var(--fandhe-color-border-muted));` 参照）。Subtle variant は
    // 背景色（`--fandhe-progress-track-bg`）だけでなく、この枠線を
    // 明示的に打ち消す `--fandhe-progress-track-shadow: none` も root へ
    // 登録しなければ Outline 専用の枠線が残ってしまう。
    let css = progress::stylesheet();
    let subtle_variant_rule = css
        .split("\n\n")
        .find(|rule| {
            rule.starts_with(
                r#"[data-scope="progress"][data-part="root"].fd-progress--variant-subtle"#,
            )
        })
        .expect("fd-progress--variant-subtle ルールが stylesheet に存在する");
    assert!(subtle_variant_rule.contains("--fandhe-progress-track-shadow: none;"));
}

#[test]
fn color_palette_axis_declares_all_six_palettes_on_root() {
    let css = progress::stylesheet();
    for palette in ["accent", "info", "success", "warning", "danger", "neutral"] {
        assert!(css.contains(&format!(
            r#"[data-scope="progress"][data-part="root"].fd-progress--color-palette-{palette}"#
        )));
    }
}

// イシュー #1688: 従来は circle-range が indeterminate 専用規則を一切
// 持たない（circle-track と同じ「対象外」扱い）ことを固定していたが、
// headless circle_range は indeterminate 時に stroke-dasharray を出力
// しない契約のため、styled 層が何も与えないと塗り色の完全リングが
// 回転するだけで complete と視覚的に区別できない不具合があった
// （モジュール冒頭 rustdoc「indeterminate アニメーション」節参照）。
// 是正後は circle-range にも固定弧の indeterminate 規則を追加したため、
// 「circle-range は対象外」の固定を「circle-track のみ対象外」へ反転する。
#[test]
fn indeterminate_state_targets_circle_range_and_range_not_circle_track() {
    let css = progress::stylesheet();
    assert!(css
        .contains(r#"[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {"#));
    assert!(css.contains(
        r#"[data-scope="progress"][data-part="circle-range"][data-state="indeterminate"] {"#
    ));
    assert!(
        css.contains(r#"[data-scope="progress"][data-part="range"][data-state="indeterminate"] {"#)
    );
    assert!(!css.contains(
        r#"[data-scope="progress"][data-part="circle-track"][data-state="indeterminate"]"#
    ));
}

#[test]
fn vertical_orientation_states_target_track_and_range() {
    let css = progress::stylesheet();
    assert!(css
        .contains(r#"[data-scope="progress"][data-part="track"][data-orientation="vertical"] {"#));
    assert!(css
        .contains(r#"[data-scope="progress"][data-part="range"][data-orientation="vertical"] {"#));
    assert!(css.contains(
        r#"[data-scope="progress"][data-part="range"][data-state="indeterminate"][data-orientation="vertical"] {"#
    ));
}

#[test]
fn reduced_motion_media_query_stops_both_indeterminate_animations() {
    let css = progress::stylesheet();
    assert!(css.contains("@media (prefers-reduced-motion: reduce) {"));
    assert!(css.matches("animation: none;").count() == 2);
}
