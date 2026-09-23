//! `card_basic::CARD_BASIC_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `card_basic::CARD_BASIC_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-card-basic {
  display: flex;
  align-items: center;
  gap: 0.75em;
  box-sizing: border-box;
  min-width: 0;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-card-basic-body {
  display: flex;
  flex: 1 1 auto;
  min-width: 0;
  flex-direction: column;
}
.fw-wire-card-basic-primary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-card-basic-secondary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  font-size: 0.875em;
}
"#;

#[test]
fn card_basic_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS,
        EXPECTED_CSS
    );
}
