//! `link::LINK_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `link::LINK_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-link {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-link-label {
  text-decoration: underline;
  text-decoration-color: var(--fw-wire-line);
  text-underline-offset: 0.15em;
}
.fw-wire-link.fw-wire-bold {
  font-weight: 600;
}
"#;

#[test]
fn link_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::link::LINK_CSS, EXPECTED_CSS);
}
