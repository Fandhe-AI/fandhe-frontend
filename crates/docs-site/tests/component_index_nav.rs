//! Themes・Primitives 索引カードグリッド（イシュー #3617）の契約テスト。
//!
//! `crate::themes_catalog` / `crate::primitives_catalog` の台帳を、実
//! `site/nav.toml`・実サイトビルドの生成物・専用 CSS・原稿と突合し、ドリフトを
//! fail-closed に検知する（`section_index_nav.rs` / `primitives_nav.rs` と同型の
//! 三方突合）。カード内の説明文は検索インデックス対象外とする設計（モジュール
//! doc 参照）のため、本テストも説明文の検索網羅は求めない。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::component_index::STYLESHEET_REL_PATH;
use fandhe_frontend_docs_site::component_page;
use fandhe_frontend_docs_site::layout::asset_href;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::primitives_catalog::{self, PrimitiveCategory};
use fandhe_frontend_docs_site::themes_catalog::{self, ThemeCategory};

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

fn section_pages(nav: &Nav, title: &str, index_path: &str) -> Vec<(String, String)> {
    let s = nav
        .sections
        .iter()
        .find(|s| s.title == title)
        .unwrap_or_else(|| panic!("nav section {title} missing"));
    assert_eq!(s.index_path, index_path);
    s.all_pages()
        .filter(|p| p.path != index_path)
        .map(|p| (p.path.clone(), p.title.clone()))
        .collect()
}

#[test]
fn themes_catalog_matches_nav_groups_in_order() {
    let nav = real_nav();
    let s = nav
        .sections
        .iter()
        .find(|s| s.title == "Themes")
        .expect("Themes section");
    let group_titles: Vec<&str> = s.groups.iter().map(|g| g.title.as_str()).collect();
    let cat_titles: Vec<&str> = ThemeCategory::all().iter().map(|c| c.title()).collect();
    assert_eq!(group_titles, cat_titles);
    for (g, cat) in s.groups.iter().zip(ThemeCategory::all()) {
        let nav_pages: Vec<(&str, &str)> = g
            .pages
            .iter()
            .map(|p| (p.path.as_str(), p.title.as_str()))
            .collect();
        let ledger: Vec<(&str, &str)> = themes_catalog::entries_in(*cat)
            .map(|e| (e.path, e.title))
            .collect();
        assert_eq!(ledger, nav_pages, "group {}", g.title);
    }
    assert_eq!(themes_catalog::THEMES.len(), 123);
}

#[test]
fn themes_catalog_matches_manuscripts_and_generated_pages() {
    let root = shared_site::repo_root();
    let mut files = BTreeSet::new();
    for e in std::fs::read_dir(root.join("site/themes")).expect("read site/themes") {
        let p = e.expect("entry").path();
        if p.extension().is_some_and(|x| x == "md") {
            files.insert(p.file_stem().unwrap().to_string_lossy().into_owned());
        }
    }
    let ledger: BTreeSet<String> = themes_catalog::entries()
        .map(|e| {
            e.path
                .trim_matches('/')
                .rsplit('/')
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(ledger, files);
    for e in themes_catalog::entries() {
        assert!(
            component_page::generated_content(e.path).is_some(),
            "{}",
            e.path
        );
    }
}

#[test]
fn primitives_catalog_paths_match_nav() {
    let nav = real_nav();
    let nav_paths: BTreeSet<String> = section_pages(&nav, "Primitives", "/primitives/")
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    let ledger: BTreeSet<String> = primitives_catalog::page_paths()
        .map(str::to_string)
        .collect();
    assert_eq!(ledger, nav_paths);
}

#[test]
fn descriptions_are_single_short_plain_lines() {
    let all = themes_catalog::entries()
        .map(|e| (e.path, e.description))
        .chain(primitives_catalog::entries().map(|e| (e.path, e.description)));
    for (path, d) in all {
        assert!(!d.trim().is_empty(), "{path}");
        assert!(!d.contains('\n'), "{path}");
        assert!(d.chars().count() <= 80, "{path}");
        assert!(!d.contains('`'), "{path}: backtick");
        assert!(!d.contains('#'), "{path}: internal issue number");
    }
}

/// カテゴリ見出し行（h2 の題名と件数 badge 文字列）を出現順に取り出す。
fn category_heads(html: &str) -> Vec<(String, String)> {
    html.split("docs-catalog-category-head\">")
        .skip(1)
        .map(|chunk| {
            let h2 = chunk.split("</h2>").next().expect("h2 end");
            let title = h2.rsplit('>').next().expect("h2 text").to_string();
            let after = chunk.split("</h2>").nth(1).expect("after h2");
            let badge = after
                .split("</span>")
                .next()
                .and_then(|s| s.rsplit('>').next())
                .expect("badge text")
                .to_string();
            (title, badge)
        })
        .collect()
}

fn card_hrefs(html: &str) -> Vec<String> {
    html.split("class=\"docs-catalog-card-link\"")
        .skip(1)
        .map(|chunk| {
            let after = chunk.split("href=\"").nth(1).expect("href after class");
            after.split('"').next().expect("href end").to_string()
        })
        .collect()
}

#[test]
fn generated_pages_link_every_part_exactly_once_with_counted_headings() {
    let nav = real_nav();
    let base = nav.site.base_path.clone();
    let cases = [("Themes", "/themes/"), ("Primitives", "/primitives/")];
    for (section, index_path) in cases {
        let rel = format!("{}index.html", index_path.trim_start_matches('/'));
        let html = read_out(&rel);
        let mut hrefs = card_hrefs(&html);
        let mut expected: Vec<String> = section_pages(&nav, section, index_path)
            .iter()
            .map(|(p, _)| asset_href(&base, p))
            .collect();
        hrefs.sort();
        expected.sort();
        assert_eq!(hrefs, expected, "{section} cards vs nav pages");
        assert!(html.contains(&asset_href(&base, STYLESHEET_REL_PATH)));

        // 層 badge は部品数と一致する。
        let layer = format!(">{section}</span>");
        // 周辺の同名テキストを除くため、カードグリッド（ul）の内側だけを数える。
        let generated = html
            .split("<ul class=\"docs-catalog-grid\">")
            .skip(1)
            .map(|c| c.split("</ul>").next().unwrap_or(""))
            .collect::<String>();
        assert_eq!(
            generated.matches(&layer).count(),
            expected.len(),
            "{section}"
        );

        let heads = category_heads(&html);
        let want: Vec<(String, String)> = if section == "Themes" {
            ThemeCategory::all()
                .iter()
                .map(|c| {
                    (
                        c.title().to_string(),
                        format!("{} 件", themes_catalog::entries_in(*c).count()),
                    )
                })
                .collect()
        } else {
            PrimitiveCategory::all()
                .iter()
                .map(|c| {
                    (
                        c.title().to_string(),
                        format!("{} 件", primitives_catalog::entries_in(*c).count()),
                    )
                })
                .collect()
        };
        assert_eq!(heads, want, "{section} category headings and counts");
        assert_eq!(heads.len(), 6);
    }
}

#[test]
fn stylesheet_is_linked_only_from_the_two_index_pages() {
    let out = &shared_site::real_site().out_dir;
    let needle = "assets/component-index.css";
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
    walk(out, out, needle, &mut linked);
    let expected: BTreeSet<String> = ["themes/index.html", "primitives/index.html"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(linked, expected);
    assert!(
        out.join(needle).is_file(),
        "component-index.css not emitted"
    );
}

#[test]
fn primitives_index_uses_site_css_not_the_showcase_variant() {
    let html = read_out("primitives/index.html");
    assert!(html.contains("assets/site.css"));
    assert!(!html.contains("site-primitives.css"));
}

#[test]
fn manuscripts_no_longer_hand_list_parts() {
    let root = shared_site::repo_root();
    let themes = std::fs::read_to_string(root.join("site/themes.md")).expect("themes.md");
    let prims = std::fs::read_to_string(root.join("site/primitives.md")).expect("primitives.md");
    assert!(!themes.contains("](./themes/"), "site/themes.md");
    assert!(!prims.contains("](./primitives/"), "site/primitives.md");
    for c in ThemeCategory::all() {
        assert!(
            !themes.contains(&format!("\n## {}\n", c.title())),
            "site/themes.md: {}",
            c.title()
        );
    }
    for c in PrimitiveCategory::all() {
        assert!(
            !prims.contains(&format!("\n## {}\n", c.title())),
            "site/primitives.md: {}",
            c.title()
        );
    }
}

#[test]
fn generated_css_and_html_class_sets_agree_and_hold_no_literal_colors() {
    use fandhe_frontend_docs_site::component_index::stylesheet;
    let css = stylesheet().expect("stylesheet").as_css().to_string();
    assert!(!css.contains('#') && !css.contains("rgb(") && !css.contains("hsl("));

    fn classes(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut rest = s;
        while let Some(i) = rest.find("docs-catalog-") {
            let tail = &rest[i..];
            let end = tail
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                .unwrap_or(tail.len());
            out.insert(tail[..end].to_string());
            rest = &tail[end..];
        }
        out
    }
    let html = read_out("themes/index.html");
    // 生成 HTML 側は class 属性値だけを対象にする（href 等の誤拾い防止）。
    let mut html_classes = BTreeSet::new();
    for chunk in html.split("class=\"").skip(1) {
        let val = chunk.split('"').next().unwrap_or("");
        html_classes.extend(classes(val));
    }
    assert_eq!(classes(&css), html_classes);
}

#[test]
fn generated_html_is_script_free_and_has_no_event_attrs_or_ids_in_cards() {
    for rel in ["themes/index.html", "primitives/index.html"] {
        let html = read_out(rel);
        let body = html
            .split("docs-catalog-category-head")
            .skip(1)
            .collect::<Vec<_>>()
            .join("");
        assert!(!body.contains("javascript:"), "{rel}");
        assert!(!body.contains("<script"), "{rel}");
        assert!(
            !body.contains(" onclick=") && !body.contains(" onload="),
            "{rel}"
        );
        // 生成節の id は見出しアンカー（h2）のみ。カード側には出さない。
        let in_cards = body.split("<ul class=\"docs-catalog-grid\">").skip(1);
        for chunk in in_cards {
            let ul = chunk.split("</ul>").next().unwrap_or("");
            assert!(!ul.contains(" id=\""), "{rel}");
        }
    }
}

#[test]
fn search_index_keeps_titles_headings_and_legend() {
    let themes = read_out("assets/search-index/themes.json");
    for c in ThemeCategory::all() {
        assert!(themes.contains(c.title()), "themes: {}", c.title());
    }
    assert!(themes.contains("掲示の読み方"));
    let prims = read_out("assets/search-index/primitives.json");
    for c in PrimitiveCategory::all() {
        assert!(prims.contains(c.title()), "primitives: {}", c.title());
    }
}
