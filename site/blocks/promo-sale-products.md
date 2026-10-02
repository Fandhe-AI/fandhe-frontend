# promo-sale-products

セール告知面（見出し・本文・primary/secondary CTA）と、割引商品 4 点の
2 列グリッドを組み合わせたブロックです。各商品カードは画像・商品名・
元値（取り消し線）・セール価格で構成します。`heading` / `text` /
`button` / `image` / `card` / `strong` の 6 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0039 です。

広い幅では告知面を左、商品グリッドを右に 2 カラムで配置し、`40rem`
未満のコンテナ幅では告知面を上、グリッドを下に縦積みします。

商品名・価格はすべて架空のデータであり、実在の企業・ブランド・PII・
実クレデンシャルは含みません。画像はビルド時生成の同梱プレースホルダー
SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。CTA は
`type="button"` の静的ボタンで、送信処理・状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, strong, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// 割引商品 4 点分の商品名（架空、実在のブランド・商品とは無関係）。
const PRODUCTS: [&str; 4] = [
    "リネンのトートバッグ",
    "陶器のマグカップ",
    "ウールのマフラー",
    "木製のコースター",
];

/// 割引商品 4 点分の元値（`PRODUCTS` と対で使う、架空の価格）。
const REGULAR_PRICES: [&str; 4] = ["¥4,800", "¥2,200", "¥6,000", "¥1,500"];

/// 割引商品 4 点分のセール価格（`PRODUCTS` と対で使う、架空の価格）。
const SALE_PRICES: [&str; 4] = ["¥3,360", "¥1,540", "¥4,200", "¥1,050"];

/// 告知面（見出し・本文・CTA 2 個）。
fn announcement() -> Node {
    div(
        vec![("class", "blocks-promo-sale-products-announcement")],
        vec![
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-sale-products-title", "")],
                vec![text("季節のセール、開催中")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-sale-products-lead", "")],
                vec![text(
                    "対象商品が最大 3 割引。数量限定のためお早めにご覧ください。",
                )],
            ),
            div(
                vec![("class", "blocks-promo-sale-products-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-promo-sale-products-cta", "")],
                        vec![text("セール商品を見る")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-promo-sale-products-cta-secondary", "")],
                        vec![text("カテゴリから探す")],
                    ),
                ],
            ),
        ],
    )
}

/// 割引商品カード 1 件（画像 + 商品名 + 価格行）。
fn product_card(name: &'static str, regular: &'static str, sale: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-sale-products-card", "")],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Square,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                    },
                    vec![("data-blocks-promo-sale-products-card-image", "")],
                )],
            ),
            card::body(
                vec![("class", "blocks-promo-sale-products-card-body")],
                vec![
                    card::title(
                        vec![("data-blocks-promo-sale-products-card-name", "")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-promo-sale-products-price-row")],
                        vec![
                            text("通常 "),
                            el("s", vec![], vec![text(regular)]),
                            strong(
                                vec![("data-blocks-promo-sale-products-card-sale-price", "")],
                                vec![text(sale)],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `promo-sale-products` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（告知面 + 商品グリッド 4 枚、モジュール冒頭「レイアウト」節参照）。
pub fn demo() -> Node {
    let cards: Vec<Node> = PRODUCTS
        .iter()
        .zip(REGULAR_PRICES)
        .zip(SALE_PRICES)
        .map(|((name, regular), sale)| product_card(name, regular, sale))
        .collect();
    div(
        vec![("class", "blocks-promo-sale-products-layout")],
        vec![div(
            vec![("class", "blocks-promo-sale-products-columns")],
            vec![
                announcement(),
                div(vec![("class", "blocks-promo-sale-products-grid")], cards),
            ],
        )],
    )
}
```

## 原案差分メモ

集約元は主参照 R0039 の 1 件のみです。

- **R0039（主参照・唯一の集約元）**: セール告知 + 割引商品グリッドの
  構成をそのまま採用しています。集約元が 1 件のため並記すべき差分は
  ありません
- **配色**: 参照元の文言・配色・装飾・アイコンは持ち込まず、
  `--fandhe-color-*`/`--fandhe-space-*` トークンのみで組み直しています
- **CTA**: ボタンはリンクではなく `type="button"` の静的表示です。送信・
  遷移処理は持ちません
- **縦積み**: `40rem` 未満のコンテナ幅で告知面を上、グリッドを下に
  縦積みします（`container-type: inline-size` によるコンテナクエリ）

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Image](../themes/image.md) /
[Card](../themes/card.md) / [Strong](../themes/strong.md)
