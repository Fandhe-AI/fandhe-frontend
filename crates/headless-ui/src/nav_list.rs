//! NavList（文書ナビ向け Link リスト）headless コンポーネント（イシュー #756、
//! 親 #748 Phase 4、ルート #726、#716 最優先候補）。
//!
//! `docs/design/docs-site-styled-ui-adoption.md` §3.1 が「pre-styled-ui の
//! `menu` 部品は WAI-ARIA `menu` ロール（キーボード操作を伴う操作ドロップ
//! ダウン・コマンドリスト向け）であり、`nav` 要素 + リンクリストという文書
//! ナビの意味論とは異なる。`menu` ロールを文書ナビへ転用するとスクリーン
//! リーダー利用者に『操作可能なメニュー』と誤って伝わりアクセシビリティを
//! 毀損する」と評価した意味論不整合を解消するために新設する専用部品。
//!
//! anatomy は `root`（`nav`）/ `heading`（`h2`、セクション見出し）/
//! `list`（`ul`）/ `item`（`li`）/ `link`（`a`）の 5 パーツ構成。
//! [`mod@crate::breadcrumb`]/[`mod@crate::link`] と同型で状態機械
//! （[`crate::state`]）は持たない。
//!
//! # `role` を一切付与しない（本部品の存在理由）
//!
//! [`root`]/[`heading`]/[`list`]/[`item`]/[`link`] のいずれも `role` 属性を
//! 一切付与しない。素の `nav`/`h2`/`ul`/`li`/`a` の暗黙 ARIA ロール
//! （`navigation`/`heading`/`list`/`listitem`/`link`）をそのまま使うことが
//! 「操作可能なメニュー」との誤読を避ける本部品の存在理由そのものである
//! （[`mod@crate::menu`] の `role="menu"`/`role="menuitem"` とは意味論上
//! 明確に区別する）。
//!
//! # `current` について
//!
//! [`link`] の `current` 引数を `true` にすると `aria-current="page"`
//! （[`crate::aria::aria_current`]）+ `data-current`
//! （[`crate::data_attrs::data_current`]）を付与する。[`mod@crate::breadcrumb`]/
//! [`mod@crate::link`] と同じ語彙を共有する。
//!
//! # `root` の `aria-label` を必須引数にする理由
//!
//! 文書に複数の `nav` ランドマークが存在する場合、アクセシブルネームが
//! ないとスクリーンリーダー利用者がランドマーク間を区別できない。
//! [`avatar::image`] の `alt` 必須化と同型の判断として、`label` を必須
//! 引数にすることでアクセシビリティ担保を型で強制する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site`（`crates/docs-site/src/nav.rs::sidebar`）が
//! 本モジュールの再エクスポート（`fandhe-frontend-pre-styled-ui` 経由、
//! イシュー #756）を使ってサイドバーの文書ナビを組み立てる想定。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（`href`/attrs/children は
//! [`fandhe_frontend_core::render`] の既定エスケープを必ず経由し、`href` の
//! 危険 URL スキームは core の許可リスト方式が属性ごと拒否する）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - キーボードナビゲーション（矢印キーでの項目間移動）は WAI-ARIA の
//!   文書ナビパターンに存在しない（通常の Tab 移動のみ）ため提供しない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_label, AriaCurrent};
use crate::data_attrs::data_current;
use fandhe_frontend_core::Node;

/// NavList の anatomy（`data-scope="nav-list"`）。
const ANATOMY: Anatomy = anatomy("nav-list");

/// `root` パーツ（`nav`）。`label` は `aria-label` として付与し必須引数
/// （本モジュール冒頭の rustdoc「`root` の `aria-label` を必須引数にする
/// 理由」参照）。
#[must_use]
pub fn root<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_label(label)];
    merged.extend(attrs);
    ANATOMY.part("root", "nav", merged, children)
}

/// `heading` パーツ（`h2`）。セクション見出し。
#[must_use]
pub fn heading(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("heading", "h2", attrs, children)
}

/// `list` パーツ（`ul`）。
#[must_use]
pub fn list(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("list", "ul", attrs, children)
}

/// `item` パーツ（`li`）。
#[must_use]
pub fn item(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "li", attrs, children)
}

/// `link` パーツ（`a`）。`current` が `true` のとき `aria-current="page"` +
/// `data-current` を付与する。`role` は一切付与しない（本モジュール冒頭の
/// rustdoc「`role` を一切付与しない」参照）。
#[must_use]
pub fn link<'a>(
    href: &'a str,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    if current {
        merged.push(aria_current(AriaCurrent::Page));
        merged.extend(data_current(true));
    }
    merged.extend(attrs);
    ANATOMY.part("link", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_nav_with_aria_label() {
        let html = render(&root("Documentation", vec![], vec![]));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"aria-label="Documentation""#));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn heading_list_item_output_expected_tags_without_role() {
        let heading_html = render(&heading(vec![], vec![text("Guides")]));
        assert!(heading_html.starts_with("<h2"));
        assert!(!heading_html.contains("role="));

        let list_html = render(&list(vec![], vec![item(vec![], vec![])]));
        assert!(list_html.starts_with("<ul"));
        assert!(list_html.contains("<li"));
        assert!(!list_html.contains("role="));
    }

    #[test]
    fn link_current_true_adds_aria_current_and_data_current_without_role() {
        let html = render(&link("/docs/intro", true, vec![], vec![text("Intro")]));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("data-current"));
        assert!(!html.contains("role="));
    }

    #[test]
    fn link_current_false_omits_aria_current_and_data_current() {
        let html = render(&link("/docs/intro", false, vec![], vec![]));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "Documentation",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn link_dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&link(url, false, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    #[test]
    fn link_href_attribute_breakout_payload_is_escaped() {
        let html = render(&link(
            "/docs\" onmouseover=\"alert(1)",
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn root_label_is_escaped() {
        let html = render(&root("\"><script>alert(1)</script>", vec![], vec![]));
        assert!(!html.contains("<script>"));
    }
}
