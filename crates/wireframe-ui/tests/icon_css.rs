//! `icon::ICON_CSS`（表示部品、#2652）の全文をバイト一致で固定する
//! golden。基盤の `icon::ICON_GLYPH_CSS`（#2606）とは別の定数であり、
//! `tests/base_css.rs` の対象ではない。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `icon::ICON_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-icon { display: inline-flex; align-items: center; justify-content: center; line-height: 1; vertical-align: middle; color: var(--fw-wire-ink); }
"#;

#[test]
fn icon_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::icon::ICON_CSS, EXPECTED_CSS);
}
