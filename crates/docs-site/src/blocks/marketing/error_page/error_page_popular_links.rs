//! `error-page-popular-links` block（イシュー #2838。親トラッキング
//! #2807「Blocks 目的別パーツ拡充ツリー Phase 2、マーケティング B」配下、
//! `crate::blocks::marketing::error_page` カテゴリ 2 件目の block。主参照は
//! 対応表 ID R1105 の 1 件のみで、他の ID との統合は行わない）。
//!
//! # 出典に関する注記
//!
//! 参照元は構造（上部ロゴ・中央寄せの空状態メッセージ・人気ページ一覧
//! （行全体がリンク）・戻るリンク・footer の 5 領域構成）のみを参照し、
//! Rust/CSS で独自に再実装する。出典の固有名・ファイル名は記載しない。
//!
//! # 使用部品
//!
//! `empty-state`（メッセージコンテナ）/ `heading`（見出し）/ `text`
//! （エラーコード・説明文）/ `item`（人気ページ 1 行）/ `list`（人気ページ
//! 一覧）/ `icon`（ロゴ・タイル・シェブロン・SNS）/ `link`（戻るリンク・
//! SNS リンク）/ `separator`（footer 上端の罫線・著作権と SNS の間の
//! 縦線）の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `empty_state::root`/`heading::heading`/`text::text`/`item::*`（10 パーツ
//! すべて）/`list::root`/`icon::icon`/`link::root`/`separator::separator`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-error-page-popular-links-*` の `data-*` 属性で渡す
//! （`crate::blocks` モジュール doc と同じ判断軸）。`empty_state::content`・
//! `list::item`・素の `div`/`footer` は `class` をそのまま透過するため、
//! それらのみ `class` でフックする。
//!
//! # `[data-scope="item"][data-part="root"]` を前置する理由
//!
//! `item::root` の recipe（`fandhe_frontend_pre_styled_ui::item::recipe`）は
//! base 宣言 `[data-scope="item"][data-part="root"] { padding: ...; }`
//! （詳細度 0,2,0）を持つ。本 block 固有フック
//! `[data-blocks-error-page-popular-links-item]` を単独属性セレクタ
//! （詳細度 0,1,0）のまま宣言しても詳細度規則上 base 側が勝つため、
//! `error_page_background_image` の image フック（モジュール doc「背景画像
//! フックの詳細度」節）と同じ判断で base と同じ 2 属性セレクタへ前置し、
//! 詳細度を揃えたうえでソース順で後勝ちさせる。
//!
//! # 行全体がリンクになる仕組み
//!
//! `item::root` は `ItemRootProps::href` が `Some` のとき `<a>` として描画
//! する契約を持つ（headless 層の仕様）。本 block は各行を `list::item`
//! （`<li>`）の直接の子として `item::root` を置き、`href` に人気ページの
//! 相対パスを渡す。hover/focus の見た目は `item` recipe の `[href]`
//! 規則（背景色変化・フォーカスリング）にすべて任せ、本 block 側では
//! 追加のインタラクション CSS を書かない。
//!
//! # href の方針（`href="#"` を使わない）
//!
//! 人気ページ 4 行はサイト内に実在する索引ページへの相対パス
//! （`../../guides/` 等、[`crate::blocks::marketing::blog::blog_grid_image`]
//! と同じ先例）を指し、`linkcheck::check_links` が fail-closed に検証する。
//! 戻るリンクは `../../`。SNS リンクは [`REPO`]/[`REPO_ISSUES`]/
//! [`REPO_RELEASES`]（自リポジトリの実在 URL）を指す。実際の利用時は
//! 自分のページ URL・SNS URL へ差し替えることを原稿側の導入文で明記する。
//!
//! # アイコンは自作の抽象幾何図形
//!
//! ロゴ・人気ページのタイル・シェブロン・SNS の各アイコンはいずれも
//! 実在ブランドのロゴ・商標を模さない自作の幾何図形（線画または塗り面）
//! である（[`crate::blocks::marketing::cta::cta_feature_links`] の
//! `geo_icon` と同型のパターン）。SNS アイコンは丸・角丸四角・三角の
//! 抽象図形とし、`role="img"` + 汎用ラベル（"GitHub"/"Issues"/"Releases"
//! はいずれも実サービス名ではなくリンク先種別を示す語として使う）を
//! 付与する。
//!
//! # ロゴをリンクにしない理由
//!
//! ロゴは装飾ではなく識別要素のため `IconProps::label` にダミー社名を渡し
//! `role="img"` として提示するが、リンク先を持たない（`href="#"` を使わない
//! 方針の帰結として、遷移先の無いリンクにはしない）。
//!
//! # 状態の扱い（開閉・切替の状態を持たない）
//!
//! 本 block は開閉・選択等の状態機械を一切持たない静的な合成例である。
//! hover/focus の見た目は `item` recipe の `[href]` 規則に委ねる
//! （モジュール doc「行全体がリンクになる仕組み」節）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで遷移処理・送信処理は一切持たない。文言は
//! すべて架空のものであり、実企業名・実サービス名・実クレデンシャル・
//! PII を含まない（著作権表記の社名は `crate::blocks::dummy_assets::
//! COMPANY_NAMES` から取る）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, footer, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自リポジトリの実在 URL（モジュール doc「href の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const REPO_ISSUES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/issues";
const REPO_RELEASES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";

const ROOT_CLASS: &str = "blocks-error-page-popular-links-root";
const LOGO_CLASS: &str = "blocks-error-page-popular-links-logo";
const CONTENT_CLASS: &str = "blocks-error-page-popular-links-content";
const POPULAR_CLASS: &str = "blocks-error-page-popular-links-popular";
const ROW_CLASS: &str = "blocks-error-page-popular-links-row";
const BACK_CLASS: &str = "blocks-error-page-popular-links-back";
const FOOTER_CLASS: &str = "blocks-error-page-popular-links-footer";
const FOOTER_ROW_CLASS: &str = "blocks-error-page-popular-links-footer-row";
const SOCIALS_CLASS: &str = "blocks-error-page-popular-links-socials";

const MESSAGE_ATTR: &str = "data-blocks-error-page-popular-links-message";
const CODE_ATTR: &str = "data-blocks-error-page-popular-links-code";
const TITLE_ATTR: &str = "data-blocks-error-page-popular-links-title";
const DESCRIPTION_ATTR: &str = "data-blocks-error-page-popular-links-description";
const POPULAR_HEADING_ATTR: &str = "data-blocks-error-page-popular-links-popular-heading";
const LIST_ATTR: &str = "data-blocks-error-page-popular-links-list";
const ITEM_ATTR: &str = "data-blocks-error-page-popular-links-item";
const TILE_ATTR: &str = "data-blocks-error-page-popular-links-tile";
const CHEVRON_ATTR: &str = "data-blocks-error-page-popular-links-chevron";
const BACK_ATTR: &str = "data-blocks-error-page-popular-links-back";
const FOOTER_RULE_ATTR: &str = "data-blocks-error-page-popular-links-footer-rule";
const COPYRIGHT_ATTR: &str = "data-blocks-error-page-popular-links-copyright";
const FOOTER_DIVIDER_ATTR: &str = "data-blocks-error-page-popular-links-footer-divider";
const SOCIAL_ATTR: &str = "data-blocks-error-page-popular-links-social";

/// 人気ページ 1 行分のデータ（モジュール doc「href の方針」節参照。
/// `icon_path_d` は `stroke` 系の自作幾何アイコンの `d` 属性値）。
struct PopularPage {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 人気ページ一覧（サイト内に実在する索引ページのみを指す）。
const POPULAR_PAGES: [PopularPage; 4] = [
    PopularPage {
        href: "../../guides/",
        title: "Guides",
        description: "Step-by-step guides for building with the framework.",
        icon_path_d: "M4 4h12v16H4zM8 8h4M8 12h4",
    },
    PopularPage {
        href: "../../api/",
        title: "API Reference",
        description: "Detailed reference for every public API surface.",
        icon_path_d: "M4 6h16M4 12h16M4 18h10",
    },
    PopularPage {
        href: "../../examples/",
        title: "Examples",
        description: "Full example projects you can run and adapt.",
        icon_path_d: "M12 3l2.5 5.5L20 9l-4 4 1 6-5-3-5 3 1-6-4-4 5.5-.5z",
    },
    PopularPage {
        href: "../../themes/",
        title: "Themes",
        description: "Pre-styled components ready to drop into your app.",
        icon_path_d: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
    },
];

/// 線画（stroke）の自作幾何アイコンを組み立てる（
/// [`crate::blocks::marketing::cta::cta_feature_links::geo_icon`] と同型の
/// パターン。装飾用途のため `IconProps::label` は付けない）。
fn stroke_icon(path_d: &'static str) -> Node {
    icon::icon(
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

/// 人気ページ行の右端に置くシェブロンアイコン（全行共通、装飾用途）。
fn chevron_icon() -> Node {
    stroke_icon("m9 6 6 6-6 6")
}

/// ロゴ（識別要素、モジュール doc「ロゴをリンクにしない理由」節参照）。
/// リンクにはせず `role="img"` + ダミー社名のアクセシブルネームのみを
/// 付与する。
fn logo_mark() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Lg,
            label: Some(dummy_assets::COMPANY_NAMES[0]),
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 3 21 12 12 21 3 12Z")], vec![])],
    )
}

/// SNS リンク用の塗り面（filled）アイコン。`label` はアクセシブルネーム
/// （モジュール doc「アイコンは自作の抽象幾何図形」節参照）。
fn social_icon(label: &'static str, shape: Node) -> Node {
    icon::icon(
        &IconProps {
            label: Some(label),
            ..IconProps::default()
        },
        vec![],
        vec![shape],
    )
}

/// 人気ページ 1 行分を組み立てる（`list::item` の直接の子として
/// `item::root` を置き、行全体をリンクにする。モジュール doc「行全体が
/// リンクになる仕組み」節参照）。
fn popular_row(page: &PopularPage) -> Node {
    list::item(
        vec![("class", ROW_CLASS)],
        vec![item::root(
            ItemRootProps {
                href: Some(page.href),
                ..ItemRootProps::default()
            },
            vec![(ITEM_ATTR, "")],
            vec![
                item::media(
                    ItemMediaVariant::Icon,
                    vec![(TILE_ATTR, "")],
                    vec![stroke_icon(page.icon_path_d)],
                ),
                item::content(
                    vec![],
                    vec![
                        item::title(vec![], vec![text(page.title)]),
                        item::description(vec![], vec![text(page.description)]),
                    ],
                ),
                item::actions(vec![(CHEVRON_ATTR, "")], vec![chevron_icon()]),
            ],
        )],
    )
}

/// `error-page-popular-links` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
#[must_use]
pub fn demo() -> Node {
    let logo = div(vec![("class", LOGO_CLASS)], vec![logo_mark()]);

    let code = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![(CODE_ATTR, "")],
        vec![text("404")],
    );

    let title = empty_state::title(
        vec![],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![(TITLE_ATTR, "")],
            vec![text("Page not found")],
        )],
    );

    let description = empty_state::description(
        vec![],
        vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![(DESCRIPTION_ATTR, "")],
            vec![text(
                "We couldn't find the page you were looking for. \
                 Try one of the popular pages below instead.",
            )],
        )],
    );

    let message = empty_state::root(
        &EmptyStateProps {
            variant: EmptyStateVariant::Plain,
            ..EmptyStateProps::default()
        },
        vec![(MESSAGE_ATTR, "")],
        vec![empty_state::content(
            vec![("class", CONTENT_CLASS)],
            vec![code, title, description],
        )],
    );

    let popular_heading = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![(POPULAR_HEADING_ATTR, "")],
        vec![text("Popular pages")],
    );

    let rows: Vec<Node> = POPULAR_PAGES.iter().map(popular_row).collect();

    let list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![(LIST_ATTR, "")],
        rows,
    );

    let popular = div(vec![("class", POPULAR_CLASS)], vec![popular_heading, list]);

    let back = div(
        vec![("class", BACK_CLASS)],
        vec![link::root(
            "../../",
            &LinkProps::default(),
            vec![(BACK_ATTR, "")],
            vec![text("← Back to home")],
        )],
    );

    let footer_rule =
        separator::separator(&SeparatorProps::default(), vec![(FOOTER_RULE_ATTR, "")]);

    let copyright = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![(COPYRIGHT_ATTR, "")],
        vec![text(format!(
            "© 2026 {}. All rights reserved.",
            dummy_assets::COMPANY_NAMES[0]
        ))],
    );

    let footer_divider = separator::separator(
        &SeparatorProps {
            orientation: Orientation::Vertical,
            ..SeparatorProps::default()
        },
        vec![(FOOTER_DIVIDER_ATTR, "")],
    );

    let socials = div(
        vec![("class", SOCIALS_CLASS)],
        vec![
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "GitHub",
                    el(
                        "circle",
                        vec![("cx", "12"), ("cy", "12"), ("r", "8")],
                        vec![],
                    ),
                )],
            ),
            link::root(
                REPO_ISSUES,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "Issues",
                    el(
                        "rect",
                        vec![
                            ("x", "5"),
                            ("y", "5"),
                            ("width", "14"),
                            ("height", "14"),
                            ("rx", "3"),
                        ],
                        vec![],
                    ),
                )],
            ),
            link::root(
                REPO_RELEASES,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(SOCIAL_ATTR, "")],
                vec![social_icon(
                    "Releases",
                    el("polygon", vec![("points", "12,4 20,20 4,20")], vec![]),
                )],
            ),
        ],
    );

    let footer_row = div(
        vec![("class", FOOTER_ROW_CLASS)],
        vec![copyright, footer_divider, socials],
    );

    let page_footer = footer(vec![("class", FOOTER_CLASS)], vec![footer_rule, footer_row]);

    div(
        vec![("class", ROOT_CLASS)],
        vec![logo, message, popular, back, page_footer],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/error-page-popular-links/",
    title: "error-page-popular-links",
    category: BlockCategory::ErrorPage,
    rust_source: "crates/docs-site/src/blocks/marketing/error_page/error_page_popular_links.rs",
    demo_class: "blocks-error-page-popular-links",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
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
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `error_page_popular_links` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型で
/// `pub(super)` ではなく本ファイル内 `const` として [`super::blocks`] から
/// `BLOCK.layout_css` 経由で連結される）。
///
/// 生の色リテラル（`#fff`/`white` 等）は使わず、可読性の確保はすべて
/// `--fandhe-color-*` トークンで行う。ブレークポイントは
/// `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`（640px = 40rem）
/// と一致するリテラル値を直書きする（`@media` 条件式の中ではテーマ
/// トークンを解決できないため、`cta_feature_links` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-error-page-popular-links {\n  padding: 0;\n}\n\
.blocks-error-page-popular-links-root {\n  display: flex;\n  flex-direction: column;\n  min-height: 28rem;\n  padding: var(--fandhe-space-10) var(--fandhe-space-6);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-error-page-popular-links-logo {\n  display: flex;\n  justify-content: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-error-page-popular-links-message] {\n  max-width: 36rem;\n  width: 100%;\n  margin-inline: auto;\n  text-align: center;\n}\n\
.blocks-error-page-popular-links-content {\n  align-items: center;\n}\n\
[data-blocks-error-page-popular-links-code] {\n  color: var(--fandhe-color-accent);\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-error-page-popular-links-description] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-error-page-popular-links-popular {\n  width: 100%;\n  max-width: 32rem;\n  margin-inline: auto;\n}\n\
[data-blocks-error-page-popular-links-popular-heading] {\n  margin-bottom: var(--fandhe-space-3);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-error-page-popular-links-list] {\n  width: 100%;\n  padding: 0;\n  margin: 0;\n}\n\
.blocks-error-page-popular-links-row {\n  list-style: none;\n}\n\
.blocks-error-page-popular-links-row + .blocks-error-page-popular-links-row {\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-error-page-popular-links-item] {\n  width: 100%;\n}\n\
[data-blocks-error-page-popular-links-tile] {\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-error-page-popular-links-chevron] {\n  color: var(--fandhe-color-fg-subtle);\n}\n\
.blocks-error-page-popular-links-back {\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-error-page-popular-links-footer {\n  margin-top: auto;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-error-page-popular-links-footer-row {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  text-align: center;\n}\n\
[data-blocks-error-page-popular-links-footer-divider] {\n  display: none;\n}\n\
.blocks-error-page-popular-links-socials {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-error-page-popular-links-social] {\n  color: var(--fandhe-color-fg-subtle);\n}\n\
@media (min-width: 40rem) {\n  .blocks-error-page-popular-links-footer-row {\n    flex-direction: row;\n    justify-content: space-between;\n    text-align: left;\n  }\n  [data-blocks-error-page-popular-links-footer-divider] {\n    display: block;\n    height: 1.75rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] がロゴ・空状態メッセージ・人気ページ一覧 4 行（各行が
    /// リンクの `item::root`）・戻るリンク・footer を正しい属性・文言で
    /// 出力し、`<form>`・`href="#"`・`data:` URI・`<script` のいずれも
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-scope="item" data-part="root""#)
                .count(),
            4,
            "demo should render exactly 4 item roots (one per popular page)"
        );
        for href in [
            "../../guides/",
            "../../api/",
            "../../examples/",
            "../../themes/",
            "../../",
        ] {
            assert!(
                html.contains(&format!(r#"href="{href}""#)),
                "demo should link to {href}"
            );
        }
        for hook in [
            MESSAGE_ATTR,
            CODE_ATTR,
            TITLE_ATTR,
            DESCRIPTION_ATTR,
            POPULAR_HEADING_ATTR,
            LIST_ATTR,
            ITEM_ATTR,
            TILE_ATTR,
            CHEVRON_ATTR,
            BACK_ATTR,
            FOOTER_RULE_ATTR,
            COPYRIGHT_ATTR,
            FOOTER_DIVIDER_ATTR,
            SOCIAL_ATTR,
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for text_fragment in [
            "404",
            "Page not found",
            "Popular pages",
            "Guides",
            "API Reference",
            "Examples",
            "Themes",
            "Back to home",
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、色リテラルではなくトークン
    /// 参照で可読性を確保していること。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_color_tokens_not_literals() {
        for selector in [
            ".blocks-error-page-popular-links {",
            ".blocks-error-page-popular-links-root {",
            ".blocks-error-page-popular-links-logo {",
            "[data-blocks-error-page-popular-links-message] {",
            ".blocks-error-page-popular-links-content {",
            "[data-blocks-error-page-popular-links-code] {",
            "[data-blocks-error-page-popular-links-description] {",
            ".blocks-error-page-popular-links-popular {",
            "[data-blocks-error-page-popular-links-popular-heading] {",
            "[data-blocks-error-page-popular-links-list] {",
            ".blocks-error-page-popular-links-row {",
            ".blocks-error-page-popular-links-row + .blocks-error-page-popular-links-row {",
            "[data-scope=\"item\"][data-part=\"root\"][data-blocks-error-page-popular-links-item] {",
            "[data-blocks-error-page-popular-links-tile] {",
            "[data-blocks-error-page-popular-links-chevron] {",
            ".blocks-error-page-popular-links-back {",
            ".blocks-error-page-popular-links-footer {",
            ".blocks-error-page-popular-links-footer-row {",
            "[data-blocks-error-page-popular-links-footer-divider] {",
            ".blocks-error-page-popular-links-socials {",
            "[data-blocks-error-page-popular-links-social] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg-muted)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-accent)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-border)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }
}
