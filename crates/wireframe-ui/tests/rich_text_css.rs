//! `rich_text::RICH_TEXT_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `rich_text::RICH_TEXT_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-rich-text {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  max-width: 100%;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-rich-text-label {
  min-width: 0;
}
.fw-wire-rich-text.fw-wire-bold {
  font-weight: 600;
}
.fw-wire-rich-text.fw-wire-vertical {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25em;
}
"#;

#[test]
fn rich_text_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS,
        EXPECTED_CSS
    );
}
