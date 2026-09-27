# chart-bar-list

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `charts`（共通 API・
`ChartData`/`SortDirection`）/ `bar-list` 部品を合成した、ランキング型の
横棒リストをカード 2 枚に並べた実例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R0052。出典の固有名・
ファイル名は記載しません）。

各カードは見出し・説明文の下にランキング形式の横棒リストを持ち、棒の
長さが最大値に対する割合を、右端の数値がその項目の値を表します。左の
カードは「流入元（今月）」、右のカードは「よく読まれたページ」という
架空のデモデータです。ランキング順は [`ChartData::sort_by_series`] を
降順で適用してから `bar_list::root` へ渡しており、部品自体は並び順を
変更しません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::charts::bar_list;
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series, SortDirection};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};

/// ランキングカード 1 枚を組み立てる小さな helper（内部専用）。`rows` は
/// `(ラベル, 値)` の組で、表示順は未ソートのままでよい（本関数の内部で
/// 降順ソートを適用する）。
fn ranking_card(title: &str, description: &str, series_name: &str, rows: &[(&str, f64)]) -> Node {
    let categories = rows.iter().map(|(label, _)| (*label).to_string()).collect();
    let values = rows.iter().map(|(_, value)| *value).collect();
    let data = ChartData::new(categories, vec![Series::new(series_name, values)])
        .expect("chart-bar-list の固定データは常に有効な ChartData を構成する")
        .sort_by_series(series_name, SortDirection::Descending)
        .expect("series_name は直前に構築した ChartData 自身の系列名と一致する");
    let list = bar_list::root(&data, series_name)
        .expect("chart-bar-list の固定データに未知系列・負値は含まれない");

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
                        vec![fandhe_frontend_core::text(title)],
                    ),
                    card::description(vec![], vec![fandhe_frontend_core::text(description)]),
                ],
            ),
            card::body(vec![], vec![list]),
        ],
    )
}

/// `chart-bar-list` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let inflow = ranking_card(
        "流入元（今月）",
        "訪問数（件）",
        "visits",
        &[
            ("直接アクセス", 1180.0),
            ("検索", 3420.0),
            ("紹介リンク", 640.0),
            ("SNS", 2260.0),
            ("ニュースレター", 980.0),
        ],
    );
    let popular_pages = ranking_card(
        "よく読まれたページ",
        "閲覧数（件）",
        "views",
        &[
            ("更新履歴", 510.0),
            ("はじめに", 4380.0),
            ("お問い合わせ", 260.0),
            ("料金", 1970.0),
            ("導入事例", 1340.0),
        ],
    );
    div(
        vec![("class", "blocks-chart-bar-list-layout")],
        vec![inflow, popular_pages],
    )
}
```

## 原案差分メモ

- 集約元は主参照（対応表 ID R0052）の 1 件のみです。代表構成そのものを
  Demo にしており、他 block のように複数版を差分として並べる対象は
  ありません。
- 系列色は既定の `--fandhe-color-chart-1` を使用しています。参照元の
  配色・アイコン・文言は持ち込まず、デモデータ・説明文はすべて独自に
  書いた架空のものです。
- 見出しは `card::title` ではなく `heading(HeadingLevel::H3, ...)` を直接
  合成しています。使用部品として `heading` を実際に組み込むためで、
  見出しの階層はページ本体の `h1` → Demo の `h2` → 本 block のカード
  見出し `h3` の順になります。
- 広い幅では 2 列、狭い幅（コンテナ幅 20rem 未満相当）では 1 列に積む
  レイアウトです。値の右寄せは `bar-list` 部品自身の 2 列グリッド構成で
  実現されており、block 固有の追加 CSS は持ちません。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Charts](../themes/charts.md) / [Bar List](../themes/bar-list.md)
