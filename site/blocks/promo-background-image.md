# promo-background-image

背景画像全面のプロモーション。`heading` / `text` / `link` / `image` /
`card` の 5 部品を合成し、背景画像に暗幕を重ねた上へ中央寄せの見出し・
説明・反転色 CTA（サイト内に実在するコレクション訴求ページへ遷移する
`link`）を置くブロックです。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

主参照は対応表 ID R1196（基準形）です。角丸カードへ収める形（R1198）と
トップページ向けの大見出し・大きめの余白形（R1202）を並記形として
集約しています。

背景画像とスクリムは `aria-hidden` の装飾扱いです。

文言はすべて架空のデータであり、実在の企業・ブランド・PII・実クレデン
シャル・価格の断定は含みません。画像はビルド時生成の同梱プレースホルダー
SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// CTA のリンク先（モジュール冒頭「`id` を使わない・`<form>` を使わない・
/// 実データを持たない」節参照）。サイト内に実在するコレクション訴求
/// ページ（`promo-collection-cards` block のページ）への相対リンクで、
/// `store_nav_mega_menu` の「セール」導線と同じ形。ビルド時に
/// `linkcheck::check_links` が実在を検証する。
const COLLECTION_HREF: &str = "../../blocks/promo-collection-cards/";

/// 各形の直前に置く短い形ラベル（`hero_background_media::variant_label`
/// と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 背景画像 + 暗幕の 2 層（`aria-hidden` で装飾扱い、3 形で共通）。
fn backdrop() -> Node {
    div(
        vec![
            ("class", "blocks-promo-background-image-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![("data-blocks-promo-background-image-image", "")],
            ),
            div(
                vec![("class", "blocks-promo-background-image-scrim")],
                vec![],
            ),
        ],
    )
}

/// 中央寄せの見出し・説明・反転色 CTA（`heading_size` のみ形ごとに変える）。
fn content(heading_size: HeadingSize) -> Node {
    div(
        vec![("class", "blocks-promo-background-image-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: heading_size,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-background-image-title", "")],
                vec![text("新しい季節のコレクション")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-background-image-lead", "")],
                vec![text(
                    "定番から限定まで、今季のコレクションをまとめてご紹介します。",
                )],
            ),
            link::root(
                COLLECTION_HREF,
                &LinkProps::default(),
                vec![("data-blocks-promo-background-image-cta", "")],
                vec![text("コレクションを見る")],
            ),
        ],
    )
}

/// 形 A（R1196 基準形）: 全面背景画像 + 暗幕 + 中央寄せコンテンツ。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-basic")],
        vec![backdrop(), content(HeadingSize::Xl3)],
    )
}

/// 形 B（R1198）: 角丸カードの内側全面へ背景画像・暗幕・コンテンツを敷く。
fn variant_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-background-image-card", "")],
        vec![backdrop(), content(HeadingSize::Xl3)],
    )
}

/// 形 C（R1202）: トップページ向け。h1 相当の大見出し・大きめの余白
/// （Demo では `HeadingLevel::H3` のまま、サイズと余白だけを再現する。
/// モジュール冒頭「3 形を 1 つの Demo に並記する」節参照）。
fn variant_top_page() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-top-page")],
        vec![backdrop(), content(HeadingSize::Xl4)],
    )
}

/// `promo-background-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-layout")],
        vec![
            variant_label("基準形（R1196）"),
            variant_basic(),
            variant_label("カード形（R1198）"),
            variant_card(),
            variant_label("トップページ形（R1202、大見出し・大きめの余白）"),
            variant_top_page(),
        ],
    )
}
```

## 原案差分メモ

- 主参照は全面背景画像 + 暗幕 + 中央寄せ CTA の基準形（R1196）。角丸カード
  へ収める形（R1198）はカードの内側全面へ同じ背景・コンテンツを敷く形へ
  並記し、トップページ向けの大見出し・大余白形（R1202）は見出しサイズと
  上下余白だけを変える形へ並記した。
- R1202 はトップページでは `HeadingLevel::H1` を想定するが、docs ページ
  本文の `<h1>` は 1 個に保つ契約のため、Demo では他 2 形と同じ
  `HeadingLevel::H3` のまま見た目のサイズ（`HeadingSize::Xl4`）と余白
  （`--fandhe-space-24`）だけを再現した。
- 暗幕は色リテラルではなく `--fandhe-color-fg` トークンを `color-mix()`
  で半透明化して作る。ライトテーマでは暗い幕の上に明るい文字が乗るが、
  ダークテーマでは前景/背景の意味が反転するため、明るい幕の上に暗い
  文字が乗る形へ反転する（意図した挙動、`hero-background-media` と同じ
  判断）。
- 背景画像とスクリムは `aria-hidden` の装飾扱いとし、画像の `alt` は
  空にした。
- 参照元の文言・配色・装飾は持ち込まず、見出し・説明・CTA の文言は
  すべて独自の架空文言に差し替えた。
- CTA は「見る」という遷移を示す文言のため `button::button`（遷移先を
  持たない `<button>`）ではなく `link::root`（`<a>`）で描画する。遷移先
  はサイト内に実在するコレクション訴求ページ（`promo-collection-cards`
  block のページへの相対リンク）とし、見出し・説明・CTA の文言をすべて
  コレクション紹介に揃えることで、告知内容と遷移先の不一致を避けた
  （`store_nav_mega_menu` の「セール」導線と同型の判断）。内部リンクの
  ため `external`/`rel="noopener noreferrer"`/`target="_blank"` は付与
  しない。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Image](../themes/image.md) /
[Card](../themes/card.md)
