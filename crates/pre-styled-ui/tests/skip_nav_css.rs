//! styled SkipNav（イシュー #776、スタイル是正はイシュー #1586）の決定的
//! CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/separator_css.rs`/`tests/skeleton_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。受け入れ条件の核である
//! `[data-scope="skip-nav"][data-part="link"]:focus-visible` 規則（focus
//! 時のみ視覚的に復元する表示規則）が確実に含まれることを、この golden
//! テストが固定する。
//!
//! イシュー #1586 で参考サイト（chakra-ui SkipNav）基準へ是正した内容
//! （未定義トークンの実トークン化・`z-index` の正式トークン化・
//! canonical フォーカスリング・文字スタイル・elevation shadow・hover 追加）
//! は [`fandhe_frontend_pre_styled_ui::skip_nav`] モジュール rustdoc 参照。

use fandhe_frontend_pre_styled_ui::skip_nav;

const SKIP_NAV_GOLDEN_CSS: &str = r#"[data-scope="skip-nav"][data-part="link"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="skip-nav"][data-part="content"] {
  outline: none;
}

[data-scope="skip-nav"][data-part="link"]:focus-visible {
  position: fixed;
  top: var(--fandhe-space-6, 1.5rem);
  left: var(--fandhe-space-6, 1.5rem);
  width: auto;
  height: auto;
  padding: var(--fandhe-space-4, 1rem);
  margin: 0;
  overflow: visible;
  clip: auto;
  white-space: normal;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-normal);
  text-decoration: none;
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-md);
  z-index: var(--fandhe-z-index-skip-nav, 1500);
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="skip-nav"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn skip_nav_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(skip_nav::stylesheet(), SKIP_NAV_GOLDEN_CSS);
}

#[test]
fn skip_nav_css_declares_focus_visible_display_rule() {
    let css = skip_nav::stylesheet();
    assert!(css.contains(r#"[data-scope="skip-nav"][data-part="link"]:focus-visible"#));
}
