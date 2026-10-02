//! `help-center-collection-grid` block（イシュー #2975。Application /
//! Help Center カテゴリ）。パンくず + 見出しの下に、ヘルプ
//! 記事コレクションをカードのグリッドで並べる合成例。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す（`profile_detail_datalist.rs`
//! 〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `breadcrumb` / `heading` / `text` / `card` / `icon` / `stat` / `avatar` /
//! `link` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0116（主参照）+ R0117（細部差のみ、Demo 上は
//!   分離しない）。輪郭カード（[`fandhe_frontend_pre_styled_ui::card::CardVariant::Outline`]）
//!   6 枚をグリッドで並べ、各カードはアイコン・題名（リンク化）・説明・
//!   記事数（`stat`）を持つ
//! - **B（著者アバター群 + 浮き上がりカード + ヘッダー）**: R0118（著者
//!   アバター群）+ R0119（浮き上がりカード + ヘッダー）。浮き上がりカード
//!   （[`fandhe_frontend_pre_styled_ui::card::CardVariant::Elevated`]）3 枚
//!   + ヘッダー行に「すべて見る」リンクを併記する
//!
//! # グリッド列数を `@container` で切り替える理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-help-center-collection-grid-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅 `48rem` 未満で 2 列、`32rem` 未満で
//! 1 列へ切り替える。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::avatar::group`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`]・
//! [`fandhe_frontend_pre_styled_ui::stat::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-help-center-collection-grid-*`）。ラッパー
//! （スタック・見出し行・グリッド・アイコン枠）は素の `<div>` のため
//! `class="blocks-help-center-collection-grid-*"` を使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # アイコンは自作の単純図形
//!
//! `icon::icon` + `el("path", ...)` による線画（`stroke="currentColor"`,
//! `fill="none"`）のみで構成する（`profile_detail_datalist` と同型の
//! 判断）。各カードの題名リンクが可視テキストを持つため、アイコン自体は
//! 装飾用途（`IconProps::default()` の `label: None` → `aria-hidden`）とし、
//! アクセシブルネームの供給は求めない。
//!
//! # `avatar::group` を `aria-hidden` にする理由
//!
//! 版 B のフッター著者アバター群は記事数（`stat`）と併記する装飾情報で
//! あり、特定個人を示す意味を持たせないため `aria-hidden="true"` を付与
//! する（`card_media_footer.rs::members_footer` と同型の判断）。
//!
//! # `href="#"` は使わない（`blog_grid_image.rs`「href の方針」節と同型）
//!
//! 題名リンク・「View all」リンクはいずれもサイト内に実在する索引ページ
//! （`/guides/<slug>/`・`/api/`・`/guides/`）への相対パスを使う。注目
//! コレクションはすべて `/guides/` 配下のため、「View all」は表示文言と
//! 遷移先を一致させて `/guides/` 索引へ向ける
//! （`linkcheck::check_links` が `crates/docs-site/tests/support/shared_site.rs`
//! 経由で fail-closed に検証する）。`external: true` は付けない
//! （reverse tabnabbing 面を持たない）。
//!
//! # ダミー素材について
//!
//! コレクション名・説明文は独自の英語ダミー文言（実企業名・実クレデン
//! シャル・PII は含まない）。著者アバターの `alt` は
//! `crate::blocks::dummy_assets::PERSON_NAMES` を使う。アバター画像は
//! ビルド時生成の同梱 SVG（[`dummy_assets::AVATAR_SRC`]）を使う（外部
//! URL・`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/help-center-collection-grid/",
    title: "help-center-collection-grid",
    category: BlockCategory::HelpCenter,
    rust_source:
        "crates/docs-site/src/blocks/application/help_center/help_center_collection_grid.rs",
    demo_class: "blocks-help-center-collection-grid",
    parts: &[
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
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

/// `help_center_collection_grid` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-help-center-collection-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-help-center-collection-grid;\n}\n\
.blocks-help-center-collection-grid-top-row {\n  display: flex;\n}\n\
.blocks-help-center-collection-grid-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-help-center-collection-grid-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-end;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-help-center-collection-grid-grid {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-help-center-collection-grid-icon {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: var(--fandhe-space-10);\n  height: var(--fandhe-space-10);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-subtle);\n  margin-block-end: var(--fandhe-space-3);\n}\n\
[data-blocks-help-center-collection-grid-footer] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
@container blocks-help-center-collection-grid (max-width: 48rem) {\n  \
.blocks-help-center-collection-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-help-center-collection-grid (max-width: 32rem) {\n  \
.blocks-help-center-collection-grid-grid {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"breadcrumb\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"stat\"",
            "data-scope=\"avatar\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn grid_appears_twice_and_cards_total_nine() {
        let html = demo_html();
        assert_eq!(
            html.matches("blocks-help-center-collection-grid-grid")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-help-center-collection-grid-card")
                .count(),
            9
        );
    }

    #[test]
    fn author_avatar_group_is_aria_hidden() {
        let html = demo_html();
        assert!(html
            .contains(r#"data-blocks-help-center-collection-grid-authors="" aria-hidden="true""#));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("target=\"_blank\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_switches_columns_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-help-center-collection-grid (max-width: 48rem)")
        );
        assert!(
            LAYOUT_CSS.contains("@container blocks-help-center-collection-grid (max-width: 32rem)")
        );
    }

    #[test]
    fn breadcrumb_has_accessible_label_and_current_page() {
        let html = demo_html();
        assert!(html.contains(r#"aria-label="Breadcrumb""#));
        assert!(html.contains("Collections"));
    }
}
