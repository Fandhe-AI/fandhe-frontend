//! `blog-split-header-grid` block（イシュー #2814。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0419 の 1 件のみを構造の
//! 参照元とする合成例。見出し左 + 記事グリッド右）。取得手段・ファイル
//! 名・内部コンポーネント識別子は記載しない（`docs/design/motion-
//! reference-adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `button` / `card` / `image` / `link` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。`blog_list_image`/`blog_featured_with_list` が使う `avatar`・
//! `separator`・`link_overlay` は使わない（著者はテキスト表示のみ、記事
//! カード全面のクリック領域も設けない）。
//!
//! # レイアウト（lg = 64rem をブレークポイントとする理由）
//!
//! `< 64rem` は見出し列（tagline・見出し・説明・ボタン）の下にカード
//! グリッドが 1 列で積まれ、`>= 64rem` で左が見出し列（2fr）・右がカード
//! 2 列グリッド（3fr）の 2 カラムへ切り替える。テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できないため（CSS custom
//! property は宣言側でのみ有効）、`fandhe_frontend_pre_styled_ui::recipe::
//! Breakpoint` の `Lg`（1024px = 64rem）と一致するリテラル値を
//! [`LAYOUT_CSS`] へ直書きする（`blog_list_image`/`blog_featured_with_list`
//! と同じ判断）。ルート grid class は [`demo`] 直下の
//! `blocks-blog-split-header-grid-layout` へ適用し、[`Block::demo_class`]
//! （`blocks-blog-split-header-grid`）とは意図的に別名にする（同名にすると
//! Demo ラッパー自身が 1 トラック目に押し込まれる、`blog_featured_with_list`
//! モジュール doc「レイアウト」節と同じ Bugbot 教訓の回避）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない。
//! `linkcheck::check_links` は外部リンクを検証対象外とするため、記事
//! タイトルのリンク先はすべて [`REPO`]（実在する GitHub リポジトリへの
//! 外部絶対 URL）に固定する（`blog_list_image` と同じ先例判断）。
//!
//! # ボタンは遷移しない静的な操作要素
//!
//! 「すべての記事を見る」ボタンは [`fandhe_frontend_pre_styled_ui::button`]
//! を使い、`href` を持たない `<button type="button">` として組み立てる
//! （既定で暗黙 submit を起こさない）。一覧ページへの実際の遷移は行わず、
//! 静的な Demo 内の操作要素として置くのみである。
//!
//! # `<time datetime>` と表示日付の一致
//!
//! 各記事は機械可読な ISO 8601 日付（`datetime` 属性）と表示用の日本語
//! 表記を別々のフィールドとして持つが、常に同じ日を指す値を組にする
//! （`blog_list_image`/`blog_featured_with_list` と同じ不変条件）。
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
//! `heading::heading` / `styled_text::text` / `badge::badge` /
//! `button::button` / `card::root` / `image::image` / `link::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-blog-split-header-grid-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`time` と、`attrs` をそのまま
//! 連結する `card::cover`/`card::body` には `class` がそのまま効くため、
//! それらは `.blocks-blog-split-header-grid-*` クラスセレクタを使う。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、記事タイトルは `HeadingLevel::H4` にする
//! （`blog_list_image`/`blog_featured_article` と同じ判断）。`card::title`
//! は `<h3>` 固定のため使わず、`card::body` の中に `heading(H4, …)` を
//! 直接置く。
//!
//! # 参照元との差分（原稿「原案差分メモ」参照）
//!
//! 参照元のカードには著者が無いが、イシューの仕様が「著者」を明示的に
//! 要求するため追加した。カードのグリッドは参照元の 48rem からではなく
//! 64rem から 2 列にする（狭い幅では常に 1 列で積む、というイシューの
//! 仕様を優先した）。
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
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    image_src: &'static str,
    author_index: usize,
}

/// 記事一覧（架空、4 件。`author_index` は
/// [`dummy_assets::PERSON_NAMES`] への添字）。
const POSTS: [Post; 4] = [
    Post {
        date_iso: "2026-09-16",
        date_label: "2026年9月16日",
        category: "設計",
        title: "見出しと一覧を分ける 2 カラム構成の考え方",
        image_src: dummy_assets::SCREENSHOT_SRC,
        author_index: 0,
    },
    Post {
        date_iso: "2026-09-09",
        date_label: "2026年9月9日",
        category: "運用",
        title: "block の索引を長文追記なしで回す",
        image_src: dummy_assets::PRODUCT_SRC,
        author_index: 1,
    },
    Post {
        date_iso: "2026-09-01",
        date_label: "2026年9月1日",
        category: "アクセシビリティ",
        title: "装飾画像の alt を空にしてよい条件",
        image_src: dummy_assets::BACKGROUND_SRC,
        author_index: 2,
    },
    Post {
        date_iso: "2026-08-24",
        date_label: "2026年8月24日",
        category: "テスト",
        title: "合成例のグリッドをブレークポイント別に固定する",
        image_src: dummy_assets::LOGO_SRC,
        author_index: 3,
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-blog-split-header-grid-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// メタ行（日付 + カテゴリ badge）。
fn meta_row(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-split-header-grid-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-split-header-grid-category", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 記事カード 1 件（画像 + メタ行 + タイトルリンク + 著者）。
fn post_card(post: &Post) -> Node {
    let author = dummy_assets::PERSON_NAMES[post.author_index];
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-blog-split-header-grid-card", "")],
        vec![
            card::cover(
                vec![("class", "blocks-blog-split-header-grid-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Square,
                        ..ImageProps::new(post.image_src, "")
                    },
                    vec![("data-blocks-blog-split-header-grid-image", "")],
                )],
            ),
            card::body(
                vec![("class", "blocks-blog-split-header-grid-body")],
                vec![
                    meta_row(post),
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![link::root(
                            REPO,
                            &LinkProps::default(),
                            vec![("data-blocks-blog-split-header-grid-title-link", "")],
                            vec![text(post.title)],
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-blog-split-header-grid-author", "")],
                        vec![text(author)],
                    ),
                ],
            ),
        ],
    )
}

/// 左列（tagline + 見出し + 説明 + ボタン）。
fn lead_column() -> Node {
    div(
        vec![("class", "blocks-blog-split-header-grid-lead")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-blog-split-header-grid-tagline", "")],
                vec![text("開発ブログ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![("data-blocks-blog-split-header-grid-heading", "")],
                vec![text("最新の開発ノート")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "設計・運用・アクセシビリティに関する記事を、書きためた順に紹介しています。",
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-blog-split-header-grid-view-all", "")],
                vec![text("すべての記事を見る")],
            ),
        ],
    )
}

/// `blog-split-header-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-blog-split-header-grid-grid")],
        POSTS.iter().map(post_card).collect(),
    );

    div(
        vec![("class", "blocks-blog-split-header-grid-layout")],
        vec![lead_column(), grid],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-split-header-grid/",
    title: "blog-split-header-grid",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_split_header_grid.rs",
    demo_class: "blocks-blog-split-header-grid",
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
            label: "Button",
            path: "/themes/button/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `blog_split_header_grid` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-blog-split-header-grid-*` と
/// `[data-blocks-blog-split-header-grid-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`blog_list_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-blog-split-header-grid-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-12);\n  background: var(--fandhe-color-bg-muted);\n  padding: var(--fandhe-space-12) var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
.blocks-blog-split-header-grid-lead {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-6);\n  max-width: 32rem;\n}\n\
[data-blocks-blog-split-header-grid-tagline] {\n  align-self: flex-start;\n}\n\
[data-blocks-blog-split-header-grid-heading] {\n  margin: 0;\n}\n\
[data-blocks-blog-split-header-grid-view-all] {\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-blog-split-header-grid-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-blog-split-header-grid-card] {\n  overflow: hidden;\n  display: flex;\n  flex-direction: column;\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-blog-split-header-grid-cover {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-blog-split-header-grid-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-blog-split-header-grid-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-blog-split-header-grid-meta {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-split-header-grid-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-blog-split-header-grid-title-link] {\n  color: inherit;\n}\n\
[data-blocks-blog-split-header-grid-category] {\n  white-space: nowrap;\n}\n\
[data-blocks-blog-split-header-grid-author] {\n  margin: 0;\n}\n\
@media (min-width: 64rem) {\n  .blocks-blog-split-header-grid-layout {\n    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);\n    column-gap: var(--fandhe-space-12);\n    align-items: start;\n  }\n  .blocks-blog-split-header-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

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
            "data-scope=\"button\"",
            "data-scope=\"card\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("datetime=\"2026-09-16\""));
        assert_eq!(
            html.matches("data-blocks-blog-split-header-grid-card=\"\"")
                .count(),
            4
        );
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(" id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件と淡い背景の帯を
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_muted_band() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("--fandhe-color-bg-muted"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウト」節の Bugbot 教訓の
    /// 固定、`blog_list_image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-blog-split-header-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-blog-split-header-grid-layout"
        );
    }
}
