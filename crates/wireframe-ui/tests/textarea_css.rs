//! `textarea::TEXTAREA_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `textarea::TEXTAREA_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-textarea {
  display: block;
  position: relative;
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.5;
}
.fw-wire-textarea-line {
  min-height: 1.5em;
}
.fw-wire-textarea-text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.fw-wire-textarea::after {
  content: "";
  position: absolute;
  right: 0.2em;
  bottom: 0.2em;
  width: 0.6em;
  height: 0.6em;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
}
.fw-wire-textarea[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 2px var(--fw-wire-fill);
}
.fw-wire-textarea[data-disabled] {
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  border-style: dashed;
}
"#;

#[test]
fn textarea_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS,
        EXPECTED_CSS
    );
}
