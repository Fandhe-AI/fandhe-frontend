//! 全ページ共通のサイトフッター（イシュー #3609）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` がページループの前に [`site_footer`] を 1 回だけ呼び、
//! 各ページへ clone して `crate::layout::docs_page_with_assets` の `footer` 引数で渡す。
//! `docs_page_with_assets` は `<body>` の最後の子（`div.docs-container` の直後）へ置く。
//! リダイレクト案内ページ（`crate::redirect`）は別経路の文書なので対象外になる。
//!
//! 手本は Blocks の `footer-link-columns` だが `demo()` は呼ばず、pre-styled-ui の
//! `heading` / `link` / `separator` / `text` を合成して docs 専用に組む。
//!
//! # DOM
//!
//! ```text
//! footer.docs-footer                          … 暗黙 contentinfo
//!   div.docs-footer-inner                     … .docs-container と同じ計測枠
//!     nav.docs-footer-nav[aria-label=Footer]
//!       div.docs-footer-columns
//!         div.docs-footer-group × セクション数  … heading(h2) + ul.docs-footer-list
//!     separator(hr)
//!     div.docs-footer-bottom                  … 著作権・ライセンス・外部リンク
//! ```
//!
//! # なぜ body 直下か
//!
//! `<footer>` は `main` / `article` / `aside` / `nav` / `section` の子孫だと暗黙ロール
//! `contentinfo` を失う。また本文 `Node` へ足すと TOC・検索インデックスに混ざる。
//! sticky のサイドバー・右目次は `.docs-container` の内側でのみ固定されるため、
//! 外側の兄弟であるフッターと重ならない。Blocks デモ内の `<footer>` は `main` の
//! 内側なので `contentinfo` にならず、ページ全体で 1 つに保たれる。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 文言は `text()` を通す。`raw_html()` と HTML 文字列の組み立ては使わない。
//! 内部 href は `parse_nav` 検証済みのパスを `layout::asset_href` で前置したものだけ、
//! 外部 URL は文字列リテラル定数だけで、すべて `external: true`
//! （`target="_blank"` + `rel="noopener noreferrer"`）にする。`id` 属性は出さない
//! （`layout::RESERVED_LAYOUT_IDS` と衝突させない）。ロゴ SVG は持ち込まない。

use fandhe_frontend_core::{div, footer, li, nav as nav_el, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{text as ps_text, TextProps, TextSize, TextVariant};

use crate::layout::{asset_href, REPOSITORY_URL};
use crate::nav::{Nav, Page, Section};

/// フッター外側の class（`<footer>`）。
pub const FOOTER_CLASS: &str = "docs-footer";
/// 計測枠の class。
pub const FOOTER_INNER_CLASS: &str = "docs-footer-inner";
/// リンク列 nav の class。
pub const FOOTER_NAV_CLASS: &str = "docs-footer-nav";
/// 列グリッドの class。
pub const FOOTER_COLUMNS_CLASS: &str = "docs-footer-columns";
/// セクション 1 列の class。
pub const FOOTER_GROUP_CLASS: &str = "docs-footer-group";
/// リンク一覧 `ul` の class。
pub const FOOTER_LIST_CLASS: &str = "docs-footer-list";
/// 下段（著作権・ライセンス・外部リンク）の class。
pub const FOOTER_BOTTOM_CLASS: &str = "docs-footer-bottom";
/// 外部リンク一覧 `ul` の class。
pub const FOOTER_EXTERNAL_CLASS: &str = "docs-footer-external";

/// フッター nav のランドマーク名（他の nav 名と重ならない英語）。
pub const FOOTER_ARIA_LABEL: &str = "Footer";

/// 1 列あたりのリンク数上限（索引ページを含む）。直下ページが多い
/// セクション（Wireframes 等）で列が肥大化するのを防ぐ。
pub const FOOTER_LINKS_PER_SECTION: usize = 5;

/// crates.io の主要クレートページ。
const CRATES_IO_URL: &str = "https://crates.io/crates/fandhe-frontend-core";
/// MIT ライセンス本文（リポジトリ直下）。
const LICENSE_MIT_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-MIT";
/// Apache-2.0 ライセンス本文（リポジトリ直下）。
const LICENSE_APACHE_URL: &str =
    "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-APACHE";

/// 著作権表記（`LICENSE-MIT` の著作権行と同じ文言）。
const COPYRIGHT_TEXT: &str = "© 2026 Fandhe-AI / fandhe-frontend contributors";

/// セクションの列に載せるページ（索引ページ先頭、続けて直下ページの宣言順、
/// 上限 [`FOOTER_LINKS_PER_SECTION`]）。グループ配下のページは含めない。
fn column_pages(section: &Section) -> Vec<&Page> {
    let mut pages: Vec<&Page> = Vec::new();
    if let Some(index) = section.all_pages().find(|p| p.path == section.index_path) {
        pages.push(index);
    }
    pages.extend(
        section
            .pages
            .iter()
            .filter(|p| p.path != section.index_path),
    );
    pages.truncate(FOOTER_LINKS_PER_SECTION);
    pages
}

fn external_link(href: &str, label: &str) -> Node {
    let props = LinkProps {
        external: true,
        ..LinkProps::default()
    };
    link::root(href, &props, vec![], vec![text(label)])
}

fn external_item(href: &str, label: &str) -> Node {
    li(vec![], vec![external_link(href, label)])
}

/// `nav` の全セクションから全ページ共通のフッターを組み立てる。
///
/// 現在ページに依存しないため `aria-current` は付けない（サイドバー・ヘッダーの
/// 現在地表示と意味を重複させない）。
pub fn site_footer(nav: &Nav) -> Node {
    let base = nav.site.base_path.as_str();
    let heading_props = HeadingProps {
        size: HeadingSize::Sm,
        ..HeadingProps::default()
    };
    let groups: Vec<Node> = nav
        .sections
        .iter()
        .map(|section| {
            let items: Vec<Node> = column_pages(section)
                .into_iter()
                .map(|page| {
                    let href = asset_href(base, &page.path);
                    li(
                        vec![],
                        vec![link::root(
                            &href,
                            &LinkProps::default(),
                            vec![],
                            vec![text(page.title.as_str())],
                        )],
                    )
                })
                .collect();
            div(
                vec![("class", FOOTER_GROUP_CLASS)],
                vec![
                    heading(
                        HeadingLevel::H2,
                        &heading_props,
                        vec![],
                        vec![text(section.title.as_str())],
                    ),
                    ul(vec![("class", FOOTER_LIST_CLASS)], items),
                ],
            )
        })
        .collect();

    let small_muted = TextProps {
        size: TextSize::Sm,
        variant: TextVariant::Muted,
        ..TextProps::default()
    };
    let bottom = div(
        vec![("class", FOOTER_BOTTOM_CLASS)],
        vec![
            div(
                vec![],
                vec![
                    ps_text(&small_muted, vec![], vec![text(COPYRIGHT_TEXT)]),
                    ps_text(
                        &small_muted,
                        vec![],
                        vec![
                            text("Licensed under "),
                            external_link(LICENSE_MIT_URL, "MIT"),
                            text(" OR "),
                            external_link(LICENSE_APACHE_URL, "Apache-2.0"),
                        ],
                    ),
                ],
            ),
            ul(
                vec![("class", FOOTER_EXTERNAL_CLASS)],
                vec![
                    external_item(REPOSITORY_URL, "GitHub"),
                    external_item(CRATES_IO_URL, "crates.io"),
                ],
            ),
        ],
    );

    footer(
        vec![("class", FOOTER_CLASS)],
        vec![div(
            vec![("class", FOOTER_INNER_CLASS)],
            vec![
                nav_el(
                    vec![
                        ("class", FOOTER_NAV_CLASS),
                        ("aria-label", FOOTER_ARIA_LABEL),
                    ],
                    vec![div(vec![("class", FOOTER_COLUMNS_CLASS)], groups)],
                ),
                separator(&SeparatorProps::default(), vec![]),
                bottom,
            ],
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::parse_nav;
    use fandhe_frontend_core::render;

    fn nav_with(base: &str, many: usize) -> Nav {
        let mut s = format!(
            "[site]\ntitle = \"T\"\nbase_path = \"{base}\"\n\n[[section]]\ntitle = \"A <script>\"\nindex_path = \"/a/\"\n\n"
        );
        // 索引ページを宣言順の途中に置き、先頭へ移ることを確かめる。
        s.push_str(
            "[[section.page]]\ntitle = \"First\"\nsource = \"s.md\"\npath = \"/a/first/\"\n\n",
        );
        s.push_str("[[section.page]]\ntitle = \"Idx\"\nsource = \"i.md\"\npath = \"/a/\"\n\n");
        for i in 0..many {
            s.push_str(&format!(
                "[[section.page]]\ntitle = \"P{i}\"\nsource = \"p{i}.md\"\npath = \"/a/p{i}/\"\n\n"
            ));
        }
        s.push_str(
            "[[section.group]]\ntitle = \"G\"\n\n[[section.group.page]]\ntitle = \"InGroup\"\nsource = \"g.md\"\npath = \"/a/g/\"\n\n",
        );
        s.push_str("[[section]]\ntitle = \"B\"\nindex_path = \"/b/\"\n\n[[section.page]]\ntitle = \"BIdx\"\nsource = \"b.md\"\npath = \"/b/\"\n");
        parse_nav(&s).expect("fixture nav parses")
    }

    #[test]
    fn one_column_per_section_in_declaration_order() {
        let html = render(&site_footer(&nav_with("", 2)));
        assert_eq!(html.matches("class=\"docs-footer-group\"").count(), 2);
        assert!(html.find("BIdx").unwrap() > html.find("Idx").unwrap());
    }

    #[test]
    fn index_page_comes_first_and_links_are_capped_without_group_pages() {
        let html = render(&site_footer(&nav_with("", 10)));
        let list = &html[html.find("docs-footer-list").unwrap()..];
        let list = &list[..list.find("</ul>").unwrap()];
        assert_eq!(list.matches("<a ").count(), FOOTER_LINKS_PER_SECTION);
        assert!(list.find("Idx").unwrap() < list.find("First").unwrap());
        assert!(!html.contains("InGroup"));
    }

    #[test]
    fn external_links_are_all_hardened() {
        let html = render(&site_footer(&nav_with("", 1)));
        assert_eq!(html.matches("target=\"_blank\"").count(), 4);
        assert_eq!(html.matches("rel=\"noopener noreferrer\"").count(), 4);
        assert!(html.contains("https://crates.io/crates/fandhe-frontend-core"));
    }

    #[test]
    fn output_has_no_ids_scripts_or_dangerous_attrs() {
        let html = render(&site_footer(&nav_with("", 1)));
        for forbidden in [
            "id=\"",
            "<form",
            "<script",
            "javascript:",
            " on",
            "href=\"#\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "{forbidden} が出力された: {html}"
            );
        }
    }

    #[test]
    fn hostile_section_title_is_escaped() {
        let html = render(&site_footer(&nav_with("", 0)));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn base_path_is_prefixed_to_internal_hrefs() {
        let html = render(&site_footer(&nav_with("/fandhe-frontend", 0)));
        assert!(html.contains("href=\"/fandhe-frontend/a/\""));
        assert!(html.contains("href=\"/fandhe-frontend/b/\""));
    }
}
