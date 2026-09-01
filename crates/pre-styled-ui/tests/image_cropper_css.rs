//! styled ImageCropper（イシュー #844）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs`/`slider_css.rs`（存在すれば）の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden テスト
//! が即座に検知する。
//!
//! イシュー #1481（親 #1479 の 2/2、分割 1/2 は #1480）で `viewport`/
//! `image` パートへ角丸・背景・操作性宣言を追加したため、この golden
//! 期待値も同一 PR 内で更新した（`crates/pre-styled-ui/src/image_cropper.rs`
//! モジュール冒頭 rustdoc「イシュー #1481」節参照）。

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
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg-muted);
  touch-action: none;
  user-select: none;
}

[data-scope="image-cropper"][data-part="image"] {
  display: block;
  max-width: 100%;
  user-select: none;
}

[data-scope="image-cropper"][data-part="selection"] {
  position: absolute;
  left: var(--fandhe-image-cropper-x, 0%);
  top: var(--fandhe-image-cropper-y, 0%);
  width: var(--fandhe-image-cropper-w, 100%);
  height: var(--fandhe-image-cropper-h, 100%);
  box-sizing: border-box;
  border: 1px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 0 0 9999px var(--fandhe-color-bg-overlay);
  cursor: move;
}

[data-scope="image-cropper"][data-part="handle"] {
  position: absolute;
  width: var(--fandhe-image-cropper-handle-size, 0.75rem);
  height: var(--fandhe-image-cropper-handle-size, 0.75rem);
  background: #ffffff;
  border: 1px solid rgba(0, 0, 0, 0.25);
  border-radius: var(--fandhe-radius-xs);
  box-shadow: var(--fandhe-shadow-sm);
  box-sizing: border-box;
  transform: translate(-50%, -50%);
}

[data-scope="image-cropper"][data-part="handle"] {
  transition-property: background, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="image-cropper"][data-part="grid"] {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5)), linear-gradient(rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5));
  background-size: 1px 100%, 1px 100%, 100% 1px, 100% 1px;
  background-position: calc(100% / 3) 0, calc(100% / 3 * 2) 0, 0 calc(100% / 3), 0 calc(100% / 3 * 2);
  background-repeat: no-repeat;
}

[data-scope="image-cropper"][data-part="root"].fd-image-cropper--size-xs {
  --fandhe-image-cropper-handle-size: 0.35rem;
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

[data-scope="image-cropper"][data-part="root"].fd-image-cropper--size-xl {
  --fandhe-image-cropper-handle-size: 1.15rem;
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
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="image-cropper"][data-part="handle"]:hover:not([data-disabled]) {
    background: #f0f0f0;
  }
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(image_cropper::stylesheet(), IMAGE_CROPPER_GOLDEN_CSS);
}
