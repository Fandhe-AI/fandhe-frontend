//! `media::MEDIA_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::media::MEDIA_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-media {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 100%;
  aspect-ratio: 16 / 9;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  overflow: hidden;
  --fw-wire-media-control-size: var(--fw-wire-control-size, 2rem);
}
.fw-wire-media .fw-wire-media-disc {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-media-control-size) * 1.5);
  height: calc(var(--fw-wire-media-control-size) * 1.5);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-media .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-media-control-size) * 0.6);
}
"#;
