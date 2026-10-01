# order-confirmation-split-image

注文確認ページの合成例です。広い幅では左半分に大きな画像を置き、右列に
支払い完了の小見出し・見出し・説明文・追跡番号・商品行・集計・配送先と
支払い情報・「買い物を続ける」リンクを縦に並べます。`image` / `heading` /
`text` / `data-list` / `separator` / `link` の 6 部品を合成します。Blocks
は既存部品の合成例であり、新しい UI 部品は追加しません。主参照は R1121
です。

氏名・住所・追跡番号・価格はすべて架空のデータであり、実在の人物・
企業・PII・実クレデンシャルは含みません。カード番号は末尾 4 桁の伏字
表現のみで、実在パターンは使いません。画像はビルド時生成の同梱
プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。決済処理・
送信先は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
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

/// 商品行 1 件（サムネイル・商品名・オプション・価格）。
fn product_line(name: &'static str, option: &'static str, price: &'static str) -> Node {
    div(
        vec![("class", "blocks-order-confirmation-split-image-line")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                },
                vec![("data-blocks-order-confirmation-split-image-thumb", "")],
            ),
            div(
                vec![("class", "blocks-order-confirmation-split-image-line-text")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(option)],
                    ),
                ],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(price)]),
        ],
    )
}

/// `order-confirmation-split-image` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let media = div(
        vec![("class", "blocks-order-confirmation-split-image-media")],
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "梱包された注文商品のイメージ")
            },
            vec![("data-blocks-order-confirmation-split-image-media", "")],
        )],
    );

    let content = div(
        vec![("class", "blocks-order-confirmation-split-image-content")],
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
                    "商品の発送準備が整い次第、登録済みのメールアドレスへ発送通知をお送りします。",
                )],
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![row("追跡番号", "FD-2026-0000-1234")],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-confirmation-split-image-products")],
                vec![
                    product_line("リネンのトートバッグ", "カラー: サンド", "¥4,800"),
                    product_line("セラミックマグカップ", "数量: 2", "¥2,400"),
                    product_line("オーガニックコットンタオル", "サイズ: M", "¥1,600"),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![
                    row("小計", "¥8,800"),
                    row("送料", "¥500"),
                    row("消費税", "¥880"),
                    div(
                        vec![("data-blocks-order-confirmation-split-image-total", "")],
                        vec![row("合計", "¥10,180")],
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-order-confirmation-split-image-info", "")],
                vec![
                    row("配送先", "Haruto Fujimaki / 東京都渋谷区桜丘町1-2-3"),
                    row("支払い方法", "クレジットカード（末尾 0000）"),
                ],
            ),
            link::root(
                "../",
                &LinkProps::default(),
                vec![("data-blocks-order-confirmation-split-image-continue", "")],
                vec![text("買い物を続ける →")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-order-confirmation-split-image-root")],
        vec![div(
            vec![("class", "blocks-order-confirmation-split-image-layout")],
            vec![media, content],
        )],
    )
}
```

## 原案差分メモ

主参照 R1121（左半分画像 + 右列に注文内容）からの差分です。

- 参照元の文言・配色・アイコンは持ち込まず、独自の文言にしました
- 狭い幅（コンテナ幅 48rem 未満）では画像を上部の帯にします。コンテナ
  クエリ（`@container`）で判定します
- 配送先と支払い情報は、幅にかかわらず `data-list` を 2 列 grid にします。
  これは block 固有の CSS が付与しています
- 集計の合計行の強調も block 固有の CSS によるものです
- 節の区切りには `separator` を使っています

## 関連情報

- [Image](../themes/image.md)
- [Heading](../themes/heading.md)
- [Text](../themes/text.md)
- [Data List](../themes/data-list.md)
- [Separator](../themes/separator.md)
- [Link](../themes/link.md)
