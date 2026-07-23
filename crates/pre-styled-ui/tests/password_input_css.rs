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
  height: var(--fandhe-password-input-height, 2.5rem);
  padding: 0 var(--fandhe-password-input-padding-x, 0.75rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  background: var(--fandhe-color-bg);
  transition: border-color 0.15s;
}

[data-scope="password-input"][data-part="input"] {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  color: var(--fandhe-color-fg);
  padding: 0;
  font-size: var(--fandhe-password-input-font-size, var(--fandhe-font-font-size-md));
}

[data-scope="password-input"][data-part="visibility-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  padding: 0 0 0 var(--fandhe-space-2);
}

[data-scope="password-input"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-sm {
  --fandhe-password-input-height: 2rem;
  --fandhe-password-input-padding-x: 0.5rem;
  --fandhe-password-input-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-md {
  --fandhe-password-input-height: 2.5rem;
  --fandhe-password-input-padding-x: 0.75rem;
  --fandhe-password-input-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="password-input"][data-part="root"].fd-password-input--size-lg {
  --fandhe-password-input-height: 3rem;
  --fandhe-password-input-padding-x: 1rem;
  --fandhe-password-input-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="password-input"][data-part="root"].fd-password-input--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}

[data-scope="password-input"][data-part="control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="password-input"][data-part="control"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="password-input"][data-part="control"]:focus-within {
  outline: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));
  outline-offset: 2px;
}

[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="password-input"][data-part="visibility-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
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
