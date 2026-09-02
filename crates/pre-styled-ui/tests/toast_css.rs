//! styled Toast（イシュー #760）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文（placement 6 variant・status 4
//! variant を含む）をバイト単位で固定する。出力順（base → variants）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::toast;

const TOAST_GOLDEN_CSS: &str = r#"[data-scope="toast"][data-part="group"] {
  position: fixed;
  z-index: var(--fandhe-z-index-toast, 9999);
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-4);
  pointer-events: none;
}

[data-scope="toast"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  min-width: min(18rem, 100%);
  max-width: calc(100vw - var(--fandhe-space-8));
  padding: var(--fandhe-space-4);
  border-radius: var(--fandhe-radius-md);
  border: 1px solid var(--fandhe-color-border);
  box-shadow: var(--fandhe-shadow-lg);
  pointer-events: auto;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope="toast"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-semibold);
}

[data-scope="toast"][data-part="description"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toast"][data-part="close-trigger"] {
  cursor: pointer;
  align-self: flex-end;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top-start {
  top: 0;
  inset-inline-start: 0;
  align-items: flex-start;
  flex-direction: column-reverse;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top {
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  align-items: center;
  flex-direction: column-reverse;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-top-end {
  top: 0;
  inset-inline-end: 0;
  align-items: flex-end;
  flex-direction: column-reverse;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom-start {
  bottom: 0;
  inset-inline-start: 0;
  align-items: flex-start;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom {
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  align-items: center;
}

[data-scope="toast"][data-part="group"].fd-toast--placement-bottom-end {
  bottom: 0;
  inset-inline-end: 0;
  align-items: flex-end;
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
