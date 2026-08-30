//! styled Switch（イシュー #682、`size`/`palette` variant 拡張は #708）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 3）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//!
//! `control` の `box-sizing: border-box` は PR #697 Cursor Bugbot 指摘
//! （review 3636964684）対応で追加した宣言。詳細は
//! `crates/pre-styled-ui/src/switch.rs` のモジュール doc を参照。
//!
//! イシュー #708: `control`/`thumb`/`label` の寸法・書体は `root` の
//! `size`/`palette` variant が登録する root スコープ CSS custom property
//! （`--fandhe-switch-*`/`--fandhe-palette*`）を `var(..., <Md/Accent 既定値>)`
//! で参照する形へ変更した（フォールバック値は変更前の固定値と同一、
//! headless 直接利用時の現行外観を維持する）。

use fandhe_frontend_pre_styled_ui::switch;

const SWITCH_GOLDEN_CSS: &str = r#"[data-scope="switch"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  cursor: pointer;
}

[data-scope="switch"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: var(--fandhe-switch-track-width, 2.5rem);
  height: var(--fandhe-switch-track-height, 1.4rem);
  border-radius: 999px;
  background: var(--fandhe-color-border);
  padding: 0 0.15rem;
  transition: background 0.15s;
}

[data-scope="switch"][data-part="thumb"] {
  width: var(--fandhe-switch-thumb-size, 1.1rem);
  height: var(--fandhe-switch-thumb-size, 1.1rem);
  border-radius: 999px;
  background: var(--fandhe-color-bg);
  transition: transform 0.15s;
}

[data-scope="switch"][data-part="label"] {
  font-size: var(--fandhe-switch-label-font-size, var(--fandhe-font-font-size-sm));
}

[data-scope="switch"][data-part="hidden-input"] {
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

[data-scope="switch"][data-part="root"].fd-switch--size-xs {
  --fandhe-switch-track-width: 1.5rem;
  --fandhe-switch-track-height: 0.9rem;
  --fandhe-switch-thumb-size: 0.6rem;
  --fandhe-switch-thumb-travel: 0.6rem;
  --fandhe-switch-label-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="switch"][data-part="root"].fd-switch--size-sm {
  --fandhe-switch-track-width: 2rem;
  --fandhe-switch-track-height: 1.15rem;
  --fandhe-switch-thumb-size: 0.85rem;
  --fandhe-switch-thumb-travel: 0.85rem;
  --fandhe-switch-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="switch"][data-part="root"].fd-switch--size-md {
  --fandhe-switch-track-width: 2.5rem;
  --fandhe-switch-track-height: 1.4rem;
  --fandhe-switch-thumb-size: 1.1rem;
  --fandhe-switch-thumb-travel: 1.1rem;
  --fandhe-switch-label-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="switch"][data-part="root"].fd-switch--size-lg {
  --fandhe-switch-track-width: 3rem;
  --fandhe-switch-track-height: 1.65rem;
  --fandhe-switch-thumb-size: 1.35rem;
  --fandhe-switch-thumb-travel: 1.35rem;
  --fandhe-switch-label-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="switch"][data-part="root"].fd-switch--size-xl {
  --fandhe-switch-track-width: 3.5rem;
  --fandhe-switch-track-height: 1.9rem;
  --fandhe-switch-thumb-size: 1.6rem;
  --fandhe-switch-thumb-travel: 1.6rem;
  --fandhe-switch-label-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="switch"][data-part="root"].fd-switch--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="switch"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="switch"][data-part="control"][data-state="checked"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="switch"][data-part="control"][data-focus-visible] {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="switch"][data-part="thumb"][data-state="checked"] {
  transform: translateX(var(--fandhe-switch-thumb-travel, 1.1rem));
}
"#;

#[test]
fn switch_stylesheet_matches_golden_fixture() {
    assert_eq!(switch::stylesheet(), SWITCH_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / popover_tooltip_css.rs と同観点: 独立呼び出し
    // 間でバイト単位の一致を固定する。
    assert_eq!(switch::stylesheet(), switch::stylesheet());
}
