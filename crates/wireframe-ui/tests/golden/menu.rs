//! `menu::MENU_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::menu::MENU_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-menu {
  display: inline-flex;
  flex-direction: column;
  box-sizing: border-box;
  min-width: 12em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-menu-search {
  display: flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-menu-search-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-menu .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-menu-item {
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-menu-item[data-active] {
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
}
.fw-wire-menu-item[data-disabled] {
  color: var(--fw-wire-ink-muted);
  opacity: 0.6;
}
"#;
