//! `icon::ICON_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::icon::ICON_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-icon { display: inline-flex; align-items: center; justify-content: center; line-height: 1; vertical-align: middle; color: var(--fw-wire-ink); }
"#;
