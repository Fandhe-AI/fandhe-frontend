//! styled CheckboxGroup（イシュー #997）の CSS 契約テスト。
//!
//! `crates/pre-styled-ui/tests/radio_group_css.rs`（対称の単一選択版）と
//! 同型の観点で、公開 API（`fandhe_frontend_pre_styled_ui::checkbox_group`）
//! 経由の統合テストとして固定する。単体テスト（`crates/pre-styled-ui/src/checkbox_group.rs`
//! 内の `#[cfg(test)]`）と重複する観点も、公開 API の安定性を独立に保証する
//! ため意図的に再掲する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::checkbox_group::{root, stylesheet};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

#[test]
fn stylesheet_is_deterministic() {
    let a = stylesheet();
    let b = stylesheet();
    assert_eq!(a, b);
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_targets_data_scope_checkbox_group_selectors() {
    let css = stylesheet();
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="root"]"#));
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"]"#));
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item-control"]"#));
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item-indicator"]"#));
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item-text"]"#));
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="label"]"#));
}

#[test]
fn stylesheet_does_not_reimplement_visually_hidden_hidden_input_rules() {
    // §設計判断（`crates/pre-styled-ui/src/checkbox_group.rs` rustdoc
    // 「`item-hidden-input` を本モジュールが持たない理由」節参照）:
    // ネイティブ `<input type="checkbox">` は `crate::checkbox` の
    // `hidden-input` slot が視覚的非表示化を担い、本 stylesheet では
    // 一切再宣言しない（`checkbox` recipe との重複実装の回帰固定）。
    let css = stylesheet();
    assert!(!css.contains("item-hidden-input"));
    assert!(!css.contains("clip: rect(0, 0, 0, 0);"));
}

#[test]
fn orientation_horizontal_switches_root_to_row_layout() {
    // イシュー #1460: 横並びは折り返し + space-4 の列間隔も伴う（縦積み用の
    // `gap`〔custom property 化〕を row-gap として継承し、column-gap のみ
    // 追加指定する）。
    let css = stylesheet();
    assert!(css.contains(
        r#"[data-scope="checkbox-group"][data-part="root"][data-orientation="horizontal"]"#
    ));
    assert!(css.contains("flex-direction: row;"));
    assert!(css.contains("flex-wrap: wrap;"));
    assert!(css.contains("column-gap: var(--fandhe-space-4);"));
}

#[test]
fn root_gap_is_custom_property_with_space_1_fallback() {
    // イシュー #1460: 2/2（#1461）が size variant で切り替える受け口。
    let css = stylesheet();
    assert!(css.contains("gap: var(--fandhe-checkbox-group-gap, var(--fandhe-space-1));"));
}

#[test]
fn disabled_item_gets_not_allowed_cursor() {
    let css = stylesheet();
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"][data-disabled]"#));
    assert!(css.contains("cursor: not-allowed;"));
}

#[test]
fn item_has_fit_content_width_for_click_area() {
    // イシュー #1460: Radix Themes item に合わせ、縦積み時のクリック領域が
    // 行幅いっぱいに広がるのを防ぐ。
    let css = stylesheet();
    assert!(css.contains("width: fit-content;"));
}

#[test]
fn checked_item_control_gets_palette_fill_not_circular_radio_shape() {
    let css = stylesheet();
    assert!(css.contains(
        r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"]"#
    ));
    // イシュー #1460: root からの invalid 伝播（custom property）が checked
    // 状態でも優先されるよう、border-color はその custom property を経由する。
    assert!(css.contains(
        "border-color: var(--fandhe-checkbox-group-control-border-color, var(--fandhe-palette, var(--fandhe-color-accent)));"
    ));
    // Radix Themes Checkbox Group の item-control は角丸の四角であり、
    // radio_group（円形）と異なることの回帰固定。
    assert!(!css.contains("border-radius: 50%;"));
}

#[test]
fn root_invalid_propagates_via_custom_property_but_disabled_does_not() {
    // イシュー #1460: `data-invalid` は headless 層が出力しないため、
    // `root` の `attrs` へ利用者が直接付与する経路のみで、その伝播は
    // custom property 経由の参照でのみ成立する（モジュール rustdoc 参照）。
    //
    // `data-disabled` は当初 `data-invalid` と同型の custom property
    // 間接参照（`--fandhe-checkbox-group-item-opacity`/`-item-cursor`/
    // `-item-pointer-events`）で `item` へ伝播させていたが、CSS だけでは
    // ネイティブ `<input>` のタブ順序を変更できずキーボード操作
    // （Tab+Space）を阻止できないこと、`pointer-events: none` が
    // cursor/tooltip 表示とクリック透過を壊すことが codex-review P1 /
    // Cursor Bugbot で指摘され（同一イシュー #1460 の再指摘）、撤去した
    // （`crates/pre-styled-ui/src/checkbox_group.rs` モジュール doc
    // 「スタイル調整」節参照）。この回帰固定は `root[data-disabled]` ブロック
    // 自体が出力されないことを固定する。
    let css = stylesheet();
    assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="root"][data-invalid]"#));
    assert!(
        css.contains("--fandhe-checkbox-group-control-border-color: var(--fandhe-color-danger);")
    );
    assert!(!css.contains(r#"[data-scope="checkbox-group"][data-part="root"][data-disabled]"#));
    assert!(!css.contains("--fandhe-checkbox-group-item-opacity"));
    assert!(!css.contains("--fandhe-checkbox-group-item-cursor"));
    assert!(!css.contains("--fandhe-checkbox-group-item-pointer-events"));
}

#[test]
fn item_control_invalid_sets_danger_border_color() {
    let css = stylesheet();
    assert!(
        css.contains(r#"[data-scope="checkbox-group"][data-part="item-control"][data-invalid]"#)
    );
    assert!(css.contains("border-color: var(--fandhe-color-danger);"));
}

#[test]
fn size_and_palette_variant_classes_are_present() {
    let html = render(&root(
        Size::Lg,
        ColorPalette::Success,
        false,
        None,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains("fd-checkbox-group--size-lg"));
    assert!(html.contains("fd-checkbox-group--color-palette-success"));
}

#[test]
fn class_attr_is_single_and_caller_class_is_dropped() {
    let html = render(&root(
        Size::Md,
        ColorPalette::Accent,
        false,
        None,
        None,
        vec![("class", "attacker-controlled")],
        vec![],
    ));
    assert_eq!(html.matches("class=\"").count(), 1);
    assert!(!html.contains("attacker-controlled"));
}

// --- XSS 回帰 ---

#[test]
fn xss_payload_in_caller_attrs_is_escaped_by_render() {
    let payload = "\" onmouseover=\"alert(1)";
    let html = render(&root(
        Size::Md,
        ColorPalette::Accent,
        false,
        None,
        None,
        vec![("data-testid", payload)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}
