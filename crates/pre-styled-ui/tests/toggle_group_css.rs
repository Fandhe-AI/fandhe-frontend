//! styled ToggleGroup（イシュー #746、`size`/`palette` variant は #1513、
//! `variant`（Outline/Ghost）軸新設は #2024）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/toggle_css.rs`（イシュー #2023 で新設）の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する（`docs/internal/pre-styled-ui-golden-test-update-guide.md`
//! §2.1 方式 (a)）。toggle_group は #2024 以前は golden 不在の部品だった
//! （同ガイド §3.3）ため、本ファイルが初回の golden 新設となる。
//!
//! イシュー #2024: shadcn/ui（`https://ui.shadcn.com/docs/components/base/toggle-group`）
//! との突合で `variant: "default" | "outline"` 相当の軸が欠落していたこと
//! を確認し、`ToggleGroupVariant`（`Outline`/`Ghost`、#2023 の
//! `crate::toggle::ToggleVariant` と同一命名）を新設した
//! （`crates/pre-styled-ui/src/toggle_group.rs` のモジュール doc「shadcn/ui
//! 突合（イシュー #2024）」節参照）。variant クラスは `root` パーツのみへ
//! 付与され、`item` パーツの border-color/background は CSS カスタム
//! プロパティ（`--fandhe-toggle-group-item-border-color`/
//! `-item-background`）経由で伝播する。既定 `Outline` はフォールバック値が
//! 旧リテラルと computed style 上同一だが、`var()` 化により CSS 文字列は
//! 変わる（純追加ではあるが既定 variant の golden 出力バイトが変化する
//! 仕様）。shadcn の `spacing`（連結/分離セグメント）軸は既存の常時連結
//! セグメント表現と対応するため本イシューでは追加しない（両軸直交・API
//! 破壊的変更最小化のため 1 軸に絞る判断、モジュール doc 参照）。

use fandhe_frontend_pre_styled_ui::toggle_group;

const TOGGLE_GROUP_GOLDEN_CSS: &str = r#"[data-scope="toggle-group"][data-part="root"] {
  display: inline-flex;
  gap: 0;
}

[data-scope="toggle-group"][data-part="item"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  border: 1px solid var(--fandhe-toggle-group-item-border-color, var(--fandhe-color-border));
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-toggle-group-item-background, var(--fandhe-color-bg));
  color: var(--fandhe-color-fg);
  padding: var(--fandhe-toggle-group-item-padding-y, 0.375rem) var(--fandhe-toggle-group-item-padding-x, 0.75rem);
  font-size: var(--fandhe-toggle-group-item-font-size, var(--fandhe-font-font-size-sm));
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--size-xs {
  --fandhe-toggle-group-item-padding-y: 0.125rem;
  --fandhe-toggle-group-item-padding-x: 0.25rem;
  --fandhe-toggle-group-item-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--size-sm {
  --fandhe-toggle-group-item-padding-y: 0.25rem;
  --fandhe-toggle-group-item-padding-x: 0.5rem;
  --fandhe-toggle-group-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--size-md {
  --fandhe-toggle-group-item-padding-y: 0.375rem;
  --fandhe-toggle-group-item-padding-x: 0.75rem;
  --fandhe-toggle-group-item-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--size-lg {
  --fandhe-toggle-group-item-padding-y: 0.5rem;
  --fandhe-toggle-group-item-padding-x: 1rem;
  --fandhe-toggle-group-item-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--size-xl {
  --fandhe-toggle-group-item-padding-y: 0.625rem;
  --fandhe-toggle-group-item-padding-x: 1.25rem;
  --fandhe-toggle-group-item-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--variant-ghost {
  --fandhe-toggle-group-item-border-color: transparent;
  --fandhe-toggle-group-item-background: transparent;
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"].fd-toggle-group--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="toggle-group"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toggle-group"][data-part="item"][data-state="on"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  color: var(--fandhe-palette-fg);
  position: relative;
  z-index: 1;
}

[data-scope="toggle-group"][data-part="item"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toggle-group"][data-part="item"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
  position: relative;
  z-index: 1;
}

[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child {
  border-start-start-radius: var(--fandhe-radius-md);
  border-end-start-radius: var(--fandhe-radius-md);
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}
[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:last-child {
  border-start-end-radius: var(--fandhe-radius-md);
  border-end-end-radius: var(--fandhe-radius-md);
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}
[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:not(:first-child):not(:last-child) {
  border-radius: 0;
}
[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {
  border-start-start-radius: var(--fandhe-radius-md);
  border-end-start-radius: var(--fandhe-radius-md);
  border-start-end-radius: var(--fandhe-radius-md);
  border-end-end-radius: var(--fandhe-radius-md);
}
[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"] + [data-scope="toggle-group"][data-part="item"] {
  margin-inline-start: -1px;
}
[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:first-child {
  border-start-start-radius: var(--fandhe-radius-md);
  border-start-end-radius: var(--fandhe-radius-md);
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}
[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:last-child {
  border-end-start-radius: var(--fandhe-radius-md);
  border-end-end-radius: var(--fandhe-radius-md);
  border-start-start-radius: 0;
  border-start-end-radius: 0;
}
[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {
  border-start-start-radius: var(--fandhe-radius-md);
  border-start-end-radius: var(--fandhe-radius-md);
  border-end-start-radius: var(--fandhe-radius-md);
  border-end-end-radius: var(--fandhe-radius-md);
}
[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"] + [data-scope="toggle-group"][data-part="item"] {
  margin-inline-start: 0;
  margin-block-start: -1px;
}

@media (hover: hover) {
  [data-scope="toggle-group"][data-part="root"]:not([data-disabled]) > [data-scope="toggle-group"][data-part="item"]:hover:not([data-disabled]):not([data-state="on"]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn toggle_group_stylesheet_matches_golden_css() {
    assert_eq!(toggle_group::stylesheet(), TOGGLE_GROUP_GOLDEN_CSS);
}
