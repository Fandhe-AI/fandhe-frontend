//! `paragraph::PARAGRAPH_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `paragraph::PARAGRAPH_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-paragraph {
  display: block;
  max-width: 100%;
  box-sizing: border-box;
  margin: 0;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.6;
  white-space: pre-line;
  overflow-wrap: anywhere;
}
.fw-wire-paragraph.fw-wire-bold {
  font-weight: 600;
}
"#;

#[test]
fn paragraph_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS,
        EXPECTED_CSS
    );
}
