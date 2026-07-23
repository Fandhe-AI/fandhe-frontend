//! styled QrCode（イシュー #774）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base → variants）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。

use fandhe_frontend_pre_styled_ui::qr_code;

const QR_CODE_GOLDEN_CSS: &str = r#"[data-scope="qr-code"][data-part="root"] {
  display: inline-flex;
  position: relative;
  --fandhe-qr-code-size: 8rem;
}

[data-scope="qr-code"][data-part="frame"] {
  width: var(--fandhe-qr-code-size);
  height: var(--fandhe-qr-code-size);
  background: var(--fandhe-color-bg);
}

[data-scope="qr-code"][data-part="pattern"] {
  fill: var(--fandhe-color-fg);
}

[data-scope="qr-code"][data-part="overlay"] {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: auto;
}

[data-scope="qr-code"][data-part="root"].fd-qr-code--size-sm {
  --fandhe-qr-code-size: 6rem;
}

[data-scope="qr-code"][data-part="root"].fd-qr-code--size-md {
  --fandhe-qr-code-size: 8rem;
}

[data-scope="qr-code"][data-part="root"].fd-qr-code--size-lg {
  --fandhe-qr-code-size: 12rem;
}
"#;

#[test]
fn qr_code_stylesheet_matches_golden_fixture() {
    assert_eq!(qr_code::stylesheet(), QR_CODE_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(qr_code::stylesheet(), qr_code::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = qr_code::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
