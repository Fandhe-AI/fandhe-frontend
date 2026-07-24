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
  border-radius: 9999px;
  cursor: pointer;
  width: var(--fandhe-carousel-trigger-size, 2.5rem);
  height: var(--fandhe-carousel-trigger-size, 2.5rem);
}

[data-scope="carousel"][data-part="next-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 9999px;
  cursor: pointer;
  width: var(--fandhe-carousel-trigger-size, 2.5rem);
  height: var(--fandhe-carousel-trigger-size, 2.5rem);
}

[data-scope="carousel"][data-part="item-group"] {
  display: flex;
  flex: 1;
  transition: transform var(--fandhe-carousel-transition-duration, 0.2s) ease;
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
  border-radius: 9999px;
  cursor: pointer;
  width: var(--fandhe-carousel-indicator-size, 0.5rem);
  height: var(--fandhe-carousel-indicator-size, 0.5rem);
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

[data-scope="carousel"][data-part="item-group"][data-orientation="vertical"] {
  transform: translateY(calc(var(--fandhe-carousel-index, 0) * -100%));
}

[data-scope="carousel"][data-part="prev-trigger"][data-disabled] {
  opacity: 0.4;
  cursor: not-allowed;
}

[data-scope="carousel"][data-part="next-trigger"][data-disabled] {
  opacity: 0.4;
  cursor: not-allowed;
}

[data-scope="carousel"][data-part="prev-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="carousel"][data-part="next-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="carousel"][data-part="indicator"][data-current] {
  background: var(--fandhe-color-accent);
}

[data-scope="carousel"][data-part="indicator"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
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
