//! styled Accordion（`size` variant 展開、イシュー #729。高さトランジション
//! 追加は #2192）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states → `@starting-style` → `@media (hover: hover)`）
//! が崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//!
//! `item-trigger`/`item-content` の padding は `root` の `size` variant が
//! 登録する root スコープ CSS custom property（`--fandhe-accordion-trigger-padding`/
//! `-content-padding`）を `var(..., <Md 既定値>)` で参照する形へ変更した
//! （フォールバック値は変更前の固定値と同一、headless 直接利用時の現行
//! 外観を維持する）。accordion は `color-palette` 軸を持たない。
//!
//! イシュー #1515（参考サイト基準への調整）で以下を追加・置換した:
//! `root` の角丸トークン化・最終 item の二重罫線解消（`LastChild`）・
//! `item-trigger` のラベル左/シェブロン右レイアウトと見出し級タイポ・
//! hover（`hover_bg_muted()` + `Hover` state、`@media (hover: hover)`
//! 集約出力）・disabled（`[data-disabled]` 消費）・transition
//! （`item-trigger`/`item-indicator`）・フォーカスリングの canonical 化
//! （`focus_ring_declarations(Token, Inset)`）。

use fandhe_frontend_pre_styled_ui::accordion;

/// イシュー #2192 直前（`content_height_transition` 未適用）の CSS 全文。
/// `ACCORDION_GOLDEN_CSS` が本イシューで追加した 3 ブロック（item-content
/// の 2 個目 base ブロック・`[hidden]` state・`@starting-style`）を除けば
/// バイト単位で不変であることを
/// `accordion_pre_2192_blocks_remain_verbatim` が固定する
/// （`docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.1 の
/// 純追加運用に倣う）。
const ACCORDION_GOLDEN_CSS_BEFORE_2192: &str = r#"[data-scope="accordion"][data-part="root"] {
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  overflow: hidden;
}

[data-scope="accordion"][data-part="item"] {
  border-bottom: 1px solid var(--fandhe-color-border-muted);
}

[data-scope="accordion"][data-part="item-trigger"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
  width: 100%;
  padding: var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  border: 0;
  text-align: left;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="accordion"][data-part="item-trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="accordion"][data-part="item-indicator"] {
  display: inline-block;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="accordion"][data-part="item-indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="accordion"][data-part="item-content"] {
  padding: var(--fandhe-accordion-content-padding, var(--fandhe-space-4));
  color: var(--fandhe-color-fg);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xs {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-2);
  --fandhe-accordion-content-padding: var(--fandhe-space-2);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-sm {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-3);
  --fandhe-accordion-content-padding: var(--fandhe-space-3);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-md {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-4);
  --fandhe-accordion-content-padding: var(--fandhe-space-4);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-lg {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-5);
  --fandhe-accordion-content-padding: var(--fandhe-space-5);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xl {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-6);
  --fandhe-accordion-content-padding: var(--fandhe-space-6);
}

[data-scope="accordion"][data-part="item"]:last-child {
  border-bottom: 0;
}

[data-scope="accordion"][data-part="item-trigger"][data-state="open"] {
  color: var(--fandhe-color-accent);
}

[data-scope="accordion"][data-part="item-indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="accordion"][data-part="item-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="accordion"][data-part="item-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

@media (hover: hover) {
  [data-scope="accordion"][data-part="item-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

const ACCORDION_GOLDEN_CSS: &str = r#"[data-scope="accordion"][data-part="root"] {
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  overflow: hidden;
}

[data-scope="accordion"][data-part="item"] {
  border-bottom: 1px solid var(--fandhe-color-border-muted);
}

[data-scope="accordion"][data-part="item-trigger"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
  width: 100%;
  padding: var(--fandhe-accordion-trigger-padding, var(--fandhe-space-4));
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  border: 0;
  text-align: left;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="accordion"][data-part="item-trigger"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="accordion"][data-part="item-indicator"] {
  display: inline-block;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="accordion"][data-part="item-indicator"] {
  transition-property: transform;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="accordion"][data-part="item-content"] {
  padding: var(--fandhe-accordion-content-padding, var(--fandhe-space-4));
  color: var(--fandhe-color-fg);
}

[data-scope="accordion"][data-part="item-content"] {
  box-sizing: border-box;
  overflow: hidden;
  height: var(--fandhe-content-height, auto);
  transition-property: height, padding-block, display;
  transition-duration: var(--fandhe-motion-duration-normal);
  transition-timing-function: var(--fandhe-motion-easing-standard);
  transition-behavior: allow-discrete;
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xs {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-2);
  --fandhe-accordion-content-padding: var(--fandhe-space-2);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-sm {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-3);
  --fandhe-accordion-content-padding: var(--fandhe-space-3);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-md {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-4);
  --fandhe-accordion-content-padding: var(--fandhe-space-4);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-lg {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-5);
  --fandhe-accordion-content-padding: var(--fandhe-space-5);
}

[data-scope="accordion"][data-part="root"].fd-accordion--size-xl {
  --fandhe-accordion-trigger-padding: var(--fandhe-space-6);
  --fandhe-accordion-content-padding: var(--fandhe-space-6);
}

[data-scope="accordion"][data-part="item"]:last-child {
  border-bottom: 0;
}

[data-scope="accordion"][data-part="item-trigger"][data-state="open"] {
  color: var(--fandhe-color-accent);
}

[data-scope="accordion"][data-part="item-indicator"][data-state="open"] {
  transform: rotate(180deg);
}

[data-scope="accordion"][data-part="item-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="accordion"][data-part="item-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope="accordion"][data-part="item-content"][hidden] {
  height: 0;
  padding-block: 0;
}

@starting-style {
  [data-scope="accordion"][data-part="item-content"] {
    height: 0;
    padding-block: 0;
  }
}

@media (hover: hover) {
  [data-scope="accordion"][data-part="item-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn accordion_stylesheet_matches_golden_fixture() {
    assert_eq!(accordion::stylesheet(), ACCORDION_GOLDEN_CSS);
}

/// イシュー #2192 純追加検証: #2192 直前の CSS 全文（`_BEFORE_2192`）に
/// 含まれる各ブロック（空行区切り）が現行 `stylesheet()` の中に連続する
/// 部分文字列として残っていることを固定する（3 ブロックの純追加であり、
/// 既存ブロックの内容・順序が一切変わっていないことの機械的な裏付け、
/// `collapsible_css.rs::collapsible_pre_2192_blocks_remain_verbatim` と同型）。
#[test]
fn accordion_pre_2192_blocks_remain_verbatim() {
    let css = accordion::stylesheet();
    for section in ACCORDION_GOLDEN_CSS_BEFORE_2192.split("\n\n") {
        assert!(
            css.contains(section),
            "#2192 以前から存在するブロックが verbatim で残っていない: {section}"
        );
    }
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(accordion::stylesheet(), accordion::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = accordion::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
