//! styled Toast（イシュー #760/#1544/#1545）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文（placement 6 variant・status 4
//! variant・action-trigger/close-trigger の hover/focus/disabled/transition・
//! enter 遷移 `@keyframes` を含む）をバイト単位で固定する。出力順
//! （base → variants → states → keyframes）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::toast;

const TOAST_GOLDEN_CSS: &str = r#"[data-scope="toast"][data-part="group"] {
  position: fixed;
  z-index: var(--fandhe-z-index-toast, 9999);
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-4);
  pointer-events: none;
  box-sizing: border-box;
  max-width: 100vw;
}

[data-scope="toast"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  position: relative;
  width: min(24rem, 100%);
  max-width: calc(100vw - var(--fandhe-space-8));
  padding: var(--fandhe-space-4);
  padding-inline-end: var(--fandhe-space-10);
  border-radius: var(--fandhe-radius-md);
  border: 1px solid var(--fandhe-color-border);
  box-shadow: var(--fandhe-shadow-lg);
  pointer-events: auto;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  animation: fd-toast-enter var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard);
}

[data-scope="toast"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-semibold);
}

[data-scope="toast"][data-part="description"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toast"][data-part="action-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  margin-block-start: var(--fandhe-space-1);
  box-sizing: border-box;
  height: var(--fandhe-space-8);
  padding: 0 var(--fandhe-space-3);
  font-family: inherit;
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-tight);
  color: inherit;
  background: transparent;
  border: 1px solid var(--fandhe-palette-muted, var(--fandhe-color-border));
  border-radius: var(--fandhe-radius-md);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-palette-muted, var(--fandhe-color-bg-muted));
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toast"][data-part="close-trigger"] {
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
  color: inherit;
  --fandhe-hover-bg: var(--fandhe-palette-muted, var(--fandhe-color-bg-muted));
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top-start {
  top: 0;
  inset-inline-start: 0;
  align-items: flex-start;
  flex-direction: column-reverse;
  --fandhe-toast-enter-translate: 0 calc(-1 * var(--fandhe-space-2));
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top {
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  align-items: center;
  flex-direction: column-reverse;
  --fandhe-toast-enter-translate: 0 calc(-1 * var(--fandhe-space-2));
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top-end {
  top: 0;
  inset-inline-end: 0;
  align-items: flex-end;
  flex-direction: column-reverse;
  --fandhe-toast-enter-translate: 0 calc(-1 * var(--fandhe-space-2));
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom-start {
  bottom: 0;
  inset-inline-start: 0;
  align-items: flex-start;
  --fandhe-toast-enter-translate: 0 var(--fandhe-space-2);
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom {
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  align-items: center;
  --fandhe-toast-enter-translate: 0 var(--fandhe-space-2);
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom-end {
  bottom: 0;
  inset-inline-end: 0;
  align-items: flex-end;
  --fandhe-toast-enter-translate: 0 var(--fandhe-space-2);
}

[data-scope="toast"][data-part="root"].fd-toast--status-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
  background: var(--fandhe-palette-subtle);
  border-color: var(--fandhe-palette-muted);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="toast"][data-part="root"].fd-toast--status-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
  background: var(--fandhe-palette-subtle);
  border-color: var(--fandhe-palette-muted);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="toast"][data-part="root"].fd-toast--status-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
  background: var(--fandhe-palette-subtle);
  border-color: var(--fandhe-palette-muted);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="toast"][data-part="root"].fd-toast--status-error {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
  background: var(--fandhe-palette-subtle);
  border-color: var(--fandhe-palette-muted);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="toast"][data-part="action-trigger"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toast"][data-part="action-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toast"][data-part="action-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="toast"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="toast"][data-part="action-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="toast"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

@keyframes fd-toast-enter {
  from {
    opacity: 0;
    translate: var(--fandhe-toast-enter-translate, 0 var(--fandhe-space-2));
  }
  to {
    opacity: 1;
    translate: 0 0;
  }
}
"#;

#[test]
fn toast_stylesheet_matches_golden_fixture() {
    assert_eq!(toast::stylesheet(), TOAST_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(toast::stylesheet(), toast::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = toast::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
