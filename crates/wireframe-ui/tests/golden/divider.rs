//! `divider::DIVIDER_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::divider::DIVIDER_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-divider {
  display: flex;
  align-items: center;
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-divider::before,
.fw-wire-divider::after {
  content: "";
  flex: 1;
}
.fw-wire-divider.fw-wire-horizontal {
  width: 100%;
  margin: 0.75em 0;
}
.fw-wire-divider.fw-wire-horizontal::before,
.fw-wire-divider.fw-wire-horizontal::after {
  border-top: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-divider.fw-wire-vertical {
  display: inline-flex;
  flex-direction: column;
  min-height: var(--fw-wire-control-size, 2rem);
  align-self: stretch;
  margin: 0 0.75em;
}
.fw-wire-divider.fw-wire-vertical::before,
.fw-wire-divider.fw-wire-vertical::after {
  border-left: var(--fw-wire-line-width) solid var(--fw-wire-line);
  min-height: 0.75em;
}
.fw-wire-divider-label {
  padding: 0 0.75em;
  white-space: nowrap;
}
.fw-wire-divider.fw-wire-vertical .fw-wire-divider-label {
  padding: 0.5em 0;
}
"#;
