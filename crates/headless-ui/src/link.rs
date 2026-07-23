//! Link（汎用インラインリンク）headless コンポーネント（イシュー #756、
//! 親 #748 Phase 4、ルート #726）。
//!
//! `docs/api/headless-ui-api.md` §4b（イシュー #716）の検討で「分類 (b):
//! SSR 静的な意味論ナビ（状態機械不要）」の追加候補と判断されたコンポーネント。
//! chakra-ui の Link に倣い、素の `a` 要素 1 パーツ（anatomy `root`）のみを
//! 提供する最小構成。[`mod@crate::breadcrumb`]/[`mod@crate::field`] と同型で、
//! 開閉のような時間変化する内部状態を持たないため自由関数のみで構成する
//! （[`crate::state`] の状態機械は適用しない）。
//!
//! [`mod@crate::link_overlay`]（カード全面クリック化）・[`mod@crate::nav_list`]
//! （文書ナビの `nav > ul > li > a` 構造）は本モジュールに依存せず並立する
//! 別モジュールとして提供する（3 者とも独立した anatomy スコープを持つ）。
//!
//! # `external` オプトイン（reverse tabnabbing 対策）
//!
//! `external` 引数を `true` にすると `target="_blank"` と
//! `rel="noopener noreferrer"` を**不可分に**付与する。片方のみを付与できる
//! API は公開しない（`target="_blank"` 単独では遷移先ページが
//! `window.opener` 経由で元ページを操作できてしまう reverse tabnabbing の
//! 脆弱性を生むため）。
//!
//! # `current` について
//!
//! `current` 引数を `true` にすると `aria-current="page"`
//! （[`crate::aria::aria_current`]）+ `data-current`
//! （[`crate::data_attrs::data_current`]）を付与する。[`mod@crate::breadcrumb`]
//! の `current_link`・[`mod@crate::nav_list`] の `link` と同じ語彙を共有する。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//! へ薄く委譲するのみで、独自の出力経路・独自のエスケープ処理は持たない。
//! styled 層（`fandhe-frontend-pre-styled-ui`）は本モジュールが出力する
//! `data-scope="link"`/`data-part="root"` セレクタを前提にスタイルを当てる。
//!
//! # セキュリティ不変条件
//!
//! - `href`/呼び出し側 `attrs`/子ノードはすべて [`fandhe_frontend_core::el`]
//!   の属性値・子ノードとして渡り、[`fandhe_frontend_core::render`] の既定
//!   エスケープ（REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用
//!   せず、HTML 文字列を直接組み立てない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//! - `href` の URL スキーム検証（`javascript:` 等の拒否）は
//!   `fandhe_frontend_core::render` 側の既定経路（許可スキームのみを通す
//!   deny-by-default。不正な値は属性ごと出力されない）が担う。本モジュールは
//!   独自の URL 検証を追加しない（[`crate::breadcrumb::link`] と同じ整理）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - クライアント側ナビゲーション連携（SPA ルーター統合）・`asChild` 相当機能。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, AriaCurrent};
use crate::data_attrs::data_current;
use fandhe_frontend_core::Node;

/// Link の anatomy（`data-scope="link"`）。
const ANATOMY: Anatomy = anatomy("link");

/// `root` パーツ（`a`）。唯一の anatomy パーツ。
///
/// - `external` が `true` のとき `target="_blank"` + `rel="noopener noreferrer"`
///   を不可分に付与する（本モジュール冒頭の rustdoc「reverse tabnabbing 対策」
///   参照）。
/// - `current` が `true` のとき `aria-current="page"` + `data-current` を
///   付与する。
#[must_use]
pub fn root<'a>(
    href: &'a str,
    external: bool,
    current: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    if external {
        merged.push(("target", "_blank"));
        merged.push(("rel", "noopener noreferrer"));
    }
    if current {
        merged.push(aria_current(AriaCurrent::Page));
        merged.extend(data_current(true));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_and_href() {
        let html = render(&root("/docs", false, false, vec![], vec![text("Docs")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/docs""#));
        assert!(html.contains(">Docs<"));
    }

    #[test]
    fn external_true_adds_target_and_rel_inseparably() {
        let html = render(&root("https://example.com", true, false, vec![], vec![]));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn external_false_omits_target_and_rel() {
        let html = render(&root("/docs", false, false, vec![], vec![]));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
    }

    #[test]
    fn current_true_adds_aria_current_and_data_current() {
        let html = render(&root("/docs", false, true, vec![], vec![]));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("data-current"));
    }

    #[test]
    fn current_false_omits_aria_current_and_data_current() {
        let html = render(&root("/docs", false, false, vec![], vec![]));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "/docs",
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- URL スキーム拒否（fail-closed、core の render() 経由） ---

    #[test]
    fn dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "\u{0}javascript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&root(url, false, false, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    // --- エスケープ回帰 ---

    #[test]
    fn href_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            "/docs\" onmouseover=\"alert(1)",
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(
            "/docs",
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
