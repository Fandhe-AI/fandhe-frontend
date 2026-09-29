# empty-state-starter-grid

まだプロジェクトが存在しない状態を示す空状態です。見出しと説明文の下に、
開始候補（テンプレート等）を「アイコン + 題名 + 説明 + 開く操作」のタイルと
してグリッドで並べ、末尾に「別の開始方法」への導線を置きます。
`item` / `icon` / `heading` / `text` / `link` の 5 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0463（代表構成）で、R1396（テンプレート選択の 2 列
グリッド・全面クリック）を集約しています。タイルの遷移先はいずれも
このサイト内に実在する索引ページ（ガイド・サンプル・Primitives 等）への
相対パスです。実際に使う際は自分のページ URL へ差し替えてください。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。タイル全体を
クリック可能にするため「開く」リンクを stretched-link（CSS の
`::after { inset: 0 }`）でタイル全面へ拡張しています。`button` は使用部品
に含めていません。「タイル全体をリンク」と「開くボタン」を両立させようと
すると `<a>` に `<button>` を入れ子にすることになり HTML として不正になる
ためです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const TILE_ATTR: &str = "data-blocks-empty-state-starter-grid-tile";
const MEDIA_ATTR: &str = "data-blocks-empty-state-starter-grid-media";
const OPEN_ATTR: &str = "data-blocks-empty-state-starter-grid-open";

/// タイル 1 件分のデータ（モジュール doc「href の方針」節参照）。
struct Tile {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 開始候補タイル 6 件（サイト内に実在する索引ページのみを指す）。
const TILES: [Tile; 6] = [
    Tile {
        href: "../../guides/",
        title: "ブログ",
        description: "記事一覧と詳細ページを備えた構成から始めます。",
        icon_path_d: "M4 4h16v16H4zM8 8h8M8 12h8M8 16h5",
    },
    Tile {
        href: "../../examples/",
        title: "ダッシュボード",
        description: "指標カードとグラフを並べた管理画面から始めます。",
        icon_path_d: "M4 4h7v7H4zM13 4h7v4h-7zM13 11h7v9h-7zM4 14h7v6H4z",
    },
    Tile {
        href: "../../primitives/",
        title: "ランディングページ",
        description: "見出しと導線を備えた 1 ページ構成から始めます。",
        icon_path_d: "M4 4h16v6H4zM4 13h16M4 17h10",
    },
    Tile {
        href: "../../themes/",
        title: "ドキュメントサイト",
        description: "サイドバー付きの技術文書構成から始めます。",
        icon_path_d: "M4 4h6v16H4zM12 4h8v16h-8zM14 8h4M14 12h4",
    },
    Tile {
        href: "../",
        title: "設定画面",
        description: "フォームと保存操作を備えた設定画面から始めます。",
        icon_path_d: "M12 3l1.2 2.4 2.6.4-1.9 1.9.5 2.6-2.4-1.3-2.4 1.3.5-2.6-1.9-1.9 2.6-.4z",
    },
    Tile {
        href: "../../api/",
        title: "API カタログ",
        description: "エンドポイント一覧を備えた参照ページから始めます。",
        icon_path_d: "M4 4h16v16H4zM4 9h16",
    },
];

/// 自作の線画（stroke）アイコンを組み立てる（`grid_list_action_tiles::
/// geo_icon` と同型のパターン。装飾用途のため `IconProps::label` は付けない）。
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
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// タイル 1 件分を組み立てる（モジュール doc「構造」節。`item::root` へ
/// `position: relative`（[`LAYOUT_CSS`]）を持たせ、`open_link` の
/// stretched-link（`::after { inset: 0 }`）でタイル全面をクリック可能に
/// する）。
fn starter_tile(tile: &Tile) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![(TILE_ATTR, "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![(MEDIA_ATTR, "")],
                vec![geo_icon(tile.icon_path_d)],
            ),
            item::content(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H4,
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
            item::actions(
                vec![],
                vec![link::root(
                    tile.href,
                    &LinkProps::default(),
                    vec![
                        (OPEN_ATTR, ""),
                        ("aria-label", &format!("{} を開く", tile.title)),
                    ],
                    vec![text("開く")],
                )],
            ),
        ],
    )
}

/// 見出し + 説明文 + タイルグリッド + 末尾導線の 1 インスタンス分を
/// 組み立てる。`columns` は `[data-columns]` の値（`@container` 側で
/// 参照、[`LAYOUT_CSS`]）。
fn starter_instance(
    heading_text: &'static str,
    description_text: &'static str,
    columns: &'static str,
    tiles: &[Tile],
) -> Node {
    div(
        vec![("class", "blocks-empty-state-starter-grid-instance")],
        vec![
            div(
                vec![("class", "blocks-empty-state-starter-grid-header")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(heading_text)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description_text)],
                    ),
                ],
            ),
            div(
                vec![
                    ("class", "blocks-empty-state-starter-grid-grid"),
                    ("data-columns", columns),
                ],
                tiles.iter().map(starter_tile).collect(),
            ),
            div(
                vec![("class", "blocks-empty-state-starter-grid-footer")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("テンプレートを使いませんか。")],
                    ),
                    link::root(
                        "../",
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("空のプロジェクトから始める")],
                    ),
                ],
            ),
        ],
    )
}

/// 版ラベル（`empty_state_invite_team::variant_label` と同型）。
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

/// `empty-state-starter-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（モジュール doc「2 版と集約元の対応」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-empty-state-starter-grid-stack")],
        vec![
            variant_label("代表構成・3 列（R0463）"),
            starter_instance(
                "最初のプロジェクトを作成しましょう",
                "開始候補から選ぶと、あらかじめ用意された構成ですぐに作業を始められます。",
                "3",
                &TILES,
            ),
            variant_label("テンプレート選択・2 列（R1396）"),
            starter_instance(
                "テンプレートを選んで開始",
                "目的に近いテンプレートを選ぶと、初期設定を省略できます。",
                "2",
                &TILES[..4],
            ),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成・3 列、R0463）**: 見出し・説明文の下に、開始候補タイル
  6 件を 3 列グリッドで並べます。
- **版 B（テンプレート選択・2 列、R1396）**: 版 A と同じ骨格のまま、候補を
  先頭 4 件へ絞り 2 列グリッドで並べます。
- 各タイルはアイコン・題名・説明・「開く」リンクで構成し、「開く」
  リンクを stretched-link（CSS の `::after { inset: 0 }`）でタイル全面へ
  拡張して、タイルのどこをクリックしても遷移できるようにしています。
- Issue の部品指示にある `button` は使用しません。「タイル全体をリンク」
  と「開くボタン」を両立させようとすると `<a>` に `<button>` を入れ子に
  することになり HTML として不正になるためです。ボタン風の外見は
  `link::root` へ枠線・padding を与えて表現しています。
- 狭幅（コンテナ幅 36rem 未満）では常に 1 列、36rem 以上では 2 列、
  56rem 以上かつ版 A（3 列指定）のみ 3 列へ切り替わります（`@container`
  によるコンテナクエリ判定）。
- 装飾アイコンは自作の線画（SVG path）で、参照元由来のアイコンセットでは
  ありません。

関連情報: [Item](../themes/item.md) / [Icon](../themes/icon.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md)
