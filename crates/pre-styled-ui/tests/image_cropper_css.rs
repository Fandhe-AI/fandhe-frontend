//! styled ImageCropper（イシュー #844）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs`/`slider_css.rs`（存在すれば）の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden テスト
//! が即座に検知する。

use fandhe_frontend_pre_styled_ui::image_cropper;

const IMAGE_CROPPER_GOLDEN_CSS: &str = r#"[data-scope="image-cropper"][data-part="root"] {
  display: inline-block;
  position: relative;
}

[data-scope="image-cropper"][data-part="viewport"] {
  position: relative;
  overflow: hidden;
  display: block;
  width: 100%;
  height: 100%;
}

[data-scope="image-cropper"][data-part="image"] {
  display: block;
  max-width: 100%;
}

[data-scope="image-cropper"][data-part="selection"] {
  position: absolute;
  left: var(--fandhe-image-cropper-x, 0%);
  top: var(--fandhe-image-cropper-y, 0%);
  width: var(--fandhe-image-cropper-w, 100%);
  height: var(--fandhe-image-cropper-h, 100%);
  box-sizing: border-box;
  border: 2px solid var(--fandhe-color-bg);
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
  cursor: move;
}

[data-scope="image-cropper"][data-part="handle"] {
  position: absolute;
  width: var(--fandhe-image-cropper-handle-size, 0.75rem);
  height: var(--fandhe-image-cropper-handle-size, 0.75rem);
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  box-sizing: border-box;
  transform: translate(-50%, -50%);
}

[data-scope="image-cropper"][data-part="grid"] {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: linear-gradient(to right, rgba(255, 255, 255, 0.5) 1px, transparent 1px), linear-gradient(to bottom, rgba(255, 255, 255, 0.5) 1px, transparent 1px);
  background-size: calc(100% / 3) 100%, 100% calc(100% / 3);
}

[data-scope="image-cropper"][data-part="root"].fd-image-cropper--size-sm {
  --fandhe-image-cropper-handle-size: 0.55rem;
}

[data-scope="image-cropper"][data-part="root"].fd-image-cropper--size-md {
  --fandhe-image-cropper-handle-size: 0.75rem;
}

[data-scope="image-cropper"][data-part="root"].fd-image-cropper--size-lg {
  --fandhe-image-cropper-handle-size: 0.95rem;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="n"] {
  top: 0;
  left: 50%;
  cursor: ns-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="s"] {
  top: 100%;
  left: 50%;
  cursor: ns-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="e"] {
  top: 50%;
  left: 100%;
  cursor: ew-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="w"] {
  top: 50%;
  left: 0;
  cursor: ew-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="ne"] {
  top: 0;
  left: 100%;
  cursor: nesw-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="nw"] {
  top: 0;
  left: 0;
  cursor: nwse-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="se"] {
  top: 100%;
  left: 100%;
  cursor: nwse-resize;
}

[data-scope="image-cropper"][data-part="handle"][data-handle-position="sw"] {
  top: 100%;
  left: 0;
  cursor: nesw-resize;
}

[data-scope="image-cropper"][data-part="handle"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 1px;
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(image_cropper::stylesheet(), IMAGE_CROPPER_GOLDEN_CSS);
}
