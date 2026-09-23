//! `breadcrumbs::BREADCRUMBS_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `breadcrumbs::BREADCRUMBS_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-breadcrumbs {
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

#[test]
fn breadcrumbs_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS,
        EXPECTED_CSS
    );
}
