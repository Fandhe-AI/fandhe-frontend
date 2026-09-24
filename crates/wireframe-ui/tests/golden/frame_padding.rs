//! `frame::frame_padding_css()` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::frame::frame_padding_css()` と
//! 突き合わせる。`size::SCALE` という共有値表から実行時に導出される
//! ため、`PARTS` に登録される他の `&str` リテラル golden よりも実効性が
//! 高い。

pub const EXPECTED_CSS: &str = r#".fw-wire-frame.fw-wire-frame-padding-xs {
  padding: calc(1.5rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-sm {
  padding: calc(1.75rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-md {
  padding: calc(2rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-lg {
  padding: calc(2.5rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-xl {
  padding: calc(3rem / 2);
}
"#;
