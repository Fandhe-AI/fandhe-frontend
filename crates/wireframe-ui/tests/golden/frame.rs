//! `frame::FRAME_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::frame::FRAME_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-frame {
  display: block;
  box-sizing: border-box;
  min-width: 0;
  border: var(--fw-wire-line-width) solid transparent;
  border-radius: var(--fw-wire-radius);
}
.fw-wire-frame.fw-wire-frame-bordered {
  border-color: var(--fw-wire-line);
}
"#;
