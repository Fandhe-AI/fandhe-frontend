//! Blocks・Wireframes 索引のカテゴリ別カード（イシュー #3618）の契約テスト。
//!
//! `crate::blocks::all_blocks()` / `crate::wireframes::WIREFRAMES` を、実
//! `site/nav.toml`・実サイトビルドの生成物・専用 CSS・原稿と突合し、ドリフトを
//! fail-closed に検知する（`component_index_nav.rs` と同型）。期待値は固定値を
//! 書かず、すべてレジストリから算出する。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::blocks::{self, BlockCategory, BlockSection};
use fandhe_frontend_docs_site::category_index::STYLESHEET_REL_PATH;
use fandhe_frontend_docs_site::layout::asset_href;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::wireframes::{WireframeCategory, WIREFRAMES};

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

/// 生成節（`docs-category-section` 以降）だけを返す。
fn generated(html: &str) -> &str {
    let at = html
        .find("class=\"docs-category-section\"")
        .expect("generated category sections");
    &html[at..]
}

/// `docs-category-card-list` 内のリンク href を出現順に集める。
fn list_hrefs(html: &str) -> Vec<String> {
    html.split("class=\"docs-category-card-list\">")
        .skip(1)
        .flat_map(|chunk| {
            let list = chunk.split("</ul>").next().unwrap_or("");
            list.split("href=\"")
                .skip(1)
                .map(|c| c.split('"').next().unwrap_or("").to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// 見出し（`<tag ...>text</tag>` の text）の直後にある badge 文字列を出現順に取り出す。
fn heads(html: &str, tag: &str) -> Vec<(String, String)> {
    let close = format!("</{tag}>");
    let open = format!("<{tag}");
    html.split(&open)
        .skip(1)
        .filter(|chunk| {
            // 生成節の見出しだけを対象にする（直後に badge span が続くもの）。
            chunk
                .split_once(&close)
                .is_some_and(|(_, after)| after.trim_start().starts_with("<span"))
        })
        .map(|chunk| {
            let (head, after) = chunk.split_once(&close).expect("close");
            let title = head.rsplit('>').next().unwrap_or("").to_string();
            let badge = after
                .split("</span>")
                .next()
                .and_then(|s| s.rsplit('>').next())
                .unwrap_or("")
                .to_string();
            (title, badge)
        })
        .collect()
}

#[test]
fn blocks_index_links_every_block_exactly_once_with_counted_headings() {
    let base = real_nav().site.base_path;
    let html = read_out("blocks/index.html");
    let gen = generated(&html);

    let mut got = list_hrefs(gen);
    let mut want: Vec<String> = blocks::all_blocks()
        .iter()
        .map(|b| asset_href(&base, b.path))
        .collect();
    got.sort();
    want.sort();
    assert_eq!(got, want, "block links vs all_blocks()");

    let all = blocks::all_blocks();
    let want_sections: Vec<(String, String)> = BlockSection::ALL
        .iter()
        .filter_map(|s| {
            let n = all.iter().filter(|b| b.category.section() == *s).count();
            (n > 0).then(|| (s.label().to_string(), format!("{n} 件")))
        })
        .collect();
    assert_eq!(heads(gen, "h2"), want_sections);

    let want_cats: Vec<(String, String)> = BlockSection::ALL
        .iter()
        .flat_map(|s| {
            BlockCategory::ALL
                .iter()
                .filter(move |c| c.section() == *s)
                .filter_map(|c| {
                    let n = all.iter().filter(|b| b.category == *c).count();
                    (n > 0).then(|| (c.label().to_string(), format!("{n} 件")))
                })
        })
        .collect();
    assert_eq!(heads(gen, "h3"), want_cats);
}

#[test]
fn wireframes_index_links_every_component_with_counted_categories() {
    let nav = real_nav();
    let base = nav.site.base_path.clone();
    let html = read_out("wireframes/index.html");
    let gen = generated(&html);

    let mut got = list_hrefs(gen);
    let mut want: Vec<String> = WIREFRAMES
        .iter()
        .map(|w| asset_href(&base, w.path))
        .collect();
    got.sort();
    want.sort();
    assert_eq!(got, want, "wireframe links vs WIREFRAMES");

    let nav_paths: BTreeSet<String> = nav
        .sections
        .iter()
        .find(|s| s.title == "Wireframes")
        .expect("Wireframes section")
        .all_pages()
        .filter(|p| p.path != "/wireframes/")
        .map(|p| asset_href(&base, &p.path))
        .collect();
    assert_eq!(got.into_iter().collect::<BTreeSet<_>>(), nav_paths);

    let want_cats: Vec<(String, String)> = WireframeCategory::ALL
        .iter()
        .map(|c| {
            let n = WIREFRAMES.iter().filter(|w| w.category == *c).count();
            assert!(n > 0, "category {} has no component", c.label());
            (c.label().replace('&', "&amp;"), format!("{n} 件"))
        })
        .collect();
    assert_eq!(heads(gen, "h3"), want_cats);
    assert_eq!(
        heads(gen, "h2"),
        vec![("部品一覧".to_string(), format!("{} 件", WIREFRAMES.len()))]
    );
}

#[test]
fn wireframes_manuscript_has_no_hand_written_link_list() {
    let md = std::fs::read_to_string(shared_site::repo_root().join("site/wireframes.md"))
        .expect("read site/wireframes.md");
    assert!(!md.contains("](./wireframes/"), "手書きリンク集は撤去済み");
}

#[test]
fn stylesheet_is_linked_only_from_the_two_index_pages() {
    let base = real_nav().site.base_path;
    let href = asset_href(&base, STYLESHEET_REL_PATH);
    assert!(shared_site::real_site()
        .out_dir
        .join(STYLESHEET_REL_PATH)
        .is_file());
    for (rel, expect) in [
        ("blocks/index.html", true),
        ("wireframes/index.html", true),
        ("blocks/login-01/index.html", false),
        ("wireframes/button/index.html", false),
        ("themes/index.html", false),
        ("index.html", false),
    ] {
        assert_eq!(read_out(rel).contains(&href), expect, "{rel}");
    }
}

#[test]
fn css_classes_and_generated_classes_match_both_ways() {
    let css = read_out(STYLESHEET_REL_PATH);
    assert!(
        !css.contains('#') && !css.contains("rgb(") && !css.contains("hsl("),
        "色リテラルを持たない"
    );
    let classes = |s: &str| -> BTreeSet<String> {
        s.split("docs-category-")
            .skip(1)
            .map(|c| {
                let name: String = c
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
                    .collect();
                format!("docs-category-{name}")
            })
            .collect()
    };
    let in_css = classes(&css);
    let mut in_html = BTreeSet::new();
    for rel in ["blocks/index.html", "wireframes/index.html"] {
        let html = read_out(rel);
        let gen = generated(&html);
        // class="..." の値だけを走査する。
        for v in gen.split("class=\"").skip(1) {
            let value = v.split('"').next().unwrap_or("");
            for c in value.split_whitespace() {
                if c.starts_with("docs-category-") {
                    in_html.insert(c.to_string());
                }
            }
        }
    }
    assert_eq!(in_css, in_html);
}

#[test]
fn generated_sections_have_no_script_handlers_or_ids() {
    for rel in ["blocks/index.html", "wireframes/index.html"] {
        let html = read_out(rel);
        let gen = generated(&html);
        let card_part: String = gen
            .split("class=\"docs-category-card-head\">")
            .skip(1)
            .map(|c| {
                let c = c
                    .split("class=\"docs-category-card-list\">")
                    .collect::<Vec<_>>();
                let head = c[0];
                let list = c.get(1).and_then(|l| l.split("</ul>").next()).unwrap_or("");
                format!("{head}{list}")
            })
            .collect();
        assert!(!card_part.contains("<script"), "{rel}");
        assert!(!card_part.contains("javascript:"), "{rel}");
        assert!(
            !card_part.contains(" onclick=") && !card_part.contains(" onload="),
            "{rel}"
        );
        assert!(
            !card_part.contains(" id=\""),
            "{rel}: card 内に id を出さない"
        );
    }
}

#[test]
fn blocks_search_index_keeps_section_names() {
    let json = read_out("assets/search-index/blocks.json");
    for s in BlockSection::ALL {
        assert!(json.contains(s.label()), "{}", s.label());
    }
}
