//! `button::BUTTON_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `button::BUTTON_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5em;
  box-sizing: border-box;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-button-label {
  white-space: nowrap;
  font-weight: 600;
}
.fw-wire-button.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-button[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
}
"#;

#[test]
fn button_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::button::BUTTON_CSS,
        EXPECTED_CSS
    );
}
