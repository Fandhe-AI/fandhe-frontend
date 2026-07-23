//! styled Highlight（イシュー #775）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/separator_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。`root` は
//! variant 軸を持たず規則を出力しないため、`mark` slot の淡色強調規則のみが
//! 含まれる（`crates/pre-styled-ui/src/highlight.rs` の recipe 定義参照）。

use fandhe_frontend_pre_styled_ui::highlight;

const HIGHLIGHT_GOLDEN_CSS: &str = r#"[data-scope="highlight"][data-part="mark"] {
  background: var(--fandhe-color-bg-subtle);
  color: inherit;
  padding-inline: 0.125rem;
  border-radius: var(--fandhe-radius-sm);
}
"#;

#[test]
fn highlight_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(highlight::css(), HIGHLIGHT_GOLDEN_CSS);
}
