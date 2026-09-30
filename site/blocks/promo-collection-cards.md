# promo-collection-cards

背景画像に暗幕を重ねたヒーロー部へ見出し・リード文・CTA を置き、その下端に
重なるようコレクションカード 3 枚を横並びで配置するブロックです。各カード
は画像 + コレクション名で構成し、カード全体をリンク化します。`heading` /
`text` / `button` / `card` / `image` / `link-overlay` の 6 部品を合成しま
す。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1199 です。

コレクション名・文言はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱プレース
ホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// コレクションカード 3 件分の名前（架空、実在の企業・ブランドとは無関係）。
const COLLECTIONS: [&str; 3] = ["新作コレクション", "定番コレクション", "限定コレクション"];

/// ヒーロー部（背景画像 + 暗幕 + 見出し・リード文・CTA）。
fn hero() -> Node {
    let backdrop = div(
        vec![
            ("class", "blocks-promo-collection-cards-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![("data-blocks-promo-collection-cards-image", "")],
            ),
            div(
                vec![("class", "blocks-promo-collection-cards-scrim")],
                vec![],
            ),
        ],
    );
    let content = div(
        vec![("class", "blocks-promo-collection-cards-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-collection-cards-title", "")],
                vec![text("新作コレクション、到着")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-collection-cards-lead", "")],
                vec![text(
                    "季節ごとに入れ替わる 3 つのコレクションから選べます。",
                )],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-promo-collection-cards-cta", "")],
                vec![text("すべて見る")],
            ),
        ],
    );
    section(
        vec![("class", "blocks-promo-collection-cards-hero")],
        vec![backdrop, content],
    )
}

/// コレクションカード 1 件（画像 + 名前、カード全体をリンク化）。
fn collection_card(name: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-collection-cards-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-promo-collection-cards-link", "")],
            vec![
                image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-promo-collection-cards-card-image", "")],
                ),
                card::body(
                    vec![("class", "blocks-promo-collection-cards-card-body")],
                    vec![heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-promo-collection-cards-card-name", "")],
                        vec![text(name)],
                    )],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", name),
                        ("data-blocks-promo-collection-cards-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// `promo-collection-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール冒頭「コレクションカードをヒーロー下端へ重ねる」
/// 節参照）。
pub fn demo() -> Node {
    let cards: Vec<Node> = COLLECTIONS
        .iter()
        .map(|name| collection_card(name))
        .collect();
    div(
        vec![("class", "blocks-promo-collection-cards-layout")],
        vec![
            hero(),
            div(vec![("class", "blocks-promo-collection-cards-grid")], cards),
        ],
    )
}
```

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Card](../themes/card.md) /
[Image](../themes/image.md) / [Link Overlay](../themes/link-overlay.md)
