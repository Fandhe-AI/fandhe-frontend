//! `checkbox::CHECKBOX_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `checkbox::CHECKBOX_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  user-select: none;
}
.fw-wire-checkbox-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 1.25em;
  height: 1.25em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  flex-shrink: 0;
}
.fw-wire-checkbox-label {
  white-space: nowrap;
}
.fw-wire-checkbox[data-active] .fw-wire-checkbox-box {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-checkbox[data-disabled] {
  opacity: 0.5;
}
"#;

#[test]
fn checkbox_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS,
        EXPECTED_CSS
    );
}
