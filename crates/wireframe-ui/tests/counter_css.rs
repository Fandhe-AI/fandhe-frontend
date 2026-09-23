//! `counter::COUNTER_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `counter::COUNTER_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-counter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 1.5em;
  height: 1.5em;
  padding: 0 0.4em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  font-variant-numeric: tabular-nums;
  line-height: 1;
  white-space: nowrap;
}
.fw-wire-counter.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
"#;

#[test]
fn counter_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::counter::COUNTER_CSS,
        EXPECTED_CSS
    );
}
