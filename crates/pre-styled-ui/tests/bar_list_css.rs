//! styled BarList（イシュー #849・親 Phase #845。参考サイト基準への調整は
//! イシュー #1591）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/pie_donut_chart_css.rs` の golden fixture
//! テストの前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。
//! `charts::bar_list::recipe()` の宣言順・宣言値が意図せず変わった場合に
//! この golden テストが即座に検知する（`#[ignore]`・部分一致への緩和は
//! 削除・弱体化しない）。
//!
//! `charts_parts_css.rs` へ追記せず本ファイルを新設したのは、同一 Phase
//! の並列イシュー（bar-chart / bar-segment 等）が同じ共有ファイルへ同時に
//! 追記するとマージコンフリクトを誘発するため（`pie_donut_chart_css.rs`
//! と同じ「部品単位ファイル」方式）。

use fandhe_frontend_pre_styled_ui::charts::bar_list;

const BAR_LIST_GOLDEN_CSS: &str = r#"[data-scope="bar-list"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
  width: 100%;
}

[data-scope="bar-list"][data-part="item"] {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--fandhe-space-1-5) var(--fandhe-space-3);
}

[data-scope="bar-list"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

[data-scope="bar-list"][data-part="track"] {
  grid-column: 1 / -1;
  background: var(--fandhe-color-bg-muted);
  border-radius: var(--fandhe-radius-sm);
  overflow: hidden;
  height: var(--fandhe-bar-list-track-height, 0.5rem);
}

[data-scope="bar-list"][data-part="bar"] {
  height: 100%;
  width: var(--fandhe-bar-list-percent, 0%);
  background: var(--fandhe-color-chart-1);
  border-radius: inherit;
}

[data-scope="bar-list"][data-part="value"] {
  font-size: var(--fandhe-font-font-size-sm);
  font-variant-numeric: tabular-nums;
  color: var(--fandhe-color-fg-muted);
}
"#;

#[test]
fn bar_list_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(bar_list::css(), BAR_LIST_GOLDEN_CSS);
}

#[test]
fn css_output_is_deterministic_across_repeated_calls() {
    assert_eq!(bar_list::css(), bar_list::css());
}

#[test]
fn css_output_never_contains_angle_bracket() {
    assert!(!bar_list::css().contains('<'));
}
