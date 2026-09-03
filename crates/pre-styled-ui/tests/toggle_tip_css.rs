//! styled ToggleTip（イシュー #761）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。
//!
//! イシュー #1546（toggle-tip のスタイルを参考サイト基準へ調整）で
//! `trigger` の ghost ボタン化・`data-state="open"`/`data-disabled`/hover/
//! transition の新規状態規則・`positioner`/`content` のトークン化を反映して
//! 更新した。

use fandhe_frontend_pre_styled_ui::toggle_tip;

const TOGGLE_TIP_GOLDEN_CSS: &str = r#"[data-scope="toggle-tip"][data-part="root"] {
  position: relative;
}

[data-scope="toggle-tip"][data-part="trigger"] {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: none;
  border-radius: var(--fandhe-radius-sm);
  padding: var(--fandhe-space-1);
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="toggle-tip"][data-part="positioner"] {
  position: absolute;
  bottom: 100%;
  left: 0;
  z-index: var(--fandhe-z-index-popover, 1100);
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="toggle-tip"][data-part="content"] {
  background: var(--fandhe-color-fg);
  color: var(--fandhe-color-bg);
  font-size: var(--fandhe-font-font-size-sm);
  border-radius: var(--fandhe-radius-sm, 0.25rem);
  box-shadow: var(--fandhe-shadow-sm);
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  max-width: 20rem;
}

[data-scope="toggle-tip"][data-part="trigger"][data-state="open"] {
  background: var(--fandhe-color-bg-muted);
  color: var(--fandhe-color-fg);
}

[data-scope="toggle-tip"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="toggle-tip"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="toggle-tip"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="toggle-tip"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn toggle_tip_stylesheet_matches_golden_fixture() {
    assert_eq!(toggle_tip::stylesheet(), TOGGLE_TIP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs と同観点: 独立呼び出し間でバイト単位の一致を固定する。
    assert_eq!(toggle_tip::stylesheet(), toggle_tip::stylesheet());
}
