//! styled Input / Textarea / NativeSelect（イシュー #737）の決定的 CSS 出力
//! ゴールデンテスト + headless 接続照合。
//!
//! `crates/pre-styled-ui/tests/checkbox_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! 加えて `crates/pre-styled-ui/tests/recipe_css.rs::base_selectors_match_actual_headless_markup`
//! と同型の「recipe が生成するセレクタ ⇔ headless 層が実際にレンダリングする
//! `data-scope`/`data-part` 属性」の接続照合を行い、recipe scope
//! （`"field"`、`crates/pre-styled-ui/src/input.rs` rustdoc「`field` scope を
//! 共有する理由」参照）が headless `field::{input,textarea,select}` の実出力と
//! ずれていないことを固定する。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};

const INPUT_GOLDEN_CSS: &str = r#"[data-scope="field"][data-part="input"] {
  box-sizing: border-box;
  width: 100%;
  font: inherit;
  color: var(--fandhe-color-fg);
  background: var(--fandhe-color-bg);
  border-radius: var(--fandhe-radius-md);
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="field"][data-part="input"].fd-field--size-xs {
  height: var(--fandhe-size-control-height-xs, 2rem);
  padding: 0 var(--fandhe-size-control-padding-x-xs, 0.625rem);
  font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="field"][data-part="input"].fd-field--size-sm {
  height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="field"][data-part="input"].fd-field--size-md {
  height: var(--fandhe-size-control-height-md, 2.5rem);
  padding: 0 var(--fandhe-size-control-padding-x-md, 1rem);
  font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md));
}

[data-scope="field"][data-part="input"].fd-field--size-lg {
  height: var(--fandhe-size-control-height-lg, 2.75rem);
  padding: 0 var(--fandhe-size-control-padding-x-lg, 1.25rem);
  font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg));
}

[data-scope="field"][data-part="input"].fd-field--size-xl {
  height: var(--fandhe-size-control-height-xl, 3rem);
  padding: 0 var(--fandhe-size-control-padding-x-xl, 1.5rem);
  font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl));
}

[data-scope="field"][data-part="input"].fd-field--variant-outline {
  border: 1px solid var(--fandhe-color-border);
}

[data-scope="field"][data-part="input"].fd-field--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  border: 1px solid transparent;
}

[data-scope="field"][data-part="input"].fd-field--variant-flushed {
  border: 0;
  border-bottom: 1px solid var(--fandhe-color-border);
  border-radius: 0;
}

[data-scope="field"][data-part="input"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="field"][data-part="input"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="field"][data-part="input"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}
"#;

const TEXTAREA_GOLDEN_CSS: &str = r#"[data-scope="field"][data-part="textarea"] {
  box-sizing: border-box;
  width: 100%;
  font: inherit;
  color: var(--fandhe-color-fg);
  background: var(--fandhe-color-bg);
  border-radius: var(--fandhe-radius-md);
  resize: vertical;
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="field"][data-part="textarea"].fd-field--size-xs {
  padding: 0.125rem var(--fandhe-size-control-padding-x-xs, 0.625rem);
  font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="field"][data-part="textarea"].fd-field--size-sm {
  padding: 0.25rem var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="field"][data-part="textarea"].fd-field--size-md {
  padding: 0.375rem var(--fandhe-size-control-padding-x-md, 1rem);
  font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-sm));
}

[data-scope="field"][data-part="textarea"].fd-field--size-lg {
  padding: 0.5rem var(--fandhe-size-control-padding-x-lg, 1.25rem);
  font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-md));
}

[data-scope="field"][data-part="textarea"].fd-field--size-xl {
  padding: 0.625rem var(--fandhe-size-control-padding-x-xl, 1.5rem);
  font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-lg));
}

[data-scope="field"][data-part="textarea"].fd-field--variant-outline {
  border: 1px solid var(--fandhe-color-border);
}

[data-scope="field"][data-part="textarea"].fd-field--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  border: 1px solid transparent;
}

[data-scope="field"][data-part="textarea"].fd-field--variant-flushed {
  border: 0;
  border-bottom: 1px solid var(--fandhe-color-border);
  border-radius: 0;
}

[data-scope="field"][data-part="textarea"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="field"][data-part="textarea"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="field"][data-part="textarea"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="field"][data-part="textarea"][data-autoresize] {
  field-sizing: content;
  resize: none;
}
"#;

const NATIVE_SELECT_GOLDEN_CSS: &str = r#"[data-scope="field"][data-part="select"] {
  box-sizing: border-box;
  width: 100%;
  font: inherit;
  color: var(--fandhe-color-fg);
  background: var(--fandhe-color-bg);
  border-radius: var(--fandhe-radius-md);
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="field"][data-part="select"].fd-field--size-xs {
  height: var(--fandhe-size-control-height-xs, 2rem);
  padding: 0 var(--fandhe-size-control-padding-x-xs, 0.625rem);
  font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="field"][data-part="select"].fd-field--size-sm {
  height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="field"][data-part="select"].fd-field--size-md {
  height: var(--fandhe-size-control-height-md, 2.5rem);
  padding: 0 var(--fandhe-size-control-padding-x-md, 1rem);
  font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md));
}

[data-scope="field"][data-part="select"].fd-field--size-lg {
  height: var(--fandhe-size-control-height-lg, 2.75rem);
  padding: 0 var(--fandhe-size-control-padding-x-lg, 1.25rem);
  font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg));
}

[data-scope="field"][data-part="select"].fd-field--size-xl {
  height: var(--fandhe-size-control-height-xl, 3rem);
  padding: 0 var(--fandhe-size-control-padding-x-xl, 1.5rem);
  font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl));
}

[data-scope="field"][data-part="select"].fd-field--variant-outline {
  border: 1px solid var(--fandhe-color-border);
}

[data-scope="field"][data-part="select"].fd-field--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  border: 1px solid transparent;
}

[data-scope="field"][data-part="select"].fd-field--variant-plain {
  background: transparent;
  border: 1px solid transparent;
}

[data-scope="field"][data-part="select"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="field"][data-part="select"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="field"][data-part="select"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}
"#;

fn field(id: &str) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

#[test]
fn input_css_matches_golden_fixture() {
    assert_eq!(input::css(), INPUT_GOLDEN_CSS);
}

#[test]
fn textarea_css_matches_golden_fixture() {
    assert_eq!(textarea::css(), TEXTAREA_GOLDEN_CSS);
}

#[test]
fn native_select_css_matches_golden_fixture() {
    assert_eq!(native_select::css(), NATIVE_SELECT_GOLDEN_CSS);
}

#[test]
fn css_outputs_are_byte_identical_across_calls() {
    assert_eq!(input::css(), input::css());
    assert_eq!(textarea::css(), textarea::css());
    assert_eq!(native_select::css(), native_select::css());
}

/// recipe セレクタ ⇔ headless 実マークアップの接続照合（モジュール doc 参照）。
#[test]
fn base_selectors_match_actual_headless_markup() {
    let f = field("f");

    let input_html = render(&input::input(&InputProps::default(), &f, vec![]));
    assert!(input_html.contains(r#"data-scope="field""#));
    assert!(input_html.contains(r#"data-part="input""#));

    let textarea_html = render(&textarea::textarea(
        &TextareaProps::default(),
        &f,
        false,
        vec![],
        vec![],
    ));
    assert!(textarea_html.contains(r#"data-scope="field""#));
    assert!(textarea_html.contains(r#"data-part="textarea""#));

    let option = el("option", vec![("value", "x")], vec![text("X")]);
    let select_html = render(&native_select::native_select(
        &NativeSelectProps::default(),
        &f,
        vec![],
        vec![option],
    ));
    assert!(select_html.contains(r#"data-scope="field""#));
    assert!(select_html.contains(r#"data-part="select""#));
}
