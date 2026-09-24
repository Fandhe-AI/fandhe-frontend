//! `annotation::ANNOTATION_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-annotation {
  display: inline-block;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-annotation-title {
  font-weight: 600;
}
.fw-wire-annotation-description {
  margin-top: 0.25em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-annotation.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-annotation.fw-wire-primary .fw-wire-annotation-description {
  color: var(--fw-wire-fill);
}
"#;
