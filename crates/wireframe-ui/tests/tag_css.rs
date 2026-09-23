//! `tag::TAG_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `tag::TAG_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.125em 0.625em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-tag-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-tag-remove {
  display: inline-flex;
  align-items: center;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-tag.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-tag.fw-wire-primary .fw-wire-tag-remove {
  color: var(--fw-wire-fill);
}
"#;

#[test]
fn tag_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::tag::TAG_CSS, EXPECTED_CSS);
}
