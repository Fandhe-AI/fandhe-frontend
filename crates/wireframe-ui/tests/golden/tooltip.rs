//! `tooltip::TOOLTIP_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::tooltip::TOOLTIP_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-tooltip {
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
