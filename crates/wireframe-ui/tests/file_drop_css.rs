//! `file_drop::FILE_DROP_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `file_drop::FILE_DROP_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-file-drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5em;
  text-align: center;
  box-sizing: border-box;
  max-width: 100%;
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.75) 1.5em;
  min-height: calc(var(--fw-wire-control-size, 2rem) * 4);
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
  user-select: none;
}
.fw-wire-file-drop-icon {
  display: flex;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-file-drop-icon .fw-wire-icon-glyph {
  width: 2em;
  height: 2em;
}
.fw-wire-file-drop-label {
  font-weight: 600;
}
.fw-wire-file-drop-hint {
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
"#;

#[test]
fn file_drop_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::file_drop::FILE_DROP_CSS,
        EXPECTED_CSS
    );
}
