//! styled SkipNav（headless ラッパー、イシュー #776、親 #766 Phase 6）。
//!
//! `fandhe_frontend_headless_ui::skip_nav`（イシュー #776）の `link`/`content`
//! 2 anatomy パーツを薄く再利用し、[`stylesheet`] で「キーボードフォーカス時
//! のみ視覚的に現れる」既定 CSS を追加提供する。薄い委譲の根拠・スコープ外
//! 事項は [`crate::separator`]/[`crate::skeleton`] の rustdoc と同じ方針に
//! 従う（headless 状態機械を要しない静的部品）。
//!
//! # focus 時表示の表現（純 CSS、hydration 配線なし）
//!
//! `fandhe-frontend-docs-site` は JS/hydration を持たないため、本部品の
//! 「focus していないときは視覚的に隠し、キーボードフォーカス時のみ表示する」
//! 挙動は [`crate::recipe::StateCondition::FocusVisible`]（`:focus-visible`
//! 疑似クラス）**のみ**で表現する。他の styled 部品が併用する
//! `data-focus-visible` 存在属性 + クライアントランタイムの付け外し方式
//! （イシュー #709、`crate::switch`/`crate::radio_group` 参照）は、本部品の
//! `link`（`<a>`）自身が実フォーカスを受け取る通常のフォーカス可能要素で
//! あるため必要ない（hidden-input パターンには該当しない）。
//!
//! `link` の base 宣言は [`crate::visually_hidden::clip_declarations`]
//! （clip 手法）をそのまま再利用し（[`crate::visually_hidden`] モジュール
//! doc 参照）、`StateCondition::FocusVisible` の宣言で `position: fixed` +
//! 座標 + 背景 + `z-index` を上書きして視覚的に復元する。
//!
//! `content`（スキップ先ターゲット）は実コンテンツを持たず、`tabindex="-1"`
//! でプログラム的フォーカスを受け取るだけの要素のため、既定のフォーカス
//! リングを消す `outline: none` のみを base として登録する。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の構成（`#<id>` 固定・スキーム
//!   注入経路なし）は headless 層（`crates/headless-ui/src/skip_nav.rs`
//!   rustdoc）が担う。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから合成する
//!   （`class` 属性は常に単一）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - 複数スキップリンク運用ガイド等のドキュメントサイト向け利用ガイド拡充。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
use crate::visually_hidden::clip_declarations;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::skip_nav::DEFAULT_ID;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/skip_nav.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["link", "content"];

/// この styled SkipNav の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("skip-nav", SLOTS)
        .base("link", clip_declarations())
        .state(
            "link",
            StateCondition::FocusVisible,
            vec![
                decl("position", "fixed"),
                decl("top", "var(--fandhe-space-md, 1rem)"),
                decl("left", "var(--fandhe-space-md, 1rem)"),
                decl("width", "auto"),
                decl("height", "auto"),
                decl(
                    "padding",
                    "var(--fandhe-space-sm, 0.5rem) var(--fandhe-space-md, 1rem)",
                ),
                decl("margin", "0"),
                decl("overflow", "visible"),
                decl("clip", "auto"),
                decl("white-space", "normal"),
                // `overflow-wrap` は base（clip_declarations 由来）で
                // `normal` を明示しているが、その値自体が CSS 初期値と
                // 同一のため、ここで打ち消す宣言は不要（イシュー #1587）。
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "box-shadow",
                    "0 0 0 2px var(--fandhe-color-accent, var(--fandhe-color-fg))",
                ),
                decl("z-index", "1200"),
            ],
        )
        .base("content", vec![decl("outline", "none")])
}

/// この styled SkipNav が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `link` パーツを組み立てる。呼び出し側 `attrs` の `class` は
/// [`drop_class_attr`] で除去する。実体は
/// [`fandhe_frontend_headless_ui::skip_nav::link`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::skip_nav::{link, DEFAULT_ID};
///
/// let node = link(DEFAULT_ID, vec![], vec![text("Skip to content")]);
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="skip-nav""#));
/// ```
#[must_use]
pub fn link<'a>(id: &str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::skip_nav::link(id, drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。呼び出し側 `attrs` の `class` は
/// [`drop_class_attr`] で除去する。実体は
/// [`fandhe_frontend_headless_ui::skip_nav::content`] へ委譲する。
#[must_use]
pub fn content<'a>(id: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::skip_nav::content(id, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn link_outputs_scope_part_and_href() {
        let html = render(&link(DEFAULT_ID, vec![], vec![text("Skip to content")]));
        assert!(html.contains(r#"data-scope="skip-nav""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(html.contains(">Skip to content<"));
    }

    #[test]
    fn content_outputs_scope_part_id_and_tabindex() {
        let html = render(&content(DEFAULT_ID, vec![], vec![]));
        assert!(html.contains(r#"data-scope="skip-nav""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&link(
            DEFAULT_ID,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="skip-nav""#));
        assert!(html.contains(r#"data-part="link""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_class_is_dropped() {
        let html = render(&link(
            DEFAULT_ID,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_declares_focus_visible_rule() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="skip-nav"][data-part="link"]:focus-visible"#));
        assert!(a.contains("position: fixed;"));
        assert!(a.contains("clip: rect(0, 0, 0, 0);"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn id_attribute_breakout_payload_is_escaped() {
        let html = render(&link("x\" onmouseover=\"alert(1)", vec![], vec![]));
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
