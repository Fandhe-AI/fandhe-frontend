//! `progress::PROGRESS_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `progress::PROGRESS_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-progress {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-progress-track {
  position: relative;
  box-sizing: border-box;
  background: var(--fw-wire-fill);
  border-radius: 999px;
  height: calc(var(--fw-wire-control-size, 2rem) * 0.25);
}
.fw-wire-progress-fill {
  position: absolute;
  inset-block: 0;
  left: 0;
  width: var(--fw-wire-progress-value);
  background: var(--fw-wire-ink);
  border-radius: inherit;
}
.fw-wire-progress.fw-wire-progress-circle {
  width: auto;
}
.fw-wire-progress.fw-wire-progress-circle .fw-wire-progress-track {
  position: relative;
  box-sizing: border-box;
  width: calc(var(--fw-wire-control-size, 2rem) * 2);
  height: calc(var(--fw-wire-control-size, 2rem) * 2);
  border-radius: 50%;
  background: conic-gradient(
    var(--fw-wire-ink) var(--fw-wire-progress-value),
    var(--fw-wire-fill) 0
  );
}
.fw-wire-progress-hole {
  position: absolute;
  inset: calc(var(--fw-wire-control-size, 2rem) * 0.3);
  box-sizing: border-box;
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-progress-value-0 { --fw-wire-progress-value: 0%; }
.fw-wire-progress-value-5 { --fw-wire-progress-value: 5%; }
.fw-wire-progress-value-10 { --fw-wire-progress-value: 10%; }
.fw-wire-progress-value-15 { --fw-wire-progress-value: 15%; }
.fw-wire-progress-value-20 { --fw-wire-progress-value: 20%; }
.fw-wire-progress-value-25 { --fw-wire-progress-value: 25%; }
.fw-wire-progress-value-30 { --fw-wire-progress-value: 30%; }
.fw-wire-progress-value-35 { --fw-wire-progress-value: 35%; }
.fw-wire-progress-value-40 { --fw-wire-progress-value: 40%; }
.fw-wire-progress-value-45 { --fw-wire-progress-value: 45%; }
.fw-wire-progress-value-50 { --fw-wire-progress-value: 50%; }
.fw-wire-progress-value-55 { --fw-wire-progress-value: 55%; }
.fw-wire-progress-value-60 { --fw-wire-progress-value: 60%; }
.fw-wire-progress-value-65 { --fw-wire-progress-value: 65%; }
.fw-wire-progress-value-70 { --fw-wire-progress-value: 70%; }
.fw-wire-progress-value-75 { --fw-wire-progress-value: 75%; }
.fw-wire-progress-value-80 { --fw-wire-progress-value: 80%; }
.fw-wire-progress-value-85 { --fw-wire-progress-value: 85%; }
.fw-wire-progress-value-90 { --fw-wire-progress-value: 90%; }
.fw-wire-progress-value-95 { --fw-wire-progress-value: 95%; }
.fw-wire-progress-value-100 { --fw-wire-progress-value: 100%; }
"#;

#[test]
fn progress_css_matches_golden() {
    assert_eq!(
        fandhe_frontend_wireframe_ui::progress::PROGRESS_CSS,
        EXPECTED_CSS
    );
}
