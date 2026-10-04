//! サイトフッターの実サイト契約（イシュー #3609）。
//!
//! `site/nav.toml` の全ページを共有ビルドし、フッターが全ページにちょうど 1 つあり
//! `main` の外（`<body>` 末尾）にあること、リダイレクト案内ページには無いこと、
//! 検索インデックスへ混入しないことを固定する。

use fandhe_frontend_docs_site::nav::{parse_nav, Nav};

#[path = "support/shared_site.rs"]
mod shared_site;

fn nav() -> Nav {
    let path = shared_site::repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(path).expect("site/nav.toml readable");
    parse_nav(&input).expect("site/nav.toml parses")
}

fn page_html(path: &str) -> String {
    let rel = path.trim_matches('/');
    let file = if rel.is_empty() {
        "index.html".to_string()
    } else {
        format!("{rel}/index.html")
    };
    std::fs::read_to_string(shared_site::real_site().out_dir.join(&file))
        .unwrap_or_else(|e| panic!("{file} should be generated: {e}"))
}

#[test]
fn every_page_has_exactly_one_footer_outside_main_at_body_end() {
    let nav = nav();
    for page in nav.all_pages() {
        let html = page_html(&page.path);
        assert_eq!(
            html.matches("<footer class=\"docs-footer\"").count(),
            1,
            "{}: docs-footer は 1 つ",
            page.path
        );
        let footer = html.rfind("<footer class=\"docs-footer\"").unwrap();
        let main_end = html.rfind("</main>").expect("main end");
        assert!(main_end < footer, "{}: footer が main の内側", page.path);
        assert!(
            html[footer..].contains("</footer></body>"),
            "{}: footer が body 末尾でない",
            page.path
        );
        assert!(
            !html.contains("role=\"contentinfo\""),
            "{}: contentinfo は暗黙ロールのみ",
            page.path
        );
    }
}

#[test]
fn footer_links_each_section_and_menu_index_exactly_once() {
    let nav = nav();
    let html = page_html("/");
    let footer = &html[html.rfind("<footer class=\"docs-footer\"").unwrap()..];
    let base = &nav.site.base_path;
    let indexes = nav
        .sections
        .iter()
        .map(|s| s.index_path.as_str())
        .chain(nav.menus.iter().map(|m| m.index_path.as_str()));
    for path in indexes {
        let href = format!(
            "href=\"{}\"",
            fandhe_frontend_docs_site::layout::asset_href(base, path)
        );
        assert_eq!(
            footer.matches(&href).count(),
            1,
            "{path} の索引リンクは 1 回"
        );
    }
    let prefix = format!("href=\"{base}/");
    assert_eq!(
        footer.matches(&prefix).count(),
        nav.sections.len() + nav.menus.len(),
        "直下ページがフッターへ混入している"
    );
    assert!(footer.contains("Licensed under"));
    assert!(footer.contains("https://crates.io/crates/fandhe-frontend-core"));
    assert!(footer.contains("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""));
}

#[test]
fn redirect_pages_have_no_footer() {
    let out = &shared_site::real_site().out_dir;
    let file = out.join("components/index.html");
    let html = std::fs::read_to_string(file).expect("redirect page should exist");
    assert!(!html.contains("docs-footer"));
}

#[test]
fn footer_text_is_not_in_search_index() {
    let out = &shared_site::real_site().out_dir;
    let assets = out.join("assets");
    let mut files = vec![assets.join("search-index.json")];
    for entry in std::fs::read_dir(assets.join("search-index"))
        .expect("search-index dir")
        .flatten()
    {
        files.push(entry.path());
    }
    for f in files {
        let body = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f:?}: {e}"));
        assert!(!body.contains("Licensed under"), "{f:?}");
        assert!(
            !body.contains(fandhe_frontend_docs_site::site_footer::FOOTER_TAGLINE),
            "{f:?}"
        );
    }
}
