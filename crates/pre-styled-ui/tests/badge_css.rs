//! styled Badge（イシュー #1555、参考サイト基準へのスタイル調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/avatar_css.rs` の golden fixture テスト
//! の前例に倣い、`badge::css()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → size → variant →
//! color-palette）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。
//! `docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3 が
//! 新規追加の必要性を指摘していた「golden 不在」の 1 件を埋める。

use fandhe_frontend_pre_styled_ui::badge;

const BADGE_GOLDEN_CSS: &str = r#"[data-scope="badge"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  border-radius: var(--fandhe-radius-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-tight);
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

[data-scope="badge"][data-part="root"].fd-badge--size-xs {
  padding: 0.03125rem 0.25rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="badge"][data-part="root"].fd-badge--size-sm {
  padding: 0.0625rem 0.375rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="badge"][data-part="root"].fd-badge--size-md {
  padding: 0.125rem 0.5rem;
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="badge"][data-part="root"].fd-badge--size-lg {
  padding: 0.25rem 0.625rem;
  font-size: var(--fandhe-font-font-size-md);
}

[data-scope="badge"][data-part="root"].fd-badge--size-xl {
  padding: 0.5rem 0.75rem;
  font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="badge"][data-part="root"].fd-badge--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
}

[data-scope="badge"][data-part="root"].fd-badge--variant-subtle {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--variant-outline {
  background: transparent;
  color: var(--fandhe-palette-fg-subtle);
  border: 1px solid var(--fandhe-palette-muted);
}

[data-scope="badge"][data-part="root"].fd-badge--variant-surface {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
  border: 1px solid var(--fandhe-palette-muted);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="badge"][data-part="root"].fd-badge--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}
"#;

#[test]
fn badge_css_matches_golden_fixture() {
    assert_eq!(badge::css(), BADGE_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(badge::css(), badge::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = badge::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
