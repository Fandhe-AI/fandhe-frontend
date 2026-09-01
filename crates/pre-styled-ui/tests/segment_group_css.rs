//! styled SegmentGroup（イシュー #743、親 #520/#545）の決定的 CSS 出力
//! ゴールデンテスト。イシュー #1498（親 #1497 分割 1/2）でインジケータ
//! 幾何の是正・トランジション/フォーカスリングの canonical 化・hover
//! フィードバック追加・shadow フォールバック除去を行った。size /
//! orientation バリアントと項目ラベルの型階層は兄弟イシュー #1499
//! （2/2）が後続で更新する予定（`segment_group.rs` 冒頭 rustdoc 参照）。
//!
//! `crates/pre-styled-ui/tests/radio_group_css.rs` の golden fixture テスト
//! の前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//! 期待値の生成・更新手順は `docs/internal/pre-styled-ui-golden-test-
//! update-guide.md`（イシュー #1427）参照。

use fandhe_frontend_pre_styled_ui::segment_group;

const SEGMENT_GROUP_GOLDEN_CSS: &str = r#"[data-scope="segment-group"][data-part="root"] {
  position: relative;
  display: inline-flex;
  background: var(--fandhe-color-bg-muted);
  border-radius: var(--fandhe-radius-md, 0.375rem);
  padding: var(--fandhe-space-1, 0.25rem);
}

[data-scope="segment-group"][data-part="indicator"] {
  position: absolute;
  z-index: 0;
  top: var(--fandhe-space-1, 0.25rem);
  left: var(--fandhe-space-1, 0.25rem);
  width: calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1));
  height: calc(100% - 2 * var(--fandhe-space-1, 0.25rem));
  transform: translateX(calc(100% * var(--fandhe-segment-group-index, 0)));
  background: var(--fandhe-color-bg);
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  box-shadow: var(--fandhe-shadow-sm);
}

[data-scope="segment-group"][data-part="indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="segment-group"][data-part="item"] {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  padding-block: var(--fandhe-segment-group-padding-block, 0.375rem);
  padding-inline: var(--fandhe-segment-group-padding-inline, 0.75rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-emphasized);
}

[data-scope="segment-group"][data-part="item"] {
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="segment-group"][data-part="item-control"] {
  display: contents;
}

[data-scope="segment-group"][data-part="item-text"] {
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-segment-group-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="segment-group"][data-part="item-hidden-input"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

[data-scope="segment-group"][data-part="root"].fd-segment-group--size-xs {
  --fandhe-segment-group-font-size: var(--fandhe-font-font-size-xs);
  --fandhe-segment-group-padding-block: 0.125rem;
  --fandhe-segment-group-padding-inline: 0.25rem;
}

[data-scope="segment-group"][data-part="root"].fd-segment-group--size-sm {
  --fandhe-segment-group-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-segment-group-padding-block: 0.25rem;
  --fandhe-segment-group-padding-inline: 0.5rem;
}

[data-scope="segment-group"][data-part="root"].fd-segment-group--size-md {
  --fandhe-segment-group-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-segment-group-padding-block: 0.375rem;
  --fandhe-segment-group-padding-inline: 0.75rem;
}

[data-scope="segment-group"][data-part="root"].fd-segment-group--size-lg {
  --fandhe-segment-group-font-size: var(--fandhe-font-font-size-md);
  --fandhe-segment-group-padding-block: 0.5rem;
  --fandhe-segment-group-padding-inline: 1rem;
}

[data-scope="segment-group"][data-part="root"].fd-segment-group--size-xl {
  --fandhe-segment-group-font-size: var(--fandhe-font-font-size-lg);
  --fandhe-segment-group-padding-block: 0.625rem;
  --fandhe-segment-group-padding-inline: 1.25rem;
}

[data-scope="segment-group"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="segment-group"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="segment-group"][data-part="indicator"][data-state="unchecked"] {
  display: none;
}

[data-scope="segment-group"][data-part="indicator"][data-orientation="vertical"] {
  width: calc(100% - 2 * var(--fandhe-space-1, 0.25rem));
  height: calc((100% - 2 * var(--fandhe-space-1, 0.25rem)) / var(--fandhe-segment-group-count, 1));
  transform: translateY(calc(100% * var(--fandhe-segment-group-index, 0)));
}

[data-scope="segment-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="segment-group"][data-part="item"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="segment-group"][data-part="item-control"][data-focus-visible] {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="segment-group"][data-part="item-text"][data-state="checked"] {
  font-weight: 600;
  color: var(--fandhe-color-accent);
}

@media (hover: hover) {
  [data-scope="segment-group"][data-part="item"]:hover:not([data-disabled]):not([data-state="checked"]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn segment_group_stylesheet_matches_golden_fixture() {
    assert_eq!(segment_group::stylesheet(), SEGMENT_GROUP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // radio_group_css.rs / switch_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(segment_group::stylesheet(), segment_group::stylesheet());
}
