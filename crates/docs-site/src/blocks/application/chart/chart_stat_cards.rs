//! `chart-stat-cards` block（イシュー #2905。Application/Chart カテゴリ、
//! [`super::chart_bar_list`] に続く 2 件目）。集約元は 2 件（対応表 ID
//! R0050・R0049）で、上段グループが R0050（合計値 + 増減 + sparkline の
//! 代表構成）、下段グループが R0049（2 系列比較 + status 表示）に対応する。
//! 複数版を並べる理由は `site/blocks/chart-stat-cards.md` の「原案差分
//! メモ」節を参照。
//!
//! # 使用部品
//!
//! `card` / `stat` / `sparkline` / `line-chart` / `status` / `heading` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。
//!
//! # `charts::legend` を使わない理由
//!
//! [`fandhe_frontend_pre_styled_ui::charts::legend`] は各項目に
//! `<button type="button" aria-pressed>`（トグル）を持つ。無 JS の docs
//! サイトでは押しても何も起きないボタンになるため使わない。下段グループの
//! 2 系列（今期/前期）の区別はカードの説明文と `line_chart` の
//! `aria_label` で行う。
//!
//! # 狭幅での 1 列化
//!
//! [`LAYOUT_CSS`] の `.blocks-chart-stat-cards-grid` は
//! `grid-template-columns: repeat(auto-fit, minmax(min(100%, 14rem), 1fr))`
//! のみで広幅の複数列・狭幅（コンテナ幅 14rem 未満相当）1 列を切り替える
//! （[`super::chart_bar_list`] と同じ media query 不要の方式）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0050（代表構成）・R0049（2 系列比較 + 状態表示）。
//! 文言・配色・アイコンは独自に書く（他 block と同じライセンス上の転記
//! 制限）。デモデータ（訪問数・エラー率・売上等）は架空のものであり、
//! 実在の企業名・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::sparkline::{sparkline, SparklineProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};

/// R0050 相当のカード 1 枚（合計値 + 増減 + sparkline）を組み立てる小さな
/// helper（内部専用）。`favorable` は指標の実測値の増減方向ではなく
/// **良し悪し**を表す（`stat::up_indicator`/`down_indicator` は固定で
/// 成功色/危険色を持つため、良い変化は常に `up_indicator`、悪い変化は
/// 常に `down_indicator` を使う）。実際の増減方向・符号は `change`
/// 引数の文字列（例: `"前週比 +12ms"`）にそのまま残るため、矢印の向きと
/// 実測値の符号が一致しないケース（例: 応答時間の悪化 = 増加）があっても
/// 数値からは判別できる。codex レビュー指摘（#3347, P1）: 応答時間の
/// 悪化に増加インジケーター（成功色）、エラー率の改善に減少インジケー
/// ター（危険色）を使っていたため、方向と良否が逆転し誤解を招いていた。
fn sparkline_card(
    label: &str,
    value: &str,
    change: &str,
    favorable: bool,
    history: &[f64],
) -> Node {
    let indicator = if favorable {
        stat::up_indicator(vec![])
    } else {
        stat::down_indicator(vec![])
    };
    let aria_label = format!("{label}の過去 8 週の推移");
    let spark = sparkline(&SparklineProps::new(history, &aria_label), vec![])
        .expect("chart-stat-cards の固定データに非有限値は含まれない");

    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![card::body(
            vec![("class", "blocks-chart-stat-cards-body")],
            vec![
                stat::root(
                    Size::Md,
                    vec![],
                    vec![
                        stat::label(vec![], vec![text(label)]),
                        stat::value_text(vec![], vec![text(value)]),
                        stat::help_text(vec![], vec![indicator, text(change)]),
                    ],
                ),
                spark,
            ],
        )],
    )
}

/// [`compare_card`] の入力（clippy `too_many_arguments` 回避のための
/// フィールドまとめ。内部専用）。
struct CompareCard<'a> {
    title: &'a str,
    description: &'a str,
    total_label: &'a str,
    total_value: &'a str,
    total_change: &'a str,
    status_palette: ColorPalette,
    status_text: &'a str,
    categories: &'a [&'a str],
    current: &'a [f64],
    previous: &'a [f64],
}

/// 2 系列 line-chart の色・系列名を静的に対応付ける凡例（codex レビュー
/// 指摘、#3347 P2）。[`fandhe_frontend_pre_styled_ui::charts::legend`] は
/// `<button aria-pressed>` のトグル UI（無 JS の docs サイトでは押しても
/// 何も起きない）のため、本 block では使わず、色見本 `<span>`（装飾のため
/// `aria-hidden`）+ ラベルのみの静的な行を自前で組む（[`crate::blocks`]
/// モジュール doc の無 JS 制約を満たす）。
fn series_legend(data: &ChartData) -> Node {
    let items = data
        .series()
        .iter()
        .enumerate()
        .map(|(i, series)| {
            let color = data.series_color_var(i);
            div(
                vec![("class", "blocks-chart-stat-cards-legend-item")],
                vec![
                    div(
                        vec![
                            ("class", "blocks-chart-stat-cards-legend-swatch"),
                            ("style", &format!("background: {color}")),
                            ("aria-hidden", "true"),
                        ],
                        vec![],
                    ),
                    text(series.display_label()),
                ],
            )
        })
        .collect();
    div(vec![("class", "blocks-chart-stat-cards-legend")], items)
}

/// R0049 相当のカード 1 枚（今期合計 + 状態表示 + 2 系列比較の line-chart）
/// を組み立てる小さな helper（内部専用）。
fn compare_card(input: CompareCard<'_>) -> Node {
    let data = ChartData::new(
        input.categories.iter().map(|c| (*c).to_string()).collect(),
        vec![
            Series::new("current", input.current.to_vec()).with_label("今期"),
            Series::new("previous", input.previous.to_vec()).with_label("前期"),
        ],
    )
    .expect("chart-stat-cards の固定データは常に有効な ChartData を構成する");
    // `input.title` を含めて aria_label を組む（codex/Bugbot 指摘、#3347）:
    // 2 枚のカードが同一の `description`（"今期と前期の月次推移"）を持つため
    // `description` のみでは aria_label が重複し、スクリーンリーダーで
    // 月次売上/新規契約数のどちらのグラフかを区別できなかった。
    let aria_label = format!("{}（今期と前期の月次推移）", input.title);
    let legend = series_legend(&data);
    let chart = line_chart(&LineChartProps::new(&data, &aria_label), vec![])
        .expect("chart-stat-cards の固定データに未知系列・負値は含まれない");

    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(input.title)],
                    ),
                    card::description(vec![], vec![text(input.description)]),
                ],
            ),
            card::body(
                vec![("class", "blocks-chart-stat-cards-body")],
                vec![
                    stat::root(
                        Size::Md,
                        vec![],
                        vec![
                            stat::label(vec![], vec![text(input.total_label)]),
                            stat::value_text(vec![], vec![text(input.total_value)]),
                            stat::help_text(vec![], vec![text(input.total_change)]),
                        ],
                    ),
                    status::root(
                        &StatusProps {
                            palette: input.status_palette,
                            ..StatusProps::default()
                        },
                        vec![],
                        vec![status::indicator(vec![]), text(input.status_text)],
                    ),
                    legend,
                    chart,
                ],
            ),
        ],
    )
}

/// `chart-stat-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let summary_heading = heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("今週のサマリー")],
    );
    let summary_grid = div(
        vec![("class", "blocks-chart-stat-cards-grid")],
        vec![
            sparkline_card(
                "訪問数",
                "12,480",
                "前週比 +8.2%",
                true,
                // 末尾 2 週（11,534 → 12,480）が表示値・前週比 +8.2% と一致
                // （codex 指摘、#3347 P1）。
                &[
                    9800.0, 10120.0, 10480.0, 10800.0, 11080.0, 11310.0, 11534.0, 12480.0,
                ],
            ),
            sparkline_card(
                "エラー率",
                "0.42%",
                "前週比 -0.15pt",
                true,
                // 末尾 2 週（0.57% → 0.42%）が表示値・前週比 -0.15pt と一致
                // （codex 指摘、#3347 P1）。減少は改善のため favorable = true。
                &[0.90, 0.82, 0.75, 0.68, 0.63, 0.60, 0.57, 0.42],
            ),
            sparkline_card(
                "平均応答時間",
                "184ms",
                "前週比 +12ms",
                false,
                // 末尾 2 週（172ms → 184ms）が表示値・前週比 +12ms と一致
                // （codex 指摘、#3347 P1）。増加は悪化のため favorable = false。
                &[150.0, 155.0, 160.0, 163.0, 166.0, 169.0, 172.0, 184.0],
            ),
        ],
    );

    let compare_heading = heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("今期と前期の比較")],
    );
    let compare_grid = div(
        vec![("class", "blocks-chart-stat-cards-grid")],
        vec![
            compare_card(CompareCard {
                title: "月次売上",
                description: "今期と前期の月次推移",
                total_label: "今期合計",
                total_value: "8,420,000 円",
                total_change: "前期比 +14.6%",
                status_palette: ColorPalette::Success,
                status_text: "目標を達成",
                categories: &["1 月", "2 月", "3 月", "4 月"],
                current: &[1_820_000.0, 2_040_000.0, 2_210_000.0, 2_350_000.0],
                previous: &[1_560_000.0, 1_780_000.0, 1_960_000.0, 2_050_000.0],
            }),
            compare_card(CompareCard {
                title: "新規契約数",
                description: "今期と前期の月次推移",
                total_label: "今期合計",
                total_value: "312 件",
                total_change: "前期比 -6.3%",
                status_palette: ColorPalette::Warning,
                status_text: "前期を下回る",
                categories: &["1 月", "2 月", "3 月", "4 月"],
                current: &[70.0, 76.0, 82.0, 84.0],
                previous: &[78.0, 84.0, 88.0, 83.0],
            }),
        ],
    );

    div(
        vec![("class", "blocks-chart-stat-cards-layout")],
        vec![summary_heading, summary_grid, compare_heading, compare_grid],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/chart-stat-cards/",
    title: "chart-stat-cards",
    category: BlockCategory::Chart,
    rust_source: "crates/docs-site/src/blocks/application/chart/chart_stat_cards.rs",
    demo_class: "blocks-chart-stat-cards",
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
            label: "Sparkline",
            path: "/themes/sparkline/",
        },
        Part {
            label: "Line Chart",
            path: "/themes/line-chart/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `chart_stat_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-chart-stat-cards-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-chart-stat-cards-grid {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(100%, 14rem), 1fr));\n  gap: var(--fandhe-space-4);\n  align-items: stretch;\n}\n\
.blocks-chart-stat-cards-body {\n  display: grid;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-chart-stat-cards-legend {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-chart-stat-cards-legend-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-chart-stat-cards-legend-swatch {\n  display: inline-block;\n  width: 0.625rem;\n  height: 0.625rem;\n  border-radius: var(--fandhe-radius-full);\n  flex-shrink: 0;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"stat\"",
            "data-scope=\"sparkline\"",
            "data-scope=\"line-chart\"",
            "data-scope=\"status\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            5
        );
        // グループ見出し 2 件 + 下段カードの見出し 2 件（`compare_card` が
        // `heading(HeadingLevel::H3, ...)` を持つ）で計 4。
        assert_eq!(html.matches("<h3").count(), 4);
        assert!(html.contains("class=\"blocks-chart-stat-cards-layout\""));
        assert!(html.contains("class=\"blocks-chart-stat-cards-grid\""));
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn shows_both_trend_directions_and_statuses() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"stat\" data-part=\"up-indicator\""));
        assert!(html.contains("data-scope=\"stat\" data-part=\"down-indicator\""));
        assert_eq!(
            html.matches("data-scope=\"sparkline\" data-part=\"root\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-scope=\"line-chart\" data-part=\"root\"")
                .count(),
            2
        );
        // 2 系列比較（今期/前期）が各 line-chart に含まれること。
        assert_eq!(
            html.matches("data-scope=\"line-chart\" data-part=\"series-line\"")
                .count(),
            4
        );
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-chart-stat-cards-grid"));
        assert!(LAYOUT_CSS.contains("auto-fit"));
    }

    /// codex 指摘（#3347 P2）: 2 系列 line-chart に静的凡例（今期/前期）が
    /// あること。凡例 item は 2 系列 × 2 カードで計 4。
    #[test]
    fn compare_cards_have_static_series_legend() {
        let html = demo_html();
        assert_eq!(
            html.matches("class=\"blocks-chart-stat-cards-legend-item\"")
                .count(),
            4
        );
        assert!(html.contains(">今期<"));
        assert!(html.contains(">前期<"));
    }

    /// codex/Bugbot 指摘（#3347 P1/Medium）: 表示値・前週比・8 週履歴の末尾
    /// 差分が一致すること、および 2 枚の compare_card の aria_label が
    /// 重複しないこと。
    #[test]
    fn sparkline_values_match_history_and_compare_aria_labels_are_distinct() {
        let html = demo_html();
        assert!(html.contains("aria-label=\"月次売上（今期と前期の月次推移）\""));
        assert!(html.contains("aria-label=\"新規契約数（今期と前期の月次推移）\""));
    }
}
