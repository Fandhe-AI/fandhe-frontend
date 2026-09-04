//! styled SkipNav（イシュー #776）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/separator_css.rs`/`tests/skeleton_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。受け入れ条件の核である
//! `[data-scope="skip-nav"][data-part="link"]:focus-visible` 規則（focus
//! 時のみ視覚的に復元する表示規則）が確実に含まれることを、この golden
//! テストが固定する。
//!
//! イシュー #1586 で参考サイト（chakra-ui `skipNavLinkRecipe`）基準へ
//! スタイル調整した際に、余白のスケール載せ・focus ring の canonical 化
//! （`outline`）・z-index のトークン化・タイポグラフィ・hover・transition
//! の追加を反映して更新した（詳細は `crates/pre-styled-ui/src/skip_nav.rs`
//! モジュール rustdoc「参考サイトとの差分」節を参照）。

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
  display: inline-flex;
  align-items: center;
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  text-decoration: none;
  user-select: none;
  color: var(--fandhe-color-fg);
  background: var(--fandhe-color-bg);
  border-radius: var(--fandhe-radius-md);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="skip-nav"][data-part="content"] {
  outline: none;
}

[data-scope="skip-nav"][data-part="link"]:focus-visible {
  position: fixed;
  top: var(--fandhe-space-6, 1.5rem);
  inset-inline-start: var(--fandhe-space-6, 1.5rem);
  width: auto;
  height: auto;
  padding: var(--fandhe-space-2-5, 0.625rem);
  margin: 0;
  overflow: visible;
  clip: auto;
  white-space: normal;
  z-index: var(--fandhe-z-index-skip-nav, 1200);
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
