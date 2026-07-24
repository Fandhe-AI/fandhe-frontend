//! styled ActionBar（headless ラッパー、イシュー #762、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::action_bar`（イシュー #762）の Root /
//! Positioner / Content / SelectionTrigger / Separator / CloseTrigger
//! 6 anatomy パーツと [`fandhe_frontend_headless_ui::action_bar::ActionBar`]
//! 状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供
//! する。薄い委譲の根拠・スコープ外事項は [`crate::dialog`]/[`crate::tooltip`]
//! の rustdoc と同じ方針に従う。
//!
//! # 画面下部固定配置（chakra-ui ActionBar 相当）
//!
//! `positioner` は `position: fixed; bottom: ...; left: 50%; transform:
//! translateX(-50%)` により画面下部中央へ固定表示する。`z-index` は
//! [`crate::menu`]/[`crate::select`] の dropdown `positioner`（10）より上、
//! [`crate::dialog`] の backdrop（1000）より下（900）とする（複数選択の操作
//! バーはダイアログの上に隠れるべきではないが、モーダルよりは背面という
//! 位置付け）。
//!
//! # data-state とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! [`crate::dialog`]/[`crate::tooltip`] と同じ判断）。
//!
//! # キーボード操作系属性の反映
//!
//! `selection-trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::tooltip`] と同じ判断）。
//!
//! # closed 時の `positioner` は必ず非表示化する（PR #575 Bugbot 指摘対応の踏襲）
//!
//! headless 層（`crates/headless-ui/src/action_bar.rs`）は ActionBar が
//! closed のとき `positioner` に `hidden` 存在属性を付与し、UA 既定スタイル
//! `[hidden] { display: none }` によって非表示化させる契約になっている。
//! [`recipe`] の base 規則は `positioner` に `display: flex` を宣言しており、
//! この author スタイルが UA スタイルより詳細度で優先されるため `[hidden]`
//! 単体では非表示化できない（[`crate::dialog`] の PR #575 Bugbot 指摘（High）
//! と同型の不具合）。[`recipe`] に `[data-scope="action-bar"]
//! [data-part="positioner"][hidden]` に対する `display: none` の明示的な
//! 上書き規則を追加し、`display: flex` より詳細度・出現順の両方で優先させる
//! ことでこれを固定する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - Portal 描画・外側クリックでの閉鎖・アニメーションは headless 層の
//!   ドキュメント（`crates/headless-ui/src/action_bar.rs`）で既にスコープ外
//!   と明記済みであり、本モジュールもそれを継承する。
//! - `placement` variant（`bottom-start`/`bottom-end`）: 既定の bottom 中央
//!   固定のみ実装する。variant 追加は `SlotRecipe::variant` で後続可能。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::action_bar::*;
// `root`/`positioner`/`content` 等の `state` 引数・`ActionBar::new`・
// `ActionBar` の `Component::Action`（dispatch 対象）はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の先例に倣う）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `action-bar` anatomy の `data-part` 一覧（`crates/headless-ui/src/action_bar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "positioner",
    "content",
    "selection-trigger",
    "separator",
    "close-trigger",
];

/// この styled ActionBar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("action-bar", SLOTS)
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("z-index", "900"),
                decl("display", "flex"),
                decl("justify-content", "center"),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-3)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.5rem"),
                decl("box-shadow", "0 0.25rem 1rem rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
            ],
        )
        .base(
            "selection-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("width", "1px"),
                decl("align-self", "stretch"),
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        // `content` の開閉状態に応じた見た目の切り替え。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        // 本モジュール冒頭 rustdoc「closed 時の positioner は必ず非表示化する」
        // 節参照: positioner の base 規則が `display: flex` を宣言しており、
        // UA 既定の `[hidden] { display: none }` を上書きしてしまうため、
        // より詳細度の高い `[hidden]` 属性セレクタで明示的に上書きする。
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "selection-trigger",
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

/// この styled ActionBar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`] と同じ契約）。
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
        assert!(a.contains(r#"[data-scope="action-bar"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="action-bar"][data-part="positioner"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_fixed_to_bottom_center_with_stacking_order() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="action-bar"][data-part="positioner"] {"#));
        assert!(css.contains("position: fixed;"));
        assert!(css.contains("left: 50%;"));
        assert!(css.contains("transform: translateX(-50%);"));
        assert!(css.contains("z-index: 900;"));
    }

    #[test]
    fn closed_positioner_hidden_attr_overrides_display_flex() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="action-bar"][data-part="positioner"][hidden] {"#));
        let rule_start = css
            .find(r#"[data-scope="action-bar"][data-part="positioner"][hidden] {"#)
            .expect("positioner[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="content"][data-state="open"]"#)
        );
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="content"][data-state="closed"]"#)
        );
    }

    #[test]
    fn selection_trigger_and_close_trigger_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="action-bar"][data-part="selection-trigger"]:focus-visible {"#
        ));
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="close-trigger"]:focus-visible {"#)
        );
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="action-bar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_action_bar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut bar = ActionBar::default();
        assert_eq!(bar.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&bar.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut bar, "open", ""));
        let hydrate_html = render(&render_for_hydration(&bar));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        // クライアント側の改ざん耐性のある復元経路が ActionBar 経由でも機能する。
        let restored = ActionBar::from_hydration_attrs(&bar.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
