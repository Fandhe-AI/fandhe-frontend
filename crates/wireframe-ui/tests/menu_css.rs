//! `menu::MENU_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `menu::MENU_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-menu {
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

#[test]
fn menu_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::menu::MENU_CSS, EXPECTED_CSS);
}
