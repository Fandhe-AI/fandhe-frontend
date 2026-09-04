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
//! 座標 + 背景 + `z-index` + 文字スタイルを上書きして視覚的に復元し、
//! [`crate::recipe::focus_ring_declarations`]（`outline` + `outline-offset`
//! の canonical 形、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//! §3）でフォーカスリングを描く。`box-shadow` によるリング表現は
//! `forced-colors: active` で消えるため使わない。
//!
//! `content`（スキップ先ターゲット）は実コンテンツを持たず、`tabindex="-1"`
//! でプログラム的フォーカスを受け取るだけの要素のため、既定のフォーカス
//! リングを消す `outline: none` のみを base として登録する（同設計文書 §3
//! 表の skip-nav 例外。プログラム的フォーカスのみを受け取る不可視要素に
//! リングを描く意味がないため）。
//!
//! # 参照サイト（chakra-ui SkipNav）との差分と意図的非採用（イシュー #1586）
//!
//! 107 部品スタイル調整（親 #1420）の 1 件として、以下の是正・非採用判断を
//! 行った:
//!
//! - **是正**: 未定義トークン `--fandhe-space-md`/`--fandhe-space-sm`
//!   （[`crate::theme`] の `DEFAULT_SPACES` に存在せず、フォールバック値
//!   でしか動作していなかった）を実在する `--fandhe-space-6`/
//!   `--fandhe-space-4` へ差し替え、`z-index` の生リテラル `1200`（popover
//!   段と同値で dialog/drawer に隠れていた）を正式トークン
//!   `--fandhe-z-index-skip-nav`（[`crate::theme`] の `DEFAULT_Z_INDICES`
//!   で `1500` と定義済み）へ差し替えた。フォーカスリングは `box-shadow`
//!   から前述の canonical `outline` 形へ、文字は無指定（ブラウザ既定の
//!   下線付き `<a>`）から `font-size: sm` + `font-weight: semibold` +
//!   下線なしへ、面には `--fandhe-shadow-md` の elevation を追加した。
//! - **hover を追加**: `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   の共通ビジュアル言語に従い、`<a>` を担う `link` slot へ
//!   `hover_surface_declarations()` を追加した。参照サイトに hover 実装は
//!   ないが、本リポジトリの共通言語（インタラクティブ slot は hover 対象）
//!   を優先する。`Hover` は `@media (hover: hover)` 配下へ集約されるため
//!   タッチ端末では発火せず、非表示時（clip 済み 1px 要素）は物理的に
//!   hover し得ないため無害。
//! - **size / variant / ColorPalette 軸を持たない**: 参照サイトも 1 種類の
//!   表示のみで、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §4 の保有判定 (d)「構造・可視性のみの Utilities は size を持たない」
//!   に該当する。palette 軸もないため [`crate::recipe::FocusRingColor::Token`]
//!   を使う。
//! - **transition は不採用**: 表示切替が `clip`/`position` の可視性トグル
//!   であり、`background` だけを遷移させると過渡的に未塗装の背景の上へ
//!   文字が乗り本文と重なって可読性を損なう。参照サイトにも transition は
//!   ない。
//! - **角丸は `--fandhe-radius-md` を維持**: chakra 参照の `l2`（0.25rem 相当）
//!   との差 0.125rem は、本テーマの button/card 系が `md` に収斂している
//!   整合を優先し意図的に合わせない。
//! - **背景トークンは `--fandhe-color-bg` を維持**: chakra 参照の
//!   `bg.panel` 相当の専用パネル背景トークンは本テーマに未定義であり、
//!   `bg`（light/dark 双方でコントラスト契約済み）で代替する。専用トークン
//!   新設の是非は別途の評価事項とする。
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
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, FocusRingColor,
    FocusRingOffset, SlotRecipe, StateCondition,
};
use crate::visually_hidden::clip_declarations;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::skip_nav::DEFAULT_ID;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/skip_nav.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["link", "content"];

/// この styled SkipNav の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    // `link` の非表示時 base 宣言（clip 手法）に加え、hover 時背景の間接
    // 参照用 custom property をここへ同居させる（`.base` を 2 回呼ぶと
    // 規則ブロックが分裂するため、1 つの Vec にまとめて渡す）。
    let mut link_base = clip_declarations();
    link_base.push(hover_bg_muted());

    // フォーカス時に視覚的に復元する宣言列（座標・面・文字スタイル）。
    // 末尾にフォーカスリングの canonical 宣言（`outline` +
    // `outline-offset`）を追加する。
    let mut link_focus_visible = vec![
        decl("position", "fixed"),
        decl("top", "var(--fandhe-space-6, 1.5rem)"),
        decl("left", "var(--fandhe-space-6, 1.5rem)"),
        decl("width", "auto"),
        decl("height", "auto"),
        decl("padding", "var(--fandhe-space-4, 1rem)"),
        decl("margin", "0"),
        decl("overflow", "visible"),
        decl("clip", "auto"),
        decl("white-space", "normal"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
        decl("line-height", "var(--fandhe-font-line-height-normal)"),
        decl("text-decoration", "none"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("box-shadow", "var(--fandhe-shadow-md)"),
        decl("z-index", "var(--fandhe-z-index-skip-nav, 1500)"),
    ];
    link_focus_visible.extend(focus_ring_declarations(
        FocusRingColor::Token,
        FocusRingOffset::Outside,
    ));

    SlotRecipe::new("skip-nav", SLOTS)
        .base("link", link_base)
        .state("link", StateCondition::FocusVisible, link_focus_visible)
        .state("link", StateCondition::Hover, hover_surface_declarations())
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
        // フォーカスリングは canonical `outline` 形（イシュー #1586）。
        assert!(a.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(!a.contains("box-shadow: 0 0 0 2px"));
        // z-index は正式トークン化済み（`crate::theme::DEFAULT_Z_INDICES`）。
        assert!(a.contains("z-index: var(--fandhe-z-index-skip-nav, 1500);"));
        // hover は `@media (hover: hover)` 配下へ集約される（共通言語）。
        assert!(a.contains("@media (hover: hover)"));
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
