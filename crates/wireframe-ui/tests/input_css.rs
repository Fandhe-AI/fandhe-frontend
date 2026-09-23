//! `input::INPUT_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `input::INPUT_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-input {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-input-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-input .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-input[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 1px var(--fw-wire-ink);
  color: var(--fw-wire-ink);
}
.fw-wire-input[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
"#;

#[test]
fn input_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::input::INPUT_CSS, EXPECTED_CSS);
}
