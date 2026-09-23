//! `annotation::ANNOTATION_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `annotation::ANNOTATION_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-annotation {
  display: inline-block;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-annotation-title {
  font-weight: 600;
}
.fw-wire-annotation-description {
  margin-top: 0.25em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-annotation.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-annotation.fw-wire-primary .fw-wire-annotation-description {
  color: var(--fw-wire-fill);
}
"#;

#[test]
fn annotation_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS,
        EXPECTED_CSS
    );
}
