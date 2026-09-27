//! `feed-upvote-cards` block（イシュー #2910。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、対応表 ID R0110 の 1 件のみを構造の
//! 参照元とする合成例。投票数付き投稿カードのフィード）。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `badge` / `button` / `status` / `icon` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `heading`/`text` は追加せず、タイトル・抜粋は `card::title`/
//! `card::description` で代替する。
//!
//! # レイアウト（狭い幅は投票列が上部横並び、`md` 以上で右列固定）
//!
//! `< 48rem`（`md` ブレークポイント未満）は投票ボタン + 得票数の列が
//! カード上部で横並びになり、本文が縦に続く。`>= 48rem` で
//! `display: grid; grid-template-columns: minmax(0, 1fr) auto` に切り替え、
//! 投票列を `grid-column: 2` へ固定して視覚上の右列にする（DOM 順は投票列が
//! 先のまま変えず、`grid-row`/`grid-column` の指定のみで視覚順を入れ替える。
//! `footer_inline_nav` 等の既存 block と同じ判断で `48rem` はテーマの
//! `Breakpoint::Md`（768px）と一致するリテラル値を直書きする）。
//!
//! # 投票ボタンを 1 件だけ「投票済み」で固定する理由
//!
//! 本 Demo は無 JS の静的表示であり、投票の送信・状態遷移は行わない。
//! `feature_accordion_image::category_button` と同じ判断で、投稿の 1 件を
//! `ButtonVariant::Solid` + `aria-pressed="true"` に固定し、残りを
//! `ButtonVariant::Outline` + `aria-pressed="false"` に固定することで、
//! 「通常」と「投票済み」の 2 状態を無 JS のままレイアウトへ並記する。
//! `disabled` は付けない（`app_shell_sidebar_header::notify_button` と同じ
//! 判断で、クリックしても何も起きない静的ボタンだが操作自体は妨げない）。
//! 得票数の `span` はボタンの `aria-label` から独立した視覚表示用であり、
//! 通常のテキストノードとしてスクリーンリーダーにもそのまま読み上げられる
//! （`aria-hidden` は付与しない）。
//!
//! # `status` の使用（本 crate 内の Blocks で初めて `status::root` を使う）
//!
//! 投稿の状態（「受付中」/「解決済み」）を [`fandhe_frontend_pre_styled_ui::status`]
//! の `root`/`indicator` で表す。`status` はレンダリング時点の静的な状態
//! 表示専用で `role="status"`（live region）を持たないため、本 Demo の
//! 「初期状態のまま固定される静的表示」という性質と一致する。
//!
//! # アバターをイニシャル fallback にする理由
//!
//! `dummy_assets::AVATAR_SRC` 等の画像アセットへ依存せず最小構成にする
//! ため、`avatar::fallback(ImageStatus::Error, …)` のみを使う
//! （`blog_list_image`/`testimonials_stack` と同じ判断）。イニシャルは
//! 氏名から機械的に導く。
//!
//! # `role="feed"` を付けない理由
//!
//! WAI-ARIA の `feed` ロールは動的な追加読み込み（`aria-busy`・
//! `aria-posinset`/`aria-setsize` 等）を伴うスクロール可能な記事流を想定した
//! ロールであり、本 Demo は件数固定・追加読み込みなしの静的なリストである。
//! 意味論の誤伝達を避けるため素の `ul`/`li` のみで構成する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root` / `avatar::root` / `badge::badge` / `button::icon_button` /
//! `status::root` / `icon::icon` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-feed-upvote-cards-*` 属性で渡し、[`LAYOUT_CSS`] 側
//! も同じ属性セレクタで対応する。素の `ul`/`li`/`div`/`span`/`time` には
//! `class` がそのまま効くため、それらは従来どおり
//! `.blocks-feed-upvote-cards-*` クラスセレクタを使う。
//!
//! # ルート class（`demo_class` との別名、Bugbot 教訓の踏襲）
//!
//! [`demo`] が返す `<ul>` は `blocks-feed-upvote-cards-list` を持ち、
//! [`Block::demo_class`]（`blocks-feed-upvote-cards`）とは意図的に別名に
//! する（`blog_list_image`/`team_avatar_grid` 等と同じ回避策）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・送信処理・状態機械を持たない
//! 静的な合成例である。文言・人名はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feed-upvote-cards/",
    title: "feed-upvote-cards",
    category: BlockCategory::Feed,
    rust_source: "crates/docs-site/src/blocks/application/feed/feed_upvote_cards.rs",
    demo_class: "blocks-feed-upvote-cards",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feed_upvote_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-feed-upvote-cards-*` と
/// `[data-blocks-feed-upvote-cards-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（`blog_list_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feed-upvote-cards-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-feed-upvote-cards-post {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-feed-upvote-cards-vote {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feed-upvote-cards-count {\n  font-weight: var(--fandhe-font-weight-semibold, 600);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-feed-upvote-cards-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-feed-upvote-cards-meta {\n  display: inline-flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feed-upvote-cards-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-feed-upvote-cards-tags {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 48rem) {\n  .blocks-feed-upvote-cards-post {\n    display: grid;\n    grid-template-columns: minmax(0, 1fr) auto;\n    align-items: start;\n    gap: var(--fandhe-space-6);\n  }\n  .blocks-feed-upvote-cards-vote {\n    grid-column: 2;\n    grid-row: 1;\n    flex-direction: column;\n  }\n  .blocks-feed-upvote-cards-main {\n    grid-column: 1;\n    grid-row: 1;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_static_vote_state() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"status\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<li").count(), 4);
        assert!(html.contains("type=\"button\""));
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1);
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 3);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_grid_switch() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class」節の Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feed-upvote-cards-list\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-feed-upvote-cards-list");
    }
}
