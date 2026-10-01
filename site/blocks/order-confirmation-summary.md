# order-confirmation-summary

注文確認ページのお礼見出し・追跡番号、商品明細 2 件、配送先・請求先・
支払い方法・配送方法の 4 情報、割引バッジ付きの集計を組み合わせた注文
確認ブロックです。`heading` / `text` / `image` / `separator` /
`data-list` / `badge` の 6 部品を合成します。Blocks は既存部品の合成例
であり、新しい UI 部品は追加しません。

商品行は数量と価格の間に縦の区切り線を挟み、配送先・請求先・支払い
方法・配送方法の 4 情報は幅の広いコンテナでは 2 列、狭いコンテナでは
1 列に積みます。集計の割引行はバッジで割引コード（`AUTUMN10`）を示し、
小計・合計と通貨・金額が一致します（商品 A ¥12,800 × 1 + 商品 B
¥2,400 × 2 = 小計 ¥17,600 → 割引 10% −¥1,760 → 合計 ¥15,840）。

宛名・住所・メール・追跡番号・カード番号はすべて架空のデータであり、
実在の人物・企業・PII・実クレデンシャルは含みません。カード番号は
末尾 4 桁の伏字表現のみで、実在パターンは使いません。商品画像は
ビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。注文処理・
決済・送信先は一切持ちません。ボタン・リンクも置いていません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 割引コードをバッジで示す集計行（`row` と異なり label 側へ text + badge の
/// 2 ノードを並べる、[`demo`] の集計節専用ヘルパ）。
fn discount_row(code: &'static str, amount: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(
                vec![],
                vec![
                    text("割引"),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(code)],
                    ),
                ],
            ),
            data_list::item_value(vec![], vec![text(amount)]),
        ],
    )
}

/// 商品行 1 件（画像・名称・説明 + 数量・縦区切り・価格の `-meta` 行）。
/// 数量と価格の間の縦区切りは R0587（大見出し + 単品 + 縦区切り）由来
/// （モジュール冒頭「構成」節参照）。
fn item_row(
    name: &'static str,
    description: &'static str,
    quantity: &'static str,
    price: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-item")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                },
                vec![("data-blocks-order-confirmation-summary-image", "")],
            ),
            div(
                vec![("class", "blocks-order-confirmation-summary-item-info")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description)],
                    ),
                    div(
                        vec![("class", "blocks-order-confirmation-summary-meta")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("数量 {quantity}"))],
                            ),
                            separator(
                                &SeparatorProps {
                                    orientation: Orientation::Vertical,
                                    ..SeparatorProps::default()
                                },
                                vec![],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(price)],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 見出し + 定義リストの 1 セクション（配送先・請求先・支払い方法・
/// 配送方法。R1124 主参照の 2 列グリッド、[`LAYOUT_CSS`] の `@container` が
/// 狭幅で 1 列化する）。
fn info_section(title: &'static str, rows: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-info-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![],
                rows,
            ),
        ],
    )
}

/// `order-confirmation-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-confirmation-summary-layout")],
        vec![
            div(
                vec![("class", "blocks-order-confirmation-summary-intro")],
                vec![
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("お支払いが完了しました")],
                    ),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ご注文ありがとうございます")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            "発送が完了次第、登録のメールアドレス宛にご連絡いたします。",
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("追跡番号: FD-7Q2K-0915")],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-items")],
                vec![
                    item_row(
                        dummy_assets::COMPANY_NAMES[0],
                        "リネン素材のトートバッグ。マチ広で普段使いしやすいサイズ感。",
                        "1",
                        "¥12,800",
                    ),
                    item_row(
                        dummy_assets::COMPANY_NAMES[1],
                        "陶器のマグカップ。電子レンジ・食洗機対応。",
                        "2",
                        "¥2,400",
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-info")],
                vec![
                    info_section(
                        "配送先",
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("住所", "東京都渋谷区 1-2-3"),
                        ],
                    ),
                    info_section(
                        "請求先",
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("メール", "haruto.fujimaki@example.com"),
                        ],
                    ),
                    info_section(
                        "支払い方法",
                        vec![
                            row("支払方法", "クレジットカード"),
                            row("カード番号", "**** **** **** 4242"),
                        ],
                    ),
                    info_section(
                        "配送方法",
                        vec![
                            row("配送方法", "通常配送"),
                            row("お届け予定", "9/27 到着予定"),
                        ],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-summary-totals")],
                vec![data_list::root(
                    DataListProps {
                        orientation: DataListOrientation::Horizontal,
                        ..DataListProps::default()
                    },
                    vec![],
                    vec![
                        row("小計", "¥17,600"),
                        discount_row("AUTUMN10", "−¥1,760"),
                        row("合計", "¥15,840"),
                    ],
                )],
            ),
        ],
    )
}
```

## 差分メモ

集約元 3 件（主参照 R1124・集約元 R0587・R0586）に対する本 block の
扱いです。

- **R1124（主参照、単品 + 住所・支払い・集計）**: 商品明細・配送先/
  請求先/支払い方法/配送方法の 4 情報・集計の構成をそのまま採用して
  います。
- **R0587（大見出し + 単品 + 縦区切り）**: 縦区切りのみ商品行の数量と
  価格の間に取り込み、大見出し自体は h3 の強調（「ご注文ありがとう
  ございます」）で代替しています。
- **R0586（大きな商品画像の横並び + 右寄せ集計）**: 右寄せ集計
  （`margin-inline-start: auto` + `max-width`）のみ取り込んでいます。
  大きな商品画像の横並び表現は、1 block 内に画像サイズの異なる表現が
  混在すると差分の主眼が読み取れなくなるため、Demo に別枠で並べて
  いません。
- **割引コードの表現**: `badge` で割引コード（`AUTUMN10`）を示し、金額
  は定義リストの値側に表示しています。
- **狭い幅での 1 列化**: コンテナ幅 40rem 未満で配送先・請求先・支払い
  方法・配送方法の 2 列グリッドを 1 列化し、商品画像も縦積みに切り替え
  ます。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Separator](../themes/separator.md) /
[Data List](../themes/data-list.md) / [Badge](../themes/badge.md)
