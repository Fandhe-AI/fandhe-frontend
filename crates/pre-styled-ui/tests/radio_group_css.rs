//! styled RadioGroup（イシュー #683、`size`/`palette` variant 拡張は #708、
//! `data-focus-visible` フォーカスリング反映は #709）の決定的 CSS 出力
//! ゴールデンテスト。イシュー #1494（親 #1493 分割 1/2）で
//! root/item/item-control の hover・transition・disabled canonical 化・
//! `data-invalid` 反映・box-sizing・focus ring の palette 統一を追加した。
//! PR #1769 レビュー指摘対応: `root` の `[data-disabled]` 規則は
//! `item` 側と opacity が多重適用されないよう `cursor: not-allowed` のみに
//! 変更した（`radio_group.rs` の該当 `.state` 呼び出しの doc コメント参照）。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::radio_group;

const RADIO_GROUP_GOLDEN_CSS: &str = r#"[data-scope="radio-group"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="radio-group"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="radio-group"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  cursor: pointer;
}

[data-scope="radio-group"][data-part="item-control"] {
  display: inline-flex;
  box-sizing: border-box;
  width: var(--fandhe-radio-group-control-size, 1rem);
  height: var(--fandhe-radio-group-control-size, 1rem);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 50%;
  background: var(--fandhe-color-bg);
  flex-shrink: 0;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="radio-group"][data-part="item-control"] {
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="radio-group"][data-part="item-text"] {
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-radio-group-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="radio-group"][data-part="item-hidden-input"] {
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

[data-scope="radio-group"][data-part="root"].fd-radio-group--size-xs {
  --fandhe-radio-group-control-size: 0.7rem;
  --fandhe-radio-group-dot-inset: 1px;
  --fandhe-radio-group-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--size-sm {
  --fandhe-radio-group-control-size: 0.85rem;
  --fandhe-radio-group-dot-inset: 2px;
  --fandhe-radio-group-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--size-md {
  --fandhe-radio-group-control-size: 1rem;
  --fandhe-radio-group-dot-inset: 3px;
  --fandhe-radio-group-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--size-lg {
  --fandhe-radio-group-control-size: 1.25rem;
  --fandhe-radio-group-dot-inset: 4px;
  --fandhe-radio-group-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--size-xl {
  --fandhe-radio-group-control-size: 1.5rem;
  --fandhe-radio-group-dot-inset: 5px;
  --fandhe-radio-group-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="radio-group"][data-part="root"].fd-radio-group--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="radio-group"][data-part="root"][data-orientation="horizontal"] {
  flex-direction: row;
}

[data-scope="radio-group"][data-part="item-control"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  box-shadow: inset 0 0 0 var(--fandhe-radio-group-dot-inset, 3px) var(--fandhe-color-bg);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="radio-group"][data-part="item-control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="radio-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="radio-group"][data-part="root"][data-disabled] {
  cursor: not-allowed;
}

[data-scope="radio-group"][data-part="item"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="radio-group"][data-part="item-control"][data-focus-visible] {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="radio-group"][data-part="item-control"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn radio_group_stylesheet_matches_golden_fixture() {
    assert_eq!(radio_group::stylesheet(), RADIO_GROUP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // switch_css.rs / popover_tooltip_css.rs と同観点: 独立呼び出し間で
    // バイト単位の一致を固定する。
    assert_eq!(radio_group::stylesheet(), radio_group::stylesheet());
}
