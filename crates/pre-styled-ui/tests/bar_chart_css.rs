//! styled BarChart（`crates/pre-styled-ui/src/charts/bar_chart.rs`、イシュー
//! #849・イシュー #1590 で参考サイト基準へ調整）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/scroll_area_css.rs` と同型の golden fixture
//! テスト（方式 (a) バイト一致）。`charts::bar_chart` はイシュー #1590 まで
//! golden 不在だったため本ファイルで新設する
//! （`docs/internal/pre-styled-ui-golden-test-update-guide.md` 参照）。
//! `crate::charts::bar_chart` モジュール冒頭 rustdoc「イシュー #1590
//! （参考サイト基準へのスタイル調整、内部整合軸）でのスコープ外判断」節を
//! 正として、出力全体をバイト単位で固定する。
//!
//! 削除・弱体化・`#[ignore]` 禁止（`.claude/rules/coding-rust.md`
//! 「テスト」節）。

use fandhe_frontend_pre_styled_ui::charts::bar_chart;

const BAR_CHART_GOLDEN_CSS: &str = r#"[data-scope="bar-chart"][data-part="root"] {
  display: block;
  max-width: 100%;
  overflow: visible;
}

[data-scope="bar-chart"][data-part="bar"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
}

[data-scope="bar-chart"][data-part="category-label"] {
  font-size: var(--fandhe-font-font-size-xs);
  font-family: var(--fandhe-font-font-body);
  fill: var(--fandhe-color-fg-muted);
}
"#;

#[test]
fn bar_chart_css_matches_golden_fixture() {
    assert_eq!(bar_chart::css(), BAR_CHART_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(bar_chart::css(), bar_chart::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = bar_chart::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
