//! `ratings::RATINGS_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `ratings::RATINGS_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-ratings {
  display: inline-flex;
  align-items: center;
  gap: calc(var(--fw-wire-control-size, 2rem) * 0.1);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-line);
}
.fw-wire-ratings-star {
  display: inline-flex;
  line-height: 0;
}
.fw-wire-ratings-star[data-active] {
  color: var(--fw-wire-ink);
}
.fw-wire-ratings-star[data-active] .fw-wire-icon-glyph {
  fill: currentColor;
}
"#;

#[test]
fn ratings_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS,
        EXPECTED_CSS
    );
}
