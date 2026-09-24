//! `icon::ICON_GLYPH_CSS`（`PARTS` 先頭に登録される基盤のグリフ基底
//! class、#2606）の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::icon::ICON_GLYPH_CSS` と突き合わせる。
//! 表示部品としての `icon::ICON_CSS`（`tests/golden/icon.rs`）とは別の
//! 定数である。

pub const EXPECTED_CSS: &str = r#".fw-wire-icon-glyph { display: inline-block; width: 1em; height: 1em; font-size: var(--fw-wire-font-size, 1rem); vertical-align: -0.125em; flex-shrink: 0; }
"#;
