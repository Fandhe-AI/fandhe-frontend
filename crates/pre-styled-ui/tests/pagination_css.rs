//! styled Pagination（イシュー #751、`docs/api/headless-ui-api.md` §4b.3
//! の保留解除）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/radio_group_css.rs`/`toggle_group_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

//!
//! イシュー #3136: `PAGINATION_GOLDEN_CSS_BEFORE_3136` は本イシュー直前
//! （attached variant 追加前、`PAGINATION_GOLDEN_CSS` と同一）の golden を
//! 保持し、`tab_nav_css.rs` の `..._pre_3126_golden` と同型の「純追加の
//! 固定」テストを持つ。

use fandhe_frontend_pre_styled_ui::pagination;

/// イシュー #3136 直前（attached variant 追加前）の golden 出力。本イシューが
/// 「純追加のみ」であることを固定するための比較対象であり、
/// `pagination::stylesheet()` の戻り値としては使わない。
const PAGINATION_GOLDEN_CSS_BEFORE_3136: &str = r#"[data-scope="pagination"][data-part="root"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="pagination"][data-part="item"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="item"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="ellipsis"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="pagination"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  padding-inline: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="prev-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  padding-inline: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="next-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-xs {
  --fandhe-pagination-item-size: 1rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-sm {
  --fandhe-pagination-item-size: 1.5rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-md {
  --fandhe-pagination-item-size: 2rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-lg {
  --fandhe-pagination-item-size: 2.5rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-xl {
  --fandhe-pagination-item-size: 3rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="pagination"][data-part="item"][data-selected] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="pagination"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="pagination"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="pagination"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="pagination"][data-part="item"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="pagination"][data-part="prev-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="pagination"][data-part="next-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

/// イシュー #3136 時点（attached variant 追加後）の golden 出力。
/// `pagination::stylesheet()` の戻り値はこちらと一致する。
const PAGINATION_GOLDEN_CSS: &str = r#"[data-scope="pagination"][data-part="root"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="pagination"][data-part="item"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="item"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="ellipsis"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="pagination"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  padding-inline: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="prev-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: var(--fandhe-pagination-item-size, 2rem);
  height: var(--fandhe-pagination-item-size, 2rem);
  padding-inline: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm));
  text-decoration: none;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="pagination"][data-part="next-trigger"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-xs {
  --fandhe-pagination-item-size: 1rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-sm {
  --fandhe-pagination-item-size: 1.5rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-md {
  --fandhe-pagination-item-size: 2rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-lg {
  --fandhe-pagination-item-size: 2.5rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="pagination"][data-part="root"].fd-pagination--size-xl {
  --fandhe-pagination-item-size: 3rem;
  --fandhe-pagination-item-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached {
  gap: 0;
}

[data-scope="pagination"][data-part="item"][data-selected] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="pagination"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="pagination"][data-part="item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="pagination"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="pagination"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="pagination"][data-part="item"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="pagination"][data-part="prev-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="pagination"][data-part="next-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="item"]:not(:first-child) {
  margin-inline-start: -1px;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="item"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="item"]:focus-visible {
  position: relative;
  z-index: 2;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="ellipsis"]:not(:first-child) {
  margin-inline-start: -1px;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="ellipsis"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="ellipsis"]:focus-visible {
  position: relative;
  z-index: 2;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="prev-trigger"]:not(:first-child) {
  margin-inline-start: -1px;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="prev-trigger"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="prev-trigger"]:focus-visible {
  position: relative;
  z-index: 2;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="next-trigger"]:not(:first-child) {
  margin-inline-start: -1px;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="next-trigger"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="next-trigger"]:focus-visible {
  position: relative;
  z-index: 2;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="item"][data-selected] {
  position: relative;
  z-index: 1;
}

[data-scope="pagination"][data-part="root"].fd-pagination--variant-attached > [data-scope="pagination"][data-part="ellipsis"] {
  box-sizing: border-box;
  border: 1px solid var(--fandhe-color-border);
  background: var(--fandhe-color-bg);
}
"#;

#[test]
fn pagination_stylesheet_matches_golden_fixture() {
    assert_eq!(pagination::stylesheet(), PAGINATION_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // radio_group_css.rs / switch_css.rs と同観点: 独立呼び出し間でバイト
    // 単位の一致を固定する。
    assert_eq!(pagination::stylesheet(), pagination::stylesheet());
}

/// イシュー #3136: 「純追加の固定」テスト（`tab_nav_css.rs` の
/// `..._pre_3126_golden` と同型）。本イシュー直前の golden
/// （`PAGINATION_GOLDEN_CSS_BEFORE_3136`）が `\n\n` 区切りで持つ各ブロックが、
/// 現行の `pagination::stylesheet()` の中に元の相対順序どおり全て含まれる
/// ことを固定する。既存ブロックの削除・改変・順序入れ替えのいずれかが
/// 起きた場合にこのテストが検知する（新規ブロックの追加だけは許容する）。
#[test]
fn stylesheet_is_pure_addition_over_pre_3136_golden() {
    let before_blocks: Vec<&str> = PAGINATION_GOLDEN_CSS_BEFORE_3136
        .trim_end_matches('\n')
        .split("\n\n")
        .collect();
    let current = pagination::stylesheet();

    let mut search_from = 0usize;
    for block in before_blocks {
        let needle = if block.starts_with("@media") {
            block.strip_suffix("\n}").unwrap_or(block)
        } else {
            block
        };
        let found = current[search_from..].find(needle);
        assert!(
            found.is_some(),
            "pre-#3136 のブロックが現行 stylesheet に見つからない、または順序が崩れている: {needle:?}"
        );
        search_from += found.unwrap() + needle.len();
    }
}
