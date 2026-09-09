//! styled Bubble（イシュー #2109、親 #2107、headless 側 anatomy は #2108）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/message_css.rs` と同型の golden fixture
//! テスト。`bubble` recipe は `root`/`content`/`reactions`/`reaction`/
//! `collapse-trigger`/`collapse-content` の 6 slot を宣言し、
//! variant/align/group-position/selected/state は headless の `data-*` を
//! `AttrEq`/`Attr`/`AttrEqAll` で参照するのみで class ベースの軸を持たない
//! （`src/bubble.rs` モジュール doc参照）。角丸連結は `AttrEqAll` のみで
//! 表現できるため raw CSS 追記を持たない（`message`/`item` との意図的
//! 差分、モジュール doc「角丸連結」節参照）。

use fandhe_frontend_pre_styled_ui::bubble;

const BUBBLE_GOLDEN_CSS: &str = "[data-scope=\"bubble\"][data-part=\"root\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  align-self: flex-start;
  max-width: var(--fandhe-bubble-max-width, 32rem);
  min-width: 0;
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  border-radius: var(--fandhe-radius-2xl);
  background: var(--fandhe-bubble-bg);
  color: var(--fandhe-bubble-fg);
  border: 1px solid var(--fandhe-bubble-border);
  font-size: var(--fandhe-font-font-size-sm);
  overflow-wrap: anywhere;
  --fandhe-bubble-bg: var(--fandhe-color-accent);
  --fandhe-bubble-fg: var(--fandhe-color-accent-fg);
  --fandhe-bubble-border: transparent;
}

[data-scope=\"bubble\"][data-part=\"content\"] {
  min-width: 0;
}

[data-scope=\"bubble\"][data-part=\"reactions\"] {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fandhe-space-1);
  align-self: flex-end;
}

[data-scope=\"bubble\"][data-part=\"reaction\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  padding: 0 var(--fandhe-space-2);
  border-radius: var(--fandhe-radius-full);
  border: 1px solid var(--fandhe-color-border);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-xs);
  line-height: 1.4;
}

[data-scope=\"bubble\"][data-part=\"reaction\"] {
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"bubble\"][data-part=\"collapse-trigger\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  padding: 0;
  border: 0;
  background: transparent;
  color: inherit;
  font-size: var(--fandhe-font-font-size-xs);
  cursor: pointer;
  text-decoration: underline;
  border-radius: var(--fandhe-radius-sm, 0.25rem);
}

[data-scope=\"bubble\"][data-part=\"collapse-trigger\"] {
  transition-property: color, opacity;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"bubble\"][data-part=\"collapse-content\"] {
  font-size: var(--fandhe-font-font-size-xs);
  opacity: 1;
}

[data-scope=\"bubble\"][data-part=\"collapse-content\"] {
  transition-property: opacity;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"solid\"] {
  --fandhe-bubble-bg: var(--fandhe-color-accent);
  --fandhe-bubble-fg: var(--fandhe-color-accent-fg);
  --fandhe-bubble-border: transparent;
}

[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"outline\"] {
  --fandhe-bubble-bg: var(--fandhe-color-bg);
  --fandhe-bubble-fg: var(--fandhe-color-fg);
  --fandhe-bubble-border: var(--fandhe-color-border);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"plain\"] {
  --fandhe-bubble-bg: transparent;
  --fandhe-bubble-fg: var(--fandhe-color-fg);
  --fandhe-bubble-border: transparent;
  padding-inline: 0;
  max-width: 100%;
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"end\"] {
  align-self: flex-end;
  margin-inline-start: auto;
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"start\"][data-group-position=\"first\"] {
  border-end-start-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"start\"][data-group-position=\"middle\"] {
  border-start-start-radius: var(--fandhe-radius-sm);
  border-end-start-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"start\"][data-group-position=\"last\"] {
  border-start-start-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"end\"][data-group-position=\"first\"] {
  border-end-end-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"end\"][data-group-position=\"middle\"] {
  border-start-end-radius: var(--fandhe-radius-sm);
  border-end-end-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"end\"][data-group-position=\"last\"] {
  border-start-end-radius: var(--fandhe-radius-sm);
}

[data-scope=\"bubble\"][data-part=\"reaction\"][data-selected] {
  background: var(--fandhe-color-accent-subtle);
  border-color: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg-subtle);
}

[data-scope=\"bubble\"][data-part=\"collapse-trigger\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"bubble\"][data-part=\"collapse-content\"][data-state=\"open\"] {
  opacity: 1;
}

[data-scope=\"bubble\"][data-part=\"collapse-content\"][data-state=\"closed\"] {
  opacity: 0;
}

[data-scope=\"bubble\"][data-part=\"collapse-content\"][hidden] {
  display: none;
}

@media (hover: hover) {
  [data-scope=\"bubble\"][data-part=\"collapse-trigger\"]:hover:not([data-disabled]) {
    opacity: 0.8;
  }
}
";

/// `stylesheet()` の全文バイト一致を固定する（golden）。意図しない宣言の
/// 混入・欠落を検知する（`docs/internal/pre-styled-ui-golden-test-update-guide.md`
/// の更新手順に従う）。
#[test]
fn css_matches_golden_snapshot() {
    assert_eq!(bubble::stylesheet(), BUBBLE_GOLDEN_CSS);
}

/// 出力は毎回同一（非決定性のハッシュ順走査等が混入していないこと）。
#[test]
fn css_is_deterministic() {
    assert_eq!(bubble::stylesheet(), bubble::stylesheet());
}

/// `<style>` タグからの脱出シーケンス（`</style`）や生の `<` を一切
/// 含まないことを固定する（REQ-1 と独立した CSS 埋め込み文脈の不変条件）。
#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = bubble::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// variant（3 値）・align（`end`）・group-position（`data-align` ×
/// `data-group-position` の 6 複合）の state 規則が漏れなく存在すること
/// を固定する。
#[test]
fn css_declares_all_variant_align_group_position_state_rules() {
    let css = bubble::stylesheet();
    for variant in ["solid", "outline", "plain"] {
        assert!(css.contains(&format!(
            "[data-scope=\"bubble\"][data-part=\"root\"][data-variant=\"{variant}\"] {{"
        )));
    }
    assert!(css.contains("[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"end\"] {"));
    for align in ["start", "end"] {
        for position in ["first", "middle", "last"] {
            assert!(css.contains(&format!(
                "[data-scope=\"bubble\"][data-part=\"root\"][data-align=\"{align}\"][data-group-position=\"{position}\"] {{"
            )));
        }
    }
}

/// variant/align/group-position は headless の `data-*` を参照するのみで、
/// class ベースのセレクタ（`fd-bubble--` プレフィックス）を生成しない
/// ことを固定する。
#[test]
fn css_does_not_generate_class_based_selectors() {
    let css = bubble::stylesheet();
    assert!(!css.contains("fd-bubble--"));
    assert!(!css.contains("class="));
}

/// `reaction` の選択状態（`data-selected`）に対応する規則が存在すること
/// を固定する。
#[test]
fn css_styles_reaction_selected_state() {
    let css = bubble::stylesheet();
    assert!(css.contains("[data-scope=\"bubble\"][data-part=\"reaction\"][data-selected] {"));
    assert!(css.contains("background: var(--fandhe-color-accent-subtle);"));
}

/// `collapse-content` のフェード（`data-state` の open/closed）と `hidden`
/// 時の `display: none` を固定する（`src/bubble.rs` モジュール doc
/// 「フェードの限界」節参照）。
#[test]
fn css_fades_collapse_content_and_hides_hidden() {
    let css = bubble::stylesheet();
    assert!(css.contains("[data-scope=\"bubble\"][data-part=\"collapse-content\"][data-state=\"open\"] {\n  opacity: 1;"));
    assert!(css.contains("[data-scope=\"bubble\"][data-part=\"collapse-content\"][data-state=\"closed\"] {\n  opacity: 0;"));
    assert!(css.contains(
        "[data-scope=\"bubble\"][data-part=\"collapse-content\"][hidden] {\n  display: none;"
    ));
    assert!(css.contains("transition-property: opacity;"));
}

/// `collapse-trigger` の hover は `@media (hover: hover)` へ集約出力
/// される（イシュー #1425 の共通ビジュアル言語規約）。
#[test]
fn css_collects_collapse_trigger_hover_under_media_hover() {
    let css = bubble::stylesheet();
    assert!(css.contains("@media (hover: hover) {"));
    assert!(css.contains(
        "[data-scope=\"bubble\"][data-part=\"collapse-trigger\"]:hover:not([data-disabled]) {"
    ));
}
