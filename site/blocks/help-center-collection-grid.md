# help-center-collection-grid

パンくずと見出しの下に、ヘルプ記事コレクションをカードのグリッドで並べる
ヘルプセンター向けブロックです。`breadcrumb` / `heading` / `text` / `card` /
`icon` / `stat` / `avatar` / `link` の 8 部品を合成します。Blocks は既存部品の合成例
であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0116（代表構成）で、R0117（細部差のみ）・R0118（著者
アバター群）・R0119（浮き上がりカード + ヘッダー）を集約しています。狭い
コンテナ幅では 3 列 → 2 列 → 1 列へグリッドを切り替えます（`@container`
によるコンテナクエリ判定）。コレクション名・説明文・著者アバターはすべて
架空のダミーデータであり、実在の企業・人物・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 本アイコン（Getting Started）。
fn book_icon() -> Node {
    geo_icon("M4 4h9a3 3 0 013 3v13a3 3 0 00-3-3H4z M20 4h-9a3 3 0 00-3 3v13a3 3 0 013-3h9z")
}

/// カードアイコン（Billing）。
fn card_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 10h18")
}

/// 盾アイコン（Security）。
fn shield_icon() -> Node {
    geo_icon("M12 3l7 3v6c0 5-3 8-7 9-4-1-7-4-7-9V6z")
}

/// 歯車アイコン（Integrations）。
fn gear_icon() -> Node {
    geo_icon(
        "M12 8a4 4 0 100 8 4 4 0 000-8z M12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 \
         M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1",
    )
}

/// チャットアイコン（Troubleshooting）。
fn chat_icon() -> Node {
    geo_icon("M4 5h16v11H8l-4 4z")
}

/// ロケットアイコン（API Reference）。
fn rocket_icon() -> Node {
    geo_icon(
        "M12 2c3 2 5 6 5 10-2 1-3 3-5 3s-3-2-5-3c0-4 2-8 5-10z \
         M9 15l-3 3v3h3l3-3 M14 8a1 1 0 100 2 1 1 0 000-2z",
    )
}

/// アイコン枠（装飾。`icon::icon` を丸角の背景で囲む）。
fn icon_frame(icon_node: Node) -> Node {
    div(
        vec![("class", "blocks-help-center-collection-grid-icon")],
        vec![icon_node],
    )
}

/// 版 A の 1 カード（輪郭カード。アイコン枠 + 題名リンク + 説明 + 記事数）。
fn collection_card(
    icon_node: Node,
    href: &'static str,
    title: &'static str,
    description: &'static str,
    article_count: &'static str,
) -> Node {
    card::root(
        CardVariant::Outline,
        vec![("data-blocks-help-center-collection-grid-card", "")],
        vec![card::body(
            vec![],
            vec![
                icon_frame(icon_node),
                heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![link::root(
                        href,
                        &LinkProps::default(),
                        vec![],
                        vec![text(title)],
                    )],
                ),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(description)],
                ),
                stat::root(
                    Size::Sm,
                    vec![("data-blocks-help-center-collection-grid-stat", "")],
                    vec![
                        stat::label(vec![], vec![text("articles")]),
                        stat::value_text(vec![], vec![text(article_count)]),
                    ],
                ),
            ],
        )],
    )
}

/// A: 代表構成（R0116 + R0117）。輪郭カード 6 枚のグリッド。
fn version_representative() -> Node {
    let header = div(
        vec![("class", "blocks-help-center-collection-grid-header")],
        vec![heading(
            HeadingLevel::H2,
            &HeadingProps::default(),
            vec![],
            vec![text("Popular collections")],
        )],
    );
    let grid = div(
        vec![("class", "blocks-help-center-collection-grid-grid")],
        vec![
            collection_card(
                book_icon(),
                "../../guides/embedding-guide/",
                "Getting Started",
                "Embed the framework into an existing page and mount your first component.",
                "24",
            ),
            collection_card(
                card_icon(),
                "../../guides/npm-asset-build/",
                "Asset Pipeline",
                "Build and gate NPM-based static assets with the install.sh pipeline.",
                "18",
            ),
            collection_card(
                shield_icon(),
                "../../guides/no-js-ssg/",
                "Zero-JS Sites",
                "Ship static pages that render correctly without client-side JavaScript.",
                "15",
            ),
            collection_card(
                gear_icon(),
                "../../guides/wasm-full-features/",
                "WASM Features",
                "Pick the wasm-full Cargo feature flags your bundle actually needs.",
                "31",
            ),
            collection_card(
                chat_icon(),
                "../../guides/pre-styled-ui-motion-feature/",
                "Motion & Animation",
                "Turn on the pre-styled-ui motion feature for interactive effects.",
                "42",
            ),
            collection_card(
                rocket_icon(),
                "../../api/",
                "API Reference",
                "Browse component and server API documentation by crate.",
                "27",
            ),
        ],
    );
    div(
        vec![("class", "blocks-help-center-collection-grid-section")],
        vec![header, grid],
    )
}

/// 版 B の 1 カード（浮き上がりカード。フッターに著者アバター群 + 記事数）。
fn featured_card(
    icon_node: Node,
    href: &'static str,
    title: &'static str,
    description: &'static str,
    authors: &[usize],
    article_count: &'static str,
) -> Node {
    let stacked_props = AvatarProps {
        stacked: true,
        ..AvatarProps::default()
    };
    let author_avatar = |index: usize| -> Node {
        let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
        avatar::root(
            &stacked_props,
            vec![],
            vec![avatar::image(
                ImageStatus::Loaded,
                dummy_assets::AVATAR_SRC,
                name,
                vec![],
            )],
        )
    };
    card::root(
        CardVariant::Elevated,
        vec![("data-blocks-help-center-collection-grid-card", "")],
        vec![
            card::body(
                vec![],
                vec![
                    icon_frame(icon_node),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![link::root(
                            href,
                            &LinkProps::default(),
                            vec![],
                            vec![text(title)],
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description)],
                    ),
                ],
            ),
            card::footer(
                vec![("data-blocks-help-center-collection-grid-footer", "")],
                vec![
                    avatar::group(
                        vec![
                            ("data-blocks-help-center-collection-grid-authors", ""),
                            ("aria-hidden", "true"),
                        ],
                        authors.iter().map(|index| author_avatar(*index)).collect(),
                    ),
                    stat::root(
                        Size::Sm,
                        vec![("data-blocks-help-center-collection-grid-stat", "")],
                        vec![
                            stat::label(vec![], vec![text("articles")]),
                            stat::value_text(vec![], vec![text(article_count)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// B: 著者アバター群 + 浮き上がりカード + ヘッダー（R0118 + R0119）。
fn version_featured() -> Node {
    let header = div(
        vec![("class", "blocks-help-center-collection-grid-header")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("Featured collections")],
            ),
            link::root(
                "../../guides/",
                &LinkProps::default(),
                vec![("data-blocks-help-center-collection-grid-view-all", "")],
                vec![text("View all")],
            ),
        ],
    );
    let grid = div(
        vec![("class", "blocks-help-center-collection-grid-grid")],
        vec![
            featured_card(
                book_icon(),
                "../../guides/animation-core/",
                "Animation Core API",
                "Call fandhe-animation and fandhe-frontend-animation directly from Rust.",
                &[0, 1, 2],
                "16",
            ),
            featured_card(
                gear_icon(),
                "../../guides/animation/",
                "Animation Features",
                "Add declarative, data-* driven animations without writing JS.",
                &[1, 2, 3],
                "22",
            ),
            featured_card(
                shield_icon(),
                "../../guides/deployment/",
                "Deployment",
                "Ship as static output (SSG) or a single-binary dist-server.",
                &[2, 3, 0],
                "11",
            ),
        ],
    );
    div(
        vec![("class", "blocks-help-center-collection-grid-section")],
        vec![header, grid],
    )
}

/// パンくずリスト（両版で共通）。
/// `../` は本ページ（`/blocks/help-center-collection-grid/`）から見て
/// Blocks インデックスを指すため、ラベルも実リンク先と一致させて
/// 「Blocks」とする（姉妹ブロック help-center-article-list 系の修正と
/// 同じ判断、PR #3428 レビュー指摘対応）。
fn breadcrumb_row() -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("Breadcrumb"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Collections")])],
                ),
            ],
        )],
    )
}

/// `help-center-collection-grid` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-help-center-collection-grid-stack")],
        vec![
            div(
                vec![("class", "blocks-help-center-collection-grid-top-row")],
                vec![breadcrumb_row()],
            ),
            version_representative(),
            version_featured(),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0116 + R0117）**: 輪郭カード（Outline）6 枚のグリッド。
  各カードはアイコン枠・題名（リンク化した見出し）・説明・記事数（`stat`）を
  持ちます。R0117 は細部（配色・余白）のみの差分のため、Demo 上で別インス
  タンスには分離していません。
- **版 B（著者アバター群 + 浮き上がりカード + ヘッダー、R0118 + R0119）**:
  浮き上がりカード（Elevated）3 枚のグリッド。ヘッダー行に「View all」
  リンクを併記し、各カードのフッターに著者アバター群（`avatar::group`、
  3 名の重なり表示）+ 記事数（`stat`）を並べます。
- 著者アバター群は装飾情報（記事数と併記するのみ）のため `aria-hidden` を
  付与し、特定個人を示す意味は持たせていません。
- アイコンは自作の線画（SVG path）で、参照元由来のアイコンセットでは
  ありません。

関連情報: [Breadcrumb](../themes/breadcrumb.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Card](../themes/card.md) / [Icon](../themes/icon.md) /
[Stat](../themes/stat.md) / [Avatar](../themes/avatar.md) / [Link](../themes/link.md)
