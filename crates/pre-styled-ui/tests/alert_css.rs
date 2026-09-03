//! styled Alert（イシュー #550、イシュー #1553 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tab_nav_css.rs` と同型の golden fixture
//! テスト。イシュー #1553 で `status`（[`AlertStatus`]、内部で
//! `palette_scale_declarations` へ移行）に加えて `variant`（[`AlertVariant`]）・
//! `size`（[`fandhe_frontend_pre_styled_ui::Size`]）の 2 軸を新設したため、
//! 出力全体をバイト単位で固定する（`alert.rs` モジュール冒頭 rustdoc
//! 「参考サイト基準への調整」節参照）。

use fandhe_frontend_pre_styled_ui::alert;

const ALERT_GOLDEN_CSS: &str = r#"[data-scope="alert"][data-part="root"] {
  display: flex;
  align-items: flex-start;
  width: 100%;
  box-sizing: border-box;
  position: relative;
  gap: var(--fandhe-alert-gap, var(--fandhe-space-3));
  padding: var(--fandhe-alert-padding, var(--fandhe-space-4));
  border: 1px solid transparent;
  border-radius: var(--fandhe-radius-md);
  font-size: var(--fandhe-alert-font-size, var(--fandhe-font-font-size-sm));
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="alert"][data-part="indicator"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: var(--fandhe-alert-indicator-size, var(--fandhe-font-font-size-xl));
  height: var(--fandhe-alert-indicator-size, var(--fandhe-font-font-size-xl));
}

[data-scope="alert"][data-part="content"] {
  display: flex;
  flex-direction: column;
  flex: 1;
  gap: var(--fandhe-space-1);
  min-width: 0;
}

[data-scope="alert"][data-part="title"] {
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="alert"][data-part="root"].fd-alert--status-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--status-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--status-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--status-error {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--status-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--variant-subtle {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
}

[data-scope="alert"][data-part="root"].fd-alert--variant-surface {
  background: var(--fandhe-palette-subtle);
  color: var(--fandhe-palette-fg-subtle);
  border-color: var(--fandhe-palette-muted);
}

[data-scope="alert"][data-part="root"].fd-alert--variant-solid {
  background: var(--fandhe-palette-emphasized);
  color: var(--fandhe-palette-fg);
}

[data-scope="alert"][data-part="root"].fd-alert--variant-outline {
  background: transparent;
  color: var(--fandhe-palette-fg-subtle);
  border-color: var(--fandhe-palette-muted);
}

[data-scope="alert"][data-part="root"].fd-alert--size-xs {
  --fandhe-alert-padding: var(--fandhe-space-2);
  --fandhe-alert-gap: var(--fandhe-space-2);
  --fandhe-alert-font-size: var(--fandhe-font-font-size-xs);
  --fandhe-alert-indicator-size: var(--fandhe-font-font-size-md);
}

[data-scope="alert"][data-part="root"].fd-alert--size-sm {
  --fandhe-alert-padding: var(--fandhe-space-3);
  --fandhe-alert-gap: var(--fandhe-space-2);
  --fandhe-alert-font-size: var(--fandhe-font-font-size-xs);
  --fandhe-alert-indicator-size: var(--fandhe-font-font-size-lg);
}

[data-scope="alert"][data-part="root"].fd-alert--size-md {
  --fandhe-alert-padding: var(--fandhe-space-4);
  --fandhe-alert-gap: var(--fandhe-space-3);
  --fandhe-alert-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-alert-indicator-size: var(--fandhe-font-font-size-xl);
}

[data-scope="alert"][data-part="root"].fd-alert--size-lg {
  --fandhe-alert-padding: var(--fandhe-space-4);
  --fandhe-alert-gap: var(--fandhe-space-3);
  --fandhe-alert-font-size: var(--fandhe-font-font-size-md);
  --fandhe-alert-indicator-size: var(--fandhe-font-font-size-2xl);
}

[data-scope="alert"][data-part="root"].fd-alert--size-xl {
  --fandhe-alert-padding: var(--fandhe-space-5);
  --fandhe-alert-gap: var(--fandhe-space-4);
  --fandhe-alert-font-size: var(--fandhe-font-font-size-lg);
  --fandhe-alert-indicator-size: var(--fandhe-font-font-size-3xl);
}
"#;

#[test]
fn alert_css_matches_golden_fixture() {
    assert_eq!(alert::css(), ALERT_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(alert::css(), alert::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = alert::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// status/variant/size の全クラスセレクタが CSS 中に存在することを固定する
/// （golden 全文一致に加え、軸ごとの網羅性を意図が読み取れる形で明示する）。
#[test]
fn css_declares_all_status_variant_size_selectors() {
    let css = alert::css();
    for class in [
        "fd-alert--status-info",
        "fd-alert--status-success",
        "fd-alert--status-warning",
        "fd-alert--status-error",
        "fd-alert--status-neutral",
        "fd-alert--variant-subtle",
        "fd-alert--variant-surface",
        "fd-alert--variant-solid",
        "fd-alert--variant-outline",
        "fd-alert--size-xs",
        "fd-alert--size-sm",
        "fd-alert--size-md",
        "fd-alert--size-lg",
        "fd-alert--size-xl",
    ] {
        assert!(
            css.contains(&format!(".{class} {{")),
            "class={class} が css() に含まれない: {css}"
        );
    }
}
