# content-columns-screenshot

`badge` / `heading` / `text` / `button` / `image` の 5 部品を合成した、
eyebrow badge + 見出し + 2 列本文 + CTA + 画面画像を積む構成です。

幅 md（48rem）以上では本文が 2 列、それより狭い幅では 1 列に切り替わりま
す。画面画像の下端は背景色へ向かうグラデーションでフェードします。文言
はすべて架空のもので、データ取得・送信は行わない静的な表示例です。
`<form>` は使用しません。

集約元は 1 件のみ（対応表 ID R0870）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 本文 2 列それぞれの段落群（架空文言、1〜2 文程度に短くして検索
/// インデックスのサイズを抑える。§8「検索インデックスのサイズ」参照）。
const COLUMNS: [&[&str]; 2] = [
    &[
        "テキストは既定エスケープを経由した `text()` ノードとしてのみ差し込みます。",
        "エスケープを迂回する明示的なオプトイン API を使わない限り、渡した文字列が構造化タグとして解釈されることはありません。",
    ],
    &[
        "画面の骨格はノード木 API で組み立てるため、文字列結合による HTML 生成は発生しません。",
        "この Demo 自体も送信処理・データ取得を持たない静的な表示です。",
    ],
];

/// 本文 1 列分（段落を `styled_text::text` で並べる）。
fn column(paragraphs: &[&str]) -> Node {
    div(
        vec![("class", "blocks-content-columns-screenshot-column")],
        paragraphs
            .iter()
            .map(|paragraph| {
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-content-columns-screenshot-paragraph", "")],
                    vec![text(*paragraph)],
                )
            })
            .collect(),
    )
}

/// `content-columns-screenshot` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（モジュール doc「レイアウトとブレークポイント」
/// 節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-content-columns-screenshot-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-columns-screenshot-eyebrow", "")],
                vec![text("導入ガイド")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("既存部品だけで画面を組み立てる")],
            ),
        ],
    );

    let columns = div(
        vec![("class", "blocks-content-columns-screenshot-columns")],
        COLUMNS
            .iter()
            .map(|paragraphs| column(paragraphs))
            .collect(),
    );

    let actions = div(
        vec![("class", "blocks-content-columns-screenshot-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("ドキュメントを読む")],
        )],
    );

    let shot = div(
        vec![("class", "blocks-content-columns-screenshot-shot")],
        vec![image::image(
            &ImageProps {
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-content-columns-screenshot-image", "")],
        )],
    );

    div(
        vec![("class", "blocks-content-columns-screenshot-layout")],
        vec![header, columns, actions, shot],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0870。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 2 列への切り替えを lg ではなく md（48rem）にしました（イシュー仕様に
  合わせたものです）。
- CTA をリンクではなく `button` パーツ（`type="button"`）にしました。
- eyebrow をスタイル付きの段落ではなく `badge` にしました。
- 見出しレベルを 1 段下げて `h3` にしました（ページ側が `## Demo` として
  `h2` を出すため）。
- 画像下端のフェード先をデモ枠の背景色（`--fandhe-color-bg-subtle`）に
  しました。デモは常にこの背景色の上に描かれるため、参照のようにページ
  背景へフェードさせるとデモ枠内に色の継ぎ目が出るためです。
- 画像の大きな影・負のマージンによる切り詰めは行わず、既存トークンの
  枠線と影（`--fandhe-shadow-lg`）に簡略化しました。
- 画像は `dummy_assets` のスクリーンショット枠プレースホルダーにし、
  `alt=""`（装飾扱い）にしました。
- 文言・配色・余白はすべて独自に書いたもの、または既存テーマトークンに
  そのまま従います。
