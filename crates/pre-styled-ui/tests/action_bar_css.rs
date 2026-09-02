//! styled ActionBar（イシュー #1516、参考サイト基準へのスタイル調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/signature_pad_css.rs` の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定
//! する（受け入れ条件「golden CSS」）。出力順（base → state）が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。`docs/internal/pre-styled-ui-golden-test-update-guide.md`
//! §3.3 が新規追加の必要性を指摘していた「golden 不在 20 部品」の 1 件を
//! 埋める。

use fandhe_frontend_pre_styled_ui::action_bar;

const ACTION_BAR_GOLDEN_CSS: &str = r#"[data-scope="action-bar"][data-part="positioner"] {
  position: fixed;
  bottom: var(--fandhe-space-4);
  left: 50%;
  transform: translateX(-50%);
  z-index: 900;
  display: flex;
  justify-content: center;
}

[data-scope="action-bar"][data-part="content"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-3);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  box-shadow: var(--fandhe-shadow-md);
  padding: var(--fandhe-space-3) var(--fandhe-space-4);
}

[data-scope="action-bar"][data-part="selection-trigger"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg);
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  border: 1px dashed var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: transparent;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="action-bar"][data-part="separator"] {
  width: 1px;
  align-self: stretch;
  background: var(--fandhe-color-border);
}

[data-scope="action-bar"][data-part="close-trigger"] {
  color: var(--fandhe-color-fg-muted);
  padding: var(--fandhe-space-1);
  border: none;
  border-radius: var(--fandhe-radius-md);
  background: transparent;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="action-bar"][data-part="content"][data-state="open"] {
  opacity: 1;
  translate: 0 0;
}

[data-scope="action-bar"][data-part="content"][data-state="closed"] {
  opacity: 0;
  translate: 0 0.5rem;
}

[data-scope="action-bar"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="action-bar"][data-part="selection-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="action-bar"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="action-bar"][data-part="selection-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="action-bar"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn action_bar_stylesheet_matches_golden_fixture() {
    assert_eq!(action_bar::stylesheet(), ACTION_BAR_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(action_bar::stylesheet(), action_bar::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = action_bar::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
