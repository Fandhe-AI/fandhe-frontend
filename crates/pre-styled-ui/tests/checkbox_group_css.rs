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
fn label_stays_on_its_own_line_under_horizontal_wrap() {
    // イシュー #1460 Cursor Bugbot 指摘: `data-orientation="horizontal"`
    // では `root` が `flex-wrap: wrap` の flex コンテナになり、`label` も
    // `item` と同じコンテナの兄弟要素であるため、対策なしでは折り返し行へ
    // 混入し得る。`root` 横並び state が定義する custom property を
    // `label` 側の `flex-basis` で受け取り、フルライン幅の独立行にする。
    let css = stylesheet();
    assert!(css.contains("--fandhe-checkbox-group-label-basis: 100%;"));
    assert!(css.contains("flex-basis: var(--fandhe-checkbox-group-label-basis, auto);"));
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

/// イシュー #1741: `fandhe-frontend-wasm-full` の
/// `focus_visible::boundary_candidates_for` フォールバック追加により、
/// item 配下の `checkbox::hidden_input` への実フォーカスが
/// `item-control[data-focus-visible]` へ届くようになった。`checkbox.rs`
/// の `control[data-focus-visible]` と同型のフォーカスリング CSS が
/// 実際に出力されることを固定する。
#[test]
fn item_control_focus_visible_has_focus_ring() {
    let css = stylesheet();
    assert!(css.contains(
        r#"[data-scope="checkbox-group"][data-part="item-control"][data-focus-visible]"#
    ));
    assert!(css.contains("outline: var(--fandhe-focus-ring-width, 2px) solid"));
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

#[test]
fn size_variant_xs_and_sm_control_sizes_are_on_4px_grid() {
    // イシュー #1461: xs/sm の control 寸法を 4px 格子（12px/14px）へ是正
    // する（`crate::checkbox` #1735 と同値）。md/lg/xl は現行外観を変えない。
    let css = stylesheet();
    let xs_selector =
        r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-xs"#;
    let xs_start = css
        .find(xs_selector)
        .unwrap_or_else(|| panic!("xs size variant selector not found in {css}"));
    let xs_end = css[xs_start..]
        .find('}')
        .map(|i| xs_start + i)
        .unwrap_or(css.len());
    assert!(
        css[xs_start..xs_end].contains("--fandhe-checkbox-group-control-size: 0.75rem;"),
        "xs control-size not on 4px grid: {}",
        &css[xs_start..xs_end]
    );

    let sm_selector =
        r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-sm"#;
    let sm_start = css
        .find(sm_selector)
        .unwrap_or_else(|| panic!("sm size variant selector not found in {css}"));
    let sm_end = css[sm_start..]
        .find('}')
        .map(|i| sm_start + i)
        .unwrap_or(css.len());
    assert!(
        css[sm_start..sm_end].contains("--fandhe-checkbox-group-control-size: 0.875rem;"),
        "sm control-size not on 4px grid: {}",
        &css[sm_start..sm_end]
    );
}

#[test]
fn size_variants_define_item_gap_and_root_gap_custom_properties() {
    // イシュー #1461: 5 段すべての size variant ブロックに item-gap（control
    // ↔ text 余白）と root gap（項目間余白、1/2 が用意した受け口）の両方の
    // custom property が登録されていることを固定する。
    let css = stylesheet();
    for size in ["xs", "sm", "md", "lg", "xl"] {
        let selector = format!(
            r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-{size}"#
        );
        let start = css
            .find(&selector)
            .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..end];
        assert!(
            block.contains("--fandhe-checkbox-group-item-gap"),
            "size={size} missing item-gap custom property: {block}"
        );
        assert!(
            block.contains("--fandhe-checkbox-group-gap"),
            "size={size} missing root gap custom property: {block}"
        );
    }
}

#[test]
fn label_is_medium_weight_and_item_text_is_not() {
    // イシュー #1461: グループ見出し `label` と項目テキスト `item-text` の
    // 2 段階の型階層（label = medium ウェイト、item-text = 通常ウェイト）
    // を固定する。
    let css = stylesheet();

    let label_selector = r#"[data-scope="checkbox-group"][data-part="label"] {"#;
    let label_start = css
        .find(label_selector)
        .unwrap_or_else(|| panic!("label base selector not found in {css}"));
    let label_end = css[label_start..]
        .find('}')
        .map(|i| label_start + i)
        .unwrap_or(css.len());
    assert!(
        css[label_start..label_end].contains("font-weight: var(--fandhe-font-font-weight-medium);"),
        "label missing medium font-weight: {}",
        &css[label_start..label_end]
    );

    let item_text_selector = r#"[data-scope="checkbox-group"][data-part="item-text"] {"#;
    let item_text_start = css
        .find(item_text_selector)
        .unwrap_or_else(|| panic!("item-text base selector not found in {css}"));
    let item_text_end = css[item_text_start..]
        .find('}')
        .map(|i| item_text_start + i)
        .unwrap_or(css.len());
    assert!(
        !css[item_text_start..item_text_end].contains("font-weight:"),
        "item-text should not declare font-weight: {}",
        &css[item_text_start..item_text_end]
    );
}

#[test]
fn item_text_prevents_text_selection() {
    // イシュー #1461: `item`（`<label>`）内テキストをクリックでトグルする
    // 操作の誤選択防止（chakra label と同じ、`crate::checkbox` の `label`
    // と対称）。
    let css = stylesheet();
    let selector = r#"[data-scope="checkbox-group"][data-part="item-text"] {"#;
    let start = css
        .find(selector)
        .unwrap_or_else(|| panic!("item-text base selector not found in {css}"));
    let end = css[start..]
        .find('}')
        .map(|i| start + i)
        .unwrap_or(css.len());
    assert!(
        css[start..end].contains("user-select: none;"),
        "item-text missing user-select: {}",
        &css[start..end]
    );
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
