//! `stack::STACK_CSS` の全文をバイト一致で固定する golden。
//!
//! 生成元 CSS 定数を手で書き写すと差分検知の意味がなくなるため、実装時に
//! `stack::STACK_CSS` の出力をそのままダンプして作成した（機械的な複製）。
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md` を参照。

const EXPECTED_CSS: &str = r#".fw-wire-stack {
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

#[test]
fn stack_css_matches_golden() {
    assert_eq!(fandhe_frontend_wireframe_ui::stack::STACK_CSS, EXPECTED_CSS);
}
