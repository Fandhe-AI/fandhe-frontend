//! `stepper::STEPPER_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::stepper::STEPPER_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-stepper {
  display: flex;
  align-items: flex-start;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-stepper-step {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375em;
  flex: 1;
  min-width: 0;
  padding-inline-start: 0.5em;
  padding-inline-end: 0.5em;
}
.fw-wire-stepper-step:not(:first-child)::before {
  content: "";
  position: absolute;
  top: calc(var(--fw-wire-control-size, 2rem) / 2);
  right: calc(50% + var(--fw-wire-control-size, 2rem) / 2);
  width: calc(100% - var(--fw-wire-control-size, 2rem));
  height: var(--fw-wire-line-width);
  background: var(--fw-wire-line-subtle);
}
.fw-wire-stepper-step[data-complete]:not(:first-child)::before,
.fw-wire-stepper-step[data-active]:not(:first-child)::before {
  background: var(--fw-wire-line);
}
.fw-wire-stepper-indicator {
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fw-wire-control-size, 2rem);
  height: var(--fw-wire-control-size, 2rem);
  border-radius: 999px;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-weight: 600;
  flex-shrink: 0;
}
.fw-wire-stepper-step[data-complete] .fw-wire-stepper-indicator {
  background: var(--fw-wire-fill);
  border-color: var(--fw-wire-line);
}
.fw-wire-stepper-step[data-active] .fw-wire-stepper-indicator {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-stepper-label {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stepper-step[data-active] .fw-wire-stepper-label {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
"#;
