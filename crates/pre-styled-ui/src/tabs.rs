//! styled Tabs（headless ラッパー第 1 弾、イシュー #551、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::tabs`（イシュー #528）は Root / List /
//! Trigger / Content / Indicator（#601、opt-in）の 5 anatomy パーツを [`tabs`] 単一の合成関数として
//! 組み立てる（パーツごとの自由関数・attrs 注入点を持たない、他 4 コンポーネント
//! との非対称点）。本モジュールは [`tabs`]・[`TabsProps`]・[`TabItem`] を
//! そのまま再エクスポートし、[`stylesheet`] で `data-scope="tabs"`/
//! `data-part="..."` セレクタに対する既定 CSS のみを追加提供する
//! （他コンポーネントと異なり、クラス注入の余地自体が headless 側の API 形状に
//! 存在しないため、CSS はセレクタ経由の適用のみで完結する）。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! Tabs は `data-state` に `"open"`/`"closed"` ではなく `"active"`/`"inactive"`
//! 語彙を使う（`crates/headless-ui/src/tabs.rs` の `DATA_STATE_ACTIVE`/
//! `DATA_STATE_INACTIVE`）。選択中の `trigger` を強調する CSS を
//! [`crate::recipe::SlotRecipe::state`]（イシュー #643）経由で [`recipe`] へ
//! 登録する（`serialize_rule` を直接呼ぶ手書きセレクタ機構は廃止した）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `trigger` は roving tabindex（`.claude/rules` 外部だが headless 層 tabs の
//! キーボードナビゲーション実装）でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::tabs::*;

/// headless `tabs` anatomy の `data-part` 一覧（`crates/headless-ui/src/tabs.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "list", "trigger", "content", "indicator"];

/// この styled Tabs の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tabs", SLOTS)
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-4)"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("border", "0"),
                decl("border-bottom", "2px solid transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "content",
            vec![
                decl("padding", "var(--fandhe-space-4) 0"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #551 受け入れ条件: 選択中の `trigger` を強調する。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "active"),
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-bottom-color", "var(--fandhe-color-accent)"),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "inactive"),
            vec![decl("display", "none")],
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Tabs が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_tabs_renders_with_headless_anatomy_attrs() {
        let props = TabsProps {
            id: "t1",
            selected: "one",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        };
        let items = vec![TabItem {
            value: "one",
            trigger: vec![],
            content: vec![],
            disabled: false,
        }];
        let html = render(&tabs(&props, items));
        assert!(html.contains(r#"data-scope="tabs""#));
        assert!(html.contains(r#"data-part="list""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_active_and_inactive() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する（Tabs は
        // open/closed ではなく active/inactive 語彙を使う）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"][data-state="active"]"#));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="content"][data-state="inactive"]"#));
    }

    #[test]
    fn ssr_selected_tab_reflects_active_data_state() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」。
        // Tabs は状態機械を持たないため（headless 側スコープ外）、SSR 側の
        // 静的選択状態が data-state="active"/"inactive" として決定的に
        // 描画されることを固定する。
        let props = TabsProps {
            id: "t1",
            selected: "one",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        };
        let items = vec![
            TabItem {
                value: "one",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
            TabItem {
                value: "two",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
        ];
        let html = render(&tabs(&props, items));
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="inactive""#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }
}
