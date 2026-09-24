# blog-featured-article

`heading` / `text` / `badge` / `card` / `image` / `avatar` / `link` の 7 部品を
合成した特集記事ブロックです。新しい UI 部品は追加していません。

静的な表示例であり、記事データの取得・ルーティング・検索は行いません。
実際の記事一覧・詳細ページの実装は利用者側の Rust コードに委ねます
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。

リンク先はすべて実在の GitHub リポジトリへの外部 URL です（`Block::demo`
が `base_path` を受け取れない制約下での判断、`footer-newsletter` 等と同じ
方針）。画像・人名は Blocks 共通のダミー素材ヘルパ（`dummy_assets`、docs
サイト内部の非公開ヘルパ）による架空のプレースホルダーです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 記事リンク先（`Block::demo` が `base_path` を受け取れない制約下での
/// 実在リンク先。モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 下段グリッド用の小さい記事カード 1 件分のデータ（架空の内容）。
struct GridArticle {
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    author: &'static str,
}

const GRID_ARTICLES: [GridArticle; 3] = [
    GridArticle {
        category: "エンジニアリング",
        title: "レビュー待ち時間を半分にした小さな習慣",
        excerpt: "チームで続けている、レビュー依頼前の 3 分間チェックリストを紹介します。",
        author: "Elena Vasquez",
    },
    GridArticle {
        category: "プロダクト",
        title: "オンボーディングを 5 ステップへ削った理由",
        excerpt: "利用開始までの手数を減らすために捨てた機能と、その判断基準です。",
        author: "Kwame Boateng",
    },
    GridArticle {
        category: "運用",
        title: "障害対応メモを検索可能にする小さな工夫",
        excerpt: "当番が変わっても過去の対応が見つかるよう、記録の書き方を揃えました。",
        author: "Mei Lindqvist",
    },
];

/// メタ行（カテゴリ badge + 日付 + 区切り記号 + 読了時間）。
fn meta_row(category: &str, date: &str, read_minutes: u32) -> Node {
    div(
        vec![("class", "blocks-blog-featured-article-meta")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-featured-article-category", "")],
                vec![text(category)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(date)],
            ),
            span(vec![("aria-hidden", "true")], vec![text("\u{00B7}")]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("{read_minutes} 分で読了"))],
            ),
        ],
    )
}

/// 著者行（fallback イニシャルの avatar + 氏名 + 肩書）。
fn byline(name: &str, role: &str) -> Node {
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-blog-featured-article-byline")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![("data-blocks-blog-featured-article-avatar", "")],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// 特集記事本体（横長画像 + 本文カラム）。基本形・バリエーションの両方が
/// 共用する（モジュール doc「2 バリエーションを並記する理由」節参照）。
fn featured_article() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-blog-featured-article-feature", "")],
        vec![div(
            vec![("class", "blocks-blog-featured-article-feature-layout")],
            vec![
                card::cover(
                    vec![],
                    vec![image::image(
                        &ImageProps {
                            aspect_ratio: AspectRatio::Video,
                            ..ImageProps::new(
                                dummy_assets::SCREENSHOT_SRC,
                                "特集記事のプレースホルダー画像",
                            )
                        },
                        vec![],
                    )],
                ),
                card::body(
                    vec![("class", "blocks-blog-featured-article-feature-body")],
                    vec![
                        meta_row("エンジニアリング", "2026-09-20", 6),
                        heading::heading(
                            HeadingLevel::H3,
                            &HeadingProps {
                                size: HeadingSize::Xl2,
                                ..HeadingProps::default()
                            },
                            vec![],
                            vec![text("レビュー待ち時間を半分にした、チームの小さな習慣")],
                        ),
                        styled_text::text(
                            &TextProps::default(),
                            vec![],
                            vec![text(
                                "導入から一週間でレビュー待ちが目に見えて減った、\
                                 私たちのチームが続けている運用ルールをまとめました。",
                            )],
                        ),
                        byline("Haruto Fujimaki", "Engineering Lead"),
                        link::root(
                            REPO,
                            &LinkProps {
                                external: true,
                                ..LinkProps::default()
                            },
                            vec![],
                            vec![text("記事を読む")],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 下段グリッド用の小さい記事カード 1 件。
fn article_card(item: &GridArticle) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &ImageProps::new(dummy_assets::PRODUCT_SRC, "記事のプレースホルダー画像"),
                    vec![],
                )],
            ),
            card::body(
                vec![],
                vec![
                    badge::badge(
                        &BadgeProps::default(),
                        vec![("data-blocks-blog-featured-article-category", "")],
                        vec![text(item.category)],
                    ),
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.excerpt)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.author)],
                    ),
                ],
            ),
        ],
    )
}

/// バリエーション（塗り帯 + 重ねた特集記事 + 下段 3 列グリッド）。
fn banded_variant() -> Node {
    let grid_cards: Vec<Node> = GRID_ARTICLES.iter().map(article_card).collect();
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-blog-featured-article-band")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("特集")],
                    ),
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("今週のおすすめ記事")],
                    ),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(
                            "チームの運用ノウハウをまとめた記事から、注目の 1 本を選びました。",
                        )],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-blog-featured-article-overlap")],
                vec![featured_article()],
            ),
            div(
                vec![("class", "blocks-blog-featured-article-grid")],
                grid_cards,
            ),
        ],
    )
}

/// `blog-featured-article` の Demo 本体（基本形とバリエーションの
/// 2 インスタンス併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-blog-featured-article")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Featured article")],
            ),
            featured_article(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Featured on band + grid")],
            ),
            banded_variant(),
        ],
    )
}
```

## 差分メモ

- 基本形（横長画像 + 本文カラム。メタ行 → 見出し → 説明文 → 著者行 →
  記事リンクの縦積み）と、上部に塗り帯を敷いてその上に特集記事を重ね、
  下段へ通常の記事カード 3 件をグリッドで続けるバリエーションの、2 通りの
  レイアウトを持ちます。
- 無 JS の docs サイトでは片方を選んで切り替える操作を実演できないため、
  基本形とバリエーションの 2 インスタンスを Demo 内に縦に並べて掲示します
  （`footer-newsletter`/`pricing-tiers-morph` と同じ 2 インスタンス併記の
  判断）。
- バリエーションの塗り帯・重なり・3 列グリッドはいずれもレイアウト用の
  生 CSS（`--fandhe-space-*`/`--fandhe-color-accent` 等のトークン参照）で
  表現し、新しい UI 部品・data-* 語彙は追加していません。
