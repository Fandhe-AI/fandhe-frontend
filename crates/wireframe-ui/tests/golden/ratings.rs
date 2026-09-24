//! `ratings::RATINGS_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-ratings {
  display: inline-flex;
  align-items: center;
  gap: calc(var(--fw-wire-control-size, 2rem) * 0.1);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-line);
}
.fw-wire-ratings-star {
  display: inline-flex;
  line-height: 0;
}
.fw-wire-ratings-star[data-active] {
  color: var(--fw-wire-ink);
}
.fw-wire-ratings-star[data-active] .fw-wire-icon-glyph {
  fill: currentColor;
}
"#;
