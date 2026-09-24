//! `question::QUESTION_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::question::QUESTION_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-question {
  display: flex;
  flex-direction: column;
  gap: 0.375em;
  max-width: 100%;
  box-sizing: border-box;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-question-label {
  font-weight: 600;
}
.fw-wire-question-description {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-question-control {
  display: block;
  max-width: 100%;
}
.fw-wire-question-hint {
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
"#;
