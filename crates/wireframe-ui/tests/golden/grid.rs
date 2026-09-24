//! `grid::GRID_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::grid::GRID_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-grid {
  display: grid;
  box-sizing: border-box;
  max-width: 100%;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-grid-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.fw-wire-grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.fw-wire-grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.fw-wire-grid-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.fw-wire-grid-cols-5 { grid-template-columns: repeat(5, minmax(0, 1fr)); }
.fw-wire-grid-cols-6 { grid-template-columns: repeat(6, minmax(0, 1fr)); }
.fw-wire-grid-cols-7 { grid-template-columns: repeat(7, minmax(0, 1fr)); }
.fw-wire-grid-cols-8 { grid-template-columns: repeat(8, minmax(0, 1fr)); }
.fw-wire-grid-cols-9 { grid-template-columns: repeat(9, minmax(0, 1fr)); }
.fw-wire-grid-cols-10 { grid-template-columns: repeat(10, minmax(0, 1fr)); }
.fw-wire-grid-cols-11 { grid-template-columns: repeat(11, minmax(0, 1fr)); }
.fw-wire-grid-cols-12 { grid-template-columns: repeat(12, minmax(0, 1fr)); }
.fw-wire-grid.fw-wire-size-xs { gap: 0.25rem; }
.fw-wire-grid.fw-wire-size-sm { gap: 0.5rem; }
.fw-wire-grid.fw-wire-size-md { gap: 1rem; }
.fw-wire-grid.fw-wire-size-lg { gap: 1.5rem; }
.fw-wire-grid.fw-wire-size-xl { gap: 2rem; }
.fw-wire-grid-item {
  min-width: 0;
  box-sizing: border-box;
  padding: 0.5em;
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line-subtle);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
}
"#;
