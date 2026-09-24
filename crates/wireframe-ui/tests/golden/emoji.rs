//! `emoji::EMOJI_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-emoji {
  display: inline-block;
  line-height: 1;
  white-space: nowrap;
  vertical-align: -0.125em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  filter: grayscale(1);
}
.fw-wire-emoji:empty {
  width: 1em;
  height: 1em;
  box-sizing: border-box;
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line);
  border-radius: 50%;
}
"#;
