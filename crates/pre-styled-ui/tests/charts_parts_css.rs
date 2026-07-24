//! `fandhe-frontend-pre-styled-ui::charts::{axis,grid,legend,tooltip}` の
//! golden CSS フィクスチャ（イシュー #847）。
//!
//! `crates/pre-styled-ui/tests/recipe_css.rs`/`table_data_list_css.rs` と
//! 同型の「`concat!` による全文一致」方式で、各 recipe が生成する静的 CSS を
//! バイト単位で固定する。`crate::recipe::StateCondition::Hover`
//! （[`fandhe_frontend_pre_styled_ui::charts::tooltip`] の唯一の消費者）が
//! 出力する `:hover` 規則もここで固定する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルのテストは以後の
//! 削除・弱体化・`#[ignore]` 化を禁止する（既存 `tests/charts_foundation.rs`
//! と同じ方針）。

use fandhe_frontend_pre_styled_ui::charts::{axis, grid, legend, tooltip};

#[test]
fn axis_css_matches_golden_fixture_byte_for_byte() {
    let expected = concat!(
        "[data-scope=\"chart\"][data-part=\"axis-line\"] {\n",
        "  stroke: var(--fandhe-color-border);\n",
        "  stroke-width: 1;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tick-line\"] {\n",
        "  stroke: var(--fandhe-color-border-muted);\n",
        "  stroke-width: 1;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tick-label\"] {\n",
        "  fill: var(--fandhe-color-fg-muted);\n",
        "  font-size: var(--fandhe-font-font-size-xs);\n",
        "  font-family: var(--fandhe-font-font-body);\n",
        "}\n",
    );
    assert_eq!(axis::css(), expected);
}

#[test]
fn grid_css_matches_golden_fixture_byte_for_byte() {
    let expected = concat!(
        "[data-scope=\"chart\"][data-part=\"grid-line\"] {\n",
        "  stroke: var(--fandhe-color-border-muted);\n",
        "  stroke-width: 1;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"grid-line\"].fd-chart--lines-dashed {\n",
        "  stroke-dasharray: 3 3;\n",
        "}\n",
    );
    assert_eq!(grid::css(), expected);
}

#[test]
fn legend_css_matches_golden_fixture_byte_for_byte() {
    let expected = concat!(
        "[data-scope=\"chart-legend\"][data-part=\"root\"] {\n",
        "  display: flex;\n",
        "  flex-wrap: wrap;\n",
        "  align-items: center;\n",
        "  gap: var(--fandhe-space-4);\n",
        "  list-style: none;\n",
        "  padding: 0;\n",
        "  margin: 0;\n",
        "  font-size: var(--fandhe-font-font-size-sm);\n",
        "  color: var(--fandhe-color-fg);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"title\"] {\n",
        "  font-weight: var(--fandhe-font-font-weight-semibold);\n",
        "  margin-right: var(--fandhe-space-2);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"item\"] {\n",
        "  display: inline-flex;\n",
        "  align-items: center;\n",
        "  gap: var(--fandhe-space-2);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"marker\"] {\n",
        "  display: inline-block;\n",
        "  width: 0.75rem;\n",
        "  height: 0.75rem;\n",
        "  border-radius: var(--fandhe-radius-full);\n",
        "  flex-shrink: 0;\n",
        "}\n",
    );
    assert_eq!(legend::css(), expected);
}

#[test]
fn tooltip_css_matches_golden_fixture_byte_for_byte() {
    let expected = concat!(
        "[data-scope=\"chart\"][data-part=\"datum\"] {\n",
        "  cursor: default;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"datum\"]:hover {\n",
        "  stroke: var(--fandhe-color-accent-emphasized);\n",
        "  stroke-width: 2;\n",
        "}\n",
    );
    assert_eq!(tooltip::css(), expected);
}

#[test]
fn tooltip_css_declares_hover_state_selector() {
    // `crate::recipe::StateCondition::Hover`（イシュー #847）の唯一の
    // 消費者であることを固定する（`recipe.rs` 側の golden テストとは別に、
    // 実際の消費モジュール経由でも `:hover` が出力されることを確認する）。
    assert!(tooltip::css().contains(":hover"));
}

#[test]
fn css_outputs_are_deterministic_across_repeated_calls() {
    assert_eq!(axis::css(), axis::css());
    assert_eq!(grid::css(), grid::css());
    assert_eq!(legend::css(), legend::css());
    assert_eq!(tooltip::css(), tooltip::css());
}

#[test]
fn css_outputs_never_contain_angle_bracket() {
    for css in [axis::css(), grid::css(), legend::css(), tooltip::css()] {
        assert!(!css.contains('<'));
    }
}
