//! styled Toolbar（イシュー #1547、参考サイト基準へのスタイル調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/action_bar_css.rs` の golden fixture テスト
//! の前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → state → hover media
//! query）が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。
//! `docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3 が
//! 新規追加の必要性を指摘していた「golden 不在」の 1 件を埋める。

use fandhe_frontend_pre_styled_ui::toolbar;

const TOOLBAR_GOLDEN_CSS: &str = r#"[data-scope="toolbar"][data-part="root"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  padding: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
}

[data-scope="toolbar"][data-part="button"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
  background: transparent;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toolbar"][data-part="link"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
  background: transparent;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
  text-decoration: none;
}

[data-scope="toolbar"][data-part="separator"] {
  background: var(--fandhe-color-border);
  width: 1px;
  align-self: stretch;
}

[data-scope="toolbar"][data-part="toggle-group"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="toolbar"][data-part="toggle-item"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
  background: transparent;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toolbar"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="toolbar"][data-part="separator"][aria-orientation="horizontal"] {
  height: 1px;
  width: 100%;
  align-self: auto;
}

[data-scope="toolbar"][data-part="toggle-item"][data-state="on"] {
  background: var(--fandhe-color-accent-subtle);
  color: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="toolbar"][data-part="button"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toolbar"][data-part="toggle-item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toolbar"][data-part="button"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="toolbar"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="toolbar"][data-part="toggle-item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="toolbar"][data-part="button"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }

  [data-scope="toolbar"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }

  [data-scope="toolbar"][data-part="toggle-item"]:hover:not([data-disabled]):not([data-state="on"]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }
}
"#;

#[test]
fn toolbar_stylesheet_matches_golden_fixture() {
    assert_eq!(toolbar::stylesheet(), TOOLBAR_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(toolbar::stylesheet(), toolbar::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = toolbar::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
