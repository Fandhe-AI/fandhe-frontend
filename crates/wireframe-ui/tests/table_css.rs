//! `table::TABLE_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `table::TABLE_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-table {
  display: block;
  box-sizing: border-box;
  width: 100%;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  overflow: hidden;
}
.fw-wire-table-header {
  background: var(--fw-wire-fill-subtle);
  font-weight: 600;
}
.fw-wire-table-body {
  display: block;
}
.fw-wire-table-row {
  display: grid;
  border-top: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
}
.fw-wire-table-header .fw-wire-table-row {
  border-top: none;
}
.fw-wire-table-body:first-child .fw-wire-table-row:first-child {
  border-top: none;
}
.fw-wire-table-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.fw-wire-table-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.fw-wire-table-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.fw-wire-table-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.fw-wire-table-cols-5 { grid-template-columns: repeat(5, minmax(0, 1fr)); }
.fw-wire-table-cols-6 { grid-template-columns: repeat(6, minmax(0, 1fr)); }
.fw-wire-table-cols-7 { grid-template-columns: repeat(7, minmax(0, 1fr)); }
.fw-wire-table-cols-8 { grid-template-columns: repeat(8, minmax(0, 1fr)); }
.fw-wire-table-cols-9 { grid-template-columns: repeat(9, minmax(0, 1fr)); }
.fw-wire-table-cols-10 { grid-template-columns: repeat(10, minmax(0, 1fr)); }
.fw-wire-table-cols-11 { grid-template-columns: repeat(11, minmax(0, 1fr)); }
.fw-wire-table-cols-12 { grid-template-columns: repeat(12, minmax(0, 1fr)); }
.fw-wire-table-cell {
  min-width: 0;
  box-sizing: border-box;
  padding: 0.4em 0.6em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-table-cell-empty {
  min-height: 1em;
}
"#;

#[test]
fn table_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::table::TABLE_CSS, EXPECTED_CSS);
}
