//! `blog-featured-with-list` block（イシュー #2809。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0777 の 1 件のみを構造の
//! 参照元とする合成例。特集記事 1 件 + 通常記事 2 件のリストを 2 カラムで
//! 並べるレイアウト）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `avatar`（イニシャル fallback のみ）/ `separator` /
//! `link` / `link-overlay` の 6 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウト（lg = 64rem をブレークポイントとする理由）
//!
//! `>= 64rem` で 2 カラム（左: 特集記事、右: 罫線区切りの記事リスト
//! 2 件）、`< 64rem` は特集記事の下へ記事リストを縦に積む 1 列表示へ
//! 切り替える。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できないため（CSS custom property は宣言側でのみ有効）、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Lg`（1024px = 64rem）と
//! 一致するリテラル値を [`LAYOUT_CSS`] へ直書きする。
//!
//! # 記事リンクの入れ子を避ける 2 段構成（`link_overlay` + 著者リンク）
//!
//! `link_overlay::overlay` は `z-index: 0` で記事カード全面に重なる
//! （`fandhe_frontend_pre_styled_ui::link_overlay` モジュール doc「全面拡張
//! の CSS 実装」節）。著者リンクを `overlay` と同じ `link_overlay::root` の
//! 内側に置くと `overlay` の下に隠れてクリックできなくなるため、記事
//! ラッパー（`[data-blocks-blog-featured-with-list-article]`）の直下へ
//! 「`link_overlay::root`（日付・見出し・抜粋・overlay）」と「著者リンク」
//! を**兄弟**として並べる（`overlay` の外に置く著者リンクだけが独立して
//! クリック可能なままになる）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない。
//! `linkcheck::check_links` は外部リンクを検証対象外とするため、記事・
//! 著者・「続きを読む」のリンク先はすべて [`REPO`]（実在する GitHub
//! リポジトリへの外部絶対 URL）に固定する（`footer_sticky_reveal`/
//! `footer_newsletter` と同じ先例判断）。死リンク `href="#"` は使わない。
//!
//! # `<time datetime>` と表示日付の一致
//!
//! 各記事は機械可読な ISO 8601 日付（`datetime` 属性）と表示用の日本語
//! 表記を別々の定数として持つが、常に同じ日を指す値を組にする（原案
//! 差分メモ「参照側の日付機械可読値と表示値の食い違いを持ち込まない」
//! 節参照）。
//!
//! # アバターをイニシャル fallback にする理由
//!
//! `dummy_assets::AVATAR_SRC` 等の画像アセットへ依存せず最小構成にする
//! ため、`avatar::fallback(ImageStatus::Error, …)` のみを使う
//! （`testimonials_stack` と同じ判断）。
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
//! `heading::heading` / `link::root` / `avatar::root` /
//! `separator::separator` / `styled_text::text` / `link_overlay::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-blog-featured-with-list-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節参照）。素の `div`/`time` には
//! `class` がそのまま効くため、それらは従来どおり `.blocks-blog-featured-
//! with-list-*` クラスセレクタを使う。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、block 内の見出しは
//! `HeadingLevel::H3` を使う（他 block と同じ先例）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・人名はすべて架空のもの（実企業名・実クレデンシャル・PII
//! を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, footer, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    excerpt: &'static str,
    author_name: &'static str,
    author_initials: &'static str,
}

/// 特集記事（架空）。
const FEATURED: Post = Post {
    date_iso: "2026-09-20",
    date_label: "2026年9月20日",
    title: "既定エスケープだけで守れる範囲を広げる",
    excerpt: "テキスト補間を必ずエスケープ経由にする設計判断が、レビューの負荷をどう下げたかをまとめました。",
    author_name: "遠藤 佑奈",
    author_initials: "EY",
};

/// 通常記事リスト（架空、2 件）。
const LIST: [Post; 2] = [
    Post {
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "単一バイナリ配布までの最短ルート",
        excerpt: "SSR から単一実行ファイルへ至る構成を、最小手順で振り返ります。",
        author_name: "冨田 千夏",
        author_initials: "TC",
    },
    Post {
        date_iso: "2026-09-05",
        date_label: "2026年9月5日",
        title: "ノード木 API で組み立てる合成例",
        excerpt: "HTML 文字列を直接組み立てず既存部品を合成するときの考え方を紹介します。",
        author_name: "宮下 大和",
        author_initials: "MY",
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-blog-featured-with-list-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// 著者リンク（アバターのイニシャル fallback + 氏名）。`overlay` の外へ
/// 兄弟として置くことでクリック可能なまま保つ（モジュール doc「記事
/// リンクの入れ子を避ける 2 段構成」節参照）。
fn author(name: &str, initials: &str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-blog-featured-with-list-author", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xs,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            text(name),
        ],
    )
}

/// 通常記事リストの 1 件（日付・見出し・抜粋 + 全面クリック用 `overlay`。
/// 著者は `overlay` の外の兄弟としてリンクを保つ）。
fn list_article(post: &Post) -> Node {
    div(
        vec![("data-blocks-blog-featured-with-list-article", "")],
        vec![
            link_overlay::root(
                vec![],
                vec![
                    post_date(post.date_iso, post.date_label),
                    heading(
                        HeadingLevel::H3,
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
            author(post.author_name, post.author_initials),
        ],
    )
}

/// 特集記事フッタ（「続きを読む」リンク + 著者）。`aria-label` は
/// [`FEATURED`] の見出しから `format!` で導出し、文言の複製によるドリフト
/// を避ける。
fn read_more_footer() -> Node {
    let aria_label = format!("続きを読む: {}", FEATURED.title);
    footer(
        vec![("class", "blocks-blog-featured-with-list-featured-footer")],
        vec![
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("aria-label", aria_label.as_str())],
                vec![
                    text("続きを読む"),
                    el("span", vec![("aria-hidden", "true")], vec![text("→")]),
                ],
            ),
            author(FEATURED.author_name, FEATURED.author_initials),
        ],
    )
}

/// `blog-featured-with-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let featured = div(
        vec![("class", "blocks-blog-featured-with-list-featured")],
        vec![
            post_date(FEATURED.date_iso, FEATURED.date_label),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(FEATURED.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(FEATURED.excerpt)],
            ),
            read_more_footer(),
        ],
    );

    let list = div(
        vec![("class", "blocks-blog-featured-with-list-list")],
        vec![
            list_article(&LIST[0]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-blog-featured-with-list-separator", "")],
            ),
            list_article(&LIST[1]),
        ],
    );

    div(
        vec![("class", "blocks-blog-featured-with-list")],
        vec![featured, list],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-featured-with-list/",
    title: "blog-featured-with-list",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_featured_with_list.rs",
    demo_class: "blocks-blog-featured-with-list",
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

/// `blog_featured_with_list` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-blog-featured-with-list*` と
/// `[data-blocks-blog-featured-with-list-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`testimonials_stack` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-blog-featured-with-list {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-blog-featured-with-list-featured {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-blog-featured-with-list-featured-footer {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-with-list-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  border-block-start: 1px solid var(--fandhe-color-border);\n  padding-block-start: var(--fandhe-space-8);\n}\n\
[data-blocks-blog-featured-with-list-article] {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-featured-with-list-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-blog-featured-with-list-author] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  position: relative;\n  z-index: 1;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-blog-featured-with-list-separator] {\n  margin: 0;\n}\n\
@media (min-width: 64rem) {\n  .blocks-blog-featured-with-list {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-8);\n  }\n  .blocks-blog-featured-with-list-list {\n    border-inline-start: 1px solid var(--fandhe-color-border);\n    border-block-start: none;\n    padding-inline-start: var(--fandhe-space-8);\n    padding-block-start: 0;\n  }\n}\n";

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
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-scope="separator""#));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains("datetime=\"2026-09-20\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }
}
