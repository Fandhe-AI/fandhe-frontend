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
//! 座標 + `padding` + `z-index` + フォーカスリングを上書きして視覚的に
//! 復元する。
//!
//! `content`（スキップ先ターゲット）は実コンテンツを持たず、`tabindex="-1"`
//! でプログラム的フォーカスを受け取るだけの要素のため、既定のフォーカス
//! リングを消す `outline: none` のみを base として登録する。
//!
//! # 参考サイトとの差分（イシュー #1586）
//!
//! ark-ui の SkipNav ページは調査時点で 404（該当部品が存在しない）、
//! Radix Primitives / Radix Themes にも同等部品はないため、参考基準は
//! chakra-ui の `skipNavLinkRecipe` 単独とする。
//!
//! - **是正**: 余白は `--fandhe-space-md`/`-sm`（未定義のため常にリテラル
//!   フォールバックへ落ちていた）から `--fandhe-space-6`/`-2-5`
//!   （chakra `top`/`insetStart: 6`・`padding: 2.5` 相当）へスケール載せ。
//!   フォーカス表現は `box-shadow` リングから
//!   [`crate::recipe::focus_ring_declarations`]（`outline` +
//!   `outline-offset`、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §3 の canonical 化）へ置換。`z-index` はリテラル `1200` から
//!   [`crate::theme`] の `skip-nav` トークン
//!   （`var(--fandhe-z-index-skip-nav, 1200)`）へ参照化。タイポグラフィ
//!   （`font-size-sm`/`font-weight-semibold`/`line-height-tight`）を
//!   chakra `textStyle: sm`/`fontWeight: semibold` に対応させて新規追加。
//!   `left` は論理プロパティ `inset-inline-start`（chakra `insetStart`
//!   相当、RTL 対応）へ変更。
//! - **意図的に合わせない / N/A**: (1) `border-radius` は chakra の `l2`
//!   （`sm` 相当）ではなく本リポジトリの他部品と同じ `--fandhe-radius-md`
//!   を維持し部品間統一を優先する。(2) hover
//!   （[`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::hover_surface_declarations`]）は chakra
//!   `SkipNavLink` にはないが、本リポジトリのインタラクティブ `<a>` slot
//!   共通規約（`docs/design/pre-styled-ui-interaction-visual-language.md`）
//!   に従い付与する。(3) size / variant / `ColorPalette` 軸は chakra
//!   `SkipNavLink` が props を一切持たないため設けない。(4) disabled は
//!   headless 層（`crates/headless-ui/src/skip_nav.rs`）が
//!   `data-disabled` を出力しないため N/A。(5) 影（box-shadow）は chakra
//!   にないため追加しない（フォーカス時の面の分離は outline のみで担う）。
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
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
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
    // `link` base: clip 手法（共有ヘルパ、`crate::visually_hidden` 契約のため
    // 改変しない）に、キーボードフォーカスで復元されたときの見た目
    // （タイポグラフィ・色・角丸・hover・transition）を連結する。
    // `SlotRecipe::base` は同一 slot への複数回呼び出しがそれぞれ独立した
    // ルールブロックを生成する（`recipe.rs` の実装契約）ため、1 本の `Vec`
    // にまとめて 1 回だけ登録する。
    let mut link_base = clip_declarations();
    link_base.extend([
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
        decl("line-height", "var(--fandhe-font-line-height-tight)"),
        decl("text-decoration", "none"),
        decl("user-select", "none"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        hover_bg_muted(),
    ]);
    link_base.extend(transition_declarations("background", MotionDuration::Fast));

    // `FocusVisible` state: `position: fixed` へ切り替えて座標・寸法・
    // z-index を上書きし、clip 手法による隠蔽を打ち消して視覚的に復元する。
    let mut link_focus = vec![
        decl("position", "fixed"),
        decl("top", "var(--fandhe-space-6, 1.5rem)"),
        decl("inset-inline-start", "var(--fandhe-space-6, 1.5rem)"),
        decl("width", "auto"),
        decl("height", "auto"),
        decl("padding", "var(--fandhe-space-2-5, 0.625rem)"),
        decl("margin", "0"),
        decl("overflow", "visible"),
        decl("clip", "auto"),
        decl("white-space", "normal"),
        decl("z-index", "var(--fandhe-z-index-skip-nav, 1200)"),
    ];
    link_focus.extend(focus_ring_declarations(
        FocusRingColor::Token,
        FocusRingOffset::Outside,
    ));

    SlotRecipe::new("skip-nav", SLOTS)
        .base("link", link_base)
        .state("link", StateCondition::FocusVisible, link_focus)
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
