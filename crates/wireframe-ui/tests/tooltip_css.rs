//! `tooltip::TOOLTIP_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `tooltip::TOOLTIP_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-tooltip {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  max-width: 100%;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-tooltip-content {
  padding: 0.375em 0.625em;
  max-width: 16em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-ink-muted);
  color: var(--fw-wire-paper);
  overflow-wrap: anywhere;
}
.fw-wire-tooltip-arrow {
  flex: 0 0 auto;
  width: 0;
  height: 0;
  border: 0.375em solid transparent;
}
.fw-wire-tooltip.fw-wire-tooltip-side-top {
  flex-direction: column;
}
.fw-wire-tooltip.fw-wire-tooltip-side-top .fw-wire-tooltip-arrow {
  border-top-color: var(--fw-wire-ink-muted);
  border-bottom-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-bottom {
  flex-direction: column-reverse;
}
.fw-wire-tooltip.fw-wire-tooltip-side-bottom .fw-wire-tooltip-arrow {
  border-bottom-color: var(--fw-wire-ink-muted);
  border-top-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-left {
  flex-direction: row;
}
.fw-wire-tooltip.fw-wire-tooltip-side-left .fw-wire-tooltip-arrow {
  border-left-color: var(--fw-wire-ink-muted);
  border-right-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-right {
  flex-direction: row-reverse;
}
.fw-wire-tooltip.fw-wire-tooltip-side-right .fw-wire-tooltip-arrow {
  border-right-color: var(--fw-wire-ink-muted);
  border-left-width: 0;
}
"#;

#[test]
fn tooltip_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::tooltip::TOOLTIP_CSS,
        EXPECTED_CSS
    );
}
