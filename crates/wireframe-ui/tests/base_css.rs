//! 実行時に導出される基盤 CSS 4 件（トークン・`Size` スコープ変数・
//! Frame padding・アイコングリフ基底 class）の全文をバイト一致で固定する
//! golden。`tokens::css()`・`size::css()`・`frame::frame_padding_css()` は
//! `size::SCALE`・`tokens::TOKENS` という共有値表から実行時に生成される
//! ため、部品ごとの `&str` リテラル golden よりも実効性が高い（変更検知の
//! 主眼はここにある）。更新手順は
//! `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_TOKENS_CSS: &str = r#":root {
  --fw-wire-paper: #ffffff;
  --fw-wire-fill-subtle: #f5f5f5;
  --fw-wire-fill: #e5e5e5;
  --fw-wire-line-subtle: #cccccc;
  --fw-wire-line: #8c8c8c;
  --fw-wire-ink-muted: #595959;
  --fw-wire-ink: #1f1f1f;
  --fw-wire-line-width: 1.5px;
  --fw-wire-radius: 4px;
  --fw-wire-font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
}
"#;

const EXPECTED_SIZE_CSS: &str = r#".fw-wire-size-xs { --fw-wire-font-size: 0.75rem; --fw-wire-control-size: 1.5rem; }
.fw-wire-size-sm { --fw-wire-font-size: 0.875rem; --fw-wire-control-size: 1.75rem; }
.fw-wire-size-md { --fw-wire-font-size: 1rem; --fw-wire-control-size: 2rem; }
.fw-wire-size-lg { --fw-wire-font-size: 1.125rem; --fw-wire-control-size: 2.5rem; }
.fw-wire-size-xl { --fw-wire-font-size: 1.25rem; --fw-wire-control-size: 3rem; }
"#;

const EXPECTED_FRAME_PADDING_CSS: &str = r#".fw-wire-frame.fw-wire-frame-padding-xs {
  padding: calc(1.5rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-sm {
  padding: calc(1.75rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-md {
  padding: calc(2rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-lg {
  padding: calc(2.5rem / 2);
}
.fw-wire-frame.fw-wire-frame-padding-xl {
  padding: calc(3rem / 2);
}
"#;

const EXPECTED_ICON_GLYPH_CSS: &str = r#".fw-wire-icon-glyph { display: inline-block; width: 1em; height: 1em; font-size: var(--fw-wire-font-size, 1rem); vertical-align: -0.125em; flex-shrink: 0; }
"#;

#[test]
fn tokens_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::tokens::css(),
        EXPECTED_TOKENS_CSS
    );
}

#[test]
fn size_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::size::css(), EXPECTED_SIZE_CSS);
}

#[test]
fn frame_padding_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::frame::frame_padding_css(),
        EXPECTED_FRAME_PADDING_CSS
    );
}

#[test]
fn icon_glyph_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::icon::ICON_GLYPH_CSS,
        EXPECTED_ICON_GLYPH_CSS
    );
}
