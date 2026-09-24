//! `calendar::CALENDAR_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::calendar::CALENDAR_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-calendar {
  display: block;
  box-sizing: border-box;
  width: 100%;
  max-width: 20em;
  padding: 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5em;
  margin-bottom: 0.5em;
}
.fw-wire-calendar-nav {
  display: inline-flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-calendar-label {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
  font-weight: 600;
}
.fw-wire-calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25em;
  margin-bottom: 0.25em;
}
.fw-wire-calendar-weekday {
  height: 0.4em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-calendar-grid {
  display: flex;
  flex-direction: column;
  gap: 0.25em;
}
.fw-wire-calendar-week {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25em;
}
.fw-wire-calendar-day {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  aspect-ratio: 1 / 1;
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
}
.fw-wire-calendar-day-empty {
  visibility: hidden;
}
.fw-wire-calendar-day[data-active] {
  background: var(--fw-wire-fill);
  border: var(--fw-wire-line-width) solid var(--fw-wire-ink);
  font-weight: 600;
}
"#;
