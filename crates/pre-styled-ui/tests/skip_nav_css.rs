//! styled SkipNav（イシュー #776）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/separator_css.rs`/`tests/skeleton_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。受け入れ条件の核である
//! `[data-scope="skip-nav"][data-part="link"]:focus-visible` 規則（focus
//! 時のみ視覚的に復元する表示規則）が確実に含まれることを、この golden
//! テストが固定する。

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
  overflow-wrap: normal;
  border-width: 0;
}

[data-scope="skip-nav"][data-part="content"] {
  outline: none;
}

[data-scope="skip-nav"][data-part="link"]:focus-visible {
  position: fixed;
  top: var(--fandhe-space-md, 1rem);
  left: var(--fandhe-space-md, 1rem);
  width: auto;
  height: auto;
  padding: var(--fandhe-space-sm, 0.5rem) var(--fandhe-space-md, 1rem);
  margin: 0;
  overflow: visible;
  clip: auto;
  white-space: normal;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-md);
  box-shadow: 0 0 0 2px var(--fandhe-color-accent, var(--fandhe-color-fg));
  z-index: 1200;
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
