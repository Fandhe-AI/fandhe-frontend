//! styled Combobox（イシュー #1467/#1468）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → compound → states、hover は `@media (hover: hover)`
//! へ集約されて末尾）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。
//!
//! 分割 1/2（イシュー #1467、control/input/trigger/clear-trigger パート）が
//! 新設し、分割 2/2（イシュー #1468、content/item/item-group/item-indicator
//! パート）が期待値を更新済み（`docs/internal/
//! pre-styled-ui-golden-test-update-guide.md` の手順に従った）。
//!
//! 期待値は 1/2・2/2 双方の是正後の
//! `crates/pre-styled-ui/src/combobox.rs::recipe` の実出力から生成した。

use fandhe_frontend_pre_styled_ui::combobox;

/// `combobox::stylesheet()` の期待値（バイト完全一致）。
const EXPECTED_CSS: &str = r#"[data-scope="combobox"][data-part="root"] {
  position: relative;
}

[data-scope="combobox"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="combobox"][data-part="control"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-combobox-control-padding, var(--fandhe-space-1) var(--fandhe-space-2));
}

[data-scope="combobox"][data-part="control"] {
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="combobox"][data-part="input"] {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: inherit;
  font: inherit;
  padding: var(--fandhe-combobox-input-padding, var(--fandhe-space-1) var(--fandhe-space-2));
}

[data-scope="combobox"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  color: var(--fandhe-color-fg-muted);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="combobox"][data-part="trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="combobox"][data-part="clear-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="combobox"][data-part="clear-trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="combobox"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 10;
  margin-top: var(--fandhe-space-1);
}

[data-scope="combobox"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-md);
  padding: var(--fandhe-combobox-content-padding, var(--fandhe-space-2));
  min-width: var(--fandhe-reference-width, auto);
}

[data-scope="combobox"][data-part="item-group-label"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="combobox"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-combobox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  border-radius: var(--fandhe-radius-sm);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="combobox"][data-part="item"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="combobox"][data-part="item-indicator"] {
  margin-left: auto;
}

[data-scope="combobox"][data-part="root"].fd-combobox--size-xs {
  --fandhe-combobox-control-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-combobox-input-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-combobox-item-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-combobox-content-padding: var(--fandhe-space-0-5);
}

[data-scope="combobox"][data-part="root"].fd-combobox--size-sm {
  --fandhe-combobox-control-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-combobox-input-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-combobox-item-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-combobox-content-padding: var(--fandhe-space-1);
}

[data-scope="combobox"][data-part="root"].fd-combobox--size-md {
  --fandhe-combobox-control-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-combobox-input-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-combobox-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-combobox-content-padding: var(--fandhe-space-2);
}

[data-scope="combobox"][data-part="root"].fd-combobox--size-lg {
  --fandhe-combobox-control-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-combobox-input-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-combobox-item-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-combobox-content-padding: var(--fandhe-space-3);
}

[data-scope="combobox"][data-part="root"].fd-combobox--size-xl {
  --fandhe-combobox-control-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-combobox-input-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-combobox-item-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-combobox-content-padding: var(--fandhe-space-4);
}

[data-scope="combobox"][data-part="control"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="combobox"][data-part="item"][data-state="open"] {
  background: var(--fandhe-color-bg-muted);
}

[data-scope="combobox"][data-part="item"][data-highlighted] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="combobox"][data-part="input"]:focus-visible {
  outline: none;
}

[data-scope="combobox"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="combobox"][data-part="input"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="combobox"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="combobox"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="combobox"][data-part="positioner"][data-positioned] {
  position: fixed;
  top: 0;
  left: 0;
  margin-top: 0;
  transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
}

@media (hover: hover) {
  [data-scope="combobox"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="combobox"][data-part="clear-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="combobox"][data-part="item"]:hover:not([data-disabled]):not([data-highlighted]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn combobox_stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(combobox::stylesheet(), EXPECTED_CSS);
}

#[test]
fn combobox_stylesheet_is_deterministic_across_calls() {
    assert_eq!(combobox::stylesheet(), combobox::stylesheet());
}
