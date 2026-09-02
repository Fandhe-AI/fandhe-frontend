//! styled Tab Nav（イシュー #996、イシュー #1541 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tabs_css.rs` と同型の golden fixture テスト。
//! イシュー #1541 で `crate::tabs` の `pub(crate)` ヘルパ共有をやめ、
//! `tab_nav.rs` は自前の宣言列を持つ（`tab_nav.rs` モジュール冒頭 rustdoc
//! 「参考サイト基準への調整」節参照。並列実行される兄弟イシュー #1542 が
//! `tabs.rs` を変更しても本 golden が影響を受けないようにする独立化）。
//! 本 golden はその宣言列（size 軸・hover・フォーカスリング・トランジション
//! を含む）の出力をバイト単位で固定する。

use fandhe_frontend_pre_styled_ui::tab_nav;

const TAB_NAV_GOLDEN_CSS: &str = r#"[data-scope="tab-nav"][data-part="root"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tab-nav"][data-part="link"] {
  padding: var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  font-size: var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;
  cursor: pointer;
  text-decoration: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tab-nav"][data-part="link"] {
  transition-property: color, background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xs {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-0-5) var(--fandhe-space-2);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-sm {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-md {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-lg {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xl {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-4) var(--fandhe-space-6);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }
}
"#;

#[test]
fn tab_nav_stylesheet_matches_golden_fixture() {
    assert_eq!(tab_nav::stylesheet(), TAB_NAV_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tab_nav::stylesheet(), tab_nav::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tab_nav::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
