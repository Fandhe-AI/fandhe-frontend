//! `paragraph::PARAGRAPH_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-paragraph {
  display: block;
  max-width: 100%;
  box-sizing: border-box;
  margin: 0;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.6;
  white-space: pre-line;
  overflow-wrap: anywhere;
}
.fw-wire-paragraph.fw-wire-bold {
  font-weight: 600;
}
"#;
