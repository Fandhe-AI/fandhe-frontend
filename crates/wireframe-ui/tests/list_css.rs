//! `list::LIST_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `list::LIST_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-list {
  display: flex;
  flex-direction: column;
  gap: 0.5em;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-list > .fw-wire-list-item {
  display: flex;
  align-items: baseline;
  gap: 0.5em;
}
.fw-wire-list > .fw-wire-list-item::before {
  content: "";
  flex-shrink: 0;
  align-self: flex-start;
  margin-top: 0.575em;
  width: 0.35em;
  height: 0.35em;
  border-radius: 50%;
  background: var(--fw-wire-ink-muted);
}
.fw-wire-list > .fw-wire-list-item .fw-wire-list {
  margin-left: 1.5em;
  margin-top: 0.25em;
}
.fw-wire-list.fw-wire-list-ordered {
  counter-reset: fw-wire-list;
}
.fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item {
  counter-increment: fw-wire-list;
}
.fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item::before {
  content: counter(fw-wire-list) ".";
  align-self: baseline;
  margin-top: 0;
  width: auto;
  height: auto;
  border-radius: 0;
  background: none;
  color: var(--fw-wire-ink-muted);
  font-variant-numeric: tabular-nums;
}
"#;

#[test]
fn list_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::list::LIST_CSS, EXPECTED_CSS);
}
