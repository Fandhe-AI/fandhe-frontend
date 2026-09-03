//! styled Spinner（イシュー #550、イシュー #1567 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skeleton_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。base（track
//! 透明化・半周弧・`--fandhe-spinner-size` 経由の寸法）→ size 5 段
//! （xs/sm/md/lg/xl）→ colorPalette 6 段 → `@keyframes` →
//! `@media (prefers-reduced-motion: reduce)` の出力順が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する（`spinner.rs` モジュール doc「参照サイトとの差分（イシュー
//! #1567）」節参照）。

use fandhe_frontend_pre_styled_ui::spinner;

const SPINNER_GOLDEN_CSS: &str = r#"[data-scope="spinner"][data-part="root"] {
  display: inline-block;
  border-radius: var(--fandhe-radius-full);
  border: 2px solid var(--fandhe-spinner-track-color, transparent);
  border-top-color: var(--fandhe-palette);
  border-inline-end-color: var(--fandhe-palette);
  width: var(--fandhe-spinner-size, 1.25rem);
  height: var(--fandhe-spinner-size, 1.25rem);
  animation: fd-spinner-spin 0.6s linear infinite;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-xs {
  --fandhe-spinner-size: 0.75rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-sm {
  --fandhe-spinner-size: 1rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-md {
  --fandhe-spinner-size: 1.25rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-lg {
  --fandhe-spinner-size: 2rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-xl {
  --fandhe-spinner-size: 2.5rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

@keyframes fd-spinner-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  [data-scope="spinner"][data-part="root"] {
    animation: none;
  }
}
"#;

#[test]
fn spinner_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(spinner::css(), SPINNER_GOLDEN_CSS);
}

/// 決定性: 呼び出しごとに完全一致すること（`crate` 冒頭の不変条件 2）。
#[test]
fn spinner_css_output_is_deterministic() {
    assert_eq!(spinner::css(), spinner::css());
}

/// `css()` は静的リテラルのみの連結であり、`<style>` タグからの
/// エスケープ（style breakout）を構造的に許さないことを固定する
/// （`crates/pre-styled-ui/tests/breadcrumb_css.rs` 準拠）。
#[test]
fn spinner_css_never_contains_style_breakout_sequences() {
    let css = spinner::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
