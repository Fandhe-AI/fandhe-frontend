# order-history-table

`table` / `data-list` / `link` / `image` / `text` / `visually-hidden` の
6 部品を合成した、注文履歴の合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

注文ごとにサマリ帯（注文番号・注文日・合計金額・請求書リンク）を置き、
その下へ商品・価格・状態・操作の 4 列を持つ商品明細表を積みます。注文
2 件分を縦に並べています。狭い幅（コンテナ幅 40rem 未満）では価格・状態
の 2 列を隠し、価格と状態の両方を商品セル内へ表示します。docs サイトは
JS ハイドレーションを行わないため、実際にビューポートを変えてリサイズ
する実演はできません。代わりに同一構造を持つ 2 インスタンス（広幅 /
`max-inline-size: 24rem` で強制的に狭幅化）を並記し、折り畳みの違いを
静的に見せます。

価格・状態はそれぞれセル本体と商品セル内の狭幅用テキストの 2 か所に
出力しますが、コンテナクエリの条件で常にどちらか一方だけが
`display: none` になるため、スクリーンリーダーによる二重読み上げは
起きません。「操作」列の見出しは視覚上は空で、スクリーンリーダー向け
にのみ列名を供給します（`visually-hidden` の使用箇所の 1 つ。他に各
テーブルの `caption`、請求書・商品リンクの補足テキスト、商品セル内の
狭幅用価格・状態テキストへの接頭辞でも使っています）。

静的な表示例であり、`<form>` 要素を持ちません。商品名・価格・注文番号・
日付はすべて架空の値で、実在の商品・企業・PII は含みません。商品画像は
ビルド時生成の同梱プレースホルダー SVG です。各注文の合計金額は商品
価格の和と一致します（注文 1: 4,800 + 3,200 + 1,600 = 9,600円、注文 2:
6,400 + 2,000 = 8,400円）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// ラベル・値の 1 項目（サマリ帯の `data-list` 項目）。
fn summary_item(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 商品 1 行分（商品名・価格・状態・商品ページへのリンク先 slug）。
fn item_row(
    name: &'static str,
    price: &'static str,
    status: &'static str,
    slug: &'static str,
) -> Node {
    let product_href = format!("https://example.com/products/{slug}");
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-order-history-table-product")],
                        vec![
                            image(
                                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                                vec![("data-blocks-order-history-table-product", "")],
                            ),
                            styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                        ],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-order-history-table-inline-price", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("価格 ")]),
                            text(price),
                        ],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-order-history-table-inline-status", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("状態 ")]),
                            text(status),
                        ],
                    ),
                ],
            ),
            table::cell(
                vec![("data-blocks-order-history-table-secondary", "")],
                vec![text(price)],
            ),
            table::cell(
                vec![("data-blocks-order-history-table-secondary", "")],
                vec![text(status)],
            ),
            table::cell(
                vec![],
                vec![link::root(
                    &product_href,
                    &LinkProps::default(),
                    vec![],
                    vec![
                        text("商品を見る"),
                        visually_hidden::root(vec![], vec![text(format!("（{name}）"))]),
                    ],
                )],
            ),
        ],
    )
}

/// 注文 1 件（サマリ帯 + 商品明細表）を組み立てる。`items` は
/// `(商品名, 価格, 状態, 商品ページ slug)` の組。
fn order_block(
    order_number: &'static str,
    order_date: &'static str,
    total: &'static str,
    items: &[(&'static str, &'static str, &'static str, &'static str)],
) -> Node {
    let invoice_href = format!("https://example.com/invoices/{order_number}");
    let caption_label = format!("注文 #{order_number} の商品明細");
    div(
        vec![("class", "blocks-order-history-table-order")],
        vec![
            div(
                vec![("class", "blocks-order-history-table-summary")],
                vec![
                    data_list::root(
                        DataListProps {
                            orientation: DataListOrientation::Horizontal,
                            size: Size::Sm,
                            ..DataListProps::default()
                        },
                        vec![("data-blocks-order-history-table-summary-list", "")],
                        vec![
                            summary_item("注文番号", order_number),
                            summary_item("注文日", order_date),
                            summary_item("合計金額", total),
                        ],
                    ),
                    link::root(
                        &invoice_href,
                        &LinkProps::default(),
                        vec![],
                        vec![
                            text("請求書を表示"),
                            visually_hidden::root(
                                vec![],
                                vec![text(format!("（注文 #{order_number}）"))],
                            ),
                        ],
                    ),
                ],
            ),
            table::root(
                TableProps::default(),
                vec![("data-blocks-order-history-table-table", "")],
                vec![
                    table::caption(
                        vec![],
                        vec![visually_hidden::root(vec![], vec![text(caption_label)])],
                    ),
                    table::header(
                        vec![],
                        vec![table::row(
                            vec![],
                            vec![
                                table::column_header(vec![], vec![text("商品")]),
                                table::column_header(
                                    vec![("data-blocks-order-history-table-secondary", "")],
                                    vec![text("価格")],
                                ),
                                table::column_header(
                                    vec![("data-blocks-order-history-table-secondary", "")],
                                    vec![text("状態")],
                                ),
                                table::column_header(
                                    vec![],
                                    vec![visually_hidden::root(vec![], vec![text("操作")])],
                                ),
                            ],
                        )],
                    ),
                    table::body(
                        vec![],
                        items
                            .iter()
                            .map(|(name, price, status, slug)| item_row(name, price, status, slug))
                            .collect(),
                    ),
                ],
            ),
        ],
    )
}

/// 注文 2 件分のダミーデータ（状態 A/B で共有する）。注文 1 は全行
/// 「配達済み」、注文 2 は「配送中」と「発送準備中」を混ぜる
/// （モジュール doc「ダミー素材について」節参照）。
fn orders() -> Vec<Node> {
    vec![
        order_block(
            "FD-2026-0912",
            "2026年09月12日",
            "¥9,600",
            &[
                (
                    "折りたたみデスクライト",
                    "¥4,800",
                    "配達済み（9月15日）",
                    "desk-lamp",
                ),
                (
                    "コットンブランケット",
                    "¥3,200",
                    "配達済み（9月15日）",
                    "cotton-blanket",
                ),
                (
                    "セラミックマグ",
                    "¥1,600",
                    "配達済み（9月15日）",
                    "ceramic-mug",
                ),
            ],
        ),
        order_block(
            "FD-2026-0827",
            "2026年08月27日",
            "¥8,400",
            &[
                ("ノートブックカバー", "¥6,400", "配送中", "notebook-cover"),
                (
                    "ワイヤレス充電パッド",
                    "¥2,000",
                    "発送準備中",
                    "wireless-charger",
                ),
            ],
        ),
    ]
}

/// `order-history-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-history-table-layout")],
        vec![
            div(
                vec![("class", "blocks-order-history-table-frame")],
                orders(),
            ),
            div(
                vec![(
                    "class",
                    "blocks-order-history-table-frame blocks-order-history-table-frame--narrow",
                )],
                orders(),
            ),
        ],
    )
}
```

## 原案差分メモ

- 参照 ID は R1117（注文ごとのサマリ帯+商品表、代表構成）のみです。集約元
  が 1 件のため、差分のすり合わせは発生していません。
- 広幅・狭幅の並記は、JS ハイドレーションを行わない docs サイトの制約
  下でリサイズ実演を代替する仕組みです。`.blocks-order-history-table-
  frame--narrow` の `max-inline-size: 24rem` で強制的に狭幅化し、
  `@container` の判定閾値（`40rem`）を下回らせています。
- 価格の値をセル本体と商品セル内のテキストの 2 か所へ出力する設計は、
  `display: none` がアクセシビリティツリーからも要素を除外する性質を
  利用した、読み上げ重複回避のための構成です（`table-responsive-stacked`
  と同じ判断軸）。狭幅で状態列が消えるのは仕様どおりで、状態は商品セル
  内には複製しません（価格のみ複製する設計、Issue のレイアウト仕様に
  従う）。
- ブラウザでの実機確認（コンテナ幅切替・ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。

関連情報: [Table](../themes/table.md) / [Data List](../themes/data-list.md) /
[Link](../themes/link.md) / [Image](../themes/image.md) /
[Text](../themes/text.md) / [Visually Hidden](../themes/visually-hidden.md)
