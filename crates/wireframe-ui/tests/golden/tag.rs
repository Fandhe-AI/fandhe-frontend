//! `tag::TAG_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::tag::TAG_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.125em 0.625em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-tag-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-tag-remove {
  display: inline-flex;
  align-items: center;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-tag.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-tag.fw-wire-primary .fw-wire-tag-remove {
  color: var(--fw-wire-fill);
}
"#;
