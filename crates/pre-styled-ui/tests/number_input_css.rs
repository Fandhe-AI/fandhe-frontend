//! styled NumberInput（イシュー #738）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::number_input;

const NUMBER_INPUT_GOLDEN_CSS: &str = r#"[data-scope="number-input"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="number-input"][data-part="label"] {
  font-size: var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="number-input"][data-part="control"] {
  position: relative;
  display: inline-flex;
  align-items: center;
}

[data-scope="number-input"][data-part="input"] {
  box-sizing: border-box;
  width: 100%;
  height: var(--fandhe-number-input-control-height, 2.5rem);
  padding: 0 var(--fandhe-number-input-trigger-size, 1.5rem) 0 var(--fandhe-space-2);
  font-size: var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm));
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radii-md, 0.375rem);
  background: var(--fandhe-color-bg);
}

[data-scope="number-input"][data-part="increment-trigger"] {
  position: absolute;
  right: 1px;
  top: 1px;
  width: var(--fandhe-number-input-trigger-size, 1.5rem);
  height: 50%;
  border: none;
  background: transparent;
  cursor: pointer;
  line-height: 1;
}

[data-scope="number-input"][data-part="decrement-trigger"] {
  position: absolute;
  right: 1px;
  bottom: 1px;
  width: var(--fandhe-number-input-trigger-size, 1.5rem);
  height: 50%;
  border: none;
  background: transparent;
  cursor: pointer;
  line-height: 1;
}

[data-scope="number-input"][data-part="root"].fd-number-input--size-sm {
  --fandhe-number-input-control-height: 2rem;
  --fandhe-number-input-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-number-input-trigger-size: 1.25rem;
}

[data-scope="number-input"][data-part="root"].fd-number-input--size-md {
  --fandhe-number-input-control-height: 2.5rem;
  --fandhe-number-input-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-number-input-trigger-size: 1.5rem;
}

[data-scope="number-input"][data-part="root"].fd-number-input--size-lg {
  --fandhe-number-input-control-height: 3rem;
  --fandhe-number-input-font-size: var(--fandhe-font-font-size-md);
  --fandhe-number-input-trigger-size: 1.75rem;
}

[data-scope="number-input"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="number-input"][data-part="input"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="number-input"][data-part="input"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="number-input"][data-part="increment-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.4;
}

[data-scope="number-input"][data-part="decrement-trigger"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.4;
}
"#;

#[test]
fn number_input_stylesheet_matches_golden_fixture() {
    assert_eq!(number_input::stylesheet(), NUMBER_INPUT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(number_input::stylesheet(), number_input::stylesheet());
}

#[test]
fn stylesheet_reflects_size_variant_selector_switch() {
    let css = number_input::stylesheet();
    for selector in [
        r#".fd-number-input--size-sm"#,
        r#".fd-number-input--size-md"#,
        r#".fd-number-input--size-lg"#,
    ] {
        assert!(css.contains(selector), "missing selector: {selector}");
    }
}
