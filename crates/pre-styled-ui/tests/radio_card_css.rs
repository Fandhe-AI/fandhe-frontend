//! styled RadioCard（イシュー #747）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/radio_group_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 2）。出力順（base → variants → states）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::radio_card;

const RADIO_CARD_GOLDEN_CSS: &str = r#"[data-scope="radio-card"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="radio-card"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="radio-card"][data-part="item"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  cursor: pointer;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  padding: var(--fandhe-radio-card-padding, 0.75rem);
  background: var(--fandhe-color-bg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="radio-card"][data-part="item-control"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  flex: 1;
}

[data-scope="radio-card"][data-part="item-content"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  flex: 1;
}

[data-scope="radio-card"][data-part="item-text"] {
  font-size: var(--fandhe-radio-card-label-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg);
}

[data-scope="radio-card"][data-part="item-description"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="radio-card"][data-part="item-addon"] {
  display: flex;
}

[data-scope="radio-card"][data-part="item-indicator"] {
  display: inline-flex;
  width: var(--fandhe-radio-card-control-size, 1rem);
  height: var(--fandhe-radio-card-control-size, 1rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 50%;
  background: var(--fandhe-color-bg);
  flex-shrink: 0;
}

[data-scope="radio-card"][data-part="item-hidden-input"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--size-xs {
  --fandhe-radio-card-padding: 0.25rem;
  --fandhe-radio-card-control-size: 0.7rem;
  --fandhe-radio-card-dot-inset: 1px;
  --fandhe-radio-card-label-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--size-sm {
  --fandhe-radio-card-padding: 0.5rem;
  --fandhe-radio-card-control-size: 0.85rem;
  --fandhe-radio-card-dot-inset: 2px;
  --fandhe-radio-card-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--size-md {
  --fandhe-radio-card-padding: 0.75rem;
  --fandhe-radio-card-control-size: 1rem;
  --fandhe-radio-card-dot-inset: 3px;
  --fandhe-radio-card-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--size-lg {
  --fandhe-radio-card-padding: 1rem;
  --fandhe-radio-card-control-size: 1.25rem;
  --fandhe-radio-card-dot-inset: 4px;
  --fandhe-radio-card-label-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--size-xl {
  --fandhe-radio-card-padding: 1.25rem;
  --fandhe-radio-card-control-size: 1.5rem;
  --fandhe-radio-card-dot-inset: 5px;
  --fandhe-radio-card-label-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="radio-card"][data-part="root"].fd-radio-card--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="radio-card"][data-part="root"][data-orientation="horizontal"] {
  flex-direction: row;
}

[data-scope="radio-card"][data-part="item"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: 0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="radio-card"][data-part="item"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="radio-card"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="radio-card"][data-part="item"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="radio-card"][data-part="item-indicator"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: inset 0 0 0 var(--fandhe-radio-card-dot-inset, 3px) var(--fandhe-color-bg);
}

@media (hover: hover) {
  [data-scope="radio-card"][data-part="item"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn radio_card_stylesheet_matches_golden_fixture() {
    assert_eq!(radio_card::stylesheet(), RADIO_CARD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(radio_card::stylesheet(), radio_card::stylesheet());
}
