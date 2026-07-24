//! styled Listbox（イシュー #750）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tags_input_css.rs`/`select_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する（受け入れ条件: golden CSS テスト）。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::listbox;

const LISTBOX_GOLDEN_CSS: &str = r#"[data-scope="listbox"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="listbox"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="listbox"][data-part="content"] {
  display: flex;
  flex-direction: column;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  overflow-y: auto;
  max-height: var(--fandhe-listbox-content-max-height, 16rem);
  padding: var(--fandhe-listbox-content-padding, var(--fandhe-space-2));
}

[data-scope="listbox"][data-part="item-group-label"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="listbox"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-listbox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  border-radius: 0.25rem;
}

[data-scope="listbox"][data-part="item-text"] {
  flex: 1;
  min-width: 0;
}

[data-scope="listbox"][data-part="value-text"] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="listbox"][data-part="root"].fd-listbox--size-sm {
  --fandhe-listbox-item-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-listbox-content-padding: var(--fandhe-space-1);
  --fandhe-listbox-content-max-height: 12rem;
}

[data-scope="listbox"][data-part="root"].fd-listbox--size-md {
  --fandhe-listbox-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-listbox-content-padding: var(--fandhe-space-2);
  --fandhe-listbox-content-max-height: 16rem;
}

[data-scope="listbox"][data-part="root"].fd-listbox--size-lg {
  --fandhe-listbox-item-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-listbox-content-padding: var(--fandhe-space-3);
  --fandhe-listbox-content-max-height: 20rem;
}

[data-scope="listbox"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="listbox"][data-part="item"][data-state="open"] {
  background: var(--fandhe-color-bg-muted);
}

[data-scope="listbox"][data-part="item"][data-highlighted] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="listbox"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="listbox"][data-part="content"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: -2px;
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(listbox::stylesheet(), LISTBOX_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_independent_calls() {
    assert_eq!(listbox::stylesheet(), listbox::stylesheet());
}
