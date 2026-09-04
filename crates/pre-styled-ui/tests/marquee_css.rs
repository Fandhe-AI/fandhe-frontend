//! styled Marquee（イシュー #831、`docs/policy/intentional-non-adoption.md`
//! §3.24 の再導入）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skeleton_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。`direction`
//! variant ごとの規則・`@keyframes`・`:hover`/`:focus-within` の常時
//! 一時停止規則・`@media (prefers-reduced-motion: reduce)` を含む全文の
//! 出力順が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。
//!
//! イシュー #1582 で `gap` のフォールバックを `var(--fandhe-space-4)`
//! （テーマトークン経由）へ変更し、`root` へ両端フェード用の
//! `mask-image`（`--fandhe-marquee-fade`、既定 `0px`）を追加した。
//!
//! イシュー #1583 で `root` base へコンテンツ枠（`position: relative`・
//! `box-sizing: border-box`・`padding: var(--fandhe-marquee-padding, 0)`）
//! を追加し、`@media (prefers-reduced-motion: reduce)` ブロックを
//! 拡張した（静止時に折り返して全文表示・両端フェード解除。
//! `crates/pre-styled-ui/src/marquee.rs` モジュール doc「イシュー #1583」
//! 節参照）。

use fandhe_frontend_pre_styled_ui::marquee;

const MARQUEE_GOLDEN_CSS: &str = r#"[data-scope="marquee"][data-part="root"] {
  display: flex;
  overflow: hidden;
  gap: var(--fandhe-marquee-gap, var(--fandhe-space-4));
  mask-image: linear-gradient(to right, transparent, black var(--fandhe-marquee-fade, 0px), black calc(100% - var(--fandhe-marquee-fade, 0px)), transparent);
  position: relative;
  box-sizing: border-box;
  padding: var(--fandhe-marquee-padding, 0);
}

[data-scope="marquee"][data-part="content"] {
  display: flex;
  flex: none;
  align-items: center;
  min-width: max-content;
  gap: var(--fandhe-marquee-gap, var(--fandhe-space-4));
  animation: fd-marquee-scroll var(--fandhe-marquee-duration, 20s) linear infinite;
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
    min-width: 0;
    flex: 1 1 auto;
    flex-wrap: wrap;
  }

  [data-scope="marquee"][data-part="content"][aria-hidden="true"] {
    display: none;
  }

  [data-scope="marquee"][data-part="item"] {
    flex: 0 1 auto;
    min-width: 0;
  }

  [data-scope="marquee"][data-part="root"] {
    mask-image: none;
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
