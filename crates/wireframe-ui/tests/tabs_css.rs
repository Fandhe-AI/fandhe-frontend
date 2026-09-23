//! `tabs::TABS_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `tabs::TABS_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-tabs {
  display: flex;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-horizontal {
  flex-direction: row;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs.fw-wire-vertical {
  display: inline-flex;
  flex-direction: column;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs-item {
  box-sizing: border-box;
  padding: 0.5em 1em;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  border-bottom: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item {
  border-bottom: none;
  border-right: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-bottom-color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item[data-active] {
  border-bottom-color: transparent;
  border-right-color: var(--fw-wire-ink);
}
"#;

#[test]
fn tabs_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::tabs::TABS_CSS, EXPECTED_CSS);
}
