//! styled Timer（イシュー #836）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/hover_card_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::timer;

const TIMER_GOLDEN_CSS: &str = r#"[data-scope="timer"][data-part="root"] {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fandhe-space-4);
}

[data-scope="timer"][data-part="area"] {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: var(--fandhe-space-4);
}

[data-scope="timer"][data-part="item"] {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope="timer"][data-part="item-value"] {
  font-variant-numeric: tabular-nums;
  font-size: var(--fandhe-font-font-size-2xl);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  color: var(--fandhe-timer-value-color, var(--fandhe-color-fg));
}

[data-scope="timer"][data-part="item-label"] {
  font-size: var(--fandhe-font-font-size-xs);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="timer"][data-part="separator"] {
  font-size: var(--fandhe-font-font-size-2xl);
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
  align-self: flex-start;
  color: var(--fandhe-color-fg-muted);
}

[data-scope="timer"][data-part="control"] {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope="timer"][data-part="action-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-2);
  box-sizing: border-box;
  min-height: var(--fandhe-size-control-height-sm);
  padding: 0 var(--fandhe-size-control-padding-x-sm);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-family: var(--fandhe-font-font-body);
  font-size: var(--fandhe-size-control-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="timer"][data-part="root"][data-state="completed"] {
  --fandhe-timer-value-color: var(--fandhe-color-accent);
}

[data-scope="timer"][data-part="action-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="timer"][data-part="action-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn stylesheet_matches_golden_css_byte_for_byte() {
    assert_eq!(timer::stylesheet(), TIMER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic_across_calls() {
    assert_eq!(timer::stylesheet(), timer::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = timer::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_covers_every_declared_slot() {
    // SLOTS（`crates/pre-styled-ui/src/timer.rs`）と anatomy
    // （`crates/headless-ui/src/timer.rs`）の同期契約を、生成 CSS 中の
    // data-part セレクタ存在で外部からも固定する。
    let css = timer::stylesheet();
    for part in [
        "root",
        "area",
        "item",
        "item-value",
        "item-label",
        "separator",
        "control",
        "action-trigger",
    ] {
        let selector = format!(r#"[data-scope="timer"][data-part="{part}"]"#);
        assert!(
            css.contains(&selector),
            "expected selector {selector} in stylesheet, got: {css}"
        );
    }
}
