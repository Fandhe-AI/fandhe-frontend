//! styled ColorSwatch（イシュー #838）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tag_kbd_code_css.rs` の体裁に倣い、`css()` が
//! 返す CSS 全文をバイト単位で固定する。出力順（base → variants）が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。

use fandhe_frontend_pre_styled_ui::color_swatch;

const COLOR_SWATCH_GOLDEN_CSS: &str = r#"[data-scope="color-swatch"][data-part="root"] {
  display: inline-block;
  vertical-align: middle;
  background-image: linear-gradient(var(--fd-swatch-color), var(--fd-swatch-color)), repeating-conic-gradient(var(--fandhe-color-border) 0% 25%, var(--fandhe-color-bg) 0% 50%);
  background-size: 100% 100%, 8px 8px;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--size-xs {
  width: 0.5rem;
  height: 0.5rem;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--size-sm {
  width: 1rem;
  height: 1rem;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--size-md {
  width: 1.5rem;
  height: 1.5rem;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--size-lg {
  width: 2rem;
  height: 2rem;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--size-xl {
  width: 2.5rem;
  height: 2.5rem;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--shape-square {
  border-radius: 0;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--shape-circle {
  border-radius: 9999px;
}

[data-scope="color-swatch"][data-part="root"].fd-color-swatch--shape-rounded {
  border-radius: var(--fandhe-radius-sm);
}
"#;

#[test]
fn color_swatch_css_matches_golden_fixture() {
    assert_eq!(color_swatch::css(), COLOR_SWATCH_GOLDEN_CSS);
}

#[test]
fn color_swatch_css_output_is_deterministic_across_calls() {
    assert_eq!(color_swatch::css(), color_swatch::css());
}

#[test]
fn color_swatch_css_never_contains_style_breakout_sequences() {
    assert!(!color_swatch::css().contains('<'));
}
