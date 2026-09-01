//! styled RatingGroup（イシュー #742）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs`/`number_input_css.rs` の golden
//! fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で
//! 固定する。出力順（base → variants → compound → states）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::rating_group;

const RATING_GROUP_GOLDEN_CSS: &str = r#"[data-scope="rating-group"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
}

[data-scope="rating-group"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-rating-group-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="rating-group"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="rating-group"][data-part="item"] {
  clip-path: polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%);
  width: var(--fandhe-rating-group-item-size, 1.25rem);
  height: var(--fandhe-rating-group-item-size, 1.25rem);
  display: inline-block;
  background: var(--fandhe-color-border);
  cursor: pointer;
  flex-shrink: 0;
}

[data-scope="rating-group"][data-part="item"] {
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="rating-group"][data-part="hidden-input"] {
  display: none;
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--size-xs {
  --fandhe-rating-group-item-size: 0.75rem;
  --fandhe-rating-group-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--size-sm {
  --fandhe-rating-group-item-size: 1rem;
  --fandhe-rating-group-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--size-md {
  --fandhe-rating-group-item-size: 1.25rem;
  --fandhe-rating-group-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--size-lg {
  --fandhe-rating-group-item-size: 1.5rem;
  --fandhe-rating-group-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--size-xl {
  --fandhe-rating-group-item-size: 1.75rem;
  --fandhe-rating-group-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="rating-group"][data-part="root"].fd-rating-group--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="rating-group"][data-part="item"][data-highlighted] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="rating-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="rating-group"][data-part="item"][data-readonly] {
  cursor: default;
}

[data-scope="rating-group"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}
"#;

#[test]
fn rating_group_stylesheet_matches_golden_fixture() {
    assert_eq!(rating_group::stylesheet(), RATING_GROUP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(rating_group::stylesheet(), rating_group::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = rating_group::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_never_references_external_resources() {
    // 星形 indicator は clip-path によるインライン表現であり、SVG ファイル・
    // icon font・画像 URL を一切参照しない（イシュー #742 受け入れ条件）。
    let css = rating_group::stylesheet();
    assert!(!css.contains("url("));
    assert!(css.contains("clip-path: polygon("));
}
