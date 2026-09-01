//! styled Select（`size` variant 展開、イシュー #729。トリガー・
//! コントロールのスタイル調整、イシュー #1501。リスト側パーツの
//! スタイル調整、イシュー #1502）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `trigger`/`item`/`content` の padding は `root` の `size` variant が
//! 登録する root スコープ CSS custom property（`--fandhe-select-trigger-padding`/
//! `-item-padding`/`-content-padding`）を `var(..., <Md 既定値>)` で参照する
//! 形へ変更した（フォールバック値は変更前の固定値と同一、headless 直接
//! 利用時の現行外観を維持する）。`--fandhe-reference-width`/`--fandhe-x`/
//! `--fandhe-y`（wasm positioning 契約、#663）は不変。select は
//! `color-palette` 軸を持たない。
//!
//! イシュー #1501（親 #1500 の 1/2 分割、`control`/`trigger`/`value-text`/
//! `indicator` 担当）で `trigger` の `border-radius` をトークン化
//! （`var(--fandhe-radius-md)`、値は `0.375rem` と同一で外観不変）し、
//! hover（`--fandhe-hover-bg` 経由）・disabled（`[data-disabled]`）・
//! transition（`border-color, background, color`）を追加した。
//! `:focus-visible` は canonical ヘルパ（`focus_ring_declarations`）へ
//! 置換した。`value-text` は base 宣言（truncation）と
//! `[data-placeholder-shown]` の muted 色を新設し、`indicator` は base
//! 宣言（`display: inline-block` + muted 色 + transition）と
//! `[data-state="open"]` の回転を新設した。
//!
//! イシュー #1502（親 #1500 の 2/2 分割、`content`/`item`/`item-group`/
//! `item-indicator` 担当）で `content` の `border-radius`（生 `0.375rem`
//! → `var(--fandhe-radius-md)`）・`box-shadow`（生 `rgba()` →
//! `var(--fandhe-shadow-md)`）、`item` の `border-radius`（生 `0.25rem`
//! → `var(--fandhe-radius-sm)`）をトークン化した。`item` へ
//! `display: flex`/`align-items: center`/`gap`（チェックマーク右端整列）・
//! hover（`hover_bg_muted()` base + `HoverExceptAttr("data-highlighted")`）・
//! disabled（`[data-disabled]`）・transition（`background, color`）を
//! 追加し、`item-indicator` へ `margin-left: auto`（`display` は非宣言、
//! headless の `hidden` 属性制御と衝突するため）を追加した。

use fandhe_frontend_pre_styled_ui::select;

const SELECT_GOLDEN_CSS: &str = r#"[data-scope="select"][data-part="root"] {
  position: relative;
}

[data-scope="select"][data-part="label"] {
  display: block;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="select"][data-part="control"] {
  display: inline-flex;
}

[data-scope="select"][data-part="trigger"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-select-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="select"][data-part="trigger"] {
  transition-property: border-color, background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="select"][data-part="value-text"] {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

[data-scope="select"][data-part="clear-trigger"] {
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="select"][data-part="indicator"] {
  display: inline-block;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="select"][data-part="indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="select"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 10;
  margin-top: var(--fandhe-space-1);
}

[data-scope="select"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-md);
  padding: var(--fandhe-select-content-padding, var(--fandhe-space-2));
  min-width: var(--fandhe-reference-width, auto);
}

[data-scope="select"][data-part="item-group-label"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="select"][data-part="item"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-select-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  border-radius: var(--fandhe-radius-sm);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="select"][data-part="item"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="select"][data-part="item-indicator"] {
  margin-left: auto;
}

[data-scope="select"][data-part="hidden-select"] {
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

[data-scope="select"][data-part="root"].fd-select--size-xs {
  --fandhe-select-trigger-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-select-item-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-select-content-padding: var(--fandhe-space-0-5);
}

[data-scope="select"][data-part="root"].fd-select--size-sm {
  --fandhe-select-trigger-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-select-item-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-select-content-padding: var(--fandhe-space-1);
}

[data-scope="select"][data-part="root"].fd-select--size-md {
  --fandhe-select-trigger-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-select-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-select-content-padding: var(--fandhe-space-2);
}

[data-scope="select"][data-part="root"].fd-select--size-lg {
  --fandhe-select-trigger-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-select-item-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-select-content-padding: var(--fandhe-space-3);
}

[data-scope="select"][data-part="root"].fd-select--size-xl {
  --fandhe-select-trigger-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-select-item-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-select-content-padding: var(--fandhe-space-4);
}

[data-scope="select"][data-part="trigger"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="select"][data-part="item"][data-state="open"] {
  background: var(--fandhe-color-bg-muted);
}

[data-scope="select"][data-part="item"][data-highlighted] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="select"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="select"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="select"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="select"][data-part="value-text"][data-placeholder-shown] {
  color: var(--fandhe-color-fg-muted);
}

[data-scope="select"][data-part="indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="select"][data-part="positioner"][data-positioned] {
  position: fixed;
  top: 0;
  left: 0;
  margin-top: 0;
  transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
}

@media (hover: hover) {
  [data-scope="select"][data-part="item"]:hover:not([data-disabled]):not([data-highlighted]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="select"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn select_stylesheet_matches_golden_fixture() {
    assert_eq!(select::stylesheet(), SELECT_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(select::stylesheet(), select::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = select::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
