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
use fandhe_frontend_docs_site::nav::{
    group_anchor_id, header_nav, nav_drawer, parse_nav, sidebar, Nav,
};

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

/// ナビ drawer の `section_idx` 番目のセクション本体（`div.docs-nav-drawer-body`）内の
/// `a` のテキスト。`class="docs-nav-drawer-section"`（閉じ引用符つき）で `li` を分割する
/// ため `-link` 付きの class とは混ざらない。
fn drawer_body_items(drawer_html: &str, section_idx: usize) -> Vec<String> {
    let seg = drawer_html
        .split("class=\"docs-nav-drawer-section\"")
        .nth(section_idx + 1)
        .expect("drawer section should exist");
    let body = seg
        .find("docs-nav-drawer-body")
        .expect("drawer body should exist");
    anchor_texts(&seg[body..])
}

/// drawer は popup（他セクション）とサイドバー（現在セクション）と同じ見出し一覧を出す
/// （イシュー #3674。どちらも `Section::headings` が唯一の情報源）。
#[test]
fn drawer_matches_popup_for_other_sections_and_sidebar_for_current_section() {
    let nav = load_nav();
    for (cur_idx, current_section) in nav.sections.iter().enumerate() {
        let mut currents = vec![current_section.all_pages().next().unwrap().path.clone()];
        if let Some(g) = current_section.groups.first() {
            currents.push(g.pages[0].path.clone());
        }
        for current in currents {
            let drawer = render(&nav_drawer(&nav, &current));
            let header = render(&header_nav(&nav, &current));
            for idx in 0..nav.sections.len() {
                let items = drawer_body_items(&drawer, idx);
                if idx == cur_idx {
                    let side = anchor_texts(&render(&sidebar(&nav, &current)));
                    assert_eq!(
                        items, side,
                        "current section {:?} at {current}",
                        nav.sections[idx].title
                    );
                } else {
                    let popup = popup_items(&header, idx);
                    assert_eq!(
                        items, popup,
                        "section {:?} at {current}",
                        nav.sections[idx].title
                    );
                }
            }
        }
    }
}

/// 受け入れ条件: Themes の Button ページで、drawer から全 8 セクションの索引と
/// 現在セクションの個別ページ（Button）へ移れる。
#[test]
fn built_site_drawer_reaches_every_section_and_current_section_pages() {
    let nav = load_nav();
    let html = read_page("themes/button/index.html");
    let base = &nav.site.base_path;
    let start = html.find("class=\"docs-nav-drawer\"").expect("drawer");
    let end = start + html[start..].find("</nav>").unwrap();
    let drawer = &html[start..end];
    for section in &nav.sections {
        let href = format!("href=\"{base}{}\"", section.index_path);
        assert!(drawer.contains(&href), "drawer lacks section link {href}");
    }
    assert!(drawer.contains(&format!("href=\"{base}/themes/button/\"")));
}

/// 実サイトの全 HTML（404 を含む）で、レイアウト固定 `id`（`RESERVED_LAYOUT_IDS`）が
/// 1 ページに高々 1 回しか現れず、ナビ drawer を含むヘッダー内の `id` が一意
/// （イシュー #3674 受け入れ条件）。本文内の `id` 重複（例: `primitives/date-picker/` の
/// デモ。drawer と無関係な既存事象）はこのテストの対象外。
#[test]
fn built_site_layout_ids_and_header_ids_are_unique_on_every_page() {
    let out = shared_site::real_site().out_dir.as_path();
    let mut stack = vec![out.to_path_buf()];
    let mut checked = 0usize;
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                let html = std::fs::read_to_string(&path).unwrap();
                let ids = ids_in(&html);
                for reserved in RESERVED_LAYOUT_IDS {
                    let n = ids.iter().filter(|i| i.as_str() == *reserved).count();
                    assert!(n <= 1, "{path:?}: reserved id {reserved} appears {n} times");
                }
                // リダイレクト案内ページはクロームを持たない（`no_js_contract.rs` 参照）。
                let Some(start) = html.find("<header") else {
                    continue;
                };
                let end = start + html[start..].find("</header>").expect("header end");
                let header_ids = ids_in(&html[start..end]);
                let uniq: BTreeSet<_> = header_ids.iter().collect();
                assert_eq!(
                    uniq.len(),
                    header_ids.len(),
                    "duplicate header ids in {path:?}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 100);
    assert!(out.join("404.html").exists());
}
