# grid-list-action-tiles

`fandhe-frontend-pre-styled-ui` の `item` / `icon` / `heading` / `text` /
`link-overlay` の 5 部品のみを合成した、境界線を共有するアクションタイルの
グリッドです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（対応表 ID R0977 の 1 件のみを参照しています。出典の固有名・ファイル名は
記載しません）。

各タイルは面色付きのアイコン枠・見出し・説明文・右上の矢印を持ち、タイル
全体が 1 つのリンクになります。境界線を共有するため、各タイルは個別の
枠線を持たず、グリッド全体の外枠だけが角丸になります。広い幅では 2 列、
狭い幅では 1 列に切り替わり、角丸は先頭タイルと末尾タイルの外側の角だけに
付きます。リンク先はサイト内の実在ページの相対パスです。実際に使う場合は
自分のページの URL へ差し替えてください。

静的な表示のみで、状態機械・`<form>`・送信処理は持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

const TILE_ATTR: &str = "data-blocks-grid-list-action-tiles-tile";
const ITEM_ATTR: &str = "data-blocks-grid-list-action-tiles-item";
const MEDIA_ATTR: &str = "data-blocks-grid-list-action-tiles-media";
const CONTENT_ATTR: &str = "data-blocks-grid-list-action-tiles-content";
const ARROW_ATTR: &str = "data-blocks-grid-list-action-tiles-arrow";

/// タイル 1 件分のデータ（モジュール doc「href の方針」節参照）。
struct Tile {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// タイル 6 件（サイト内に実在する索引ページのみを指す）。
const TILES: [Tile; 6] = [
    Tile {
        href: "../../guides/",
        title: "ガイドを読む",
        description: "導入から実践までの手順を順番に確認できます。",
        icon_path_d: "M4 4h12v16H4zM8 8h4M8 12h4",
    },
    Tile {
        href: "../../examples/",
        title: "サンプルを試す",
        description: "実際に動くサンプルプロジェクトを確認できます。",
        icon_path_d: "M12 3l2.5 5.5L20 9l-4 4 1 6-5-3-5 3 1-6-4-4 5.5-.5z",
    },
    Tile {
        href: "../../primitives/",
        title: "Primitives を見る",
        description: "構造とアクセシビリティを担う headless 部品一覧です。",
        icon_path_d: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
    },
    Tile {
        href: "../../themes/",
        title: "Themes を見る",
        description: "そのまま使える意匠付きの部品一覧です。",
        icon_path_d: "M4 6h16M4 12h16M4 18h10",
    },
    Tile {
        href: "../",
        title: "Blocks 一覧に戻る",
        description: "既存部品を組み合わせた合成例の一覧です。",
        icon_path_d: "M20 12H4M10 6l-6 6 6 6",
    },
    Tile {
        href: "../../api/",
        title: "API Reference を読む",
        description: "公開 API の詳細なリファレンスを確認できます。",
        icon_path_d: "m9 6 6 6-6 6",
    },
];

/// 自作の線画（stroke）アイコンを組み立てる（
/// [`super::super::marketing::cta::cta_feature_links::geo_icon`] と同型の
/// パターン。装飾用途のため `IconProps::label` は付けない）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// タイル 1 件分を組み立てる（モジュール doc「タイル全体のリンク化と矢印
/// の配置」節）。
fn action_tile(tile: &Tile) -> Node {
    link_overlay::root(
        vec![(TILE_ATTR, "")],
        vec![
            item::root(
                ItemRootProps::default(),
                vec![(ITEM_ATTR, "")],
                vec![
                    item::media(
                        ItemMediaVariant::Icon,
                        vec![(MEDIA_ATTR, "")],
                        vec![geo_icon(tile.icon_path_d)],
                    ),
                    item::content(
                        vec![(CONTENT_ATTR, "")],
                        vec![
                            heading(
                                HeadingLevel::H3,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(tile.title)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(tile.description)],
                            ),
                        ],
                    ),
                    item::actions(vec![(ARROW_ATTR, "")], vec![geo_icon("m9 18 6-6-6-6")]),
                ],
            ),
            overlay(tile.href, vec![("aria-label", tile.title)], vec![]),
        ],
    )
}

/// `grid-list-action-tiles` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-grid-list-action-tiles-grid")],
        TILES.iter().map(action_tile).collect(),
    )
}
```

## 原案差分メモ

参照（対応表 ID R0977。出典の固有名・ファイル名は記載しません）から
取り込んだのは構造（境界共有のタイルグリッド・アイコン枠 + 見出し +
説明文 + 矢印の並び・角丸の分岐）のみであり、次の点を独自に設計・変更
しています。

- 集約元は R0977 の 1 件のみです（他 ID との統合は行っていません）。
- アイコンの面色はタイルごとに変えず、`item` の Icon variant が持つ共通の
  面色トークンをそのまま使っています。
- アイコンはすべて自作の抽象幾何図形です（実在ブランドのロゴ・商標は
  模していません）。
- 文言（見出し・説明文）はすべて独自に書いたものです。docs サイト内の
  ページ導線を題材にしています。
- リンク先はサイト内に実在する索引ページ（Guides / Examples /
  Primitives / Themes / Blocks 一覧 / API Reference）への相対パスです。
- 狭い幅では 1 列に切り替わり、角丸は先頭タイルと末尾タイルの外側の角
  だけに限定しています（2 列時は 4 隅それぞれ 1 タイルのみが角丸を持ち
  ます）。
