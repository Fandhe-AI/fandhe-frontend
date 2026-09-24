//! `counter::COUNTER_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::counter::COUNTER_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-counter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 1.5em;
  height: 1.5em;
  padding: 0 0.4em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  font-variant-numeric: tabular-nums;
  line-height: 1;
  white-space: nowrap;
}
.fw-wire-counter.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
"#;
