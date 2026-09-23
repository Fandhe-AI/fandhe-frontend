//! `frame::FRAME_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `frame::FRAME_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-frame {
  display: block;
  box-sizing: border-box;
  min-width: 0;
  border: var(--fw-wire-line-width) solid transparent;
  border-radius: var(--fw-wire-radius);
}
.fw-wire-frame.fw-wire-frame-bordered {
  border-color: var(--fw-wire-line);
}
"#;

#[test]
fn frame_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::frame::FRAME_CSS, EXPECTED_CSS);
}
