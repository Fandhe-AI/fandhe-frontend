//! styled Collapsible（イシュー #1682）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/accordion_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。`collapsible` は size/variant 軸を
//! 提供しないため（`crate::collapsible` モジュール doc 参照）、variant
//! セクションは存在しない。

use fandhe_frontend_pre_styled_ui::collapsible;

const COLLAPSIBLE_GOLDEN_CSS: &str = r#"[data-scope="collapsible"][data-part="root"] {
  display: block;
}

[data-scope="collapsible"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  background: transparent;
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
  border: 0;
  border-radius: var(--fandhe-radius-md);
  cursor: pointer;
  text-align: left;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="collapsible"][data-part="indicator"] {
  display: inline-block;
  color: var(--fandhe-color-fg-muted);
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="collapsible"][data-part="content"] {
  margin-top: var(--fandhe-space-2);
  padding: var(--fandhe-space-4);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
}

[data-scope="collapsible"][data-part="trigger"][data-state="open"] {
  color: var(--fandhe-color-accent);
}

[data-scope="collapsible"][data-part="indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="collapsible"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="collapsible"][data-part="content"][data-disabled] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="collapsible"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="collapsible"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn collapsible_stylesheet_matches_golden_fixture() {
    assert_eq!(collapsible::stylesheet(), COLLAPSIBLE_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(collapsible::stylesheet(), collapsible::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = collapsible::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
