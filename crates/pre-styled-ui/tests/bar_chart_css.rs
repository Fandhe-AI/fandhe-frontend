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
//!
//! イシュー #2082（shadcn/ui Charts（bar）突合）で `value-label`/
//! `inside-label`/`bar[data-active]` の 3 ブロックを末尾へ純追加した。
//! [`BAR_CHART_GOLDEN_CSS_BEFORE_2082`] は #2082 直前の全量と一致し、
//! `bar_chart_css_is_a_pure_append_since_2082` が新 golden がこの定数から
//! 始まる（バイト単位で先頭一致する）ことを固定する
//! （`crates/pre-styled-ui/tests/scroll_area_css.rs` と同型の純追加固定）。

use fandhe_frontend_pre_styled_ui::charts::bar_chart;

/// イシュー #2082 直前（#1590 まで）の `bar_chart::css()` 全量。
const BAR_CHART_GOLDEN_CSS_BEFORE_2082: &str = r#"[data-scope="bar-chart"][data-part="root"] {
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

[data-scope="bar-chart"][data-part="value-label"] {
  font-size: var(--fandhe-font-font-size-xs);
  font-family: var(--fandhe-font-font-body);
  fill: var(--fandhe-color-fg);
}

[data-scope="bar-chart"][data-part="inside-label"] {
  font-size: var(--fandhe-font-font-size-xs);
  font-family: var(--fandhe-font-font-body);
  fill: var(--fandhe-color-bg);
}

[data-scope="bar-chart"][data-part="bar"][data-active] {
  fill-opacity: 0.8;
  stroke: currentColor;
  stroke-dasharray: 4;
  stroke-dashoffset: 4;
}

[data-scope="bar-chart"][data-part="bar"][data-hidden] {
  display: none;
}

[data-scope="bar-chart"][data-part="value-label"][data-hidden] {
  display: none;
}

[data-scope="bar-chart"][data-part="inside-label"][data-hidden] {
  display: none;
}
"#;

#[test]
fn bar_chart_css_matches_golden_fixture() {
    assert_eq!(bar_chart::css(), BAR_CHART_GOLDEN_CSS);
}

#[test]
fn bar_chart_css_is_a_pure_append_since_2082() {
    assert!(bar_chart::css().starts_with(BAR_CHART_GOLDEN_CSS_BEFORE_2082));
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
