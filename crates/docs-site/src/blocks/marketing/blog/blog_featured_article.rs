//! `blog-featured-article` block（イシュー #2808。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」・Phase 2「マーケティング B」配下）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `image` / `avatar` / `link` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（購入者限定素材のライセンス上の転記制限、
//! `docs/design/motion-reference-adoption-policy.md` §9 と同じ方針）。
//!
//! # 2 バリエーションを並記する理由（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 基本形（横長画像 + 本文カラム）と、塗り帯へ重ねた特集記事 + 下段 3 列
//! グリッドのバリエーションを、`footer-newsletter`/`pricing-tiers-morph` と
//! 同型の**2 インスタンス併記**で示す（片方を選んで JS で切り替える機構は
//! 持たない）。
//!
//! # `drop_class_attr` を考慮した CSS フックの選び方
//!
//! `card::root` / `badge::badge` / `heading::heading` / `text::text`
//! （`styled_text` として import、後述）/ `image::image` /
//! `link::root` / `avatar::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去してから合成する契約を持つ
//! ため、Demo 固有スタイルは `data-blocks-blog-featured-article-*` 属性で
//! 渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。`card::body`/
//! `card::cover` と素の `div` には `class` がそのまま効くため、それらは
//! `class` で渡す。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れないため、
//! 内部パスへのリンクは作れない（`linkcheck::check_links` の fail-closed
//! 検証を満たしつつ実在の遷移先を示す必要がある）。`footer-newsletter`
//! 等の前例と同じく、外部の絶対 URL `https://github.com/Fandhe-AI/
//! fandhe-frontend` を記事リンク先として使う。
//!
//! # スケール外の rem 値を直書きする理由
//!
//! 帯へ特集記事カードを重ねる負のマージン（[`LAYOUT_CSS`] 参照）は、
//! 既存の `--fandhe-space-*` トークンの刻みに一致する値が無いため、
//! `testimonials_stack` の積層オフセット・`pricing_tiers_morph` の重なり
//! 演出と同じ判断でリテラル rem 値を直書きする。
//!
//! # 見出しレベル（h3/h4）の理由
//!
//! Demo 内に `<h1>` を置くとページ本文の `<h1>`（Markdown 原稿の `# `）と
//! 重複するため、特集記事の見出しは `<h3>`、グリッドカードの見出しは
//! `<h4>` にする（他の Blocks 実装と同じ判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                vec![("class", "blocks-blog-featured-article-grid-card-body")],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-featured-article/",
    title: "blog-featured-article",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_featured_article.rs",
    demo_class: "blocks-blog-featured-article",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `blog_featured_article` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。[`BLOCK`] の `layout_css`
/// （`crate::blocks::LayoutCss::Static`）として自己申告し、
/// [`crate::blocks::stylesheet`] が [`crate::blocks::all_blocks`] を
/// 走査して連結する）。
const LAYOUT_CSS: &str = "\
.blocks-blog-featured-article {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
.blocks-blog-featured-article > p:not(:first-child) {\n  margin-top: 1.5rem;\n}\n\
.blocks-blog-featured-article-feature-layout {\n  display: grid;\n  grid-template-columns: 1fr 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-blog-featured-article-feature-layout {\n    grid-template-columns: 1fr;\n  }\n}\n\
.blocks-blog-featured-article-feature-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-blog-featured-article-grid-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-article-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-article-byline {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-article-band {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-12) var(--fandhe-space-6) 4rem;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-article-overlap {\n  position: relative;\n  margin-top: -3rem;\n  padding-inline: var(--fandhe-space-4);\n}\n\
.blocks-blog-featured-article-grid {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: var(--fandhe-space-4);\n  margin-top: var(--fandhe-space-6);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-blog-featured-article-grid {\n    grid-template-columns: 1fr;\n  }\n}\n\
[data-blocks-blog-featured-article-feature] {\n  overflow: hidden;\n}\n\
[data-blocks-blog-featured-article-category] {\n  flex-shrink: 0;\n}\n\
[data-blocks-blog-featured-article-avatar] {\n  flex-shrink: 0;\n}\n";
