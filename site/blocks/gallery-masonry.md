# gallery-masonry

`badge` / `heading` / `text` / `image` の合成例（既存部品のみで組んだ、比率
の異なる画像を段組みで並べるギャラリーです）。Blocks セクションは新規部品
を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください（主参照は対応表 ID R0504。出典の固有名・
ファイル名は記載しません）。

段組みは JavaScript を使わず CSS の `column-count`（multi-column）のみで
実現しており、幅に応じて 1〜3 段へ流し込まれます（`sm` 未満は 1 段、`sm`
以上は 2 段、`lg` 以上は 3 段）。9 枚の画像はそれぞれ異なる比率（正方形・
横長・縦長・動画サムネイル相当）を持ち、画像部品の比率指定だけでその違い
を表現しています。本 Demo は静的な表示例であり、`<form>` 要素を一切持ち
ません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const INTRO_CLASS: &str = "blocks-gallery-masonry-intro";
const GRID_CLASS: &str = "blocks-gallery-masonry-grid";
const ITEM_CLASS: &str = "blocks-gallery-masonry-item";

/// 循環的に割り当てる [`AspectRatio`] variant（9 枚分）。比率の違いを
/// 画像部品側の指定のみで表現する（モジュール doc「masonry 風段組みの
/// 実装方式」節）。9 枚とも同一プレースホルダー画像のため `alt` は持たず
/// `alt=""` で出力する（モジュール doc「画像素材」節）。
const ASPECT_RATIOS: [AspectRatio; 9] = [
    AspectRatio::Portrait,
    AspectRatio::Landscape,
    AspectRatio::Square,
    AspectRatio::Video,
    AspectRatio::Portrait,
    AspectRatio::Landscape,
    AspectRatio::Square,
    AspectRatio::Video,
    AspectRatio::Portrait,
];

pub fn demo() -> Node {
    let intro = div(
        vec![("class", INTRO_CLASS)],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("Gallery")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("最新の一枚")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text("幅に応じて 1〜3 段に流し込む段組みギャラリーです。")],
            ),
        ],
    );

    let items: Vec<Node> = ASPECT_RATIOS
        .iter()
        .map(|aspect_ratio| {
            div(
                vec![("class", ITEM_CLASS)],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: *aspect_ratio,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-gallery-masonry-image", "")],
                )],
            )
        })
        .collect();

    let grid = div(vec![("class", GRID_CLASS)], items);

    div(vec![], vec![intro, grid])
}
```
