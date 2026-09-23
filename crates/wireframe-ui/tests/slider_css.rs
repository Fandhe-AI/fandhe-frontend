//! `slider::SLIDER_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `slider::SLIDER_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-slider {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  user-select: none;
  padding-inline: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  padding-block: calc(var(--fw-wire-control-size, 2rem) * 0.15);
}
.fw-wire-slider-track {
  position: relative;
  box-sizing: border-box;
  background: var(--fw-wire-fill);
  border-radius: 999px;
  height: calc(var(--fw-wire-control-size, 2rem) * 0.2);
}
.fw-wire-slider-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: var(--fw-wire-slider-value);
  background: var(--fw-wire-ink);
  border-radius: inherit;
}
.fw-wire-slider-thumb {
  position: absolute;
  top: 50%;
  left: var(--fw-wire-slider-value);
  width: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  box-sizing: border-box;
  transform: translate(-50%, -50%);
  border: var(--fw-wire-line-width) solid var(--fw-wire-ink);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-slider.fw-wire-vertical {
  width: auto;
  height: calc(var(--fw-wire-control-size, 2rem) * 6);
  padding-inline: calc(var(--fw-wire-control-size, 2rem) * 0.15);
  padding-block: calc(var(--fw-wire-control-size, 2rem) * 0.25);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-track {
  height: 100%;
  width: calc(var(--fw-wire-control-size, 2rem) * 0.2);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-fill {
  top: auto;
  bottom: 0;
  left: 0;
  width: 100%;
  height: var(--fw-wire-slider-value);
}
.fw-wire-slider.fw-wire-vertical .fw-wire-slider-thumb {
  top: auto;
  left: 50%;
  bottom: var(--fw-wire-slider-value);
  transform: translate(-50%, 50%);
}
.fw-wire-slider[data-active] .fw-wire-slider-thumb {
  box-shadow: 0 0 0 3px var(--fw-wire-line-subtle);
}
.fw-wire-slider[data-disabled] {
  opacity: 0.5;
}
.fw-wire-slider[data-disabled] .fw-wire-slider-thumb {
  border-style: dashed;
}
.fw-wire-slider-value-0 { --fw-wire-slider-value: 0%; }
.fw-wire-slider-value-5 { --fw-wire-slider-value: 5%; }
.fw-wire-slider-value-10 { --fw-wire-slider-value: 10%; }
.fw-wire-slider-value-15 { --fw-wire-slider-value: 15%; }
.fw-wire-slider-value-20 { --fw-wire-slider-value: 20%; }
.fw-wire-slider-value-25 { --fw-wire-slider-value: 25%; }
.fw-wire-slider-value-30 { --fw-wire-slider-value: 30%; }
.fw-wire-slider-value-35 { --fw-wire-slider-value: 35%; }
.fw-wire-slider-value-40 { --fw-wire-slider-value: 40%; }
.fw-wire-slider-value-45 { --fw-wire-slider-value: 45%; }
.fw-wire-slider-value-50 { --fw-wire-slider-value: 50%; }
.fw-wire-slider-value-55 { --fw-wire-slider-value: 55%; }
.fw-wire-slider-value-60 { --fw-wire-slider-value: 60%; }
.fw-wire-slider-value-65 { --fw-wire-slider-value: 65%; }
.fw-wire-slider-value-70 { --fw-wire-slider-value: 70%; }
.fw-wire-slider-value-75 { --fw-wire-slider-value: 75%; }
.fw-wire-slider-value-80 { --fw-wire-slider-value: 80%; }
.fw-wire-slider-value-85 { --fw-wire-slider-value: 85%; }
.fw-wire-slider-value-90 { --fw-wire-slider-value: 90%; }
.fw-wire-slider-value-95 { --fw-wire-slider-value: 95%; }
.fw-wire-slider-value-100 { --fw-wire-slider-value: 100%; }
"#;

#[test]
fn slider_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::slider::SLIDER_CSS,
        EXPECTED_CSS
    );
}
