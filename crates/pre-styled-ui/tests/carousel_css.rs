//! styled Carousel（イシュー #754）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/select_css.rs`/`slider_css.rs` の golden
//! fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で
//! 固定する。出力順（base → variants → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! `item-group` の `transform` が `--fandhe-carousel-index`
//! （headless `Carousel::item_group` の `style` 契約、`crates/headless-ui/src/carousel.rs`
//! 参照）を参照する点、`data-orientation="vertical"` で `translateY` へ
//! 切り替える点、`size` variant のみで `color-palette` 軸を持たない点は
//! `crate::carousel` module doc を参照。
//!
//! イシュー #1518 で hover（trigger/indicator）・フォーカスリング・
//! disabled 減光・トランジションを Phase 0 共通規約（イシュー #1424/#1425）
//! へ追随させた際に、この golden も新しい `stylesheet()` 出力へ更新した
//! （更新手順は `docs/internal/pre-styled-ui-golden-test-update-guide.md`
//! 参照）。

use fandhe_frontend_pre_styled_ui::carousel;

const CAROUSEL_GOLDEN_CSS: &str = r#"[data-scope="carousel"][data-part="root"] {
  position: relative;
  overflow: hidden;
}

[data-scope="carousel"][data-part="control"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope="carousel"][data-part="prev-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-full, 9999px);
  cursor: pointer;
  width: var(--fandhe-carousel-trigger-size, 2.5rem);
  height: var(--fandhe-carousel-trigger-size, 2.5rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="carousel"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-full, 9999px);
  cursor: pointer;
  width: var(--fandhe-carousel-trigger-size, 2.5rem);
  height: var(--fandhe-carousel-trigger-size, 2.5rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="carousel"][data-part="item-group"] {
  display: flex;
  flex: 1;
  transition-property: transform;
  transition-duration: var(--fandhe-carousel-transition-duration, var(--fandhe-motion-duration-normal, 200ms));
  transition-timing-function: var(--fandhe-motion-easing-standard);
  transform: translateX(calc(var(--fandhe-carousel-index, 0) * -100%));
}

[data-scope="carousel"][data-part="item"] {
  flex: 0 0 100%;
}

[data-scope="carousel"][data-part="indicator-group"] {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-2);
}

[data-scope="carousel"][data-part="indicator"] {
  display: inline-block;
  background: var(--fandhe-color-bg-muted);
  border: none;
  border-radius: var(--fandhe-radius-full, 9999px);
  cursor: pointer;
  width: var(--fandhe-carousel-indicator-size, 0.5rem);
  height: var(--fandhe-carousel-indicator-size, 0.5rem);
  --fandhe-hover-bg: var(--fandhe-color-bg-emphasized);
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="carousel"][data-part="root"].fd-carousel--size-xs {
  --fandhe-carousel-trigger-size: 1.5rem;
  --fandhe-carousel-indicator-size: 0.25rem;
}

[data-scope="carousel"][data-part="root"].fd-carousel--size-sm {
  --fandhe-carousel-trigger-size: 2rem;
  --fandhe-carousel-indicator-size: 0.375rem;
}

[data-scope="carousel"][data-part="root"].fd-carousel--size-md {
  --fandhe-carousel-trigger-size: 2.5rem;
  --fandhe-carousel-indicator-size: 0.5rem;
}

[data-scope="carousel"][data-part="root"].fd-carousel--size-lg {
  --fandhe-carousel-trigger-size: 3rem;
  --fandhe-carousel-indicator-size: 0.625rem;
}

[data-scope="carousel"][data-part="root"].fd-carousel--size-xl {
  --fandhe-carousel-trigger-size: 3.5rem;
  --fandhe-carousel-indicator-size: 0.75rem;
}

[data-scope="carousel"][data-part="item-group"][data-orientation="vertical"] {
  transform: translateY(calc(var(--fandhe-carousel-index, 0) * -100%));
}

[data-scope="carousel"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="carousel"][data-part="next-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="carousel"][data-part="prev-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="carousel"][data-part="next-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="carousel"][data-part="indicator"][data-current] {
  background: var(--fandhe-color-accent);
}

[data-scope="carousel"][data-part="indicator"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="carousel"][data-part="prev-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="carousel"][data-part="next-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }

  [data-scope="carousel"][data-part="indicator"]:hover:not([data-disabled]):not([data-current]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn carousel_stylesheet_matches_golden_css() {
    assert_eq!(carousel::stylesheet(), CAROUSEL_GOLDEN_CSS);
}

#[test]
fn carousel_stylesheet_is_deterministic_across_calls() {
    assert_eq!(carousel::stylesheet(), carousel::stylesheet());
}
