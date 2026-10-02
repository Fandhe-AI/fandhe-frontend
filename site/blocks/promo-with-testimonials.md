# promo-with-testimonials

背景画像の上に淡い覆いを重ね、上段へ見出し・説明・CTA を、下段へ顧客の
引用 3 件を 3 列で並べるブロックです。`heading` / `text` / `button` /
`image` / `blockquote` / `icon` の 6 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1195 です。

キャンペーン文言・引用・人名・役職はすべて架空のデータであり、実在の
企業・ブランド・PII・実クレデンシャルは含みません。画像はビルド時生成の
同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

const ROOT_CLASS: &str = "blocks-promo-with-testimonials-layout";
const BACKDROP_CLASS: &str = "blocks-promo-with-testimonials-backdrop";
const VEIL_CLASS: &str = "blocks-promo-with-testimonials-veil";
const PROMO_CLASS: &str = "blocks-promo-with-testimonials-promo";
const QUOTES_CLASS: &str = "blocks-promo-with-testimonials-quotes";

const IMAGE_ATTR: &str = "data-blocks-promo-with-testimonials-image";
const CTA_ATTR: &str = "data-blocks-promo-with-testimonials-cta";
const QUOTE_ATTR: &str = "data-blocks-promo-with-testimonials-quote";
const NAME_ATTR: &str = "data-blocks-promo-with-testimonials-name";
const ROLE_ATTR: &str = "data-blocks-promo-with-testimonials-role";

/// 引用カードの先頭に置く装飾アイコン（抽象的な引用符の図形。実在ブランド
/// の図形は使わない）。`label: None`（既定）のため `aria-hidden="true"` が
/// 付く装飾アイコンになる。
fn quote_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![("d", "M7 7h4v6l-3 5H5l2-5H7V7Zm8 0h4v6l-3 5h-3l2-5h-0V7Z")],
            vec![],
        )],
    )
}

/// 引用カード 1 件（装飾アイコン + 引用文 + 発言者名・役職）。
fn testimonial(quote: &'static str, name: &'static str, role: &'static str) -> Node {
    blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![(QUOTE_ATTR, "")],
        vec![
            quote_icon(),
            blockquote::content(vec![], vec![text(quote)]),
            blockquote::caption(
                vec![],
                vec![div(
                    vec![],
                    vec![
                        styled_text::text(
                            &TextProps {
                                weight: fandhe_frontend_pre_styled_ui::text::TextWeight::Semibold,
                                ..TextProps::default()
                            },
                            vec![(NAME_ATTR, "")],
                            vec![text(name)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![(ROLE_ATTR, "")],
                            vec![text(role)],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 上段（見出し・説明・CTA）。
fn promo() -> Node {
    div(
        vec![("class", PROMO_CLASS)],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("秋の感謝キャンペーン開催中")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "対象商品が期間限定で特別価格に。お客様の声とともにご案内します。",
                )],
            ),
            button::button(
                &ButtonProps::default(),
                vec![(CTA_ATTR, "")],
                vec![text("キャンペーンを見る")],
            ),
        ],
    )
}

/// `promo-with-testimonials` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ契約）。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", VEIL_CLASS)], vec![]),
        ],
    );

    let quotes = div(
        vec![("class", QUOTES_CLASS)],
        (0..3)
            .map(|index| {
                testimonial(
                    dummy_assets::TESTIMONIAL_QUOTES[index],
                    dummy_assets::PERSON_NAMES[index],
                    dummy_assets::JOB_TITLES[index],
                )
            })
            .collect(),
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, promo(), quotes])
}
```

## 差分メモ

集約元は主参照 R1195 の 1 件のみです。

- **R1195（主参照・唯一の集約元）**: 背景画像の上に見出し・説明・CTA を
  置き、下段に顧客の引用を並べる構成をそのまま採用しています。集約元が
  1 件のため並記すべき差分はありません。
- **覆いを bg の color-mix で作る判断**: `--fandhe-color-bg` を 85% 混ぜた
  淡い面にすることで、文字色を通常の `--fandhe-color-fg`（反転なし）の
  まま保っています。全面を暗く反転させる
  `testimonial-background-image` とは逆の設計判断です。
- **状態表示**: 無 JS のため本 block はトグル系の状態（開閉・選択等）を
  持たず、状態違いの静的並記は行っていません。
- **md 未満の 1 列**: `48rem` は `Breakpoint::Md`
  （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`）と一致する
  リテラル値です。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Image](../themes/image.md) /
[Blockquote](../themes/blockquote.md) / [Icon](../themes/icon.md)
