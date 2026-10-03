//! 本文先頭のパンくず付きページ見出し（イシュー #3607）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` のページループが、生成節の挿入（blocks / wireframes /
//! page_sections）がすべて済んだ直後に [`wrap_page_heading`] を呼ぶ。`Nav` と `Page` を
//! 同時に持つのがそのループだけなので、`layout` ではなくここで組み立てる。
//!
//! 出力構造:
//!
//! ```text
//! header.docs-page-heading
//!   div.docs-page-breadcrumb > nav[data-scope=breadcrumb][aria-label=Breadcrumb] > ol
//!   h1                       … Markdown 由来の h1 を移設（無ければ page.title で補う）
//! ```
//!
//! # h1 の扱い
//!
//! h1 は pre-styled-ui の `heading` に置き換えず、既存ノードをそのまま移す。`heading` は
//! `data-scope` を持つため検索インデックス（`data-scope` 部分木を除外）から h1 の文言が
//! 落ち、「文書の h1 = `data-scope` の外の `<h1>`」という判定も崩れるため。h1 は
//! TOC・見出しアンカーの対象外なので、移設で TOC は変わらない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! セクション名・グループ名・ページ名は `text()`、href は `layout::asset_href` で
//! 組み立てる。`raw_html()` は使わない。href は `parse_nav` 検証済みの `index_path` のみ。

use fandhe_frontend_core::{div, h1, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::Size;

use crate::layout::asset_href;
use crate::nav::{Nav, Page};

/// 見出し部ラッパーの class。
pub const PAGE_HEADING_CLASS: &str = "docs-page-heading";
/// パンくずラッパーの class（pre-styled-ui の root は class を捨てるため外側に付ける）。
pub const PAGE_BREADCRUMB_CLASS: &str = "docs-page-breadcrumb";
/// パンくず nav のランドマーク名（既存の nav 名と重ならない英語）。
pub const BREADCRUMB_ARIA_LABEL: &str = "Breadcrumb";

/// `current_path` のページのパンくずを `Nav` から組み立てる。
///
/// 所属セクションが無いとき（トップ等）は `None`。セクション索引ページでは
/// 自分自身へ戻るリンクを作らず、セクション名のみを現在項目にする。
pub fn breadcrumb_nav(nav: &Nav, current_path: &str) -> Option<Node> {
    let section = nav.section_for_path(current_path)?;
    let sep = || breadcrumb::separator(vec![], vec![text("/")]);
    let mut items: Vec<Node> = Vec::new();

    if section.index_path == current_path {
        items.push(breadcrumb::item(
            vec![],
            vec![breadcrumb::current_link(
                vec![],
                vec![text(section.title.as_str())],
            )],
        ));
    } else {
        let href = asset_href(&nav.site.base_path, &section.index_path);
        items.push(breadcrumb::item(
            vec![],
            vec![breadcrumb::link(
                &href,
                vec![],
                vec![text(section.title.as_str())],
            )],
        ));
        if let Some(group) = section.group_for_path(current_path) {
            items.push(sep());
            items.push(breadcrumb::item(
                vec![],
                vec![span(vec![], vec![text(group.title.as_str())])],
            ));
        }
        let title = section
            .all_pages()
            .find(|p| p.path == current_path)
            .map(|p| p.title.as_str())?;
        items.push(sep());
        items.push(breadcrumb::item(
            vec![],
            vec![breadcrumb::current_link(vec![], vec![text(title)])],
        ));
    }
    let root = breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::default(),
        Some(BREADCRUMB_ARIA_LABEL),
        vec![],
        vec![breadcrumb::list(vec![], items)],
    );
    Some(div(vec![("class", PAGE_BREADCRUMB_CLASS)], vec![root]))
}

/// 最上位の最初の `h1` を取り出し、パンくずと合わせた見出し部を先頭へ置く。
///
/// トップページ（`/`）と所属セクションの無いページでは `blocks` をそのまま返す。
/// h1 が無ければ `page.title` で補い、h1 がちょうど 1 つになるようにする。
pub fn wrap_page_heading(nav: &Nav, page: &Page, blocks: Vec<Node>) -> Vec<Node> {
    if page.path == "/" {
        return blocks;
    }
    let Some(crumbs) = breadcrumb_nav(nav, &page.path) else {
        return blocks;
    };
    let mut rest: Vec<Node> = Vec::with_capacity(blocks.len());
    let mut title_node: Option<Node> = None;
    for node in blocks {
        if title_node.is_none() && matches!(&node, Node::Element { tag: "h1", .. }) {
            title_node = Some(node);
        } else {
            rest.push(node);
        }
    }
    let title_node = title_node.unwrap_or_else(|| h1(vec![], vec![text(page.title.as_str())]));
    let mut out = Vec::with_capacity(rest.len() + 1);
    out.push(header(
        vec![("class", PAGE_HEADING_CLASS)],
        vec![crumbs, title_node],
    ));
    out.extend(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::parse_nav;
    use fandhe_frontend_core::render;

    const NAV: &str = r#"
[site]
title = "T"
base_path = "/base"

[[section]]
title = "Guides <x>"
index_path = "/guides/"

[[section.page]]
title = "Guides"
source = "site/guides.md"
path = "/guides/"

[[section.page]]
title = "Direct"
source = "site/d.md"
path = "/guides/direct/"

[[section.group]]
title = "Grp"

[[section.group.page]]
title = "<script>alert(1)</script>"
source = "site/g.md"
path = "/guides/g/"
"#;

    fn nav() -> Nav {
        parse_nav(NAV).expect("fixture nav parses")
    }

    fn page(nav: &Nav, path: &str) -> Page {
        nav.sections
            .iter()
            .flat_map(|s| s.all_pages())
            .find(|p| p.path == path)
            .cloned()
            .unwrap()
    }

    #[test]
    fn direct_page_has_section_and_current() {
        let n = nav();
        let html = render(&breadcrumb_nav(&n, "/guides/direct/").unwrap());
        assert!(html.contains("href=\"/base/guides/\""));
        assert!(html.contains("Guides &lt;x&gt;"));
        assert!(html.contains("aria-current=\"page\""));
        assert!(html.contains("aria-label=\"Breadcrumb\""));
        assert!(!html.contains("Grp"));
    }

    #[test]
    fn group_page_has_group_and_escapes_title() {
        let n = nav();
        let html = render(&breadcrumb_nav(&n, "/guides/g/").unwrap());
        assert!(html.contains("<span>Grp</span>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 2);
        assert_eq!(html.matches("role=\"presentation\"").count(), 2);
    }

    #[test]
    fn index_page_has_only_current_section() {
        let n = nav();
        let html = render(&breadcrumb_nav(&n, "/guides/").unwrap());
        assert!(!html.contains("<a "));
        assert!(html.contains("aria-current=\"page\""));
        assert!(!html.contains("role=\"presentation\""));
    }

    #[test]
    fn top_page_is_untouched() {
        let n = nav();
        let p = Page {
            title: "Top".into(),
            source: "site/index.md".into(),
            path: "/".into(),
        };
        let out = wrap_page_heading(&n, &p, vec![h1(vec![], vec![text("Top")])]);
        assert_eq!(out.len(), 1);
        assert!(!render(&out[0]).contains(PAGE_HEADING_CLASS));
    }

    #[test]
    fn moves_first_h1_only_and_supplies_missing() {
        let n = nav();
        let p = page(&n, "/guides/direct/");
        let blocks = vec![
            div(vec![], vec![text("intro")]),
            h1(vec![], vec![text("A")]),
            h1(vec![], vec![text("B")]),
        ];
        let out = wrap_page_heading(&n, &p, blocks);
        let first = render(&out[0]);
        assert!(first.starts_with("<header class=\"docs-page-heading\">"));
        assert!(first.contains("<h1>A</h1>"));
        assert!(first.find("Breadcrumb").unwrap() < first.find("<h1>").unwrap());
        assert!(!first.contains("<h2") && !first.contains("<h3"));
        assert_eq!(out.len(), 3);

        let out = wrap_page_heading(&n, &p, vec![div(vec![], vec![])]);
        assert!(render(&out[0]).contains("<h1>Direct</h1>"));
    }
}
