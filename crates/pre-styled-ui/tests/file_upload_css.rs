//! styled FileUpload（イシュー #840）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tags_input_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件: golden CSS テスト）。出力順（base → variants → compound →
//! states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。`hidden-input` slot への CSS 非登録も
//! この golden 全文の非存在確認で固定される。

use fandhe_frontend_pre_styled_ui::file_upload;

const FILE_UPLOAD_GOLDEN_CSS: &str = r#"[data-scope="file-upload"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="file-upload"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="file-upload"][data-part="dropzone"] {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  padding: var(--fandhe-file-upload-dropzone-padding, var(--fandhe-space-6));
  gap: var(--fandhe-space-2);
  border: 2px dashed var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  cursor: pointer;
}

[data-scope="file-upload"][data-part="trigger"] {
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-color-fg);
  cursor: pointer;
  font-size: var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm));
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
}

[data-scope="file-upload"][data-part="item-group"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  padding: 0;
  margin: 0;
  list-style: none;
}

[data-scope="file-upload"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  box-sizing: border-box;
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg);
  font-size: var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="file-upload"][data-part="item"] {
  transition-property: border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="file-upload"][data-part="item-name"] {
  flex: 1 1 auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fandhe-color-fg);
}

[data-scope="file-upload"][data-part="item-size-text"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="file-upload"][data-part="item-delete-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 1rem;
  height: 1rem;
  padding: 0;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  background: transparent;
  color: inherit;
  cursor: pointer;
  line-height: 1;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="file-upload"][data-part="item-delete-trigger"] {
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="file-upload"][data-part="clear-trigger"] {
  align-self: flex-start;
  border: none;
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  cursor: pointer;
  font-size: var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="file-upload"][data-part="root"].fd-file-upload--size-xs {
  --fandhe-file-upload-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="file-upload"][data-part="root"].fd-file-upload--size-sm {
  --fandhe-file-upload-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="file-upload"][data-part="root"].fd-file-upload--size-md {
  --fandhe-file-upload-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="file-upload"][data-part="root"].fd-file-upload--size-lg {
  --fandhe-file-upload-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="file-upload"][data-part="root"].fd-file-upload--size-xl {
  --fandhe-file-upload-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="file-upload"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="file-upload"][data-part="dropzone"][data-dragging] {
  border-color: var(--fandhe-color-accent);
  background: var(--fandhe-color-bg-subtle);
}

[data-scope="file-upload"][data-part="dropzone"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="file-upload"][data-part="dropzone"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="file-upload"][data-part="trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="file-upload"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="file-upload"][data-part="item"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="file-upload"][data-part="item-delete-trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="file-upload"][data-part="clear-trigger"][data-disabled] {
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope="file-upload"][data-part="item-delete-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(file_upload::stylesheet(), FILE_UPLOAD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_independent_calls() {
    assert_eq!(file_upload::stylesheet(), file_upload::stylesheet());
}

#[test]
fn golden_css_never_registers_hidden_input_slot() {
    assert!(!FILE_UPLOAD_GOLDEN_CSS.contains(r#"[data-part="hidden-input"]"#));
}
