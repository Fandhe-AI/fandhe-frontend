//! styled Tag / Kbd / Code（イシュー #768）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の複数部品同居前例に
//! 倣い、3 部品の `css()` が返す CSS 全文をバイト単位で固定する（受け入れ
//! 条件「3 部品の golden CSS」）。出力順（base → variants）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::{code, kbd, tag};

const TAG_GOLDEN_CSS: &str = r#"[data-scope="tag"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  border-radius: var(--fandhe-radius-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="tag"][data-part="label"] {
  display: inline-flex;
  align-items: center;
}

[data-scope="tag"][data-part="close-trigger"] {
  display: inline-flex;
  align-items: center;
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 0;
  color: inherit;
}

[data-scope="tag"][data-part="root"].fd-tag--size-sm {
  padding: 0.0625rem 0.375rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tag"][data-part="root"].fd-tag--size-md {
  padding: 0.125rem 0.5rem;
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tag"][data-part="root"].fd-tag--size-lg {
  padding: 0.25rem 0.625rem;
  font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tag"][data-part="root"].fd-tag--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
}

[data-scope="tag"][data-part="root"].fd-tag--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
}

[data-scope="tag"][data-part="root"].fd-tag--variant-outline {
  background: transparent;
  color: var(--fandhe-palette);
  border: 1px solid var(--fandhe-color-border);
}

[data-scope="tag"][data-part="root"].fd-tag--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
}

[data-scope="tag"][data-part="root"].fd-tag--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
}

[data-scope="tag"][data-part="root"].fd-tag--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
}

[data-scope="tag"][data-part="root"].fd-tag--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
}

[data-scope="tag"][data-part="root"].fd-tag--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
}
"#;

const KBD_GOLDEN_CSS: &str = r#"[data-scope="kbd"][data-part="root"] {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  background: var(--fandhe-color-bg-subtle);
  border: 1px solid var(--fandhe-color-border);
  border-bottom-width: 2px;
  border-radius: var(--fandhe-radius-sm);
  padding: 0.0625rem 0.375rem;
  font-size: var(--fandhe-font-font-size-xs);
}
"#;

const CODE_GOLDEN_CSS: &str = r#"[data-scope="code"][data-part="root"] {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  background: var(--fandhe-color-bg-subtle);
  border-radius: var(--fandhe-radius-sm);
  padding: 0.0625rem 0.375rem;
  font-size: var(--fandhe-font-font-size-sm);
}
"#;

#[test]
fn tag_css_matches_golden_fixture() {
    assert_eq!(tag::css(), TAG_GOLDEN_CSS);
}

#[test]
fn kbd_css_matches_golden_fixture() {
    assert_eq!(kbd::css(), KBD_GOLDEN_CSS);
}

#[test]
fn code_css_matches_golden_fixture() {
    assert_eq!(code::css(), CODE_GOLDEN_CSS);
}

#[test]
fn all_three_css_outputs_are_deterministic_across_calls() {
    assert_eq!(tag::css(), tag::css());
    assert_eq!(kbd::css(), kbd::css());
    assert_eq!(code::css(), code::css());
}
