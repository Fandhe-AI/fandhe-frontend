//! `emoji::EMOJI_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `emoji::EMOJI_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-emoji {
  display: inline-block;
  line-height: 1;
  white-space: nowrap;
  vertical-align: -0.125em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  filter: grayscale(1);
}
.fw-wire-emoji:empty {
  width: 1em;
  height: 1em;
  box-sizing: border-box;
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line);
  border-radius: 50%;
}
"#;

#[test]
fn emoji_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS, EXPECTED_CSS);
}
