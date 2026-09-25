# gallery-masonry

`badge` / `heading` / `text` / `image` の 4 部品だけで組み立てた、段組み配置のギャラリーです。新しい UI 部品は追加していません。

タグライン（badge）→ 見出し（heading）→ 説明（text）の下に、比率の異なる 9 枚の画像を CSS の段組み（`column-count`）へ流し込みます。段組みは上から下へ画像を詰めてから次の段へ移る素朴な masonry 風の見え方で、行揃えの最適化は行いません。

比率は 2 つの手段を併用して与えています。7 枚は image 部品の比率指定（正方形・横長・縦長・動画サムネイル比）で、残り 2 枚は比率指定を「本来の比率」のままにし、配置側の CSS で個別に上書きしています。

sm（640px）未満は 1 段、sm 以上で 2 段、lg（1024px）以上で 3 段になります。`<form>` は使わず、送信処理・データ取得を持たない静的な合成例です。画像はすべて同一のプレースホルダーで実データを持たず、装飾用途として `alt=""` にしています。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 段組みタイル 1 枚分の比率指定。`ratio_override` が `Some` のときは
/// `aspect` を [`AspectRatio::Auto`] にしたうえでラッパ属性値として使う
/// （モジュール doc「比率の与え方」節）。
struct Item {
    aspect: AspectRatio,
    ratio_override: Option<&'static str>,
}

const ITEMS: [Item; 9] = [
    Item {
        aspect: AspectRatio::Portrait,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Landscape,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Auto,
        ratio_override: Some("tall"),
    },
    Item {
        aspect: AspectRatio::Square,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Video,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Portrait,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Auto,
        ratio_override: Some("wide"),
    },
    Item {
        aspect: AspectRatio::Landscape,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Square,
        ratio_override: None,
    },
];

/// 見出しエリア（タグライン → 見出し → 説明）。
fn header() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("Gallery")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("比率違いの画像を段組みで流し込む")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "比率の異なる 9 枚の画像を CSS の段組みへ流し込みます。sm 未満は 1 段、sm 以上で 2 段、lg 以上で 3 段になります。",
        )],
    );
    div(
        vec![("class", "blocks-gallery-masonry-header")],
        vec![eyebrow, title, lead],
    )
}

/// 段組み 1 タイル分（画像 1 枚）。`ratio_override` があればラッパへ
/// `data-blocks-gallery-masonry-ratio` 属性を付与する。
fn item(entry: &Item) -> Node {
    let mut attrs = vec![("class", "blocks-gallery-masonry-item")];
    if let Some(ratio) = entry.ratio_override {
        attrs.push(("data-blocks-gallery-masonry-ratio", ratio));
    }
    div(
        attrs,
        vec![image::image(
            &ImageProps {
                aspect_ratio: entry.aspect,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-gallery-masonry-image", "")],
        )],
    )
}

/// `gallery-masonry` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-gallery-masonry-grid")],
        ITEMS.iter().map(item).collect(),
    );
    div(
        vec![("class", "blocks-gallery-masonry-stack")],
        vec![header(), grid],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0504。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 参照元は 9 枚それぞれに異なる比率（8 種類）を与えていますが、本
  block では image 部品の variant 4 種（正方形・横長・縦長・動画
  サムネイル比）と、配置側の直接指定 2 種（2:3・3:2）の計 6 種に
  絞りました。
- 参照元は `md` で 2 段・`lg` で 3 段でしたが、本 block はイシューの
  指定に合わせて `sm`（640px）で 2 段・`lg`（1024px）で 3 段にしました。
- `column-count` による段組みは、上から下へ画像を詰めてから次の段へ
  移る配置になります（グリッドのような行揃えにはなりません）。参照元の
  見た目に厳密には合わせず、この挙動をそのまま採用しています。
- 文言・配色は独自に書き起こしたもので、実在ブランドの文言・配色は
  持ち込んでいません。
- 画像は `dummy_assets` のプレースホルダーで `alt=""`（装飾扱い）に
  しました。
- `id`・`href`・`<form>` は出力しません。
