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
  --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg));
}

@media (hover: hover) {
  [data-scope="scroll-area"][data-part="viewport"]:hover:not([data-disabled]) {
    --fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg));
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
