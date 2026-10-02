//! styled Calendar の決定的 CSS 出力ゴールデンテスト（親トラッキング
//! #1450: 分割 1/2 #1451 で新設、分割 2/2 #1452 でヘッダー・ビュー切り替え・
//! 週表示の是正を反映して更新、イシュー #3132 で枠線なし variant・大セル
//! 月表示を純追加）。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` / `angle_slider_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → states →
//! `@media (hover: hover)` → 子孫セレクタ末尾追記）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! 分割 1/2（#1451）は月グリッドと日セルの状態表現（table / table-row /
//! table-body / table-cell / day-trigger）を、分割 2/2（#1452）は
//! ヘッダー行・ナビトリガー・週ヘッダー・root 枠（heading / prev-trigger /
//! next-trigger / table-header / table-head-cell / root）を是正した。
//! イシュー #3132 は `root` variant（`variant`/`cell-size` 軸）2 ブロックと
//! 末尾の子孫セレクタ（`CELL_SIZE_LARGE_CSS`）を純追加した。各是正内容は
//! `crates/pre-styled-ui/src/calendar.rs` モジュール冒頭 rustdoc を参照。
//!
//! 期待値は `crates/pre-styled-ui/src/calendar.rs::recipe`/`stylesheet` の
//! 実出力から生成した。

use fandhe_frontend_pre_styled_ui::calendar;

/// イシュー #3132 直前（`variant`/`cell-size` 軸追加前）の golden 出力。
/// 本イシューが「純追加のみ」であることを固定するための比較対象であり、
/// `calendar::stylesheet()` の戻り値としては使わない
/// （[`crate::tab_nav`]（`tab_nav_css.rs`）の `TAB_NAV_GOLDEN_CSS_BEFORE_3125`
/// と同型のパターン）。
const EXPECTED_CSS_BEFORE_3132: &str = r#"[data-scope="calendar"][data-part="root"] {
  display: inline-grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-calendar-root-padding, var(--fandhe-space-3));
}

[data-scope="calendar"][data-part="heading"] {
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: var(--fandhe-font-font-size-sm);
  grid-row: 1;
  grid-column: 2;
}

[data-scope="calendar"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  grid-row: 1;
  grid-column: 1;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="prev-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  grid-row: 1;
  grid-column: 3;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="next-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="table"] {
  border-collapse: collapse;
  width: 100%;
  grid-column: 1 / -1;
}

[data-scope="calendar"][data-part="table-head-cell"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  font-weight: 500;
  padding: var(--fandhe-space-1);
  text-align: center;
  border-width: 0;
  background: transparent;
}

[data-scope="calendar"][data-part="table-cell"] {
  padding: 1px;
  text-align: center;
  border-width: 0;
  background: transparent;
}

[data-scope="calendar"][data-part="day-trigger"] {
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="day-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-xs {
  --fandhe-calendar-root-padding: var(--fandhe-space-1);
  --fandhe-calendar-day-size: var(--fandhe-space-4);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-sm {
  --fandhe-calendar-root-padding: var(--fandhe-space-2);
  --fandhe-calendar-day-size: var(--fandhe-space-6);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-md {
  --fandhe-calendar-root-padding: var(--fandhe-space-3);
  --fandhe-calendar-day-size: var(--fandhe-space-8);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-lg {
  --fandhe-calendar-root-padding: var(--fandhe-space-4);
  --fandhe-calendar-day-size: var(--fandhe-space-10);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-xl {
  --fandhe-calendar-root-padding: var(--fandhe-space-5);
  --fandhe-calendar-day-size: var(--fandhe-space-12);
}

[data-scope="calendar"][data-part="day-trigger"][data-today] {
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 2px;
}

[data-scope="calendar"][data-part="day-trigger"][data-outside-month] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="calendar"][data-part="day-trigger"][data-selected] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
  --fandhe-hover-bg: var(--fandhe-color-accent);
}

[data-scope="calendar"][data-part="day-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="day-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="calendar"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="calendar"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="calendar"][data-part="day-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="calendar"][data-part="prev-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="calendar"][data-part="next-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

/// `calendar::stylesheet()` の現行期待値（バイト完全一致）。
///
/// [`EXPECTED_CSS_BEFORE_3132`] に対し、root variant 2 ブロック
/// （`variant`/`cell-size` 軸、size variant 5 段の直後）と、末尾の子孫
/// セレクタ（`CELL_SIZE_LARGE_CSS`、`@media (hover: hover)` ブロックの後）を
/// 純追加した。
const EXPECTED_CSS: &str = r#"[data-scope="calendar"][data-part="root"] {
  display: inline-grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-calendar-root-padding, var(--fandhe-space-3));
}

[data-scope="calendar"][data-part="heading"] {
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: var(--fandhe-font-font-size-sm);
  grid-row: 1;
  grid-column: 2;
}

[data-scope="calendar"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  grid-row: 1;
  grid-column: 1;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="prev-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  grid-row: 1;
  grid-column: 3;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="next-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="table"] {
  border-collapse: collapse;
  width: 100%;
  grid-column: 1 / -1;
}

[data-scope="calendar"][data-part="table-head-cell"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  font-weight: 500;
  padding: var(--fandhe-space-1);
  text-align: center;
  border-width: 0;
  background: transparent;
}

[data-scope="calendar"][data-part="table-cell"] {
  padding: 1px;
  text-align: center;
  border-width: 0;
  background: transparent;
}

[data-scope="calendar"][data-part="day-trigger"] {
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-sm);
  width: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  height: var(--fandhe-calendar-day-size, var(--fandhe-space-8));
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="calendar"][data-part="day-trigger"] {
  transition-property: background, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-xs {
  --fandhe-calendar-root-padding: var(--fandhe-space-1);
  --fandhe-calendar-day-size: var(--fandhe-space-4);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-sm {
  --fandhe-calendar-root-padding: var(--fandhe-space-2);
  --fandhe-calendar-day-size: var(--fandhe-space-6);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-md {
  --fandhe-calendar-root-padding: var(--fandhe-space-3);
  --fandhe-calendar-day-size: var(--fandhe-space-8);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-lg {
  --fandhe-calendar-root-padding: var(--fandhe-space-4);
  --fandhe-calendar-day-size: var(--fandhe-space-10);
}

[data-scope="calendar"][data-part="root"].fd-calendar--size-xl {
  --fandhe-calendar-root-padding: var(--fandhe-space-5);
  --fandhe-calendar-day-size: var(--fandhe-space-12);
}

[data-scope="calendar"][data-part="root"].fd-calendar--variant-plain {
  border-width: 0;
}

[data-scope="calendar"][data-part="root"].fd-calendar--cell-size-large {
  display: grid;
}

[data-scope="calendar"][data-part="day-trigger"][data-today] {
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 2px;
}

[data-scope="calendar"][data-part="day-trigger"][data-outside-month] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="calendar"][data-part="day-trigger"][data-selected] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
  --fandhe-hover-bg: var(--fandhe-color-accent);
}

[data-scope="calendar"][data-part="day-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="day-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="calendar"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="calendar"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="calendar"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="calendar"][data-part="day-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="calendar"][data-part="prev-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="calendar"][data-part="next-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}


[data-scope="calendar"][data-part="root"].fd-calendar--cell-size-large [data-scope="calendar"][data-part="table"] {
  table-layout: fixed;
}

[data-scope="calendar"][data-part="root"].fd-calendar--cell-size-large [data-scope="calendar"][data-part="table-cell"] {
  height: var(--fandhe-calendar-cell-height, var(--fandhe-space-24));
  vertical-align: top;
  text-align: start;
  padding: var(--fandhe-space-1);
  border: 1px solid var(--fandhe-color-border);
}
"#;

#[test]
fn stylesheet_matches_golden_fixture_byte_for_byte() {
    assert_eq!(calendar::stylesheet(), EXPECTED_CSS);
}

/// イシュー #3132: 「純追加の固定」テスト。イシュー #3132 直前の golden
/// （[`EXPECTED_CSS_BEFORE_3132`]）が `\n\n` 区切りで持つ各ブロックが、
/// 現行の `calendar::stylesheet()` の中に元の相対順序どおり全て含まれる
/// ことを固定する。既存ブロックの削除・改変・順序入れ替えのいずれかが
/// 起きた場合にこのテストが検知する（新規ブロックの追加だけは許容する）。
#[test]
fn stylesheet_is_pure_addition_over_pre_3132_golden() {
    let before_blocks: Vec<&str> = EXPECTED_CSS_BEFORE_3132
        .trim_end_matches('\n')
        .split("\n\n")
        .collect();
    let current = calendar::stylesheet();

    let mut search_from = 0usize;
    for block in before_blocks {
        let found = current[search_from..].find(block);
        assert!(
            found.is_some(),
            "pre-#3132 のブロックが現行 stylesheet に見つからない、または順序が崩れている: {block:?}"
        );
        search_from += found.unwrap() + block.len();
    }
}

#[test]
fn stylesheet_wraps_hover_state_in_hover_media_query() {
    let css = calendar::stylesheet();
    assert!(css.contains("@media (hover: hover) {"));
    assert!(css.contains(
        r#"[data-scope="calendar"][data-part="day-trigger"]:hover:not([data-disabled])"#
    ));
}

#[test]
fn stylesheet_references_focus_ring_token() {
    let css = calendar::stylesheet();
    assert!(css.contains("var(--fandhe-color-focus-ring, var(--fandhe-color-accent))"));
}

#[test]
fn stylesheet_uses_common_disabled_visual_language_for_day_trigger() {
    let css = calendar::stylesheet();
    assert!(css.contains(
        "[data-scope=\"calendar\"][data-part=\"day-trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
    ));
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = calendar::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
