//! 本文先頭のパンくず付きページ見出しの実サイト契約（イシュー #3607）。
//!
//! `site/nav.toml` の全ページ（トップ `/` を除く）を共有ビルドし、パンくずが
//! `Nav` の階層（セクション / グループ / ページ）を指すこと、文書の h1
//! （`data-scope` を持たない素の `<h1>`）がページに 1 つで見出し部に入っていることを
//! 固定する。Themes / Blocks のデモ内の h1 は `data-scope="heading"` 付きのため
//! 数えない（文書の見出し構造に含まれない）。

use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::page_header::{BREADCRUMB_ARIA_LABEL, PAGE_HEADING_CLASS};

#[path = "support/shared_site.rs"]
mod shared_site;

fn nav() -> Nav {
    let path = shared_site::repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(path).expect("site/nav.toml readable");
    parse_nav(&input).expect("site/nav.toml parses")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

/// 見出し部（`<header class="docs-page-heading">` から `</header>` まで）を切り出す。
fn heading_part(html: &str) -> &str {
    let start = html
        .find(&format!("<header class=\"{PAGE_HEADING_CLASS}\">"))
        .expect("docs-page-heading が無い");
    let rest = &html[start..];
    &rest[..rest.find("</header>").expect("header が閉じない")]
}

#[test]
fn every_non_top_page_has_breadcrumb_heading_with_single_h1() {
    let nav = nav();
    let base = nav.site.base_path.clone();
    for section in &nav.sections {
        for page in section.all_pages() {
            if page.path == "/" {
                continue;
            }
            let html = page_html(&page.path);
            assert_eq!(
                html.matches(&format!("class=\"{PAGE_HEADING_CLASS}\""))
                    .count(),
                1,
                "{}: 見出し部は 1 つ",
                page.path
            );
            let head = heading_part(&html);
            assert!(
                head.contains(&format!("aria-label=\"{BREADCRUMB_ARIA_LABEL}\"")),
                "{}: パンくず nav が無い",
                page.path
            );
            if section.index_path == page.path {
                assert!(
                    !head.contains("<a "),
                    "{}: 索引ページは自分へのリンクを作らない",
                    page.path
                );
            } else {
                let href = format!("href=\"{base}{}\"", section.index_path);
                assert!(head.contains(&href), "{}: セクションへのリンク", page.path);
                if let Some(g) = section
                    .groups
                    .iter()
                    .find(|g| g.pages.iter().any(|p| p.path == page.path))
                {
                    assert!(
                        head.contains(&format!("<span>{}</span>", esc(&g.title))),
                        "{}: グループ名",
                        page.path
                    );
                }
            }
            let current = if section.index_path == page.path {
                esc(&section.title)
            } else {
                esc(&page.title)
            };
            assert!(
                head.contains(&format!("{current}</span></li>")),
                "{}: 末尾が現在項目でない",
                page.path
            );
            assert!(head.contains("aria-current=\"page\""));
            assert_eq!(
                html.matches("<h1>").count(),
                1,
                "{}: data-scope 外の h1 は 1 つ",
                page.path
            );
            assert!(
                head.find("Breadcrumb").unwrap() < head.find("<h1>").expect("h1 は見出し部内"),
                "{}: h1 はパンくずの後",
                page.path
            );
        }
    }
}

#[test]
fn top_page_has_no_page_heading() {
    let html = page_html("/");
    assert!(!html.contains(PAGE_HEADING_CLASS));
}

fn expect_trail(path: &str, expected: &[&str]) {
    let html = page_html(path);
    let head = heading_part(&html);
    let mut pos = 0;
    for e in expected {
        let found = head[pos..]
            .find(e)
            .unwrap_or_else(|| panic!("{path}: {e:?} が順序どおりに無い: {head}"));
        pos += found + e.len();
    }
}

#[test]
fn breadcrumb_trails_per_page_kind() {
    let b = "/fandhe-frontend";
    // 索引ページ
    expect_trail("/guides/", &["aria-current=\"page\"", "Guides</span>"]);
    // Guides（直下）
    expect_trail(
        "/guides/component-authoring/",
        &[
            &format!("href=\"{b}/guides/\""),
            ">Guides</a>",
            "aria-current=\"page\"",
            "コンポーネント記述ガイド</span>",
        ],
    );
    // API
    expect_trail(
        "/api/server-api/",
        &[
            &format!("href=\"{b}/api/\""),
            ">API Reference</a>",
            "fandhe-frontend-server SSG API</span>",
        ],
    );
    // Themes（グループ）
    expect_trail(
        "/themes/button/",
        &[
            &format!("href=\"{b}/themes/\""),
            ">Themes</a>",
            "<span>Forms</span>",
            "Button</span>",
        ],
    );
    // Primitives（グループ）
    expect_trail(
        "/primitives/accordion/",
        &[
            &format!("href=\"{b}/primitives/\""),
            ">Primitives</a>",
            "<span>Overlay / Disclosure</span>",
            "Accordion</span>",
        ],
    );
    // Blocks（グループ）
    expect_trail(
        "/blocks/login-01/",
        &[
            &format!("href=\"{b}/blocks/\""),
            ">Blocks</a>",
            "<span>Auth</span>",
            "login-01</span>",
        ],
    );
    // Wireframes
    expect_trail(
        "/wireframes/button/",
        &[
            &format!("href=\"{b}/wireframes/\""),
            ">Wireframes</a>",
            "Button</span>",
        ],
    );
    // Examples
    expect_trail(
        "/examples/ssr-routing/",
        &[
            &format!("href=\"{b}/examples/\""),
            ">Examples</a>",
            "ssr-routing</span>",
        ],
    );
}

/// 見出し部（パンくず + 移設した h1）を足しても検索インデックスの本文テキストが
/// 変わらないこと。パンくずは `data-scope` で除外され、h1 は従来どおり本文に残る。
#[test]
fn page_heading_does_not_change_search_index_text() {
    use fandhe_frontend_core::{div, h1, h2, p, text};
    use fandhe_frontend_docs_site::page_header::wrap_page_heading;
    use fandhe_frontend_docs_site::search_index::page_entry;

    let nav = nav();
    let page = nav
        .sections
        .iter()
        .flat_map(|s| s.all_pages())
        .find(|p| p.path == "/guides/component-authoring/")
        .cloned()
        .unwrap();
    let blocks = || {
        vec![
            h1(vec![], vec![text("Title")]),
            p(vec![], vec![text("body")]),
            h2(vec![], vec![text("Sec")]),
        ]
    };
    let before = page_entry("/x/", "T", &div(vec![], blocks()), false);
    let after = page_entry(
        "/x/",
        "T",
        &div(vec![], wrap_page_heading(&nav, &page, blocks())),
        false,
    );
    assert_eq!(before.text, after.text);
    assert_eq!(before.sections.len(), after.sections.len());
}

#[test]
fn blocks_heading_precedes_demo_section() {
    let html = page_html("/blocks/login-01/");
    let head = html.find(PAGE_HEADING_CLASS).expect("見出し部");
    let demo = html.find(">Demo</h2>").expect("Demo 節");
    assert!(head < demo);
}
