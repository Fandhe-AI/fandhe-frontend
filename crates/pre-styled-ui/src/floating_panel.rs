//! styled FloatingPanel（headless ラッパー、イシュー #827、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::floating_panel`（イシュー #827）の Root /
//! Trigger / Positioner / Content / Header / Title / Control / StageTrigger /
//! CloseTrigger / Body 10 anatomy パーツと
//! [`fandhe_frontend_headless_ui::floating_panel::FloatingPanel`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::popover`]/[`crate::dialog`] の
//! rustdoc と同じ方針に従う。
//!
//! # data-state/data-stage とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`crate::popover`] と同じ方針で登録する。加えて `body` の
//! `data-stage="minimized"` を折り畳み（`display: none`。ヘッダのみ表示）、
//! `positioner` の `data-stage="maximized"` をビューポート全面表示
//! （`transform: none; inset: 0`）へ切り替える（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger`/`stage-trigger`/`close-trigger` はフォーカス可能なボタン要素で
//! あり、キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner の配置と z-index tier
//!
//! `positioner` はドラッグ移動によりビューポート絶対座標へ置かれるため
//! `position: fixed; left: 0; top: 0` を基点とし、headless 側が出力する
//! `--fandhe-x`/`--fandhe-y`（[`fandhe_frontend_headless_ui::floating_panel::FloatingPanel::position_style`]）
//! を `transform: translate3d(...)` で反映する。z-index は
//! [`crate::dialog`] のモーダル層（1000/1001）未満・[`crate::menu`]/
//! [`crate::popover`] の dropdown 層（10）超の専用 tier（`900`）とする
//! （FloatingPanel は非モーダルだがトリガー付近に留まる dropdown より
//! 手前に来る想定のため、独自の中間層を割り当てる）。`positioner` は base
//! 規則で `display` を宣言しないため、closed 時に headless 層が付与する
//! `hidden` 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能
//! する（[`crate::popover`] と同じ構造判断、dialog PR #575 の不具合を構造的
//! に回避する）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - ドラッグ移動・リサイズの実 DOM 配線: headless 層のドキュメント
//!   （`crates/headless-ui/src/floating_panel.rs`）で既にスコープ外と明記
//!   済みであり、本モジュールもそれを継承する（`--fandhe-x`/`--fandhe-y`
//!   の消費のみを提供する）。
//! - variant（size 等）ごとのクラス切り替えは他 headless ラッパーと同じく
//!   スコープ外とする。
//! - フォーカストラップ・Escape キー閉鎖・`lazyMount`・topmost 管理は
//!   headless 層のスコープ外を継承する。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::floating_panel::*;
// `root`/`trigger` 等の `state`/`stage` 引数・`FloatingPanel::new`・
// `FloatingPanel` の `Component::Action`（dispatch 対象）はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（`crate::popover` と同じ契約、イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `floating_panel` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/floating_panel.rs` の `ANATOMY.part(...)` 呼び
/// 出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイル
/// を合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "header",
    "title",
    "control",
    "stage-trigger",
    "close-trigger",
    "body",
];

/// この styled FloatingPanel の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("floating-panel", SLOTS)
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("left", "0"),
                decl("top", "0"),
                decl("z-index", "900"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 24px), var(--fandhe-y, 24px), 0)",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("min-width", "16rem"),
            ],
        )
        .base(
            "header",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                decl("cursor", "move"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "stage-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "none"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "none"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base("body", vec![decl("padding", "var(--fandhe-space-4)")])
        // `content` の開閉状態に応じた見た目の切り替え（`crate::popover` と同じ判断）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // minimized: body（本文）を折り畳み、ヘッダのみ表示する。
        .state(
            "body",
            StateCondition::AttrEq("data-stage", "minimized"),
            vec![decl("display", "none")],
        )
        // maximized: ビューポート全面表示へ切り替える（ドラッグ座標の transform を中和）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-stage", "maximized"),
            vec![decl("transform", "none"), decl("inset", "0")],
        )
        // キーボード操作時のみのフォーカスリング（[`crate::dialog`] と同じ判断）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "stage-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled FloatingPanel が生成する静的 CSS 全量を返す（決定的。
/// [`crate::popover::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="floating-panel"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="floating-panel"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_fixed_positioned_with_transform_translate() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="floating-panel"][data-part="positioner"]"#));
        assert!(css.contains("position: fixed;"));
        assert!(css.contains("translate3d(var(--fandhe-x, 24px), var(--fandhe-y, 24px), 0)"));
    }

    #[test]
    fn positioner_z_index_is_between_dropdown_and_modal_tiers() {
        let css = stylesheet();
        assert!(css.contains("z-index: 900;"));
    }

    #[test]
    fn stylesheet_links_data_state_to_content_visibility() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="content"][data-state="closed"]"#
        ));
        assert!(css.contains("visibility: hidden;"));
    }

    #[test]
    fn stylesheet_links_data_stage_to_body_and_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="body"][data-stage="minimized"]"#
        ));
        assert!(css.contains("display: none;"));
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="positioner"][data-stage="maximized"]"#
        ));
        assert!(css.contains("transform: none;"));
        assert!(css.contains("inset: 0;"));
    }

    #[test]
    fn trigger_stage_trigger_and_close_trigger_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="floating-panel"][data-part="trigger"]:focus-visible {"#)
        );
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="stage-trigger"]:focus-visible {"#
        ));
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="close-trigger"]:focus-visible {"#
        ));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, Stage::Default, vec![], vec![]));
        assert!(html.contains(r#"data-scope="floating-panel""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_floating_panel_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut p = FloatingPanel::default();
        assert_eq!(p.state(), OpenState::Closed);

        let ssr_html = render(&p.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut p, "open", ""));
        assert!(dispatch(&mut p, "maximize", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));
        assert!(hydrate_html.contains(r#"data-hydrate-stage="maximized""#));

        let restored = FloatingPanel::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
        assert_eq!(restored.stage(), Stage::Maximized);
    }

    #[test]
    fn position_style_is_consumable_by_positioner_recipe_css_vars() {
        let p = FloatingPanel::new(OpenState::Open, Stage::Default, 100.0, 50.0);
        let style = p.position_style();
        let html = render(&p.positioner(vec![("style", &style)], vec![]));
        assert!(html.contains("--fandhe-x: 100px;"));
        assert!(html.contains("--fandhe-y: 50px;"));
    }
}
