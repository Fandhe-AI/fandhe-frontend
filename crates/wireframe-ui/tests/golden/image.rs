//! `image::IMAGE_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::image::IMAGE_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-image {
  --fw-wire-image-x-color: var(--fw-wire-line);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-control-size, 2rem) * 3);
  height: calc(var(--fw-wire-control-size, 2rem) * 3);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-image.fw-wire-image-circle {
  border-radius: 50%;
}
.fw-wire-image.fw-wire-image-placeholder {
  background-image:
    linear-gradient(to top right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2)),
    linear-gradient(to bottom right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2));
}
.fw-wire-image.fw-wire-primary {
  --fw-wire-image-x-color: var(--fw-wire-paper);
  color: var(--fw-wire-image-x-color);
  background-color: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
}
"#;
