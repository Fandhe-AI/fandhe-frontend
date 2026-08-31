//! Button（イシュー #830 で icon-only 修飾 variant を追加）の golden CSS
//! テスト。
//!
//! `crates/pre-styled-ui/src/button.rs` は単一 recipe styled 部品として
//! `crates/pre-styled-ui/tests/*_css.rs`（`image_icon_css.rs`・
//! `download_trigger_css.rs` 等）と同型の「CSS 全文をバイト単位で固定する」
//! golden テストを従来持っていなかった。イシュー #830 で icon-only 修飾
//! variant（非公開 `ButtonIcon` 軸）・compound variant 3 件を追加した機会に
//! `button::css()` の golden を新設し、以後の宣言変更（既存 variant の
//! 誤った書き換え・compound variant の意図しない追加削除）を機械的に検知
//! できるようにする（イシュー #830 受け入れ条件 2「golden CSS 再固定」）。

use fandhe_frontend_pre_styled_ui::button;

/// `button::css()` の期待値（バイト完全一致）。
///
/// 出力順は `SlotRecipe::css`（`crates/pre-styled-ui/src/recipe.rs`）の
/// 契約どおり「base → variants（登録順: size → variant → color-palette →
/// icon-only）→ states（登録順: disabled → focus-visible。hover のみ
/// `@media (hover: hover)` へ集約され常に末尾）」。size variant は
/// イシュー #1449 で `--fandhe-size-control-*` トークン（イシュー #1678
/// 新設）を参照するよう変更し、icon-only は 5 段の均等 padding
/// compound variant を `padding: 0` へ簡約した（`button.rs` モジュール
/// 冒頭 rustdoc「size スケール・icon-only・loading」節参照）。
const EXPECTED_CSS: &str = r#"[data-scope="button"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: var(--fandhe-radius-md);
  font-family: var(--fandhe-font-font-body);
  cursor: pointer;
  text-decoration: none;
}

[data-scope="button"][data-part="root"] {
  transition-property: background, border-color, color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="button"][data-part="root"].fd-button--size-xs {
  height: var(--fandhe-size-control-height-xs, 2rem);
  padding: 0 var(--fandhe-size-control-padding-x-xs, 0.625rem);
  font-size: var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs));
}

[data-scope="button"][data-part="root"].fd-button--size-sm {
  height: var(--fandhe-size-control-height-sm, 2.25rem);
  padding: 0 var(--fandhe-size-control-padding-x-sm, 0.75rem);
  font-size: var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm));
}

[data-scope="button"][data-part="root"].fd-button--size-md {
  height: var(--fandhe-size-control-height-md, 2.5rem);
  padding: 0 var(--fandhe-size-control-padding-x-md, 1rem);
  font-size: var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md));
}

[data-scope="button"][data-part="root"].fd-button--size-lg {
  height: var(--fandhe-size-control-height-lg, 2.75rem);
  padding: 0 var(--fandhe-size-control-padding-x-lg, 1.25rem);
  font-size: var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg));
}

[data-scope="button"][data-part="root"].fd-button--size-xl {
  height: var(--fandhe-size-control-height-xl, 3rem);
  padding: 0 var(--fandhe-size-control-padding-x-xl, 1.5rem);
  font-size: var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl));
}

[data-scope="button"][data-part="root"].fd-button--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
  border: none;
  --fandhe-hover-bg: var(--fandhe-palette-emphasized);
}

[data-scope="button"][data-part="root"].fd-button--variant-outline {
  background: transparent;
  color: var(--fandhe-palette);
  border: 1px solid var(--fandhe-palette);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-ghost {
  background: transparent;
  color: var(--fandhe-palette);
  border: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="button"][data-part="root"].fd-button--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
  border: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="button"][data-part="root"].fd-button--icon-only {
  aspect-ratio: 1 / 1;
  padding: 0;
}

[data-scope="button"][data-part="root"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="button"][data-part="root"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="button"][data-part="root"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn button_css_matches_golden_byte_for_byte() {
    assert_eq!(
        button::css(),
        EXPECTED_CSS,
        "button::css() の出力が golden と一致しない。意図した宣言変更なら \
         EXPECTED_CSS を更新すること（本ファイル冒頭 rustdoc 参照）"
    );
}

#[test]
fn button_css_is_deterministic() {
    assert_eq!(button::css(), button::css());
}
