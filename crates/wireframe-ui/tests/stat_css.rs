//! `stat::STAT_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `stat::STAT_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-stat {
  display: inline-flex;
  flex-direction: column;
  gap: 0.25em;
  box-sizing: border-box;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-stat-label {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-value {
  font-size: 2em;
  font-weight: 600;
  line-height: 1.1;
}
.fw-wire-stat-delta {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-delta-icon .fw-wire-icon-glyph {
  width: 1em;
  height: 1em;
}
.fw-wire-stat-up {
  font-weight: 600;
  color: var(--fw-wire-ink);
}
"#;

#[test]
fn stat_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::stat::STAT_CSS, EXPECTED_CSS);
}
