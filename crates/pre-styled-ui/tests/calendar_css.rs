//! styled Calendar の決定的 CSS 出力ゴールデンテスト（親トラッキング
//! #1450: 分割 1/2 #1451 で新設、分割 2/2 #1452 でヘッダー・ビュー切り替え・
//! 週表示の是正を反映して更新）。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` / `angle_slider_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → variants → states →
//! `@media (hover: hover)`）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! 分割 1/2（#1451）は月グリッドと日セルの状態表現（table / table-row /
//! table-body / table-cell / day-trigger）を、分割 2/2（#1452、本更新）は
//! ヘッダー行・ナビトリガー・週ヘッダー・root 枠（heading / prev-trigger /
//! next-trigger / table-header / table-head-cell / root）を是正した。両者の
//! 是正内容は `crates/pre-styled-ui/src/calendar.rs` モジュール冒頭 rustdoc
//! を参照。
//!
//! 期待値は `crates/pre-styled-ui/src/calendar.rs::recipe` の実出力から
//! 生成した。

use fandhe_frontend_pre_styled_ui::calendar;

/// `calendar::stylesheet()` の期待値（バイト完全一致）。
///
/// 出力順は `SlotRecipe::css`（`crates/pre-styled-ui/src/recipe.rs`）の
/// 契約どおり「base（`SLOTS` 宣言順: root → heading → prev-trigger →
/// prev-trigger transition base → next-trigger → next-trigger transition
/// base → table → table-head-cell → table-cell → day-trigger →
/// day-trigger transition base）→ variants（登録順: size 5 段）→
/// states（登録順: day-trigger selected → today → outside-month →
/// disabled → focus-visible → prev-trigger disabled → hover → focus-visible
/// → next-trigger disabled → hover → focus-visible）→
/// `@media (hover: hover)`（day-trigger → prev-trigger → next-trigger の
/// hover、states ループとは別集計で末尾にまとめて出力）」。
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

[data-scope="calendar"][data-part="day-trigger"][data-selected] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
  --fandhe-hover-bg: var(--fandhe-color-accent);
}

[data-scope="calendar"][data-part="day-trigger"][data-today] {
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 2px;
}

[data-scope="calendar"][data-part="day-trigger"][data-outside-month] {
  color: var(--fandhe-color-fg-muted);
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

#[test]
fn stylesheet_matches_golden_fixture_byte_for_byte() {
    assert_eq!(calendar::stylesheet(), EXPECTED_CSS);
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
