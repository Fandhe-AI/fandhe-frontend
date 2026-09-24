//! `modal::MODAL_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::modal::MODAL_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-modal {
  display: grid;
  place-items: center;
  min-height: 12rem;
  padding: 1.5rem;
  box-sizing: border-box;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-modal-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75em;
  width: 100%;
  box-sizing: border-box;
  padding: 1.25em;
  background: var(--fw-wire-paper);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  line-height: 1.4;
}
.fw-wire-modal-title {
  font-weight: 600;
}
.fw-wire-modal-body {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-modal-actions {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 0.5em;
  flex-wrap: wrap;
}
.fw-wire-modal.fw-wire-modal-max-width-xs .fw-wire-modal-panel {
  max-width: 20rem;
}
.fw-wire-modal.fw-wire-modal-max-width-sm .fw-wire-modal-panel {
  max-width: 26rem;
}
.fw-wire-modal.fw-wire-modal-max-width-md .fw-wire-modal-panel {
  max-width: 32rem;
}
.fw-wire-modal.fw-wire-modal-max-width-lg .fw-wire-modal-panel {
  max-width: 40rem;
}
.fw-wire-modal.fw-wire-modal-max-width-xl .fw-wire-modal-panel {
  max-width: 50rem;
}
"#;
