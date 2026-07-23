//! styled Stat（イシュー #769）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/checkbox_card_css.rs` の golden fixture
//! テストの前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する。
//! 出力順（base → variants）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。

use fandhe_frontend_pre_styled_ui::stat;

const STAT_GOLDEN_CSS: &str = r#"[data-scope="stat"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope="stat"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="stat"][data-part="value-text"] {
  display: flex;
  align-items: baseline;
  gap: var(--fandhe-space-1);
  font-size: var(--fandhe-stat-value-font-size, var(--fandhe-font-font-size-2xl));
  font-weight: var(--fandhe-font-font-weight-semibold);
  margin: 0;
}

[data-scope="stat"][data-part="value-unit"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="stat"][data-part="help-text"] {
  display: block;
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="stat"][data-part="up-indicator"] {
  display: inline-block;
  width: 0.75em;
  height: 0.75em;
  clip-path: polygon(50% 0%, 100% 100%, 0% 100%);
  background: var(--fandhe-color-success-emphasized);
}

[data-scope="stat"][data-part="down-indicator"] {
  display: inline-block;
  width: 0.75em;
  height: 0.75em;
  clip-path: polygon(0% 0%, 100% 0%, 50% 100%);
  background: var(--fandhe-color-danger-emphasized);
}

[data-scope="stat"][data-part="root"].fd-stat--size-sm {
  --fandhe-stat-value-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="stat"][data-part="root"].fd-stat--size-md {
  --fandhe-stat-value-font-size: var(--fandhe-font-font-size-2xl);
}

[data-scope="stat"][data-part="root"].fd-stat--size-lg {
  --fandhe-stat-value-font-size: var(--fandhe-font-font-size-3xl);
}
"#;

#[test]
fn stat_css_matches_golden_fixture() {
    assert_eq!(stat::css(), STAT_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(stat::css(), stat::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = stat::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn selectors_match_actual_rendered_data_part_attributes() {
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_pre_styled_ui::recipe::Size;

    let css = stat::css();
    for part in [
        "root",
        "label",
        "value-text",
        "value-unit",
        "help-text",
        "up-indicator",
        "down-indicator",
    ] {
        assert!(
            css.contains(&format!(r#"[data-scope="stat"][data-part="{part}"]"#)),
            "css に data-part={part} のセレクタが含まれること"
        );
    }

    // recipe が宣言する data-part と、実レンダリング関数が出力する
    // data-part 属性が一致することを固定する。
    let html = render(&stat::root(
        Size::Md,
        vec![],
        vec![stat::label(vec![], vec![text("x")])],
    ));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
}
