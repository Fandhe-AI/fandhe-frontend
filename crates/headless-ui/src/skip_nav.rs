//! SkipNav（イシュー #776、親 #766 Phase 6、`docs/design/component-coverage-map.md`
//! skip-nav 行「実装対象 #776」の消化）。
//!
//! chakra-ui/ark-ui の SkipNav ユーティリティに倣い、キーボード操作時のみ
//! 視覚的に現れる「本文へスキップ」リンクを提供する（WCAG 2.1 SC 2.4.1
//! Bypass Blocks）。[`mod@crate::link`]/[`mod@crate::field`] と同型の、
//! 時間変化する内部状態を持たない純粋関数のみで構成する（[`crate::state`]
//! の状態機械は適用しない）。
//!
//! anatomy は `link`（`a`）/ `content`（`div`）の 2 パーツ構成。[`link`] が
//! ページ先頭でクリックされると、ブラウザは `href` の `#<id>` フラグメントへ
//! ジャンプし、[`content`] の `tabindex="-1"` により実 DOM フォーカスも
//! 移動する（`tabindex="-1"` はプログラム的フォーカスのみ許可し Tab 順序には
//! 加えない契約属性、`docs/api/headless-ui-api.md` 準拠）。
//!
//! # href の構成（スキーム注入経路を持たない）
//!
//! [`link`] は呼び出し側から任意の URL を受け取らず、常に `#<id>`
//! （フラグメントのみ）を内部で組み立てる。[`crate::link::root`] のような
//! 任意スキームの `href` を受理する API とは異なり、本モジュールは
//! `javascript:` 等のスキーム注入経路を構造的に持たない（受け入れ条件・
//! セキュリティ考慮の核）。
//!
//! # 契約属性の除去（fail-closed）
//!
//! [`link`] の `href` と [`content`] の `id`/`tabindex` はいずれも本モジュールが
//! 決定する契約属性であり、呼び出し側 `attrs` に同名のキー（大文字小文字を
//! 無視）が含まれていても除去してから合成する（[`crate::separator::separator`]
//! 相当の `fandhe-frontend-pre-styled-ui` 側前例、および
//! [`crate::skeleton`]（`fandhe-frontend-pre-styled-ui`）の `aria-hidden` 除去と
//! 同型の fail-closed 判断）。呼び出し側が偽装した値を混入させると、支援技術
//! ・フォーカス移動先の双方が誤った状態を読み取ってしまうため。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`] へ薄く委譲するのみで、独自の出力経路・
//!   独自のエスケープ処理は持たない。
//! - styled 層（`fandhe-frontend-pre-styled-ui::skip_nav`）は本モジュールが
//!   出力する `data-scope="skip-nav"` セレクタを前提に、[`crate::recipe::StateCondition::FocusVisible`]
//!   （`fandhe-frontend-pre-styled-ui` 側 API）で focus 時のみ表示する CSS を
//!   当てる（docs-site は hydration を持たないため `data-focus-visible` 配線
//!   ではなく純 CSS の `:focus-visible` に依拠する）。
//! - `fandhe-frontend-docs-site` は [`DEFAULT_ID`] を使ってページ骨格へ 1 個の
//!   [`link`]/[`content`] を常時挿入する（レイアウト実適用、イシュー #776
//!   計画 §docs-site 節参照）。
//!
//! # セキュリティ不変条件
//!
//! - `id`/子ノード・呼び出し側 `attrs` はすべて [`fandhe_frontend_core::el`]
//!   の属性値・子ノードとして渡り、[`fandhe_frontend_core::render`] の既定
//!   エスケープ（REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用
//!   せず、HTML 文字列を直接組み立てない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//! - `format!("#{id}")` は属性値という**データ**の組み立てであり、
//!   `.claude/rules/coding-rust.md` が禁止する「HTML 文字列の直接組み立て」
//!   ではない（[`mod@crate::field`] の `format!("{id}-control")` と同型の整理）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - 複数スキップリンク運用ガイド等のドキュメントサイト向け利用ガイド拡充。

use crate::anatomy::{anatomy, Anatomy};
use fandhe_frontend_core::Node;

/// SkipNav の anatomy（`data-scope="skip-nav"`）。
const ANATOMY: Anatomy = anatomy("skip-nav");

/// 既定のスキップ先 id（ark-ui の `"chakra-skip-nav"` 相当）。
///
/// `fandhe-frontend-docs-site` のようにページ全体へ 1 個だけ SkipNav を
/// 適用する呼び出し側は、この定数を [`link`]/[`content`] 両方へそのまま渡す。
pub const DEFAULT_ID: &str = "fandhe-skip-nav";

/// `link` パーツ（`a`）。`href="#<id>"` を常時出力する。
///
/// 呼び出し側 `attrs` に含まれる `href`（大文字小文字を無視）は除去してから
/// 合成する（モジュール冒頭 rustdoc「契約属性の除去」節参照）。
#[must_use]
pub fn link<'a>(id: &str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let href = format!("#{id}");
    let attrs: Vec<(&str, &str)> = attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("href"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("href", href.as_str())];
    merged.extend(attrs);
    ANATOMY.part("link", "a", merged, children)
}

/// `content` パーツ（`div`）。`id="<id>"` + `tabindex="-1"` を常時出力する。
///
/// `tabindex="-1"` は [`link`] クリック時のプログラム的フォーカス移動のみを
/// 許可し、通常の Tab 順序には加えない（クリックしていない状態では他の
/// フォーカス可能要素より先に Tab で辿り着かない）。呼び出し側 `attrs` に
/// 含まれる `id`/`tabindex`（大文字小文字を無視）は除去してから合成する。
#[must_use]
pub fn content<'a>(id: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs: Vec<(&str, &str)> = attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("id") && !k.eq_ignore_ascii_case("tabindex"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![("id", id), ("tabindex", "-1")];
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn link_outputs_scope_part_and_href() {
        let html = render(&link(DEFAULT_ID, vec![], vec![text("Skip to content")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="skip-nav""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(r##"href="#fandhe-skip-nav""##));
        assert!(html.contains(">Skip to content<"));
    }

    #[test]
    fn content_outputs_scope_part_id_and_tabindex() {
        let html = render(&content(DEFAULT_ID, vec![], vec![]));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="skip-nav""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"id="fandhe-skip-nav""#));
        assert!(html.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn custom_id_propagates_to_both_parts() {
        let html_link = render(&link("custom-target", vec![], vec![]));
        assert!(html_link.contains(r##"href="#custom-target""##));
        let html_content = render(&content("custom-target", vec![], vec![]));
        assert!(html_content.contains(r#"id="custom-target""#));
    }

    #[test]
    fn caller_supplied_href_is_dropped_case_insensitively() {
        for key in ["href", "HREF", "Href"] {
            let html = render(&link(
                DEFAULT_ID,
                vec![(key, "javascript:alert(1)")],
                vec![],
            ));
            assert!(!html.contains("javascript:"));
            assert_eq!(html.matches("href=").count(), 1, "key={key} html={html}");
            assert!(html.contains(r##"href="#fandhe-skip-nav""##));
        }
    }

    #[test]
    fn caller_supplied_id_and_tabindex_are_dropped_case_insensitively() {
        for (key, spoofed) in [
            ("id", "attacker"),
            ("ID", "attacker"),
            ("tabindex", "0"),
            ("TabIndex", "0"),
        ] {
            let html = render(&content(DEFAULT_ID, vec![(key, spoofed)], vec![]));
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("id=").count(), 1, "key={key} html={html}");
            assert_eq!(
                html.matches("tabindex=").count(),
                1,
                "key={key} html={html}"
            );
            assert!(html.contains(r#"id="fandhe-skip-nav""#));
            assert!(html.contains(r#"tabindex="-1""#));
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html_link = render(&link(
            DEFAULT_ID,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html_link.contains(r#"data-scope="skip-nav""#));
        assert!(html_link.contains(r#"data-part="link""#));
        assert!(!html_link.contains("attacker"));

        let html_content = render(&content(
            DEFAULT_ID,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html_content.contains(r#"data-scope="skip-nav""#));
        assert!(html_content.contains(r#"data-part="content""#));
        assert!(!html_content.contains("attacker"));
    }

    // --- id 経由の href/id 属性エスケープ回帰 ---

    #[test]
    fn id_attribute_breakout_payload_is_escaped_in_href() {
        let html = render(&link("x\" onmouseover=\"alert(1)", vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn id_attribute_breakout_payload_is_escaped_in_content_id() {
        let html = render(&content("x\" onmouseover=\"alert(1)", vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&link(
            DEFAULT_ID,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
