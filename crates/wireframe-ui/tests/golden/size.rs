//! `size::css()` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::size::css()` と突き合わせる。
//! `size::SCALE` という共有値表から実行時に導出されるため、`PARTS` に
//! 登録される他の `&str` リテラル golden よりも実効性が高い。

pub const EXPECTED_CSS: &str = r#".fw-wire-size-xs { --fw-wire-font-size: 0.75rem; --fw-wire-control-size: 1.5rem; }
.fw-wire-size-sm { --fw-wire-font-size: 0.875rem; --fw-wire-control-size: 1.75rem; }
.fw-wire-size-md { --fw-wire-font-size: 1rem; --fw-wire-control-size: 2rem; }
.fw-wire-size-lg { --fw-wire-font-size: 1.125rem; --fw-wire-control-size: 2.5rem; }
.fw-wire-size-xl { --fw-wire-font-size: 1.25rem; --fw-wire-control-size: 3rem; }
"#;
