//! styled CheckboxCard（イシュー #747）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/checkbox_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 2）。出力順（base → variants → states）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。
//!
//! `indicator-check` の `base` に `display` 宣言が無いことは
//! `crates/pre-styled-ui/src/checkbox_card.rs` の inline テスト
//! `indicator_check_base_has_no_display_declaration` で別途固定する
//! （本ファイルは CSS 全文の完全一致のみを担う）。

use fandhe_frontend_pre_styled_ui::checkbox_card;

const CHECKBOX_CARD_GOLDEN_CSS: &str = r#"[data-scope="checkbox-card"][data-part="root"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  cursor: pointer;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  padding: var(--fandhe-checkbox-card-padding, 0.75rem);
  background: var(--fandhe-color-bg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="checkbox-card"][data-part="control"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  flex: 1;
}

[data-scope="checkbox-card"][data-part="content"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  flex: 1;
}

[data-scope="checkbox-card"][data-part="label"] {
  font-size: var(--fandhe-checkbox-card-label-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg);
}

[data-scope="checkbox-card"][data-part="description"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="checkbox-card"][data-part="addon"] {
  display: flex;
}

[data-scope="checkbox-card"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fandhe-checkbox-card-control-size, 1rem);
  height: var(--fandhe-checkbox-card-control-size, 1rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg);
  flex-shrink: 0;
  transition: background 0.15s, border-color 0.15s;
}

[data-scope="checkbox-card"][data-part="indicator-check"] {
  width: var(--fandhe-checkbox-card-check-width, 0.25rem);
  height: var(--fandhe-checkbox-card-check-height, 0.5rem);
  border-right: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  border-bottom: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  transform: rotate(45deg);
  margin-bottom: 0.1rem;
}

[data-scope="checkbox-card"][data-part="hidden-input"] {
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

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-xs {
  --fandhe-checkbox-card-padding: 0.25rem;
  --fandhe-checkbox-card-control-size: 0.7rem;
  --fandhe-checkbox-card-check-width: 0.15rem;
  --fandhe-checkbox-card-check-height: 0.3rem;
  --fandhe-checkbox-card-dash-width: 0.3rem;
  --fandhe-checkbox-card-label-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-sm {
  --fandhe-checkbox-card-padding: 0.5rem;
  --fandhe-checkbox-card-control-size: 0.85rem;
  --fandhe-checkbox-card-check-width: 0.2rem;
  --fandhe-checkbox-card-check-height: 0.4rem;
  --fandhe-checkbox-card-dash-width: 0.4rem;
  --fandhe-checkbox-card-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-md {
  --fandhe-checkbox-card-padding: 0.75rem;
  --fandhe-checkbox-card-control-size: 1rem;
  --fandhe-checkbox-card-check-width: 0.25rem;
  --fandhe-checkbox-card-check-height: 0.5rem;
  --fandhe-checkbox-card-dash-width: 0.5rem;
  --fandhe-checkbox-card-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-lg {
  --fandhe-checkbox-card-padding: 1rem;
  --fandhe-checkbox-card-control-size: 1.25rem;
  --fandhe-checkbox-card-check-width: 0.3rem;
  --fandhe-checkbox-card-check-height: 0.6rem;
  --fandhe-checkbox-card-dash-width: 0.6rem;
  --fandhe-checkbox-card-label-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-xl {
  --fandhe-checkbox-card-padding: 1.25rem;
  --fandhe-checkbox-card-control-size: 1.5rem;
  --fandhe-checkbox-card-check-width: 0.35rem;
  --fandhe-checkbox-card-check-height: 0.7rem;
  --fandhe-checkbox-card-dash-width: 0.7rem;
  --fandhe-checkbox-card-label-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="checkbox-card"][data-part="root"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: 0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox-card"][data-part="root"][data-state="indeterminate"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: 0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox-card"][data-part="root"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="checkbox-card"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="checkbox-card"][data-part="root"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="checkbox-card"][data-part="indicator"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox-card"][data-part="indicator"][data-state="indeterminate"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox-card"][data-part="indicator-check"][data-state="indeterminate"] {
  transform: none;
  border-right: 0;
  border-bottom: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  width: var(--fandhe-checkbox-card-dash-width, 0.5rem);
  height: 0;
  margin-bottom: 0;
}

@media (hover: hover) {
  [data-scope="checkbox-card"][data-part="root"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn checkbox_card_stylesheet_matches_golden_fixture() {
    assert_eq!(checkbox_card::stylesheet(), CHECKBOX_CARD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(checkbox_card::stylesheet(), checkbox_card::stylesheet());
}
