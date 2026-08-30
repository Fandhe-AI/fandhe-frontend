//! styled Drawer（イシュー #758）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/dialog_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。placement 4 方向
//! （`data-placement="start"/"end"/"top"/"bottom"`）の layout 規則を含む。

use fandhe_frontend_pre_styled_ui::drawer;

const DRAWER_GOLDEN_CSS: &str = r#"[data-scope="drawer"][data-part="trigger"] {
  cursor: pointer;
  color: var(--fandhe-color-fg);
}

[data-scope="drawer"][data-part="backdrop"] {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
}

[data-scope="drawer"][data-part="positioner"] {
  position: fixed;
  inset: 0;
  z-index: 1001;
  display: flex;
}

[data-scope="drawer"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  padding: var(--fandhe-drawer-content-padding, var(--fandhe-space-6));
  box-sizing: border-box;
  overflow-y: auto;
}

[data-scope="drawer"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-lg);
  font-weight: var(--fandhe-font-font-weight-semibold);
  margin: 0 0 var(--fandhe-space-2) 0;
}

[data-scope="drawer"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  margin: 0;
}

[data-scope="drawer"][data-part="close-trigger"] {
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-xs {
  --fandhe-drawer-size: 12rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-sm {
  --fandhe-drawer-size: 16rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-md {
  --fandhe-drawer-size: 20rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-lg {
  --fandhe-drawer-size: 28rem;
}

[data-scope="drawer"][data-part="root"].fd-drawer--size-xl {
  --fandhe-drawer-size: 36rem;
}

[data-scope="drawer"][data-part="positioner"][data-placement="start"] {
  flex-direction: row;
  justify-content: flex-start;
}

[data-scope="drawer"][data-part="positioner"][data-placement="end"] {
  flex-direction: row;
  justify-content: flex-end;
}

[data-scope="drawer"][data-part="positioner"][data-placement="top"] {
  flex-direction: column;
  justify-content: flex-start;
}

[data-scope="drawer"][data-part="positioner"][data-placement="bottom"] {
  flex-direction: column;
  justify-content: flex-end;
}

[data-scope="drawer"][data-part="content"][data-placement="start"] {
  width: var(--fandhe-drawer-size, 20rem);
  height: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="end"] {
  width: var(--fandhe-drawer-size, 20rem);
  height: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="top"] {
  height: var(--fandhe-drawer-size, 20rem);
  width: 100%;
}

[data-scope="drawer"][data-part="content"][data-placement="bottom"] {
  height: var(--fandhe-drawer-size, 20rem);
  width: 100%;
}

[data-scope="drawer"][data-part="backdrop"][data-state="open"] {
  opacity: 1;
}

[data-scope="drawer"][data-part="backdrop"][data-state="closed"] {
  opacity: 0;
}

[data-scope="drawer"][data-part="content"][data-state="open"] {
  opacity: 1;
}

[data-scope="drawer"][data-part="content"][data-state="closed"] {
  opacity: 0;
}

[data-scope="drawer"][data-part="positioner"][hidden] {
  display: none;
}

[data-scope="drawer"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="drawer"][data-part="close-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn drawer_stylesheet_matches_golden_fixture() {
    assert_eq!(drawer::stylesheet(), DRAWER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(drawer::stylesheet(), drawer::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = drawer::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
