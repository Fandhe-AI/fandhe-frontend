//! styled Select（`size` variant 展開、イシュー #729）の決定的 CSS 出力
//! ゴールデンテスト。
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
  border-radius: 0.375rem;
  padding: var(--fandhe-select-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
}

[data-scope="select"][data-part="clear-trigger"] {
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
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
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  padding: var(--fandhe-select-content-padding, var(--fandhe-space-2));
  min-width: var(--fandhe-reference-width, auto);
}

[data-scope="select"][data-part="item-group-label"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="select"][data-part="item"] {
  padding: var(--fandhe-select-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  border-radius: 0.25rem;
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

[data-scope="select"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="select"][data-part="positioner"][data-positioned] {
  position: fixed;
  top: 0;
  left: 0;
  margin-top: 0;
  transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
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
