//! `button::BUTTON_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::button::BUTTON_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5em;
  box-sizing: border-box;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-button-label {
  white-space: nowrap;
  font-weight: 600;
}
.fw-wire-button.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-button[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
}
"#;
