//! `chart::CHART_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `chart::CHART_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-chart {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-chart-plot {
  display: flex;
  align-items: flex-end;
  gap: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  box-sizing: border-box;
  border-inline-start: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-block-end: var(--fw-wire-line-width) solid var(--fw-wire-line);
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  height: calc(var(--fw-wire-control-size, 2rem) * 4);
}
.fw-wire-chart-bar {
  flex: 1 0 auto;
  box-sizing: border-box;
  min-width: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  background: var(--fw-wire-ink-muted, var(--fw-wire-ink));
  border-radius: var(--fw-wire-radius) var(--fw-wire-radius) 0 0;
  height: var(--fw-wire-chart-value);
}
.fw-wire-chart.fw-wire-horizontal .fw-wire-chart-plot {
  flex-direction: column;
  align-items: stretch;
  height: auto;
}
.fw-wire-chart.fw-wire-horizontal .fw-wire-chart-bar {
  flex: 0 0 auto;
  min-width: 0;
  min-height: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  width: var(--fw-wire-chart-value);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  border-radius: 0 var(--fw-wire-radius) var(--fw-wire-radius) 0;
}
.fw-wire-chart-value-0 { --fw-wire-chart-value: 0%; }
.fw-wire-chart-value-5 { --fw-wire-chart-value: 5%; }
.fw-wire-chart-value-10 { --fw-wire-chart-value: 10%; }
.fw-wire-chart-value-15 { --fw-wire-chart-value: 15%; }
.fw-wire-chart-value-20 { --fw-wire-chart-value: 20%; }
.fw-wire-chart-value-25 { --fw-wire-chart-value: 25%; }
.fw-wire-chart-value-30 { --fw-wire-chart-value: 30%; }
.fw-wire-chart-value-35 { --fw-wire-chart-value: 35%; }
.fw-wire-chart-value-40 { --fw-wire-chart-value: 40%; }
.fw-wire-chart-value-45 { --fw-wire-chart-value: 45%; }
.fw-wire-chart-value-50 { --fw-wire-chart-value: 50%; }
.fw-wire-chart-value-55 { --fw-wire-chart-value: 55%; }
.fw-wire-chart-value-60 { --fw-wire-chart-value: 60%; }
.fw-wire-chart-value-65 { --fw-wire-chart-value: 65%; }
.fw-wire-chart-value-70 { --fw-wire-chart-value: 70%; }
.fw-wire-chart-value-75 { --fw-wire-chart-value: 75%; }
.fw-wire-chart-value-80 { --fw-wire-chart-value: 80%; }
.fw-wire-chart-value-85 { --fw-wire-chart-value: 85%; }
.fw-wire-chart-value-90 { --fw-wire-chart-value: 90%; }
.fw-wire-chart-value-95 { --fw-wire-chart-value: 95%; }
.fw-wire-chart-value-100 { --fw-wire-chart-value: 100%; }
"#;

#[test]
fn chart_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::chart::CHART_CSS, EXPECTED_CSS);
}
