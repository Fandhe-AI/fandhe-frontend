//! styled Card（イシュー #550、イシュー #1557 で参考サイト基準へ調整）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/callout_css.rs` と同型の golden fixture
//! テスト。イシュー #1557 で size 軸を `--fandhe-card-*` custom property
//! へ一本化し、区切り線を廃止・影段を `shadow-md` へ是正したため、出力全体
//! をバイト単位で固定する（`card.rs` モジュール冒頭 rustdoc「参考サイト
//! 基準への調整」節参照）。

use fandhe_frontend_pre_styled_ui::card;

const CARD_GOLDEN_CSS: &str = r#"[data-scope="card"][data-part="root"] {
  display: flex;
  flex-direction: column;
  position: relative;
  min-width: 0;
  box-sizing: border-box;
  overflow-wrap: break-word;
  color: var(--fandhe-color-fg);
  border: 1px solid transparent;
  border-radius: var(--fandhe-card-radius, var(--fandhe-radius-lg));
}

[data-scope="card"][data-part="header"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1-5);
  padding: var(--fandhe-card-padding, var(--fandhe-space-4));
}

[data-scope="card"][data-part="body"] {
  display: flex;
  flex-direction: column;
  flex: 1;
  padding: var(--fandhe-card-padding, var(--fandhe-space-4));
}

[data-scope="card"][data-part="footer"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-card-padding, var(--fandhe-space-4));
}

[data-scope="card"][data-part="title"] {
  margin: 0;
  font-size: var(--fandhe-card-title-font-size, var(--fandhe-font-font-size-lg));
  font-weight: var(--fandhe-font-font-weight-semibold);
  line-height: var(--fandhe-font-line-height-tight);
}

[data-scope="card"][data-part="description"] {
  margin: 0;
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="card"][data-part="action"] {
  grid-column: 2;
  grid-row: 1 / span 2;
  justify-self: end;
  align-self: start;
}

[data-scope="card"][data-part="cover"] {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-start-start-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);
  border-start-end-radius: calc(var(--fandhe-card-radius, var(--fandhe-radius-lg)) - 1px);
}

[data-scope="card"][data-part="root"].fd-card--size-xs {
  --fandhe-card-padding: var(--fandhe-space-3);
  --fandhe-card-radius: var(--fandhe-radius-md);
  --fandhe-card-title-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="card"][data-part="root"].fd-card--size-sm {
  --fandhe-card-padding: var(--fandhe-space-4);
  --fandhe-card-radius: var(--fandhe-radius-lg);
  --fandhe-card-title-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="card"][data-part="root"].fd-card--size-md {
  --fandhe-card-padding: var(--fandhe-space-6);
  --fandhe-card-radius: var(--fandhe-radius-lg);
  --fandhe-card-title-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="card"][data-part="root"].fd-card--size-lg {
  --fandhe-card-padding: var(--fandhe-space-8);
  --fandhe-card-radius: var(--fandhe-radius-xl);
  --fandhe-card-title-font-size: var(--fandhe-font-font-size-xl);
}

[data-scope="card"][data-part="root"].fd-card--size-xl {
  --fandhe-card-padding: var(--fandhe-space-10);
  --fandhe-card-radius: var(--fandhe-radius-2xl);
  --fandhe-card-title-font-size: var(--fandhe-font-font-size-2xl);
}

[data-scope="card"][data-part="root"].fd-card--variant-elevated {
  background: var(--fandhe-color-bg);
  box-shadow: var(--fandhe-shadow-md);
}

[data-scope="card"][data-part="root"].fd-card--variant-outline {
  background: var(--fandhe-color-bg);
  border-color: var(--fandhe-color-border);
}

[data-scope="card"][data-part="root"].fd-card--variant-subtle {
  background: var(--fandhe-color-bg-subtle);
}

[data-scope="card"][data-part="header"][data-has-action] {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  column-gap: var(--fandhe-space-4);
  align-items: start;
}

[data-scope="card"][data-part="header"][data-bordered] {
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="card"][data-part="footer"][data-bordered] {
  border-top: 1px solid var(--fandhe-color-border);
}
"#;

#[test]
fn card_css_matches_golden_snapshot() {
    assert_eq!(card::css(), CARD_GOLDEN_CSS);
}

#[test]
fn card_css_declares_all_slot_selectors() {
    let css = card::css();
    assert!(css.contains(r#"[data-scope="card"][data-part="root"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="header"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="body"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="footer"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="title"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="description"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="action"]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="cover"]"#));
}

#[test]
fn card_css_declares_variant_and_size_classes() {
    let css = card::css();
    for class in [
        "fd-card--variant-elevated",
        "fd-card--variant-outline",
        "fd-card--variant-subtle",
        "fd-card--size-xs",
        "fd-card--size-sm",
        "fd-card--size-md",
        "fd-card--size-lg",
        "fd-card--size-xl",
    ] {
        assert!(css.contains(class), "missing class {class} in css: {css}");
    }
}

#[test]
fn card_css_references_theme_tokens_only() {
    let css = card::css();
    assert!(css.contains("var(--fandhe-color-fg)"));
    assert!(css.contains("var(--fandhe-space-4)"));
    assert!(css.contains("var(--fandhe-radius-lg)"));
    assert!(css.contains("var(--fandhe-shadow-md)"));
    // ハードコードされた生カラーリテラル（`#` 始まり）を含まないことを固定する
    // （イシュー #606 方針: 色宣言は必ずテーマトークン経由）。
    assert!(!css.contains('#'));
}

/// イシュー #1557: header/footer の区切り線は既定で持たない（padding のみで
/// 段を分ける）ことを固定する。イシュー #2046 で `data-bordered` opt-in 状態
/// を純追加したため、`border-bottom`/`border-top` 自体は `[data-bordered]`
/// を伴う規則にのみ許容し、それより前（既定の base/variant 規則）には一切
/// 出現しないことを検証する契約へ精緻化した（#1557 の「既定は区切り線なし」
/// という意図は保たれる）。
#[test]
fn card_css_does_not_declare_header_footer_border() {
    let css = card::css();
    let first_bordered = css
        .find("[data-bordered]")
        .expect("data-bordered state rule should exist");
    let default_css = &css[..first_bordered];
    assert!(!default_css.contains("border-bottom"));
    assert!(!default_css.contains("border-top"));
    // opt-in 状態自体は 1px 罫線をトークン経由で宣言する。
    assert!(css.contains("border-bottom: 1px solid var(--fandhe-color-border);"));
    assert!(css.contains("border-top: 1px solid var(--fandhe-color-border);"));
}

/// イシュー #1557: 影段が `shadow-sm` から `shadow-md` へ是正されたことを
/// 固定する（chakra `elevated` = `shadow.md`）。
#[test]
fn card_css_uses_shadow_md_not_shadow_sm() {
    let css = card::css();
    assert!(css.contains("var(--fandhe-shadow-md)"));
    assert!(!css.contains("var(--fandhe-shadow-sm)"));
}

/// `<style>` breakout（`</style>`）・HTML コンテキストへの脱出経路
/// （`<` 単体）が生成 CSS に含まれないことを固定する
/// （`.claude/rules/security.md` 参照）。
#[test]
fn card_css_never_contains_style_breakout_sequences() {
    let css = card::css();
    assert!(!css.contains("</style>"));
    assert!(!css.contains('<'));
}

#[test]
fn card_css_is_deterministic() {
    assert_eq!(card::css(), card::css());
}

/// イシュー #2046: `data-has-action`（header の action grid 化）・
/// `data-bordered`（header/footer 区切り線）opt-in 状態のセレクタが
/// 出力されることを固定する。
#[test]
fn card_css_declares_action_and_bordered_state_selectors() {
    let css = card::css();
    assert!(css.contains(r#"[data-scope="card"][data-part="header"][data-has-action]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="header"][data-bordered]"#));
    assert!(css.contains(r#"[data-scope="card"][data-part="footer"][data-bordered]"#));
}
