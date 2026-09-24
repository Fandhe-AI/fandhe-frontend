//! `blog-list-image` block（イシュー #2812。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0776 の 1 件のみを構造の
//! 参照元とする合成例。見出し + 画像横並びの記事リスト）。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `image` / `avatar` / `separator` /
//! `link` / `link-overlay` の 8 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウト（lg = 64rem をブレークポイントとする理由）
//!
//! `< 64rem` は画像が上・本文が下の縦積み、`>= 64rem` で画像が左・本文が
//! 右の横並びへ切り替える。テーマの breakpoint トークンは `@media` 条件式の
//! 中では解決できないため（CSS custom property は宣言側でのみ有効）、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`blog_featured_with_list` と同じ判断）。ルート grid class は [`demo`]
//! 直下の `blocks-blog-list-image-layout` へ適用し、[`Block::demo_class`]
//! （`blocks-blog-list-image`）とは意図的に別名にする（`blog_featured_
//! with_list` モジュール doc「レイアウト」節と同じ Bugbot 教訓の回避）。
//!
//! # 記事リンクの入れ子を避ける 2 段構成（`link_overlay` + 著者リンク）
//!
//! `link_overlay::overlay` は `z-index: 0` で記事本文全面に重なるため、
//! 著者リンクを `overlay` と同じ `link_overlay::root` の内側に置くと
//! `overlay` の下に隠れてクリックできなくなる。そのため記事本文
//! （`[data-blocks-blog-list-image-body]`）の直下へ「`link_overlay::root`
//! （メタ行・見出し・抜粋・overlay）」と「区切り線 + 著者行」を**兄弟**
//! として並べ、著者リンクには `position: relative; z-index: 1` を付けて
//! overlay より前面に出す（`blog_featured_with_list` と同じ 2 段構成）。
//! 画像はクリック範囲に含めない（`alt=""` の装飾扱いのため、クリック範囲
//! から外れても情報は失われない）。カテゴリは `badge::badge`（リンクでは
//! ない）にする。overlay の内側に 3 つ目のクリック可能要素を重ねない
//! ためである（参照元はカテゴリがリンクだったが構造のみ取り込む）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない。
//! `linkcheck::check_links` は外部リンクを検証対象外とするため、記事・
//! 著者のリンク先はすべて [`REPO`]（実在する GitHub リポジトリへの外部
//! 絶対 URL）に固定する（`blog_featured_with_list` と同じ先例判断）。
//!
//! # `<time datetime>` と表示日付の一致
//!
//! 各記事は機械可読な ISO 8601 日付（`datetime` 属性）と表示用の日本語
//! 表記を別々のフィールドとして持つが、常に同じ日を指す値を組にする
//! （`blog_featured_with_list` と同じ不変条件）。
//!
//! # アバターをイニシャル fallback にする理由
//!
//! `dummy_assets::AVATAR_SRC` 等の画像アセットへ依存せず最小構成にする
//! ため、`avatar::fallback(ImageStatus::Error, …)` のみを使う
//! （`blog_featured_with_list`/`testimonials_stack` と同じ判断）。イニシャル
//! は氏名から機械的に導く（`blog_featured_article::byline` と同型）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `image::image` /
//! `avatar::root` / `separator::separator` / `link::root` /
//! `link_overlay::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-blog-list-image-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`article`/`time` には `class` が
//! そのまま効くため、それらは従来どおり `.blocks-blog-list-image-*`
//! クラスセレクタを使う。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、記事タイトルは `HeadingLevel::H4` にする
//! （`blog_featured_article` と同じ判断）。
//!
//! # 正方形画像（参照元との差分）
//!
//! 参照元は狭い幅で 16:9、広い幅で 2:1 だったが、本実装は狭い幅・広い幅
//! いずれでも正方形（[`AspectRatio::Square`]）を保つ（原稿「原案差分メモ」
//! 参照）。狭い幅で画像が巨大化しないよう `max-width` の上限を付ける。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・人名はすべて架空のもの（実企業名・実クレデンシャル・PII
//! を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{article, div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    image_src: &'static str,
    author_index: usize,
}

/// 記事一覧（架空、3 件）。`author_index` は
/// [`dummy_assets::PERSON_NAMES`]/[`dummy_assets::JOB_TITLES`] への添字。
const POSTS: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        category: "設計",
        title: "ノード木 API で HTML 文字列組み立てを避ける",
        excerpt: "既定エスケープを弱めずに合成例を増やすための、部品合成の考え方をまとめました。",
        image_src: dummy_assets::SCREENSHOT_SRC,
        author_index: 0,
    },
    Post {
        date_iso: "2026-09-10",
        date_label: "2026年9月10日",
        category: "運用",
        title: "block 追加を長文追記なしで回せるようにした話",
        excerpt: "ドキュメントへの逐次追記をやめ、レジストリと原稿を正にした運用の振り返りです。",
        image_src: dummy_assets::PRODUCT_SRC,
        author_index: 1,
    },
    Post {
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        category: "アクセシビリティ",
        title: "画像スロットの alt を空にしてよい条件",
        excerpt: "装飾目的の画像とそうでない画像を切り分ける、実務上の判断基準を紹介します。",
        image_src: dummy_assets::BACKGROUND_SRC,
        author_index: 2,
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", "blocks-blog-list-image-date"), ("datetime", iso)],
        vec![text(label)],
    )
}

/// メタ行（カテゴリ badge + 日付）。
fn meta_row(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-list-image-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-list-image-category", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 著者行（アバターのイニシャル fallback + 氏名 + 肩書）。`overlay` の外へ
/// 兄弟として置くことでクリック可能なまま保つ（モジュール doc「記事
/// リンクの入れ子を避ける 2 段構成」節参照）。
fn author_row(name: &str, role: &str) -> Node {
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("data-blocks-blog-list-image-author", "")],
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
            div(
                vec![],
                vec![
                    link::root(
                        REPO,
                        &LinkProps::default(),
                        vec![("data-blocks-blog-list-image-author-link", "")],
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

/// 記事 1 件（正方形画像 + 本文。本文上部はメタ行・見出し・抜粋 +
/// 全面クリック用 `overlay`、下端に区切り線 + 著者行）。
fn post_item(post: &Post) -> Node {
    let author = dummy_assets::PERSON_NAMES[post.author_index];
    let role = dummy_assets::JOB_TITLES[post.author_index % dummy_assets::JOB_TITLES.len()];
    article(
        vec![("class", "blocks-blog-list-image-article")],
        vec![
            div(
                vec![("class", "blocks-blog-list-image-figure")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Square,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(post.image_src, "")
                    },
                    vec![("data-blocks-blog-list-image-image", "")],
                )],
            ),
            div(
                vec![("class", "blocks-blog-list-image-body")],
                vec![
                    link_overlay::root(
                        vec![("data-blocks-blog-list-image-main", "")],
                        vec![
                            meta_row(post),
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(post.title)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(post.excerpt)],
                            ),
                            overlay(REPO, vec![("aria-label", post.title)], vec![]),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-blog-list-image-footer")],
                        vec![
                            separator(
                                &SeparatorProps::default(),
                                vec![("data-blocks-blog-list-image-separator", "")],
                            ),
                            author_row(author, role),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `blog-list-image` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-blog-list-image-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("開発ブログ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "設計・運用・アクセシビリティに関する記事を掲載しています。",
                )],
            ),
        ],
    );

    let list = div(
        vec![("class", "blocks-blog-list-image-list")],
        POSTS.iter().map(post_item).collect(),
    );

    div(
        vec![("class", "blocks-blog-list-image-layout")],
        vec![header, list],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-list-image/",
    title: "blog-list-image",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_list_image.rs",
    demo_class: "blocks-blog-list-image",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `blog_list_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-blog-list-image-*` と
/// `[data-blocks-blog-list-image-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（`blog_featured_with_list` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-blog-list-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-blog-list-image-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-list-image-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-blog-list-image-article {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-blog-list-image-figure {\n  width: 100%;\n  max-width: 16rem;\n}\n\
[data-blocks-blog-list-image-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-blog-list-image-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-blocks-blog-list-image-main] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-blog-list-image-meta {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-blog-list-image-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-blog-list-image-footer {\n  margin-top: auto;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-blog-list-image-separator] {\n  margin: 0;\n}\n\
[data-blocks-blog-list-image-author] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-blog-list-image-author-link] {\n  position: relative;\n  z-index: 1;\n}\n\
@media (min-width: 64rem) {\n  .blocks-blog-list-image-article {\n    flex-direction: row;\n    gap: var(--fandhe-space-8);\n  }\n  .blocks-blog-list-image-figure {\n    flex: 0 0 16rem;\n    max-width: 16rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"image\"",
            "data-scope=\"avatar\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("datetime=\"2026-09-18\""));
        assert!(html.contains("data-blocks-blog-list-image-author-link"));
        assert_eq!(html.matches("<article").count(), 3);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_row_switch() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("flex-direction: row"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウト」節の Bugbot 教訓の
    /// 固定、`blog_featured_with_list` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-blog-list-image-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-blog-list-image-layout");
    }
}
