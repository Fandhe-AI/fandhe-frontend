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
//! 参照）。PR #1925 codex-review 指摘 是正（1 回目）: `data-orientation="vertical"`
//! の `item-group` へ `flex-direction: column` を追加した（`display: flex`
//! が既定で横並びのままだと `translateY` だけを上書きしても track 全体が
//! 上へ動くだけでスライドが正しく切り替わらないため）。
//!
//! PR #1925 codex-review 指摘 是正（2 回目・P1 2 件 + Cursor Bugbot 指摘）:
//! 上記の `flex-direction: column` だけでは `item-group` に確定した高さが
//! なく `item` の `flex: 0 0 100%`（主軸=高さ）が解決できないため、
//! `item-group[data-orientation="vertical"]` 自身へ
//! `--fandhe-carousel-height` トークン（既定 20rem）による確定高さと
//! `overflow: hidden` を追加した（`root` 側に確定高さを与えると `root` の
//! 兄弟パーツ `control` が `root` の `overflow: hidden` で隠れてしまうため、
//! `root` の高さは子要素合計に追随する auto のまま変更しない。
//! `crate::carousel` の該当 `.state()` rustdoc 参照）。
//!
//! PR #1925 codex-review 指摘 是正（3 回目・P1 1 件 + Cursor Bugbot 指摘）:
//! 上記（2 回目）の是正は `item-group` 自身に確定高さ・`overflow: hidden`・
//! `translateY` の 3 つを同時に持たせてしまい、クリップ領域と移動対象が
//! 同一要素になっていたため、`item-group` を動かすとクリップ座標系ごと
//! 移動して index=1 以降で次のスライドが表示されない不具合があった。
//! `item-group[data-orientation="vertical"]` は確定高さ・
//! `overflow: hidden` を保持したまま `transform: none`（base の横方向
//! `translateX` を打ち消す）で**静止したクリッパー**に徹し、代わりに
//! `item[data-orientation="vertical"]` へ `translateY` とトランジションを
//! 移した（`item` は `flex: 0 0 100%` で `item-group` と同じ高さを持つため、
//! 全 `item` へ同じ量の `translateY` を適用すると `item-group` 自体を
//! 動かすのと幾何学的に等価になる。`crate::carousel` module doc
//! 「transform ベースのスライド位置表現」節・`recipe()` の該当 `.state()`
//! rustdoc 参照）。
//!
//! PR #1925 codex-review 指摘 是正（4 回目・P1 1 件 + Cursor Bugbot 指摘）:
//! `item` は `flex: 0 0 100%` のみで既定 `min-height: auto`（縦方向）・
//! `min-width: auto`（横方向）が残り、内容が表示領域より大きいと
//! `item` 自身が伸びて `translateY`/`translateX` の百分率基準（自身の
//! border box）が `item-group` の高さ・幅と食い違う。`item` base へ
//! `min-width: 0` を、`item[data-orientation="vertical"]` へ
//! `min-height: 0` と `overflow: hidden` を追加し、内容の大きさに関わらず
//! `flex: 0 0 100%` の解決値へ寸法を固定した（`crate::carousel` の該当
//! `.base("item", ...)`/`.state("item", ...)` rustdoc 参照）。
//!
//! PR #1925 codex-review 指摘 是正（5 回目・P1 2 件）: 横方向 `item` は
//! 4 回目の是正で `min-width: 0` のみを追加していたため寸法は表示領域へ
//! 揃ったが `overflow` が既定の `visible` のままで、幅を超える内容
//! （大きい画像等）は境界でクリップされず隣のスライドへはみ出していた。
//! orientation によらず `item` base の宣言へ `min-width: 0`/
//! `min-height: 0`/`overflow: hidden` を寄せ（縦方向 state 側の重複宣言
//! `min-height: 0`/`overflow: hidden` は削除）、横縦で対称な寸法固定・
//! 内容クリップにした（`crate::carousel` の `.base("item", ...)` rustdoc
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
  min-width: 0;
  min-height: 0;
  overflow: hidden;
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
  flex-direction: column;
  height: var(--fandhe-carousel-height, 20rem);
  overflow: hidden;
  transform: none;
}

[data-scope="carousel"][data-part="item"][data-orientation="vertical"] {
  transition-property: transform;
  transition-duration: var(--fandhe-carousel-transition-duration, var(--fandhe-motion-duration-normal, 200ms));
  transition-timing-function: var(--fandhe-motion-easing-standard);
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
