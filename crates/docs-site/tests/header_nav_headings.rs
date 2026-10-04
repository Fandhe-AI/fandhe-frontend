//! ヘッダーの Assets メガパネル（`header_nav`、イシュー #3701）が `site/nav.toml` の
//! `[[menu]]` と一致すること、ナビ drawer の項目がヘッダーと同じ構成（項目・順序・
//! 表記・説明・現在位置）であること、グループアンカーの実在・一意性を実サイトで
//! 固定する（イシュー #3670 / #3674 / #3701 / #3702）。
//!
//! drawer は見出しを展開する `details` を持たず、ヘッダーと同じ `Nav::header_entries` から
//! 作る。ここでは描画結果の HTML から項目列を取り出して比較し、構成の乖離を落とす。
//! アンカーの実在はビルド時のリンク検証（`linkcheck`）も fail-closed に守る。

use std::collections::BTreeSet;

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::layout::RESERVED_LAYOUT_IDS;
use fandhe_frontend_docs_site::nav::{group_anchor_id, header_nav, nav_drawer, parse_nav, Nav};

#[path = "support/shared_site.rs"]
mod shared_site;

fn load_nav() -> Nav {
    let path = shared_site::repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(path).expect("site/nav.toml should be readable");
    parse_nav(&input).expect("site/nav.toml should parse")
}

/// `marker` の直後から次の `<` までのテキストを取り出す。
fn text_after(s: &str, marker: &str) -> String {
    let i = s.find(marker).expect("marker should exist") + marker.len();
    s[i..i + s[i..].find('<').unwrap()].to_string()
}

/// ヘッダーのメガパネル内のカード（href・タイトル・説明）を出現順に集める。
fn mega_cards(header_html: &str) -> Vec<(String, String, String)> {
    let start = header_html
        .find("class=\"docs-header-mega\"")
        .expect("mega panel should exist");
    let panel = &header_html[start..];
    let panel = &panel[..panel.find("</ul>").expect("mega grid should close")];
    let mut cards = Vec::new();
    let mut rest = panel;
    // カードの開始タグは `<a href="..." class="docs-header-mega-card"...>`。
    while let Some(i) = rest.find("<a href=\"") {
        rest = &rest[i + 9..];
        let href = rest[..rest.find('"').unwrap()].to_string();
        let end = rest.find("</a>").unwrap();
        let card = &rest[..end];
        assert!(card.contains("docs-header-mega-card"));
        cards.push((
            href,
            text_after(card, "docs-header-mega-title\">"),
            text_after(card, "docs-header-mega-desc\">"),
        ));
        rest = &rest[end..];
    }
    cards
}

/// ヘッダーのトリガー（`a.docs-header-trigger`）のテキストを出現順に集める。
fn trigger_titles(header_html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = header_html;
    while let Some(i) = rest.find("class=\"docs-header-trigger\"") {
        rest = &rest[i..];
        let gt = rest.find('>').unwrap();
        let end = rest.find("</a>").unwrap();
        out.push(rest[gt + 1..end].to_string());
        rest = &rest[end..];
    }
    out
}

/// ヘッダー・drawer 共通の項目: `(href, タイトル, 説明, 所属表示 aria-current="true" の有無)`。
/// ヘッダーのトリガー・メガカード、drawer のセクションリンク・メンバーリンクを
/// 同じ形へ正規化する（説明を持たない項目は空文字列）。
fn nav_items(html: &str) -> Vec<(String, String, String, bool)> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find("<a href=\"") {
        rest = &rest[i + 9..];
        let href = rest[..rest.find('"').unwrap()].to_string();
        let tag_end = rest.find('>').unwrap();
        let tag = &rest[..tag_end];
        let end = rest.find("</a>").unwrap();
        let body = &rest[tag_end + 1..end];
        let current = tag.contains("aria-current=\"true\"");
        if tag.contains("docs-header-mega-card") {
            out.push((
                href,
                text_after(body, "docs-header-mega-title\">"),
                text_after(body, "docs-header-mega-desc\">"),
                current,
            ));
        } else if tag.contains("docs-nav-drawer-menu-link") {
            out.push((
                href,
                text_after(body, "docs-nav-drawer-menu-title\">"),
                text_after(body, "docs-nav-drawer-menu-desc\">"),
                current,
            ));
        } else {
            out.push((href, body.to_string(), String::new(), current));
        }
        rest = &rest[end..];
    }
    out
}

/// 実際の `site/nav.toml` で、ヘッダーのトリガーが 4 項目、Assets のカードが
/// `[[menu.item]]` の宣言順（Primitives → Themes → Blocks → Wireframes → Examples）に
/// タイトル・href・説明まで一致する（イシュー #3701）。
#[test]
fn mega_panel_cards_match_nav_toml_menu() {
    let nav = load_nav();
    assert_eq!(nav.menus.len(), 1);
    let menu = &nav.menus[0];
    let header = render(&header_nav(&nav, "/guides/"));
    assert_eq!(
        trigger_titles(&header),
        ["Getting Started", "Guides", "Assets", "API Reference"]
    );
    let expected: Vec<(String, String, String)> = nav
        .menu_members(menu)
        .map(|(s, i)| {
            (
                format!("{}{}", nav.site.base_path, s.index_path),
                s.title.clone(),
                i.description.clone(),
            )
        })
        .collect();
    assert_eq!(mega_cards(&header), expected);
    let titles: Vec<&str> = expected.iter().map(|c| c.1.as_str()).collect();
    assert_eq!(
        titles,
        ["Primitives", "Themes", "Blocks", "Wireframes", "Examples"]
    );
    // 見出し一覧 popup は出ない。
    assert!(!header.contains("docs-header-dropdown"));
}

/// グループ見出しの索引アンカー（`#<slug>`）は索引ページ側だけが持ち、ヘッダーにも
/// drawer にも出ない（#3701 / #3702）。アンカー id は一意で予約 id でない。
#[test]
fn group_headings_are_not_linked_from_header_or_drawer_and_ids_are_unique() {
    let nav = load_nav();
    for section in &nav.sections {
        let drawer = render(&nav_drawer(&nav, &section.index_path));
        let header = render(&header_nav(&nav, &section.index_path));
        let mut ids = BTreeSet::new();
        for g in &section.groups {
            let id = group_anchor_id(&g.title);
            let frag = format!("#{id}\"");
            assert!(!drawer.contains(&frag), "drawer must not link {frag}");
            assert!(!header.contains(&frag), "header must not link {frag}");
            assert!(ids.insert(id.clone()), "duplicate anchor id {id}");
            assert!(!RESERVED_LAYOUT_IDS.contains(&id.as_str()), "reserved {id}");
        }
    }
}

fn read_page(rel: &str) -> String {
    let p = shared_site::real_site().out_dir.join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// ページ内のヘッダーナビ（`nav.docs-header-nav`）だけを返す。`<header>` 全体には
/// ナビ drawer も含まれ、drawer 側の `aria-current` と混ざるため分ける。
fn header_of(html: &str) -> String {
    let start = html
        .find("class=\"docs-header-nav\"")
        .expect("header nav should exist");
    let end = start + html[start..].find("</nav>").expect("header nav end");
    html[start..end].to_string()
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
fn built_site_header_has_mega_panel_and_index_anchors_resolve() {
    let nav = load_nav();
    let site = shared_site::real_site();
    assert!(site.report.written.len() > 100);
    let base = &nav.site.base_path;
    let trigger_current =
        format!("href=\"{base}/assets/\" class=\"docs-header-trigger\" aria-current=\"true\"");

    // Themes/Button: Assets トリガーと Themes カードだけが所属表示。見出しアンカーは出ない。
    let header = header_of(&read_page("themes/button/index.html"));
    assert_eq!(mega_cards(&header).len(), 5);
    assert!(header.contains(&trigger_current));
    assert!(header.contains(&format!(
        "href=\"{base}/themes/\" class=\"docs-header-mega-card\" aria-current=\"true\""
    )));
    assert_eq!(header.matches("aria-current=\"true\"").count(), 2);
    assert!(!header.contains("#forms"));

    // /assets/: トリガーだけが強調され、カードは強調されない。
    let assets = header_of(&read_page("assets/index.html"));
    assert!(assets.contains(&trigger_current));
    assert_eq!(assets.matches("aria-current=\"true\"").count(), 1);

    // /guides/: Guides トリガーだけが強調される（Assets は強調されない）。
    let guides = header_of(&read_page("guides/index.html"));
    assert_eq!(guides.matches("aria-current=\"true\"").count(), 1);
    assert!(guides.contains(&format!(
        "href=\"{base}/guides/\" class=\"docs-header-trigger\" aria-current=\"true\""
    )));

    // 各グループ付きセクションの索引で id が一意で、グループアンカーが実在する。
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

/// drawer の項目（href・タイトル・説明・所属表示）が、ヘッダーのトリガー列とメガカード列
/// （Assets トリガーの直後）を合わせたものと、順序まで一致する（イシュー #3702）。
/// 現在位置は、単独セクション・Assets 見出し・メンバー配下・未登録パスの各場合で検証する。
#[test]
fn drawer_items_match_header_entries_and_menu_members() {
    let nav = load_nav();
    assert_eq!(nav.menus.len(), 1);
    let menu = &nav.menus[0];
    let menu_href = format!("{}{}", nav.site.base_path, menu.index_path);
    for path in ["/guides/", "/themes/button/", "/assets/", "/no-such-page/"] {
        let header = render(&header_nav(&nav, path));
        let drawer = render(&nav_drawer(&nav, path));
        let header_items = nav_items(&header);
        let drawer_items = nav_items(&drawer);
        // ヘッダー側はトリガー → メガカードの順で出る。メニューのカードはトリガーの直後に
        // 続くため、そのまま連結した列が drawer の見出し + メンバー列と一致する。
        assert_eq!(drawer_items, header_items, "items differ at {path}");
        // 構成の期待: Getting Started, Guides, Assets, 5 メンバー, API Reference。
        let titles: Vec<&str> = drawer_items.iter().map(|i| i.1.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Getting Started",
                "Guides",
                "Assets",
                "Primitives",
                "Themes",
                "Blocks",
                "Wireframes",
                "Examples",
                "API Reference"
            ],
            "{path}"
        );
        assert!(drawer_items.iter().any(|i| i.0 == menu_href));
        // 所属表示の集合もヘッダーと一致する（上の完全一致に含まれるが、件数を明示する）。
        let current: Vec<&str> = drawer_items
            .iter()
            .filter(|i| i.3)
            .map(|i| i.1.as_str())
            .collect();
        match path {
            "/themes/button/" => assert_eq!(current, ["Assets", "Themes"]),
            "/assets/" => assert_eq!(current, ["Assets"]),
            "/guides/" => assert_eq!(current, ["Guides"]),
            _ => assert!(current.is_empty()),
        }
    }
}

/// 受け入れ条件: Themes の Button ページで、drawer から全 8 セクションの索引と Assets
/// 集約ページへ移れ、見出しを展開する `details` を持たず、現在ページ自体へのリンクも
/// 持たない（セクション内の移動は索引ページ経由の 2 ホップ。索引ページが子ページを全件
/// リンクすることは `section_index_nav.rs` / `category_index_nav.rs` /
/// `component_index_nav.rs` が固定するためここでは重複して検証しない）。
#[test]
fn built_site_drawer_reaches_every_section_index_and_assets() {
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
    // タッチ端末ではヘッダーナビが非表示のため、drawer が Assets 集約ページの入口になる（#3701）。
    assert!(
        drawer.contains(&format!("href=\"{base}/assets/\"")),
        "drawer lacks menu index link"
    );
    assert!(!drawer.contains("<details"), "drawer must not have details");
    assert!(!drawer.contains("<summary"), "drawer must not have summary");
    assert!(
        !drawer.contains(&format!("href=\"{base}/themes/button/\"")),
        "drawer must not list individual pages"
    );
}

/// 実サイトの全 HTML（404 を含む）で、レイアウト固定 `id`（`RESERVED_LAYOUT_IDS`）が
/// 1 ページに高々 1 回しか現れず、ナビ drawer を含むヘッダー内の `id` が一意
/// （イシュー #3674 受け入れ条件）。#3701 の「`id` の増加なし」もここで守る:
/// ヘッダーナビ（`nav.docs-header-nav`）自体は `id` を一切持たない。本文内の `id` 重複
/// （例: `primitives/date-picker/` のデモ。drawer と無関係な既存事象）はこのテストの対象外。
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
                let header = &html[start..end];
                let header_ids = ids_in(header);
                let uniq: BTreeSet<_> = header_ids.iter().collect();
                assert_eq!(
                    uniq.len(),
                    header_ids.len(),
                    "duplicate header ids in {path:?}"
                );
                if let Some(n) = header.find("class=\"docs-header-nav\"") {
                    let nav_end = n + header[n..].find("</nav>").expect("nav end");
                    let nav_html = &header[n..nav_end];
                    for forbidden in [
                        " id=\"",
                        "role=\"",
                        "aria-expanded",
                        "aria-haspopup",
                        "aria-controls",
                    ] {
                        assert!(
                            !nav_html.contains(forbidden),
                            "{path:?}: header nav must not contain {forbidden}"
                        );
                    }
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 100);
    assert!(out.join("404.html").exists());
}
