//! `spinner::SPINNER_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-spinner {
  display: inline-block;
  box-sizing: border-box;
  width: var(--fw-wire-control-size);
  height: var(--fw-wire-control-size);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-top-color: var(--fw-wire-ink);
  border-radius: 50%;
  vertical-align: middle;
  flex-shrink: 0;
}
"#;
