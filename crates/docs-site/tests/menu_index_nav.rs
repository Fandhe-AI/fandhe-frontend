//! メニュー集約ページ `/assets/`（イシュー #3700）の契約テスト。
//!
//! `site/nav.toml` の `[[menu]]`（宣言順・説明・メンバー）を唯一の正として、
//! 実サイトビルドの生成物が (1) カードを宣言順に並べる、(2) Landing 骨格で
//! サイドバー（表示）・右目次・前後ナビを持たない、(3) どのページの
//! サイドバー・前後ナビにも現れない、ことを固定する。

use fandhe_frontend_docs_site::layout::asset_href;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};

#[path = "support/shared_site.rs"]
mod shared_site;

fn real_nav() -> Nav {
    let toml = std::fs::read_to_string(shared_site::repo_root().join("site/nav.toml"))
        .expect("read site/nav.toml");
    parse_nav(&toml).expect("parse site/nav.toml")
}

fn read_out(rel: &str) -> String {
    std::fs::read_to_string(shared_site::real_site().out_dir.join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// `<a class="docs-index-card-link" href="H">T</a>` を出現順に `(H, T)` で返す。
fn card_links(html: &str) -> Vec<(String, String)> {
    let needle = "<a class=\"docs-index-card-link\" href=\"";
    html.split(needle)
        .skip(1)
        .map(|chunk| {
            let (href, rest) = chunk.split_once("\">").expect("href end");
            let (title, _) = rest.split_once("</a>").expect("anchor end");
            (href.to_string(), title.to_string())
        })
        .collect()
}

#[test]
fn nav_declares_the_assets_menu_with_five_members_in_panel_order() {
    let nav = real_nav();
    assert_eq!(nav.menus.len(), 1);
    let menu = &nav.menus[0];
    assert_eq!(menu.index_path, "/assets/");
    let order: Vec<&str> = menu.items.iter().map(|i| i.section.as_str()).collect();
    assert_eq!(
        order,
        [
            "/primitives/",
            "/themes/",
            "/blocks/",
            "/wireframes/",
            "/examples/"
        ]
    );
}

/// 宣言順とカード順（リンク先・タイトル・説明）の一致。
#[test]
fn cards_match_menu_declaration_order_titles_and_descriptions() {
    let nav = real_nav();
    let menu = &nav.menus[0];
    let html = read_out("assets/index.html");
    let expected: Vec<(String, String)> = nav
        .menu_members(menu)
        .map(|(section, _)| {
            (
                asset_href(&nav.site.base_path, &section.index_path),
                section.title.clone(),
            )
        })
        .collect();
    assert_eq!(expected.len(), 5);
    assert_eq!(card_links(&html), expected);
    for item in &menu.items {
        assert!(
            html.contains(item.description.as_str()),
            "description missing: {}",
            item.description
        );
    }
}

#[test]
fn page_uses_landing_layout_without_toc_or_prev_next_and_has_single_breadcrumb() {
    let html = read_out("assets/index.html");
    assert!(html.contains("docs-container docs-landing"));
    for absent in [
        "docs-toc-aside",
        "docs-toc-inline",
        "class=\"prev-next",
        "<nav class=\"prev-next",
    ] {
        assert!(!html.contains(absent), "must not contain {absent}");
    }
    let crumb_start = html
        .find("docs-page-breadcrumb")
        .expect("breadcrumb wrapper");
    let crumb = &html[crumb_start..];
    let crumb = &crumb[..crumb.find("</nav>").expect("breadcrumb end")];
    assert_eq!(crumb.matches("aria-current=\"page\"").count(), 1);
    assert!(crumb.contains("Assets"));
    assert!(!crumb.contains("<a "));
}

fn walk_html(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("read_dir") {
        let p = e.expect("entry").path();
        if p.is_dir() {
            walk_html(&p, out);
        } else if p.extension().is_some_and(|x| x == "html") {
            out.push(p);
        }
    }
}

/// `/assets/` はどのページのサイドバー・前後ナビにも出ない（ヘッダー・
/// ドロワー・フッター等の別導線は #3701〜#3703 の対象でここでは問わない）。
#[test]
fn menu_page_is_absent_from_every_sidebar_and_prev_next() {
    let nav = real_nav();
    let href = format!("href=\"{}\"", asset_href(&nav.site.base_path, "/assets/"));
    let out = shared_site::real_site().out_dir.as_path();
    let mut files = Vec::new();
    walk_html(out, &mut files);
    assert!(files.len() > 100);
    for f in files {
        let html = std::fs::read_to_string(&f).unwrap_or_default();
        for (start, end) in [
            ("<aside class=\"docs-sidebar\"", "</aside>"),
            ("<nav class=\"prev-next\"", "</nav>"),
        ] {
            if let Some(i) = html.find(start) {
                let range = &html[i..];
                let range = &range[..range.find(end).unwrap_or(range.len())];
                assert!(!range.contains(&href), "{} links /assets/", f.display());
            }
        }
    }
}
