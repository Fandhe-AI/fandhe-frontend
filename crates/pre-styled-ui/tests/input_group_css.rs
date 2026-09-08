//! styled Input Group（イシュー #2063、親 #2061）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/fieldset_css.rs` と同型の golden fixture
//! テスト。`input_group` recipe は `root`/`addon`/`text`/`button` の 4 slot
//! を宣言し、変数（size/variant/color-palette）を一切持たない
//! （`src/input_group.rs` モジュール doc「variant 軸: 持たない」節参照）。

use fandhe_frontend_pre_styled_ui::input_group;

const INPUT_GROUP_GOLDEN_CSS: &str = "[data-scope=\"input-group\"][data-part=\"root\"] {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  color: var(--fandhe-color-fg);
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  transition-property: border-color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"input-group\"][data-part=\"addon\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-2);
  box-sizing: border-box;
  padding-block: var(--fandhe-space-1);
  padding-inline: var(--fandhe-space-2);
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  user-select: none;
  cursor: text;
}

[data-scope=\"input-group\"][data-part=\"text\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
}

[data-scope=\"input-group\"][data-part=\"button\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  box-sizing: border-box;
  min-height: var(--fandhe-size-control-height-xs, 2rem);
  padding: 0 var(--fandhe-space-2);
  border: 0;
  border-radius: var(--fandhe-radius-sm);
  background: transparent;
  color: var(--fandhe-color-fg);
  font: inherit;
  font-size: var(--fandhe-font-font-size-sm);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"input-group\"][data-part=\"root\"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"input-group\"][data-part=\"root\"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}

[data-scope=\"input-group\"][data-part=\"root\"][data-disabled] {
  cursor: not-allowed;
}

[data-scope=\"input-group\"][data-part=\"addon\"][data-align=\"inline-start\"] {
  order: -1;
}

[data-scope=\"input-group\"][data-part=\"addon\"][data-align=\"inline-end\"] {
  order: 1;
}

[data-scope=\"input-group\"][data-part=\"addon\"][data-align=\"block-start\"] {
  order: -1;
  flex-basis: 100%;
  justify-content: flex-start;
  padding: var(--fandhe-space-2) var(--fandhe-space-3) 0;
}

[data-scope=\"input-group\"][data-part=\"addon\"][data-align=\"block-end\"] {
  order: 1;
  flex-basis: 100%;
  justify-content: flex-start;
  padding: 0 var(--fandhe-space-3) var(--fandhe-space-2);
}

[data-scope=\"input-group\"][data-part=\"addon\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"input-group\"][data-part=\"button\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope=\"input-group\"][data-part=\"button\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope=\"input-group\"][data-part=\"button\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"input\"] {
  flex: 1 1 0%;
  min-width: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"input\"]:focus-visible {
  outline: none;
}

[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"textarea\"] {
  flex: 1 1 0%;
  min-width: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"textarea\"]:focus-visible {
  outline: none;
}
";

#[test]
fn input_group_css_matches_golden_fixture() {
    assert_eq!(input_group::stylesheet(), INPUT_GROUP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(input_group::stylesheet(), input_group::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = input_group::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// raw CSS 追記の対象セレクタ（`root > field::input`/`root > field::textarea`
/// の枠線・角丸・背景リセットと `:focus-visible` の outline 無効化）が
/// 存在することを固定する（`src/input_group.rs` モジュール doc「raw CSS
/// 追記の理由」節参照）。
#[test]
fn css_appends_inner_control_reset_and_focus_override_rules() {
    let css = input_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"input\"] {"
    ));
    assert!(css.contains(
        "[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"textarea\"] {"
    ));
    assert!(css.contains(
        "[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"input\"]:focus-visible {\n  outline: none;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"input-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"textarea\"]:focus-visible {\n  outline: none;\n}"
    ));
}

/// `:focus-within` によるフォーカスリングがトークン参照（palette 軸を
/// 持たないため常に `Token`）であることを固定する。
#[test]
fn css_focus_within_ring_uses_token_color() {
    let css = input_group::stylesheet();
    assert!(css.contains("[data-scope=\"input-group\"][data-part=\"root\"]:focus-within {"));
    assert!(css.contains("var(--fandhe-color-focus-ring, var(--fandhe-color-accent))"));
}

/// `root` は内側コントロールの `data-disabled` による減光との二重適用を
/// 避けるため `disabled_declarations()`（`opacity: 0.5`）を持たない
/// （`src/input_group.rs` モジュール doc「`root` に
/// `disabled_declarations()` を付与しない理由」節参照）。
#[test]
fn css_does_not_apply_opacity_to_root_disabled_state() {
    let css = input_group::stylesheet();
    assert!(!css
        .contains("[data-scope=\"input-group\"][data-part=\"root\"][data-disabled] {\n  opacity"));
    assert!(css.contains("[data-scope=\"input-group\"][data-part=\"root\"][data-disabled] {\n  cursor: not-allowed;\n}"));
}

/// `data-align` 4 値すべての state 規則が存在することを固定する。
#[test]
fn css_declares_all_four_data_align_state_rules() {
    let css = input_group::stylesheet();
    for align in ["inline-start", "inline-end", "block-start", "block-end"] {
        assert!(css.contains(&format!(
            "[data-scope=\"input-group\"][data-part=\"addon\"][data-align=\"{align}\"] {{"
        )));
    }
}
