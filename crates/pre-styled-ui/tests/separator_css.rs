//! styled Separator（イシュー #772）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/skeleton_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する（受け入れ条件
//! 1）。`orientation`（horizontal/vertical）・`variant`（solid/dashed/dotted、
//! イシュー #1585 で dotted 追加）の 2 軸それぞれの規則の出力順が崩れた
//! 場合や意図しない宣言の追加・欠落があった場合に、この golden テストが
//! 即座に検知する。
//!
//! イシュー #2053（shadcn/ui 突合）で `group`/`label`（ラベル付き区切り線、
//! chakra-ui の HStack + Text 合成相当の pre-styled-only パート）を追加。
//! `SlotRecipe::css` は base（slots 宣言順）→ variants（登録順）の順で
//! 出力するため、新規 2 ブロックは既存 `root` の base ブロック直後・
//! `orientation` variant ブロックの前に挿入される（末尾ではない）。既存
//! 6 ブロックはバイト単位で不変。

use fandhe_frontend_pre_styled_ui::separator;

const SEPARATOR_GOLDEN_CSS: &str = r#"[data-scope="separator"][data-part="root"] {
  border-width: 0;
  border-color: var(--fandhe-color-border);
  margin: 0;
  flex-shrink: 0;
}

[data-scope="separator"][data-part="group"] {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: var(--fandhe-space-3);
}

[data-scope="separator"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
  white-space: nowrap;
}

[data-scope="separator"][data-part="root"].fd-separator--orientation-horizontal {
  border-top-width: var(--fandhe-separator-thickness, 1px);
  width: 100%;
}

[data-scope="separator"][data-part="root"].fd-separator--orientation-vertical {
  border-inline-start-width: var(--fandhe-separator-thickness, 1px);
  align-self: stretch;
  height: var(--fandhe-separator-height, auto);
}

[data-scope="separator"][data-part="root"].fd-separator--variant-solid {
  border-style: solid;
}

[data-scope="separator"][data-part="root"].fd-separator--variant-dashed {
  border-style: dashed;
}

[data-scope="separator"][data-part="root"].fd-separator--variant-dotted {
  border-style: dotted;
}
"#;

#[test]
fn separator_css_matches_golden_fixture_byte_for_byte() {
    assert_eq!(separator::css(), SEPARATOR_GOLDEN_CSS);
}
