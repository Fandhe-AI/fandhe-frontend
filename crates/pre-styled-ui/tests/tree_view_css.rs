//! styled TreeView の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/json_tree_view_css.rs` 等の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → states → `@media (hover: hover)`）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1578（参考サイト基準への調整）で新規追加した。追加・置換内容
//! （行の寸法・余白のトークン化・`:empty` によるシェブロン描画・hover/
//! disabled/transition・canonical フォーカスリング・選択文字色の
//! `accent-fg-subtle` への是正・indent-guide の絶対配置化）は
//! `crates/pre-styled-ui/src/tree_view.rs` モジュール doc「参考サイト基準
//! への調整（イシュー #1578）」節に記録する。

use fandhe_frontend_pre_styled_ui::tree_view;

const TREE_VIEW_GOLDEN_CSS: &str = r#"[data-scope="tree-view"][data-part="root"] {
  display: flex;
  flex-direction: column;
  flex: 1 1 auto;
  min-width: 0;
}

[data-scope="tree-view"][data-part="label"] {
  display: block;
  margin-block-end: var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg);
  user-select: none;
}

[data-scope="tree-view"][data-part="tree"] {
  display: flex;
  flex-direction: column;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="tree-view"][data-part="branch-control"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-tree-view-row-padding-block, var(--fandhe-space-1-5)) var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3));
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  user-select: none;
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
  width: var(--fandhe-tree-view-icon-size, var(--fandhe-space-4));
  height: var(--fandhe-tree-view-icon-size, var(--fandhe-space-4));
  color: var(--fandhe-color-fg-muted);
  transform-origin: center;
  transform: rotate(calc(var(--fandhe-tree-view-indicator-base-angle, 0deg) + var(--fandhe-tree-view-indicator-open-angle, 0deg)));
}

[data-scope="tree-view"][data-part="branch-indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tree-view"][data-part="branch-text"] {
  flex: 1 1 auto;
  min-width: 0;
}

[data-scope="tree-view"][data-part="branch-content"] {
  position: relative;
  padding-inline-start: var(--fandhe-tree-view-indent, 1rem);
}

[data-scope="tree-view"][data-part="branch-indent-guide"] {
  position: absolute;
  inset-block: 0;
  inset-inline-start: calc(var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3)) + var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2);
  width: 1px;
  background: var(--fandhe-color-border);
}

[data-scope="tree-view"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-tree-view-row-padding-block, var(--fandhe-space-1-5)) var(--fandhe-tree-view-row-padding-inline, var(--fandhe-space-3));
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  user-select: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tree-view"][data-part="item"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tree-view"][data-part="item-text"] {
  flex: 1 1 auto;
  min-width: 0;
}

[data-scope="tree-view"][data-part="item-indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: var(--fandhe-tree-view-icon-size, var(--fandhe-space-4));
  height: var(--fandhe-tree-view-icon-size, var(--fandhe-space-4));
}

[data-scope="tree-view"][data-part="branch-indicator"][data-state="open"] {
  --fandhe-tree-view-indicator-open-angle: 90deg;
}

[data-scope="tree-view"][data-part="branch-indicator"]:empty {
  width: calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2);
  height: calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 2);
  margin: calc(var(--fandhe-tree-view-icon-size, var(--fandhe-space-4)) / 4);
  border-inline-end: 2px solid currentColor;
  border-block-end: 2px solid currentColor;
  --fandhe-tree-view-indicator-base-angle: -45deg;
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
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope="tree-view"][data-part="item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
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
fn tree_view_stylesheet_is_deterministic() {
    assert_eq!(tree_view::stylesheet(), tree_view::stylesheet());
}

#[test]
fn tree_view_stylesheet_never_contains_style_breakout_sequences() {
    let css = tree_view::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
