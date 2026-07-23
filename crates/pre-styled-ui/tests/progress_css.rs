//! styled Progress（circular 対応、イシュー #763）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/drawer_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states → `@keyframes`）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。circle 系
//! （Circle/CircleTrack/CircleRange, SVG）のみを対象とする（linear は本
//! イシューのスコープ外、`crates/pre-styled-ui/src/progress.rs` rustdoc 参照）。

use fandhe_frontend_pre_styled_ui::progress;

const PROGRESS_GOLDEN_CSS: &str = r#"[data-scope="progress"][data-part="label"] {
  color: var(--fandhe-color-fg);
}

[data-scope="progress"][data-part="value-text"] {
  color: var(--fandhe-color-fg-muted);
  font-variant-numeric: tabular-nums;
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
  transition: stroke-dashoffset 0.2s ease;
}

[data-scope="progress"][data-part="root"].fd-progress--size-sm {
  --fandhe-progress-size: 2rem;
  --fandhe-progress-thickness: 0.2rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-md {
  --fandhe-progress-size: 3rem;
  --fandhe-progress-thickness: 0.25rem;
}

[data-scope="progress"][data-part="root"].fd-progress--size-lg {
  --fandhe-progress-size: 4rem;
  --fandhe-progress-thickness: 0.3rem;
}

[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {
  animation: fd-progress-circle-spin 1s linear infinite;
}

@keyframes fd-progress-circle-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
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
    for part in ["circle", "circle-track", "circle-range"] {
        assert!(
            css.contains(&format!(r#"[data-scope="progress"][data-part="{part}"]"#)),
            "missing selector for data-part={part}: css={css}"
        );
    }
}

#[test]
fn size_variant_declares_all_three_sizes_on_root() {
    let css = progress::stylesheet();
    for (size, geo_size, thickness) in [
        ("sm", "2rem", "0.2rem"),
        ("md", "3rem", "0.25rem"),
        ("lg", "4rem", "0.3rem"),
    ] {
        assert!(css.contains(&format!(
            r#"[data-scope="progress"][data-part="root"].fd-progress--size-{size}"#
        )));
        assert!(css.contains(&format!("--fandhe-progress-size: {geo_size};")));
        assert!(css.contains(&format!("--fandhe-progress-thickness: {thickness};")));
    }
}

#[test]
fn indeterminate_state_targets_circle_slot_only() {
    let css = progress::stylesheet();
    assert!(css
        .contains(r#"[data-scope="progress"][data-part="circle"][data-state="indeterminate"] {"#));
    assert!(!css.contains(
        r#"[data-scope="progress"][data-part="circle-track"][data-state="indeterminate"]"#
    ));
    assert!(!css.contains(
        r#"[data-scope="progress"][data-part="circle-range"][data-state="indeterminate"]"#
    ));
}
