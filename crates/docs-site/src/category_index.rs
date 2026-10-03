//! Blocks・Wireframes 索引（`/blocks/`・`/wireframes/`）のカテゴリ別カード
//! （イシュー #3618）。
//!
//! # 役割・呼び出し文脈
//!
//! 2 つの索引ページはこれまで「区分 h2 → カテゴリ h3 → リンクの箇条書き」
//! （Blocks、約 330 件）と手書きのリンク集（Wireframes、49 件）で、右目次が
//! 長く、Wireframes はレジストリとの二重管理だった。本モジュールは
//! [`crate::page_sections`]（汎用生成節フック、#3598）へ登録する生成関数
//! （[`render_blocks`] / [`render_wireframes`]）と専用 CSS（[`stylesheet`]）を
//! 提供し、レジストリ（[`crate::blocks::all_blocks`] / [`crate::wireframes::WIREFRAMES`]）
//! からカテゴリ別カードを差し込む。block・部品・カテゴリを追加しても索引原稿の
//! 手編集は不要（#2733 の「レジストリから生成する」方針を維持）。
//!
//! # 構造（Blocks・Wireframes 共通）
//!
//! ```text
//! div.docs-category-section          … 区分 1 件（Wireframes は 1 区分）
//!   div.docs-category-section-head
//!     h2                              … 区分名（TOC・検索に載る）
//!     badge                           … 区分の合計件数（h2 の兄弟）
//!   ul.docs-category-grid
//!     li.docs-category-card
//!       card::root > card::body
//!         div.docs-category-card-head
//!           h3                        … カテゴリ名（リンクではない）
//!           badge                     … カテゴリの件数
//!         ul.docs-category-card-list
//!           li > a                    … block / 部品へのリンク（全件）
//! ```
//!
//! 件数 badge は Themes/Primitives 索引・サイドバーのグループ件数と同じ props
//! （Subtle / Neutral / Sm、「N 件」）。カードは複数リンクを持つため、
//! `component_index` の全面リンク（`::after` 伸張）は使わず、リストを折りたたまず
//! 全件を載せる（無 JS でも全リンクが初期表示で見え、キーボードで到達できる）。
//! pre-styled-ui の root は呼び出し側の `class` を破棄するため、`docs-category-*`
//! class は常にラッパー要素へ付ける。
//!
//! # 検索インデックスとの関係
//!
//! li の class を意図的に `docs-index-card` にしない。card（`data-scope`）配下の
//! カテゴリ名・block 名は TOC と検索テキストから外れるが、各 block・部品ページが
//! 個別に索引化済みで重複ヒットを避けられる。区分 h2 は `data-scope` の外に置くため
//! TOC と検索に残る。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API のみで組み、題名・カテゴリ名・件数は既定エスケープを通る
//! （`raw_html()` 不使用、REQ-1）。href は [`crate::layout::asset_href`] による
//! サイト内の絶対パスだけで、入力はコンパイル時定数のレジストリ（外部入力を
//! 含まない）。JS・`on*=` 属性は出さない。`id` はカテゴリカードの `li` にだけ
//! `nav::group_anchor_id(group.title)` を付ける（ヘッダー popup のリンク先、#3670）。

use fandhe_frontend_core::{a, div, h2, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size, StyleSheet, StylesheetError};

use crate::blocks::{self, BlockCategory, BlockSection};
use crate::layout::asset_href;
use crate::wireframes::{WireframeCategory, WIREFRAMES};

/// 生成節が配線する追加 CSS の出力先（[`crate::page_sections::PAGE_STYLESHEETS`]
/// の `rel_path`）。
pub const STYLESHEET_REL_PATH: &str = "assets/category-index.css";

/// リンク 1 件（カード内の 1 行）。
#[derive(Debug, Clone, Copy)]
struct Item<'a> {
    path: &'a str,
    title: &'a str,
}

/// カテゴリ 1 件（カード 1 枚）。
#[derive(Debug, Clone)]
struct Group<'a> {
    title: &'a str,
    items: Vec<Item<'a>>,
}

/// 区分 1 件（h2 + カードグリッド）。
#[derive(Debug, Clone)]
struct Section<'a> {
    title: &'a str,
    groups: Vec<Group<'a>>,
}

/// 件数 badge（Subtle / Neutral / Sm、「N 件」）。
fn count_badge(n: usize) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Sm,
            palette: ColorPalette::Neutral,
            shape: None,
        },
        vec![],
        vec![text(format!("{n} 件"))],
    )
}

/// カード 1 枚（`li.docs-category-card`）を組む。件数は `items.len()` から算出する。
fn card_node(base_path: &str, group: &Group<'_>) -> Node {
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(group.title)],
    );
    let anchor_id = crate::nav::group_anchor_id(group.title);
    let links: Vec<Node> = group
        .items
        .iter()
        .map(|item| {
            let href = asset_href(base_path, item.path);
            li(
                vec![],
                vec![a(vec![("href", href.as_str())], vec![text(item.title)])],
            )
        })
        .collect();
    li(
        vec![("class", "docs-category-card"), ("id", anchor_id.as_str())],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "docs-category-card-head")],
                        vec![title, count_badge(group.items.len())],
                    ),
                    ul(vec![("class", "docs-category-card-list")], links),
                ],
            )],
        )],
    )
}

/// 区分 1 件を組む。合計件数は各カテゴリの件数の和から算出する。
fn section_node(base_path: &str, section: &Section<'_>) -> Node {
    let total: usize = section.groups.iter().map(|g| g.items.len()).sum();
    div(
        vec![("class", "docs-category-section")],
        vec![
            div(
                vec![("class", "docs-category-section-head")],
                vec![h2(vec![], vec![text(section.title)]), count_badge(total)],
            ),
            ul(
                vec![("class", "docs-category-grid")],
                section
                    .groups
                    .iter()
                    .map(|g| card_node(base_path, g))
                    .collect(),
            ),
        ],
    )
}

fn render_sections(base_path: &str, sections: &[Section<'_>]) -> Vec<Node> {
    sections
        .iter()
        .filter(|s| !s.groups.is_empty())
        .map(|s| section_node(base_path, s))
        .collect()
}

/// `/blocks/` 用: 区分ごとにカテゴリのカードを並べる（台帳は
/// [`blocks::all_blocks`]）。0 件の区分・カテゴリは省き、カテゴリ内は `path`
/// の辞書順（並列 PR による登録順の揺れを表示順へ持ち込まない）。
#[must_use]
pub fn render_blocks(base_path: &str) -> Vec<Node> {
    let all = blocks::all_blocks();
    let sections: Vec<Section<'_>> = BlockSection::ALL
        .iter()
        .map(|section| {
            let groups = BlockCategory::ALL
                .iter()
                .filter(|c| c.section() == *section)
                .filter_map(|category| {
                    let mut items: Vec<Item<'_>> = all
                        .iter()
                        .filter(|b| b.category == *category)
                        .map(|b| Item {
                            path: b.path,
                            title: b.title,
                        })
                        .collect();
                    if items.is_empty() {
                        return None;
                    }
                    items.sort_by_key(|i| i.path);
                    Some(Group {
                        title: category.label(),
                        items,
                    })
                })
                .collect();
            Section {
                title: section.label(),
                groups,
            }
        })
        .collect();
    render_sections(base_path, &sections)
}

/// `/wireframes/` 用: カテゴリ別カード（台帳は [`WIREFRAMES`]）。区分は
/// 「部品一覧」の 1 つだけ。
#[must_use]
pub fn render_wireframes(base_path: &str) -> Vec<Node> {
    let groups: Vec<Group<'_>> = WireframeCategory::ALL
        .iter()
        .filter_map(|category| {
            let mut items: Vec<Item<'_>> = WIREFRAMES
                .iter()
                .filter(|w| w.category == *category)
                .map(|w| Item {
                    path: w.path,
                    title: w.title,
                })
                .collect();
            if items.is_empty() {
                return None;
            }
            items.sort_by_key(|i| i.path);
            Some(Group {
                title: category.label(),
                items,
            })
        })
        .collect();
    render_sections(
        base_path,
        &[Section {
            title: "部品一覧",
            groups,
        }],
    )
}

/// 索引カード専用 CSS（`assets/category-index.css`）。色・余白・角丸は既存の
/// `--fandhe-*` トークンだけを参照し、新しいトークンは作らない。
/// `.docs-content` 配下の typography ミラーに詳細度で勝つため、セレクタは
/// `.docs-content` を前置する。最小トラック幅 13rem は 1440px で 3 列、768px で
/// 2 列、375px で 1 列を得るための値（`component_index` と同じ）。
///
/// # Errors
///
/// [`StyleSheet::push_css`] の検証（`<`・制御文字の拒否）に落ちた場合。
/// 定数のため通常は到達しないが、黙って欠けた CSS を公開しない fail-closed。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_css(CATEGORY_INDEX_CSS)?;
    Ok(sheet)
}

const CATEGORY_INDEX_CSS: &str = "\
.docs-content .docs-category-section {
  margin-block-end: var(--fandhe-space-6);
}
.docs-content .docs-category-section-head {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: var(--fandhe-space-2);
}
.docs-content .docs-category-section-head h2 {
  margin-block-end: 0;
}
.docs-content ul.docs-category-grid {
  list-style: none;
  margin: var(--fandhe-space-4, 1rem) 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 13rem), 1fr));
  align-items: start;
  gap: var(--fandhe-space-3, 0.75rem);
}
.docs-content li.docs-category-card {
  margin: 0;
  min-width: 0;
}
.docs-content .docs-category-card > [data-scope=\"card\"] {
  box-sizing: border-box;
}
.docs-content .docs-category-card-head {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: var(--fandhe-space-2);
  margin-block-end: var(--fandhe-space-2);
}
.docs-content .docs-category-card h3 {
  margin: 0;
  padding: 0;
  border: 0;
  font-size: var(--fandhe-font-font-size-md);
}
.docs-content ul.docs-category-card-list {
  list-style: none;
  margin: 0;
  padding: 0;
  font-size: var(--fandhe-font-font-size-sm);
}
.docs-content ul.docs-category-card-list > li {
  margin: 0;
  padding: 0;
  line-height: 1.6;
}
.docs-content ul.docs-category-card-list a {
  overflow-wrap: anywhere;
}
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn html(nodes: &[Node]) -> String {
        nodes.iter().map(render).collect()
    }

    #[test]
    fn titles_categories_and_counts_are_escaped_and_derived() {
        let sections = [Section {
            title: "<i>&",
            groups: vec![Group {
                title: "<b>&c",
                items: vec![
                    Item {
                        path: "/x/",
                        title: "<script>t</script>",
                    },
                    Item {
                        path: "/y/",
                        title: "u",
                    },
                ],
            }],
        }];
        let out = html(&render_sections("/b", &sections));
        assert!(!out.contains("<i>") && !out.contains("<b>") && !out.contains("<script"));
        assert!(out.contains("&lt;i&gt;&amp;"));
        assert!(out.contains("&lt;b&gt;&amp;c"));
        assert!(out.contains("&lt;script&gt;t&lt;/script&gt;"));
        assert_eq!(out.matches("2 件").count(), 2);
        assert!(out.contains("href=\"/b/x/\""));
        // id はカテゴリ名の slug（`group_anchor_id`）のみ（#3670）。
        assert_eq!(out.matches(" id=\"").count(), 1);
        assert!(out.contains(" id=\"b-c\""));
        assert!(!out.contains("javascript:"));
    }

    #[test]
    fn empty_groups_and_sections_are_omitted() {
        let sections = [Section {
            title: "EMPTY-SECTION",
            groups: vec![],
        }];
        assert!(render_sections("", &sections).is_empty());
    }

    #[test]
    fn blocks_render_one_link_per_block() {
        let out = html(&render_blocks(""));
        let n = blocks::all_blocks().len();
        assert_eq!(out.matches("<li><a href=\"/blocks/").count(), n);
        assert!(out.contains(">Application<") && out.contains(">Auth<"));
        assert!(out.contains("href=\"/blocks/login-01/\""));
        if let Some(empty) = BlockCategory::ALL
            .iter()
            .find(|c| !blocks::all_blocks().iter().any(|b| b.category == **c))
        {
            let marker = format!(">{}<", empty.label());
            assert!(!out.contains(&marker), "{marker}");
        }
    }

    #[test]
    fn wireframes_render_one_link_per_component() {
        let out = html(&render_wireframes(""));
        assert_eq!(
            out.matches("<li><a href=\"/wireframes/").count(),
            WIREFRAMES.len()
        );
    }

    #[test]
    fn stylesheet_assembles() {
        assert!(stylesheet()
            .unwrap()
            .as_css()
            .contains("docs-category-grid"));
    }
}
