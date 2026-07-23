//! styled PinInput（イシュー #739）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs`/`popover_tooltip_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する（受け入れ条件: golden CSS テスト）。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::pin_input;

const PIN_INPUT_GOLDEN_CSS: &str = r#"[data-scope="pin-input"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope="pin-input"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pin-input"][data-part="control"] {
  display: flex;
  gap: var(--fandhe-space-2);
}

[data-scope="pin-input"][data-part="input"] {
  box-sizing: border-box;
  width: var(--fandhe-pin-input-size, 2.5rem);
  height: var(--fandhe-pin-input-size, 2.5rem);
  font-size: var(--fandhe-pin-input-font-size, var(--fandhe-font-font-size-md));
  text-align: center;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  transition: border-color 0.15s, background 0.15s;
}

[data-scope="pin-input"][data-part="root"].fd-pin-input--size-sm {
  --fandhe-pin-input-size: 2rem;
  --fandhe-pin-input-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="pin-input"][data-part="root"].fd-pin-input--size-md {
  --fandhe-pin-input-size: 2.5rem;
  --fandhe-pin-input-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="pin-input"][data-part="root"].fd-pin-input--size-lg {
  --fandhe-pin-input-size: 3rem;
  --fandhe-pin-input-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="pin-input"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="pin-input"][data-part="input"][data-complete] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="pin-input"][data-part="input"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="pin-input"][data-part="input"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(pin_input::stylesheet(), PIN_INPUT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_independent_calls() {
    assert_eq!(pin_input::stylesheet(), pin_input::stylesheet());
}
