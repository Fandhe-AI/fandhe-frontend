//! styled Highlight（イシュー #775、イシュー #1435 で variant/palette 軸を
//! 追加）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/mark_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。`root` は
//! 素通しのコンテナのため規則を持たず、`mark` slot の base（余白・角丸）・
//! variant 4 種・colorPalette 6 種の宣言が含まれる
//! （`crates/pre-styled-ui/src/highlight.rs` の recipe 定義参照）。

use fandhe_frontend_pre_styled_ui::highlight;

const HIGHLIGHT_GOLDEN_CSS: &str = r#"[data-scope="highlight"][data-part="mark"] {
  padding-inline: 0.25em;
  border-radius: var(--fandhe-radius-sm);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--variant-solid {
  background: var(--fandhe-palette);
  color: var(--fandhe-palette-fg);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--variant-text {
  background: transparent;
  color: var(--fandhe-palette);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--variant-plain {
  background: transparent;
  color: inherit;
  padding-inline: 0;
  border-radius: 0;
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="highlight"][data-part="mark"].fd-highlight--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}
"#;

#[test]
fn highlight_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(highlight::css(), HIGHLIGHT_GOLDEN_CSS);
}
