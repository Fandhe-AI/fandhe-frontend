//! styled Data Table（イシュー #2127、親 #2124。headless 側 anatomy は
//! #2125）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/message_scroller_css.rs` と同型の golden
//! fixture テスト。`data-table` recipe は `root`/`toolbar`/
//! `column-header`/`sort-trigger`/`select-all`/`select-row`/`footer`/
//! `selection-count` の 8 slot を宣言し、`data-loading`/`data-empty`/
//! `data-sort`/`data-state`/`data-hidden` は headless の `data-*` を
//! `Attr`/`AttrEq` で参照するのみで class ベースの軸を持たない
//! （`src/data_table.rs` モジュール doc「軸を持たない理由」節参照）。
//! `sort-trigger[data-sort="..."]::after` の raw CSS 追記（ソート方向
//! アイコン）も golden に含む。

use fandhe_frontend_pre_styled_ui::data_table;

const DATA_TABLE_GOLDEN_CSS: &str = "[data-scope=\"data-table\"][data-part=\"root\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-data-table-gap, var(--fandhe-space-3));
  width: 100%;
}

[data-scope=\"data-table\"][data-part=\"toolbar\"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
}

[data-scope=\"data-table\"][data-part=\"column-header\"] {
  text-align: start;
  padding: var(--fandhe-data-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4));
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"data-table\"][data-part=\"sort-trigger\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  margin-inline: calc(-1 * var(--fandhe-space-2));
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  background: transparent;
  border: 0;
  border-radius: var(--fandhe-radius-md);
  color: inherit;
  font: inherit;
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"data-table\"][data-part=\"select-all\"] {
  padding: var(--fandhe-data-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4));
  border-bottom: var(--fandhe-data-table-row-border, 1px solid var(--fandhe-color-border-muted));
  vertical-align: middle;
  width: var(--fandhe-data-table-select-width, 2.5rem);
  text-align: center;
}

[data-scope=\"data-table\"][data-part=\"select-row\"] {
  padding: var(--fandhe-data-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4));
  border-bottom: var(--fandhe-data-table-row-border, 1px solid var(--fandhe-color-border-muted));
  vertical-align: middle;
  width: var(--fandhe-data-table-select-width, 2.5rem);
  text-align: center;
}

[data-scope=\"data-table\"][data-part=\"footer\"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-3);
  padding-block: var(--fandhe-space-2);
}

[data-scope=\"data-table\"][data-part=\"selection-count\"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
  flex: 1 1 auto;
}

[data-scope=\"data-table\"][data-part=\"root\"][data-loading] {
  cursor: progress;
}

[data-scope=\"data-table\"][data-part=\"root\"][data-empty] {
  --fandhe-data-table-empty-min-height: 12rem;
  min-height: var(--fandhe-data-table-empty-min-height);
}

[data-scope=\"data-table\"][data-part=\"sort-trigger\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope=\"data-table\"][data-part=\"sort-trigger\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"ascending\"]::after {
  content: \"▲\";
}

[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"descending\"]::after {
  content: \"▼\";
}

[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"none\"]::after {
  content: \"↕\";
  opacity: 0.5;
}
";

/// [`data_table::stylesheet`] の出力が golden fixture と完全一致すること
/// を固定する（`message_scroller_css.rs` と同型の契約）。CSS の意匠を
/// 変更する場合は本 const を実測値へ更新する
/// （`docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.1
/// 「data_table」節の手順参照）。
#[test]
fn stylesheet_matches_golden_snapshot() {
    assert_eq!(data_table::stylesheet(), DATA_TABLE_GOLDEN_CSS);
}

/// `stylesheet()` が決定的（同一入力で常に同一出力）であり、`</style`
/// 脱出シーケンス・`<` を含まないことを固定する（`src/data_table.rs`
/// モジュール doc「セキュリティ不変条件」節参照）。
#[test]
fn stylesheet_is_deterministic_and_has_no_style_breakout_sequences() {
    let a = data_table::stylesheet();
    let b = data_table::stylesheet();
    assert_eq!(a, b);
    assert!(!a.contains("</style"));
    assert!(!a.contains('<'));
}

/// `sort-trigger` の `[data-sort="ascending"|"descending"|"none"]::after`
/// raw CSS 追記を固定する（`src/data_table.rs` モジュール doc「ソート
/// 方向アイコンを raw CSS `::after` で表現する理由」節参照）。
#[test]
fn css_has_sort_direction_after_rules_for_all_three_states() {
    let css = data_table::stylesheet();
    assert!(css.contains(
        "[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"ascending\"]::after {\n  content: \"\u{25b2}\";\n}\n"
    ));
    assert!(css.contains(
        "[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"descending\"]::after {\n  content: \"\u{25bc}\";\n}\n"
    ));
    assert!(css.contains(
        "[data-scope=\"data-table\"][data-part=\"sort-trigger\"][data-sort=\"none\"]::after {\n  content: \"\u{2195}\";\n  opacity: 0.5;\n}\n"
    ));
}

/// `select-all`/`select-row` が `table` scope の CSS 変数
/// （`--fandhe-table-*`）を一切参照しないことを固定する（`src/data_table.rs`
/// モジュール doc「`select-all`/`select-row` が `--fandhe-table-*` を
/// 参照しない理由」節・`crates/docs-site/tests/css_var_scope_prefix.rs`
/// 契約参照）。
#[test]
fn select_cells_never_reference_table_scope_css_vars() {
    let css = data_table::stylesheet();
    assert!(!css.contains("--fandhe-table-"));
}

/// `root[data-loading]`/`root[data-empty]` を CSS セレクタとして参照する
/// のみで、`data_table` 自身が `data-loading`/`data-empty` を組み立てて
/// いないことを固定する（headless-sourced の一環）。
#[test]
fn css_references_root_loading_and_empty_states() {
    let css = data_table::stylesheet();
    assert!(css.contains(r#"[data-scope="data-table"][data-part="root"][data-loading]"#));
    assert!(css.contains(r#"[data-scope="data-table"][data-part="root"][data-empty]"#));
}
