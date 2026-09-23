//! `cursor::CURSOR_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `cursor::CURSOR_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-cursor {
  display: inline-flex;
  align-items: flex-start;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-cursor .fw-wire-icon-glyph {
  fill: var(--fw-wire-paper);
}
.fw-wire-cursor-label {
  margin-top: 0.75em;
  padding: 0.125em 0.5em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
  font-size: 0.75em;
  line-height: 1.4;
  white-space: nowrap;
}
"#;

#[test]
fn cursor_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS,
        EXPECTED_CSS
    );
}
