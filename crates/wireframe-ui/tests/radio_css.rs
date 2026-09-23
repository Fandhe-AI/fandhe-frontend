//! `radio::RADIO_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `radio::RADIO_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-radio {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-radio-control {
  position: relative;
  box-sizing: border-box;
  width: 1.25em;
  height: 1.25em;
  flex-shrink: 0;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-radio-label {
  white-space: nowrap;
}
.fw-wire-radio[data-active] .fw-wire-radio-control {
  border-color: var(--fw-wire-ink);
}
.fw-wire-radio[data-active] .fw-wire-radio-control::after {
  content: "";
  position: absolute;
  inset: 0.25em;
  border-radius: 50%;
  background: var(--fw-wire-ink);
}
.fw-wire-radio[data-disabled] {
  opacity: 0.5;
}
.fw-wire-radio[data-disabled] .fw-wire-radio-control {
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
"#;

#[test]
fn radio_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::radio::RADIO_CSS, EXPECTED_CSS);
}
