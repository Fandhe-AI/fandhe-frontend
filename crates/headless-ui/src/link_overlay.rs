//! LinkOverlay（カード全面クリック化）headless コンポーネント（イシュー #756、
//! 親 #748 Phase 4、ルート #726）。
//!
//! `docs/api/headless-ui-api.md` §4b（イシュー #716）の追加候補。ark-ui には
//! 対応する headless 実体がなく、chakra-ui の LinkBox/LinkOverlay パターン
//! （`docs/design/docs-site-styled-ui-adoption.md` §3.2 が「pre-styled-ui の
//! `card` はアンカー全面クリック化に非対応」と評価した課題を解消する部品）に
//! 倣う。anatomy は `root`（`div`、位置決めコンテキスト）/ `overlay`（`a`、
//! カード全面へ拡張されるリンク）の 2 パーツ構成。[`mod@crate::breadcrumb`]/
//! [`mod@crate::link`] と同型で状態機械（[`crate::state`]）は持たない。
//!
//! # 全面拡張の実装方針（`::before` 疑似要素を使わない理由）
//!
//! 一般的な LinkOverlay 実装（chakra-ui 等）は `::before` 疑似要素で
//! アンカー自身の描画位置を変えずにクリック領域だけを拡張するが、本クレートの
//! styled 層（`SlotRecipe`）は疑似要素セレクタを表現できない
//! （`crates/pre-styled-ui/src/recipe.rs` 参照）。そのため [`overlay`] 自身を
//! `position: absolute; inset: 0;` で [`root`] 全面へ展開する方式を採る
//! （styled 層の CSS 責務。headless 層自体は CSS を持たない）。
//!
//! この方式では [`overlay`] がフローから外れるため、**[`root`] の高さは
//! `overlay` 以外の子ノード（見出し・画像・説明文等の通常フロー要素）が
//! 確立する契約**とする。[`overlay`] に可視テキストを子ノードとして渡すと、
//! その文字は `root` 全面に展開された `overlay` 自身の内側整列に従って
//! 描画される（可視デザイン上は `root` の他の子ノードとして見出しを別途
//! 描画し、`overlay` へはアクセシブルネームのみを `aria-label` 等で与える
//! 運用を推奨する。単一リンクのみで完結するカード（例:
//! `fandhe-frontend-docs-site` の前後ページャ）では `overlay` の子ノードに
//! 直接タイトルを渡してもよい）。
//!
//! # 参照突合（イシュー #1650）
//!
//! 参照実体は chakra-ui の `LinkBox`/`LinkOverlay` のみ（ark-ui の
//! `link-overlay` ページは 404 で対応部品なし、Radix Primitives/Radix
//! Themes にも対応部品なし。`docs/design/component-coverage-map.md`
//! 参照）。chakra-ui は Anatomy 節・Keyboard Interactions 節・`data-*`
//! 語彙・独自 `role`/`aria-*` のいずれも持たない styled 部品であり、本実装の
//! `root`/`overlay` 2 パーツ・`data-scope`/`data-part` は参照側に概念が
//! 存在しない superset である（過不足なし）。状態を表す `data-*`
//! （`data-state`/`data-disabled`/`data-motion` 等）は参照側・本実装とも
//! 一切出力しない（`docs/policy/intentional-non-adoption.md` §3.25 規則 2:
//! 装飾・アニメーション関心を headless へ持ち込まない不変条件）。
//!
//! キーボード操作はネイティブ `a[href]` に委ねる（Tab/Shift+Tab でのフォー
//! カス移動、Enter での起動。Space は起動しない）。ARIA も独自付与せず
//! 暗黙の `link` ロールに委ねる。意図的に合わせなかった点: chakra-ui の
//! `isExternal`（現行 v3 docs では削除済み）・`asChild` は追加しない
//! （前者は位置引数追加が破壊的変更になり呼び出し元全体へ波及、後者は
//! `docs/policy/intentional-non-adoption.md` §3.25 の Slot 行が保留継続と
//! している）。chakra-ui の `LinkBox` が CSS の子孫セレクタで行う「内側
//! リンクの前面化」も headless 層は CSS を持たないため利用者側 CSS の
//! 責務とする。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`] へ薄く委譲するのみ。styled 層
//! （`fandhe-frontend-pre-styled-ui`）は `data-scope="link-overlay"` の
//! `root`/`overlay` セレクタを前提にスタイルを当てる。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（`href`/attrs/children は
//! [`fandhe_frontend_core::render`] の既定エスケープを必ず経由し、`href` の
//! 危険 URL スキームは core の許可リスト方式が属性ごと拒否する）。
//! [`overlay`] が固定付与する `href` は呼び出し側 `attrs` からの同名
//! なりすまし（`fandhe_frontend_core::el` は同名属性を重複除去しないため
//! 単に連結すると `href` が二重出力されうる）を [`drop_reserved`] で
//! 除去する（`crate::breadcrumb::link` と同型、イシュー #1650）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - `root` 内に複数の対話要素（別リンク・ボタン等）を混在させる場合、
//!   `overlay` より前面に出す z-index 調整は呼び出し側（styled 層・
//!   アプリ側）の責務とする（headless 層は z-index の既定値を持たない）。

use crate::anatomy::{anatomy, Anatomy};
use fandhe_frontend_core::Node;

/// LinkOverlay の anatomy（`data-scope="link-overlay"`）。
const ANATOMY: Anatomy = anatomy("link-overlay");

/// [`overlay`] が固定付与する予約キー。呼び出し側 `attrs` からのなりすまし
/// を [`drop_reserved`] で除去する（イシュー #1650、`crate::breadcrumb` と
/// 同型）。
const OVERLAY_RESERVED: &[&str] = &["href"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは同名属性の
/// 重複出力・固定属性のなりすましを許してしまう（`crate::breadcrumb::
/// drop_reserved` と同型、イシュー #1650）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `root` パーツ（`div`）。位置決めコンテキスト（styled 層が
/// `position: relative` を当てる前提）。
#[must_use]
pub fn root(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// `overlay` パーツ（`a`）。[`root`] 全面へ拡張されるリンク（styled 層が
/// `position: absolute; inset: 0;` を当てる前提。本モジュール冒頭の rustdoc
/// 「全面拡張の実装方針」参照）。
#[must_use]
pub fn overlay<'a>(href: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, OVERLAY_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![("href", href)];
    merged.extend(attrs);
    ANATOMY.part("overlay", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_div_with_scope_and_part() {
        let html = render(&root(vec![], vec![]));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn overlay_outputs_anchor_with_href() {
        let html = render(&overlay("/docs/next", vec![], vec![text("Next")]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="overlay""#));
        assert!(html.contains(r#"href="/docs/next""#));
        assert!(html.contains(">Next<"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));

        let html = render(&overlay(
            "/docs",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="overlay""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn overlay_dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&overlay(url, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    #[test]
    fn overlay_href_attribute_breakout_payload_is_escaped() {
        let html = render(&overlay("/docs\" onmouseover=\"alert(1)", vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn overlay_children_script_payload_is_escaped() {
        let html = render(&overlay(
            "/docs",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// [`overlay`] の呼び出し側 `attrs` に `href` を混入させても、固定
    /// 付与された正規 `href` のみが出力されることを固定する（大文字小文字
    /// 無視、イシュー #1650）。`fandhe_frontend_core::el` は同名属性を
    /// 重複除去しないため、この保証が [`drop_reserved`] 側の責務である。
    #[test]
    fn overlay_drops_caller_supplied_href_case_insensitively() {
        let html = render(&overlay(
            "https://example.com/docs",
            vec![
                ("HREF", "javascript:alert(1)"),
                ("href", "https://example.com/evil"),
            ],
            vec![],
        ));
        assert_eq!(
            html.matches("href=").count(),
            1,
            "href は正規値の 1 回のみ出力される: {html}"
        );
        assert!(html.contains(r#"href="https://example.com/docs""#));
        assert!(!html.contains("evil"));
        assert!(!html.contains("javascript:"));
    }
}
