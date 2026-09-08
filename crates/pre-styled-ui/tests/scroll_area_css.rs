//! styled ScrollArea（イシュー #825、イシュー #1584 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/spinner_css.rs` と同型の golden fixture
//! テスト（方式 (a) バイト一致）。scroll_area は #1584 まで golden 不在
//! だったため本ファイルで新設する（`docs/internal/pre-styled-ui-golden-test-update-guide.md`
//! 参照）。`crate::scroll_area` モジュール冒頭 rustdoc「参考サイト基準への
//! スタイル調整（イシュー #1584）」節を正として、出力全体をバイト単位で
//! 固定する。

use fandhe_frontend_pre_styled_ui::scroll_area;

const SCROLL_AREA_GOLDEN_CSS: &str = r#"[data-scope="scroll-area"][data-part="root"] {
  position: relative;
  overflow: hidden;
  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)));
}

[data-scope="scroll-area"][data-part="viewport"] {
  height: 100%;
  width: 100%;
  overflow: auto;
  scrollbar-width: thin;
  scrollbar-color: var(--fandhe-scroll-area-thumb-bg) transparent;
}

[data-scope="scroll-area"][data-part="content"] {
  display: block;
}

[data-scope="scroll-area"][data-part="scrollbar"] {
  display: none;
}

[data-scope="scroll-area"][data-part="thumb"] {
  display: none;
}

[data-scope="scroll-area"][data-part="corner"] {
  display: none;
}

[data-scope="scroll-area"][data-part="viewport"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
  --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));
}

@media (hover: hover) {
  [data-scope="scroll-area"][data-part="viewport"]:hover:not([data-disabled]) {
    --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));
  }
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar {
  width: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);
  height: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-track {
  background: transparent;
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-thumb {
  background: var(--fandhe-scroll-area-thumb-bg);
  border-radius: var(--fandhe-radius-full);
  border: 2px solid transparent;
  background-clip: content-box;
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-corner {
  background: transparent;
}
[data-scope="scroll-area"][data-part="viewport"][data-orientation="horizontal"] > [data-scope="scroll-area"][data-part="content"] {
  display: flex;
  width: max-content;
}
@keyframes fandhe-scroll-area-fade-reveal-start {
  from {
    --fandhe-scroll-area-fade-start-driven: 0px;
  }
  to {
    --fandhe-scroll-area-fade-start-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));
  }
}
@keyframes fandhe-scroll-area-fade-reveal-end {
  from {
    --fandhe-scroll-area-fade-end-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));
  }
  to {
    --fandhe-scroll-area-fade-end-driven: 0px;
  }
}
[data-scope="scroll-area"][data-part="viewport"][data-fade] {
  mask-image: linear-gradient(to bottom, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);
  mask-repeat: no-repeat;
}
[data-scope="scroll-area"][data-part="viewport"][data-fade][data-orientation="horizontal"] {
  mask-image: linear-gradient(to right, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);
}
[data-scope="scroll-area"][data-part="viewport"][data-fade][data-orientation="horizontal"]:dir(rtl) {
  mask-image: linear-gradient(to left, transparent 0, #000 var(--fandhe-scroll-area-fade-start, var(--fandhe-scroll-area-fade-start-driven, 0px)), #000 calc(100% - var(--fandhe-scroll-area-fade-end, var(--fandhe-scroll-area-fade-end-driven, 0px))), transparent 100%);
}
@supports (animation-timeline: scroll()) {
  [data-scope="scroll-area"][data-part="viewport"][data-fade] {
    animation: fandhe-scroll-area-fade-reveal-start 1ms linear, fandhe-scroll-area-fade-reveal-end 1ms linear;
    animation-timeline: scroll(self block), scroll(self block);
    animation-range: 0 var(--fandhe-scroll-area-fade-reveal, var(--fandhe-space-8, 2rem)), calc(100% - var(--fandhe-scroll-area-fade-reveal, var(--fandhe-space-8, 2rem))) 100%;
    animation-fill-mode: both;
  }
  [data-scope="scroll-area"][data-part="viewport"][data-fade][data-orientation="horizontal"] {
    animation-timeline: scroll(self inline), scroll(self inline);
  }
}
@supports not (animation-timeline: scroll()) {
  [data-scope="scroll-area"][data-part="viewport"][data-fade] {
    --fandhe-scroll-area-fade-start-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));
    --fandhe-scroll-area-fade-end-driven: var(--fandhe-scroll-area-fade-size, min(12%, var(--fandhe-space-10, 2.5rem)));
  }
}
"#;

/// イシュー #825/#1584 まで固定していた golden 全文（`SCROLL_AREA_GOLDEN_CSS`
/// の `[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-corner`
/// ブロックまで）。イシュー #2054 で追記した横スクロール・端フェード規則が
/// 既存出力を変更せず末尾へ**純追加**されたことを機械固定するための
/// 接頭辞比較専用の定数（`docs/design/shadcn-reference-adoption-policy.md`
/// §8 の golden 純追加原則）。
const SCROLL_AREA_GOLDEN_CSS_BEFORE_2054: &str = r#"[data-scope="scroll-area"][data-part="root"] {
  position: relative;
  overflow: hidden;
  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)));
}

[data-scope="scroll-area"][data-part="viewport"] {
  height: 100%;
  width: 100%;
  overflow: auto;
  scrollbar-width: thin;
  scrollbar-color: var(--fandhe-scroll-area-thumb-bg) transparent;
}

[data-scope="scroll-area"][data-part="content"] {
  display: block;
}

[data-scope="scroll-area"][data-part="scrollbar"] {
  display: none;
}

[data-scope="scroll-area"][data-part="thumb"] {
  display: none;
}

[data-scope="scroll-area"][data-part="corner"] {
  display: none;
}

[data-scope="scroll-area"][data-part="viewport"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
  --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));
}

@media (hover: hover) {
  [data-scope="scroll-area"][data-part="viewport"]:hover:not([data-disabled]) {
    --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))));
  }
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar {
  width: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);
  height: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-track {
  background: transparent;
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-thumb {
  background: var(--fandhe-scroll-area-thumb-bg);
  border-radius: var(--fandhe-radius-full);
  border: 2px solid transparent;
  background-clip: content-box;
}
[data-scope="scroll-area"][data-part="viewport"]::-webkit-scrollbar-corner {
  background: transparent;
}
"#;

#[test]
fn scroll_area_stylesheet_matches_golden_fixture() {
    assert_eq!(scroll_area::stylesheet(), SCROLL_AREA_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(scroll_area::stylesheet(), scroll_area::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = scroll_area::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_purely_appends_2054_rules_after_pre_existing_output() {
    // イシュー #2054: 横スクロール・端フェード規則は #825/#1584 までの
    // 既存出力の**末尾へ追記**されるのみで、既存ブロックの内容・順序は
    // 一切変更しないことを固定する（golden 純追加原則）。
    let css = scroll_area::stylesheet();
    assert!(
        css.starts_with(SCROLL_AREA_GOLDEN_CSS_BEFORE_2054),
        "既存（#825/#1584 まで）の golden 全文が新出力の先頭に一致しません"
    );
}
