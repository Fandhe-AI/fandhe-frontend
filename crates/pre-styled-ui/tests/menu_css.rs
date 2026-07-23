//! styled Menu（`size` variant 展開、イシュー #729）の決定的 CSS 出力
//! ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `trigger`/`item`/`content` の padding は `root` の `size` variant が
//! 登録する root スコープ CSS custom property（`--fandhe-menu-trigger-padding`/
//! `-item-padding`/`-content-padding`）を `var(..., <Md 既定値>)` で参照する
//! 形へ変更した（フォールバック値は変更前の固定値と同一、headless 直接
//! 利用時の現行外観を維持する）。`--fandhe-reference-width`/`--fandhe-arrow-*`/
//! `--fandhe-x`/`--fandhe-y`（wasm positioning 契約、#663/#588）は不変。
//! menu は `color-palette` 軸を持たない。

use fandhe_frontend_pre_styled_ui::menu;

const MENU_GOLDEN_CSS: &str = r#"[data-scope="menu"][data-part="root"] {
  position: relative;
}

[data-scope="menu"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  padding: var(--fandhe-menu-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-3));
}

[data-scope="menu"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 10;
  margin-top: var(--fandhe-space-1);
}

[data-scope="menu"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  padding: var(--fandhe-menu-content-padding, var(--fandhe-space-2));
  min-width: var(--fandhe-reference-width, 10rem);
}

[data-scope="menu"][data-part="arrow"] {
  position: absolute;
  left: var(--fandhe-arrow-x, 50%);
  top: var(--fandhe-arrow-y, 0);
  transform: translate(-50%, -50%);
}

[data-scope="menu"][data-part="arrow-tip"] {
  width: 0.5rem;
  height: 0.5rem;
  background: var(--fandhe-color-bg);
  border-left: 1px solid var(--fandhe-color-border);
  border-top: 1px solid var(--fandhe-color-border);
  transform: rotate(45deg);
}

[data-scope="menu"][data-part="item"] {
  padding: var(--fandhe-menu-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));
  cursor: pointer;
  border-radius: 0.25rem;
}

[data-scope="menu"][data-part="item-group-label"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="menu"][data-part="separator"] {
  border: 0;
  border-top: 1px solid var(--fandhe-color-border-muted);
  margin: var(--fandhe-space-2) 0;
}

[data-scope="menu"][data-part="root"].fd-menu--size-sm {
  --fandhe-menu-trigger-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-menu-item-padding: var(--fandhe-space-1) var(--fandhe-space-2);
  --fandhe-menu-content-padding: var(--fandhe-space-1);
}

[data-scope="menu"][data-part="root"].fd-menu--size-md {
  --fandhe-menu-trigger-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-menu-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  --fandhe-menu-content-padding: var(--fandhe-space-2);
}

[data-scope="menu"][data-part="root"].fd-menu--size-lg {
  --fandhe-menu-trigger-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-menu-item-padding: var(--fandhe-space-3) var(--fandhe-space-4);
  --fandhe-menu-content-padding: var(--fandhe-space-3);
}

[data-scope="menu"][data-part="trigger"][data-state="open"] {
  border-color: var(--fandhe-color-accent);
}

[data-scope="menu"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="menu"][data-part="item"][data-highlighted] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
}

[data-scope="menu"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="menu"][data-part="positioner"][data-positioned] {
  position: fixed;
  top: 0;
  left: 0;
  margin-top: 0;
  transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);
}
"#;

#[test]
fn menu_stylesheet_matches_golden_fixture() {
    assert_eq!(menu::stylesheet(), MENU_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(menu::stylesheet(), menu::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = menu::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
