//! styled Callout（イシュー #994）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skip_nav_css.rs`/`tests/separator_css.rs` の
//! golden fixture テストの前例に倣い、`callout::css()` が返す CSS 全文を
//! バイト単位で固定する。加えて受け入れ条件（recipe / CSS 出力の契約検証）
//! として、slot セレクタ・variant/size/color-palette クラス・テーマトークン
//! 参照（ハードコード色を含まないこと）を個別に検証する。

use fandhe_frontend_pre_styled_ui::callout;

const CALLOUT_GOLDEN_CSS: &str = r#"[data-scope="callout"][data-part="root"] {
  display: flex;
  gap: 0.75rem;
  border-radius: var(--fandhe-radius-md);
}

[data-scope="callout"][data-part="icon"] {
  flex-shrink: 0;
}

[data-scope="callout"][data-part="text"] {
  min-width: 0;
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="callout"][data-part="root"].fd-callout--size-xs {
  padding: 0.25rem 0.5rem;
}

[data-scope="callout"][data-part="text"].fd-callout--size-xs {
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="callout"][data-part="root"].fd-callout--size-sm {
  padding: 0.5rem 0.75rem;
}

[data-scope="callout"][data-part="text"].fd-callout--size-sm {
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="callout"][data-part="root"].fd-callout--size-md {
  padding: 0.75rem 1rem;
}

[data-scope="callout"][data-part="text"].fd-callout--size-md {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="callout"][data-part="root"].fd-callout--size-lg {
  padding: 1rem 1.25rem;
}

[data-scope="callout"][data-part="text"].fd-callout--size-lg {
  font-size: var(--fandhe-font-font-size-md);
}

[data-scope="callout"][data-part="root"].fd-callout--size-xl {
  padding: 1.25rem 1.5rem;
}

[data-scope="callout"][data-part="text"].fd-callout--size-xl {
  font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-soft {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-surface {
  background: var(--fandhe-color-bg-subtle);
  color: var(--fandhe-palette);
  border: 1px solid var(--fandhe-color-border);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-outline {
  background: transparent;
  color: var(--fandhe-palette);
  border: 1px solid var(--fandhe-palette);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}
"#;

#[test]
fn callout_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(callout::css(), CALLOUT_GOLDEN_CSS);
}

#[test]
fn callout_css_declares_all_slot_selectors() {
    let css = callout::css();
    assert!(css.contains(r#"[data-scope="callout"][data-part="root"]"#));
    assert!(css.contains(r#"[data-scope="callout"][data-part="icon"]"#));
    assert!(css.contains(r#"[data-scope="callout"][data-part="text"]"#));
}

#[test]
fn callout_css_declares_variant_and_size_and_palette_classes() {
    let css = callout::css();
    for class in [
        "fd-callout--variant-soft",
        "fd-callout--variant-surface",
        "fd-callout--variant-outline",
        "fd-callout--size-sm",
        "fd-callout--size-md",
        "fd-callout--size-lg",
        "fd-callout--color-palette-accent",
        "fd-callout--color-palette-info",
        "fd-callout--color-palette-success",
        "fd-callout--color-palette-warning",
        "fd-callout--color-palette-danger",
    ] {
        assert!(css.contains(class), "missing class {class} in css: {css}");
    }
}

#[test]
fn callout_css_references_theme_tokens_only() {
    let css = callout::css();
    assert!(css.contains("var(--fandhe-palette)"));
    assert!(css.contains("var(--fandhe-radius-md)"));
    // ハードコードされた生カラーリテラル（`#` 始まり）を含まないことを固定する
    // （イシュー #606 方針: 色宣言は必ずテーマトークン経由）。
    assert!(!css.contains('#'));
}

#[test]
fn callout_css_is_deterministic() {
    assert_eq!(callout::css(), callout::css());
}
