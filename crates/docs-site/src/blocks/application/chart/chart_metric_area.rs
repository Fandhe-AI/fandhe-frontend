//! `chart-metric-area` block（イシュー #2904。親トラッキング #2892）。
//!
//! # 使用部品
//!
//! `card` / `stat` / `area-chart` / `button` / `charts` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 2 variant 併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 「指標切り替え」（`variant="switch"`。切り替えボタンの下に選択中 1 件の
//! 面グラフを表示）と「合計＋内訳」（`variant="breakdown"`。合計と系列別
//! 内訳を並べ、下に積み上げ面グラフ＋凡例を表示）を 1 ページに縦へ併記
//! する（片方を選んで JS で切り替える機構は持たない）。指標切り替えボタン
//! は `aria-pressed` で選択状態を静的に伝えるのみで、クリックしても表示は
//! 変化しない（[`LAYOUT_CSS`] 参照）。
//!
//! # `<button>` の内側に `dl`/`dd` を置かない理由
//!
//! `stat::root`/`value_text` は `<dl>`/`<dd>` を出力するが、`<button>` の
//! content model は phrasing content に限られるため、指標切り替えボタンの
//! 内側ではラベル・値・増減をいずれも `span` で組む（`stat::up_indicator`/
//! `down_indicator` の装飾 `span` のみを流用する）。合計・内訳側は
//! ボタンで括らない静的な見出しのため、`stat::root` の `<dl>` 意味論を
//! そのまま使える。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `card::root` / `stat::root` / `button::button` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! ため、Demo 固有のスタイルフックは `data-blocks-chart-metric-area-*`
//! 属性で渡す（`dashboard_01` と同じ判断）。`card::header`/`body`/`title`/
//! `description`・素の `div`/`span` には `class` がそのまま効く。
//!
//! # gradient id の一意化
//!
//! `blocks_contract::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! が全 block を横断して ID 重複を検査するため、2 variant の
//! `gradient_id` は互いに異なる値にする。
//!
//! # 集約元との差分
//!
//! 主参照（指標切り替え、`variant="switch"`）と、見出しに合計値と系列別の
//! 内訳を並べる版（`variant="breakdown"`）を集約した。文言・配色・
//! アイコン・出典の固有名は持ち込まず、デモ文言・データは独自に用意する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::area_chart::{
    self, AreaChartProps, AreaCurve, AreaFill, AreaStack,
};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::charts::data::{total, ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::legend::{legend, LegendProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::Size;

/// 指標切り替えボタン 1 個分（`<button>` の phrasing content 制約のため
/// `span` のみで組む、モジュール doc「`<button>` の内側に…」節参照）。
fn metric_button(
    label: &'static str,
    value: &'static str,
    delta: &'static str,
    up: bool,
    selected: bool,
) -> Node {
    let mut attrs: Vec<(&str, &str)> =
        vec![("aria-pressed", if selected { "true" } else { "false" })];
    if selected {
        attrs.push(("data-blocks-chart-metric-area-selected", ""));
    }
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            ..ButtonProps::default()
        },
        attrs,
        vec![
            span(
                vec![("class", "blocks-chart-metric-area-metric-label")],
                vec![text(label)],
            ),
            span(
                vec![("class", "blocks-chart-metric-area-metric-value")],
                vec![text(value)],
            ),
            span(
                vec![("class", "blocks-chart-metric-area-metric-delta")],
                vec![
                    if up {
                        stat::up_indicator(vec![])
                    } else {
                        stat::down_indicator(vec![])
                    },
                    text(delta),
                ],
            ),
        ],
    )
}

/// 「指標切り替え」variant（主参照。`variant="switch"`）。3 指標のうち
/// 「Sessions」のみ選択中の固定状態を示す（無 JS のため切り替えない）。
fn instance_switch() -> Node {
    let metrics = div(
        vec![("data-blocks-chart-metric-area-metrics", "")],
        vec![
            metric_button("Active users", "8,420", "+3.2%", true, false),
            metric_button("Sessions", "21,930", "+11.4%", true, true),
            metric_button("Conversion", "4.6%", "-0.3%", false, false),
        ],
    );

    let categories: Vec<String> = dummy_assets::SAMPLE_CHART_CATEGORIES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let data = ChartData::new(
        categories,
        vec![Series::new(
            "sessions",
            dummy_assets::SAMPLE_CHART_SERIES_A.to_vec(),
        )],
    )
    .expect("chart-metric-area 固定データは常に有効な ChartData を構築できる");

    let chart = area_chart::area_chart(
        &AreaChartProps {
            curve: AreaCurve::Natural,
            fill: AreaFill::Gradient,
            gradient_id: "blocks-chart-metric-area-switch",
            show_x_axis: true,
            show_y_axis: true,
            show_grid: true,
            width: 720.0,
            height: 220.0,
            ..AreaChartProps::new(&data, "Sessions over the selected period")
        },
        vec![("data-blocks-chart-metric-area-chart", "")],
    )
    .expect("chart-metric-area 固定データは常に有効な area_chart を構築できる");

    div(
        vec![
            ("class", "blocks-chart-metric-area-layout"),
            ("data-blocks-chart-metric-area-variant", "switch"),
        ],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![
                card::header(
                    vec![],
                    vec![
                        card::title(vec![], vec![text("Workspace activity")]),
                        card::description(vec![], vec![text("Last 6 months, by metric")]),
                    ],
                ),
                card::body(vec![], vec![metrics, chart]),
            ],
        )],
    )
}

/// 「合計＋内訳」variant（`variant="breakdown"`）。合計値と系列別内訳を
/// 見出しに並べ、下に積み上げ面グラフ＋凡例を表示する。
fn instance_breakdown() -> Node {
    let categories: Vec<String> = dummy_assets::SAMPLE_CHART_CATEGORIES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let series_a = Series::new("desktop", dummy_assets::SAMPLE_CHART_SERIES_A.to_vec());
    let series_b = Series::new("mobile", dummy_assets::SAMPLE_CHART_SERIES_B.to_vec());
    let total_value = total(&series_a) + total(&series_b);
    let data = ChartData::new(categories, vec![series_a, series_b])
        .expect("chart-metric-area 固定データは常に有効な ChartData を構築できる");

    let summary = stat::root(
        Size::Lg,
        vec![("data-blocks-chart-metric-area-total", "")],
        vec![
            stat::label(vec![], vec![text("Total")]),
            stat::value_text(vec![], vec![text(format!("{total_value:.0}"))]),
            stat::help_text(
                vec![],
                vec![
                    stat::up_indicator(vec![]),
                    text("+9.1% vs. previous period"),
                ],
            ),
        ],
    );

    let breakdown = div(
        vec![("class", "blocks-chart-metric-area-breakdown-list")],
        vec![
            stat::root(
                Size::Sm,
                vec![],
                vec![
                    stat::label(vec![], vec![text("Desktop")]),
                    stat::value_text(
                        vec![],
                        vec![text(format!("{:.0}", total(&data.series()[0])))],
                    ),
                ],
            ),
            stat::root(
                Size::Sm,
                vec![],
                vec![
                    stat::label(vec![], vec![text("Mobile")]),
                    stat::value_text(
                        vec![],
                        vec![text(format!("{:.0}", total(&data.series()[1])))],
                    ),
                ],
            ),
        ],
    );

    let chart = area_chart::area_chart(
        &AreaChartProps {
            curve: AreaCurve::Natural,
            fill: AreaFill::Gradient,
            stack: AreaStack::Normal,
            gradient_id: "blocks-chart-metric-area-breakdown",
            show_x_axis: true,
            show_y_axis: true,
            show_grid: true,
            legend: true,
            width: 720.0,
            height: 220.0,
            ..AreaChartProps::new(
                &data,
                "Desktop and mobile sessions over the selected period",
            )
        },
        vec![("data-blocks-chart-metric-area-chart", "")],
    )
    .expect("chart-metric-area 固定データは常に有効な area_chart を構築できる");

    let legend_node = legend(&data, &LegendProps::default());

    div(
        vec![
            ("class", "blocks-chart-metric-area-layout"),
            ("data-blocks-chart-metric-area-variant", "breakdown"),
        ],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![
                card::header(
                    vec![
                        ("class", "blocks-chart-metric-area-summary-row"),
                        ("data-blocks-chart-metric-area-summary-row", ""),
                    ],
                    vec![summary, breakdown],
                ),
                card::body(vec![], vec![chart, legend_node]),
            ],
        )],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-chart-metric-area-caption")],
        vec![text(label)],
    )
}

/// `chart-metric-area` の Demo 本体（2 variant 併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-chart-metric-area-stack")],
        vec![
            caption("指標切り替え（選択中の 1 指標を面グラフで表示）"),
            instance_switch(),
            caption("合計＋内訳（積み上げ面グラフ＋凡例）"),
            instance_breakdown(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/chart-metric-area/",
    title: "chart-metric-area",
    category: BlockCategory::Chart,
    rust_source: "crates/docs-site/src/blocks/application/chart/chart_metric_area.rs",
    demo_class: "blocks-chart-metric-area",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Area Chart",
            path: "/themes/area-chart/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Charts",
            path: "/themes/charts/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `chart_metric_area` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。指標切り替えボタン行は狭幅で縦積み、40rem 以上で横並びにする。
/// 選択中のボタンは下線とアクセント色で強調する。合計＋内訳の見出し行は
/// 狭幅で縦積み、32rem 以上で横並びにする。
const LAYOUT_CSS: &str = "\
.blocks-chart-metric-area-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-chart-metric-area-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-chart-metric-area-metrics] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin-block-end: var(--fandhe-space-4);\n}\n\
.blocks-chart-metric-area-metric-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-chart-metric-area-metric-value {\n  display: block;\n  font-size: var(--fandhe-font-font-size-lg, 1.125rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-chart-metric-area-metric-delta {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-chart-metric-area-selected] {\n  border-block-end: 2px solid var(--fandhe-color-accent);\n}\n\
[data-blocks-chart-metric-area-chart] {\n  width: 100%;\n}\n\
[data-blocks-chart-metric-area-chart] svg {\n  width: 100%;\n  height: auto;\n}\n\
[data-blocks-chart-metric-area-summary-row] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-chart-metric-area-breakdown-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-chart-metric-area-metrics] {\n    flex-direction: row;\n    gap: var(--fandhe-space-6);\n  }\n\
}\n\
@media (min-width: 32rem) {\n  \
[data-blocks-chart-metric-area-summary-row] {\n    flex-direction: row;\n    align-items: flex-start;\n  }\n  \
.blocks-chart-metric-area-breakdown-list {\n    flex-direction: row;\n    gap: var(--fandhe-space-6);\n    border-inline-start: 1px solid var(--fandhe-color-border);\n    padding-inline-start: var(--fandhe-space-6);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 部品を持ち、非対話制約（`<form>` なし・
    /// `data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"stat\"",
            "data-scope=\"button\"",
            "data-scope=\"chart-legend\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("<svg"));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// ボタンが 3 つとも `type="button"` で、選択中がちょうど 1 件であること。
    #[test]
    fn switch_metrics_have_exactly_one_selected() {
        let html = render(&demo());
        // breakdown 側の legend トリガー（系列 2 件分）も `aria-pressed` を
        // 持つため、指標切り替えボタンの選択状態は一意なフック属性
        // （`data-blocks-chart-metric-area-selected`）で確認する。
        assert_eq!(
            html.matches("data-blocks-chart-metric-area-selected")
                .count(),
            1
        );
        // 指標ボタン 3 + legend トリガー 2 = type="button" 5 件。
        assert_eq!(html.matches("type=\"button\"").count(), 5);
    }

    /// `<button` の内側に `<dl`/`<dd` が現れないこと（phrasing content 制約）。
    #[test]
    fn metric_buttons_do_not_contain_dl_or_dd() {
        let html = render(&demo());
        for button_html in html.split("<button").skip(1) {
            let end = button_html.find("</button>").unwrap_or(button_html.len());
            let inner = &button_html[..end];
            assert!(!inner.contains("<dl"));
            assert!(!inner.contains("<dd"));
        }
    }

    /// 2 variant がそれぞれ 1 回ずつ出て、caption も 2 件あること。
    #[test]
    fn demo_renders_both_variants() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-chart-metric-area-variant=\"switch\""));
        assert!(html.contains("data-blocks-chart-metric-area-variant=\"breakdown\""));
        assert_eq!(html.matches("blocks-chart-metric-area-caption").count(), 2);
    }

    /// gradient id が 2 種類とも出力されていて互いに異なること（ID 重複回避）。
    #[test]
    fn gradient_ids_are_unique_across_variants() {
        let html = render(&demo());
        assert!(html.contains("blocks-chart-metric-area-switch"));
        assert!(html.contains("blocks-chart-metric-area-breakdown"));
    }

    /// [`LAYOUT_CSS`] が `@media (min-width: …)` を持つこと。
    #[test]
    fn layout_css_has_responsive_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 32rem)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-chart-metric-area-layout"));
        assert_ne!(super::BLOCK.demo_class, "blocks-chart-metric-area-layout");
    }
}
