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
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::sparkline::{sparkline, SparklineProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};

/// R0050 相当のカード 1 枚（合計値 + 増減 + sparkline）を組み立てる小さな
/// helper（内部専用）。`stat::up_indicator`/`down_indicator` は
/// `fandhe-frontend-pre-styled-ui` 側で増加/減少の意味に固定されている
/// （矢印形状 + 成功色/危険色の両方を持つ）ため、矢印の選択は必ず
/// `history` 末尾 2 点の実測値の増減方向で行う（`favorable`＝良し悪しでは
/// 選ばない）。良し悪しは矢印とは独立に、`change` テキストへ付与する
/// 色クラス（[`LAYOUT_CSS`] の `.blocks-chart-stat-cards-change--*`）で
/// 表現する。codex レビュー指摘（#3347, P1）: 従来は `favorable` で矢印を
/// 選んでいたため、エラー率の改善（減少）に上向き矢印、応答時間の悪化
/// （増加）に下向き矢印が出て、数値の増減方向と矛盾していた。
///
/// 矢印自体の色は [`LAYOUT_CSS`] の `.blocks-chart-stat-cards-indicator`
/// で中立色（`--fandhe-color-fg-muted`）へ上書きする（codex レビュー
/// 指摘、#3347 P2）: `up_indicator`/`down_indicator` は成功色/危険色に
/// 固定されているため、実測値は増加だが悪化（応答時間の増加等）という
/// ケースで、矢印（上向き＝成功色）と `change` テキスト（unfavorable＝
/// 危険色）が同じ行で逆の意味の色を示してしまう。矢印は「増減方向」の
/// みを形状で示し、色による良し悪しの表現は `change` テキストへ一元化
/// する。上書き用セレクタは `stat` の base（`[data-scope="stat"][data-
/// part="up-indicator"]` 相当、詳細度 0,2,0）へ本 block 固有クラスを
/// 前置し詳細度 0,3,0 へ揃える（`hero_background_media` の secondary CTA
/// 上書きと同じ手法。CSS 出力順〔pre-styled-ui.css → blocks.css〕により
/// 同値でも後勝ちで確実に上書きできるが、同型の判断に揃えて明示的に
/// 詳細度で勝たせる）。
fn sparkline_card(
    label: &str,
    value: &str,
    change: &str,
    favorable: bool,
    history: &[f64],
) -> Node {
    let increased = history
        .last()
        .zip(history.len().checked_sub(2).and_then(|i| history.get(i)))
        .is_some_and(|(last, prev)| last > prev);
    // 中立色クラス（`.blocks-chart-stat-cards-indicator`）を付与し、矢印の
    // 成功色/危険色を上書きする（上記 doc コメント参照）。
    let indicator = if increased {
        stat::up_indicator(vec![("class", "blocks-chart-stat-cards-indicator")])
    } else {
        stat::down_indicator(vec![("class", "blocks-chart-stat-cards-indicator")])
    };
    let change_class = if favorable {
        "blocks-chart-stat-cards-change--favorable"
    } else {
        "blocks-chart-stat-cards-change--unfavorable"
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
                        stat::help_text(
                            vec![],
                            vec![
                                indicator,
                                span(vec![("class", change_class)], vec![text(change)]),
                            ],
                        ),
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
                    // グループ見出し（`compare_heading`、「今期と前期の比較」）
                    // が H3 のため、その配下に並ぶ個々のカード見出し（月次
                    // 売上/新規契約数）は 1 段下げた H4 にする（codex レビュー
                    // 指摘、#3347 P2）。
                    heading(
                        HeadingLevel::H4,
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
- 見出しは `card::title` ではなく `heading(...)` を直接合成しています。
  使用部品として `heading` を実際に組み込むためで、見出しの階層はページ
  本体の `h1` → Demo の `h2` → グループ見出し（`summary_heading`/
  `compare_heading`）の `h3` → 下段カード個別見出し（月次売上/新規契約数）
  の `h4` の順になります（codex レビュー指摘、#3347 P2。以前はグループ
  見出しとカード見出しが同じ `h3` で並列していました）。
- 広い幅では複数列、狭い幅（コンテナ幅 14rem 未満相当）では 1 列に積む
  レイアウトです。カード内部（`stat`/`sparkline`/`status`/`line-chart` の
  縦並び）は block 固有の `.blocks-chart-stat-cards-body` グリッドで
  間隔を揃えています。

関連情報: [Card](../themes/card.md) / [Stat](../themes/stat.md) /
[Sparkline](../themes/sparkline.md) / [Line Chart](../themes/line-chart.md) /
[Status](../themes/status.md) / [Heading](../themes/heading.md)
