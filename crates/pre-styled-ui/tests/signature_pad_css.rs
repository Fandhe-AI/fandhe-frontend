//! styled SignaturePad（イシュー #1503）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/qr_code_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → state）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。`docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3
//! が新規追加の必要性を指摘していた「golden 不在 20 部品」の 1 件を埋める。

use fandhe_frontend_pre_styled_ui::signature_pad;

const SIGNATURE_PAD_GOLDEN_CSS: &str = r#"[data-scope="signature-pad"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="signature-pad"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="signature-pad"][data-part="control"] {
  position: relative;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg-muted);
  min-width: 16rem;
  min-height: 8rem;
  cursor: crosshair;
  touch-action: none;
}

[data-scope="signature-pad"][data-part="segment"] {
  display: block;
  width: 100%;
}

[data-scope="signature-pad"][data-part="segment-path"] {
  fill: none;
  stroke: var(--fandhe-color-fg);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

[data-scope="signature-pad"][data-part="guide"] {
  position: absolute;
  left: var(--fandhe-space-3);
  right: var(--fandhe-space-3);
  bottom: var(--fandhe-space-6);
  border-bottom: 1px dashed var(--fandhe-color-border);
}

[data-scope="signature-pad"][data-part="clear-trigger"] {
  align-self: flex-start;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="signature-pad"][data-part="clear-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="signature-pad"][data-part="clear-trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="signature-pad"][data-part="control"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="signature-pad"][data-part="control"][data-readonly] {
  touch-action: auto;
  cursor: default;
}

[data-scope="signature-pad"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope="signature-pad"][data-part="clear-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn signature_pad_stylesheet_matches_golden_fixture() {
    assert_eq!(signature_pad::stylesheet(), SIGNATURE_PAD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(signature_pad::stylesheet(), signature_pad::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = signature_pad::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
