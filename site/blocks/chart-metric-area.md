# chart-metric-area

`fandhe-frontend-pre-styled-ui` の `card` / `stat` / `area-chart` / `button` /
`charts` 部品を合成した、指標サマリーと面グラフの block です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を
組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0051。出典の固有名・ファイル名は記載しません）。

Demo はレイアウト違いの 2 variant を並記します。「指標切り替え」
（`variant="switch"`、R0051 主参照）は 3 指標の切り替えボタンを並べ、選択中
の 1 件のみ下線とアクセント色で強調し、その下に選択中の指標 1 系列の面
グラフを表示します。「合計＋内訳」（`variant="breakdown"`、R0053）は見出し
に合計値と系列別の内訳（Desktop / Mobile）を並べ、その下に 2 系列の積み
上げ面グラフと凡例を表示します。docs サイトは JS ハイドレーションを行わ
ないため、指標切り替えボタンは `aria-pressed` で選択状態を静的に伝える
のみで、クリックしても表示は変化しません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。数値・ラベルはすべて独自に書いた架空のもの
であり、実企業名・実在人物・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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

/// `legend()`（`fandhe_frontend_pre_styled_ui::charts::legend`）が生成する
/// `data-part="trigger"` の `<button type="button">` をすべて無効化する
/// （PR #3345 codex-review P1 是正、モジュール doc参照）。`legend()` は
/// disabled オプションを持たない共有部品のため、返された [`Node`] 木を
/// 走査し `disabled`/`data-disabled`/`aria-disabled="true"`（`metric_button`
/// と同じ 3 点セット、`button::button` の disabled 規約に合わせる）を
/// 該当ボタンへ追加で付与する。`aria-pressed` はそのまま残す
/// （`metric_button` も選択状態を静的に伝える `aria-pressed` を disabled と
/// 併存させており、同じ扱い）。
fn disable_legend_triggers(node: Node) -> Node {
    match node {
        Node::Element {
            tag,
            mut attrs,
            children,
        } => {
            if tag == "button"
                && attrs
                    .iter()
                    .any(|(k, v)| k == "data-part" && v == "trigger")
            {
                attrs.push(("disabled".to_string(), String::new()));
                attrs.push(("data-disabled".to_string(), String::new()));
                attrs.push(("aria-disabled".to_string(), "true".to_string()));
            }
            let children = children.into_iter().map(disable_legend_triggers).collect();
            Node::Element {
                tag,
                attrs,
                children,
            }
        }
        other => other,
    }
}

/// 指標切り替えボタン 1 個分（`<button>` の phrasing content 制約のため
/// `span` のみで組む、モジュール doc「`<button>` の内側に…」節参照）。
fn metric_button(
    label: &'static str,
    value: &str,
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
            disabled: true,
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

    // 選択中「Sessions」の表示値は直下の面グラフが描画する系列
    // （`SAMPLE_CHART_SERIES_A`）の合計から算出する（PR #3345 codex-review
    // P2 指摘の是正: 固定文言だとグラフのデータと食い違う）。
    let sessions_total = total(&data.series()[0]);
    let metrics = div(
        vec![("data-blocks-chart-metric-area-metrics", "")],
        vec![
            metric_button("Active users", "8,420", "+3.2%", true, false),
            metric_button(
                "Sessions",
                &format!("{sessions_total:.0}"),
                "+11.4%",
                true,
                true,
            ),
            metric_button("Conversion", "4.6%", "-0.3%", false, false),
        ],
    );

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
            // `size: Size::Lg` は plot の `--fandhe-area-chart-height`
            // トークンを 220px（viewBox の高さと一致）へ揃える。CSS 上書き
            // による特異性の上書き合戦はしない（モジュール doc「面グラフ
            // 高さは size: Size::Lg で揃え…」節、Cursor Bugbot Medium 指摘
            // の是正）。
            size: Size::Lg,
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
            stat::value_text(
                vec![],
                vec![
                    text(format!("{total_value:.0}")),
                    // `stat::help_text` の `<span>` は `<dl>` 直下では
                    // 定義リストとして不正（PR #3345 codex-review P2
                    // 指摘の是正）。`<dd>`（`value_text`）の内側へ移す
                    // （表示の縦積みは `LAYOUT_CSS` の
                    // `[data-blocks-chart-metric-area-total]
                    // [data-part="value-text"]` 上書きで維持する）。
                    stat::help_text(
                        vec![],
                        vec![
                            stat::up_indicator(vec![]),
                            text("+9.1% vs. previous period"),
                        ],
                    ),
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
            // switch 側と同じ理由（`size: Size::Lg` で揃える。モジュール
            // doc参照）。
            size: Size::Lg,
            ..AreaChartProps::new(
                &data,
                "Desktop and mobile sessions over the selected period",
            )
        },
        vec![("data-blocks-chart-metric-area-chart", "")],
    )
    .expect("chart-metric-area 固定データは常に有効な area_chart を構築できる");

    let legend_node = disable_legend_triggers(legend(&data, &LegendProps::default()));

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
```

## 集約元との差分メモ

- R0051（主参照）は指標切り替え型のレイアウトです。本 Demo の
  `variant="switch"` に対応します。3 指標のうち 1 件を選択中として固定し
  （無 JS のため切り替えません）、選択中の指標のみの面グラフを表示します。
- R0053 は見出しに合計値と系列別の内訳を並べる版です。本 Demo の
  `variant="breakdown"` に対応します。積み上げ面グラフに凡例を併設し、
  系列色と内訳の対応が読み取れるようにしています。
- 文言・配色・装飾・アイコンは参照元から持ち込まず、すべて独自に書いた
  架空の数値・ラベルです。

関連情報: [Card](../themes/card.md) / [Stat](../themes/stat.md) /
[Area Chart](../themes/area-chart.md) / [Button](../themes/button.md) /
[Charts](../themes/charts.md)
