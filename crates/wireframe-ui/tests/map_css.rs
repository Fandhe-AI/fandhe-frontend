//! `map::MAP_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `map::MAP_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-map {
  position: relative;
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  height: calc(var(--fw-wire-control-size, 2rem) * 6);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
}
.fw-wire-map-zoom-far {
  --fw-wire-map-cell: calc(var(--fw-wire-control-size, 2rem) * 0.5);
}
.fw-wire-map-zoom-medium {
  --fw-wire-map-cell: var(--fw-wire-control-size, 2rem);
}
.fw-wire-map-zoom-near {
  --fw-wire-map-cell: calc(var(--fw-wire-control-size, 2rem) * 2);
}
.fw-wire-map-tile {
  position: absolute;
  inset: 0;
  background-image:
    repeating-linear-gradient(
      0deg,
      var(--fw-wire-line-subtle) 0,
      var(--fw-wire-line-subtle) var(--fw-wire-line-width),
      transparent var(--fw-wire-line-width),
      transparent var(--fw-wire-map-cell)
    ),
    repeating-linear-gradient(
      90deg,
      var(--fw-wire-line-subtle) 0,
      var(--fw-wire-line-subtle) var(--fw-wire-line-width),
      transparent var(--fw-wire-line-width),
      transparent var(--fw-wire-map-cell)
    );
}
.fw-wire-map-area {
  position: absolute;
  top: 15%;
  left: 10%;
  width: 30%;
  height: 25%;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-map-road-h {
  position: absolute;
  top: 55%;
  left: 0;
  width: 100%;
  height: calc(var(--fw-wire-line-width) * 3);
  background: var(--fw-wire-line);
}
.fw-wire-map-road-v {
  position: absolute;
  top: 0;
  left: 40%;
  width: calc(var(--fw-wire-line-width) * 3);
  height: 100%;
  background: var(--fw-wire-line);
}
.fw-wire-map-marker {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  color: var(--fw-wire-ink);
}
.fw-wire-map-marker .fw-wire-icon-glyph {
  width: 1.5em;
  height: 1.5em;
}
"#;

#[test]
fn map_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::map::MAP_CSS, EXPECTED_CSS);
}
