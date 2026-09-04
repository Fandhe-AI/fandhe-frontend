//! styled `charts::bar_segment`（イシュー #849・親 Phase #845、イシュー
//! #1592 で参考サイト基準へ調整）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/scroll_area_css.rs` と同型の golden fixture
//! テスト（`docs/internal/pre-styled-ui-golden-test-update-guide.md` §2.1
//! 方式 (a) バイト一致）。`charts::bar_segment` は #1592 まで
//! golden 不在（同ガイド §3.3「charts 内部パーツ」）だったため本ファイルで
//! 新設する。共有ファイル `charts_parts_css.rs`（`charts::axis`/`grid`/
//! `legend`/`tooltip` 用）へは追記せず、`bar_chart`/`bar_list` と同様に
//! 部品単位の独立ファイルとする（並列進行中の兄弟イシュー #1590/#1591 の
//! golden 新設との共有ファイル競合を避けるため）。
//!
//! `crate::charts::bar_segment` モジュール冒頭 rustdoc「参考サイト基準への
//! 調整（イシュー #1592）」節を正として、出力全体をバイト単位で固定する。
//! 更新手順は上記ガイド §5 を参照（`#[ignore]` 追加・`contains` への
//! 緩和は禁止、`.claude/rules/coding-rust.md` 「テスト」節）。

use fandhe_frontend_pre_styled_ui::charts::bar_segment;

const BAR_SEGMENT_GOLDEN_CSS: &str = r#"[data-scope="bar-segment"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-3, 0.75rem);
  width: 100%;
}

[data-scope="bar-segment"][data-part="bar"] {
  display: flex;
  width: 100%;
  height: var(--fandhe-bar-segment-bar-height, 0.75rem);
  border-radius: var(--fandhe-radius-sm);
  background: var(--fandhe-color-bg-muted);
  overflow: hidden;
}

[data-scope="bar-segment"][data-part="segment"] {
  height: 100%;
  width: var(--fandhe-bar-segment-percent, 0%);
  box-shadow: inset -1px 0 0 var(--fandhe-color-bg);
}

[data-scope="bar-segment"][data-part="legend"] {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fandhe-space-3, 0.75rem) var(--fandhe-space-4, 1rem);
}

[data-scope="bar-segment"][data-part="legend-item"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2, 0.5rem);
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="bar-segment"][data-part="legend-marker"] {
  width: 0.75rem;
  height: 0.75rem;
  border-radius: var(--fandhe-radius-full, 9999px);
  flex-shrink: 0;
}

[data-scope="bar-segment"][data-part="legend-label"] {
  color: var(--fandhe-color-fg);
}

[data-scope="bar-segment"][data-part="legend-value"] {
  color: var(--fandhe-color-fg-muted);
  font-variant-numeric: tabular-nums;
}

[data-scope="bar-segment"][data-part="segment"]:last-child {
  box-shadow: none;
}
"#;

#[test]
fn bar_segment_css_matches_golden_fixture() {
    assert_eq!(bar_segment::css(), BAR_SEGMENT_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(bar_segment::css(), bar_segment::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = bar_segment::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
