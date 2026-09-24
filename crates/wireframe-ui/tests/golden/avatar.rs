//! `avatar::AVATAR_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::avatar::AVATAR_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: var(--fw-wire-control-size, 2rem);
  height: var(--fw-wire-control-size, 2rem);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1;
}
.fw-wire-avatar.fw-wire-avatar-circle {
  border-radius: 50%;
}
.fw-wire-avatar .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-control-size, 2rem) * 0.6);
}
"#;
