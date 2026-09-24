//! `stack::STACK_CSS` の期待値リテラル（golden）。値は
//! `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` が
//! `fandhe_frontend_wireframe_ui::stack::STACK_CSS` と突き合わせる。

pub const EXPECTED_CSS: &str = r#".fw-wire-stack {
  display: flex;
  box-sizing: border-box;
  max-width: 100%;
  gap: 1rem;
}
.fw-wire-stack.fw-wire-horizontal {
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
}
.fw-wire-stack.fw-wire-vertical {
  flex-direction: column;
  align-items: stretch;
}
.fw-wire-stack.fw-wire-stack-gap-xs {
  gap: 0.25rem;
}
.fw-wire-stack.fw-wire-stack-gap-sm {
  gap: 0.5rem;
}
.fw-wire-stack.fw-wire-stack-gap-md {
  gap: 1rem;
}
.fw-wire-stack.fw-wire-stack-gap-lg {
  gap: 1.5rem;
}
.fw-wire-stack.fw-wire-stack-gap-xl {
  gap: 2rem;
}
"#;
