//! `image::IMAGE_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `image::IMAGE_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-image {
  --fw-wire-image-x-color: var(--fw-wire-line);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-control-size, 2rem) * 3);
  height: calc(var(--fw-wire-control-size, 2rem) * 3);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-image.fw-wire-image-circle {
  border-radius: 50%;
}
.fw-wire-image.fw-wire-image-placeholder {
  background-image:
    linear-gradient(to top right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2)),
    linear-gradient(to bottom right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2));
}
.fw-wire-image.fw-wire-primary {
  --fw-wire-image-x-color: var(--fw-wire-paper);
  color: var(--fw-wire-image-x-color);
  background-color: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
}
"#;

#[test]
fn image_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::image::IMAGE_CSS, EXPECTED_CSS);
}
