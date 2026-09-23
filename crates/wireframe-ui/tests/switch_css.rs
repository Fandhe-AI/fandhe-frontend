//! `switch::SWITCH_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `switch::SWITCH_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-switch {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  min-height: var(--fw-wire-control-size, 2rem);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-switch-track {
  position: relative;
  box-sizing: border-box;
  flex: 0 0 auto;
  width: var(--fw-wire-control-size, 2rem);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.55);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-switch-thumb {
  position: absolute;
  top: 50%;
  left: 0.125em;
  transform: translateY(-50%);
  height: calc(100% - 0.25em);
  aspect-ratio: 1 / 1;
  box-sizing: border-box;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-switch-label {
  white-space: nowrap;
}
.fw-wire-switch[data-active] .fw-wire-switch-track {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
}
.fw-wire-switch[data-active] .fw-wire-switch-thumb {
  left: auto;
  right: 0.125em;
  border-color: var(--fw-wire-paper);
}
.fw-wire-switch[data-disabled] {
  opacity: 0.5;
}
.fw-wire-switch[data-disabled] .fw-wire-switch-track {
  border-style: dashed;
}
"#;

#[test]
fn switch_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::switch::SWITCH_CSS,
        EXPECTED_CSS
    );
}
