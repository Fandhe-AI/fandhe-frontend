//! `toast::TOAST_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `toast::TOAST_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-toast {
  display: flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  max-width: 100%;
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.4) 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
  box-shadow: 0 0.25em 0 var(--fw-wire-line-subtle);
  user-select: none;
}
.fw-wire-toast-icon {
  display: flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-toast-icon .fw-wire-icon-glyph {
  width: 1.25em;
  height: 1.25em;
}
.fw-wire-toast-message {
  flex: 1;
  min-width: 0;
}
.fw-wire-toast-dismiss {
  display: inline-flex;
  flex-shrink: 0;
  margin-inline-start: auto;
  color: var(--fw-wire-ink-muted);
}
"#;

#[test]
fn toast_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::toast::TOAST_CSS, EXPECTED_CSS);
}
