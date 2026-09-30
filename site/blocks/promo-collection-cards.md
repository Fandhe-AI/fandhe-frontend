# promo-collection-cards

背景画像に暗幕を重ねたヒーロー部へ補助行・見出し・リード文・primary/
secondary CTA を置き、その下端に重なるようコレクションカード 3 枚を横並び
で配置するブロックです。各カードは画像 + コレクション名 + 点数の補助行で
構成し、カード全体をリンク化します。`heading` / `text` / `button` /
`card` / `image` / `link-overlay` の 6 部品を合成します。Blocks は既存
部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1199 です。

カードは hover / キーボードフォーカスで枠と影が変わります。

コレクション名・点数・文言はすべて架空のデータであり、実在の企業・
ブランド・PII・実クレデンシャルは含みません。画像はビルド時生成の同梱
プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// コレクションカード 3 件分の名前（架空、実在の企業・ブランドとは無関係）。
const COLLECTIONS: [&str; 3] = ["新作コレクション", "定番コレクション", "限定コレクション"];

/// コレクションカード 3 件分の点数表示（架空、`COLLECTIONS` と対で使う）。
const COLLECTION_COUNTS: [&str; 3] = ["全 12 点", "全 24 点", "全 6 点"];

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
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-collection-cards-eyebrow", "")],
                vec![text("季節限定のラインアップ")],
            ),
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
            div(
                vec![("class", "blocks-promo-collection-cards-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-promo-collection-cards-cta", "")],
                        vec![text("すべて見る")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-promo-collection-cards-cta-secondary", "")],
                        vec![text("コレクション一覧")],
                    ),
                ],
            ),
        ],
    );
    section(
        vec![("class", "blocks-promo-collection-cards-hero")],
        vec![backdrop, content],
    )
}

/// コレクションカード 1 件（画像 + 名前 + 点数、カード全体をリンク化）。
fn collection_card(name: &'static str, count: &'static str) -> Node {
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
                    vec![
                        heading::heading(
                            HeadingLevel::H4,
                            &HeadingProps {
                                size: HeadingSize::Md,
                                ..HeadingProps::default()
                            },
                            vec![("data-blocks-promo-collection-cards-card-name", "")],
                            vec![text(name)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![("data-blocks-promo-collection-cards-card-count", "")],
                            vec![text(count)],
                        ),
                    ],
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
        .zip(COLLECTION_COUNTS)
        .map(|(name, count)| collection_card(name, count))
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

## 差分メモ

集約元は主参照 R1199 の 1 件のみです。

- **R1199（主参照・唯一の集約元）**: 背景画像ヒーロー + カード 3 枚の
  構成をそのまま採用しています。集約元が 1 件のため並記すべき差分は
  ありません。
- **暗幕の色**: `color-mix()` + `--fandhe-color-fg` トークンで作るため、
  ダークテーマでは前景/背景の意味が反転し「明るい幕 + 暗い文字」になる
  既知の挙動です（`hero-background-media` と同じ判断）。
- **secondary CTA の反転色**: Outline variant を `currentColor` に揃える
  ことで、暗幕の上でも背景色と衝突しない配色にしています。
- **状態表示**: 無 JS のため hover / focus-within は CSS のみで表現して
  います。本 block はトグル系の状態（開閉・選択等）を持たないため、
  状態違いの静的並記は行っていません。
- **sm 未満の縦積み**: `40rem` は `Breakpoint::Sm`
  （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`）と一致する
  リテラル値です。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Card](../themes/card.md) /
[Image](../themes/image.md) / [Link Overlay](../themes/link-overlay.md)
