//! `nav_item::NAV_ITEM_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `nav_item::NAV_ITEM_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-nav-item {
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

#[test]
fn nav_item_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS,
        EXPECTED_CSS
    );
}
