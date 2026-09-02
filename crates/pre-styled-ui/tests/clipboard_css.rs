//! styled Clipboard の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/accordion_css.rs`/`breadcrumb_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1519（参考サイト基準への調整）で新規追加した。追加・置換内容
//! （hover / フォーカスリング / トランジション / タイポグラフィ / トークン化
//! / `input` 背景の白背景相当への変更）は
//! `crates/pre-styled-ui/src/clipboard.rs` モジュール doc「参考サイト基準
//! への調整（イシュー #1519）」節に記録する。

use fandhe_frontend_pre_styled_ui::clipboard;

const CLIPBOARD_GOLDEN_CSS: &str = r#"[data-scope="clipboard"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="clipboard"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="clipboard"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope="clipboard"][data-part="input"] {
  flex: 1;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-family: var(--fandhe-font-font-mono);
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="clipboard"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-1);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="clipboard"][data-part="trigger"] {
  transition-property: background, color, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="clipboard"][data-part="indicator"] {
  align-items: center;
}

[data-scope="clipboard"][data-part="value-text"] {
  font-family: var(--fandhe-font-font-mono);
  font-size: var(--fandhe-font-font-size-sm);
  word-break: break-all;
}

[data-scope="clipboard"][data-part="trigger"][data-copied] {
  border-color: var(--fandhe-color-success, var(--fandhe-color-accent));
  color: var(--fandhe-color-success, var(--fandhe-color-accent));
}

[data-scope="clipboard"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="clipboard"][data-part="input"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="clipboard"][data-part="indicator"][data-state="hidden"] {
  display: none;
}

@media (hover: hover) {
  [data-scope="clipboard"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn clipboard_stylesheet_matches_golden_fixture() {
    assert_eq!(clipboard::stylesheet(), CLIPBOARD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(clipboard::stylesheet(), clipboard::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = clipboard::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
