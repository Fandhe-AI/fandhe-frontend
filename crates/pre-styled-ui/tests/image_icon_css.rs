//! styled Image/Icon（イシュー #770）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の golden fixture
//! テストの前例に倣い、2 部品分の CSS 全文を 1 ファイルへまとめてバイト
//! 単位で固定する（受け入れ条件 1: golden CSS・Size variant）。出力順
//! （base → variants、`crate::recipe` モジュール doc の順序規約）が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。

use fandhe_frontend_pre_styled_ui::{icon, image};

const IMAGE_GOLDEN_CSS: &str = r#"[data-scope="image"][data-part="root"] {
  display: block;
  max-width: 100%;
  height: auto;
}

[data-scope="image"][data-part="root"].fd-image--fit-cover {
  object-fit: cover;
}

[data-scope="image"][data-part="root"].fd-image--fit-contain {
  object-fit: contain;
}

[data-scope="image"][data-part="root"].fd-image--fit-fill {
  object-fit: fill;
}

[data-scope="image"][data-part="root"].fd-image--fit-scale-down {
  object-fit: scale-down;
}

[data-scope="image"][data-part="root"].fd-image--fit-none {
  object-fit: none;
}

[data-scope="image"][data-part="root"].fd-image--aspect-ratio-auto {
  aspect-ratio: auto;
}

[data-scope="image"][data-part="root"].fd-image--aspect-ratio-square {
  aspect-ratio: 1 / 1;
}

[data-scope="image"][data-part="root"].fd-image--aspect-ratio-landscape {
  aspect-ratio: 4 / 3;
}

[data-scope="image"][data-part="root"].fd-image--aspect-ratio-portrait {
  aspect-ratio: 3 / 4;
}

[data-scope="image"][data-part="root"].fd-image--aspect-ratio-video {
  aspect-ratio: 16 / 9;
}

[data-scope="image"][data-part="root"].fd-image--shape-square {
  border-radius: var(--fandhe-radius-none);
}

[data-scope="image"][data-part="root"].fd-image--shape-rounded {
  border-radius: var(--fandhe-radius-md);
}

[data-scope="image"][data-part="root"].fd-image--shape-circle {
  border-radius: var(--fandhe-radius-full);
}
"#;

const ICON_GOLDEN_CSS: &str = r#"[data-scope="icon"][data-part="root"] {
  display: inline-block;
  flex-shrink: 0;
  color: currentColor;
  vertical-align: middle;
}

[data-scope="icon"][data-part="root"].fd-icon--size-xs {
  width: 0.75rem;
  height: 0.75rem;
}

[data-scope="icon"][data-part="root"].fd-icon--size-sm {
  width: 1rem;
  height: 1rem;
}

[data-scope="icon"][data-part="root"].fd-icon--size-md {
  width: 1.25rem;
  height: 1.25rem;
}

[data-scope="icon"][data-part="root"].fd-icon--size-lg {
  width: 1.5rem;
  height: 1.5rem;
}

[data-scope="icon"][data-part="root"].fd-icon--size-xl {
  width: 1.75rem;
  height: 1.75rem;
}
"#;

#[test]
fn image_css_matches_golden_fixture() {
    assert_eq!(image::css(), IMAGE_GOLDEN_CSS);
}

#[test]
fn icon_css_matches_golden_fixture() {
    assert_eq!(icon::css(), ICON_GOLDEN_CSS);
}

#[test]
fn image_css_is_byte_identical_across_calls() {
    assert_eq!(image::css(), image::css());
}

#[test]
fn icon_css_is_byte_identical_across_calls() {
    assert_eq!(icon::css(), icon::css());
}
