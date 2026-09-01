//! styled Slider の決定的 CSS 出力ゴールデンテスト（イシュー #1505:
//! トラック・レンジ・サムのスタイル調整、親トラッキング #1504 の 1/2）。
//!
//! `crates/pre-styled-ui/tests/angle_slider_css.rs`（イシュー #1445/#1446、
//! 親 #1444）の golden fixture テストの前例に倣い、`stylesheet()` が返す
//! CSS 全文をバイト単位で固定する。出力順（base → variants → compound →
//! states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。
//!
//! 期待値は `crates/pre-styled-ui/src/slider.rs::recipe` の実出力から
//! 生成した（#1505 のトラック/レンジ/サムの是正を反映済み）。marker/
//! label/value-text の是正・orientation 状態規則の再設計は姉妹イシュー
//! #1506（親 #1504 の 2/2）の担当のため、本 golden は #1505 完了時点の
//! 出力を固定する。

use fandhe_frontend_pre_styled_ui::slider;

/// `slider::stylesheet()` の期待値（バイト完全一致）。
///
/// 出力順は `SlotRecipe::css`（`crates/pre-styled-ui/src/recipe.rs`）の
/// 契約どおり「base（登録順: root → root disabled state → label → control
/// → control vertical state → track → track vertical state → range →
/// range vertical state → thumb → thumb vertical state → thumb disabled
/// state → thumb transition base）→ variants（登録順: size → color-palette）
/// → states（登録順: root disabled → control vertical → track vertical →
/// range vertical → thumb vertical → thumb disabled → thumb focus-visible
/// → thumb hover）」。
const EXPECTED_CSS: &str = r#"[data-scope="slider"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="slider"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="slider"][data-part="control"] {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: var(--fandhe-slider-track-length, 12rem);
}

[data-scope="slider"][data-part="track"] {
  position: relative;
  width: 100%;
  height: var(--fandhe-slider-track-height, 0.375rem);
  border-radius: var(--fandhe-radius-full, 999px);
  background: var(--fandhe-color-border);
}

[data-scope="slider"][data-part="range"] {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: var(--fandhe-slider-percent, 0%);
  border-radius: var(--fandhe-radius-full, 999px);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="slider"][data-part="thumb"] {
  position: absolute;
  top: 50%;
  left: var(--fandhe-slider-percent, 0%);
  transform: translate(-50%, -50%);
  width: var(--fandhe-slider-thumb-size, 1.1rem);
  height: var(--fandhe-slider-thumb-size, 1.1rem);
  border-radius: var(--fandhe-radius-full, 999px);
  background: var(--fandhe-color-bg);
  border: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: var(--fandhe-shadow-sm);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  box-sizing: border-box;
  cursor: pointer;
}

[data-scope="slider"][data-part="thumb"] {
  transition-property: background, border-color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="slider"][data-part="root"].fd-slider--size-xs {
  --fandhe-slider-track-height: 0.125rem;
  --fandhe-slider-thumb-size: 0.6rem;
}

[data-scope="slider"][data-part="root"].fd-slider--size-sm {
  --fandhe-slider-track-height: 0.25rem;
  --fandhe-slider-thumb-size: 0.85rem;
}

[data-scope="slider"][data-part="root"].fd-slider--size-md {
  --fandhe-slider-track-height: 0.375rem;
  --fandhe-slider-thumb-size: 1.1rem;
}

[data-scope="slider"][data-part="root"].fd-slider--size-lg {
  --fandhe-slider-track-height: 0.5rem;
  --fandhe-slider-thumb-size: 1.35rem;
}

[data-scope="slider"][data-part="root"].fd-slider--size-xl {
  --fandhe-slider-track-height: 0.625rem;
  --fandhe-slider-thumb-size: 1.6rem;
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="slider"][data-part="root"].fd-slider--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="slider"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="slider"][data-part="control"][data-orientation="vertical"] {
  width: auto;
  height: var(--fandhe-slider-track-length, 12rem);
}

[data-scope="slider"][data-part="track"][data-orientation="vertical"] {
  width: var(--fandhe-slider-track-height, 0.375rem);
  height: 100%;
}

[data-scope="slider"][data-part="range"][data-orientation="vertical"] {
  top: auto;
  bottom: 0;
  left: 0;
  width: 100%;
  height: var(--fandhe-slider-percent, 0%);
}

[data-scope="slider"][data-part="thumb"][data-orientation="vertical"] {
  top: var(--fandhe-slider-percent, 0%);
  left: 50%;
  bottom: auto;
}

[data-scope="slider"][data-part="thumb"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="slider"][data-part="thumb"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="slider"][data-part="thumb"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn slider_css_matches_golden_byte_for_byte() {
    assert_eq!(
        slider::stylesheet(),
        EXPECTED_CSS,
        "slider::stylesheet() の出力が golden と一致しない。意図した \
         宣言変更なら EXPECTED_CSS を更新すること（本ファイル冒頭 rustdoc 参照）"
    );
}

#[test]
fn slider_css_is_deterministic() {
    assert_eq!(slider::stylesheet(), slider::stylesheet());
}

#[test]
fn slider_css_never_contains_style_breakout_sequences() {
    let css = slider::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
