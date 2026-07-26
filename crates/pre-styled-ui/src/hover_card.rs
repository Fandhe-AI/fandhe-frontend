//! styled HoverCard（headless ラッパー、イシュー #759、親トラッキング #520/#726）。
//!
//! `fandhe_frontend_headless_ui::hover_card`（イシュー #759）の Root /
//! Trigger / Positioner / Content / Arrow / ArrowTip 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::hover_card::HoverCard`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::tooltip`]/[`crate::popover`] の rustdoc と
//! 同じ方針に従う（構造上最も近い先行例は [`crate::tooltip`]）。
//!
//! # data-state とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! [`crate::tooltip`] と同じ判断）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger` はフォーカス可能なリンク要素であり、キーボード操作時のみの
//! フォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::tooltip`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置
//!
//! headless 側 `root`（`crates/headless-ui/src/hover_card.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与する
//! （[`crate::popover`]/[`crate::tooltip`] と同じ判断）。`positioner` は
//! `position: absolute; top: 100%; left: 0; z-index: 10` の dropdown 型
//! オーバーレイとする（[`crate::popover`] と同じ tier。tooltip の
//! `z-index: 1100` より前面性が低い補助オーバーレイとして扱う）。
//! `positioner` は base 規則で `display` を宣言しないため、closed 時に
//! headless 層が付与する `hidden` 存在属性は UA 既定
//! `[hidden] { display: none }` がそのまま機能する（[`crate::tooltip`]/
//! [`crate::popover`] と同じ構造的な回避、dialog で発生した PR #575 Bugbot
//! 指摘（High）と同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない
//!
//! [`crate::tooltip`] と同じ判断: hover card の `content` はプレビュー内容へ
//! 幅が追随すべきであり、`sameWidth` 相当は用途として不適切なため意図的に
//! `--fandhe-reference-width` を消費しない（既存の CSS 変数規約自体には
//! 反しない選択であることをここに明記する）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは他の styled 部品と同じく
//!   スコープ外とする。
//! - `openDelay`/`closeDelay`/`interactive` は headless 層のドキュメント
//!   （`crates/headless-ui/src/hover_card.rs`）で既にスコープ外と明記済みの
//!   クライアントサイド実行時挙動であり、本モジュールもそれを継承する。
//! - `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!   [`crate::tooltip`]/[`crate::popover`] と同じ理由で本イシューの対象外。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。オーバーレイ配置系の
// [`crate::popover`]/[`crate::tooltip`] と同じ判断で variant 軸を提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::hover_card::*;
// `root`/`trigger` 等の `state` 引数・`HoverCard::new`・`HoverCard` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（[`crate::tooltip`]/[`crate::popover`] と同じ判断、
// イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `hover-card` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/hover_card.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
];

/// この styled HoverCard の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("hover-card", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("color", "var(--fandhe-color-accent)"),
                decl("cursor", "pointer"),
                decl("text-decoration", "underline"),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("max-width", "20rem"),
            ],
        )
        // `content` の開閉状態に応じた見た目の切り替え（[`crate::tooltip`] と
        // 同じ判断）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled HoverCard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::hover_card::HoverCardDelays;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="hover-card"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="hover-card"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"hover-card\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="hover-card"][data-part="content"][data-state="closed"]"#)
        );
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="hover-card"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn content_does_not_consume_reference_width_css_var() {
        // hover card の content はプレビュー内容へ幅が追随すべきであり、
        // sameWidth 相当は不適切なため意図的に --fandhe-reference-width を
        // 消費しないことを固定する（モジュール doc 参照）。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-reference-width"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="hover-card""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_hover_card_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut hc = HoverCard::default();
        assert_eq!(hc.state(), OpenState::Closed);

        let ssr_html = render(&hc.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut hc, "open", ""));
        let hydrate_html = render(&render_for_hydration(&hc));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
