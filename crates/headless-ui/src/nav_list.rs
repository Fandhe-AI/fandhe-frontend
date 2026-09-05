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
//!
//! # [`mod@crate::navigation_menu`] との使い分け（イシュー #993）
//!
//! 本モジュールは状態機械を一切持たない静的なリンク集（見出し + リンク
//! リストのみ、ディスクロージャなし）である。[`mod@crate::navigation_menu`]
//! は Trigger/Content によるディスクロージャ（クリックでパネルが開閉する）
//! と「高々 1 個の Trigger だけが開く」状態機械を持つ点で異なる。両者とも
//! `role` を明示付与しない判断は共通であり、使い分けの軸は role の有無では
//! なく**ディスクロージャの有無**である。単なるリンク集は本モジュールを、
//! 開閉するナビゲーションパネルが必要な場合は
//! [`mod@crate::navigation_menu`] を使う。
//!
//! # 参考サイト突合（イシュー #1653）
//!
//! `docs/design/component-coverage-map.md:837` のとおり、本部品は ark-ui /
//! Radix Primitives / Radix Themes に 1:1 対応物を持たない fandhe 独自部品
//! （#756）。イシューが指す chakra-ui `List`
//! （`.agents/skills/chakra-ui/references/components/typography/list.md`）は
//! `variant`/`align`/`colorPalette`/`asChild`/`List.Indicator` を持つ**汎用の
//! marker 付きリスト**であり、Anatomy 図・Keyboard Interactions 表・独自
//! ARIA を持たない。chakra `List` の本リポジトリでの真の対応物は Themes 層
//! [`fandhe-frontend-pre-styled-ui` の `list`](https://docs.rs/fandhe-frontend-pre-styled-ui)
//! （#771）であり、本モジュールは「`nav` ランドマーク + 見出し + リンク
//! リスト」という文書ナビの意味論を持つ別部品として区別する。突合の結論:
//!
//! - **anatomy**: 参照側に 1:1 の Anatomy 図なし。chakra `List.Root`/
//!   `List.Item` は本モジュールの [`list`]/[`item`] に相当し、[`root`]
//!   （`nav`）/[`heading`]（`h2`）/[`link`]（`a`、`aria-current`/`data-current`
//!   語彙）は文書ナビ固有の superset。**増減なし**。
//! - **`data-*`**: 参照側は状態 `data-*` を持たない。`data-current` は
//!   [`mod@crate::link`]/[`mod@crate::breadcrumb`] と共有する本リポジトリ
//!   独自語彙であり、削除は `fandhe-frontend-pre-styled-ui` の golden CSS
//!   セレクタへ波及する破壊的変更のため意図的に維持する。**増減なし**。
//! - **WAI-ARIA**: 上記「`role` を一切付与しない」節・`aria-label` 必須化の
//!   とおり。参照側も独自 ARIA を持たず暗黙ロール依存の点で一致する。
//! - **キーボード**: 参照側に表なし。ネイティブ `a[href]` の Tab /
//!   Shift+Tab によるフォーカス移動と Enter による起動のみ（Space は `<a>`
//!   を起動しない）。矢印キーでの roving は上記「スコープ外」節のとおり
//!   文書ナビパターン外であり意図的に非提供のまま。
//! - **是正**: [`crate::breadcrumb`]/[`crate::link_overlay`] と同型の予約
//!   キーなりすまし除去（[`drop_reserved`]）を追加した（従来
//!   [`fandhe_frontend_core::el`] が属性の重複除去をしないため、呼び出し側
//!   `attrs` 経由で `aria-label`/`href`/`aria-current`/`data-current` を
//!   重複出力・なりすまし可能だった）。
//! - **意図的に合わせなかった差分**: chakra `List.Indicator`（装飾マーカー。
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 2 により headless
//!   へ持ち込まず Themes 層の責務とする）/ `as="ol"`（文書ナビは順序なし
//!   リストとして `ul` 固定）/ `variant`・`align`・`colorPalette`・
//!   `unstyled`（装飾軸、Themes 責務）/ `asChild`（Slot 相当の再導入は同
//!   §3.25 の再評価トリガーに従属）/ `heading` の見出しレベル可変化（API
//!   拡張のため本イシューでは非対応、別 issue 化候補）。
//!
//! anatomy / `data-*` / ARIA の増減はゼロのため、Themes 側 #1529（closed）
//! への追加通知は不要と判断した。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_label, AriaCurrent};
use crate::data_attrs::data_current;
use fandhe_frontend_core::Node;

/// NavList の anatomy（`data-scope="nav-list"`）。
const ANATOMY: Anatomy = anatomy("nav-list");

/// [`root`] が固定付与する予約キー（イシュー #1653、`crate::breadcrumb` と
/// 同型）。
const ROOT_RESERVED: &[&str] = &["aria-label"];

/// [`link`] が固定付与する予約キー。`aria-current`/`data-current` は
/// `current` の真偽に関わらず無条件に除去する（`current=false` の呼び出し
/// へ呼び出し側が `aria-current`/`data-current` を渡すのが、まさに防ぎたい
/// 現在ページなりすましのため）。
const LINK_RESERVED: &[&str] = &["href", "aria-current", "data-current"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは同名属性の
/// 重複出力・状態属性のなりすましを許してしまう（`crate::breadcrumb::drop_reserved`
/// と同型、イシュー #1653）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `root` パーツ（`nav`）。`label` は `aria-label` として付与し必須引数
/// （本モジュール冒頭の rustdoc「`root` の `aria-label` を必須引数にする
/// 理由」参照）。
#[must_use]
pub fn root<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
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
    let attrs = drop_reserved(attrs, LINK_RESERVED);
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

    #[test]
    fn root_aria_label_spoofing_is_dropped() {
        // 呼び出し側 `attrs` に `aria-label` を紛れ込ませても、本モジュール
        // が固定付与する値のみが出力される（イシュー #1653）。
        let html = render(&root(
            "Documentation",
            vec![("aria-label", "attacker")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label").count(), 1);
        assert!(html.contains(r#"aria-label="Documentation""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn link_href_spoofing_is_dropped() {
        // `attrs` 経由の `href` なりすましは無視され、第一引数の `href` の
        // みが出力される（重複属性・URL 差し替えの防止、イシュー #1653）。
        let html = render(&link(
            "/docs/intro",
            false,
            vec![("href", "javascript:alert(1)")],
            vec![],
        ));
        assert_eq!(html.matches("href=").count(), 1);
        assert!(html.contains(r#"href="/docs/intro""#));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn link_aria_current_and_data_current_spoofing_is_dropped_even_when_not_current() {
        // `current=false` でも `attrs` 経由の `aria-current`/`data-current`
        // なりすましは除去される（現在ページなりすましの防止、イシュー
        // #1653）。
        let html = render(&link(
            "/docs/intro",
            false,
            vec![("aria-current", "page"), ("data-current", "")],
            vec![],
        ));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }
}
