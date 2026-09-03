//! styled LinkOverlay（headless ラッパー、イシュー #756、#716 追加候補の
//! 消化）。
//!
//! `fandhe_frontend_headless_ui::link_overlay`（イシュー #756）の Root /
//! Overlay 2 anatomy パーツを薄く再利用し、[`stylesheet`] で「カード全面
//! クリック化」の既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::breadcrumb`]/[`crate::avatar`] の rustdoc と同じ方針に従う。
//!
//! # 全面拡張の CSS 実装
//!
//! `::before` 疑似要素を使わず `overlay` 自身を展開する方式を採る理由は
//! headless 層（`crates/headless-ui/src/link_overlay.rs`）の rustdoc
//! 「全面拡張の実装方針」を参照。[`recipe`] は `root` に `position: relative`、
//! `overlay` に `position: absolute; inset: 0;` を登録する。呼び出し側は
//! `overlay` 以外の子ノード（見出し・画像等）で `root` の高さを確立する
//! 契約を維持する。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（headless 層 → [`fandhe_frontend_core::render`]
//! の既定エスケープを必ず経由し、`raw_html()` の新規使用なし、`href` の URL
//! スキーム検証は headless 層が担う）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `root` 内に `overlay` 以外の対話要素を配置する場合の z-index 調整は
//!   呼び出し側の責務（headless 層 rustdoc 参照）。
//!
//! # 参照サイト比較（イシュー #1580、7 軸チェック）
//!
//! 参照軸は chakra-ui のみ（`docs/design/component-coverage-map.md` の該当行は
//! ark-ui / Radix が「—」であり、本モジュールが薄く再利用する headless 層
//! （`crates/headless-ui/src/link_overlay.rs`）の rustdoc も「ark-ui には
//! 対応する headless 実体がない」と明記している）。chakra の `LinkBox` /
//! `LinkOverlay` recipe は構造のみ（`position: relative` と `::before` に
//! よる全面拡張、`cursor: inherit`）で、参照スクショに見える枠線・角丸・
//! 余白付きカードはラッパー側（`Box` の `borderWidth`/`p`/`rounded` や
//! `Card`）の props に由来し、`LinkOverlay` 自身の意匠ではない。
//!
//! - **サイズ / バリアント / 色**: 追加しない。chakra `LinkOverlay` に
//!   size / variant / colorPalette 軸が存在しないため。`root` にも色・
//!   カード意匠を付けない（付けると `link_overlay::root` と
//!   `card::root` を合成する呼び出し側で二重枠線になり、`root` は
//!   位置決めコンテキストのみという既存契約が壊れる）。
//! - **`data-*` 状態**: 変更なし（headless 層が状態属性を出さない）。
//! - **ダーク**: 個別対応不要。フォーカスリングは
//!   `--fandhe-color-focus-ring` 系トークン経由で `Theme::to_css` の
//!   一元機構に自動追従する。
//! - **フォーカス（是正）**: `overlay` に `StateCondition::FocusVisible`
//!   の状態規則として [`crate::recipe::focus_ring_declarations`] を追加した。
//!   `FocusRingColor::Token`: 本部品は palette 軸を持たないため
//!   （`crate::nav_list` と同判断）。`FocusRingOffset::Outside`: `root` に
//!   `overflow: hidden` を持つ祖先 slot がなく、`overlay` は
//!   `inset: 0` で `root` 全面へ展開されるためリングがカード全体を囲む
//!   （chakra で `LinkOverlay` にキーボードフォーカスした際の見え方と同等）。
//! - **余白・角丸・影（是正）**: `overlay` base に
//!   `border-radius: inherit` を追加した（`crate::avatar` の `image` が
//!   同じ理由で持つ宣言と同型）。呼び出し側が `root` に角丸（`card` 等）
//!   を与えた場合、フォーカスリングの `outline` がその角丸へ追従する。
//!   **追補（Bugbot 指摘、PR #1853）**: CSS の `inherit` は宣言先要素の
//!   直接の親の計算値を参照するため、`root` 自身が角丸を持たない
//!   （既定 0）ままでは、角丸な祖先（`card::root` 等）にラップされていても
//!   `root` の計算値が 0 のまま `overlay` へ継承されフォーカスリングが
//!   角丸に追従しない。`root` base にも `border-radius: inherit` を追加し、
//!   `root` の直接の親要素が持つ角丸を `root` → `overlay` の 2 段で
//!   連鎖させた（`inherit` は 1 段先の直接の親要素の計算値しか参照
//!   できないため、`root` と角丸要素の間にさらに角丸を持たない要素が
//!   挟まる構成には届かない）。
//!   padding・shadow は追加しない（本部品は構造のみを担う）。
//! - **hover**: 意図的に付けない。`overlay` は
//!   `position: absolute; z-index: 0` の位置指定要素であり、
//!   `hover_surface_declarations()`（`background: ...`）を当てると
//!   カード本文（見出し・説明文）より上に塗り潰しが描かれて読めなくなる。
//!   chakra `LinkBox`/`LinkOverlay` 自体も hover 意匠を持たない（hover は
//!   ラッパー側の責務）。`root` は
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 の
//!   「hover はインタラクティブ slot のみ」に該当しない。
//! - **disabled**: 該当なし（headless 層がリンクに `data-disabled` を
//!   出さない）。
//! - **transition**: 付けない（アニメーション対象となる hover 背景等の
//!   プロパティを持たないため）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::link_overlay::overlay;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/link_overlay.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "overlay"];

/// この styled LinkOverlay の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("link-overlay", SLOTS)
        .base(
            "root",
            vec![
                decl("position", "relative"),
                // イシュー #1580 Bugbot 指摘: `overlay` の
                // `border-radius: inherit` は CSS の継承規則上 `root` の
                // 計算値を参照するため、`root` 自身が角丸を持たない
                // （既定 0）ままでは、`root` の直接の親要素が角丸
                // （`card::root` 等）を持っていてもフォーカスリングの
                // `outline` が角丸へ追従しない。`root` にも `inherit` を
                // 連鎖させ、直接の親要素が持つ角丸を `overlay` まで
                // 伝播させる（`inherit` は 1 段先までしか参照できないため、
                // さらに祖先を挟む構成には届かない）。
                decl("border-radius", "inherit"),
            ],
        )
        .base(
            "overlay",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("z-index", "0"),
                // イシュー #1580: 呼び出し側が `root` に角丸（`card` 等）を
                // 与えた場合、フォーカスリングの `outline` がその角丸へ
                // 追従するようにする（`crate::avatar` の `image` と同型）。
                decl("border-radius", "inherit"),
            ],
        )
        // イシュー #1580: キーボード操作時のみのフォーカスリング。
        // `Token`: link-overlay は palette 軸を持たない部品。`Outside`:
        // `overlay` の祖先（`root`）に `overflow: hidden` を持つ slot が
        // ないため。
        .state(
            "overlay",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled LinkOverlay が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツ（位置決めコンテキスト）を組み立てる。呼び出し側
/// `attrs` の `class` は [`drop_class_attr`] で除去する（本部品は `root` に
/// variant クラスを持たないが、他 styled 部品との一貫性のため同様に扱う）。
/// 実体は [`fandhe_frontend_headless_ui::link_overlay::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::link_overlay;
///
/// let node = link_overlay::root(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="link-overlay" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::link_overlay::root(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_overlay_outputs_expected_tag() {
        let html = render(&overlay("/docs/next", vec![], vec![text("Next")]));
        assert!(html.contains("<a"));
        assert!(html.contains(r#"href="/docs/next""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_positioning_declarations() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("position: relative"));
        assert!(a.contains("position: absolute"));
        assert!(a.contains("inset: 0"));
        assert!(a.contains("border-radius: inherit"));
        assert!(a.contains(":focus-visible"));
        assert!(a.contains("outline"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

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
}
