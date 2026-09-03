//! styled Skeleton（イシュー #764）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/checkbox_card_css.rs` の golden fixture テスト
//! の前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する（受け入れ条件
//! 1・2）。variant（text/circle/rect）ごとの規則・animation
//! （pulse/shine/none、イシュー #1566 で追加）ごとの規則・`@keyframes`・
//! `@media (prefers-reduced-motion: reduce)` を含む全文の出力順が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::skeleton;

const SKELETON_GOLDEN_CSS: &str = r#"[data-scope="skeleton"][data-part="root"] {
  display: block;
  background: var(--fandhe-color-bg-emphasized);
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
  flex-shrink: 0;
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--variant-rect {
  width: 100%;
  height: var(--fandhe-skeleton-height, 5rem);
  border-radius: var(--fandhe-radius-md);
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-pulse {
  animation: fd-skeleton-pulse 1.2s ease-in-out infinite;
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-shine {
  background-image: linear-gradient(270deg, var(--fandhe-skeleton-shine-from, var(--fandhe-color-bg-muted)), var(--fandhe-skeleton-shine-to, var(--fandhe-color-bg-emphasized)), var(--fandhe-skeleton-shine-to, var(--fandhe-color-bg-emphasized)), var(--fandhe-skeleton-shine-from, var(--fandhe-color-bg-muted)));
  background-size: 400% 100%;
  animation: fd-skeleton-shine 5s ease-in-out infinite;
}

[data-scope="skeleton"][data-part="root"].fd-skeleton--animation-none {
  animation: none;
}

@keyframes fd-skeleton-pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

@keyframes fd-skeleton-shine {
  from {
    background-position: 200% 0;
  }
  to {
    background-position: -200% 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  [data-scope="skeleton"][data-part="root"],
  [data-scope="skeleton"][data-part="root"].fd-skeleton--animation-pulse,
  [data-scope="skeleton"][data-part="root"].fd-skeleton--animation-shine {
    animation: none;
  }
}
"#;

#[test]
fn skeleton_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(skeleton::css(), SKELETON_GOLDEN_CSS);
}
