//! styled AngleSlider の決定的 CSS 出力ゴールデンテスト（イシュー #1445:
//! トラック・サム・マーカーのスタイル調整、イシュー #1446: 値テキストと
//! ラベルの型階層、親トラッキング #1444 の分割 1/2・2/2 の統合後の状態）。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! 期待値は base 取り込み後の `crates/pre-styled-ui/src/angle_slider.rs::recipe`
//! の実出力から生成した（#1445 のトラック・サム・マーカー是正と #1446 の
//! ラベル・値テキスト型階層是正の双方を反映済み）。

use fandhe_frontend_pre_styled_ui::angle_slider;

/// `angle_slider::stylesheet()` の期待値（バイト完全一致）。
///
/// 出力順は `SlotRecipe::css`（`crates/pre-styled-ui/src/recipe.rs`）の
/// 契約どおり「base（登録順: root → root disabled state → label → control →
/// thumb → thumb transition base → value-text）→ variants（登録順:
/// size → color-palette）→ states（登録順: root disabled → thumb disabled →
/// thumb hover → thumb focus-visible）」。
const EXPECTED_CSS: &str = r#"[data-scope="angle-slider"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="angle-slider"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-xs);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

[data-scope="angle-slider"][data-part="control"] {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fandhe-angle-slider-track-size, 4.5rem);
  height: var(--fandhe-angle-slider-track-size, 4.5rem);
  border-radius: var(--fandhe-radius-full);
  background: radial-gradient(circle, var(--fandhe-color-fg-muted) 0 2px, transparent 2px), radial-gradient(circle closest-side, var(--fandhe-color-bg) 0 calc(100% - 6px), transparent calc(100% - 6px)), repeating-conic-gradient(var(--fandhe-color-border) 0deg 1deg, transparent 1deg 30deg), var(--fandhe-color-bg);
  box-shadow: var(--fandhe-shadow-sm);
}

[data-scope="angle-slider"][data-part="thumb"] {
  position: absolute;
  top: 50%;
  left: 50%;
  width: var(--fandhe-angle-slider-thumb-size, 0.9rem);
  height: var(--fandhe-angle-slider-thumb-size, 0.9rem);
  margin-top: calc(var(--fandhe-angle-slider-track-size, 4.5rem) / -2);
  margin-left: calc(var(--fandhe-angle-slider-thumb-size, 0.9rem) / -2);
  transform-origin: calc(var(--fandhe-angle-slider-thumb-size, 0.9rem) / 2) calc(var(--fandhe-angle-slider-track-size, 4.5rem) / 2);
  transform: rotate(var(--fandhe-angle, 0deg));
  border-radius: var(--fandhe-radius-full);
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  --fandhe-hover-bg: var(--fandhe-palette-emphasized);
  box-sizing: border-box;
  cursor: pointer;
}

[data-scope="angle-slider"][data-part="thumb"] {
  transition-property: background, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="angle-slider"][data-part="value-text"] {
  font-size: var(--fandhe-angle-slider-value-font-size, var(--fandhe-font-font-size-lg));
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  color: var(--fandhe-color-fg);
  font-variant-numeric: tabular-nums;
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--size-xs {
  --fandhe-angle-slider-track-size: 2.5rem;
  --fandhe-angle-slider-thumb-size: 0.5rem;
  --fandhe-angle-slider-value-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--size-sm {
  --fandhe-angle-slider-track-size: 3.5rem;
  --fandhe-angle-slider-thumb-size: 0.7rem;
  --fandhe-angle-slider-value-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--size-md {
  --fandhe-angle-slider-track-size: 4.5rem;
  --fandhe-angle-slider-thumb-size: 0.9rem;
  --fandhe-angle-slider-value-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--size-lg {
  --fandhe-angle-slider-track-size: 5.5rem;
  --fandhe-angle-slider-thumb-size: 1.1rem;
  --fandhe-angle-slider-value-font-size: var(--fandhe-font-font-size-xl);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--size-xl {
  --fandhe-angle-slider-track-size: 6.5rem;
  --fandhe-angle-slider-thumb-size: 1.3rem;
  --fandhe-angle-slider-value-font-size: var(--fandhe-font-font-size-2xl);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"].fd-angle-slider--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="angle-slider"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="angle-slider"][data-part="thumb"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="angle-slider"][data-part="thumb"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="angle-slider"][data-part="thumb"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn angle_slider_css_matches_golden_byte_for_byte() {
    assert_eq!(
        angle_slider::stylesheet(),
        EXPECTED_CSS,
        "angle_slider::stylesheet() の出力が golden と一致しない。意図した \
         宣言変更なら EXPECTED_CSS を更新すること（本ファイル冒頭 rustdoc 参照）"
    );
}

#[test]
fn angle_slider_css_is_deterministic() {
    assert_eq!(angle_slider::stylesheet(), angle_slider::stylesheet());
}

#[test]
fn angle_slider_css_never_contains_style_breakout_sequences() {
    let css = angle_slider::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
