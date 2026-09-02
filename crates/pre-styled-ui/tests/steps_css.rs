//! styled Steps（イシュー #752）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/rating_group_css.rs`/`slider_css.rs`（存在すれば）
//! の golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden テスト
//! が即座に検知する。

use fandhe_frontend_pre_styled_ui::steps;

const STEPS_GOLDEN_CSS: &str = r#"[data-scope="steps"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-4);
}

[data-scope="steps"][data-part="list"] {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--fandhe-space-2);
  list-style: none;
  margin: 0;
  padding: 0;
}

[data-scope="steps"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  flex: 1;
}

[data-scope="steps"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-3);
  background: none;
  border: none;
  border-radius: var(--fandhe-radius-md);
  cursor: pointer;
  font: inherit;
  font-size: var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  color: inherit;
  padding: 0;
  text-align: start;
}

[data-scope="steps"][data-part="trigger"] {
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="steps"][data-part="trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="steps"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fandhe-steps-indicator-size, 2rem);
  height: var(--fandhe-steps-indicator-size, 2rem);
  border-radius: 999px;
  border: 2px solid var(--fandhe-color-border);
  color: var(--fandhe-color-fg);
  flex-shrink: 0;
}

[data-scope="steps"][data-part="separator"] {
  flex: 1;
  height: 2px;
  background: var(--fandhe-color-border);
}

[data-scope="steps"][data-part="content"] {
  color: var(--fandhe-color-fg);
}

[data-scope="steps"][data-part="completed-content"] {
  color: var(--fandhe-color-fg);
}

[data-scope="steps"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  cursor: pointer;
  font: inherit;
  font-size: var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm));
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope="steps"][data-part="prev-trigger"] {
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="steps"][data-part="prev-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="steps"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  cursor: pointer;
  font: inherit;
  font-size: var(--fandhe-steps-font-size, var(--fandhe-font-font-size-sm));
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
}

[data-scope="steps"][data-part="next-trigger"] {
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="steps"][data-part="next-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="steps"][data-part="root"].fd-steps--size-xs {
  --fandhe-steps-indicator-size: 1rem;
  --fandhe-steps-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="steps"][data-part="root"].fd-steps--size-sm {
  --fandhe-steps-indicator-size: 1.5rem;
  --fandhe-steps-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="steps"][data-part="root"].fd-steps--size-md {
  --fandhe-steps-indicator-size: 2rem;
  --fandhe-steps-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="steps"][data-part="root"].fd-steps--size-lg {
  --fandhe-steps-indicator-size: 2.5rem;
  --fandhe-steps-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="steps"][data-part="root"].fd-steps--size-xl {
  --fandhe-steps-indicator-size: 3rem;
  --fandhe-steps-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="steps"][data-part="root"][data-orientation="vertical"] {
  flex-direction: row;
  align-items: flex-start;
}

[data-scope="steps"][data-part="list"][data-orientation="vertical"] {
  flex-direction: column;
  align-items: stretch;
}

[data-scope="steps"][data-part="item"][data-orientation="vertical"] {
  flex-direction: column;
  align-items: flex-start;
  min-height: var(--fandhe-steps-connector-min-height, 2.5rem);
}

[data-scope="steps"][data-part="item"]:last-child {
  flex: none;
  min-height: auto;
}

[data-scope="steps"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="steps"][data-part="indicator"][data-state="current"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="steps"][data-part="indicator"][data-state="complete"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-color-bg);
}

[data-scope="steps"][data-part="separator"][data-orientation="vertical"] {
  width: 2px;
  height: auto;
  align-self: stretch;
  margin-left: calc(var(--fandhe-steps-indicator-size, 2rem) / 2 - 1px);
}

[data-scope="steps"][data-part="separator"][data-complete] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="steps"][data-part="content"][data-state="closed"] {
  display: none;
}

[data-scope="steps"][data-part="content"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="steps"][data-part="completed-content"][data-state="closed"] {
  display: none;
}

[data-scope="steps"][data-part="completed-content"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="steps"][data-part="prev-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="steps"][data-part="next-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="steps"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="steps"][data-part="prev-trigger"]:hover:not([data-disabled]):not([disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="steps"][data-part="next-trigger"]:hover:not([data-disabled]):not([disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn steps_stylesheet_matches_golden_fixture() {
    assert_eq!(steps::stylesheet(), STEPS_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(steps::stylesheet(), steps::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = steps::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_never_references_external_resources() {
    let css = steps::stylesheet();
    assert!(!css.contains("url("));
}
