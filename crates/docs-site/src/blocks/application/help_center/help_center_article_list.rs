//! `help-center-article-list` block（イシュー #2974。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下、Application / Help Center カテゴリ）。
//! パンくず → コレクション見出し（線画アイコン・題名・
//! 説明・記事数バッジ）→ 記事一覧カードを合成する。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`profile_detail_datalist.rs` と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `breadcrumb` / `heading` / `card` / `stat` / `badge` / `icon` / `link`
//! の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成。R0120 主参照 + R0122 の見出しアイコン・件数バッジ）**:
//!   見出しに線画アイコン + 題名 + 記事数バッジを添え、1 枚のカードへ
//!   記事 6 件をフラットに並べる。
//! - **B（グループ見出しで分割。R0121）**: 同じ見出し（アイコンは省き
//!   バッジのみ）の下、カード内を H3 のグループ見出しで 3 区分
//!   （リファレンスを読む・部品を探す・サイト内を移動する）し、区分
//!   ごとに記事を並べる。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::breadcrumb::root`]・
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]・
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::stat::root`]・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::icon::icon`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-help-center-article-list-*`）。レイアウト用ラッパー
//! （素の `<div>`/`<ul>`/`<li>`/`<p>`）は `class="blocks-help-center-
//! article-list-*"` を使う（`profile_detail_datalist` と同型の判断）。
//!
//! # 行リンクの詳細度（`link::root` base 宣言との勝ち負け）
//!
//! [`fandhe_frontend_pre_styled_ui::link::root`] の base 宣言は
//! `[data-scope="link"][data-part="root"]`（詳細度 0,2,0）のため、行リンク
//! 固有の規則は `[data-scope="link"][data-part="root"][data-blocks-help-
//! center-article-list-row]` で前置して勝たせる
//! （`error_page_popular_links` と同型の判断）。
//!
//! # カード余白オーバーライドの詳細度（`card::root` size variant との勝ち負け）
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`] の size variant は
//! `[data-scope="card"][data-part="root"].fd-card--size-<size>`（詳細度
//! 0,3,0）で `--fandhe-card-padding` を宣言する。狭幅時の padding
//! override を単独属性セレクタ（`[data-blocks-help-center-article-list-
//! card]`、詳細度 0,1,0）のまま宣言すると詳細度規則上 size variant 側が
//! 勝ち、狭幅でも padding が詰まらない。行リンクの規則と同じ判断で、
//! card 側フックも base と同じ 2 属性セレクタへ前置して詳細度を揃え
//! （`[data-scope="card"][data-part="root"][data-blocks-help-center-
//! article-list-card]`）、ソース順（`blocks.css` は `pre-styled-ui.css`
//! の後に読み込まれる、`crate::build::build_site` の stylesheet 配線順）
//! で後勝ちさせる。
//!
//! # インスタンス内セクション間の余白
//!
//! `[data-blocks-help-center-article-list-instance]`（パンくず・見出し・
//! stat・card を縦に並べるラッパー）へ `display: flex` + `gap` を宣言し、
//! 各パーツ自身の margin に頼らずセクション間の間隔を確保する（隣接パーツ
//! が marginless のまま密着しないようにする）。
//!
//! # 狭い幅ではカードの余白を詰め、矢印は右端に残す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist` と同型のパターン）。
//! [`LAYOUT_CSS`] のラッパー `.blocks-help-center-article-list-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `36rem` 未満の
//! とき `card` の padding とカード内行の padding を詰める。シェブロンは
//! `margin-inline-start: auto` のまま右端に残る。
//!
//! # `<form>` を使わない・`href="#"` を使わない・記事名と遷移先を一致させる
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理を持たない静的表示のみである。個別記事へ実際に遷移できる
//! 専用ページは用意していないため、記事名はダミーの見出しではなく
//! 遷移先ページの内容をそのまま表す文言にし（例: 「利用ガイドを読む」→
//! `../../guides/`）、記事名と無関係なページへ遷移しないようにする。
//! パンくずの中間項目も同様に、実際の遷移先（ドキュメントサイトの
//! トップ `../../`）を表す「ドキュメントトップ」を label とする。
//! パンくずは `../`（Blocks 索引）・`../../`（ドキュメントトップ）、
//! 記事行はサイト内実在ページの相対パス（`../../guides/`・`../../api/`・
//! `../../primitives/`・`../../themes/`・`../../wireframes/`・`../../`）を
//! 割り当てる（`href="#"` は使わない、`linkcheck` fail-closed）。
//!
//! # アイコンは自作の単純図形
//!
//! `icon::icon` + `el("path", ...)` による線画のみで構成する（実在ブランド
//! のアイコンセットは使わない）。見出しの本アイコン・行末のシェブロンの
//! いずれも装飾用途（`IconProps::default()` の `label: None` →
//! `aria-hidden`）とし、アクセシブルネームは可視テキストが担う。
//!
//! # ダミー素材について
//!
//! コレクション名・記事タイトル・件数はすべて独自の架空ダミーであり、
//! 実企業名・実クレデンシャル・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// パンくず 1 本（ドキュメントトップ → Blocks → 現在のコレクション。
/// 実サイト階層〔ドキュメントトップ配下に Blocks、その配下に本ページ〕と
/// 一致させる順序、イシュー #3427 レビュー指摘対応）。
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
                    vec![breadcrumb::link(
                        "../../",
                        vec![],
                        vec![text("ドキュメントトップ")],
                    )],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(
                        vec![],
                        vec![text("サイトの歩き方")],
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
        vec![text("サイトの歩き方")],
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
                    "ドキュメントサイト内の主要セクションへの入り口をまとめた\
                     記事一覧です。目的のページが見つからないときはこちらから\
                     探してください。",
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
        vec![stat_item("記事", "6"), stat_item("最終更新", "2026-09-18")],
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
            article_row("利用ガイドを読む", "../../guides/"),
            article_row("API リファレンスを開く", "../../api/"),
            article_row("Primitives 部品を探す", "../../primitives/"),
            article_row("Themes 部品を探す", "../../themes/"),
            article_row("Wireframes 部品を探す", "../../wireframes/"),
            article_row("ドキュメントトップへ戻る", "../../"),
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
            collection_heading(true, "6"),
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
                    "リファレンスを読む",
                    vec![
                        ("利用ガイドを読む", "../../guides/"),
                        ("API リファレンスを開く", "../../api/"),
                    ],
                ),
                article_group(
                    "部品を探す",
                    vec![
                        ("Primitives 部品を探す", "../../primitives/"),
                        ("Themes 部品を探す", "../../themes/"),
                    ],
                ),
                article_group(
                    "サイト内を移動する",
                    vec![
                        ("Wireframes 部品を探す", "../../wireframes/"),
                        ("ドキュメントトップへ戻る", "../../"),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/help-center-article-list/",
    title: "help-center-article-list",
    category: BlockCategory::HelpCenter,
    rust_source: "crates/docs-site/src/blocks/application/help_center/help_center_article_list.rs",
    demo_class: "blocks-help-center-article-list",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `help_center_article_list` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
///
/// 行リンクの規則は `[data-scope="link"][data-part="root"]`（詳細度
/// 0,2,0）に勝つよう `[data-blocks-help-center-article-list-row]` を
/// 併記する（モジュール doc「行リンクの詳細度」節参照）。カードの狭幅
/// padding override も同様に `[data-scope="card"][data-part="root"]`
/// （size variant の詳細度 0,3,0）に勝つよう併記する（モジュール doc
/// 「カード余白オーバーライドの詳細度」節参照）。row-end のシェブロン
/// アイコンの減色規則も同様に、icon 基底の
/// `[data-scope="icon"][data-part="root"] { color: currentColor }`
/// （詳細度 0,2,0）に勝つよう `svg` 側の `[data-scope="icon"]
/// [data-part="root"]` を併記して詳細度 0,3,0 へ引き上げる
/// （`!important` は使わない）。
const LAYOUT_CSS: &str = "\
.blocks-help-center-article-list-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-help-center-article-list;\n}\n\
[data-blocks-help-center-article-list-instance] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-help-center-article-list-heading {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-help-center-article-list-heading-row] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-help-center-article-list-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-help-center-article-list-stats {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-help-center-article-list-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-help-center-article-list-group + .blocks-help-center-article-list-group {\n  margin-block-start: var(--fandhe-space-6);\n}\n\
.blocks-help-center-article-list-rows {\n  display: flex;\n  flex-direction: column;\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-help-center-article-list-rows > li + li {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-help-center-article-list-row] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-3);\n  padding-inline: var(--fandhe-space-4);\n  color: var(--fandhe-color-fg);\n}\n\
[data-blocks-help-center-article-list-row] > svg[data-scope=\"icon\"][data-part=\"root\"] {\n  margin-inline-start: auto;\n  color: var(--fandhe-color-fg-subtle);\n  flex-shrink: 0;\n}\n\
@container blocks-help-center-article-list (max-width: 36rem) {\n  \
[data-scope=\"card\"][data-part=\"root\"][data-blocks-help-center-article-list-card] {\n    --fandhe-card-padding: var(--fandhe-space-3);\n  }\n  \
[data-scope=\"link\"][data-part=\"root\"][data-blocks-help-center-article-list-row] {\n    padding-inline: var(--fandhe-space-3);\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"stat\"",
            "data-scope=\"badge\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-help-center-article-list-instance")
                .count(),
            2
        );
    }

    #[test]
    fn article_row_count_matches_expected() {
        let html = demo_html();
        // A: 6 件、B: 2+2+2=6 件、計 12 件。
        assert_eq!(
            html.matches("data-blocks-help-center-article-list-row")
                .count(),
            12
        );
    }

    #[test]
    fn version_a_badge_count_matches_its_own_row_count() {
        // 版 A の見出しバッジ・stat が「6 件」で、実際に並ぶ記事も 6 件で
        // 一致すること（P2 回帰防止）。
        let html = render(&super::version_flat());
        assert_eq!(
            html.matches("data-blocks-help-center-article-list-row")
                .count(),
            6
        );
        assert!(html.contains("6 件"));
        assert!(!html.contains("12 件"));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-help-center-article-list (max-width: 36rem)")
        );
        assert!(LAYOUT_CSS.contains("margin-inline-start: auto;"));
    }

    #[test]
    fn heading_icon_appears_only_in_variant_a() {
        let html = demo_html();
        // book_icon の path d 冒頭部分がちょうど 1 回だけ出現する。
        assert_eq!(
            html.matches("M4 5c4-2 8-1 8 1v13c0-2-4-3-8-1zm16 0c-4-2-8-1-8 1v13c0-2 4-3 8-1z")
                .count(),
            1
        );
    }
}
