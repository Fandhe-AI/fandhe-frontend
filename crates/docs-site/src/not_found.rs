//! docs サイトの 404 ページ（イシュー #3623）。
//!
//! # 役割・呼び出し文脈
//!
//! GitHub Pages は未知の URL に対してサイトルート直下の `404.html` を返す。
//! 本モジュールはその本文・ページ枠・書き出し用文字列を組み立てる。
//! [`crate::build::build_site`] が次の順で使う。
//!
//! 1. [`page`] を linkcheck の対象ページ列へ加える（リンク切れは fail-closed）
//! 2. [`document_html`] の文字列を [`OUTPUT_PATH`] へ `ssg::generate_assets` で書く
//!
//! `ssg::generate_pages` は `<path>/index.html` 固定かつドット入りパスを拒否するため
//! 使えない。`generate_assets` へは `generate_pages` と同じ「固定の DOCTYPE 前置 +
//! `render()`」の文字列だけを渡す。
//!
//! # nav・検索インデックスに載せない理由
//!
//! `nav.toml` に登録せず、検索インデックスは `nav.all_pages()` ループ内でしか集めない。
//! よって構造的に除外され、除外述語は持たない（リダイレクトページと同じ構造）。
//!
//! # 絶対リンクにする理由
//!
//! 404 は任意の深さの URL で表示されるため、リンクはすべて [`crate::layout::asset_href`]
//! 経由の `base_path` 付き絶対パスにする。
//!
//! # セキュリティ（REQ-1）
//!
//! ノード木 API と pre-styled-ui 部品だけで組み、`raw_html()`・インライン
//! スクリプト・`on*=` 属性・フォーム・URL の読み取り反射は持たない。
//! `nav` 由来の文字列は既定エスケープを経由する。

use fandhe_frontend_core::{div, render, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};

use crate::layout;
use crate::nav::{self, Nav};

/// 書き出し先（サイトルート直下）。`generate_assets` に渡す。
pub const OUTPUT_PATH: &str = "/404.html";

/// ページの `<title>`。
pub const PAGE_TITLE: &str = "ページが見つかりません";

/// 本文ラッパーの class。
pub const NOT_FOUND_CLASS: &str = "docs-not-found";

/// 主要セクションリンク一覧ラッパーの class。
pub const NOT_FOUND_LINKS_CLASS: &str = "docs-not-found-links";

/// 本文（empty-state + セクションリンク一覧）を返す。
///
/// 見出しは pre-styled の `heading`（`data-scope` 付き）なので目次・アンカー注入の
/// 対象外となり、`docs-container--no-toc` になる。
#[must_use]
pub fn body(nav: &Nav) -> Node {
    let base = nav.site.base_path.as_str();
    let hrefs: Vec<String> = nav
        .sections
        .iter()
        .map(|s| layout::asset_href(base, &s.index_path))
        .collect();
    let items: Vec<Node> = nav
        .sections
        .iter()
        .zip(hrefs.iter())
        .map(|(section, href)| {
            list::item(
                vec![],
                vec![link::root(
                    href,
                    &LinkProps::default(),
                    vec![],
                    vec![text(section.title.clone())],
                )],
            )
        })
        .collect();

    let empty = empty_state::root(
        &EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::title(
                    vec![],
                    vec![heading(
                        HeadingLevel::H1,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(PAGE_TITLE)],
                    )],
                ),
                empty_state::description(
                    vec![],
                    vec![text(
                        "お探しのページは移動または削除された可能性があります。\
                         下のセクション一覧から探すか、JavaScript が有効な環境ではヘッダーの検索をお使いください。",
                    )],
                ),
            ],
        )],
    );

    div(
        vec![("class", NOT_FOUND_CLASS)],
        vec![
            empty,
            div(
                vec![("class", NOT_FOUND_LINKS_CLASS)],
                vec![
                    heading(
                        HeadingLevel::H2,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("主なセクション")],
                    ),
                    list::root(ListType::Unordered, ListVariant::default(), vec![], items),
                ],
            ),
        ],
    )
}

/// 完全なページ枠の文書ノードを返す。サイドバーは nav 未登録パスの
/// フォールバック（全セクション描画）を使う。
#[must_use]
pub fn page(nav: &Nav) -> Node {
    layout::docs_page_with_assets(
        PAGE_TITLE,
        &nav.site.base_path,
        nav::sidebar(nav, OUTPUT_PATH),
        body(nav),
        &[],
        Some(nav::header_nav(nav, OUTPUT_PATH)),
        Some(crate::site_footer::site_footer(nav)),
    )
}

/// 書き出し用の HTML 文字列（固定 DOCTYPE + `render()`）。
#[must_use]
pub fn document_html(node: &Node) -> String {
    format!("<!DOCTYPE html>\n{}", render(node))
}

/// [`page`] を文書文字列にしたもの。
#[must_use]
pub fn html(nav: &Nav) -> String {
    document_html(&page(nav))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::parse_nav;
    use fandhe_frontend_core::find_attr_values;

    fn nav_with(base: &str) -> Nav {
        parse_nav(&format!(
            "[site]\ntitle = \"T\"\nbase_path = \"{base}\"\n\n\
             [[section]]\ntitle = \"Start\"\nindex_path = \"/\"\n\n\
             [[section.page]]\ntitle = \"Home\"\npath = \"/\"\nsource = \"site/index.md\"\n\n\
             [[section]]\ntitle = \"Guides\"\nindex_path = \"/guides/\"\n\n\
             [[section.page]]\ntitle = \"G\"\npath = \"/guides/\"\nsource = \"site/guides.md\"\n"
        ))
        .expect("fixture nav")
    }

    #[test]
    fn output_is_deterministic_and_inert() {
        let nav = nav_with("/x");
        let a = html(&nav);
        assert_eq!(a, html(&nav));
        assert!(a.starts_with("<!DOCTYPE html>\n"));
        for banned in [
            "<form",
            "href=\"#\"",
            "javascript:",
            "src=\"data:",
            " onclick=",
        ] {
            assert!(!a.contains(banned), "{banned}");
        }
    }

    #[test]
    fn all_hrefs_are_absolute_or_fragment_or_https() {
        let nav = nav_with("/x");
        for href in find_attr_values(&page(&nav), "href") {
            assert!(
                href.starts_with("/x/") || href.starts_with('#') || href.starts_with("https://"),
                "{href}"
            );
        }
    }

    #[test]
    fn body_lists_sections_in_order_with_single_h1() {
        let nav = nav_with("/x");
        let b = render(&body(&nav));
        assert!(b.contains(NOT_FOUND_CLASS) && b.contains(NOT_FOUND_LINKS_CLASS));
        assert_eq!(b.matches("<h1").count(), 1);
        let hrefs = find_attr_values(&body(&nav), "href");
        assert_eq!(hrefs, vec!["/x/".to_string(), "/x/guides/".to_string()]);
    }

    #[test]
    fn base_path_variants_normalise() {
        for base in ["", "/x"] {
            let nav = nav_with(base);
            let hrefs = find_attr_values(&body(&nav), "href");
            let b = base.trim_end_matches('/');
            assert_eq!(hrefs[1], format!("{b}/guides/"));
        }
    }
}
