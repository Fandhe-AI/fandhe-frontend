//! styled Timeline（イシュー #769）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/checkbox_card_css.rs` の golden fixture
//! テストの前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。
//! `variant`/`size`/`color-palette` 3 軸すべての組み合わせが決定的に
//! 出力されることも併せて固定する。

use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size, VariantValue};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};

const TIMELINE_GOLDEN_CSS: &str = r#"[data-scope="timeline"][data-part="root"] {
  display: flex;
  flex-direction: column;
  list-style: none;
  margin: 0;
  padding: 0;
}

[data-scope="timeline"][data-part="item"] {
  display: grid;
  grid-template-columns: var(--fandhe-timeline-indicator-size, 1.5rem) 1fr;
  gap: var(--fandhe-space-2);
}

[data-scope="timeline"][data-part="connector"] {
  grid-column: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
}

[data-scope="timeline"][data-part="separator"] {
  flex: 1;
  width: var(--fandhe-timeline-separator-width, 2px);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="timeline"][data-part="indicator"] {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fandhe-timeline-indicator-size, 1.5rem);
  height: var(--fandhe-timeline-indicator-size, 1.5rem);
  border-radius: var(--fandhe-radius-full, 9999px);
  background: var(--fandhe-timeline-indicator-bg, var(--fandhe-palette, var(--fandhe-color-accent)));
  color: var(--fandhe-timeline-indicator-fg, var(--fandhe-palette-fg, var(--fandhe-color-accent-fg)));
  border: var(--fandhe-timeline-indicator-border, none);
}

[data-scope="timeline"][data-part="content"] {
  grid-column: 2;
  padding-bottom: var(--fandhe-space-4);
}

[data-scope="timeline"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-semibold);
  color: var(--fandhe-color-fg);
}

[data-scope="timeline"][data-part="description"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="timeline"][data-part="root"].fd-timeline--variant-solid {
  --fandhe-timeline-indicator-bg: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-timeline-indicator-fg: var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  --fandhe-timeline-indicator-border: none;
}

[data-scope="timeline"][data-part="root"].fd-timeline--variant-subtle {
  --fandhe-timeline-indicator-bg: var(--fandhe-color-bg-subtle);
  --fandhe-timeline-indicator-fg: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-timeline-indicator-border: none;
}

[data-scope="timeline"][data-part="root"].fd-timeline--variant-outline {
  --fandhe-timeline-indicator-bg: var(--fandhe-color-bg);
  --fandhe-timeline-indicator-fg: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-timeline-indicator-border: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="timeline"][data-part="root"].fd-timeline--variant-plain {
  --fandhe-timeline-indicator-bg: transparent;
  --fandhe-timeline-indicator-fg: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-timeline-indicator-border: none;
}

[data-scope="timeline"][data-part="root"].fd-timeline--size-sm {
  --fandhe-timeline-indicator-size: 1.125rem;
  --fandhe-timeline-separator-width: 1.5px;
}

[data-scope="timeline"][data-part="root"].fd-timeline--size-md {
  --fandhe-timeline-indicator-size: 1.5rem;
  --fandhe-timeline-separator-width: 2px;
}

[data-scope="timeline"][data-part="root"].fd-timeline--size-lg {
  --fandhe-timeline-indicator-size: 1.875rem;
  --fandhe-timeline-separator-width: 2.5px;
}

[data-scope="timeline"][data-part="root"].fd-timeline--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="timeline"][data-part="root"].fd-timeline--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="timeline"][data-part="root"].fd-timeline--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="timeline"][data-part="root"].fd-timeline--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="timeline"][data-part="root"].fd-timeline--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}
"#;

#[test]
fn timeline_css_matches_golden_fixture() {
    assert_eq!(timeline::css(), TIMELINE_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(timeline::css(), timeline::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = timeline::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn all_variant_axis_combinations_produce_deterministic_root_classes() {
    use fandhe_frontend_core::render;

    for variant in [
        TimelineVariant::Solid,
        TimelineVariant::Subtle,
        TimelineVariant::Outline,
        TimelineVariant::Plain,
    ] {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            for palette in [
                ColorPalette::Accent,
                ColorPalette::Info,
                ColorPalette::Success,
                ColorPalette::Warning,
                ColorPalette::Danger,
            ] {
                let html_a = render(&timeline::root(variant, size, palette, vec![], vec![]));
                let html_b = render(&timeline::root(variant, size, palette, vec![], vec![]));
                assert_eq!(html_a, html_b);
                assert!(html_a.contains(&format!("fd-timeline--variant-{}", variant.value())));
            }
        }
    }
}
