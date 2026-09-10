//! styled Questionnaire（イシュー #2119、親 #2116、headless 側 anatomy は
//! #2117、wasm-full 配線は #2118）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/marker_css.rs` と同型の golden fixture
//! テスト。`questionnaire` recipe は 11 slot を宣言し、class 軸を持たず
//! headless の `data-*` を `Attr`/`AttrEq` で参照するのみである（`src/
//! questionnaire.rs` モジュール doc参照）。`options` slot 配下の
//! radio-group/checkbox-group item をカード状に整形する子孫セレクタは
//! `SlotRecipe::state`（単一 slot 前提）では表現できず raw CSS として
//! 追記する。

use fandhe_frontend_pre_styled_ui::questionnaire;

const QUESTIONNAIRE_GOLDEN_CSS: &str = "[data-scope=\"questionnaire\"][data-part=\"root\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-4);
}

[data-scope=\"questionnaire\"][data-part=\"progress\"] {
  display: block;
  height: var(--fandhe-questionnaire-progress-height, 0.375rem);
  border-radius: var(--fandhe-radius-full, 999px);
  background: linear-gradient(to right, var(--fandhe-color-accent) var(--fandhe-questionnaire-percent, 0%), var(--fandhe-color-bg-muted) 0);
}

[data-scope=\"questionnaire\"][data-part=\"question\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-3);
  margin: 0;
  padding: var(--fandhe-space-4);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg);
  min-width: 0;
}

[data-scope=\"questionnaire\"][data-part=\"prompt\"] {
  padding: 0;
  font-size: var(--fandhe-font-font-size-md);
  font-weight: var(--fandhe-font-font-weight-semibold);
  color: var(--fandhe-color-fg);
}

[data-scope=\"questionnaire\"][data-part=\"description\"] {
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"questionnaire\"][data-part=\"options\"] {
  display: grid;
  gap: var(--fandhe-space-2);
}

[data-scope=\"questionnaire\"][data-part=\"freeform\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope=\"questionnaire\"][data-part=\"actions\"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fandhe-space-2);
  justify-content: flex-end;
}

[data-scope=\"questionnaire\"][data-part=\"back\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--fandhe-size-control-height-md, 2.25rem);
  padding: 0 var(--fandhe-space-4);
  border-radius: var(--fandhe-radius-md);
  border: 1px solid transparent;
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  background: transparent;
  border-color: var(--fandhe-color-border);
  color: var(--fandhe-color-fg);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"questionnaire\"][data-part=\"next\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--fandhe-size-control-height-md, 2.25rem);
  padding: 0 var(--fandhe-space-4);
  border-radius: var(--fandhe-radius-md);
  border: 1px solid transparent;
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  background: var(--fandhe-color-accent);
  border-color: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"questionnaire\"][data-part=\"skip\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--fandhe-size-control-height-md, 2.25rem);
  padding: 0 var(--fandhe-space-4);
  border-radius: var(--fandhe-radius-md);
  border: 1px solid transparent;
  font-weight: var(--fandhe-font-font-weight-medium);
  cursor: pointer;
  background: transparent;
  border-color: transparent;
  color: var(--fandhe-color-fg-muted);
  transition-property: background, border-color, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"questionnaire\"][data-part=\"progress\"][data-complete] {
  background: linear-gradient(to right, var(--fandhe-color-success) var(--fandhe-questionnaire-percent, 0%), var(--fandhe-color-bg-muted) 0);
}

[data-scope=\"questionnaire\"][data-part=\"question\"][hidden] {
  display: none;
}

[data-scope=\"questionnaire\"][data-part=\"question\"][data-state=\"completed\"] {
  border-color: var(--fandhe-color-success);
  background: var(--fandhe-color-success-subtle);
}

[data-scope=\"questionnaire\"][data-part=\"question\"][data-state=\"upcoming\"] {
  border-style: dashed;
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"questionnaire\"][data-part=\"question\"][data-answered] {
  box-shadow: inset 3px 0 0 var(--fandhe-color-accent);
}

[data-scope=\"questionnaire\"][data-part=\"question\"][data-skipped] {
  opacity: 0.7;
  font-style: italic;
}

[data-scope=\"questionnaire\"][data-part=\"question\"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope=\"questionnaire\"][data-part=\"back\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"questionnaire\"][data-part=\"back\"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"back\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"next\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"questionnaire\"][data-part=\"next\"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"next\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"skip\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"questionnaire\"][data-part=\"skip\"][disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"skip\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope=\"questionnaire\"][data-part=\"back\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  }

  [data-scope=\"questionnaire\"][data-part=\"next\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    --fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));
  }

  [data-scope=\"questionnaire\"][data-part=\"skip\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  }
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"radio-group\"][data-part=\"item\"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg);
  cursor: pointer;
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"radio-group\"][data-part=\"item\"][data-state=\"checked\"] {
  border-color: var(--fandhe-color-accent);
  background: var(--fandhe-color-accent-subtle);
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"checkbox-group\"][data-part=\"item\"] {
  display: flex;
  align-items: flex-start;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg);
  cursor: pointer;
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"checkbox-group\"][data-part=\"item\"][data-state=\"checked\"] {
  border-color: var(--fandhe-color-accent);
  background: var(--fandhe-color-accent-subtle);
}

[data-scope=\"questionnaire\"][data-part=\"options\"] [data-scope=\"checkbox-group\"][data-part=\"item\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}
";

#[test]
fn stylesheet_matches_golden_byte_for_byte() {
    assert_eq!(questionnaire::stylesheet(), QUESTIONNAIRE_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic() {
    let a = questionnaire::stylesheet();
    let b = questionnaire::stylesheet();
    assert_eq!(a, b);
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let out = questionnaire::stylesheet();
    assert!(!out.contains("</style"));
    assert!(!out.contains('<'));
}

#[test]
fn stylesheet_declares_all_eleven_slots() {
    let out = questionnaire::stylesheet();
    for slot in [
        "root",
        "progress",
        "question",
        "prompt",
        "description",
        "options",
        "freeform",
        "actions",
        "back",
        "next",
        "skip",
    ] {
        let selector = format!(r#"[data-scope="questionnaire"][data-part="{slot}"]"#);
        assert!(out.contains(&selector), "missing selector for slot {slot}");
    }
}

#[test]
fn stylesheet_declares_completed_and_upcoming_but_not_active() {
    let out = questionnaire::stylesheet();
    // `active` は base と同値のため state 規則を書かない
    // （`src/questionnaire.rs` モジュール doc参照）。
    assert!(out.contains(r#"[data-state="completed"]"#));
    assert!(out.contains(r#"[data-state="upcoming"]"#));
    assert!(!out.contains(r#"[data-state="active"]"#));
}

#[test]
fn stylesheet_declares_hidden_display_none_for_question() {
    let out = questionnaire::stylesheet();
    assert!(out.contains("[hidden] {"));
    assert!(out.contains("display: none;"));
}

#[test]
fn stylesheet_declares_answered_skipped_invalid_disabled_complete_state_rules() {
    let out = questionnaire::stylesheet();
    assert!(out.contains("[data-answered]"));
    assert!(out.contains("[data-skipped]"));
    assert!(out.contains("[data-invalid]"));
    assert!(out.contains("[disabled]"));
    assert!(out.contains("[data-disabled]"));
    assert!(out.contains("[data-complete]"));
}

#[test]
fn stylesheet_declares_choice_card_descendant_rules_for_radio_and_checkbox_group() {
    let out = questionnaire::stylesheet();
    assert!(out.contains(
        r#"[data-scope="questionnaire"][data-part="options"] [data-scope="radio-group"][data-part="item"] {"#
    ));
    assert!(out.contains(
        r#"[data-scope="questionnaire"][data-part="options"] [data-scope="checkbox-group"][data-part="item"] {"#
    ));
    assert!(out.contains(
        r#"[data-scope="questionnaire"][data-part="options"] [data-scope="radio-group"][data-part="item"][data-state="checked"] {"#
    ));
    assert!(out.contains(
        r#"[data-scope="questionnaire"][data-part="options"] [data-scope="checkbox-group"][data-part="item"][data-state="checked"] {"#
    ));
}

#[test]
fn stylesheet_never_generates_class_based_variant_axis() {
    // class ベースの見た目軸（`fd-questionnaire--` 等）は生成しない
    // （headless の `data-*` を参照するのみ、モジュール doc参照）。
    let out = questionnaire::stylesheet();
    assert!(!out.contains("fd-questionnaire--"));
    assert!(!out.contains("class="));
}
