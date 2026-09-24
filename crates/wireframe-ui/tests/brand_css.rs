//! `brand::BRAND_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `brand::BRAND_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-brand {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: var(--fw-wire-control-size, 2rem);
  height: var(--fw-wire-control-size, 2rem);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1;
}
.fw-wire-brand .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-control-size, 2rem) * 0.6);
}
"#;

#[test]
fn brand_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::brand::BRAND_CSS, EXPECTED_CSS);
}
