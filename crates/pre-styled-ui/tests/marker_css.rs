//! styled Marker（イシュー #2115、親 #2113、headless 側 anatomy は
//! #2114）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/attachment_css.rs` と同型の golden
//! fixture テスト。`marker` recipe は `root`/`icon`/`content` の 3 slot を
//! 宣言し、variant（`data-variant`）/tone（`data-tone`）は headless の
//! `data-*` を `AttrEq` で参照するのみで class ベースの軸を持たない
//! （`src/marker.rs` モジュール doc参照）。`data-variant="label"` 時に
//! 挟み込む [`fandhe_frontend_pre_styled_ui::separator::separator`] の
//! 伸縮規則・中央ラベルの折返し抑止は `root`/separator/`content` に
//! またがる子孫セレクタのため `SlotRecipe::state`（単一 slot 前提）では
//! 表現できず raw CSS として追記する（モジュール doc「区切り線の描画
//! 方式」節参照）。

use fandhe_frontend_pre_styled_ui::marker;

const MARKER_GOLDEN_CSS: &str = "[data-scope=\"marker\"][data-part=\"root\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-xs);
  color: var(--fandhe-color-fg-muted);
  --fandhe-marker-line: var(--fandhe-color-border);
}

[data-scope=\"marker\"][data-part=\"icon\"] {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  line-height: 1;
}

[data-scope=\"marker\"][data-part=\"content\"] {
  min-width: 0;
}

[data-scope=\"marker\"][data-part=\"root\"][data-variant=\"divider\"] {
  border-bottom: 1px solid var(--fandhe-marker-line, var(--fandhe-color-border));
  padding-bottom: var(--fandhe-space-2);
}

[data-scope=\"marker\"][data-part=\"root\"][data-variant=\"label\"] {
  justify-content: center;
  text-align: center;
}

[data-scope=\"marker\"][data-part=\"root\"][data-tone=\"info\"] {
  color: var(--fandhe-color-info-fg-subtle);
  --fandhe-marker-line: var(--fandhe-color-info-muted);
}

[data-scope=\"marker\"][data-part=\"root\"][data-tone=\"warning\"] {
  color: var(--fandhe-color-warning-fg-subtle);
  --fandhe-marker-line: var(--fandhe-color-warning-muted);
}

[data-scope=\"marker\"][data-part=\"root\"][data-tone=\"danger\"] {
  color: var(--fandhe-color-danger-fg-subtle);
  --fandhe-marker-line: var(--fandhe-color-danger-muted);
}

[data-scope=\"marker\"][data-part=\"root\"][data-variant=\"label\"] > [data-scope=\"separator\"][data-part=\"root\"] {
  flex: 1 1 0%;
  width: auto;
  border-color: var(--fandhe-marker-line, var(--fandhe-color-border));
}

[data-scope=\"marker\"][data-part=\"root\"][data-variant=\"label\"] > [data-scope=\"marker\"][data-part=\"content\"] {
  white-space: nowrap;
}
";

#[test]
fn stylesheet_matches_golden_byte_for_byte() {
    assert_eq!(marker::stylesheet(), MARKER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_deterministic() {
    let a = marker::stylesheet();
    let b = marker::stylesheet();
    assert_eq!(a, b);
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let out = marker::stylesheet();
    assert!(!out.contains("</style"));
    assert!(!out.contains('<'));
}

#[test]
fn stylesheet_declares_all_three_slots() {
    let out = marker::stylesheet();
    assert!(out.contains(r#"[data-scope="marker"][data-part="root"] {"#));
    assert!(out.contains(r#"[data-scope="marker"][data-part="icon"] {"#));
    assert!(out.contains(r#"[data-scope="marker"][data-part="content"] {"#));
}

#[test]
fn stylesheet_declares_variant_and_tone_state_rules() {
    let out = marker::stylesheet();
    assert!(out.contains(r#"[data-variant="divider"]"#));
    assert!(out.contains(r#"[data-variant="label"]"#));
    assert!(out.contains(r#"[data-tone="info"]"#));
    assert!(out.contains(r#"[data-tone="warning"]"#));
    assert!(out.contains(r#"[data-tone="danger"]"#));
    // `neutral` は base と同値のため state 規則を書かない
    // （`src/marker.rs` モジュール doc参照）。
    assert!(!out.contains(r#"[data-tone="neutral"]"#));
}

#[test]
fn stylesheet_declares_label_separator_descendant_rules() {
    let out = marker::stylesheet();
    assert!(out.contains(
        r#"[data-scope="marker"][data-part="root"][data-variant="label"] > [data-scope="separator"][data-part="root"] {"#
    ));
    assert!(out.contains("flex: 1 1 0%;"));
    assert!(out.contains(
        r#"[data-scope="marker"][data-part="root"][data-variant="label"] > [data-scope="marker"][data-part="content"] {"#
    ));
    assert!(out.contains("white-space: nowrap;"));
}

#[test]
fn stylesheet_never_generates_class_based_variant_or_tone_axes() {
    // class ベースの variant/tone クラス（`fd-marker--` 等）は生成しない
    // （headless の `data-*` を参照するのみ、モジュール doc参照）。
    let out = marker::stylesheet();
    assert!(!out.contains("fd-marker--"));
    assert!(!out.contains("class="));
}
