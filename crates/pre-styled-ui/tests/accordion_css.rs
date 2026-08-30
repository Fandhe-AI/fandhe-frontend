//! styled Accordion（`size` variant 展開、イシュー #729）の決定的 CSS 出力
//! ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `item-trigger`/`item-content` の padding は `root` の `size` variant が
//! 登録する root スコープ CSS custom property（`--fandhe-accordion-trigger-padding`/
//! `-content-padding`）を `var(..., <Md 既定値>)` で参照する形へ変更した
//! （フォールバック値は変更前の固定値と同一、headless 直接利用時の現行
//! 外観を維持する）。accordion は `color-palette` 軸を持たない。

use fandhe_frontend_pre_styled_ui::accordion;

const ACCORDION_GOLDEN_CSS: &str = r#"[data-scope="accordion"][data-part="root"] {
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.5rem;
  overflow: hidden;
}

[data-scope="accordion"][data-part="item"] {
  border-bottom: 1px solid var(--fandhe-color-border-muted);
}

[data-scope="accordion"][data-part="item-trigger"] {
  display: flex;
  width: 100%;
  padding: var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  cursor: pointer;
  border: 0;
  text-align: left;
}

[data-scope="accordion"][data-part="item-indicator"] {
  display: inline-block;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="accordion"][data-part="item-content"] {
  padding: var(--fandhe-accordion-content-padding, var(--fandhe-space-4));
  color: var(--fandhe-color-fg);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xs {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-2);
  --fandhe-accordion-content-padding: var(--fandhe-space-2);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-sm {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-3);
  --fandhe-accordion-content-padding: var(--fandhe-space-3);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-md {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-4);
  --fandhe-accordion-content-padding: var(--fandhe-space-4);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-lg {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-5);
  --fandhe-accordion-content-padding: var(--fandhe-space-5);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xl {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-6);
  --fandhe-accordion-content-padding: var(--fandhe-space-6);
}

[data-scope="accordion"][data-part="item-trigger"][data-state="open"] {
  color: var(--fandhe-color-accent);
}

[data-scope="accordion"][data-part="item-indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="accordion"][data-part="item-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn accordion_stylesheet_matches_golden_fixture() {
    assert_eq!(accordion::stylesheet(), ACCORDION_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(accordion::stylesheet(), accordion::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = accordion::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
