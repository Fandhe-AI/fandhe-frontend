//! Themes・Primitives 索引（`/themes/`・`/primitives/`）のカテゴリ別カード
//! グリッド（イシュー #3617）。
//!
//! # 役割・呼び出し文脈
//!
//! 2 つの索引ページはこれまで手書き Markdown のリンク集で、`site/nav.toml`
//! と二重管理になりずれていた（Themes は 5 部品が欠落）。本モジュールは
//! [`crate::page_sections`]（汎用生成節フック、#3598）へ登録する生成関数
//! （[`render_themes`] / [`render_primitives`]）と専用 CSS（[`stylesheet`]）を
//! 提供し、台帳（[`crate::themes_catalog`] / [`crate::primitives_catalog`]）から
//! カテゴリ別のカードグリッドを差し込む。呼び出し元は
//! [`crate::build::build_site_with`] 内の `insert_generated_sections_with` と、
//! 書き出し時の `PageStylesheet::build`（[`stylesheet`]）のみ。
//!
//! # 構造（カテゴリ 1 件あたり）
//!
//! ```text
//! div.docs-catalog-category
//!   div.docs-catalog-category-head
//!     h2                      … カテゴリ名（TOC・検索に載る。id は見出しアンカーが採番）
//!     badge                   … 件数。h2 の兄弟（h2 内に入れると TOC 題名が汚れる）
//!   ul.docs-catalog-grid
//!     li.docs-catalog-card    … position: relative
//!       card::root > card::body
//!         heading(H3) > a.docs-catalog-card-link   … ::after で全面クリック化
//!         text(Muted, Sm)                          … 台帳の 1 行説明
//!         span.docs-catalog-card-meta > badge      … 層（Themes / Primitives）
//! ```
//!
//! 画像サムネイルは使わない（部品数が多く、軽量さを優先）。pre-styled-ui の root は
//! 呼び出し側の `class` を破棄するため、`docs-catalog-*` class は常にラッパー要素
//! （`div` / `ul` / `li` / `a` / `span`）へ付ける。
//!
//! # 検索インデックスとの関係
//!
//! li の class を意図的に `docs-index-card` にしない。`search_index` の特例
//! （カード内全文の収集）を効かせず、カード内（`data-scope` 配下）を検索テキスト
//! から外す。部品ページ自体が個別に索引化済みで重複ヒットを避けられ、123 枚分の
//! 説明文が 1 ページ 4000 バイトの切り詰めで凡例を押し出すこともなくなるため。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API のみで組み、題名・説明・件数は既定エスケープを通る（`raw_html()`
//! 不使用、REQ-1）。href は [`crate::layout::asset_href`] によるサイト内の絶対
//! パスだけで、台帳はコンパイル時定数（外部入力を含まない）。JS・`on*=` 属性・
//! `id` は出さない。

use fandhe_frontend_core::{a, div, h2, li, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{text as text_part, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size, StyleSheet, StylesheetError};

use crate::layout::asset_href;
use crate::primitives_catalog::{self, PrimitiveCategory};
use crate::themes_catalog::{self, ThemeCategory};

/// 生成節が配線する追加 CSS の出力先（[`crate::page_sections::PAGE_STYLESHEETS`]
/// の `rel_path`）。
pub const STYLESHEET_REL_PATH: &str = "assets/component-index.css";

/// カードが属する層。層 badge の文言と見た目を決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Themes,
    Primitives,
}

impl Layer {
    fn label(self) -> &'static str {
        match self {
            Layer::Themes => "Themes",
            Layer::Primitives => "Primitives",
        }
    }

    fn badge_props(self) -> BadgeProps {
        match self {
            Layer::Themes => BadgeProps {
                variant: BadgeVariant::Subtle,
                size: Size::Sm,
                palette: ColorPalette::Accent,
                shape: None,
            },
            Layer::Primitives => BadgeProps {
                variant: BadgeVariant::Outline,
                size: Size::Sm,
                palette: ColorPalette::Neutral,
                shape: None,
            },
        }
    }
}

/// カード 1 枚分の表示データ（台帳エントリから写す）。
#[derive(Debug, Clone, Copy)]
struct CardData<'a> {
    path: &'a str,
    title: &'a str,
    description: &'a str,
}

/// カード 1 枚（`li.docs-catalog-card`）を組む。
fn card_node(base_path: &str, c: &CardData<'_>, layer: Layer) -> Node {
    let href = asset_href(base_path, c.path);
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![],
        vec![a(
            vec![("class", "docs-catalog-card-link"), ("href", href.as_str())],
            vec![text(c.title)],
        )],
    );
    let desc = text_part(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(c.description)],
    );
    let meta = span(
        vec![("class", "docs-catalog-card-meta")],
        vec![badge::badge(
            &layer.badge_props(),
            vec![],
            vec![text(layer.label())],
        )],
    );
    li(
        vec![("class", "docs-catalog-card")],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![card::body(vec![], vec![title, desc, meta])],
        )],
    )
}

/// カテゴリ 1 件（見出し行 + カードグリッド）を組む。件数は `cards.len()` から算出する。
fn category_node(
    base_path: &str,
    category_title: &str,
    cards: &[CardData<'_>],
    layer: Layer,
) -> Node {
    let count = badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Sm,
            palette: ColorPalette::Neutral,
            shape: None,
        },
        vec![],
        vec![text(format!("{} 件", cards.len()))],
    );
    div(
        vec![("class", "docs-catalog-category")],
        vec![
            div(
                vec![("class", "docs-catalog-category-head")],
                vec![
                    // 明示 id（値は自動採番と同じ slug）。ヘッダー popup・サイドバーと
                    // `nav::group_anchor_id` を共有し、自動採番への暗黙依存を避ける（#3670）。
                    h2(
                        vec![("id", crate::nav::group_anchor_id(category_title).as_str())],
                        vec![text(category_title)],
                    ),
                    count,
                ],
            ),
            ul(
                vec![("class", "docs-catalog-grid")],
                cards
                    .iter()
                    .map(|c| card_node(base_path, c, layer))
                    .collect(),
            ),
        ],
    )
}

/// `/themes/` 用: カテゴリ別カードグリッド（台帳は [`themes_catalog`]）。
#[must_use]
pub fn render_themes(base_path: &str) -> Vec<Node> {
    ThemeCategory::all()
        .iter()
        .map(|cat| {
            let cards: Vec<CardData<'_>> = themes_catalog::entries_in(*cat)
                .map(|e| CardData {
                    path: e.path,
                    title: e.title,
                    description: e.description,
                })
                .collect();
            category_node(base_path, cat.title(), &cards, Layer::Themes)
        })
        .collect()
}

/// `/primitives/` 用: カテゴリ別カードグリッド（台帳は [`primitives_catalog`]）。
#[must_use]
pub fn render_primitives(base_path: &str) -> Vec<Node> {
    PrimitiveCategory::all()
        .iter()
        .map(|cat| {
            let cards: Vec<CardData<'_>> = primitives_catalog::entries_in(*cat)
                .map(|e| CardData {
                    path: e.path,
                    title: e.title,
                    description: e.description,
                })
                .collect();
            category_node(base_path, cat.title(), &cards, Layer::Primitives)
        })
        .collect()
}

/// 索引カード専用 CSS（`assets/component-index.css`）。色・余白・角丸は既存の
/// `--fandhe-*` トークンだけを参照し、新しいトークンは作らない。
/// `.docs-content` 配下の typography ミラー（`.docs-content ul/li/a/h2/p`）に
/// 詳細度で勝つため、セレクタは `.docs-content` を前置する。最小トラック幅
/// 13rem は、本文幅 46rem で 1440px 幅に 3 列、768px 幅に 2 列、375px 幅に
/// 1 列を得るための値（15rem では 1440px でも 2 列にしかならない）。
///
/// # Errors
///
/// [`StyleSheet::push_css`] の検証（`<`・制御文字の拒否）に落ちた場合。
/// 定数のため通常は到達しないが、黙って欠けた CSS を公開しない fail-closed。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_css(COMPONENT_INDEX_CSS)?;
    Ok(sheet)
}

const COMPONENT_INDEX_CSS: &str = "\
.docs-content .docs-catalog-category {
  margin-block-end: var(--fandhe-space-6);
}
.docs-content .docs-catalog-category-head {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: var(--fandhe-space-2);
}
.docs-content .docs-catalog-category-head h2 {
  margin-block-end: 0;
}
.docs-content ul.docs-catalog-grid {
  list-style: none;
  margin: var(--fandhe-space-4, 1rem) 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 13rem), 1fr));
  gap: var(--fandhe-space-3, 0.75rem);
}
.docs-content li.docs-catalog-card {
  position: relative;
  margin: 0;
  min-width: 0;
}
.docs-content .docs-catalog-card > [data-scope=\"card\"] {
  height: 100%;
  box-sizing: border-box;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}
.docs-content .docs-catalog-card:hover > [data-scope=\"card\"] {
  border-color: var(--fandhe-color-accent);
  background-color: var(--fandhe-color-bg-subtle);
}
.docs-content .docs-catalog-card h3 {
  margin: 0 0 var(--fandhe-space-1);
  padding: 0;
  border: 0;
  font-size: var(--fandhe-font-font-size-md);
}
.docs-content .docs-catalog-card p {
  margin: 0;
}
.docs-content .docs-catalog-card-meta {
  display: block;
  margin-block-start: var(--fandhe-space-2);
}
.docs-content a.docs-catalog-card-link,
.docs-content a.docs-catalog-card-link:hover {
  color: var(--fandhe-color-fg);
  text-decoration: none;
}
.docs-content a.docs-catalog-card-link::after {
  content: \"\";
  position: absolute;
  inset: 0;
  border-radius: var(--fandhe-radius-sm);
}
.docs-content a.docs-catalog-card-link:focus-visible {
  outline: none;
}
.docs-content a.docs-catalog-card-link:focus-visible::after {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
@media (prefers-reduced-motion: reduce) {
  .docs-content .docs-catalog-card > [data-scope=\"card\"] {
    transition: none;
  }
}
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn titles_and_descriptions_are_escaped_by_default() {
        let c = CardData {
            path: "/x/",
            title: "<b>&t</b>",
            description: "<script>d</script>",
        };
        let html = render(&card_node("/b", &c, Layer::Themes));
        assert!(!html.contains("<b>") && !html.contains("<script>"));
        assert!(html.contains("&lt;b&gt;&amp;t&lt;/b&gt;"));
        assert!(html.contains("href=\"/b/x/\""));
    }

    #[test]
    fn category_title_and_count_are_escaped_and_derived() {
        let cards = [CardData {
            path: "/x/",
            title: "t",
            description: "d",
        }];
        let html = render(&category_node("", "<i>&", &cards, Layer::Primitives));
        assert!(!html.contains("<i>"));
        assert!(html.contains("&lt;i&gt;&amp;"));
        assert!(html.contains("1 件"));
        // id はカテゴリ名の slug（`nav::group_anchor_id`）の 1 件だけ（#3670）。
        assert_eq!(html.matches(" id=\"").count(), 1);
        assert!(html.contains(" id=\"i\""));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn themes_render_one_card_per_catalog_entry() {
        let html: String = render_themes("").iter().map(render).collect();
        assert_eq!(
            html.matches("class=\"docs-catalog-card\"").count(),
            themes_catalog::THEMES.len()
        );
    }

    #[test]
    fn primitives_render_one_card_per_catalog_entry() {
        let html: String = render_primitives("").iter().map(render).collect();
        assert_eq!(
            html.matches("class=\"docs-catalog-card\"").count(),
            primitives_catalog::PRIMITIVES.len()
        );
    }

    #[test]
    fn stylesheet_assembles() {
        assert!(stylesheet().unwrap().as_css().contains("docs-catalog-grid"));
    }
}
