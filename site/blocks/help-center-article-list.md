# help-center-article-list

パンくず → コレクション見出し（アイコン・題名・説明・記事数バッジ）→
記事一覧カードを組み合わせたヘルプセンターの記事一覧ブロックです。
`breadcrumb` / `heading` / `card` / `stat` / `badge` / `icon` / `link` の
7 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0120（代表構成）で、R0122（見出しへのアイコン + 件数
バッジ）を集約しています。R0121（記事をグループ見出しで区切る構成）は
版 B として並記します。コレクション名・記事タイトル・件数はすべて架空の
データであり、実在の企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, li, p, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::Size;

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

/// 開いた本のアイコン（コレクション見出し用。版 A のみに付ける、
/// モジュール doc「2 版と集約元の対応」節参照）。
fn book_icon() -> Node {
    geo_icon("M4 5c4-2 8-1 8 1v13c0-2-4-3-8-1zm16 0c-4-2-8-1-8 1v13c0-2 4-3 8-1z")
}

/// 行末のシェブロン（右矢印。狭幅でも `margin-inline-start: auto` で
/// 右端に残す、モジュール doc「狭い幅では」節参照）。
fn chevron_icon() -> Node {
    geo_icon("M9 5l7 7-7 7")
}

/// パンくず 1 本（Blocks → ヘルプセンター → 現在のコレクション）。
/// A/B 共通で使う（`page_heading_meta.rs` の常時パンくず付きインスタンスと
/// 同型の合成）。
fn breadcrumb_row() -> Node {
    breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::Plain,
        Some("パンくずリスト"),
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
                    vec![breadcrumb::link(
                        "../../",
                        vec![],
                        vec![text("ヘルプセンター")],
                    )],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(
                        vec![],
                        vec![text("アカウントと請求")],
                    )],
                ),
            ],
        )],
    )
}

/// コレクション見出し行（見出し + 説明 + stat 2 個）。`with_icon` は版 A
/// のみ `true`（R0122）。
fn collection_heading(with_icon: bool, article_count: &'static str) -> Node {
    let mut heading_row_children = Vec::new();
    if with_icon {
        heading_row_children.push(book_icon());
    }
    heading_row_children.push(heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("アカウントと請求")],
    ));
    heading_row_children.push(badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(format!("{article_count} 件"))],
    ));
    div(
        vec![("class", "blocks-help-center-article-list-heading")],
        vec![
            div(
                vec![("data-blocks-help-center-article-list-heading-row", "")],
                heading_row_children,
            ),
            p(
                vec![("class", "blocks-help-center-article-list-description")],
                vec![text(
                    "アカウント設定・お支払い方法の変更・請求書の再発行など、\
                     契約管理に関するよくある質問をまとめています。",
                )],
            ),
        ],
    )
}

/// stat 1 個（ラベル + 値）。
fn stat_item(label: &'static str, value: &'static str) -> Node {
    stat::root(
        Size::Sm,
        vec![],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// stat 行（記事数・最終更新の 2 個を横並び）。
fn stat_row() -> Node {
    div(
        vec![("class", "blocks-help-center-article-list-stats")],
        vec![stat_item("記事", "12"), stat_item("最終更新", "2026-09-18")],
    )
}

/// 記事 1 行（タイトル + 行末シェブロン）。
fn article_row(title: &'static str, href: &'static str) -> Node {
    li(
        vec![],
        vec![link::root(
            href,
            &LinkProps::default(),
            vec![("data-blocks-help-center-article-list-row", "")],
            vec![span(vec![], vec![text(title)]), chevron_icon()],
        )],
    )
}

/// A: 代表構成（R0120 + R0122）。1 枚のカードへ記事 6 件をフラットに並べる。
fn version_flat() -> Node {
    let rows = ul(
        vec![("class", "blocks-help-center-article-list-rows")],
        vec![
            article_row("お支払い方法を登録・変更する", "../../guides/"),
            article_row("請求書（領収書）を再発行する", "../../api/"),
            article_row("プランをアップグレードする", "../../primitives/"),
            article_row("年払いと月払いの違い", "../../themes/"),
            article_row("解約・返金ポリシーについて", "../../wireframes/"),
            article_row("チームメンバーへの請求権限の付与", "../../"),
        ],
    );
    let card_node = card::root(
        CardProps::default(),
        vec![("data-blocks-help-center-article-list-card", "")],
        vec![card::body(vec![], vec![rows])],
    );
    div(
        vec![
            ("data-blocks-help-center-article-list-instance", ""),
            ("data-blocks-help-center-article-list-variant", "a"),
        ],
        vec![
            breadcrumb_row(),
            collection_heading(true, "12"),
            stat_row(),
            card_node,
        ],
    )
}

/// グループ見出し + 記事 `<li>` 列の 1 区分。
fn article_group(title: &'static str, rows: Vec<(&'static str, &'static str)>) -> Node {
    div(
        vec![("class", "blocks-help-center-article-list-group")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            ul(
                vec![("class", "blocks-help-center-article-list-rows")],
                rows.into_iter()
                    .map(|(title, href)| article_row(title, href))
                    .collect(),
            ),
        ],
    )
}

/// B: グループ見出しで分割（R0121）。カード内を 3 区分に分ける。
fn version_grouped() -> Node {
    let card_node = card::root(
        CardProps::default(),
        vec![("data-blocks-help-center-article-list-card", "")],
        vec![card::body(
            vec![],
            vec![
                article_group(
                    "はじめに",
                    vec![
                        ("アカウントを作成する", "../../guides/"),
                        ("初期設定チェックリスト", "../../api/"),
                    ],
                ),
                article_group(
                    "アカウント",
                    vec![
                        ("メールアドレスを変更する", "../../primitives/"),
                        ("二段階認証を設定する", "../../themes/"),
                    ],
                ),
                article_group(
                    "請求",
                    vec![
                        ("お支払い方法を登録・変更する", "../../wireframes/"),
                        ("請求書（領収書）を再発行する", "../../"),
                    ],
                ),
            ],
        )],
    );
    div(
        vec![
            ("data-blocks-help-center-article-list-instance", ""),
            ("data-blocks-help-center-article-list-variant", "b"),
        ],
        vec![breadcrumb_row(), collection_heading(false, "6"), card_node],
    )
}

/// `help-center-article-list` の Demo 本体（版 A・B を縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-help-center-article-list-stack")],
        vec![version_flat(), version_grouped()],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0120 + R0122）**: 見出しに開いた本の線画アイコン + 題名
  + 記事数バッジを添え、`stat` を 2 個（記事数・最終更新）横並びで表示した
  あと、1 枚の `card` に記事 6 件をフラットに並べます。
- **版 B（グループ見出しで分割、R0121）**: 見出しはアイコンを省いてバッジ
  のみとし、カード内を H3 のグループ見出し（はじめに・アカウント・請求）
  で 3 区分し、区分ごとに記事を並べます。
- 記事行間の罫線・狭幅時のカード余白の詰めは `card`/`link` 部品自体の機能
  ではなく、本 block の CSS が付与しています。
- 狭幅（コンテナ幅 36rem 未満）では `card` のパディングとカード内行の
  パディングが詰まりますが、行末のシェブロンは `margin-inline-start: auto`
  のまま右端に残ります（`@container` によるコンテナクエリ判定）。
- 見出し・シェブロンのアイコンは自作の線画（SVG path）で、参照元由来の
  アイコンセットではありません。

関連情報: [Breadcrumb](../themes/breadcrumb.md) /
[Heading](../themes/heading.md) / [Card](../themes/card.md) /
[Stat](../themes/stat.md) / [Badge](../themes/badge.md) /
[Icon](../themes/icon.md) / [Link](../themes/link.md)
