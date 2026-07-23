//! styled Tabs（`size`/`color-palette` variant 展開、イシュー #729）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `trigger`/`content` の寸法は `root` の `size` variant が登録する root
//! スコープ CSS custom property（`--fandhe-tabs-trigger-padding`/
//! `-content-padding`）を `var(..., <Md 既定値>)` で参照する形へ変更した
//! （フォールバック値は変更前の固定値と同一、headless 直接利用時の現行
//! 外観を維持する）。選択中 `trigger` の強調色（`border-bottom-color`）は
//! `color-palette` variant が登録する `--fandhe-palette`（`var(...,
//! var(--fandhe-color-accent))`）経由に変更した（フォールバックは Accent
//! 相当で変更前と同一）。

use fandhe_frontend_pre_styled_ui::tabs;

const TABS_GOLDEN_CSS: &str = r#"[data-scope="tabs"][data-part="list"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tabs"][data-part="trigger"] {
  padding: var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  cursor: pointer;
}

[data-scope="tabs"][data-part="content"] {
  padding: var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0);
  color: var(--fandhe-color-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-sm {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tabs-content-padding: var(--fandhe-space-3) 0;
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-md {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tabs-content-padding: var(--fandhe-space-4) 0;
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-lg {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tabs-content-padding: var(--fandhe-space-5) 0;
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}

[data-scope="tabs"][data-part="trigger"][data-state="active"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="tabs"][data-part="content"][data-state="inactive"] {
  display: none;
}

[data-scope="tabs"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn tabs_stylesheet_matches_golden_fixture() {
    assert_eq!(tabs::stylesheet(), TABS_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tabs::stylesheet(), tabs::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tabs::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
