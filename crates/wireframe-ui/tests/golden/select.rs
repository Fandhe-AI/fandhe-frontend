//! `select::SELECT_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::select::SELECT_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-select {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
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
.fw-wire-select-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-select-indicator {
  display: inline-flex;
  margin-left: auto;
}
.fw-wire-select .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-select[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 1px var(--fw-wire-ink);
}
.fw-wire-select[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
"#;
