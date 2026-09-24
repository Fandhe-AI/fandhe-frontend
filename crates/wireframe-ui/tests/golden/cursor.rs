//! `cursor::CURSOR_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-cursor {
  display: inline-flex;
  align-items: flex-start;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-cursor .fw-wire-icon-glyph {
  fill: var(--fw-wire-paper);
}
.fw-wire-cursor-label {
  margin-top: 0.75em;
  padding: 0.125em 0.5em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
  font-size: 0.75em;
  line-height: 1.4;
  white-space: nowrap;
}
"#;
