//! styled Splitter（イシュー #826）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs`/`slider`（`src/slider.rs` 内
//! インラインテスト）の golden fixture テストの前例に倣い、`stylesheet()`
//! が返す CSS 全文をバイト単位で固定する。出力順（base → variants →
//! states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::splitter;

const SPLITTER_GOLDEN_CSS: &str = r#"[data-scope="splitter"][data-part="root"] {
  display: flex;
  align-items: stretch;
  width: 100%;
}

[data-scope="splitter"][data-part="panel"] {
  flex-basis: var(--fandhe-splitter-size, auto);
  flex-grow: 0;
  flex-shrink: 1;
  overflow: hidden;
}

[data-scope="splitter"][data-part="resize-trigger"] {
  flex: 0 0 var(--fandhe-splitter-trigger-size, 0.25rem);
  background: var(--fandhe-color-border);
  cursor: col-resize;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fandhe-radius-full, 999px);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="splitter"][data-part="resize-trigger"] {
  transition-property: background, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="splitter"][data-part="resize-trigger-indicator"] {
  width: 0.75rem;
  height: 0.75rem;
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-full, 999px);
  box-shadow: var(--fandhe-shadow-sm);
  pointer-events: none;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-xs {
  --fandhe-splitter-trigger-size: 0.0625rem;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-sm {
  --fandhe-splitter-trigger-size: 0.125rem;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-md {
  --fandhe-splitter-trigger-size: 0.25rem;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-lg {
  --fandhe-splitter-trigger-size: 0.375rem;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-xl {
  --fandhe-splitter-trigger-size: 0.5rem;
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="splitter"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="splitter"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="splitter"][data-part="resize-trigger"][data-orientation="vertical"] {
  cursor: row-resize;
}

[data-scope="splitter"][data-part="resize-trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="splitter"][data-part="resize-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

@media (hover: hover) {
  [data-scope="splitter"][data-part="resize-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn splitter_stylesheet_matches_golden_fixture() {
    assert_eq!(splitter::stylesheet(), SPLITTER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(splitter::stylesheet(), splitter::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = splitter::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_never_references_external_resources() {
    let css = splitter::stylesheet();
    assert!(!css.contains("url("));
}
