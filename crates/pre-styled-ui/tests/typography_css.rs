//! タイポグラフィ静的部品 8 種（イシュー #771、Heading / Text / Em / Mark /
//! Blockquote / List。イシュー #995 で Quote / Strong を追加）の決定的
//! CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skeleton_css.rs` の golden fixture テストの
//! 前例に倣い、各部品の `css()` が返す CSS 全文をバイト単位で固定する。
//! variant/size ごとの規則の出力順が崩れた場合や意図しない宣言の追加・欠落
//! があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::{blockquote, em, heading, list, mark, quote, strong, text};

const HEADING_GOLDEN_CSS: &str = r#"[data-scope="heading"][data-part="root"] {
  margin: 0;
  font-weight: var(--fandhe-font-font-weight-semibold);
  letter-spacing: -0.01em;
}

[data-scope="heading"][data-part="root"].fd-heading--size-xs {
  font-size: var(--fandhe-font-font-size-xs);
  line-height: 1.3;
}

[data-scope="heading"][data-part="root"].fd-heading--size-sm {
  font-size: var(--fandhe-font-font-size-sm);
  line-height: 1.25;
}

[data-scope="heading"][data-part="root"].fd-heading--size-md {
  font-size: var(--fandhe-font-font-size-md);
  line-height: 1.3;
}

[data-scope="heading"][data-part="root"].fd-heading--size-lg {
  font-size: var(--fandhe-font-font-size-lg);
  line-height: 1.3;
}

[data-scope="heading"][data-part="root"].fd-heading--size-xl {
  font-size: var(--fandhe-font-font-size-xl);
  line-height: 1.3;
}

[data-scope="heading"][data-part="root"].fd-heading--size-xl2 {
  font-size: var(--fandhe-font-font-size-2xl);
  line-height: 1.25;
}

[data-scope="heading"][data-part="root"].fd-heading--size-xl3 {
  font-size: var(--fandhe-font-font-size-3xl);
  line-height: 1.2;
}

[data-scope="heading"][data-part="root"].fd-heading--size-xl4 {
  font-size: var(--fandhe-font-font-size-4xl);
  line-height: 1.15;
}
"#;

const TEXT_GOLDEN_CSS: &str = r#"[data-scope="text"][data-part="root"] {
  margin: 0;
}

[data-scope="text"][data-part="root"].fd-text--size-xs {
  font-size: var(--fandhe-font-font-size-xs);
  line-height: 1.4;
}

[data-scope="text"][data-part="root"].fd-text--size-sm {
  font-size: var(--fandhe-font-font-size-sm);
  line-height: 1.45;
}

[data-scope="text"][data-part="root"].fd-text--size-md {
  font-size: var(--fandhe-font-font-size-md);
  line-height: 1.5;
}

[data-scope="text"][data-part="root"].fd-text--size-lg {
  font-size: var(--fandhe-font-font-size-lg);
  line-height: 1.5;
}

[data-scope="text"][data-part="root"].fd-text--size-xl {
  font-size: var(--fandhe-font-font-size-xl);
  line-height: 1.55;
}
"#;

const EM_GOLDEN_CSS: &str = r#"[data-scope="em"][data-part="root"] {
  font-style: italic;
}
"#;

const MARK_GOLDEN_CSS: &str = r#"[data-scope="mark"][data-part="root"] {
  border-radius: var(--fandhe-radius-sm);
  padding-inline: 0.25em;
}

[data-scope="mark"][data-part="root"].fd-mark--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
}

[data-scope="mark"][data-part="root"].fd-mark--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
}

[data-scope="mark"][data-part="root"].fd-mark--variant-text {
  background: transparent;
  color: var(--fandhe-palette);
}

[data-scope="mark"][data-part="root"].fd-mark--variant-plain {
  background: transparent;
  color: inherit;
  padding-inline: 0;
  border-radius: 0;
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="mark"][data-part="root"].fd-mark--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}
"#;

const BLOCKQUOTE_GOLDEN_CSS: &str = r#"[data-scope="blockquote"][data-part="root"] {
  margin: 0;
  padding-inline-start: 1rem;
  padding-block: 0.5rem;
  --fandhe-blockquote-caption-fg: var(--fandhe-color-fg-muted);
}

[data-scope="blockquote"][data-part="content"] {
  margin: 0;
}

[data-scope="blockquote"][data-part="caption"] {
  display: block;
  margin-block-start: 0.5rem;
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-blockquote-caption-fg);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--variant-subtle {
  border-inline-start: 4px solid var(--fandhe-palette-muted);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
  border-inline-start: 4px solid var(--fandhe-palette-emphasized);
  border-radius: var(--fandhe-radius-sm);
  --fandhe-blockquote-caption-fg: var(--fandhe-palette-fg);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--variant-plain {
  background: transparent;
  border-inline-start: 4px solid var(--fandhe-palette);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="blockquote"][data-part="root"].fd-blockquote--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}
"#;

const LIST_GOLDEN_CSS: &str = r#"[data-scope="list"][data-part="root"] {
  margin: 0;
}

[data-scope="list"][data-part="item"] {
  margin-block: var(--fandhe-space-1);
  line-height: 1.5;
}

[data-scope="list"][data-part="indicator"] {
  display: inline-block;
  margin-inline-end: var(--fandhe-space-2);
  vertical-align: middle;
  flex-shrink: 0;
}

[data-scope="list"][data-part="root"].fd-list--variant-marker {
  list-style: revert;
  padding-inline-start: var(--fandhe-space-6);
}

[data-scope="list"][data-part="root"].fd-list--variant-plain {
  list-style: none;
  padding-inline-start: 0;
}

[data-scope="list"][data-part="item"].fd-list--variant-plain {
  display: inline-flex;
  align-items: flex-start;
}

[data-scope="list"][data-part="item"]::marker {
  color: var(--fandhe-color-fg-muted);
}
"#;

const QUOTE_GOLDEN_CSS: &str = r#"[data-scope="quote"][data-part="root"] {
  font-style: italic;
}
"#;

const STRONG_GOLDEN_CSS: &str = r#"[data-scope="strong"][data-part="root"] {
  font-weight: var(--fandhe-font-font-weight-bold);
}
"#;

#[test]
fn heading_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(heading::css(), HEADING_GOLDEN_CSS);
}

#[test]
fn text_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(text::css(), TEXT_GOLDEN_CSS);
}

#[test]
fn em_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(em::css(), EM_GOLDEN_CSS);
}

#[test]
fn mark_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(mark::css(), MARK_GOLDEN_CSS);
}

#[test]
fn blockquote_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(blockquote::css(), BLOCKQUOTE_GOLDEN_CSS);
}

#[test]
fn list_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(list::css(), LIST_GOLDEN_CSS);
}

#[test]
fn quote_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(quote::css(), QUOTE_GOLDEN_CSS);
}

#[test]
fn strong_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(strong::css(), STRONG_GOLDEN_CSS);
}
