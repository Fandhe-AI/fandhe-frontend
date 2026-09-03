//! styled Tour（イシュー #841）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1551（親 #1549 の 2/2）: `content`/`title`/`description`/
//! `progress-text`/`close-trigger`/`action-trigger` の是正
//! （`crates/pre-styled-ui/src/tour.rs` モジュール冒頭 rustdoc「イシュー
//! #1551」節参照）を反映して golden を更新した。末尾に
//! `@media (hover: hover)` ブロック（close-trigger/action-trigger の
//! hover 規則）が新設された。

use fandhe_frontend_pre_styled_ui::tour;

const TOUR_GOLDEN_CSS: &str = r#"[data-scope="tour"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  z-index: var(--fandhe-z-index-overlay, 1100);
  background: var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5));
}

[data-scope="tour"][data-part="spotlight"] {
  position: fixed;
  z-index: calc(var(--fandhe-z-index-overlay, 1100) + 1);
  top: var(--fandhe-tour-spotlight-y, 40%);
  left: var(--fandhe-tour-spotlight-x, 40%);
  width: var(--fandhe-tour-spotlight-width, 20%);
  height: var(--fandhe-tour-spotlight-height, 20%);
  border-radius: var(--fandhe-tour-spotlight-radius, var(--fandhe-radius-sm, 0.25rem));
  box-shadow: 0 0 0 var(--fandhe-tour-spotlight-ring-width, 2px) var(--fandhe-palette, var(--fandhe-color-accent, #3182ce)), 0 0 0 max(100vw, 100vh) var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.5));
  pointer-events: none;
}

[data-scope="tour"][data-part="positioner"] {
  position: fixed;
  z-index: var(--fandhe-z-index-modal, 1102);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  padding: var(--fandhe-space-4);
  box-sizing: border-box;
  max-width: 100vw;
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
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg, 0.5rem);
  box-shadow: var(--fandhe-shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25));
  box-sizing: border-box;
  padding: var(--fandhe-tour-content-padding, var(--fandhe-space-6));
  max-width: var(--fandhe-tour-content-max-width, 24rem);
}

[data-scope="tour"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-md);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  margin: 0 0 var(--fandhe-space-2) 0;
  padding-inline-end: calc(var(--fandhe-space-8) + var(--fandhe-space-2));
}

[data-scope="tour"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="tour"][data-part="progress-text"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
  line-height: var(--fandhe-font-line-height-normal);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="tour"][data-part="close-trigger"] {
  position: absolute;
  inset-block-start: var(--fandhe-space-2);
  inset-inline-end: var(--fandhe-space-2);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fandhe-space-8);
  height: var(--fandhe-space-8);
  overflow: hidden;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  background: transparent;
  padding: var(--fandhe-space-1);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tour"][data-part="action-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-family: inherit;
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-tight);
  border: none;
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  cursor: pointer;
  margin-inline-end: var(--fandhe-space-2);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="tour"][data-part="root"].fd-tour--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
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

[data-scope="tour"][data-part="positioner"][data-side="top"][data-align="start"] {
  left: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="top"][data-align="end"] {
  left: auto;
  right: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="bottom"][data-align="start"] {
  left: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="bottom"][data-align="end"] {
  left: auto;
  right: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="left"][data-align="start"] {
  top: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="left"][data-align="end"] {
  top: auto;
  bottom: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="right"][data-align="start"] {
  top: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][data-side="right"][data-align="end"] {
  top: auto;
  bottom: var(--fandhe-space-4);
  transform: none;
}

[data-scope="tour"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="content"][hidden] {
  display: none;
}

[data-scope="tour"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tour"][data-part="action-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tour"][data-part="action-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tour"][data-part="action-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="tour"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="tour"][data-part="action-trigger"]:hover:not([data-disabled]):not([disabled]) {
    background: var(--fandhe-hover-bg);
  }
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
