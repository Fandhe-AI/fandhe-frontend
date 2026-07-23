//! styled VisuallyHidden（イシュー #776）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/separator_css.rs`/`tests/skeleton_css.rs` の
//! golden fixture テストの前例に倣い、`css()` が返す CSS 全文をバイト単位で
//! 固定する。clip 手法の宣言列（`crate::visually_hidden::clip_declarations`、
//! `crate::skip_nav` の `link` base とも共有する単一情報源）に意図しない
//! 追加・欠落・順序変更があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::visually_hidden;

const VISUALLY_HIDDEN_GOLDEN_CSS: &str = r#"[data-scope="visually-hidden"][data-part="root"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
"#;

#[test]
fn visually_hidden_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(visually_hidden::css(), VISUALLY_HIDDEN_GOLDEN_CSS);
}
