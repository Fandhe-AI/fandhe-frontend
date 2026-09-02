//! styled HoverCard（イシュー #759。イシュー #1523 で参照サイト基準へ調整。
//! PR #1799 codex-review/Bugbot 指摘を受け、headless 層の `hidden` 属性
//! ライフサイクルと競合し機能しなかった `content` の開閉フェード
//! transition/opacity 宣言を削除済み）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/pagination_css.rs`/`radio_group_css.rs` の
//! golden fixture テストの前例に倣い、`stylesheet()` が返す CSS 全文を
//! バイト単位で固定する。出力順（base → states → `@media (hover: hover)`
//! 末尾集約）が崩れた場合や意図しない宣言の追加・欠落があった場合に、この
//! golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::hover_card;

const HOVER_CARD_GOLDEN_CSS: &str = r#"[data-scope="hover-card"][data-part="root"] {
  position: relative;
}

[data-scope="hover-card"][data-part="trigger"] {
  color: var(--fandhe-color-accent);
  cursor: pointer;
  text-decoration: underline;
}

[data-scope="hover-card"][data-part="trigger"] {
  transition-property: color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="hover-card"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: var(--fandhe-z-index-popover, 10);
  margin-top: var(--fandhe-space-1);
}

[data-scope="hover-card"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg, 0.5rem);
  box-shadow: var(--fandhe-shadow-md, 0 4px 6px rgba(0, 0, 0, 0.15));
  padding: var(--fandhe-space-4);
  max-width: 20rem;
}

[data-scope="hover-card"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="hover-card"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="hover-card"][data-part="trigger"]:hover:not([data-disabled]) {
    color: var(--fandhe-color-accent-emphasized);
  }
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(hover_card::stylesheet(), HOVER_CARD_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_calls() {
    assert_eq!(hover_card::stylesheet(), hover_card::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = hover_card::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_covers_every_declared_slot() {
    // SLOTS（`crates/pre-styled-ui/src/hover_card.rs`）と anatomy
    // （`crates/headless-ui/src/hover_card.rs`）の同期契約を、生成 CSS 中の
    // data-part セレクタ存在で外部からも固定する。
    let css = hover_card::stylesheet();
    for part in [
        "root",
        "trigger",
        "positioner",
        "content",
        "arrow",
        "arrow-tip",
    ] {
        // arrow/arrow-tip は base 規則を持たないため stylesheet() には
        // 現れない（recipe() 参照）。root/trigger/positioner/content のみ
        // セレクタ存在を検証する。
        if part == "arrow" || part == "arrow-tip" {
            continue;
        }
        let selector = format!(r#"[data-scope="hover-card"][data-part="{part}"]"#);
        assert!(
            css.contains(&selector),
            "expected selector {selector} in stylesheet, got: {css}"
        );
    }
}
