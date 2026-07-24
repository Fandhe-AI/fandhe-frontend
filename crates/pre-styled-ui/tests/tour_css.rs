//! styled Tour（イシュー #841）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::tour;

const TOUR_GOLDEN_CSS: &str = r#"[data-scope="tour"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  z-index: 1100;
  background: rgba(0, 0, 0, 0.5);
}

[data-scope="tour"][data-part="spotlight"] {
  position: fixed;
  z-index: 1101;
  top: var(--fandhe-tour-spotlight-y, 40%);
  left: var(--fandhe-tour-spotlight-x, 40%);
  width: var(--fandhe-tour-spotlight-width, 20%);
  height: var(--fandhe-tour-spotlight-height, 20%);
  border-radius: var(--fandhe-radius-md);
  box-shadow: 0 0 0 max(100vw, 100vh) rgba(0, 0, 0, 0.5);
  pointer-events: none;
}

[data-scope="tour"][data-part="positioner"] {
  position: fixed;
  z-index: 1102;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  padding: var(--fandhe-space-4);
}

[data-scope="tour"][data-part="arrow"] {
  position: relative;
}

[data-scope="tour"][data-part="arrow-tip"] {
  width: 0.75rem;
  height: 0.75rem;
  background: var(--fandhe-color-bg);
  transform: rotate(45deg);
}

[data-scope="tour"][data-part="content"] {
  position: relative;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25));
  padding: var(--fandhe-space-6);
  max-width: 24rem;
}

[data-scope="tour"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-lg);
  font-weight: var(--fandhe-font-font-weight-semibold);
  margin: 0 0 var(--fandhe-space-2) 0;
}

[data-scope="tour"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="tour"][data-part="progress-text"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="tour"][data-part="close-trigger"] {
  position: absolute;
  top: var(--fandhe-space-2);
  right: var(--fandhe-space-2);
  cursor: pointer;
  background: none;
  border: none;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="tour"][data-part="action-trigger"] {
  cursor: pointer;
  font: inherit;
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-color-bg);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}

[data-scope="tour"][data-part="backdrop"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="spotlight"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="positioner"][data-side="top"] {
  top: var(--fandhe-space-4);
  transform: translateX(-50%);
}

[data-scope="tour"][data-part="positioner"][data-side="bottom"] {
  top: auto;
  bottom: var(--fandhe-space-4);
  transform: translateX(-50%);
}

[data-scope="tour"][data-part="positioner"][data-side="left"] {
  left: var(--fandhe-space-4);
  transform: translateY(-50%);
}

[data-scope="tour"][data-part="positioner"][data-side="right"] {
  left: auto;
  right: var(--fandhe-space-4);
  transform: translateY(-50%);
}

[data-scope="tour"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="content"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="close-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="tour"][data-part="action-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn tour_stylesheet_matches_golden_fixture() {
    assert_eq!(tour::stylesheet(), TOUR_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tour::stylesheet(), tour::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tour::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_never_references_external_resources() {
    let css = tour::stylesheet();
    assert!(!css.contains("url("));
}
