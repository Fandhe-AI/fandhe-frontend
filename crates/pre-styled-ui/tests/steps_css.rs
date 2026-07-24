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
  gap: var(--fandhe-space-2);
  background: none;
  border: none;
  cursor: pointer;
  font: inherit;
  color: inherit;
  padding: 0;
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

[data-scope="steps"][data-part="prev-trigger"] {
  cursor: pointer;
  font: inherit;
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope="steps"][data-part="next-trigger"] {
  cursor: pointer;
  font: inherit;
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-color-bg);
}

[data-scope="steps"][data-part="root"].fd-steps--size-sm {
  --fandhe-steps-indicator-size: 1.5rem;
}

[data-scope="steps"][data-part="root"].fd-steps--size-md {
  --fandhe-steps-indicator-size: 2rem;
}

[data-scope="steps"][data-part="root"].fd-steps--size-lg {
  --fandhe-steps-indicator-size: 2.5rem;
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="steps"][data-part="root"].fd-steps--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
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

[data-scope="steps"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
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

[data-scope="steps"][data-part="completed-content"][data-state="closed"] {
  display: none;
}

[data-scope="steps"][data-part="prev-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="steps"][data-part="prev-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="steps"][data-part="next-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="steps"][data-part="next-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
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
