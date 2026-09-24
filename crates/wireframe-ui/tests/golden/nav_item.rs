//! `nav_item::NAV_ITEM_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5em;
  width: 100%;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-nav-item-label {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-nav-item-counter {
  flex: 0 0 auto;
  padding: 0.125em 0.5em;
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  font-size: 0.875em;
  line-height: 1.4;
}
.fw-wire-nav-item[data-active] {
  background: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-nav-item[data-active] .fw-wire-nav-item-counter {
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
}
.fw-wire-nav-item.fw-wire-vertical {
  flex-direction: column;
  align-items: center;
  gap: 0.25em;
}
"#;
