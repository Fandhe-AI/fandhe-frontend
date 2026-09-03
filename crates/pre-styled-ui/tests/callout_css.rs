//! styled Callout（イシュー #994、イシュー #1556 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/alert_css.rs` と同型の golden fixture
//! テスト。イシュー #1556 で size 軸を `--fandhe-callout-*` custom
//! property へ一本化し、variant の配色を palette 6 役割トークンへ移行した
//! ため、出力全体をバイト単位で固定する（`callout.rs` モジュール冒頭
//! rustdoc「参考サイト基準への調整」節参照）。

use fandhe_frontend_pre_styled_ui::callout;

const CALLOUT_GOLDEN_CSS: &str = r#"[data-scope="callout"][data-part="root"] {
  display: flex;
  align-items: flex-start;
  box-sizing: border-box;
  gap: var(--fandhe-callout-gap, var(--fandhe-space-3));
  padding: var(--fandhe-callout-padding, var(--fandhe-space-4));
  border: 1px solid transparent;
  border-radius: var(--fandhe-callout-radius, var(--fandhe-radius-lg));
  font-size: var(--fandhe-callout-font-size, var(--fandhe-font-font-size-sm));
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="callout"][data-part="icon"] {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  height: calc(1em * var(--fandhe-font-line-height-normal));
}

[data-scope="callout"][data-part="text"] {
  min-width: 0;
}

[data-scope="callout"][data-part="root"].fd-callout--size-xs {
  --fandhe-callout-padding: var(--fandhe-space-2);
  --fandhe-callout-gap: var(--fandhe-space-2);
  --fandhe-callout-radius: var(--fandhe-radius-sm);
  --fandhe-callout-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="callout"][data-part="root"].fd-callout--size-sm {
  --fandhe-callout-padding: var(--fandhe-space-3);
  --fandhe-callout-gap: var(--fandhe-space-2);
  --fandhe-callout-radius: var(--fandhe-radius-md);
  --fandhe-callout-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="callout"][data-part="root"].fd-callout--size-md {
  --fandhe-callout-padding: var(--fandhe-space-4);
  --fandhe-callout-gap: var(--fandhe-space-3);
  --fandhe-callout-radius: var(--fandhe-radius-lg);
  --fandhe-callout-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="callout"][data-part="root"].fd-callout--size-lg {
  --fandhe-callout-padding: var(--fandhe-space-5);
  --fandhe-callout-gap: var(--fandhe-space-4);
  --fandhe-callout-radius: var(--fandhe-radius-xl);
  --fandhe-callout-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="callout"][data-part="root"].fd-callout--size-xl {
  --fandhe-callout-padding: var(--fandhe-space-6);
  --fandhe-callout-gap: var(--fandhe-space-4);
  --fandhe-callout-radius: var(--fandhe-radius-2xl);
  --fandhe-callout-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-soft {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-surface {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
  border-color: var(--fandhe-palette-muted);
}

[data-scope="callout"][data-part="root"].fd-callout--variant-outline {
  background: transparent;
  color: var(--fandhe-palette-fg-subtle);
  border-color: var(--fandhe-palette-muted);
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
        "fd-callout--size-xs",
        "fd-callout--size-sm",
        "fd-callout--size-md",
        "fd-callout--size-lg",
        "fd-callout--size-xl",
        "fd-callout--color-palette-accent",
        "fd-callout--color-palette-info",
        "fd-callout--color-palette-success",
        "fd-callout--color-palette-warning",
        "fd-callout--color-palette-danger",
        "fd-callout--color-palette-neutral",
    ] {
        assert!(css.contains(class), "missing class {class} in css: {css}");
    }
}

#[test]
fn callout_css_references_theme_tokens_only() {
    let css = callout::css();
    assert!(css.contains("var(--fandhe-palette-fg-subtle)"));
    assert!(css.contains("var(--fandhe-space-4)"));
    assert!(css.contains("var(--fandhe-radius-lg)"));
    // ハードコードされた生カラーリテラル（`#` 始まり）を含まないことを固定する
    // （イシュー #606 方針: 色宣言は必ずテーマトークン経由）。
    assert!(!css.contains('#'));
    // イシュー #1556: padding の生 `rem` リテラルへ後退していないことを固定する
    // （size 軸は `--fandhe-callout-padding` custom property 経由のみ）。
    assert!(!css.contains("padding: 0."));
}

/// イシュー #1556: text slot はもはや size ごとのクラス宣言を持たない
/// （font-size は root からの継承のみで決まる、`text()` の破壊的変更に
/// 対応する CSS 側の契約）。
#[test]
fn callout_css_does_not_declare_size_classes_for_text_slot() {
    let css = callout::css();
    assert!(!css.contains(r#"[data-part="text"].fd-callout--size-"#));
}

#[test]
fn callout_css_is_deterministic() {
    assert_eq!(callout::css(), callout::css());
}
