//! styled Tab Nav（イシュー #996）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tabs_css.rs` と同型の golden fixture テスト。
//! `tab-nav` は `tabs` と scope が異なるため出力される CSS 規則
//! （セレクタ文字列）は別物だが、宣言列は `crates/pre-styled-ui/src/tabs.rs`
//! の `pub(crate)` ヘルパから再利用している（`crates/pre-styled-ui/src/tab_nav.rs`
//! モジュール冒頭 rustdoc「CSS 共有の設計」節参照）。本 golden はその再利用の
//! 結果を scope 別にバイト単位で固定する。

use fandhe_frontend_pre_styled_ui::tab_nav;

const TAB_NAV_GOLDEN_CSS: &str = r#"[data-scope="tab-nav"][data-part="root"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tab-nav"][data-part="link"] {
  padding: var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  text-decoration: none;
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
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
