//! styled TreeView の決定的 CSS 出力ゴールデンテスト（イシュー #1578）。
//!
//! `crates/pre-styled-ui/tests/json_tree_view_css.rs` 等の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定
//! する。出力順（base → variant → state → `@media (hover: hover)` 集約）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden テスト
//! が即座に検知する。
//!
//! イシュー #1578（参考サイト基準への調整・`size` variant 導入）で新規
//! 追加した。追加・置換内容（選択行の `accent-fg-subtle` 是正・hover/
//! disabled/transition/フォーカスリングの canonical 化・`size` variant・
//! indicator の列幅固定）は `crates/pre-styled-ui/src/tree_view.rs`
//! モジュール doc「参考サイト基準への調整（イシュー #1578）」節に記録する。

use fandhe_frontend_pre_styled_ui::tree_view;

const TREE_VIEW_GOLDEN_CSS: &str = r#"[data-scope="tree-view"][data-part="label"] {
  font-size: var(--fandhe-tree-view-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg);
  margin-block-end: var(--fandhe-space-2);
}

[data-scope="tree-view"][data-part="tree"] {
  display: flex;
  flex-direction: column;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-tree-view-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="tree-view"][data-part="branch-control"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-tree-view-row-gap, var(--fandhe-space-2));
  padding: var(--fandhe-tree-view-row-padding, var(--fandhe-space-1-5) var(--fandhe-space-2-5));
  color: var(--fandhe-color-fg);
  cursor: pointer;
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tree-view"][data-part="branch-control"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tree-view"][data-part="branch-indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  inline-size: var(--fandhe-tree-view-indicator-size, 1em);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="tree-view"][data-part="branch-indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tree-view"][data-part="branch-content"] {
  display: flex;
  padding-inline-start: var(--fandhe-tree-view-indent, 1rem);
}

[data-scope="tree-view"][data-part="branch-indent-guide"] {
  border-inline-start: 1px solid var(--fandhe-color-border-muted);
  margin-inline-start: calc(var(--fandhe-tree-view-indent, 1rem) / 2);
  flex: 0 0 auto;
}

[data-scope="tree-view"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-tree-view-row-gap, var(--fandhe-space-2));
  padding: var(--fandhe-tree-view-row-padding, var(--fandhe-space-1-5) var(--fandhe-space-2-5));
  color: var(--fandhe-color-fg);
  cursor: pointer;
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tree-view"][data-part="item"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tree-view"][data-part="item-indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  inline-size: var(--fandhe-tree-view-indicator-size, 1em);
  color: var(--fandhe-color-accent);
}

[data-scope="tree-view"][data-part="root"].fd-tree-view--size-xs {
  --fandhe-tree-view-row-padding: var(--fandhe-space-0-5) var(--fandhe-space-1-5);
  --fandhe-tree-view-font-size: var(--fandhe-font-font-size-xs);
  --fandhe-tree-view-row-gap: var(--fandhe-space-1);
}

[data-scope="tree-view"][data-part="root"].fd-tree-view--size-sm {
  --fandhe-tree-view-row-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-tree-view-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-tree-view-row-gap: var(--fandhe-space-1-5);
}

[data-scope="tree-view"][data-part="root"].fd-tree-view--size-md {
  --fandhe-tree-view-row-padding: var(--fandhe-space-1-5) var(--fandhe-space-2-5);
  --fandhe-tree-view-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-tree-view-row-gap: var(--fandhe-space-2);
}

[data-scope="tree-view"][data-part="root"].fd-tree-view--size-lg {
  --fandhe-tree-view-row-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-tree-view-font-size: var(--fandhe-font-font-size-md);
  --fandhe-tree-view-row-gap: var(--fandhe-space-2);
}

[data-scope="tree-view"][data-part="root"].fd-tree-view--size-xl {
  --fandhe-tree-view-row-padding: var(--fandhe-space-2-5) var(--fandhe-space-4);
  --fandhe-tree-view-font-size: var(--fandhe-font-font-size-lg);
  --fandhe-tree-view-row-gap: var(--fandhe-space-2-5);
}

[data-scope="tree-view"][data-part="branch-indicator"][data-state="open"] {
  transform: rotate(90deg);
}

[data-scope="tree-view"][data-part="branch-content"][hidden] {
  display: none;
}

[data-scope="tree-view"][data-part="branch-control"][data-selected] {
  background: var(--fandhe-color-accent-subtle);
  color: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="tree-view"][data-part="item"][data-selected] {
  background: var(--fandhe-color-accent-subtle);
  color: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="tree-view"][data-part="item-indicator"][data-selected] {
  color: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="tree-view"][data-part="item-indicator"][hidden] {
  display: none;
}

[data-scope="tree-view"][data-part="branch-control"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tree-view"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tree-view"][data-part="branch-control"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tree-view"][data-part="item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="tree-view"][data-part="branch-control"]:hover:not([data-disabled]):not([data-selected]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="tree-view"][data-part="item"]:hover:not([data-disabled]):not([data-selected]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn tree_view_stylesheet_matches_golden_fixture() {
    assert_eq!(tree_view::stylesheet(), TREE_VIEW_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tree_view::stylesheet(), tree_view::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tree_view::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
