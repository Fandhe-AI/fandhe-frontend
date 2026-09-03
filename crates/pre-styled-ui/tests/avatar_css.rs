//! styled Avatar（イシュー #1554、参考サイト基準へのスタイル調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/toolbar_css.rs` の golden fixture テスト
//! の前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → variant → state）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//! `docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3 が
//! 新規追加の必要性を指摘していた「golden 不在」の 1 件を埋める。

use fandhe_frontend_pre_styled_ui::avatar;

const AVATAR_GOLDEN_CSS: &str = r#"[data-scope="avatar"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-sizing: border-box;
  overflow: hidden;
  flex-shrink: 0;
  user-select: none;
}

[data-scope="avatar"][data-part="image"] {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
}

[data-scope="avatar"][data-part="fallback"] {
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: 1;
  text-transform: uppercase;
}

[data-scope="avatar"][data-part="root"].fd-avatar--size-xs {
  width: 1.5rem;
  height: 1.5rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="avatar"][data-part="root"].fd-avatar--size-sm {
  width: 2rem;
  height: 2rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="avatar"][data-part="root"].fd-avatar--size-md {
  width: 2.5rem;
  height: 2.5rem;
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="avatar"][data-part="root"].fd-avatar--size-lg {
  width: 3rem;
  height: 3rem;
  font-size: var(--fandhe-font-font-size-md);
}

[data-scope="avatar"][data-part="root"].fd-avatar--size-xl {
  width: 3.5rem;
  height: 3.5rem;
  font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="avatar"][data-part="root"].fd-avatar--shape-circle {
  border-radius: var(--fandhe-radius-full);
}

[data-scope="avatar"][data-part="root"].fd-avatar--shape-rounded {
  border-radius: var(--fandhe-radius-lg);
}

[data-scope="avatar"][data-part="root"].fd-avatar--shape-square {
  border-radius: 0;
}

[data-scope="avatar"][data-part="root"].fd-avatar--variant-subtle {
  background: var(--fandhe-palette-muted);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
}

[data-scope="avatar"][data-part="root"].fd-avatar--variant-outline {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-palette-fg-subtle);
  border: 1px solid var(--fandhe-palette-muted);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="avatar"][data-part="root"].fd-avatar--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="avatar"][data-part="image"][data-state="hidden"] {
  display: none;
}

[data-scope="avatar"][data-part="fallback"][data-state="hidden"] {
  display: none;
}
"#;

#[test]
fn avatar_stylesheet_matches_golden_fixture() {
    assert_eq!(avatar::stylesheet(), AVATAR_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(avatar::stylesheet(), avatar::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = avatar::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
