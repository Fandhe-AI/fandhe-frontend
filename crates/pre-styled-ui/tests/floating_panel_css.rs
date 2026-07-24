//! styled FloatingPanel（イシュー #827）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs`/`popover.rs` 内蔵テストの先例に
//! 倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → compound → states）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::floating_panel;

const FLOATING_PANEL_GOLDEN_CSS: &str = r#"[data-scope="floating-panel"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
}

[data-scope="floating-panel"][data-part="positioner"] {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 900;
  transform: translate3d(var(--fandhe-x, 24px), var(--fandhe-y, 24px), 0);
}

[data-scope="floating-panel"][data-part="content"] {
  display: flex;
  flex-direction: column;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  min-width: 16rem;
}

[data-scope="floating-panel"][data-part="header"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-3) var(--fandhe-space-4);
  border-bottom: 1px solid var(--fandhe-color-border);
  cursor: move;
}

[data-scope="floating-panel"][data-part="title"] {
  font-size: var(--fandhe-font-font-size-lg);
  font-weight: var(--fandhe-font-font-weight-semibold);
  margin: 0;
}

[data-scope="floating-panel"][data-part="control"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="floating-panel"][data-part="stage-trigger"] {
  cursor: pointer;
  background: none;
  border: none;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="floating-panel"][data-part="close-trigger"] {
  cursor: pointer;
  background: none;
  border: none;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="floating-panel"][data-part="body"] {
  padding: var(--fandhe-space-4);
}

[data-scope="floating-panel"][data-part="content"][data-state="closed"] {
  visibility: hidden;
}

[data-scope="floating-panel"][data-part="body"][data-stage="minimized"] {
  display: none;
}

[data-scope="floating-panel"][data-part="positioner"][data-stage="maximized"] {
  transform: none;
  inset: 0;
}

[data-scope="floating-panel"][data-part="trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="floating-panel"][data-part="stage-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="floating-panel"][data-part="close-trigger"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(floating_panel::stylesheet(), FLOATING_PANEL_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_independent_calls() {
    assert_eq!(floating_panel::stylesheet(), floating_panel::stylesheet());
}
