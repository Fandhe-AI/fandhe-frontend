//! `pagination::PAGINATION_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-pagination {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-pagination-item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 2em;
  height: 2em;
  padding: 0 0.5em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink-muted);
  white-space: nowrap;
}
.fw-wire-pagination-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-pagination-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-pagination-gap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
"#;
