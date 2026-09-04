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
//! イシュー #1583 で `root` の枠（`padding-block`・`color`・`background`・
//! `border`・`border-radius`）と `item` の見た目（`inline-flex`・
//! `align-items`・`gap`・`white-space: nowrap`）を追加し、
//! `@media (prefers-reduced-motion: reduce)` の挙動を「アニメーション停止
//! のみ」から「停止 + `root` の横スクロール化 + 両端フェード無効化 +
//! 複製非表示」へ拡張した。
//!
//! PR #1856 codex-review P1 是正で、`decorative: false`（既定）の `root`
//! が固定付与する `tabindex="0"`（`crates/pre-styled-ui/src/marquee.rs`
//! 参照。reduced-motion 時の横スクロール領域へのキーボード到達性の
//! 是正）に対応する `:focus-visible` フォーカスリング宣言を追加した。

use fandhe_frontend_pre_styled_ui::marquee;

const MARQUEE_GOLDEN_CSS: &str = r#"[data-scope="marquee"][data-part="root"] {
  display: flex;
  overflow: hidden;
  gap: var(--fandhe-marquee-gap, var(--fandhe-space-4));
  position: relative;
  box-sizing: border-box;
  padding-block: var(--fandhe-marquee-padding-y, 0);
  color: var(--fandhe-color-fg);
  background: var(--fandhe-marquee-bg, transparent);
  border: var(--fandhe-marquee-border, none);
  border-radius: var(--fandhe-marquee-radius, 0);
  mask-image: linear-gradient(to right, transparent, black var(--fandhe-marquee-fade, 0px), black calc(100% - var(--fandhe-marquee-fade, 0px)), transparent);
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
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  white-space: nowrap;
}

[data-scope="marquee"][data-part="root"].fd-marquee--direction-start {
  --fandhe-marquee-direction: normal;
}

[data-scope="marquee"][data-part="root"].fd-marquee--direction-end {
  --fandhe-marquee-direction: reverse;
}

[data-scope="marquee"][data-part="root"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: -2px;
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
  [data-scope="marquee"][data-part="root"] {
    overflow-x: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--fandhe-color-border) transparent;
    mask-image: none;
  }

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
