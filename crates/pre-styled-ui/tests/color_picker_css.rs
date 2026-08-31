//! styled ColorPicker（イシュー #839、状態表現の是正はイシュー #1464）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/color_swatch_css.rs` の体裁に倣い、`css()` が
//! 返す CSS 全文をバイト単位で固定する。出力順（base → `.state(...)` 登録順）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。Area の 2 レイヤーグラデーション・色相スライダー
//! の静的 7 ストップグラデーション・アルファスライダーのチェッカーボード
//! 表現に加え、サム 3 slot（`area-thumb`/`hue-slider-thumb`/
//! `alpha-slider-thumb`）の `[data-disabled]`/`:hover`/`:focus-visible`/
//! `transition-property` が固定対象の中核（`crates/pre-styled-ui/src/
//! color_picker.rs::recipe` rustdoc 参照）。

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
  display: inline-block;
  width: 1.75rem;
  height: 1.75rem;
  padding: 0;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  background-image: linear-gradient(var(--fandhe-color-picker-preview, #000), var(--fandhe-color-picker-preview, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%);
  background-size: 100% 100%, 8px 8px;
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
  width: var(--fandhe-color-picker-thumb-size, 1rem);
  height: var(--fandhe-color-picker-thumb-size, 1rem);
  border-radius: var(--fandhe-radius-full);
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  transform: translate(-50%, -50%);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="area-thumb"] {
  transition-property: box-shadow, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="color-picker"][data-part="hue-slider"] {
  position: relative;
  width: 12rem;
  height: 0.75rem;
}

[data-scope="color-picker"][data-part="hue-slider-track"] {
  position: absolute;
  inset: 0;
  border-radius: var(--fandhe-radius-full);
  background-image: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
}

[data-scope="color-picker"][data-part="hue-slider-thumb"] {
  position: absolute;
  top: 50%;
  left: var(--fandhe-color-picker-thumb-percent, 0%);
  transform: translate(-50%, -50%);
  width: var(--fandhe-color-picker-thumb-size, 1rem);
  height: var(--fandhe-color-picker-thumb-size, 1rem);
  border-radius: var(--fandhe-radius-full);
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="hue-slider-thumb"] {
  transition-property: box-shadow, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="color-picker"][data-part="alpha-slider"] {
  position: relative;
  width: 12rem;
  height: 0.75rem;
}

[data-scope="color-picker"][data-part="alpha-slider-track"] {
  position: absolute;
  inset: 0;
  border-radius: var(--fandhe-radius-full);
  background-image: linear-gradient(to right, transparent, var(--fandhe-color-picker-alpha-color, #000)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%);
  background-size: 100% 100%, 8px 8px;
}

[data-scope="color-picker"][data-part="alpha-slider-thumb"] {
  position: absolute;
  top: 50%;
  left: var(--fandhe-color-picker-thumb-percent, 0%);
  transform: translate(-50%, -50%);
  width: var(--fandhe-color-picker-thumb-size, 1rem);
  height: var(--fandhe-color-picker-thumb-size, 1rem);
  border-radius: var(--fandhe-radius-full);
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  background: transparent;
  cursor: pointer;
}

[data-scope="color-picker"][data-part="alpha-slider-thumb"] {
  transition-property: box-shadow, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
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

[data-scope="color-picker"][data-part="area-thumb"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="color-picker"][data-part="area-thumb"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope="color-picker"][data-part="hue-slider-thumb"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="color-picker"][data-part="hue-slider-thumb"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="color-picker"][data-part="alpha-slider-thumb"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="color-picker"][data-part="alpha-slider-thumb"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="color-picker"][data-part="area-thumb"]:hover:not([data-disabled]) {
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.45);
  }

  [data-scope="color-picker"][data-part="hue-slider-thumb"]:hover:not([data-disabled]) {
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.45);
  }

  [data-scope="color-picker"][data-part="alpha-slider-thumb"]:hover:not([data-disabled]) {
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.45);
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
