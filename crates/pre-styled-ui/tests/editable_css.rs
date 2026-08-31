//! styled Editable（イシュー #745）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/number_input_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::editable;

const EDITABLE_GOLDEN_CSS: &str = r#"[data-scope="editable"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="editable"][data-part="label"] {
  font-size: var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="editable"][data-part="area"] {
  position: relative;
  display: inline-grid;
}

[data-scope="editable"][data-part="input"] {
  grid-area: 1 / 1;
  box-sizing: border-box;
  width: 100%;
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm));
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md, 0.375rem);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope="editable"][data-part="input"] {
  transition-property: border-color, background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="editable"][data-part="preview"] {
  grid-area: 1 / 1;
  display: inline-block;
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm));
  border: 1px solid transparent;
  border-radius: var(--fandhe-radius-md, 0.375rem);
  cursor: text;
}

[data-scope="editable"][data-part="preview"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="editable"][data-part="control"] {
  display: inline-flex;
  gap: var(--fandhe-space-1);
}

[data-scope="editable"][data-part="edit-trigger"] {
  border: none;
  background: transparent;
  cursor: pointer;
}

[data-scope="editable"][data-part="submit-trigger"] {
  border: none;
  background: transparent;
  cursor: pointer;
}

[data-scope="editable"][data-part="cancel-trigger"] {
  border: none;
  background: transparent;
  cursor: pointer;
}

[data-scope="editable"][data-part="root"].fd-editable--size-xs {
  --fandhe-editable-font-size: var(--fandhe-font-font-size-xs, 0.75rem);
}

[data-scope="editable"][data-part="root"].fd-editable--size-sm {
  --fandhe-editable-font-size: var(--fandhe-font-font-size-xs, 0.75rem);
}

[data-scope="editable"][data-part="root"].fd-editable--size-md {
  --fandhe-editable-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="editable"][data-part="root"].fd-editable--size-lg {
  --fandhe-editable-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="editable"][data-part="root"].fd-editable--size-xl {
  --fandhe-editable-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="editable"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="editable"][data-part="input"][data-readonly] {
  cursor: default;
}

[data-scope="editable"][data-part="input"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="editable"][data-part="input"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="editable"][data-part="preview"][hidden] {
  display: none;
}

[data-scope="editable"][data-part="preview"][data-placeholder-shown] {
  color: var(--fandhe-color-fg-muted, currentColor);
}

[data-scope="editable"][data-part="edit-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.4;
}

[data-scope="editable"][data-part="submit-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.4;
}

[data-scope="editable"][data-part="cancel-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.4;
}
"#;

#[test]
fn editable_stylesheet_matches_golden_fixture() {
    assert_eq!(editable::stylesheet(), EDITABLE_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(editable::stylesheet(), editable::stylesheet());
}

#[test]
fn stylesheet_reflects_size_variant_selector_switch() {
    let css = editable::stylesheet();
    for selector in [
        r#".fd-editable--size-sm"#,
        r#".fd-editable--size-md"#,
        r#".fd-editable--size-lg"#,
    ] {
        assert!(css.contains(selector), "missing selector: {selector}");
    }
}
