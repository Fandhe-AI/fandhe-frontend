//! styled Drawer（イシュー #758）の決定的 CSS 出力ゴールデンテスト。
//! イシュー #2193 で `close-trigger` の text variant
//! （`[data-variant="text"]`）state 規則を追加した。イシュー #2387 で
//! content/backdrop へ presence（enter/exit）を適用し、旧 `data-state`
//! 連動の静的切替（4 ブロック）を削除した。イシュー #3128 で
//! pre-styled-only `body`/`footer` パート（base 2 ブロック）と
//! `content[data-has-body]`/`content[data-has-body][hidden]`（opt-in flex
//! column 化 state 2 ブロック）を純追加した。イシュー #3129 で
//! pre-styled-only `header` パート（base 1 ブロック、アクセント色帯の
//! レイアウト）と、アクセント塗り（`header`/`description`/`close-trigger`
//! への `data-tone="accent"` state 3 ブロック）・外側 close-trigger 配置
//! （`content[data-close-outside]` state 1 ブロック + `close-trigger
//! [data-close-outside="<start|end|top|bottom>"]` state 4 ブロック）を
//! 純追加した。
//!
//! `crates/pre-styled-ui/tests/dialog_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。placement 4 方向
//! （`data-placement="start"/"end"/"top"/"bottom"`）の layout 規則を含む。

use fandhe_frontend_pre_styled_ui::drawer;

const DRAWER_GOLDEN_CSS: &str = r#"[data-scope="drawer"][data-part="trigger"] {
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  cursor: pointer;
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="drawer"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  z-index: var(--fandhe-z-index-overlay, 1000);
  background: var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4));
}

[data-scope="drawer"][data-part="backdrop"] {
  opacity: 1;
  transition-property: opacity, display;
  transition-duration: var(--fandhe-motion-duration-slow);
  transition-timing-function: var(--fandhe-motion-easing-standard);
  transition-behavior: allow-discrete;
}

[data-scope="drawer"][data-part="positioner"] {
  position: fixed;
  inset: 0;
  z-index: var(--fandhe-z-index-modal, 1001);
  display: flex;
}

[data-scope="drawer"][data-part="content"] {
  position: relative;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  box-shadow: var(--fandhe-shadow-lg);
  padding: var(--fandhe-drawer-content-padding, var(--fandhe-space-6));
  box-sizing: border-box;
  overflow-y: auto;
}

[data-scope="drawer"][data-part="content"] {
  opacity: 1;
  transition-property: opacity, transform, display;
  transition-duration: var(--fandhe-motion-duration-slow);
  transition-timing-function: var(--fandhe-motion-easing-standard);
  transition-behavior: allow-discrete;
}

[data-scope="drawer"][data-part="header"] {
  margin-block-start: calc(-1 * var(--fandhe-drawer-content-padding, var(--fandhe-space-6)));
  margin-inline: calc(-1 * var(--fandhe-drawer-content-padding, var(--fandhe-space-6)));
  margin-block-end: var(--fandhe-space-4);
  padding-block-start: var(--fandhe-drawer-content-padding, var(--fandhe-space-6));
  padding-inline: var(--fandhe-drawer-content-padding, var(--fandhe-space-6));
  padding-block-end: var(--fandhe-space-2);
}

[data-scope="drawer"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-lg);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  margin: 0 0 var(--fandhe-space-2) 0;
  padding-inline-end: calc(var(--fandhe-space-8) + var(--fandhe-space-2));
}

[data-scope="drawer"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  line-height: var(--fandhe-font-line-height-normal);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="drawer"][data-part="body"] {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
}

[data-scope="drawer"][data-part="footer"] {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--fandhe-space-3);
  margin-block-start: var(--fandhe-space-4);
}

[data-scope="drawer"][data-part="close-trigger"] {
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

[data-scope="drawer"][data-part="root"].fd-drawer--size-xs {
  --fandhe-drawer-size: 12rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-sm {
  --fandhe-drawer-size: 16rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-md {
  --fandhe-drawer-size: 20rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-lg {
  --fandhe-drawer-size: 28rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-xl {
  --fandhe-drawer-size: 36rem;
}

[data-scope="drawer"][data-part="content"][data-has-body] {
  display: flex;
  flex-direction: column;
}

[data-scope="drawer"][data-part="content"][data-has-body][hidden] {
  display: none;
}

[data-scope="drawer"][data-part="positioner"][data-placement="start"] {
  flex-direction: row;
  justify-content: flex-start;
}

[data-scope="drawer"][data-part="positioner"][data-placement="end"] {
  flex-direction: row;
  justify-content: flex-end;
}

[data-scope="drawer"][data-part="positioner"][data-placement="top"] {
  flex-direction: column;
  justify-content: flex-start;
}

[data-scope="drawer"][data-part="positioner"][data-placement="bottom"] {
  flex-direction: column;
  justify-content: flex-end;
}

[data-scope="drawer"][data-part="content"][data-placement="start"] {
  width: var(--fandhe-drawer-size, 20rem);
  height: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="end"] {
  width: var(--fandhe-drawer-size, 20rem);
  height: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="top"] {
  height: var(--fandhe-drawer-size, 20rem);
  width: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="bottom"] {
  height: var(--fandhe-drawer-size, 20rem);
  width: 100%;
}

[data-scope="drawer"][data-part="content"][hidden] {
  opacity: 0;
  transform: scale(0.95);
}

[data-scope="drawer"][data-part="backdrop"][hidden] {
  opacity: 0;
}

[data-scope="drawer"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="drawer"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="drawer"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="drawer"][data-part="close-trigger"][data-variant="text"] {
  position: static;
  inset-block-start: auto;
  inset-inline-end: auto;
  box-sizing: border-box;
  width: auto;
  height: auto;
  overflow: visible;
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  color: var(--fandhe-color-fg);
}

[data-scope="drawer"][data-part="header"][data-tone="accent"] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="drawer"][data-part="description"][data-tone="accent"] {
  color: var(--fandhe-color-accent-fg);
}

[data-scope="drawer"][data-part="close-trigger"][data-tone="accent"] {
  color: var(--fandhe-color-accent-fg);
  --fandhe-hover-bg: var(--fandhe-color-accent-emphasized);
}

[data-scope="drawer"][data-part="content"][data-close-outside] {
  overflow: visible;
}

[data-scope="drawer"][data-part="close-trigger"][data-close-outside="end"] {
  inset-inline-end: calc(100% + var(--fandhe-space-2));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="drawer"][data-part="close-trigger"][data-close-outside="start"] {
  inset-inline-end: auto;
  inset-inline-start: calc(100% + var(--fandhe-space-2));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="drawer"][data-part="close-trigger"][data-close-outside="top"] {
  inset-block-start: calc(100% + var(--fandhe-space-2));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="drawer"][data-part="close-trigger"][data-close-outside="bottom"] {
  inset-block-start: auto;
  inset-block-end: calc(100% + var(--fandhe-space-2));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

@starting-style {
  [data-scope="drawer"][data-part="content"] {
    opacity: 0;
    transform: scale(0.95);
  }

  [data-scope="drawer"][data-part="backdrop"] {
    opacity: 0;
  }
}

@media (hover: hover) {
  [data-scope="drawer"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="drawer"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn drawer_stylesheet_matches_golden_fixture() {
    assert_eq!(drawer::stylesheet(), DRAWER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(drawer::stylesheet(), drawer::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = drawer::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
