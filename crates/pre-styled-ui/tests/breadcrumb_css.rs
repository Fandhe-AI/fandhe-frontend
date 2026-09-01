//! styled Breadcrumb（`size`/`variant` variant 展開、イシュー #755）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/accordion_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1517（参考サイト基準への調整）で以下を追加・置換した:
//! `list`/`item` の `gap` トークン化（生 `0.375rem` → `var(--fandhe-space-1-5)`）・
//! `link` の hover（`@media (hover: hover)` 集約出力、`fg-muted` → `fg`）・
//! `link` のキーボードフォーカスリング（`focus_ring_declarations(Token,
//! Outside)`）・`link` の色 transition・フォーカスリング形状のための
//! `link` `border-radius` 追加。意図的に追随しない差分（size 5 段維持・
//! colorPalette 軸不採用・disabled 状態なし・非対話 slot への hover なし）
//! は `crates/pre-styled-ui/src/breadcrumb.rs` モジュール doc「参考サイト
//! 基準への調整（イシュー #1517）」節に記録する。

use fandhe_frontend_pre_styled_ui::breadcrumb;

const BREADCRUMB_GOLDEN_CSS: &str = r#"[data-scope="breadcrumb"][data-part="list"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fandhe-space-1-5);
  list-style: none;
  margin: 0;
  padding: 0;
  font-size: var(--fandhe-breadcrumb-font-size, var(--fandhe-font-font-size-md));
}

[data-scope="breadcrumb"][data-part="item"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1-5);
}

[data-scope="breadcrumb"][data-part="link"] {
  color: var(--fandhe-color-fg-muted);
  text-decoration: var(--fandhe-breadcrumb-link-text-decoration, none);
  border-radius: var(--fandhe-radius-sm);
}

[data-scope="breadcrumb"][data-part="link"] {
  transition-property: color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="breadcrumb"][data-part="current-link"] {
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="breadcrumb"][data-part="separator"] {
  display: inline-flex;
  align-items: center;
  color: var(--fandhe-color-fg-subtle);
}

[data-scope="breadcrumb"][data-part="ellipsis"] {
  display: inline-flex;
  align-items: center;
  color: var(--fandhe-color-fg-subtle);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--size-xs {
  --fandhe-breadcrumb-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--size-sm {
  --fandhe-breadcrumb-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--size-md {
  --fandhe-breadcrumb-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--size-lg {
  --fandhe-breadcrumb-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--size-xl {
  --fandhe-breadcrumb-font-size: var(--fandhe-font-font-size-xl);
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--variant-plain {
  --fandhe-breadcrumb-link-text-decoration: none;
}

[data-scope="breadcrumb"][data-part="root"].fd-breadcrumb--variant-underline {
  --fandhe-breadcrumb-link-text-decoration: underline;
}

[data-scope="breadcrumb"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="breadcrumb"][data-part="link"]:hover:not([data-disabled]) {
    color: var(--fandhe-color-fg);
  }
}
"#;

#[test]
fn breadcrumb_stylesheet_matches_golden_fixture() {
    assert_eq!(breadcrumb::stylesheet(), BREADCRUMB_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(breadcrumb::stylesheet(), breadcrumb::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = breadcrumb::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
