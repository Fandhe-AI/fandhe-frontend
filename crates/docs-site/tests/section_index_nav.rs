//! セクション索引カードグリッド（イシュー #3616）の契約テスト。
//!
//! `crate::section_index` の定数台帳（Guides / API Reference / Examples の
//! 3 セクションのトップ）を、実 `site/nav.toml`・実サイトビルドの生成物・
//! 専用 CSS・原稿と突合し、ドリフトを fail-closed に検知する
//! （`primitives_nav.rs` / `blocks_nav.rs` と同型の三方突合）。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::layout::asset_href;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::section_index::{
    self, IndexCard, API_GROUPS, EXAMPLES, GUIDES, STYLESHEET_REL_PATH,
};

#[path = "support/shared_site.rs"]
mod shared_site;

/// (セクション nav タイトル, index_path, 台帳)
fn ledgers() -> Vec<(&'static str, &'static str, Vec<IndexCard>)> {
    vec![
        ("Guides", "/guides/", GUIDES.to_vec()),
        ("Examples", "/examples/", EXAMPLES.to_vec()),
        (
            "API Reference",
            "/api/",
            API_GROUPS
                .iter()
                .flat_map(|g| g.cards.iter().copied())
                .collect(),
        ),
    ]
}

fn real_nav() -> Nav {
    let toml = std::fs::read_to_string(shared_site::repo_root().join("site/nav.toml"))
        .expect("read site/nav.toml");
    parse_nav(&toml).expect("parse site/nav.toml")
}

/// セクション配下のページ（`index_path` 自身を除く）を宣言順で返す。
fn nav_children(nav: &Nav, section: &str, index_path: &str) -> Vec<(String, String)> {
    let s = nav
        .sections
        .iter()
        .find(|s| s.title == section)
        .unwrap_or_else(|| panic!("nav section {section} missing"));
    assert_eq!(s.index_path, index_path);
    s.all_pages()
        .filter(|p| p.path != index_path)
        .map(|p| (p.path.clone(), p.title.clone()))
        .collect()
}

#[test]
fn ledger_matches_nav_children() {
    let nav = real_nav();
    for (section, index_path, cards) in ledgers() {
        let expected = nav_children(&nav, section, index_path);
        let mut actual: Vec<(String, String)> = cards
            .iter()
            .map(|c| (c.path.to_string(), c.title.to_string()))
            .collect();
        if section == "API Reference" {
            // API はグループ化されるため順序ではなく集合で一致させる。
            let mut e = expected.clone();
            e.sort();
            actual.sort();
            assert_eq!(actual, e, "{section}");
            let uniq: BTreeSet<_> = actual.iter().collect();
            assert_eq!(uniq.len(), actual.len(), "duplicate card in {section}");
        } else {
            assert_eq!(actual, expected, "{section}");
        }
    }
}

#[test]
fn descriptions_are_nonempty_single_short_lines() {
    for (section, _, cards) in ledgers() {
        for c in cards {
            assert!(!c.description.trim().is_empty(), "{section}: {}", c.path);
            assert!(!c.description.contains('\n'), "{section}: {}", c.path);
            assert!(c.description.chars().count() <= 80, "{section}: {}", c.path);
        }
    }
}

fn read_out(rel: &str) -> String {
    std::fs::read_to_string(shared_site::real_site().out_dir.join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn card_hrefs(html: &str) -> Vec<String> {
    html.split("class=\"docs-index-card-link\"")
        .skip(1)
        .map(|chunk| {
            let after = chunk.split("href=\"").nth(1).expect("href after class");
            after.split('"').next().expect("href end").to_string()
        })
        .collect()
}

#[test]
fn generated_pages_link_every_child_exactly_once() {
    let nav = real_nav();
    let base = nav.site.base_path.clone();
    for (section, index_path, _) in ledgers() {
        let rel = format!("{}index.html", index_path.trim_start_matches('/'));
        let html = read_out(&rel);
        let mut hrefs = card_hrefs(&html);
        let mut expected: Vec<String> = nav_children(&nav, section, index_path)
            .iter()
            .map(|(p, _)| asset_href(&base, p))
            .collect();
        hrefs.sort();
        expected.sort();
        assert_eq!(hrefs, expected, "{section} cards vs nav children");
        assert!(html.contains(&asset_href(&base, STYLESHEET_REL_PATH)));
    }
}

#[test]
fn stylesheet_is_linked_only_from_the_three_index_pages() {
    let out = &shared_site::real_site().out_dir;
    let href_part = "assets/section-index.css";
    let mut linked = BTreeSet::new();
    fn walk(
        dir: &std::path::Path,
        root: &std::path::Path,
        needle: &str,
        out: &mut BTreeSet<String>,
    ) {
        for e in std::fs::read_dir(dir).expect("read_dir") {
            let p = e.expect("entry").path();
            if p.is_dir() {
                walk(&p, root, needle, out);
            } else if p.extension().is_some_and(|x| x == "html") {
                let s = std::fs::read_to_string(&p).unwrap_or_default();
                if s.contains(needle) {
                    out.insert(p.strip_prefix(root).unwrap().to_string_lossy().into_owned());
                }
            }
        }
    }
    walk(out, out, href_part, &mut linked);
    let expected: BTreeSet<String> = ["guides/index.html", "api/index.html", "examples/index.html"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(linked, expected);
    assert!(
        out.join(href_part).is_file(),
        "section-index.css not emitted"
    );
}

#[test]
fn manuscripts_no_longer_hand_list_children() {
    let root = shared_site::repo_root();
    let guides = std::fs::read_to_string(root.join("site/guides.md")).expect("guides.md");
    // 残してよいのはリード段落のクイックスタートへのリンク 1 件のみ。
    assert!(
        guides.matches("](../docs/").count() <= 1,
        "site/guides.md: hand-written child list remains"
    );
    let api = std::fs::read_to_string(root.join("site/api.md")).expect("api.md");
    assert!(
        !api.contains("](../docs/api/"),
        "site/api.md: hand-written API link remains"
    );
}

#[test]
fn search_index_keeps_the_three_section_tops() {
    for (file, title) in [
        ("guides", "ガイド一覧"),
        ("api-reference", "API リファレンス"),
        ("examples", "サンプル集"),
    ] {
        let json = read_out(&format!("assets/search-index/{file}.json"));
        assert!(
            json.contains(title),
            "{file}: title missing from search index"
        );
    }
    let api = read_out("assets/search-index/api-reference.json");
    for g in API_GROUPS {
        assert!(
            api.contains(g.title),
            "group heading {} not indexed",
            g.title
        );
    }
}

#[test]
fn search_index_keeps_card_titles_and_descriptions() {
    use fandhe_frontend_docs_site::section_index::{EXAMPLES, GUIDES};
    let cards = |file: &str, cs: &[fandhe_frontend_docs_site::section_index::IndexCard]| {
        let json = read_out(&format!("assets/search-index/{file}.json"));
        for c in cs {
            assert!(
                json.contains(c.title),
                "{file}: {} title not indexed",
                c.path
            );
            assert!(
                json.contains(c.description),
                "{file}: {} description not indexed",
                c.path
            );
        }
    };
    cards("guides", GUIDES);
    cards("examples", EXAMPLES);
    let flat: Vec<_> = API_GROUPS
        .iter()
        .flat_map(|g| g.cards.iter().copied())
        .collect();
    cards("api-reference", &flat);
}

fn class_tokens(html: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for chunk in html.split("class=\"").skip(1) {
        let v = chunk.split('"').next().unwrap_or_default();
        for t in v.split_whitespace() {
            if t.starts_with("docs-index-") {
                out.insert(t.to_string());
            }
        }
    }
    out
}

#[test]
fn generated_classes_and_stylesheet_selectors_match_both_ways() {
    let mut html = String::new();
    for f in [
        section_index::render_guides,
        section_index::render_examples,
        section_index::render_api,
    ] {
        for n in f("/b") {
            html.push_str(&fandhe_frontend_core::render(&n));
        }
    }
    let used = class_tokens(&html);
    let css = section_index::stylesheet()
        .expect("stylesheet")
        .as_css()
        .to_string();
    let mut in_css = BTreeSet::new();
    for chunk in css.split(".docs-index-").skip(1) {
        let name: String = chunk
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        in_css.insert(format!("docs-index-{name}"));
    }
    assert_eq!(used, in_css);
    assert!(!css.contains('#'), "no hex color literals or id selectors");
    assert!(!css.contains("rgb(") && !css.contains("hsl("));
}

#[test]
fn generated_html_is_script_free() {
    let mut html = String::new();
    for f in [
        section_index::render_guides,
        section_index::render_examples,
        section_index::render_api,
    ] {
        for n in f("/b") {
            html.push_str(&fandhe_frontend_core::render(&n));
        }
    }
    assert!(!html.contains("javascript:") && !html.contains("<script"));
    assert!(!html.contains(" on") || !html.contains("=\"on"));
    assert!(!html.contains(" id=\""));
}
