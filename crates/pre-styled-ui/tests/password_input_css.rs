//! styled PasswordInput（イシュー #740）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 3）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::password_input;

const PASSWORD_INPUT_GOLDEN_CSS: &str = r#"[data-scope="password-input"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="password-input"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="password-input"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: 100%;
  height: var(--fandhe-password-input-height, var(--fandhe-size-control-height-md, 2.5rem));
  padding: 0 var(--fandhe-password-input-padding-x, var(--fandhe-size-control-padding-x-md, 1rem));
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
}

[data-scope="password-input"][data-part="control"] {
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="password-input"][data-part="input"] {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  color: var(--fandhe-color-fg);
  padding: 0;
  font-size: var(--fandhe-password-input-font-size, var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md)));
}

[data-scope="password-input"][data-part="visibility-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  padding: var(--fandhe-space-1);
  margin-left: var(--fandhe-space-1);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="password-input"][data-part="visibility-trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="password-input"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
}

[data-scope="password-input"][data-part="indicator"] {
  transition-property: color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-xs {
  --fandhe-password-input-height: var(--fandhe-size-control-height-xs, 2rem);
  --fandhe-password-input-padding-x: var(--fandhe-size-control-padding-x-xs, 0.625rem);
  --fandhe-password-input-font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-sm {
  --fandhe-password-input-height: var(--fandhe-size-control-height-sm, 2.25rem);
  --fandhe-password-input-padding-x: var(--fandhe-size-control-padding-x-sm, 0.75rem);
  --fandhe-password-input-font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-md {
  --fandhe-password-input-height: var(--fandhe-size-control-height-md, 2.5rem);
  --fandhe-password-input-padding-x: var(--fandhe-size-control-padding-x-md, 1rem);
  --fandhe-password-input-font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md));
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-lg {
  --fandhe-password-input-height: var(--fandhe-size-control-height-lg, 2.75rem);
  --fandhe-password-input-padding-x: var(--fandhe-size-control-padding-x-lg, 1.25rem);
  --fandhe-password-input-font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg));
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-xl {
  --fandhe-password-input-height: var(--fandhe-size-control-height-xl, 3rem);
  --fandhe-password-input-padding-x: var(--fandhe-size-control-padding-x-xl, 1.5rem);
  --fandhe-password-input-font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl));
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="password-input"][data-part="control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="password-input"][data-part="control"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="password-input"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="password-input"][data-part="visibility-trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="password-input"][data-part="indicator"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}

@media (hover: hover) {
  [data-scope="password-input"][data-part="visibility-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn password_input_stylesheet_matches_golden_fixture() {
    assert_eq!(password_input::stylesheet(), PASSWORD_INPUT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間でバイト
    // 単位の一致を固定する。
    assert_eq!(password_input::stylesheet(), password_input::stylesheet());
}
