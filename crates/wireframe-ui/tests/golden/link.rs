//! `link::LINK_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::link::LINK_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-link {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-link-label {
  text-decoration: underline;
  text-decoration-color: var(--fw-wire-line);
  text-underline-offset: 0.15em;
}
.fw-wire-link.fw-wire-bold {
  font-weight: 600;
}
"#;
