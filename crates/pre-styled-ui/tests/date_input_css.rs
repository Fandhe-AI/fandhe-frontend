//! styled DateInput（イシュー #834）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/number_input_css.rs`/`splitter_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する（受け入れ条件）。出力順（base → variants →
//! compound → states）が崩れた場合や意図しない宣言の追加・欠落があった
//! 場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::date_input;

const DATE_INPUT_GOLDEN_CSS: &str = r#"[data-scope="date-input"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="date-input"][data-part="label"] {
  font-size: var(--fandhe-date-input-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="date-input"][data-part="control"] {
  display: inline-flex;
  align-items: center;
}

[data-scope="date-input"][data-part="segment-group"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md, 0.375rem);
  padding: 0 var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
}

[data-scope="date-input"][data-part="segment"] {
  box-sizing: border-box;
  height: var(--fandhe-date-input-segment-size, 2.5rem);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: var(--fandhe-date-input-font-size, var(--fandhe-font-font-size-sm));
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  padding: 0 var(--fandhe-space-1);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="date-input"][data-part="segment"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="date-input"][data-part="root"].fd-date-input--size-xs {
  --fandhe-date-input-segment-size: 1.5rem;
  --fandhe-date-input-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="date-input"][data-part="root"].fd-date-input--size-sm {
  --fandhe-date-input-segment-size: 2rem;
  --fandhe-date-input-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="date-input"][data-part="root"].fd-date-input--size-md {
  --fandhe-date-input-segment-size: 2.5rem;
  --fandhe-date-input-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="date-input"][data-part="root"].fd-date-input--size-lg {
  --fandhe-date-input-segment-size: 3rem;
  --fandhe-date-input-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="date-input"][data-part="root"].fd-date-input--size-xl {
  --fandhe-date-input-segment-size: 3.5rem;
  --fandhe-date-input-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="date-input"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="date-input"][data-part="segment-group"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="date-input"][data-part="segment-group"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="date-input"][data-part="segment"][data-placeholder] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="date-input"][data-part="segment"][data-readonly] {
  cursor: default;
}

[data-scope="date-input"][data-part="segment"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

@media (hover: hover) {
  [data-scope="date-input"][data-part="segment"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn date_input_stylesheet_matches_golden_fixture() {
    assert_eq!(date_input::stylesheet(), DATE_INPUT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(date_input::stylesheet(), date_input::stylesheet());
}

#[test]
fn stylesheet_reflects_size_variant_selector_switch() {
    let css = date_input::stylesheet();
    for selector in [
        r#".fd-date-input--size-sm"#,
        r#".fd-date-input--size-md"#,
        r#".fd-date-input--size-lg"#,
    ] {
        assert!(css.contains(selector), "missing selector: {selector}");
    }
}
