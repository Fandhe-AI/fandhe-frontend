//! 全ページ共通のサイトフッター（イシュー #3609 / #3703）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` がページループの前に [`site_footer`] を 1 回だけ呼び、
//! 各ページへ clone して `crate::layout::docs_page_with_assets` の `footer` 引数で渡す。
//! `docs_page_with_assets` は `<body>` の最後の子（`div.docs-container` の直後）へ置く。
//! リダイレクト案内ページ（`crate::redirect`）は別経路の文書なので対象外になる。
//!
//! #3703 でヘッダー（`Nav::header_entries`、Assets メガメニュー）に合わせ、
//! ブランド列 + Docs / メニュー（Assets）/ Resources の列構成へ再編した。
//! リンクは各索引ページだけで、セクション直下ページの一覧は持たない。
//!
//! # DOM
//!
//! ```text
//! footer.docs-footer                          … 暗黙 contentinfo
//!   div.docs-footer-inner                     … .docs-container と同じ計測枠
//!     div.docs-footer-top                     … ブランド列 + nav のグリッド
//!       div.docs-footer-brand                 … nav の外（リンク集ではないため）
//!         p.docs-footer-brand-name            … [site].title（リンクにしない）
//!         p.docs-footer-tagline               … [site].tagline（未指定は FOOTER_TAGLINE）
//!       nav.docs-footer-nav[aria-label=Footer]
//!         div.docs-footer-columns
//!           div.docs-footer-group             … Docs（メニューに属さないセクション索引）
//!           div.docs-footer-group × メニュー数  … menu.title（Overview + メンバー索引）
//!           div.docs-footer-group             … Resources（GitHub / crates.io。version_badge 指定時は GitHub のみ）
//!     separator(hr)
//!     div.docs-footer-bottom                  … 著作権（[site].copyright）+ ライセンス行または帰属表記
//! ```
//!
//! # 列の生成規則
//!
//! `Nav::header_entries` を 1 回走査し、`Section` は Docs 列、`Menu` はメニューごとの列へ
//! 振り分ける。Docs 列は項目が 0 件なら出さない。メニュー列の先頭は集約ページ
//! （`menu.index_path`、文言は [`FOOTER_MENU_OVERVIEW_LABEL`]）で、続いて
//! `Nav::menu_members` の宣言順。ブランド名を `/` へのリンクにしないのは、
//! Getting Started の索引と href が重複し「各索引がちょうど 1 回」の契約と衝突するため。
//! ブランド列は pre-styled-ui の recipe が呼び出し側の `class` を捨てるため素の `p` で組む。
//!
//! # なぜ body 直下か
//!
//! `<footer>` は `main` / `article` / `aside` / `nav` / `section` の子孫だと暗黙ロール
//! `contentinfo` を失う。また本文 `Node` へ足すと TOC・検索インデックスに混ざる。
//! sticky のサイドバー・右目次は `.docs-container` の内側でのみ固定されるため、
//! 外側の兄弟であるフッターと重ならない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 文言は `text()` を通す。`raw_html()` と HTML 文字列の組み立ては使わない。
//! 内部 href は `parse_nav` 検証済みのパスを `layout::asset_href` で前置したものだけ、
//! 外部 URL は文字列リテラル定数だけで、すべて `external: true`
//! （`target="_blank"` + `rel="noopener noreferrer"`）にする。`id`・`role`・`aria-current`
//! は出さない（`layout::RESERVED_LAYOUT_IDS` と衝突させず、リンク集に操作意味論を与えない）。
//! ロゴ SVG は持ち込まない。

use fandhe_frontend_core::{div, el, footer, li, nav as nav_el, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{text as ps_text, TextProps, TextSize, TextVariant};

use crate::layout::{asset_href, REPOSITORY_URL};
use crate::nav::{HeaderEntry, Nav};

/// フッター外側の class（`<footer>`）。
pub const FOOTER_CLASS: &str = "docs-footer";
/// 計測枠の class。
pub const FOOTER_INNER_CLASS: &str = "docs-footer-inner";
/// ブランド列と nav を並べる上段グリッドの class。
pub const FOOTER_TOP_CLASS: &str = "docs-footer-top";
/// ブランド列の class。
pub const FOOTER_BRAND_CLASS: &str = "docs-footer-brand";
/// ブランド名 `p` の class。
pub const FOOTER_BRAND_NAME_CLASS: &str = "docs-footer-brand-name";
/// タグライン `p` の class。
pub const FOOTER_TAGLINE_CLASS: &str = "docs-footer-tagline";
/// リンク列 nav の class。
pub const FOOTER_NAV_CLASS: &str = "docs-footer-nav";
/// 列グリッドの class。
pub const FOOTER_COLUMNS_CLASS: &str = "docs-footer-columns";
/// リンク 1 列の class。
pub const FOOTER_GROUP_CLASS: &str = "docs-footer-group";
/// リンク一覧 `ul` の class。
pub const FOOTER_LIST_CLASS: &str = "docs-footer-list";
/// 下段（著作権・ライセンス）の class。
pub const FOOTER_BOTTOM_CLASS: &str = "docs-footer-bottom";

/// フッター nav のランドマーク名（他の nav 名と重ならない英語）。
pub const FOOTER_ARIA_LABEL: &str = "Footer";

/// ブランド列のタグラインの既定値（`[site].tagline` 未指定時、#3721。設計は
/// `docs/design/docs-site-external-use.md`）。
pub const FOOTER_TAGLINE: &str =
    "AI 時代のセキュリティリスクを抑える Rust 製フロントエンドフレームワーク";
/// Docs 列の見出し。
pub const FOOTER_DOCS_HEADING: &str = "Docs";
/// Resources 列の見出し。
pub const FOOTER_RESOURCES_HEADING: &str = "Resources";
/// メニュー列先頭（集約ページ）のリンク文言。見出し `menu.title` との重複を避ける。
pub const FOOTER_MENU_OVERVIEW_LABEL: &str = "Overview";

/// crates.io の主要クレートページ。
const CRATES_IO_URL: &str = "https://crates.io/crates/fandhe-frontend-core";
/// MIT ライセンス本文（リポジトリ直下）。
const LICENSE_MIT_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-MIT";
/// Apache-2.0 ライセンス本文（リポジトリ直下）。
const LICENSE_APACHE_URL: &str =
    "https://github.com/Fandhe-AI/fandhe-frontend/blob/main/LICENSE-APACHE";

/// 帰属表記が指す docs-site 本体のリポジトリ URL。`layout::REPOSITORY_URL`（将来
/// `[site].repository_url` で差し替わる値）とは意図的に共有せず、設定から独立した定数にする
/// （帰属表記を設定で消せない・書き換えられないための不変条件、設計文書 §6）。
const ATTRIBUTION_REPOSITORY_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 著作権表記の既定値（`[site].copyright` 未指定時。`LICENSE-MIT` の著作権行と同じ文言）。
const COPYRIGHT_TEXT: &str = "© 2026 Fandhe-AI / fandhe-frontend contributors";

fn external_link(href: &str, label: &str) -> Node {
    let props = LinkProps {
        external: true,
        ..LinkProps::default()
    };
    link::root(href, &props, vec![], vec![text(label)])
}

fn external_item(href: &str, label: &str) -> Node {
    li(vec![], vec![external_link(href, label)])
}

/// 内部索引ページへのリンク行。href は検証済みパスに `base_path` を前置したもの。
fn internal_item(base: &str, path: &str, label: &str) -> Node {
    let href = asset_href(base, path);
    li(
        vec![],
        vec![link::root(
            &href,
            &LinkProps::default(),
            vec![],
            vec![text(label)],
        )],
    )
}

/// 見出し + リンク一覧の 1 列。
fn column(heading_text: &str, items: Vec<Node>) -> Node {
    let heading_props = HeadingProps {
        size: HeadingSize::Sm,
        ..HeadingProps::default()
    };
    div(
        vec![("class", FOOTER_GROUP_CLASS)],
        vec![
            heading(
                HeadingLevel::H2,
                &heading_props,
                vec![],
                vec![text(heading_text)],
            ),
            ul(vec![("class", FOOTER_LIST_CLASS)], items),
        ],
    )
}

/// `nav` から全ページ共通のフッターを組み立てる。
///
/// 現在ページに依存しないため `aria-current` は付けない（サイドバー・ヘッダーの
/// 現在地表示と意味を重複させない）。
pub fn site_footer(nav: &Nav) -> Node {
    let base = nav.site.base_path.as_str();

    let mut docs_items: Vec<Node> = Vec::new();
    let mut menu_columns: Vec<Node> = Vec::new();
    for entry in nav.header_entries() {
        match entry {
            HeaderEntry::Section(section) => {
                docs_items.push(internal_item(base, &section.index_path, &section.title));
            }
            HeaderEntry::Menu(menu) => {
                let mut items = vec![internal_item(
                    base,
                    &menu.index_path,
                    FOOTER_MENU_OVERVIEW_LABEL,
                )];
                items.extend(
                    nav.menu_members(menu).map(|(section, _)| {
                        internal_item(base, &section.index_path, &section.title)
                    }),
                );
                menu_columns.push(column(&menu.title, items));
            }
        }
    }

    let mut groups: Vec<Node> = Vec::new();
    if !docs_items.is_empty() {
        groups.push(column(FOOTER_DOCS_HEADING, docs_items));
    }
    groups.extend(menu_columns);
    // crates.io リンクは fandhe-frontend-core のページなので、`version_badge` を指定した
    // 外部サイトでは出さない（badge と同じ「本家固有の表示」として連動させる）。
    let mut resources = vec![external_item(REPOSITORY_URL, "GitHub")];
    if nav.site.version_badge.is_none() {
        resources.push(external_item(CRATES_IO_URL, "crates.io"));
    }
    groups.push(column(FOOTER_RESOURCES_HEADING, resources));

    let brand = div(
        vec![("class", FOOTER_BRAND_CLASS)],
        vec![
            el(
                "p",
                vec![("class", FOOTER_BRAND_NAME_CLASS)],
                vec![text(nav.site.title.as_str())],
            ),
            el(
                "p",
                vec![("class", FOOTER_TAGLINE_CLASS)],
                vec![text(nav.site.tagline.as_deref().unwrap_or(FOOTER_TAGLINE))],
            ),
        ],
    );

    let small_muted = TextProps {
        size: TextSize::Sm,
        variant: TextVariant::Muted,
        ..TextProps::default()
    };
    // ブランド系キーの指定有無で下段 2 行目だけを切り替える。未指定側のノード列は従来
    // 出力とバイト一致させるため触らない。指定側でも MIT / Apache-2.0 へのリンクは
    // 残し、docs-site 本体への帰属を示す（消す・差し替えるキーは作らない）。
    let license_line = if nav.site.is_brand_customized() {
        vec![
            text("Built with "),
            external_link(ATTRIBUTION_REPOSITORY_URL, "fandhe-frontend docs-site"),
            text(" ("),
            external_link(LICENSE_MIT_URL, "MIT"),
            text(" OR "),
            external_link(LICENSE_APACHE_URL, "Apache-2.0"),
            text(")"),
        ]
    } else {
        vec![
            text("Licensed under "),
            external_link(LICENSE_MIT_URL, "MIT"),
            text(" OR "),
            external_link(LICENSE_APACHE_URL, "Apache-2.0"),
        ]
    };
    let copyright = nav.site.copyright.as_deref().unwrap_or(COPYRIGHT_TEXT);
    let bottom = div(
        vec![("class", FOOTER_BOTTOM_CLASS)],
        vec![
            ps_text(&small_muted, vec![], vec![text(copyright)]),
            ps_text(&small_muted, vec![], license_line),
        ],
    );

    footer(
        vec![("class", FOOTER_CLASS)],
        vec![div(
            vec![("class", FOOTER_INNER_CLASS)],
            vec![
                div(
                    vec![("class", FOOTER_TOP_CLASS)],
                    vec![
                        brand,
                        nav_el(
                            vec![
                                ("class", FOOTER_NAV_CLASS),
                                ("aria-label", FOOTER_ARIA_LABEL),
                            ],
                            vec![div(vec![("class", FOOTER_COLUMNS_CLASS)], groups)],
                        ),
                    ],
                ),
                separator(&SeparatorProps::default(), vec![]),
                bottom,
            ],
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::parse_nav;
    use fandhe_frontend_core::render;

    fn nav_with(base: &str, with_menu: bool) -> Nav {
        let mut s = format!("[site]\ntitle = \"Site <i>\"\nbase_path = \"{base}\"\n\n");
        for (title, idx) in [
            ("A <script>", "/a/"),
            ("M1", "/m1/"),
            ("B", "/b/"),
            ("M2", "/m2/"),
        ] {
            s.push_str(&format!(
                "[[section]]\ntitle = \"{title}\"\nindex_path = \"{idx}\"\n\n[[section.page]]\ntitle = \"Idx\"\nsource = \"i.md\"\npath = \"{idx}\"\n\n[[section.page]]\ntitle = \"Child\"\nsource = \"c.md\"\npath = \"{idx}child/\"\n\n"
            ));
        }
        if with_menu {
            s.push_str(
                "[[menu]]\ntitle = \"Menu <b>\"\nindex_path = \"/m/\"\nsource = \"m.md\"\n\n[[menu.item]]\nsection = \"/m2/\"\ndescription = \"two\"\n\n[[menu.item]]\nsection = \"/m1/\"\ndescription = \"one\"\n",
            );
        }
        parse_nav(&s).expect("fixture nav parses")
    }

    fn count(html: &str, pat: &str) -> usize {
        html.matches(pat).count()
    }

    #[test]
    fn columns_are_docs_then_menus_then_resources() {
        let html = render(&site_footer(&nav_with("", true)));
        assert_eq!(count(&html, "class=\"docs-footer-group\""), 3);
        let d = html.find(">Docs<").unwrap();
        let m = html.find(">Menu &lt;b&gt;<").unwrap();
        let r = html.find(">Resources<").unwrap();
        assert!(d < m && m < r);
    }

    #[test]
    fn docs_column_lists_only_non_menu_section_indexes() {
        let html = render(&site_footer(&nav_with("", true)));
        let start = html.find(">Docs<").unwrap();
        let end = html.find(">Menu &lt;b&gt;<").unwrap();
        let docs = &html[start..end];
        assert_eq!(count(docs, "<a "), 2);
        assert!(docs.contains("href=\"/a/\"") && docs.contains("href=\"/b/\""));
        assert!(!docs.contains("child") && !docs.contains("/m1/"));
    }

    #[test]
    fn menu_column_starts_with_menu_index_then_members_in_item_order() {
        let html = render(&site_footer(&nav_with("", true)));
        let start = html.find(">Menu &lt;b&gt;<").unwrap();
        let end = html.find(">Resources<").unwrap();
        let col = &html[start..end];
        let a = col.find("href=\"/m/\"").unwrap();
        let b = col.find("href=\"/m2/\"").unwrap();
        let c = col.find("href=\"/m1/\"").unwrap();
        assert!(a < b && b < c);
        assert!(col.contains(">Overview<"));
    }

    #[test]
    fn each_internal_index_appears_exactly_once() {
        let html = render(&site_footer(&nav_with("", true)));
        for p in ["/a/", "/b/", "/m1/", "/m2/", "/m/"] {
            assert_eq!(count(&html, &format!("href=\"{p}\"")), 1, "{p}");
        }
        assert_eq!(count(&html, "child/"), 0);
    }

    #[test]
    fn without_menu_only_docs_and_resources_columns() {
        let html = render(&site_footer(&nav_with("", false)));
        assert_eq!(count(&html, "class=\"docs-footer-group\""), 2);
    }

    #[test]
    fn brand_column_is_outside_nav_and_not_a_link() {
        let html = render(&site_footer(&nav_with("", true)));
        let brand = html.find("docs-footer-brand\"").unwrap();
        let nav_pos = html.find("<nav").unwrap();
        assert!(brand < nav_pos);
        let region = &html[brand..nav_pos];
        assert!(region.contains("Site &lt;i&gt;"));
        assert!(!region.contains("<a "));
    }

    #[test]
    fn external_links_are_all_hardened() {
        let html = render(&site_footer(&nav_with("", true)));
        assert_eq!(count(&html, "target=\"_blank\""), 4);
        assert_eq!(count(&html, "rel=\"noopener noreferrer\""), 4);
        let bottom = &html[html.find("docs-footer-bottom").unwrap()..];
        assert!(!bottom.contains("crates.io/crates") && !bottom.contains(">GitHub<"));
        let res_start = html.find(">Resources<").unwrap();
        let res = &html[res_start..html.find("<hr").unwrap_or(html.len())];
        assert!(!res.contains("LICENSE"));
    }

    #[test]
    fn output_has_no_ids_scripts_or_dangerous_attrs() {
        let html = render(&site_footer(&nav_with("", true)));
        for forbidden in [
            "id=\"",
            "<form",
            "<script",
            "javascript:",
            " on",
            "href=\"#\"",
            "aria-current",
            "role=\"contentinfo\"",
            "role=\"menu",
            "aria-expanded",
        ] {
            assert!(
                !html.contains(forbidden),
                "{forbidden} が出力された: {html}"
            );
        }
    }

    #[test]
    fn hostile_titles_are_escaped() {
        let html = render(&site_footer(&nav_with("", true)));
        assert!(!html.contains("<script>") && !html.contains("<b>") && !html.contains("<i>"));
        assert!(html.contains("A &lt;script&gt;"));
    }

    /// `[site]` へ任意キーを足した nav（#3721）。
    fn nav_with_site_keys(extra: &str) -> Nav {
        parse_nav(&format!(
            "[site]\ntitle = \"S\"\nbase_path = \"\"\n{extra}\n[[section]]\ntitle = \"A\"\nindex_path = \"/a/\"\n\n[[section.page]]\ntitle = \"Idx\"\nsource = \"i.md\"\npath = \"/a/\"\n"
        ))
        .expect("fixture nav parses")
    }

    fn footer_html(extra: &str) -> String {
        render(&site_footer(&nav_with_site_keys(extra)))
    }

    #[test]
    fn defaults_show_license_line_and_crates_io() {
        let html = footer_html("");
        assert!(html.contains(FOOTER_TAGLINE));
        assert!(html.contains(COPYRIGHT_TEXT));
        assert!(html.contains("Licensed under"));
        assert!(!html.contains("Built with"));
        assert!(html.contains("crates.io/crates"));
        assert_eq!(count(&html, "target=\"_blank\""), 4);
    }

    #[test]
    fn tagline_override_replaces_default_and_is_escaped() {
        let html = footer_html("tagline = \"Hi <script>x</script>\"\n");
        assert!(html.contains("Hi &lt;script&gt;x&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains(FOOTER_TAGLINE));
    }

    #[test]
    fn copyright_override_replaces_default_and_is_escaped() {
        let html = footer_html("copyright = \"(c) <i>Me</i>\"\n");
        assert!(html.contains("(c) &lt;i&gt;Me&lt;/i&gt;"));
        assert!(!html.contains("<i>Me"));
        assert!(!html.contains(COPYRIGHT_TEXT));
    }

    #[test]
    fn version_badge_key_removes_crates_io_link_in_both_forms() {
        for extra in ["version_badge = \"v1\"\n", "version_badge = \"\"\n"] {
            let html = footer_html(extra);
            assert!(!html.contains("crates.io/crates"), "{extra}");
            assert!(html.contains(">GitHub<"), "{extra}");
        }
    }

    #[test]
    fn customized_footer_switches_to_attribution_and_keeps_license_links() {
        for extra in [
            "tagline = \"t\"\n",
            "copyright = \"c\"\n",
            "version_badge = \"\"\n",
        ] {
            let html = footer_html(extra);
            assert!(html.contains("Built with "), "{extra}");
            assert!(!html.contains("Licensed under"), "{extra}");
            // GitHub リンク（REPOSITORY_URL）と帰属リンクは同じ URL を指す。
            assert_eq!(
                count(&html, &format!("href=\"{ATTRIBUTION_REPOSITORY_URL}\"")),
                2,
                "{extra}"
            );
            assert!(html.contains(">fandhe-frontend docs-site<"), "{extra}");
            assert_eq!(count(&html, "LICENSE-MIT\""), 1, "{extra}");
            assert_eq!(count(&html, "LICENSE-APACHE\""), 1, "{extra}");
            let blank = count(&html, "target=\"_blank\"");
            assert_eq!(
                blank,
                count(&html, "rel=\"noopener noreferrer\""),
                "{extra}"
            );
            // GitHub + 帰属 + MIT + Apache-2.0 に、crates.io が残る構成だけ +1。
            let expected = if extra.starts_with("version_badge") {
                4
            } else {
                5
            };
            assert_eq!(blank, expected, "{extra}");
        }
    }

    #[test]
    fn lang_only_footer_is_identical_to_default() {
        assert_eq!(footer_html("lang = \"en\"\n"), footer_html(""));
    }

    #[test]
    fn customized_footer_still_has_no_ids_or_scripts() {
        let html = footer_html("tagline = \"t\"\ncopyright = \"c\"\nversion_badge = \"v\"\n");
        for forbidden in ["id=\"", "<script", "javascript:", "aria-current"] {
            assert!(!html.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn base_path_is_prefixed_to_internal_hrefs() {
        let html = render(&site_footer(&nav_with("/fandhe-frontend", true)));
        assert!(html.contains("href=\"/fandhe-frontend/a/\""));
        assert!(html.contains("href=\"/fandhe-frontend/m/\""));
    }
}
