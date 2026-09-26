# content-image-tiles

`heading` / `text` / `image` / `stat` の 4 部品を合成した、見出し + 本文/
画像タイルの 2 列 + 下段の数値指標を積む構成です。

右列は正方形の画像タイルを 2 列 × 2 段（計 4 枚）で並べ、偶数番目（2・4
枚目）だけ下へずらします。タイルの角丸はテーマトークン
（`--fandhe-radius-md`）で揃えています。幅 lg（64rem）以上では本文列と
画像タイル列の 2 列、それより狭い幅では 1 列に切り替わり、タイルは本文の
後に続きます。文言・数値指標はすべて架空のもので、データ取得・送信は
行わない静的な表示例です。`<form>` は使用しません。

集約元は 1 件のみ（対応表 ID R0869）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 本文の段落群（架空文言、1〜2 文程度に短くして検索インデックスの
/// サイズを抑える）。
const PARAGRAPHS: [&str; 2] = [
    "画像タイルは既存の image パーツを並べるだけで組み立てており、独自の画像コンポーネントは追加していません。",
    "下段の数値指標は stat パーツをそのまま並べた合成であり、送信処理やデータ取得は行いません。",
];

/// 下段の数値指標（架空値、`(ラベル, 値)` の組）。
const STATS: [(&str, &str); 4] = [
    ("導入チーム", "1,200+"),
    ("平均応答", "120ms"),
    ("稼働継続", "99.9%"),
    ("対応言語", "18"),
];

/// タイル 1 枚（正方形・角丸の商品プレースホルダー画像）。偶数番目
/// （2・4 枚目）は `data-blocks-content-image-tiles-offset` を付与して
/// 下へずらす（モジュール doc「タイルの配置」節）。
fn tile(offset: bool) -> Node {
    let mut attrs = vec![("class", "blocks-content-image-tiles-tile")];
    if offset {
        attrs.push(("data-blocks-content-image-tiles-offset", ""));
    }
    div(
        attrs,
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-content-image-tiles-image", "")],
        )],
    )
}

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-content-image-tiles-stat", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `content-image-tiles` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウトとブレークポイント」節）。
pub fn demo() -> Node {
    let header = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("既存部品だけで画像タイルと指標を組み立てる")],
    );

    let body = div(
        vec![("class", "blocks-content-image-tiles-body")],
        PARAGRAPHS
            .iter()
            .map(|paragraph| {
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-content-image-tiles-paragraph", "")],
                    vec![text(*paragraph)],
                )
            })
            .collect(),
    );

    let tiles = div(
        vec![("class", "blocks-content-image-tiles-tiles")],
        vec![tile(false), tile(true), tile(false), tile(true)],
    );

    let columns = div(
        vec![("class", "blocks-content-image-tiles-columns")],
        vec![body, tiles],
    );

    let stats = div(
        vec![("class", "blocks-content-image-tiles-stats")],
        STATS
            .iter()
            .map(|(label, value)| stat_item(label, value))
            .collect(),
    );

    div(
        vec![("class", "blocks-content-image-tiles-layout")],
        vec![header, columns, stats],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0869。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しレベルを 1 段下げて `h3` にしました（ページ側が `## Demo` として
  `h2` を出すため）。
- 画像は `dummy_assets` の商品プレースホルダーにし、`alt=""`（装飾扱い）
  にしました。
- タイルの角丸は `ImageShape::Rounded`（`--fandhe-radius-md`）だけで
  揃え、独自の `border-radius` は書いていません。
- 数値指標は `stat` パーツ（`<dl>` 構造）で表現しました。
- 1 列へ切り替える幅はイシュー仕様どおり lg（64rem）にしました。
- 文言・配色・余白はすべて独自に書いたもの、または既存テーマトークンに
  そのまま従います。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Stat](../themes/stat.md)
