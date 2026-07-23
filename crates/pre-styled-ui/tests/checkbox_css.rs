//! styled Checkbox（イシュー #730）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの前例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する（受け入れ条件 3）。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! `indicator` の `base` に `display` 宣言が無いことは
//! `crates/pre-styled-ui/src/checkbox.rs` の inline テスト
//! `indicator_base_has_no_display_declaration` で別途固定する（本ファイルは
//! CSS 全文の完全一致のみを担う）。

use fandhe_frontend_pre_styled_ui::checkbox;

const CHECKBOX_GOLDEN_CSS: &str = r#"[data-scope="checkbox"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  cursor: pointer;
}

[data-scope="checkbox"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fandhe-checkbox-control-size, 1rem);
  height: var(--fandhe-checkbox-control-size, 1rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg);
  flex-shrink: 0;
  transition: background 0.15s, border-color 0.15s;
}

[data-scope="checkbox"][data-part="indicator"] {
  width: var(--fandhe-checkbox-check-width, 0.25rem);
  height: var(--fandhe-checkbox-check-height, 0.5rem);
  border-right: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  border-bottom: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  transform: rotate(45deg);
  margin-bottom: 0.1rem;
}

[data-scope="checkbox"][data-part="label"] {
  font-size: var(--fandhe-checkbox-label-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="checkbox"][data-part="hidden-input"] {
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

[data-scope="checkbox"][data-part="root"].fd-checkbox--size-sm {
  --fandhe-checkbox-control-size: 0.85rem;
  --fandhe-checkbox-check-width: 0.2rem;
  --fandhe-checkbox-check-height: 0.4rem;
  --fandhe-checkbox-dash-width: 0.4rem;
  --fandhe-checkbox-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--size-md {
  --fandhe-checkbox-control-size: 1rem;
  --fandhe-checkbox-check-width: 0.25rem;
  --fandhe-checkbox-check-height: 0.5rem;
  --fandhe-checkbox-dash-width: 0.5rem;
  --fandhe-checkbox-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--size-lg {
  --fandhe-checkbox-control-size: 1.25rem;
  --fandhe-checkbox-check-width: 0.3rem;
  --fandhe-checkbox-check-height: 0.6rem;
  --fandhe-checkbox-dash-width: 0.6rem;
  --fandhe-checkbox-label-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="checkbox"][data-part="root"].fd-checkbox--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}

[data-scope="checkbox"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="checkbox"][data-part="control"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox"][data-part="control"][data-state="indeterminate"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="checkbox"][data-part="control"][data-focus-visible] {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="checkbox"][data-part="indicator"][data-state="indeterminate"] {
  transform: none;
  border-right: 0;
  border-bottom: 2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg));
  width: var(--fandhe-checkbox-dash-width, 0.5rem);
  height: 0;
  margin-bottom: 0;
}
"#;

#[test]
fn checkbox_stylesheet_matches_golden_fixture() {
    assert_eq!(checkbox::stylesheet(), CHECKBOX_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(checkbox::stylesheet(), checkbox::stylesheet());
}
