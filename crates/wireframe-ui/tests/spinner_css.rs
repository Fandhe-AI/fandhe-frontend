//! `spinner::SPINNER_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `spinner::SPINNER_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-spinner {
  display: inline-block;
  box-sizing: border-box;
  width: var(--fw-wire-control-size);
  height: var(--fw-wire-control-size);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-top-color: var(--fw-wire-ink);
  border-radius: 50%;
  vertical-align: middle;
  flex-shrink: 0;
}
"#;

#[test]
fn spinner_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS,
        EXPECTED_CSS
    );
}
