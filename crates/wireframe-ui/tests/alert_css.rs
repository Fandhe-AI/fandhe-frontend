//! `alert::ALERT_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `alert::ALERT_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-alert {
  display: flex;
  align-items: flex-start;
  gap: 0.625em;
  box-sizing: border-box;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-alert-icon {
  display: flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-alert-icon .fw-wire-icon-glyph {
  width: 1.25em;
  height: 1.25em;
}
.fw-wire-alert-body {
  display: flex;
  flex-direction: column;
  gap: 0.25em;
  min-width: 0;
}
.fw-wire-alert-title {
  font-weight: 600;
}
.fw-wire-alert-description {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-alert-warning {
  background: var(--fw-wire-fill-subtle);
  border-inline-start-width: calc(var(--fw-wire-line-width) * 3);
}
.fw-wire-alert-error {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-alert-error .fw-wire-alert-icon {
  color: var(--fw-wire-paper);
}
.fw-wire-alert-error .fw-wire-alert-description {
  color: var(--fw-wire-fill);
}
"#;

#[test]
fn alert_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::alert::ALERT_CSS, EXPECTED_CSS);
}
