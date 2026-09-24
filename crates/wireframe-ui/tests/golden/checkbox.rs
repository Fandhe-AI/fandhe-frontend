//! `checkbox::CHECKBOX_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  user-select: none;
}
.fw-wire-checkbox-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 1.25em;
  height: 1.25em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  flex-shrink: 0;
}
.fw-wire-checkbox-label {
  white-space: nowrap;
}
.fw-wire-checkbox[data-active] .fw-wire-checkbox-box {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-checkbox[data-disabled] {
  opacity: 0.5;
}
"#;
