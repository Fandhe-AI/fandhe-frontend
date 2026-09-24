//! `stat::STAT_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::stat::STAT_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-stat {
  display: inline-flex;
  flex-direction: column;
  gap: 0.25em;
  box-sizing: border-box;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-stat-label {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-value {
  font-size: 2em;
  font-weight: 600;
  line-height: 1.1;
}
.fw-wire-stat-delta {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-delta-icon .fw-wire-icon-glyph {
  width: 1em;
  height: 1em;
}
.fw-wire-stat-up {
  font-weight: 600;
  color: var(--fw-wire-ink);
}
"#;
