//! styled ToggleTip（イシュー #761）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants → compound → states）が崩れた場合や意図しない
//! 宣言の追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::toggle_tip;

const TOGGLE_TIP_GOLDEN_CSS: &str = r#"[data-scope="toggle-tip"][data-part="root"] {
  position: relative;
}

[data-scope="toggle-tip"][data-part="trigger"] {
  cursor: pointer;
}

[data-scope="toggle-tip"][data-part="positioner"] {
  position: absolute;
  bottom: 100%;
  left: 0;
  z-index: 1100;
  margin-bottom: var(--fandhe-space-1);
}

[data-scope="toggle-tip"][data-part="content"] {
  background: var(--fandhe-color-fg);
  color: var(--fandhe-color-bg);
  font-size: var(--fandhe-font-font-size-sm);
  border-radius: 0.25rem;
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  max-width: 20rem;
}

[data-scope="toggle-tip"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="toggle-tip"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
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
