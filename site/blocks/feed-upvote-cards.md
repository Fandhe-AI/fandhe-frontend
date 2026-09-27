# feed-upvote-cards

`fandhe-frontend-pre-styled-ui` の `card` / `avatar` / `badge` / `button` /
`status` / `icon` の 6 部品のみを合成した、投票数付き投稿カードのフィード
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（集約元は対応表 ID R0110 の 1 件のみです。出典の固有名・ファイル名は
記載しません）。

各カードは投票ボタン + 得票数の列と、投稿者アバター・氏名・日付・状態・
タイトル・抜粋・タグから成る本文で構成されています。狭い幅では投票列が
カード上部で横並びになり、`md`（768px）以上で投票列がカード右側へ固定
されます。投票は無 JS の静的表示で、投稿の 1 件だけを「投票済み」
（塗りつぶしボタン + `aria-pressed="true"`）に固定し、送信・状態遷移は
行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 投稿 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    /// [`dummy_assets::PERSON_NAMES`] への添字。
    author_index: usize,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    excerpt: &'static str,
    tags: &'static [&'static str],
    votes: u32,
    /// `true` の投稿のみ投票済み（モジュール doc「投票ボタンを 1 件だけ
    /// 『投票済み』で固定する理由」節）で固定表示する。
    voted: bool,
    status_label: &'static str,
    status_palette: ColorPalette,
}

/// 投稿一覧（架空、4 件）。`voted: true` はちょうど 1 件のみ。
const POSTS: [Post; 4] = [
    Post {
        author_index: 0,
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "既定エスケープの回帰テストを増強しました",
        excerpt:
            "SSR/SSG/CSR の各経路で XSS 回帰テストを追加した提案です。レビューをお願いします。",
        tags: &["設計", "テスト"],
        votes: 42,
        voted: false,
        status_label: "受付中",
        status_palette: ColorPalette::Info,
    },
    Post {
        author_index: 1,
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        title: "block 追加 PR のレビュー時間を短縮する提案",
        excerpt: "レジストリと原稿を分離したことで、レビュー観点を絞り込めるようになりました。",
        tags: &["運用"],
        votes: 128,
        voted: true,
        status_label: "解決済み",
        status_palette: ColorPalette::Success,
    },
    Post {
        author_index: 2,
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "Wireframe UI のダークモード対応について",
        excerpt: "モノクロトークンをダークモードでどう反転させるか、意見を募集しています。",
        tags: &["デザイン", "アクセシビリティ"],
        votes: 7,
        voted: false,
        status_label: "受付中",
        status_palette: ColorPalette::Info,
    },
    Post {
        author_index: 3,
        date_iso: "2026-09-05",
        date_label: "2026年9月5日",
        title: "docs サイト検索インデックスのサイズ上限メモ",
        excerpt: "検索インデックスの決定性とサイズ上限の関係を整理したメモです。",
        tags: &["ドキュメント"],
        votes: 15,
        voted: false,
        status_label: "受付中",
        status_palette: ColorPalette::Info,
    },
];

/// 上向きシェブロンの装飾アイコン（`label: None`、モジュール doc
/// 「投票ボタンを 1 件だけ『投票済み』で固定する理由」節参照）。
fn vote_icon() -> Node {
    icon::icon(
        &IconProps {
            size: button::icon_size_for(Size::Sm),
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M6 15l6-6 6 6"),
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

/// 投票ボタン + 得票数の列。
fn vote_column(post: &Post) -> Node {
    let (variant, pressed) = if post.voted {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    // ラベルは投稿ごとに一意な固定文字列にする（同一ページ内で複数の
    // 「投稿に投票」だけのアクセシブル名が重複しないようにするため）。
    let label = format!("「{}」に投票", post.title);
    let button = button::icon_button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        &label,
        vec![
            ("aria-pressed", pressed),
            ("data-blocks-feed-upvote-cards-vote-button", ""),
        ],
        vec![vote_icon()],
    );
    div(
        vec![("class", "blocks-feed-upvote-cards-vote")],
        vec![
            button,
            span(
                vec![("class", "blocks-feed-upvote-cards-count")],
                vec![text(post.votes.to_string())],
            ),
        ],
    )
}

/// 投稿者アバター（イニシャル fallback）+ 氏名 + 日付 + 状態。
fn post_meta(post: &Post) -> Node {
    let name = dummy_assets::PERSON_NAMES[post.author_index];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-feed-upvote-cards-meta")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            span(
                vec![("class", "blocks-feed-upvote-cards-author")],
                vec![text(name)],
            ),
            el(
                "time",
                vec![
                    ("class", "blocks-feed-upvote-cards-date"),
                    ("datetime", post.date_iso),
                ],
                vec![text(post.date_label)],
            ),
            status::root(
                &StatusProps {
                    palette: post.status_palette,
                    ..StatusProps::default()
                },
                vec![("data-blocks-feed-upvote-cards-status", "")],
                vec![status::indicator(vec![]), text(post.status_label)],
            ),
        ],
    )
}

/// タグ列（`BadgeVariant::Subtle` 固定）。
fn tags_row(tags: &'static [&'static str]) -> Node {
    div(
        vec![("class", "blocks-feed-upvote-cards-tags")],
        tags.iter()
            .map(|tag| {
                badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(*tag)],
                )
            })
            .collect(),
    )
}

/// 投稿 1 件（投票列 + 本文）。
fn post_card(post: &Post) -> Node {
    li(
        vec![],
        vec![card::root(
            CardVariant::Outline,
            vec![("data-blocks-feed-upvote-cards-card", "")],
            vec![card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-feed-upvote-cards-post")],
                    vec![
                        vote_column(post),
                        div(
                            vec![("class", "blocks-feed-upvote-cards-main")],
                            vec![
                                post_meta(post),
                                card::title(vec![], vec![text(post.title)]),
                                card::description(vec![], vec![text(post.excerpt)]),
                                tags_row(post.tags),
                            ],
                        ),
                    ],
                )],
            )],
        )],
    )
}

/// `feed-upvote-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「ルート class」節）。
pub fn demo() -> Node {
    ul(
        vec![("class", "blocks-feed-upvote-cards-list")],
        POSTS.iter().map(post_card).collect(),
    )
}
```

## 原案差分メモ

参照（対応表 ID R0110 のみを主参照とし、出典の固有名・ファイル名は
記載しません）から取り込んだのは「投票列 + 投稿カード」という構造の
みであり、次の点を独自に設計・変更しています。

- 見出しレベルの節は追加せず、タイトル・抜粋は `card::title`/
  `card::description` のみで表現しています（使用部品を Issue 記載の
  6 件に保つため）。
- 投票は静的な固定表示にし、投稿の 1 件だけを「投票済み」
  （塗りつぶしボタン + `aria-pressed="true"`）として並記しています。
  実際の投票処理・状態遷移・件数の増減は実装していません。
- アバターは画像アセットに依存せず、氏名から機械的に導いたイニシャル
  の fallback 表示にしています。
- 状態表示（「受付中」/「解決済み」）は独自のバッジ文言・トークン
  （`status::root` の `ColorPalette::Info`/`ColorPalette::Success`）に
  揃えています。
- 投稿の氏名・日付・タイトル・抜粋・タグ・得票数はすべて独自に書いた
  架空のものです（実企業名・実データ・実在人物とは無関係です）。
- 狭い幅では投票列をカード上部の横並びに、`md`（768px）以上で投票列を
  右列へ固定する形にしています。
