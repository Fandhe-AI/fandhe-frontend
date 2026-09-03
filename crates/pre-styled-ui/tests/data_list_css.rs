//! styled DataList（イシュー #767、イシュー #1559 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! 4 slot・少規則で diff が読みやすい部品のため、`crates/pre-styled-ui/tests/
//! callout_css.rs`（イシュー #1556）・`badge_css.rs`（イシュー #1555）と
//! 同型の golden fixture 形式（方式 (a)、
//! `docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.1 参照）を
//! 採る。`crates/pre-styled-ui/tests/table_data_list_css.rs`
//! （方式 (b)、決定性 + 重要規則の存在確認）は Table と共存する既存契約の
//! ため維持し、本ファイルは重複ではなく `css()` 全文の追加固定である。

use fandhe_frontend_pre_styled_ui::data_list;

const DATA_LIST_GOLDEN_CSS: &str = r#"[data-scope="data-list"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-data-list-gap, var(--fandhe-space-4));
}

[data-scope="data-list"][data-part="item"] {
  display: var(--fandhe-data-list-item-display, flex);
  flex-direction: var(--fandhe-data-list-item-flex-direction, column);
  gap: var(--fandhe-data-list-item-gap, var(--fandhe-space-1));
}

[data-scope="data-list"][data-part="item-label"] {
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  min-width: var(--fandhe-data-list-label-min-width, auto);
  color: var(--fandhe-data-list-label-color, var(--fandhe-color-fg-muted));
  font-weight: var(--fandhe-data-list-label-font-weight, var(--fandhe-font-font-weight-normal));
  font-size: var(--fandhe-data-list-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="data-list"][data-part="item-value"] {
  margin: 0;
  display: flex;
  flex: 1;
  align-items: center;
  gap: var(--fandhe-space-2);
  min-width: 0;
  color: var(--fandhe-data-list-value-color, var(--fandhe-color-fg));
  font-size: var(--fandhe-data-list-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="data-list"][data-part="root"].fd-data-list--orientation-vertical {
  --fandhe-data-list-item-display: flex;
  --fandhe-data-list-item-flex-direction: column;
  --fandhe-data-list-item-gap: var(--fandhe-space-1);
  --fandhe-data-list-label-min-width: auto;
}

[data-scope="data-list"][data-part="root"].fd-data-list--orientation-horizontal {
  --fandhe-data-list-item-display: flex;
  --fandhe-data-list-item-flex-direction: row;
  --fandhe-data-list-item-gap: var(--fandhe-space-4);
  --fandhe-data-list-label-min-width: 7.5rem;
}

[data-scope="data-list"][data-part="root"].fd-data-list--variant-subtle {
  --fandhe-data-list-label-color: var(--fandhe-color-fg-muted);
  --fandhe-data-list-label-font-weight: var(--fandhe-font-font-weight-normal);
  --fandhe-data-list-value-color: var(--fandhe-color-fg);
}

[data-scope="data-list"][data-part="root"].fd-data-list--variant-bold {
  --fandhe-data-list-label-color: var(--fandhe-color-fg);
  --fandhe-data-list-label-font-weight: var(--fandhe-font-font-weight-medium);
  --fandhe-data-list-value-color: var(--fandhe-color-fg-muted);
}

[data-scope="data-list"][data-part="root"].fd-data-list--size-xs {
  --fandhe-data-list-gap: var(--fandhe-space-2);
  --fandhe-data-list-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="data-list"][data-part="root"].fd-data-list--size-sm {
  --fandhe-data-list-gap: var(--fandhe-space-3);
  --fandhe-data-list-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="data-list"][data-part="root"].fd-data-list--size-md {
  --fandhe-data-list-gap: var(--fandhe-space-4);
  --fandhe-data-list-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="data-list"][data-part="root"].fd-data-list--size-lg {
  --fandhe-data-list-gap: var(--fandhe-space-5);
  --fandhe-data-list-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="data-list"][data-part="root"].fd-data-list--size-xl {
  --fandhe-data-list-gap: var(--fandhe-space-6);
  --fandhe-data-list-font-size: var(--fandhe-font-font-size-lg);
}
"#;

#[test]
fn data_list_css_matches_golden_fixture() {
    assert_eq!(data_list::css(), DATA_LIST_GOLDEN_CSS);
}

#[test]
fn data_list_css_is_deterministic() {
    assert_eq!(data_list::css(), data_list::css());
}

#[test]
fn data_list_css_never_contains_style_breakout_sequences() {
    let css = data_list::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// ハードコードされた生カラーリテラル（`#` 始まり）を含まないことを固定
/// する（イシュー #606 方針: 色宣言は必ずテーマトークン経由。
/// `crate::callout` の同名契約テストと同型）。
#[test]
fn data_list_css_references_theme_tokens_only() {
    let css = data_list::css();
    assert!(!css.contains('#'));
}
