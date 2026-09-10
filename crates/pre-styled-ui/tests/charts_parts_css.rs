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
        "  stroke: var(--fandhe-color-border);\n",
        "  stroke-width: 1;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tick-label\"] {\n",
        "  fill: var(--fandhe-color-fg-muted);\n",
        "  font-size: var(--fandhe-font-font-size-xs);\n",
        "  font-family: var(--fandhe-font-font-body);\n",
        "  font-variant-numeric: tabular-nums;\n",
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
        "  width: var(--fandhe-space-3);\n",
        "  height: var(--fandhe-space-3);\n",
        "  border-radius: var(--fandhe-radius-full);\n",
        "  flex-shrink: 0;\n",
        "}\n",
        "\n",
        // イシュー #2077: 系列アイコン（shadcn/ui `ChartConfig.icon` 相当）の
        // 代替スロット。既存 5 ブロックは不変（golden 純追加原則）。
        "[data-scope=\"chart-legend\"][data-part=\"icon\"] {\n",
        "  display: inline-flex;\n",
        "  align-items: center;\n",
        "  justify-content: center;\n",
        "  flex-shrink: 0;\n",
        "  width: var(--fandhe-space-3);\n",
        "  height: var(--fandhe-space-3);\n",
        "}\n",
        "\n",
        // イシュー #2133: 凡例 item を button + aria-pressed 化する trigger
        // slot（marker/icon/label を包む）。既存 6 ブロック（root/title/item/
        // marker/icon の base 5 件 + 本ブロック直前までの並び）は不変
        // （golden 純追加原則）。
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"] {\n",
        "  appearance: none;\n",
        "  background: none;\n",
        "  border: 0;\n",
        "  padding: 0;\n",
        "  margin: 0;\n",
        "  font: inherit;\n",
        "  color: inherit;\n",
        "  cursor: pointer;\n",
        "  display: inline-flex;\n",
        "  align-items: center;\n",
        "  gap: var(--fandhe-space-2);\n",
        "  transition-property: opacity;\n",
        "  transition-duration: var(--fandhe-motion-duration-fast);\n",
        "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
        "}\n",
        "\n",
        // イシュー #2086: shadcn/ui Charts（tooltip）突合による opt-in variant
        // 軸（align/marker）。既存ブロックは不変（golden 純追加原則）。
        "[data-scope=\"chart-legend\"][data-part=\"root\"].fd-chart-legend--align-center {\n",
        "  justify-content: center;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"root\"].fd-chart-legend--align-end {\n",
        "  justify-content: flex-end;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"marker\"].fd-chart-legend--marker-square {\n",
        "  border-radius: var(--fandhe-radius-sm);\n",
        "}\n",
        "\n",
        // イシュー #2133: trigger の状態規則（非表示系列の減光・
        // キーボードフォーカスリング）。SlotRecipe::css() は state を
        // variant の後段に出力する（呼び出し順ではなく実出力位置で固定）。
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"][aria-pressed=\"false\"] {\n",
        "  opacity: 0.5;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart-legend\"][data-part=\"trigger\"]:focus-visible {\n",
        "  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n",
        "  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n",
        "}\n",
    );
    assert_eq!(legend::css(), expected);
}

#[test]
fn tooltip_css_matches_golden_fixture_byte_for_byte() {
    // イシュー #2129: hit-area・SSR ツールチップ DOM（frame/tooltip-layer/
    // tooltip/tooltip-label/tooltip-item/tooltip-indicator/tooltip-name/
    // tooltip-value）の base ブロックを `datum` 直後・`@media (hover: hover)`
    // ブロック直前へ純追加した（`hit-area` の `:focus-visible` state のみ
    // `@media` ブロックの前・states 出力順で追加）。
    let expected = concat!(
        "[data-scope=\"chart\"][data-part=\"datum\"] {\n",
        "  cursor: default;\n",
        "  stroke: var(--fandhe-color-bg);\n",
        "  stroke-width: 1;\n",
        "  transition-property: stroke, stroke-width;\n",
        "  transition-duration: var(--fandhe-motion-duration-fast);\n",
        "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"hit-area\"] {\n",
        "  outline: none;\n",
        "  cursor: default;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"frame\"] {\n",
        "  position: relative;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip-layer\"] {\n",
        "  position: absolute;\n",
        "  inset: 0;\n",
        "  pointer-events: none;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip\"] {\n",
        "  position: absolute;\n",
        "  left: var(--fandhe-chart-tooltip-x, 0px);\n",
        "  top: var(--fandhe-chart-tooltip-y, 0px);\n",
        "  transform: translate(-50%, calc(-100% - var(--fandhe-space-2)));\n",
        "  pointer-events: none;\n",
        "  background: var(--fandhe-color-bg);\n",
        "  color: var(--fandhe-color-fg);\n",
        "  border: 1px solid var(--fandhe-color-border);\n",
        "  border-radius: var(--fandhe-radius-md, 0.375rem);\n",
        "  box-shadow: var(--fandhe-shadow-md);\n",
        "  padding: var(--fandhe-space-2) var(--fandhe-space-3);\n",
        "  font-size: var(--fandhe-font-font-size-xs);\n",
        "  font-family: var(--fandhe-font-font-body);\n",
        "  white-space: nowrap;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip-label\"] {\n",
        "  font-weight: 600;\n",
        "  margin-bottom: var(--fandhe-space-1);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip-item\"] {\n",
        "  display: flex;\n",
        "  align-items: center;\n",
        "  gap: var(--fandhe-space-2);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip-indicator\"] {\n",
        "  display: inline-block;\n",
        "  width: var(--fandhe-space-2);\n",
        "  height: var(--fandhe-space-2);\n",
        "  border-radius: 2px;\n",
        "  background: var(--fandhe-chart-tooltip-color, currentColor);\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"tooltip-value\"] {\n",
        "  margin-left: auto;\n",
        "  font-variant-numeric: tabular-nums;\n",
        "}\n",
        "\n",
        "[data-scope=\"chart\"][data-part=\"hit-area\"]:focus-visible {\n",
        "  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n",
        "  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n",
        "}\n",
        "\n",
        // イシュー #2133: 凡例トグルで隠した系列のツールチップ行を連動して
        // 隠す（末尾純追加、既存ブロックは不変）。
        "[data-scope=\"chart\"][data-part=\"tooltip-item\"][data-hidden] {\n",
        "  display: none;\n",
        "}\n",
        "\n",
        "@media (hover: hover) {\n",
        "  [data-scope=\"chart\"][data-part=\"datum\"]:hover:not([data-disabled]) {\n",
        "    stroke: var(--fandhe-color-fg);\n",
        "    stroke-width: 2;\n",
        "  }\n",
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
