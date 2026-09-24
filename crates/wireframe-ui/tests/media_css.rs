//! `media::MEDIA_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `media::MEDIA_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-media {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 100%;
  aspect-ratio: 16 / 9;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  overflow: hidden;
  --fw-wire-media-control-size: var(--fw-wire-control-size, 2rem);
}
.fw-wire-media .fw-wire-media-disc {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-media-control-size) * 1.5);
  height: calc(var(--fw-wire-media-control-size) * 1.5);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-media .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-media-control-size) * 0.6);
}
"#;

#[test]
fn media_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::media::MEDIA_CSS, EXPECTED_CSS);
}
