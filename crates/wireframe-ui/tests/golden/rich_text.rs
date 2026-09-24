//! `rich_text::RICH_TEXT_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-rich-text {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  max-width: 100%;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-rich-text-label {
  min-width: 0;
}
.fw-wire-rich-text.fw-wire-bold {
  font-weight: 600;
}
.fw-wire-rich-text.fw-wire-vertical {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25em;
}
"#;
