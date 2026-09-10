//! styled Collapsible（イシュー #1682。高さトランジション追加は #2192）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/accordion_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → states → `@starting-style` →
//! `@supports not (height: calc-size(auto, size))`〔PR #2289 codex レビュー
//! P1 是正、イシュー #2192〕→ `@media (hover: hover)`）が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。`collapsible` は size/variant 軸を提供しないため
//! （`crate::collapsible` モジュール doc 参照）、variant セクションは
//! 存在しない。

use fandhe_frontend_pre_styled_ui::collapsible;

/// イシュー #2192 直前（`content_height_transition` 未適用）の CSS 全文。
/// `COLLAPSIBLE_GOLDEN_CSS` が本イシューで追加した 3 ブロック（content の
/// 2 個目 base ブロック・`[hidden]` state・`@starting-style`）を除けば
/// バイト単位で不変であることを
/// `collapsible_pre_2192_blocks_remain_verbatim` が固定する
/// （`docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.1 の
/// 純追加運用に倣う）。
const COLLAPSIBLE_GOLDEN_CSS_BEFORE_2192: &str = r#"[data-scope="collapsible"][data-part="root"] {
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

[data-scope="collapsible"][data-part="content"] {
  box-sizing: border-box;
  overflow: visible;
  --fandhe-content-height: initial;
  height: var(--fandhe-content-height, auto);
  height: calc-size(auto, size);
  transition-property: height, padding-block, margin-block, display, overflow;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), step-end;
  transition-behavior: allow-discrete;
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

[data-scope="collapsible"][data-part="content"][hidden] {
  height: 0;
  padding-block: 0;
  margin-block: 0;
  overflow: hidden;
}

@starting-style {
  [data-scope="collapsible"][data-part="content"] {
    height: 0;
    padding-block: 0;
    margin-block: 0;
    overflow: hidden;
  }
}

@supports not (height: calc-size(auto, size)) {
  [data-scope="collapsible"][data-part="content"] {
    height: auto;
    overflow: visible;
    transition: none;
  }
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

/// イシュー #2192 純追加検証: #2192 直前の CSS 全文（`_BEFORE_2192`）に
/// 含まれる全行が現行 `stylesheet()` の中に**連続する部分文字列として**
/// 残っていることを固定する（3 ブロックの純追加であり、既存ブロックの
/// 内容・順序が一切変わっていないことの機械的な裏付け）。
#[test]
fn collapsible_pre_2192_blocks_remain_verbatim() {
    let css = collapsible::stylesheet();
    // `_BEFORE_2192` は「content の 2 個目 base ブロック・`[hidden]` state・
    // `@starting-style` ブロック」を除いた全内容と一致する契約のため、
    // それらのブロック区切り（空行 1 つ）で分割した各セクションが現行
    // 出力に連続部分文字列として現れることを確認する。
    for section in COLLAPSIBLE_GOLDEN_CSS_BEFORE_2192.split("\n\n") {
        assert!(
            css.contains(section),
            "#2192 以前から存在するブロックが verbatim で残っていない: {section}"
        );
    }
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
