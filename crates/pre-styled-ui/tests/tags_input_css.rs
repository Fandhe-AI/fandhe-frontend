//! styled TagsInput（イシュー #744）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/pin_input_css.rs`/`switch_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する（受け入れ条件: golden CSS テスト）。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1698（外枠パート root/control/input のスタイル調整、親
//! #1510）で以下を反映済み: `root` の disabled を
//! `crate::recipe::disabled_declarations()`（宣言順 opacity → cursor）へ
//! 置換・`control` の角丸を `--fandhe-radius-md` へ変更・`control` へ
//! transition/`:focus-within` フォーカスリング/`[data-disabled]` を追加・
//! `input` の `:focus-visible` outline を削除（リングは `control` へ移設）。

use fandhe_frontend_pre_styled_ui::tags_input;

const TAGS_INPUT_GOLDEN_CSS: &str = r#"[data-scope="tags-input"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="tags-input"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tags-input"][data-part="control"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fandhe-space-2);
  box-sizing: border-box;
  padding: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
}

[data-scope="tags-input"][data-part="control"] {
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tags-input"][data-part="input"] {
  flex: 1 1 auto;
  min-width: var(--fandhe-tags-input-input-min-width, 6rem);
  box-sizing: border-box;
  border: none;
  outline: none;
  background: transparent;
  font-size: var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm));
  color: var(--fandhe-color-fg);
}

[data-scope="tags-input"][data-part="item-preview"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  font-size: var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm));
  padding: var(--fandhe-tags-input-chip-padding-y, 0.125rem) var(--fandhe-tags-input-chip-padding-x, 0.5rem);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-color-fg);
}

[data-scope="tags-input"][data-part="item-text"] {
  white-space: nowrap;
}

[data-scope="tags-input"][data-part="item-delete-trigger"] {
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
}

[data-scope="tags-input"][data-part="clear-trigger"] {
  align-self: flex-start;
  border: none;
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  cursor: pointer;
  font-size: var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="tags-input"][data-part="root"].fd-tags-input--size-xs {
  --fandhe-tags-input-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tags-input"][data-part="root"].fd-tags-input--size-sm {
  --fandhe-tags-input-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tags-input"][data-part="root"].fd-tags-input--size-md {
  --fandhe-tags-input-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tags-input"][data-part="root"].fd-tags-input--size-lg {
  --fandhe-tags-input-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tags-input"][data-part="root"].fd-tags-input--size-xl {
  --fandhe-tags-input-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="tags-input"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tags-input"][data-part="control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="tags-input"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tags-input"][data-part="control"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="tags-input"][data-part="item-preview"][data-highlighted] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="tags-input"][data-part="item-delete-trigger"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="tags-input"][data-part="input"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="tags-input"][data-part="clear-trigger"][data-disabled] {
  cursor: not-allowed;
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(tags_input::stylesheet(), TAGS_INPUT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_independent_calls() {
    assert_eq!(tags_input::stylesheet(), tags_input::stylesheet());
}
