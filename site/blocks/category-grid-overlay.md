# category-grid-overlay

上部の見出し行とカテゴリ画像タイルの均等グリッドで構成する目的別パーツです。
`heading` / `link` / `link-overlay` / `image` / `text` の 5 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

各タイルは同じ大きさの画像で、下部へ名称と短い説明を画像へ重ねて表示します。
タイル全体を 1 つのリンクにし、フォーカス時はタイル内側に枠線を表示します。
コンテナ幅に応じて列数を 1 → 2 → 3 → 4 と増やし、狭い幅では 1〜2 列、広い幅で
3〜4 列になります。

主参照は対応表 ID R0036（8 枚の正方形タイル・タイル全体がリンク）で、R0035・
R0038・R0610 を集約しています。文言・配色・装飾は参照元から持ち込まず、カテゴリ
名称・説明はすべて架空のデータです。画像はビルド時生成の同梱プレースホルダー
SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。上の Demo は
「見出しあり・正方形タイル 8 枚」（Variant A）と「見出しなし・縦長タイル
4 枚」（Variant B）の 2 variant で構成します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// カテゴリ 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Category {
    name: &'static str,
    description: &'static str,
}

/// 正方形タイル 8 件（Variant A、R0036 主参照）。
const CATEGORIES_SQUARE: [Category; 8] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を支える定番アイテム",
    },
    Category {
        name: "文具",
        description: "書く・まとめる・持ち運ぶ道具一式",
    },
    Category {
        name: "照明",
        description: "空間の雰囲気を整える光源",
    },
    Category {
        name: "収納",
        description: "暮らしを整える棚・箱・ラック",
    },
    Category {
        name: "テキスタイル",
        description: "肌触りにこだわった布製品",
    },
    Category {
        name: "アウトドア",
        description: "屋外での時間を快適にする道具",
    },
    Category {
        name: "植物",
        description: "部屋に緑を添える鉢植え各種",
    },
    Category {
        name: "ギフト",
        description: "贈る相手を選ばない定番の贈り物",
    },
];

/// 縦長タイル 4 件（Variant B、R0035 の枚数 + R0610 の比率）。
const CATEGORIES_PORTRAIT: [Category; 4] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を支える定番アイテム",
    },
    Category {
        name: "文具",
        description: "書く・まとめる・持ち運ぶ道具一式",
    },
    Category {
        name: "照明",
        description: "空間の雰囲気を整える光源",
    },
    Category {
        name: "収納",
        description: "暮らしを整える棚・箱・ラック",
    },
];

/// カテゴリタイル 1 件を組み立てる（モジュール doc「重なり順」節参照）。
fn tile(cat: &Category, ratio: AspectRatio) -> Node {
    link_overlay::root(
        vec![("data-blocks-category-grid-overlay-tile", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: ratio,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-category-grid-overlay-image", "")],
            ),
            div(
                vec![
                    ("class", "blocks-category-grid-overlay-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            div(
                vec![("class", "blocks-category-grid-overlay-content")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-category-grid-overlay-name", "")],
                        vec![text(cat.name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(cat.description)],
                    ),
                ],
            ),
            overlay(
                REPO,
                vec![
                    ("aria-label", cat.name),
                    ("data-blocks-category-grid-overlay-overlay", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// Variant A: 見出し行 + 正方形タイル 8 枚（R0036 主参照 + R0610 の見出し
/// 行）。
fn variant_square() -> Node {
    let tiles: Vec<Node> = CATEGORIES_SQUARE
        .iter()
        .map(|cat| tile(cat, AspectRatio::Square))
        .collect();
    div(
        vec![("class", "blocks-category-grid-overlay-variant")],
        vec![
            div(
                vec![("class", "blocks-category-grid-overlay-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("カテゴリから探す")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![
                            text("すべてのカテゴリ"),
                            span(vec![("aria-hidden", "true")], vec![text(" \u{2192}")]),
                        ],
                    ),
                ],
            ),
            div(vec![("class", "blocks-category-grid-overlay-grid")], tiles),
        ],
    )
}

/// Variant B: 見出しなし + 縦長タイル 4 枚（R0035 の枚数 + R0610 の比率）。
fn variant_portrait() -> Node {
    let tiles: Vec<Node> = CATEGORIES_PORTRAIT
        .iter()
        .map(|cat| tile(cat, AspectRatio::Portrait))
        .collect();
    div(
        vec![("class", "blocks-category-grid-overlay-variant")],
        vec![div(
            vec![("class", "blocks-category-grid-overlay-grid")],
            tiles,
        )],
    )
}

/// `category-grid-overlay` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウトと `@container` の理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-grid-overlay")],
        vec![variant_square(), variant_portrait()],
    )
}
```

## 原案差分メモ

- **R0035（4 枚均等・リンクなし版）**: 全タイルをリンク化し、タイル全体を
  クリック可能にしました。
- **R0036（主参照、8 枚正方形タイル・全体リンク）**: Variant A（見出し行 +
  正方形タイル 8 枚）として取り込みました。
- **R0038（3 枚・説明と買い物リンクを重ねる版）**: 個別の買い物リンクは入れ子
  リンクを避けるためタイル全体のリンク 1 本へ統合し、説明文はすべてのタイルへ
  重ねる形で取り込みました。
- **R0610（見出し行 + 一覧ボタン、縦長 4 枚）**: 見出し行と「すべてのカテゴリ」
  リンクは Variant A に、縦長比率は Variant B（見出しなし・縦長タイル 4 枚）に
  取り込みました。一覧ボタンはページ遷移のため `<a>`（リンク）として表現
  しています。
- 見出しは `<h3>` にしています（ページ側が `<h1>`/`<h2>` を出すため）。タイル
  名称自体は見出しにせず、`link-overlay` の `aria-label` でアクセシブルな名前を
  与えています。
- 配色は `--fandhe-color-fg`/`--fandhe-color-bg` の反転ペアにし、実写画像は
  ダミー商品画像プレースホルダーに置き換えました。
- 列数は viewport 幅ではなく Demo 枠自身の実測幅を基準にしたコンテナクエリ
  （`@container`）で切り替えています。

関連情報: [Heading](../themes/heading.md) / [Link](../themes/link.md) /
[Link Overlay](../themes/link-overlay.md) / [Image](../themes/image.md) /
[Text](../themes/text.md)
