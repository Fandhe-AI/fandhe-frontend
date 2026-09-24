//! `breadcrumbs::BREADCRUMBS_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-breadcrumbs {
  display: flex;
  flex-wrap: nowrap;
  overflow-x: auto;
  align-items: center;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink-muted);
}
.fw-wire-breadcrumbs-item {
  box-sizing: border-box;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item {
  margin-left: 0.4em;
}
.fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item::before {
  content: "/";
  display: inline-block;
  margin-right: 0.4em;
  color: var(--fw-wire-line);
}
.fw-wire-breadcrumbs-item[data-active] {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-breadcrumbs-item[data-active]::before {
  font-weight: normal;
}
"#;
