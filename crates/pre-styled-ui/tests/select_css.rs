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
//!
//! イシュー #2019（shadcn/ui 突合）で `content` へ `overflow-y: auto` +
//! `max-height: var(--fandhe-select-content-max-height, 16rem)`（`size`
//! variant 別の固定 rem スケール、listbox #1502 と同型）を新設し、
//! `trigger` へ `[data-invalid]`（`border-color: var(--fandhe-color-danger)`）・
//! `[data-readonly]`（`cursor: default`、`[data-disabled]` より前に登録）
//! を追加した。詳細は `crates/pre-styled-ui/src/select.rs` モジュール
//! rustdoc「shadcn/ui 突合（イシュー #2019）」節参照。
//!
//! イシュー #2186 で `separator`/`scroll-up-button`/`scroll-down-button` の
//! base 3 ブロックを `hidden-select` ブロック直後（`SLOTS` 末尾追加、純追加
//! 原則）へ新設した。詳細は `crates/pre-styled-ui/src/select.rs` モジュール
//! rustdoc「Separator / ScrollButton の着装（イシュー #2186）」節参照。

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
  overflow-y: auto;
  max-height: var(--fandhe-select-content-max-height, 16rem);
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

[data-scope="select"][data-part="separator"] {
  height: 1px;
  background: var(--fandhe-color-border-muted);
  margin: var(--fandhe-space-1) calc(-1 * var(--fandhe-select-content-padding, var(--fandhe-space-2)));
  pointer-events: none;
}

[data-scope="select"][data-part="scroll-up-button"] {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  padding: var(--fandhe-space-1) 0;
  margin: 0 calc(-1 * var(--fandhe-select-content-padding, var(--fandhe-space-2)));
  position: sticky;
  top: 0;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg-muted);
  z-index: 1;
}

[data-scope="select"][data-part="scroll-down-button"] {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  padding: var(--fandhe-space-1) 0;
  margin: 0 calc(-1 * var(--fandhe-select-content-padding, var(--fandhe-space-2)));
  position: sticky;
  bottom: 0;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg-muted);
  z-index: 1;
}

[data-scope="select"][data-part="root"].fd-select--size-xs {
  --fandhe-select-trigger-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-select-item-padding: var(--fandhe-space-0-5) var(--fandhe-space-1);
  --fandhe-select-content-padding: var(--fandhe-space-0-5);
  --fandhe-select-content-max-height: 8rem;
}

[data-scope="select"][data-part="root"].fd-select--size-sm {
  --fandhe-select-trigger-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-select-item-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-select-content-padding: var(--fandhe-space-1);
  --fandhe-select-content-max-height: 12rem;
}

[data-scope="select"][data-part="root"].fd-select--size-md {
  --fandhe-select-trigger-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-select-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-select-content-padding: var(--fandhe-space-2);
  --fandhe-select-content-max-height: 16rem;
}

[data-scope="select"][data-part="root"].fd-select--size-lg {
  --fandhe-select-trigger-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-select-item-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-select-content-padding: var(--fandhe-space-3);
  --fandhe-select-content-max-height: 20rem;
}

[data-scope="select"][data-part="root"].fd-select--size-xl {
  --fandhe-select-trigger-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-select-item-padding: var(--fandhe-space-4) var(--fandhe-space-5);
  --fandhe-select-content-padding: var(--fandhe-space-4);
  --fandhe-select-content-max-height: 24rem;
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

[data-scope="select"][data-part="trigger"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope="select"][data-part="trigger"][data-readonly] {
  cursor: default;
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

// イシュー #2019（shadcn/ui 突合）で新設した `trigger`[data-invalid]/
// [data-readonly] と `content` の max-height/overflow-y を検証する。

#[test]
fn trigger_invalid_attr_is_styled() {
    let css = select::stylesheet();
    assert!(css.contains(r#"[data-scope="select"][data-part="trigger"][data-invalid] {"#));
    assert!(css.contains("border-color: var(--fandhe-color-danger);"));
}

#[test]
fn trigger_readonly_attr_is_styled_and_disabled_takes_precedence_when_both_set() {
    // `date_input.rs::segment_disabled_cursor_overrides_readonly_by_source_order`
    // と同型: disabled かつ readonly が同一 trigger に共存する場合、
    // `[data-disabled]` 規則を `[data-readonly]` 規則より後段に登録する
    // ことで同一詳細度・登録順の後勝ちにより disabled を優先させる
    // （モジュール rustdoc「shadcn/ui 突合（イシュー #2019）」節参照）。
    let css = select::stylesheet();
    let readonly_idx = css
        .find(r#"[data-scope="select"][data-part="trigger"][data-readonly] {"#)
        .expect("trigger readonly rule must exist");
    assert!(css.contains("cursor: default;"));
    let disabled_idx = css
        .find(r#"[data-scope="select"][data-part="trigger"][data-disabled] {"#)
        .expect("trigger disabled rule must exist");
    assert!(
        disabled_idx > readonly_idx,
        "trigger[data-disabled] must be registered after trigger[data-readonly] so it wins by source order"
    );
    let disabled_block = &css[disabled_idx..];
    let block_end = disabled_block.find('}').unwrap_or(disabled_block.len());
    assert!(disabled_block[..block_end].contains("cursor: not-allowed;"));
}

#[test]
fn content_is_scrollable_with_max_height_token() {
    let css = select::stylesheet();
    let content_idx = css
        .find(r#"[data-scope="select"][data-part="content"] {"#)
        .expect("content base rule must exist");
    let block_end = css[content_idx..]
        .find('}')
        .unwrap_or(css.len() - content_idx);
    let content_block = &css[content_idx..content_idx + block_end];
    assert!(content_block.contains("overflow-y: auto;"));
    assert!(content_block.contains("max-height: var(--fandhe-select-content-max-height, 16rem);"));
}

#[test]
fn size_variants_carry_content_max_height_scale() {
    let css = select::stylesheet();
    assert!(css.contains("--fandhe-select-content-max-height: 8rem;"));
    assert!(css.contains("--fandhe-select-content-max-height: 12rem;"));
    assert!(css.contains("--fandhe-select-content-max-height: 16rem;"));
    assert!(css.contains("--fandhe-select-content-max-height: 20rem;"));
    assert!(css.contains("--fandhe-select-content-max-height: 24rem;"));
}

// イシュー #2186: separator / scroll-up-button / scroll-down-button の着装。

#[test]
fn separator_and_scroll_buttons_have_base_rules() {
    let css = select::stylesheet();
    assert!(css.contains(r#"[data-scope="select"][data-part="separator"] {"#));
    assert!(css.contains(r#"[data-scope="select"][data-part="scroll-up-button"] {"#));
    assert!(css.contains(r#"[data-scope="select"][data-part="scroll-down-button"] {"#));
    assert!(css.contains("height: 1px;"));
    assert!(css.contains("position: sticky;"));
    assert!(css.contains("top: 0;"));
    assert!(css.contains("bottom: 0;"));
}

#[test]
fn golden_prefix_through_hidden_select_is_unchanged() {
    // #2019 以前からの golden 前半（`root` 〜 `hidden-select` ブロック末尾）
    // が本イシューの純追加で変化していないことをバイト単位で固定する
    // （`crates/pre-styled-ui/tests/scroll_area_css.rs` #2054 と同型のパターン）。
    const GOLDEN_PREFIX_THROUGH_HIDDEN_SELECT: &str = r#"[data-scope="select"][data-part="root"] {
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
  overflow-y: auto;
  max-height: var(--fandhe-select-content-max-height, 16rem);
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
"#;
    assert!(select::stylesheet().starts_with(GOLDEN_PREFIX_THROUGH_HIDDEN_SELECT));
}
