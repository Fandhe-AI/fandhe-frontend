//! VisuallyHidden（イシュー #776、親 #766 Phase 6、`docs/design/component-coverage-map.md`
//! visually-hidden 行「実装対象 #776」の消化）。
//!
//! chakra-ui/ark-ui の VisuallyHidden ユーティリティに倣い、視覚的には隠す
//! （clip 手法、styled 層 `fandhe-frontend-pre-styled-ui::visually_hidden` が
//! CSS 責務を持つ）が支援技術（スクリーンリーダー）には読ませ続けるテキスト
//! コンテナを提供する。[`mod@crate::field`]/[`mod@crate::link`] と同型の、
//! 時間変化する内部状態を持たない純粋関数のみで構成する（[`crate::state`]
//! の状態機械は適用しない）。
//!
//! # `aria-hidden` を付けない不変条件（受け入れ条件の核）
//!
//! 本モジュールの [`root`] は `aria-hidden` を一切出力しない。
//! [`crate::checkbox::control`]/`fandhe-frontend-pre-styled-ui::skeleton` の
//! ような装飾的要素は `aria-hidden="true"` を固定付与して支援技術から隠すが、
//! VisuallyHidden は逆に「視覚的には隠すが支援技術には読ませる」ことこそが
//! 存在意義であり、`aria-hidden` を付けると DOM に残したままの意味が消える。
//! そのため他の headless コンポーネントのような契約属性の fail-closed 除去は
//! 行わない（そもそも本コンポーネントが決定する契約属性自体を持たない。
//! `data-scope`/`data-part` の偽装除去は [`crate::anatomy::Anatomy::part`] が
//! 既存どおり担う）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`] へ薄く委譲するのみで、独自の出力経路・
//!   独自のエスケープ処理は持たない。
//! - styled 層（`fandhe-frontend-pre-styled-ui::visually_hidden`）は本モジュール
//!   が出力する `data-scope="visually-hidden"`/`data-part="root"` セレクタを
//!   前提に clip 手法の CSS を当てる。
//!
//! # セキュリティ不変条件
//!
//! - 子ノード・呼び出し側 `attrs` はすべて [`fandhe_frontend_core::el`] の
//!   属性値・子ノードとして渡り、[`fandhe_frontend_core::render`] の既定
//!   エスケープ（REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用
//!   せず、HTML 文字列を直接組み立てない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - crates.io への公開・`examples/headless-pre-styled-ui` の追随は公開
//!   イシュー側のスコープ。
//! - chakra の `asChild`/`as` 合成 API（本フレームワークはノード木 API のため
//!   対象外）。

use crate::anatomy::{anatomy, Anatomy};
use fandhe_frontend_core::Node;

/// VisuallyHidden の anatomy（`data-scope="visually-hidden"`）。
const ANATOMY: Anatomy = anatomy("visually-hidden");

/// `root` パーツ（`span`）。唯一の anatomy パーツ。
///
/// 子ノードのテキストは視覚的には隠れるがスクリーンリーダーには読み上げ
/// られる（styled 層が clip 手法の CSS を当てる前提。本関数自体はマークアップ
/// の組み立てのみを担い、スタイリングは持たない）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "span", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(vec![], vec![text("補足テキスト")]));
        assert!(html.starts_with("<span"));
        assert!(html.contains(r#"data-scope="visually-hidden""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(">補足テキスト<"));
    }

    #[test]
    fn root_does_not_emit_aria_hidden_by_default() {
        let html = render(&root(vec![], vec![]));
        assert!(!html.contains("aria-hidden"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="visually-hidden""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn attrs_value_breakout_payload_is_escaped() {
        let html = render(&root(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }
}
