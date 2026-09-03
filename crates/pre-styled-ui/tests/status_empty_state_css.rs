//! styled Status / EmptyState（イシュー #765）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/toggle_tip_css.rs`（複数部品を 1 ファイルへ
//! まとめる形式は `form_controls_css.rs`/`popover_tooltip_css.rs` の前例）
//! に倣い、両部品の `css()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants、`crate::recipe::SlotRecipe::css` の doc コメント参照）
//! が崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::{empty_state, status};

const STATUS_GOLDEN_CSS: &str = r#"[data-scope="status"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2, 0.5rem);
}

[data-scope="status"][data-part="indicator"] {
  box-sizing: border-box;
  width: var(--fandhe-status-dot-size, 0.5rem);
  height: var(--fandhe-status-dot-size, 0.5rem);
  border-radius: var(--fandhe-radius-full);
  background: var(--fandhe-palette);
  flex-shrink: 0;
}

[data-scope="status"][data-part="root"].fd-status--size-xs {
  font-size: var(--fandhe-font-font-size-xs);
  --fandhe-status-dot-size: var(--fandhe-space-1, 0.25rem);
}

[data-scope="status"][data-part="root"].fd-status--size-sm {
  font-size: var(--fandhe-font-font-size-xs);
  --fandhe-status-dot-size: var(--fandhe-space-1-5, 0.375rem);
}

[data-scope="status"][data-part="root"].fd-status--size-md {
  font-size: var(--fandhe-font-font-size-sm);
  --fandhe-status-dot-size: var(--fandhe-space-2, 0.5rem);
}

[data-scope="status"][data-part="root"].fd-status--size-lg {
  font-size: var(--fandhe-font-font-size-md);
  --fandhe-status-dot-size: var(--fandhe-space-2-5, 0.625rem);
}

[data-scope="status"][data-part="root"].fd-status--size-xl {
  font-size: var(--fandhe-font-font-size-lg);
  --fandhe-status-dot-size: var(--fandhe-space-3, 0.75rem);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}


@media (forced-colors: active) {
  [data-scope="status"][data-part="indicator"] {
    border: 1px solid CanvasText;
  }
}
"#;

const EMPTY_STATE_GOLDEN_CSS: &str = r#"[data-scope="empty-state"][data-part="root"] {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 100%;
  padding: var(--fandhe-empty-state-padding, var(--fandhe-space-12) var(--fandhe-space-8));
}

[data-scope="empty-state"][data-part="content"] {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-empty-state-gap, var(--fandhe-space-2));
  text-align: center;
}

[data-scope="empty-state"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  font-size: var(--fandhe-empty-state-indicator-size, var(--fandhe-font-font-size-4xl));
  color: var(--fandhe-color-fg-subtle);
  margin-bottom: var(--fandhe-empty-state-section-gap, var(--fandhe-space-4));
}

[data-scope="empty-state"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-semibold);
  font-size: var(--fandhe-empty-state-title-size, var(--fandhe-font-font-size-lg));
}

[data-scope="empty-state"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-empty-state-description-size, var(--fandhe-font-font-size-sm));
}

[data-scope="empty-state"][data-part="actions"] {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: var(--fandhe-space-2);
  margin-top: var(--fandhe-empty-state-section-gap, var(--fandhe-space-4));
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-xs {
  --fandhe-empty-state-padding: var(--fandhe-space-4) var(--fandhe-space-3);
  --fandhe-empty-state-gap: var(--fandhe-space-1);
  --fandhe-empty-state-section-gap: var(--fandhe-space-2);
  --fandhe-empty-state-indicator-size: var(--fandhe-font-font-size-xl);
  --fandhe-empty-state-title-size: var(--fandhe-font-font-size-sm);
  --fandhe-empty-state-description-size: var(--fandhe-font-font-size-xs);
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-sm {
  --fandhe-empty-state-padding: var(--fandhe-space-6) var(--fandhe-space-4);
  --fandhe-empty-state-gap: var(--fandhe-space-1-5);
  --fandhe-empty-state-section-gap: var(--fandhe-space-2-5);
  --fandhe-empty-state-indicator-size: var(--fandhe-font-font-size-2xl);
  --fandhe-empty-state-title-size: var(--fandhe-font-font-size-md);
  --fandhe-empty-state-description-size: var(--fandhe-font-font-size-xs);
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-md {
  --fandhe-empty-state-padding: var(--fandhe-space-12) var(--fandhe-space-8);
  --fandhe-empty-state-gap: var(--fandhe-space-2);
  --fandhe-empty-state-section-gap: var(--fandhe-space-4);
  --fandhe-empty-state-indicator-size: var(--fandhe-font-font-size-4xl);
  --fandhe-empty-state-title-size: var(--fandhe-font-font-size-lg);
  --fandhe-empty-state-description-size: var(--fandhe-font-font-size-sm);
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-lg {
  --fandhe-empty-state-padding: var(--fandhe-space-16) var(--fandhe-space-12);
  --fandhe-empty-state-gap: var(--fandhe-space-3);
  --fandhe-empty-state-section-gap: var(--fandhe-space-5);
  --fandhe-empty-state-indicator-size: 3.75rem;
  --fandhe-empty-state-title-size: var(--fandhe-font-font-size-xl);
  --fandhe-empty-state-description-size: var(--fandhe-font-font-size-md);
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-xl {
  --fandhe-empty-state-padding: var(--fandhe-space-20) var(--fandhe-space-16);
  --fandhe-empty-state-gap: var(--fandhe-space-4);
  --fandhe-empty-state-section-gap: var(--fandhe-space-6);
  --fandhe-empty-state-indicator-size: 4.5rem;
  --fandhe-empty-state-title-size: var(--fandhe-font-font-size-2xl);
  --fandhe-empty-state-description-size: var(--fandhe-font-font-size-lg);
}
"#;

#[test]
fn status_css_matches_golden_fixture() {
    assert_eq!(status::css(), STATUS_GOLDEN_CSS);
}

#[test]
fn empty_state_css_matches_golden_fixture() {
    assert_eq!(empty_state::css(), EMPTY_STATE_GOLDEN_CSS);
}

#[test]
fn css_output_is_byte_identical_across_calls() {
    // recipe_determinism.rs と同観点: 独立呼び出し間でバイト単位の一致を固定する。
    assert_eq!(status::css(), status::css());
    assert_eq!(empty_state::css(), empty_state::css());
}
