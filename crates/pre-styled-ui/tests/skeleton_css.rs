//! styled Skeleton（イシュー #764）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/checkbox_card_css.rs` の golden fixture テスト
//! の前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する（受け入れ条件
//! 1・2）。variant（text/circle/rect）ごとの規則・`@keyframes`・
//! `@media (prefers-reduced-motion: reduce)` を含む全文の出力順が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::skeleton;

const SKELETON_GOLDEN_CSS: &str = r#"[data-scope="skeleton"][data-part="root"] {
  display: block;
  background: var(--fandhe-color-bg-subtle);
  animation: fd-skeleton-pulse 1.5s ease-in-out infinite;
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--variant-text {
  width: 100%;
  height: 1em;
  border-radius: var(--fandhe-radius-sm);
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--variant-circle {
  width: var(--fandhe-skeleton-size, 2.5rem);
  height: var(--fandhe-skeleton-size, 2.5rem);
  border-radius: var(--fandhe-radius-full);
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--variant-rect {
  width: 100%;
  height: var(--fandhe-skeleton-height, 5rem);
  border-radius: var(--fandhe-radius-md);
}

@keyframes fd-skeleton-pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}

@media (prefers-reduced-motion: reduce) {
  [data-scope="skeleton"][data-part="root"] {
    animation: none;
  }
}
"#;

#[test]
fn skeleton_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(skeleton::css(), SKELETON_GOLDEN_CSS);
}
