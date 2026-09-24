//! `toast::TOAST_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::toast::TOAST_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-toast {
  display: flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  max-width: 100%;
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.4) 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
  box-shadow: 0 0.25em 0 var(--fw-wire-line-subtle);
  user-select: none;
}
.fw-wire-toast-icon {
  display: flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-toast-icon .fw-wire-icon-glyph {
  width: 1.25em;
  height: 1.25em;
}
.fw-wire-toast-message {
  flex: 1;
  min-width: 0;
}
.fw-wire-toast-dismiss {
  display: inline-flex;
  flex-shrink: 0;
  margin-inline-start: auto;
  color: var(--fw-wire-ink-muted);
}
"#;
