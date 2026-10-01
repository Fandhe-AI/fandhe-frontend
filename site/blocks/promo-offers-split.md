# promo-offers-split

上段にオファー 3 件を横に並べた帯（各オファーは短い見出しと説明をまとめた
1 個のリンク）、下段に左テキスト・右画像の 2 列を置くブロックです。
`heading` / `text` / `button` / `image` / `link` / `separator` の 6 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1200 です。

オファー文言・リード文はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱プレースホルダー
SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Orientation;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// オファー帯 3 件分の（短い見出し, 説明）の組。架空の文言であり、実在の
/// キャンペーン条件を示すものではない。
const OFFERS: [(&str, &str); 3] = [
    ("送料無料", "全国どこでも配送料がかかりません"),
    ("30 日間返品可", "気に入らなければ全額返金いたします"),
    (
        "会員限定ポイント",
        "次回以降のお買い物に使えるポイントを進呈",
    ),
];

/// オファー帯の区切り線 1 組（横向き・縦向きを両方出力し、
/// `64rem` 境界で表示側を切り替える。モジュール冒頭「区切り線を横・縦の
/// 2 個で切り替える理由」節参照）。
fn offer_divider() -> Vec<Node> {
    vec![
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Horizontal,
                ..SeparatorProps::default()
            },
            vec![("data-blocks-promo-offers-split-divider", "horizontal")],
        ),
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Vertical,
                ..SeparatorProps::default()
            },
            vec![("data-blocks-promo-offers-split-divider", "vertical")],
        ),
    ]
}

/// オファー 1 件（短い見出し + 説明をまとめたリンク 1 個）。
fn offer(title: &'static str, description: &'static str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-promo-offers-split-offer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-offers-split-offer-title", "")],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-offers-split-offer-desc", "")],
                vec![text(description)],
            ),
        ],
    )
}

/// 上段のオファー帯（オファー 3 件 + 区切り線 2 組）。
fn offers_strip() -> Node {
    let mut children = Vec::new();
    for (index, (title, description)) in OFFERS.iter().enumerate() {
        if index > 0 {
            children.extend(offer_divider());
        }
        children.push(offer(title, description));
    }
    div(
        vec![("class", "blocks-promo-offers-split-offers")],
        children,
    )
}

/// 下段（左テキスト・右画像の 2 列）。
fn split() -> Node {
    section(
        vec![("class", "blocks-promo-offers-split-split")],
        vec![
            div(
                vec![("class", "blocks-promo-offers-split-content")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl3,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-promo-offers-split-title", "")],
                        vec![text("季節の特典をまとめてご案内")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Lg,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-promo-offers-split-lead", "")],
                        vec![text(
                            "対象商品のご購入で、上段 3 つの特典をすべて自動的に適用します。",
                        )],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-promo-offers-split-cta", "")],
                        vec![text("対象商品を見る")],
                    ),
                ],
            ),
            image::image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                vec![("data-blocks-promo-offers-split-image", "")],
            ),
        ],
    )
}

/// `promo-offers-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ契約）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-offers-split-layout")],
        vec![offers_strip(), split()],
    )
}
```

## 差分メモ

集約元は主参照 R1200 の 1 件のみです。

- **R1200（主参照・唯一の集約元）**: オファー帯 + 左テキスト・右画像の
  2 列構成をそのまま採用しています。集約元が 1 件のため並記すべき差分は
  ありません。
- **区切り線の向き切替**: `separator` は `role="separator"` と
  `aria-orientation` を常時出力し、装飾専用のオプションを持ちません。
  見た目だけを CSS で回転させると `aria-orientation` が表示と食い違うため、
  横向き・縦向きの区切り線をそれぞれ出力し `display: none` で表示/非表示を
  切り替えています（表示中の向きと aria は常に一致します）。
- **オファー見出しを `text` で表す理由**: `## Demo` 自体が h2・下段の本
  見出しが h3 の文書構造の中で、オファー 3 件へさらに見出し要素を使うと
  見出し構造が騒がしくなるため、`TextWeight::Semibold` の `text` で表して
  います。
- **状態表示**: 無 JS のため本 block はトグル系の状態（開閉・選択等）を
  持たず、状態違いの静的並記は行っていません。
- **lg 未満の縦積み**: `64rem` は `Breakpoint::Lg`
  （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`）と一致する
  リテラル値です。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Image](../themes/image.md) /
[Link](../themes/link.md) / [Separator](../themes/separator.md)
