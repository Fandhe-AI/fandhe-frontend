//! `card_basic::CARD_BASIC_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-card-basic {
  display: flex;
  align-items: center;
  gap: 0.75em;
  box-sizing: border-box;
  min-width: 0;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-card-basic-body {
  display: flex;
  flex: 1 1 auto;
  min-width: 0;
  flex-direction: column;
}
.fw-wire-card-basic-primary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-card-basic-secondary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  font-size: 0.875em;
}
"#;
