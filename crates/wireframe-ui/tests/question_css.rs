//! `question::QUESTION_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `question::QUESTION_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-question {
  display: flex;
  flex-direction: column;
  gap: 0.375em;
  max-width: 100%;
  box-sizing: border-box;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-question-label {
  font-weight: 600;
}
.fw-wire-question-description {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-question-control {
  display: block;
  max-width: 100%;
}
.fw-wire-question-hint {
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
"#;

#[test]
fn question_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::question::QUESTION_CSS,
        EXPECTED_CSS
    );
}
