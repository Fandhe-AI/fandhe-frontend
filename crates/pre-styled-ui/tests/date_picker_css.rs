//! styled DatePicker（イシュー #1471/#1472/#1473）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/combobox_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → compound → states、hover は `@media (hover: hover)`
//! へ集約されて末尾）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。
//!
//! 分割 1/3（イシュー #1471、control/input/trigger/clear-trigger パート）が
//! 新設し、分割 2/3（カレンダーグリッド、#1472）・3/3（ビュー切り替え・
//! ポジショナ、#1473）が期待値を更新する（`docs/internal/
//! pre-styled-ui-golden-test-update-guide.md` の手順に従う）。
//!
//! date_picker は「golden 不在」部品だった（同ガイド §3.3）ため、本ファイルは
//! 1/3 の是正後の `crates/pre-styled-ui/src/date_picker.rs::recipe` の実出力
//! から新規生成した期待値である。

use fandhe_frontend_pre_styled_ui::date_picker;

/// `date_picker::stylesheet()` の期待値（バイト完全一致）。
const EXPECTED_CSS: &str = r#"[data-scope="date-picker"][data-part="root"] {
  position: relative;
}

[data-scope="date-picker"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="date-picker"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope="date-picker"][data-part="input"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-date-picker-input-padding, var(--fandhe-space-2) var(--fandhe-space-3));
}

[data-scope="date-picker"][data-part="input"] {
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="date-picker"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="date-picker"][data-part="trigger"] {
  transition-property: border-color, background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="date-picker"][data-part="clear-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="date-picker"][data-part="clear-trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="date-picker"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 10;
  margin-top: var(--fandhe-space-1);
}

[data-scope="date-picker"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  padding: var(--fandhe-date-picker-content-padding, var(--fandhe-space-2));
}

[data-scope="date-picker"][data-part="root"].fd-date-picker--size-xs {
  --fandhe-date-picker-input-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-date-picker-content-padding: var(--fandhe-space-0-5);
}

[data-scope="date-picker"][data-part="root"].fd-date-picker--size-sm {
  --fandhe-date-picker-input-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-date-picker-content-padding: var(--fandhe-space-1);
}

[data-scope="date-picker"][data-part="root"].fd-date-picker--size-md {
  --fandhe-date-picker-input-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-date-picker-content-padding: var(--fandhe-space-2);
}

[data-scope="date-picker"][data-part="root"].fd-date-picker--size-lg {
  --fandhe-date-picker-input-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-date-picker-content-padding: var(--fandhe-space-3);
}

[data-scope="date-picker"][data-part="root"].fd-date-picker--size-xl {
  --fandhe-date-picker-input-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-date-picker-content-padding: var(--fandhe-space-4);
}

[data-scope="date-picker"][data-part="trigger"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="date-picker"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="date-picker"][data-part="input"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="date-picker"][data-part="input"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="date-picker"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope="date-picker"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="date-picker"][data-part="clear-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn stylesheet_matches_golden_fixture_byte_for_byte() {
    assert_eq!(date_picker::stylesheet(), EXPECTED_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(date_picker::stylesheet(), date_picker::stylesheet());
}
