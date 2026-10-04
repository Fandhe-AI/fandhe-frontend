//! メニュー集約ページ（`/assets/` 等）のカードグリッド生成（イシュー #3700）。
//!
//! # 役割・呼び出し文脈
//!
//! `site/nav.toml` の `[[menu]]`（#3699）は複数セクションを 1 つのヘッダー項目へ
//! 束ねる。ヘッダーのトリガーは無 JS の静的アンカーなので、タッチ端末や
//! スクリーンリーダーで選んだ遷移先となる集約ページが要る。本モジュールは
//! そのページへ差し込むカード列を作る。[`crate::page_sections::PAGE_SECTIONS`]
//! に登録され、[`crate::build::build_site_with`] のメニューページ生成ループから
//! [`crate::page_sections::insert_generated_sections_with`] 経由で呼ばれる。
//!
//! # カードの正
//!
//! カードの並び・説明・リンク先は `nav.toml` を唯一の正とする。並びは
//! `[[menu.item]]` の宣言順、タイトルはメンバーセクションの `title`、説明は
//! `description`、リンク先はセクションの `index_path`。見た目は
//! [`crate::section_index`] のカード（`docs-index-*`）を再利用し、新しい class・
//! CSS は持たない。
//!
//! # セキュリティ上の不変条件（REQ-1）
//!
//! 文字列はすべて `text()` 経由で既定エスケープされ、`raw_html()` は使わない。
//! href は `parse_nav` が検証済みの `index_path` だけから `asset_href` で組む。

use fandhe_frontend_core::Node;
use fandhe_frontend_pre_styled_ui::heading::HeadingLevel;

use crate::nav::Nav;
use crate::section_index;

/// 集約ページが配線する追加 CSS（セクション索引と共通）。
pub const STYLESHEET_REL_PATH: &str = section_index::STYLESHEET_REL_PATH;

/// `path` を `index_path` に持つメニューのカードグリッドを返す。
/// 完全一致のみ（メンバー配下のページでは空）。一致しなければ空の `Vec`。
#[must_use]
pub fn render(nav: &Nav, path: &str) -> Vec<Node> {
    let Some(menu) = nav.menus.iter().find(|m| m.index_path == path) else {
        return Vec::new();
    };
    let cards: Vec<Node> = nav
        .menu_members(menu)
        .map(|(section, item)| {
            section_index::link_card(
                &nav.site.base_path,
                &section.index_path,
                &section.title,
                &item.description,
                HeadingLevel::H2,
            )
        })
        .collect();
    vec![section_index::link_grid(cards)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::parse_nav;
    use fandhe_frontend_core::render as render_html;

    fn nav() -> Nav {
        parse_nav(
            r#"
[site]
title = "T"
base_path = "/b"

[[section]]
title = "A <script>"
index_path = "/a/"

[[section.page]]
title = "A"
source = "site/a.md"
path = "/a/"

[[section]]
title = "B"
index_path = "/bb/"

[[section.page]]
title = "B"
source = "site/b.md"
path = "/bb/"

[[menu]]
title = "Assets"
index_path = "/assets/"
source = "site/assets.md"

[[menu.item]]
section = "/bb/"
description = "desc b \" & x"

[[menu.item]]
section = "/a/"
description = "desc a"
"#,
        )
        .unwrap()
    }

    #[test]
    fn cards_follow_menu_item_order_with_prefixed_hrefs() {
        let html = render_html(&render(&nav(), "/assets/")[0]);
        let b = html.find("href=\"/b/bb/\"").unwrap();
        let a = html.find("href=\"/b/a/\"").unwrap();
        assert!(b < a);
        assert_eq!(html.matches("docs-index-card-link").count(), 2);
    }

    #[test]
    fn strings_are_escaped() {
        let html = render_html(&render(&nav(), "/assets/")[0]);
        assert!(!html.contains("<script>"));
        assert!(html.contains("A &lt;script&gt;"));
        assert!(html.contains("&amp; x"));
    }

    #[test]
    fn unknown_or_member_path_is_empty() {
        assert!(render(&nav(), "/a/").is_empty());
        assert!(render(&nav(), "/nope/").is_empty());
    }
}
