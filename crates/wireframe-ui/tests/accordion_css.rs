//! `accordion::ACCORDION_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `accordion::ACCORDION_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-accordion {
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-accordion-item {
  box-sizing: border-box;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-accordion-item:last-child {
  border-bottom: none;
}
.fw-wire-accordion-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5em;
  box-sizing: border-box;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0.5em 0.75em;
}
.fw-wire-accordion-title {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-accordion-indicator {
  display: inline-flex;
  flex: 0 0 auto;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-accordion-body {
  box-sizing: border-box;
  padding: 0 0.75em 0.75em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-accordion-item[data-active] {
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-accordion-item[data-active] .fw-wire-accordion-title {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-accordion-item[data-active] .fw-wire-accordion-indicator {
  color: var(--fw-wire-ink);
}
"#;

#[test]
fn accordion_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::accordion::ACCORDION_CSS,
        EXPECTED_CSS
    );
}
