# hero-image-top

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `button` /
`image` の 5 部品のみを合成した、画像を先頭に置くヒーローの合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください（主参照は
対応表 ID R0128 の 1 件で、集約元も同一のため差分はありません。出典の
固有名・ファイル名は記載しません）。

最上段に横長画像を全幅・16:9 で角丸表示し、その下を `md`（768px）以上で
左に見出し・右に本文 + CTA ボタン群という 2 カラム構成にします。狭い画面
幅では「画像 → 見出し → 本文 → CTA」の順に 1 列縦積みになります。
`<form>` は持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// `hero-image-top` の Demo 本体（横長画像 + 「見出し / 本文+CTA」の
/// 2 カラム）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let hero_image = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            shape: ImageShape::Rounded,
            ..ImageProps::new(
                dummy_assets::BACKGROUND_SRC,
                "架空のプロダクト画面を模したプレースホルダー画像",
            )
        },
        vec![("data-blocks-hero-image-top-image", "")],
    );

    let left = div(
        vec![("class", "blocks-hero-image-top-left")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("New release")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("Launch faster with a workspace built for teams")],
            ),
        ],
    );

    let actions = div(
        vec![("class", "blocks-hero-image-top-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    );

    let right = div(
        vec![("class", "blocks-hero-image-top-right")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "計画から公開までをひとつのワークスペースにまとめ、チーム全員が同じ状況を見ながら進められます。",
                )],
            ),
            actions,
        ],
    );

    let columns = div(
        vec![("class", "blocks-hero-image-top-columns")],
        vec![left, right],
    );

    div(
        vec![("class", "blocks-hero-image-top-layout")],
        vec![hero_image, columns],
    )
}
```

## 原案差分メモ

- 構造（先頭に横長画像、下に見出しと本文+CTA の 2 列）のみを採用し、文言・
  配色・素材はすべて独自に書いた架空のものにしました（実企業名・実在
  ブランド・PII は含みません）。
- 画像は実在の写真を用意できないため、ビルド時生成のモノトーンドット
  タイル（`dummy_assets::BACKGROUND_SRC`）をそのまま横長画像枠として
  使いました。16:9 は `AspectRatio::Video`、角丸はテーマ radius トークン
  （`ImageShape::Rounded`）で表しています。
- ブレークポイントは `md`（768px = 48rem）をリテラル直書きにしました
  （テーマの breakpoint トークンは `@media` 条件式の中では解決できない
  ため）。
- CTA ボタンは `button::button` 既定（`type="button"`）で遷移先を持たない
  静的表示にし、`<form>` は持ち込んでいません。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Image](../themes/image.md)
