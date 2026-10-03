//! ヘッダー popup（`header_nav`）とサイドバー（`sidebar`）の見出し一覧の一致、
//! およびグループアンカーの実在・一意性を実サイト（`site/nav.toml`）で固定する
//! （イシュー #3670）。
//!
//! 両者は `Section::headings`（唯一の情報源）から見出しを作る。ここでは描画結果の
//! HTML から項目列（表記・順序）を取り出して比較し、構造の共有が崩れたら落とす。
//! アンカーの実在はビルド時のリンク検証（`linkcheck`）も fail-closed に守る。

use std::collections::BTreeSet;

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::layout::RESERVED_LAYOUT_IDS;
use fandhe_frontend_docs_site::nav::{group_anchor_id, header_nav, parse_nav, sidebar, Nav};

#[path = "support/shared_site.rs"]
mod shared_site;

fn load_nav() -> Nav {
    let path = shared_site::repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(path).expect("site/nav.toml should be readable");
    parse_nav(&input).expect("site/nav.toml should parse")
}

/// `<a ...>text</a>` の text を出現順に集める（描画は既定エスケープ済みなので
/// エスケープ後の文字列同士で比較できる）。
fn anchor_texts(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find("<a ") {
        rest = &rest[i..];
        let Some(gt) = rest.find('>') else { break };
        let Some(end) = rest.find("</a>") else { break };
        out.push(rest[gt + 1..end].to_string());
        rest = &rest[end + 4..];
    }
    out
}

/// サイドバーの見出し列: 直下ページ（最初の `<details` より前の `a`）→ グループ名。
fn sidebar_headings(html: &str) -> Vec<String> {
    let head_end = html.find("<details").unwrap_or(html.len());
    let mut out = anchor_texts(&html[..head_end]);
    let marker = "docs-nav-group-title\">";
    let mut rest = html;
    while let Some(i) = rest.find(marker) {
        rest = &rest[i + marker.len()..];
        let end = rest.find('<').expect("group title should be closed");
        out.push(rest[..end].to_string());
    }
    out
}

/// `section_idx` 番目のセクションの popup 項目（`ul.docs-header-dropdown` 内の `a`）。
fn popup_items(header_html: &str, section_idx: usize) -> Vec<String> {
    let seg = header_html
        .split("class=\"docs-header-group\"")
        .nth(section_idx + 1)
        .expect("header group should exist");
    let dd = seg
        .find("docs-header-dropdown")
        .expect("dropdown should exist");
    // split 済みなので seg は当該セクション内。トリガー `a` は dropdown より前。
    anchor_texts(&seg[dd..])
}

#[test]
fn popup_matches_sidebar_headings_for_every_section() {
    let nav = load_nav();
    assert_eq!(nav.sections.len(), 8);
    for (idx, section) in nav.sections.iter().enumerate() {
        // 先頭ページと、グループがあれば先頭グループ配下の 1 ページで検証する。
        let mut currents = vec![section.all_pages().next().unwrap().path.clone()];
        if let Some(g) = section.groups.first() {
            currents.push(g.pages[0].path.clone());
        }
        for current in currents {
            let side = sidebar_headings(&render(&sidebar(&nav, &current)));
            let popup = popup_items(&render(&header_nav(&nav, &current)), idx);
            assert_eq!(side, popup, "section {:?} current {current}", section.title);
        }
    }
}

#[test]
fn group_heading_hrefs_point_to_index_anchor_and_ids_are_unique() {
    let nav = load_nav();
    for section in &nav.sections {
        if section.groups.is_empty() {
            continue;
        }
        let html = render(&header_nav(&nav, &section.index_path));
        let mut ids = BTreeSet::new();
        for g in &section.groups {
            let id = group_anchor_id(&g.title);
            let href = format!(
                "href=\"{}{}#{}\"",
                nav.site.base_path, section.index_path, id
            );
            assert!(html.contains(&href), "missing {href}");
            assert!(ids.insert(id.clone()), "duplicate anchor id {id}");
            assert!(!RESERVED_LAYOUT_IDS.contains(&id.as_str()), "reserved {id}");
        }
    }
}

fn read_page(rel: &str) -> String {
    let p = shared_site::real_site().out_dir.join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// ページ内の `id="..."` を全件集める。
fn ids_in(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find(" id=\"") {
        rest = &rest[i + 5..];
        let end = rest.find('"').unwrap();
        out.push(rest[..end].to_string());
    }
    out
}

#[test]
fn built_site_popup_counts_and_anchors_resolve() {
    let nav = load_nav();
    let site = shared_site::real_site();
    assert!(site.report.written.len() > 100);
    let header = read_page("themes/button/index.html");
    let base = &nav.site.base_path;
    assert!(header.contains(&format!("href=\"{base}/themes/#forms\"")));

    for (title, n) in [("Themes", 7), ("Blocks", 66), ("Wireframes", 8)] {
        let idx = nav.sections.iter().position(|s| s.title == title).unwrap();
        assert_eq!(popup_items(&header, idx).len(), n, "{title}");
    }

    // 各グループ付きセクションの索引で id が一意で、popup の全アンカーが実在する。
    for section in nav.sections.iter().filter(|s| !s.groups.is_empty()) {
        let rel = format!("{}index.html", section.index_path.trim_start_matches('/'));
        let html = read_page(&rel);
        let ids = ids_in(&html);
        let uniq: BTreeSet<_> = ids.iter().collect();
        assert_eq!(uniq.len(), ids.len(), "duplicate ids in {rel}");
        for g in &section.groups {
            let id = group_anchor_id(&g.title);
            assert!(ids.contains(&id), "{rel}: missing id {id}");
        }
    }
}
