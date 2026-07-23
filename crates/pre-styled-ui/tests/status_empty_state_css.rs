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
  gap: 0.5rem;
}

[data-scope="status"][data-part="indicator"] {
  width: var(--fandhe-status-dot-size, 0.5rem);
  height: var(--fandhe-status-dot-size, 0.5rem);
  border-radius: var(--fandhe-radius-full);
  background: var(--fandhe-palette);
  flex-shrink: 0;
}

[data-scope="status"][data-part="root"].fd-status--size-sm {
  font-size: var(--fandhe-font-font-size-xs);
  --fandhe-status-dot-size: 0.375rem;
}

[data-scope="status"][data-part="root"].fd-status--size-md {
  font-size: var(--fandhe-font-font-size-sm);
  --fandhe-status-dot-size: 0.5rem;
}

[data-scope="status"][data-part="root"].fd-status--size-lg {
  font-size: var(--fandhe-font-font-size-md);
  --fandhe-status-dot-size: 0.625rem;
}

[data-scope="status"][data-part="root"].fd-status--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="status"][data-part="root"].fd-status--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}
"#;

const EMPTY_STATE_GOLDEN_CSS: &str = r#"[data-scope="empty-state"][data-part="root"] {
  display: flex;
  align-items: center;
  justify-content: center;
}

[data-scope="empty-state"][data-part="content"] {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fandhe-space-2, 0.5rem);
  text-align: center;
}

[data-scope="empty-state"][data-part="indicator"] {
  font-size: 2rem;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="empty-state"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-semibold);
}

[data-scope="empty-state"][data-part="description"] {
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="empty-state"][data-part="actions"] {
  display: flex;
  gap: var(--fandhe-space-2, 0.5rem);
  margin-top: var(--fandhe-space-2, 0.5rem);
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-sm {
  padding: 2rem;
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-md {
  padding: 3rem;
}

[data-scope="empty-state"][data-part="root"].fd-empty-state--size-lg {
  padding: 4rem;
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
