//! styled Popover / Tooltip（イシュー #664）の決定的 CSS 出力ゴールデンテスト。
//!
//! [`crates/pre-styled-ui/tests/recipe_css.rs`] の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 3）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//!
//! `TOOLTIP_GOLDEN_CSS` はイシュー #1548（tooltip のスタイルを参考サイト
//! 基準へ調整）で更新した。`trigger` の枠線付きボタン化・hover/disabled/
//! focus-visible の共通ビジュアル言語（`crate::recipe`）への載せ替え・
//! `positioner` の z-index トークン化・`content` の角丸トークン化と影の
//! 新設を反映する（詳細は `crates/pre-styled-ui/src/tooltip.rs` の
//! モジュール rustdoc「イシュー #1548 の参照サイト比較（7 軸チェック）」
//! 節を参照）。

use fandhe_frontend_pre_styled_ui::{popover, tooltip};

const POPOVER_GOLDEN_CSS: &str = r#"[data-scope="popover"][data-part="root"] {
  position: relative;
}

[data-scope="popover"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md, 0.375rem);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="popover"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: var(--fandhe-z-index-popover, 10);
  margin-top: var(--fandhe-space-1);
}

[data-scope="popover"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg, 0.5rem);
  box-shadow: var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15));
  padding: var(--fandhe-space-4);
  min-width: var(--fandhe-reference-width, auto);
}

[data-scope="popover"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-lg);
  font-weight: var(--fandhe-font-font-weight-semibold);
  margin: 0 0 var(--fandhe-space-2) 0;
}

[data-scope="popover"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  margin: 0;
}

[data-scope="popover"][data-part="close-trigger"] {
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  padding: var(--fandhe-space-1);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="popover"][data-part="trigger"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="popover"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="popover"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="popover"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="popover"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="popover"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="popover"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

const TOOLTIP_GOLDEN_CSS: &str = r#"[data-scope="tooltip"][data-part="root"] {
  position: relative;
}

[data-scope="tooltip"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md, 0.375rem);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tooltip"][data-part="positioner"] {
  position: absolute;
  bottom: 100%;
  left: 0;
  z-index: var(--fandhe-z-index-tooltip, 1100);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="tooltip"][data-part="content"] {
  background: var(--fandhe-color-fg);
  color: var(--fandhe-color-bg);
  font-size: var(--fandhe-font-font-size-sm);
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  box-shadow: var(--fandhe-shadow-sm);
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  max-width: 20rem;
}

[data-scope="tooltip"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="tooltip"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tooltip"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="tooltip"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn popover_stylesheet_matches_golden_fixture() {
    assert_eq!(popover::stylesheet(), POPOVER_GOLDEN_CSS);
}

#[test]
fn tooltip_stylesheet_matches_golden_fixture() {
    assert_eq!(tooltip::stylesheet(), TOOLTIP_GOLDEN_CSS);
}

#[test]
fn stylesheets_are_byte_identical_across_calls() {
    // recipe_determinism.rs と同観点: 独立呼び出し間でバイト単位の一致を固定する。
    assert_eq!(popover::stylesheet(), popover::stylesheet());
    assert_eq!(tooltip::stylesheet(), tooltip::stylesheet());
}
