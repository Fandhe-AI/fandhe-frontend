//! styled Toggle（イシュー #746、`size`/`palette` variant は #1512、
//! `variant`（Outline/Ghost）軸新設は #2023）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! toggle は #2023 以前は golden 不在の部品だった
//! （`docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3）ため、
//! 本ファイルが初回の golden 新設となる。
//!
//! イシュー #2023: shadcn/ui（https://ui.shadcn.com/docs/components/base/toggle）
//! との突合で `variant: "default" | "outline"` 相当の軸が欠落していたこと
//! を確認し、`ToggleVariant`（`Outline`/`Ghost`）を新設した
//! （`crates/pre-styled-ui/src/toggle.rs` のモジュール doc「shadcn/ui
//! 突合（イシュー #2023）」節参照）。`Ghost` は `border-color`/`background`
//! の 2 宣言を上書きする variant 規則としてのみ追加し、既存 base/state/
//! palette/size 規則は一切変更しない（純追加原則）。

use fandhe_frontend_pre_styled_ui::toggle;

const TOGGLE_GOLDEN_CSS: &str = r#"[data-scope="toggle"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  padding: var(--fandhe-toggle-padding-y, 0.375rem) var(--fandhe-toggle-padding-x, 0.75rem);
  font-size: var(--fandhe-toggle-font-size, var(--fandhe-font-font-size-sm));
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="toggle"][data-part="root"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toggle"][data-part="indicator"] {
  display: inline-flex;
}

[data-scope="toggle"][data-part="root"].fd-toggle--size-xs {
  --fandhe-toggle-padding-y: 0.125rem;
  --fandhe-toggle-padding-x: 0.25rem;
  --fandhe-toggle-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="toggle"][data-part="root"].fd-toggle--size-sm {
  --fandhe-toggle-padding-y: 0.25rem;
  --fandhe-toggle-padding-x: 0.5rem;
  --fandhe-toggle-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toggle"][data-part="root"].fd-toggle--size-md {
  --fandhe-toggle-padding-y: 0.375rem;
  --fandhe-toggle-padding-x: 0.75rem;
  --fandhe-toggle-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toggle"][data-part="root"].fd-toggle--size-lg {
  --fandhe-toggle-padding-y: 0.5rem;
  --fandhe-toggle-padding-x: 1rem;
  --fandhe-toggle-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="toggle"][data-part="root"].fd-toggle--size-xl {
  --fandhe-toggle-padding-y: 0.625rem;
  --fandhe-toggle-padding-x: 1.25rem;
  --fandhe-toggle-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="toggle"][data-part="root"].fd-toggle--variant-ghost {
  border-color: transparent;
  background: transparent;
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="toggle"][data-part="root"].fd-toggle--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="toggle"][data-part="root"][data-state="on"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg);
  --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
}

[data-scope="toggle"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toggle"][data-part="root"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="toggle"][data-part="indicator"][data-state="off"] {
  display: none;
}

@media (hover: hover) {
  [data-scope="toggle"][data-part="root"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn toggle_stylesheet_matches_golden_fixture() {
    assert_eq!(toggle::stylesheet(), TOGGLE_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / switch_css.rs と同観点: 独立呼び出し間でバイト
    // 単位の一致を固定する。
    assert_eq!(toggle::stylesheet(), toggle::stylesheet());
}
