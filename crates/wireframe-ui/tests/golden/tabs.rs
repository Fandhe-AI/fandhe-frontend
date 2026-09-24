//! `tabs::TABS_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::tabs::TABS_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-tabs {
  display: flex;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-horizontal {
  flex-direction: row;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs.fw-wire-vertical {
  display: inline-flex;
  flex-direction: column;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs-item {
  box-sizing: border-box;
  padding: 0.5em 1em;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  border-bottom: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item {
  border-bottom: none;
  border-right: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-bottom-color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item[data-active] {
  border-bottom-color: transparent;
  border-right-color: var(--fw-wire-ink);
}
"#;
