# chart-stat-cards

`fandhe-frontend-pre-styled-ui` の `card` / `stat` / `sparkline` / `line-chart` /
`status` / `heading` 部品を合成した、折れ線付き統計カードの実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を
組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0050・R0049 の 2 件。出典の固有名・ファイル名は記載しません）。

上段は「今週のサマリー」見出しの下に統計カードを 3 枚並べ、各カードは
ラベル・合計値・前週比の増減・小さな sparkline を持ちます（主参照
R0050 の代表構成）。下段は「今期と前期の比較」見出しの下に統計カードを
2 枚並べ、各カードは今期合計・状態表示（達成/未達）・今期と前期を比較する
2 系列の line-chart を持ちます（主参照 R0049）。いずれも広い幅では横並び、
狭い幅（コンテナ幅 14rem 未満相当）では 1 列に積みます。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。文言・数値はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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
/// helper（内部専用）。`up` が `true` なら増加インジケーター、`false` なら
/// 減少インジケーターを使う。
fn sparkline_card(label: &str, value: &str, change: &str, up: bool, history: &[f64]) -> Node {
    let indicator = if up {
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
    let aria_label = format!("{}（今期と前期の月次推移）", input.description);
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
                &[820.0, 910.0, 880.0, 1020.0, 1150.0, 1080.0, 1240.0, 1248.0],
            ),
            sparkline_card(
                "エラー率",
                "0.42%",
                "前週比 -0.15pt",
                false,
                &[0.9, 0.8, 0.75, 0.7, 0.6, 0.55, 0.48, 0.42],
            ),
            sparkline_card(
                "平均応答時間",
                "184ms",
                "前週比 +12ms",
                true,
                &[160.0, 165.0, 158.0, 170.0, 175.0, 172.0, 180.0, 184.0],
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
```

## 原案差分メモ

- 集約元は 2 件です。上段グループ（3 枚のカード）が主参照（対応表 ID
  R0050）の代表構成で、合計値・増減・sparkline のみを持ちます。下段
  グループ（2 枚のカード）が主参照（対応表 ID R0049）の 2 系列比較 + 状態
  表示で、今期/前期の line-chart と `status` 部品による達成/未達の表示を
  併せ持ちます。
- `charts::legend`（凡例トグルボタン）は使いません。無 JS の docs サイト
  では押しても何も起きないボタンになるためで、下段の 2 系列（今期/前期）
  の区別はカードの説明文と line-chart の `aria_label` で行います。
- 系列色は既定の `--fandhe-color-chart-1`/`chart-2` を使用しています。
  参照元の配色・アイコン・文言は持ち込まず、デモデータ・説明文はすべて
  独自に書いた架空のものです。
- 見出しは `card::title` ではなく `heading(HeadingLevel::H3, ...)` を直接
  合成しています。使用部品として `heading` を実際に組み込むためで、
  見出しの階層はページ本体の `h1` → Demo の `h2` → グループ見出しの
  `h3` の順になります。
- 広い幅では複数列、狭い幅（コンテナ幅 14rem 未満相当）では 1 列に積む
  レイアウトです。カード内部（`stat`/`sparkline`/`status`/`line-chart` の
  縦並び）は block 固有の `.blocks-chart-stat-cards-body` グリッドで
  間隔を揃えています。

関連情報: [Card](../themes/card.md) / [Stat](../themes/stat.md) /
[Sparkline](../themes/sparkline.md) / [Line Chart](../themes/line-chart.md) /
[Status](../themes/status.md) / [Heading](../themes/heading.md)
