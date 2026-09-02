//! styled Dialog（`size` variant 展開、イシュー #729）の決定的 CSS 出力
//! ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `content`/`title` の寸法・書体は `root` の `size` variant が登録する
//! root スコープ CSS custom property（`--fandhe-dialog-content-padding`/
//! `-content-max-width`/`-title-font-size`）を `var(..., <Md 既定値>)` で
//! 参照する形へ変更した（フォールバック値は変更前の固定値と同一、headless
//! 直接利用時の現行外観を維持する）。dialog は `color-palette` 軸を持たない。
//!
//! イシュー #1692（親 #1520）: 外枠パート（trigger/backdrop/positioner/
//! content）の枠・影・サイズを調整した。trigger のボタン化（背景・枠・
//! 角丸・padding・hover surface・transition）、backdrop の
//! `--fandhe-color-bg-overlay`/`--fandhe-z-index-overlay` トークン化、
//! positioner の `--fandhe-z-index-modal` トークン化、content の
//! `--fandhe-radius-lg` トークン化 + `--fandhe-shadow-lg` 新規追加を反映
//! する（詳細は `crates/pre-styled-ui/src/dialog.rs` rustdoc 参照）。
//!
//! イシュー #1693 で `title`/`description`/`close-trigger`（内部パート）の
//! スタイル調整を追加した（詳細は `crates/pre-styled-ui/src/dialog.rs`
//! モジュール冒頭 rustdoc「内部パートのスタイル調整」節参照）。開閉
//! トランジション（`transition-property` 追加）は headless 層の `hidden`
//! 即時付与契約と両立しない（codex-review #1795 P1 指摘）ため見送った
//! （同 rustdoc「開閉トランジションを追加しない理由」節参照）。

use fandhe_frontend_pre_styled_ui::dialog;

const DIALOG_GOLDEN_CSS: &str = r#"[data-scope="dialog"][data-part="trigger"] {
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  cursor: pointer;
  color: var(--fandhe-color-fg);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="dialog"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  z-index: var(--fandhe-z-index-overlay, 1000);
  background: var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4));
}

[data-scope="dialog"][data-part="positioner"] {
  position: fixed;
  inset: 0;
  z-index: var(--fandhe-z-index-modal, 1001);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--fandhe-space-4);
}

[data-scope="dialog"][data-part="content"] {
  position: relative;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border-radius: var(--fandhe-radius-lg, 0.5rem);
  box-shadow: var(--fandhe-shadow-lg);
  padding: var(--fandhe-dialog-content-padding, var(--fandhe-space-6));
  max-width: var(--fandhe-dialog-content-max-width, 32rem);
  width: 100%;
}

[data-scope="dialog"][data-part="title"] {
  font-size: var(--fandhe-dialog-title-font-size, var(--fandhe-font-font-size-lg));
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  margin: 0 0 var(--fandhe-space-2) 0;
  padding-inline-end: var(--fandhe-space-8);
}

[data-scope="dialog"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  line-height: var(--fandhe-font-line-height-normal);
  margin: 0 0 var(--fandhe-space-4) 0;
}

[data-scope="dialog"][data-part="close-trigger"] {
  position: absolute;
  inset-block-start: var(--fandhe-space-2);
  inset-inline-end: var(--fandhe-space-2);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fandhe-space-8);
  height: var(--fandhe-space-8);
  overflow: hidden;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  background: transparent;
  padding: var(--fandhe-space-1);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="dialog"][data-part="root"].fd-dialog--size-xs {
  --fandhe-dialog-content-padding: var(--fandhe-space-2);
  --fandhe-dialog-content-max-width: 16rem;
  --fandhe-dialog-title-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="dialog"][data-part="root"].fd-dialog--size-sm {
  --fandhe-dialog-content-padding: var(--fandhe-space-4);
  --fandhe-dialog-content-max-width: 24rem;
  --fandhe-dialog-title-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="dialog"][data-part="root"].fd-dialog--size-md {
  --fandhe-dialog-content-padding: var(--fandhe-space-6);
  --fandhe-dialog-content-max-width: 32rem;
  --fandhe-dialog-title-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="dialog"][data-part="root"].fd-dialog--size-lg {
  --fandhe-dialog-content-padding: var(--fandhe-space-8);
  --fandhe-dialog-content-max-width: 42rem;
  --fandhe-dialog-title-font-size: var(--fandhe-font-font-size-xl);
}

[data-scope="dialog"][data-part="root"].fd-dialog--size-xl {
  --fandhe-dialog-content-padding: var(--fandhe-space-10);
  --fandhe-dialog-content-max-width: 52rem;
  --fandhe-dialog-title-font-size: var(--fandhe-font-font-size-2xl);
}

[data-scope="dialog"][data-part="backdrop"][data-state="open"] {
  opacity: 1;
}

[data-scope="dialog"][data-part="backdrop"][data-state="closed"] {
  opacity: 0;
}

[data-scope="dialog"][data-part="content"][data-state="open"] {
  transform: scale(1);
}

[data-scope="dialog"][data-part="content"][data-state="closed"] {
  transform: scale(0.95);
}

[data-scope="dialog"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="dialog"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="dialog"][data-part="close-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="dialog"][data-part="close-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="dialog"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn dialog_stylesheet_matches_golden_fixture() {
    assert_eq!(dialog::stylesheet(), DIALOG_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(dialog::stylesheet(), dialog::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = dialog::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
