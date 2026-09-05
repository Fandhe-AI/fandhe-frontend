//! styled JsonTreeView の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/clipboard_css.rs` 等の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。
//!
//! イシュー #1563（参考サイト基準への調整）で新規追加した。追加・置換内容
//! （型別配色の `-fg-subtle` トークン化・monospace フォント・key の medium・
//! boolean/null の太さと斜体・object/array の斜体）は
//! `crates/pre-styled-ui/src/json_tree_view.rs` モジュール doc「参考サイト
//! 基準への調整（イシュー #1563）」節に記録する。`data-kind="bool"` →
//! `"boolean"` はイシュー #1661（headless-ui 側 `JsonValue::kind()` の
//! 語彙統一に追随した破壊的変更）。

use fandhe_frontend_pre_styled_ui::json_tree_view;

const JSON_TREE_VIEW_GOLDEN_CSS: &str = r#"[data-scope="json-tree-view"][data-part="key"] {
  color: var(--fandhe-color-fg);
  font-family: var(--fandhe-font-font-mono);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="json-tree-view"][data-part="value"] {
  color: var(--fandhe-color-fg-muted);
  font-family: var(--fandhe-font-font-mono);
}

[data-scope="json-tree-view"][data-part="value"][data-kind="string"] {
  color: var(--fandhe-color-success-fg-subtle);
}

[data-scope="json-tree-view"][data-part="value"][data-kind="number"] {
  color: var(--fandhe-color-info-fg-subtle);
}

[data-scope="json-tree-view"][data-part="value"][data-kind="boolean"] {
  color: var(--fandhe-color-warning-fg-subtle);
  font-weight: var(--fandhe-font-font-weight-semibold);
}

[data-scope="json-tree-view"][data-part="value"][data-kind="null"] {
  color: var(--fandhe-color-fg-muted);
  font-weight: var(--fandhe-font-font-weight-semibold);
  font-style: italic;
}

[data-scope="json-tree-view"][data-part="value"][data-kind="object"] {
  font-style: italic;
}

[data-scope="json-tree-view"][data-part="value"][data-kind="array"] {
  font-style: italic;
}
"#;

#[test]
fn json_tree_view_stylesheet_matches_golden_fixture() {
    assert_eq!(json_tree_view::stylesheet(), JSON_TREE_VIEW_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(json_tree_view::stylesheet(), json_tree_view::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = json_tree_view::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
