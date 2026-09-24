//! `textarea::TEXTAREA_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-textarea {
  display: block;
  position: relative;
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.5;
}
.fw-wire-textarea-line {
  min-height: 1.5em;
}
.fw-wire-textarea-text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.fw-wire-textarea::after {
  content: "";
  position: absolute;
  right: 0.2em;
  bottom: 0.2em;
  width: 0.6em;
  height: 0.6em;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
}
.fw-wire-textarea[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 2px var(--fw-wire-fill);
}
.fw-wire-textarea[data-disabled] {
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  border-style: dashed;
}
"#;
