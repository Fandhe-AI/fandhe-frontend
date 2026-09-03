//! styled Marquee（イシュー #831、`docs/policy/intentional-non-adoption.md`
//! §3.24 の再導入）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skeleton_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。`direction`/
//! `edge`（イシュー #1582）variant ごとの規則・`@keyframes`・
//! `:hover`/`:focus-within` の常時一時停止規則・
//! `@media (prefers-reduced-motion: reduce)` を含む全文の出力順が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。

use fandhe_frontend_pre_styled_ui::marquee;

const MARQUEE_GOLDEN_CSS: &str = r#"[data-scope="marquee"][data-part="root"] {
  display: flex;
  overflow: hidden;
  gap: var(--fandhe-marquee-gap, var(--fandhe-space-4));
}

[data-scope="marquee"][data-part="content"] {
  display: flex;
  flex: none;
  align-items: center;
  min-width: max-content;
  gap: var(--fandhe-marquee-gap, var(--fandhe-space-4));
  animation-name: fd-marquee-scroll;
  animation-duration: var(--fandhe-marquee-duration, 20s);
  animation-timing-function: linear;
  animation-iteration-count: var(--fandhe-marquee-loop-count, infinite);
  animation-delay: var(--fandhe-marquee-delay, 0s);
  animation-fill-mode: forwards;
  animation-direction: var(--fandhe-marquee-direction, normal);
}

[data-scope="marquee"][data-part="item"] {
  flex: none;
}

[data-scope="marquee"][data-part="root"].fd-marquee--direction-start {
  --fandhe-marquee-direction: normal;
}

[data-scope="marquee"][data-part="root"].fd-marquee--direction-end {
  --fandhe-marquee-direction: reverse;
}

[data-scope="marquee"][data-part="root"].fd-marquee--edge-none {
  mask-image: none;
}

[data-scope="marquee"][data-part="root"].fd-marquee--edge-fade {
  mask-image: linear-gradient(to right, transparent, black var(--fandhe-marquee-edge-size, 20%), black calc(100% - var(--fandhe-marquee-edge-size, 20%)), transparent);
}

@keyframes fd-marquee-scroll {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(calc(-100% - var(--fandhe-marquee-gap, var(--fandhe-space-4))));
  }
}

[data-scope="marquee"][data-part="root"]:hover [data-part="content"],
[data-scope="marquee"][data-part="root"]:focus-within [data-part="content"] {
  animation-play-state: paused;
}

@media (prefers-reduced-motion: reduce) {
  [data-scope="marquee"][data-part="content"] {
    animation: none;
  }

  [data-scope="marquee"][data-part="content"][aria-hidden="true"] {
    display: none;
  }
}
"#;

#[test]
fn marquee_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(marquee::css(), MARQUEE_GOLDEN_CSS);
}

#[test]
fn marquee_css_never_contains_external_resource_references() {
    assert!(!marquee::css().contains("url("));
}
