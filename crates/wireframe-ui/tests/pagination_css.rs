//! `pagination::PAGINATION_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `pagination::PAGINATION_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-pagination {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-pagination-item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 2em;
  height: 2em;
  padding: 0 0.5em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink-muted);
  white-space: nowrap;
}
.fw-wire-pagination-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-pagination-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-pagination-gap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
"#;

#[test]
fn pagination_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS,
        EXPECTED_CSS
    );
}
