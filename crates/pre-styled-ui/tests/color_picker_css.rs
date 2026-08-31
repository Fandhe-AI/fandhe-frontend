//! styled ColorPicker（イシュー #839）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/color_swatch_css.rs` の体裁に倣い、`css()` が
//! 返す CSS 全文をバイト単位で固定する。出力順（base のみ、variant なし）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。Area の 2 レイヤーグラデーション・色相スライダー
//! の静的 7 ストップグラデーション・アルファスライダーのチェッカーボード
//! 表現が固定対象の中核（`crates/pre-styled-ui/src/color_picker.rs::recipe`
//! rustdoc 参照）。

use fandhe_frontend_pre_styled_ui::color_picker;

const COLOR_PICKER_GOLDEN_CSS: &str = r#"[data-scope="color-picker"][data-part="root"] {
  display: inline-block;
  position: relative;
}

[data-scope="color-picker"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="color-picker"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope="color-picker"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem));
  height: var(--fandhe-color-picker-trigger-size, var(--fandhe-size-control-height-md, 2.5rem));
  padding: var(--fandhe-space-1);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  cursor: pointer;
  background-image: linear-gradient(var(--fandhe-color-picker-preview, transparent), var(--fandhe-color-picker-preview, transparent)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%), linear-gradient(var(--fandhe-color-bg), var(--fandhe-color-bg));
  background-size: 100% 100%, 8px 8px, 100% 100%;
  background-origin: content-box, content-box, border-box;
  background-clip: content-box, content-box, border-box;
  transition-property: border-color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="color-picker"][data-part="positioner"] {
  position: absolute;
  z-index: 1;
}

[data-scope="color-picker"][data-part="content"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-3);
  padding: var(--fandhe-space-3);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
}

[data-scope="color-picker"][data-part="area"] {
  position: relative;
  width: 12rem;
  height: 8rem;
  border-radius: var(--fandhe-radius-sm);
  overflow: hidden;
  cursor: crosshair;
}

[data-scope="color-picker"][data-part="area-background"] {
  position: absolute;
  inset: 0;
  background-image: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, var(--fandhe-color-picker-hue-color, #ff0000));
}

[data-scope="color-picker"][data-part="area-thumb"] {
  position: absolute;
  left: var(--fandhe-color-picker-x, 0%);
  top: var(--fandhe-color-picker-y, 0%);
  width: 0.9rem;
  height: 0.9rem;
  border-radius: 9999px;
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  transform: translate(-50%, -50%);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="hue-slider"] {
  position: relative;
  width: 12rem;
  height: 0.75rem;
}

[data-scope="color-picker"][data-part="hue-slider-track"] {
  position: absolute;
  inset: 0;
  border-radius: 999px;
  background-image: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
}

[data-scope="color-picker"][data-part="hue-slider-thumb"] {
  position: absolute;
  top: 50%;
  left: var(--fandhe-color-picker-thumb-percent, 0%);
  transform: translate(-50%, -50%);
  width: 1rem;
  height: 1rem;
  border-radius: 9999px;
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="alpha-slider"] {
  position: relative;
  width: 12rem;
  height: 0.75rem;
}

[data-scope="color-picker"][data-part="alpha-slider-track"] {
  position: absolute;
  inset: 0;
  border-radius: 999px;
  background-image: linear-gradient(to right, transparent, var(--fandhe-color-picker-alpha-color, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%);
  background-size: 100% 100%, 8px 8px;
}

[data-scope="color-picker"][data-part="alpha-slider-thumb"] {
  position: absolute;
  top: 50%;
  left: var(--fandhe-color-picker-thumb-percent, 0%);
  transform: translate(-50%, -50%);
  width: 1rem;
  height: 1rem;
  border-radius: 9999px;
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="channel-input"] {
  width: 6rem;
  font-family: monospace;
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope="color-picker"][data-part="value-text"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg);
}

[data-scope="color-picker"][data-part="trigger"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="color-picker"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="color-picker"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="color-picker"][data-part="trigger"]:hover:not([data-disabled]) {
    border-color: var(--fandhe-color-border-emphasized);
  }
}
"#;

#[test]
fn color_picker_css_matches_golden_fixture() {
    assert_eq!(color_picker::css(), COLOR_PICKER_GOLDEN_CSS);
}

#[test]
fn color_picker_css_output_is_deterministic_across_calls() {
    assert_eq!(color_picker::css(), color_picker::css());
}

#[test]
fn color_picker_css_never_contains_style_breakout_sequences() {
    assert!(!color_picker::css().contains('<'));
}
