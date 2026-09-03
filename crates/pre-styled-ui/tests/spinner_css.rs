//! styled Spinner（イシュー #550、イシュー #1567 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/alert_css.rs` と同型の golden fixture
//! テスト（方式 (a) バイト一致）。イシュー #1567 は Rust API シグネチャを
//! 変更せず既存の `size`/`color-palette` 2 軸の CSS 出力値のみを是正した
//! ため、`spinner.rs` モジュール冒頭 rustdoc「参照サイトとの差分」節を
//! 正として、出力全体をバイト単位で固定する。

use fandhe_frontend_pre_styled_ui::spinner;

const SPINNER_GOLDEN_CSS: &str = r#"[data-scope="spinner"][data-part="root"] {
  display: inline-block;
  box-sizing: border-box;
  flex-shrink: 0;
  border-radius: var(--fandhe-radius-full);
  border-width: var(--fandhe-spinner-thickness, 2px);
  border-style: solid;
  border-color: var(--fandhe-spinner-track-color, transparent);
  border-top-color: var(--fandhe-palette);
  border-inline-end-color: var(--fandhe-palette);
  animation-name: fd-spinner-spin;
  animation-duration: var(--fandhe-spinner-duration, 0.6s);
  animation-timing-function: linear;
  animation-iteration-count: infinite;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-xs {
  width: 0.75rem;
  height: 0.75rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-sm {
  width: 1rem;
  height: 1rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-md {
  width: 1.25rem;
  height: 1.25rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-lg {
  width: 2rem;
  height: 2rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--size-xl {
  width: 2.5rem;
  height: 2.5rem;
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="spinner"][data-part="root"].fd-spinner--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

@keyframes fd-spinner-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  [data-scope="spinner"][data-part="root"] {
    animation: none;
  }
}
"#;

#[test]
fn spinner_css_matches_golden_fixture() {
    assert_eq!(spinner::css(), SPINNER_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(spinner::css(), spinner::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = spinner::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// size/color-palette の全クラスセレクタが CSS 中に存在することを固定する
/// （golden 全文一致に加え、軸ごとの網羅性を意図が読み取れる形で明示する）。
#[test]
fn css_declares_all_size_and_palette_selectors() {
    let css = spinner::css();
    for class in [
        "fd-spinner--size-xs",
        "fd-spinner--size-sm",
        "fd-spinner--size-md",
        "fd-spinner--size-lg",
        "fd-spinner--size-xl",
        "fd-spinner--color-palette-accent",
        "fd-spinner--color-palette-info",
        "fd-spinner--color-palette-success",
        "fd-spinner--color-palette-warning",
        "fd-spinner--color-palette-danger",
        "fd-spinner--color-palette-neutral",
    ] {
        assert!(
            css.contains(&format!(".{class} {{")),
            "class={class} が css() に含まれない: {css}"
        );
    }
}
