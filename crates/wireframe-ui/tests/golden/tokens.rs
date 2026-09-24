//! `tokens::css()` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::tokens::css()` と突き合わせる。
//! `size::SCALE`/`tokens::TOKENS` という共有値表から実行時に導出される
//! ため、`PARTS` に登録される他の `&str` リテラル golden よりも実効性が
//! 高い（変更検知の主眼はここにある）。

pub const EXPECTED_CSS: &str = r#":root {
  --fw-wire-paper: #ffffff;
  --fw-wire-fill-subtle: #f5f5f5;
  --fw-wire-fill: #e5e5e5;
  --fw-wire-line-subtle: #cccccc;
  --fw-wire-line: #8c8c8c;
  --fw-wire-ink-muted: #595959;
  --fw-wire-ink: #1f1f1f;
  --fw-wire-line-width: 1.5px;
  --fw-wire-radius: 4px;
  --fw-wire-font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
}
"#;
