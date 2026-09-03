//! styled LinkOverlay（イシュー #1580、参考サイト基準へのスタイル調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/avatar_css.rs` の golden fixture テスト
//! の前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → variant → state）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//! `docs/internal/pre-styled-ui-golden-test-update-guide.md` §3.3 が
//! 新規追加の必要性を指摘していた「golden 不在」の 1 件を埋める。

use fandhe_frontend_pre_styled_ui::link_overlay;

const LINK_OVERLAY_GOLDEN_CSS: &str = r#"[data-scope="link-overlay"][data-part="root"] {
  position: relative;
}

[data-scope="link-overlay"][data-part="overlay"] {
  position: absolute;
  inset: 0;
  z-index: 0;
  border-radius: inherit;
}

[data-scope="link-overlay"][data-part="overlay"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}
"#;

#[test]
fn link_overlay_stylesheet_matches_golden_fixture() {
    assert_eq!(link_overlay::stylesheet(), LINK_OVERLAY_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(link_overlay::stylesheet(), link_overlay::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = link_overlay::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

// headless 層（`crates/headless-ui/src/link_overlay.rs`）の slot 名
// （`root` / `overlay`）と本 golden の `data-part` セレクタが一致することを
// 固定する（headless/pre-styled 間のドリフト検知）。
#[test]
fn golden_selectors_match_headless_slot_names() {
    let css = link_overlay::stylesheet();
    assert!(css.contains(r#"[data-part="root"]"#));
    assert!(css.contains(r#"[data-part="overlay"]"#));
}
