//! styled Switch（イシュー #682）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/popover_tooltip_css.rs` の golden fixture
//! テストの前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件 3）。出力順（base → variants → compound → states）が
//! 崩れた場合や意図しない宣言の追加・欠落があった場合に、この golden
//! テストが即座に検知する。
//!
//! `control` の `box-sizing: border-box` は PR #697 Cursor Bugbot 指摘
//! （review 3636964684）対応で追加した宣言。詳細は
//! `crates/pre-styled-ui/src/switch.rs` のモジュール doc を参照。

use fandhe_frontend_pre_styled_ui::switch;

const SWITCH_GOLDEN_CSS: &str = r#"[data-scope="switch"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  cursor: pointer;
}

[data-scope="switch"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: 2.5rem;
  height: 1.4rem;
  border-radius: 999px;
  background: var(--fandhe-color-border);
  padding: 0 0.15rem;
  transition: background 0.15s;
}

[data-scope="switch"][data-part="thumb"] {
  width: 1.1rem;
  height: 1.1rem;
  border-radius: 999px;
  background: var(--fandhe-color-bg);
  transition: transform 0.15s;
}

[data-scope="switch"][data-part="label"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="switch"][data-part="hidden-input"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

[data-scope="switch"][data-part="root"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}

[data-scope="switch"][data-part="control"][data-state="checked"] {
  background: var(--fandhe-color-accent);
}

[data-scope="switch"][data-part="control"][data-focus-visible] {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}

[data-scope="switch"][data-part="thumb"][data-state="checked"] {
  transform: translateX(1.1rem);
}
"#;

#[test]
fn switch_stylesheet_matches_golden_fixture() {
    assert_eq!(switch::stylesheet(), SWITCH_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    // recipe_determinism.rs / popover_tooltip_css.rs と同観点: 独立呼び出し
    // 間でバイト単位の一致を固定する。
    assert_eq!(switch::stylesheet(), switch::stylesheet());
}
