//! styled Toolbar（headless ラッパー、イシュー #991、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::toolbar`（イシュー #991）の Root / Button /
//! Link / Separator / ToggleGroup / ToggleItem 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::toolbar::Toolbar`] roving tabindex 状態
//! 機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する
//! （[`crate::action_bar`] と同型の薄い委譲）。
//!
//! # レイアウト
//!
//! `root` は `display: flex` + `gap` の横並びを既定とし、
//! `data-orientation="vertical"` のとき `flex-direction: column` へ切り替える
//! （headless 層が `data-orientation` を固定出力する契約、
//! `crates/headless-ui/src/toolbar.rs` 参照）。
//!
//! # separator の向き別太さ
//!
//! headless 層の `separator` は toolbar 自身の向きと直交する
//! `aria-orientation` を出力する（横向き toolbar → 縦線）。本モジュールは
//! `aria-orientation` の値そのものをセレクタに使い、縦線（`width: 1px;
//! align-self: stretch`）と横線（`height: 1px; width: 100%`）を出し分ける。
//!
//! # focus-visible リング
//!
//! `button`/`link`/`toggle-item` はいずれもネイティブなフォーカス可能要素
//! （`<button>`/`<a>`）であり、キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::action_bar`]/[`crate::dialog`] と同じ判断）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/toolbar.rs`）のモジュール doc
//! 「スコープ外」節をそのまま継承する（矢印キー実 DOM 配線・skip-disabled
//! モード・`loopFocus` の視覚表現・オーバーフロー時のスクロール折りたたみ）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::toolbar::*;
// `Orientation` は `root`/`separator` の引数型・`Toolbar::new` の引数型として
// 呼び出し側が組み立てる必要があるが、`toolbar` モジュールの glob 再エクス
// ポートでは到達しない（`data_attrs` モジュール由来のため）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（[`crate::action_bar`] の `OpenState`/
// `DisclosureAction` と同型のパターン）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// headless `toolbar` anatomy の `data-part` 一覧（`crates/headless-ui/src/toolbar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "button",
    "link",
    "separator",
    "toggle-group",
    "toggle-item",
];

/// この styled Toolbar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("toolbar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.5rem"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base(
            "button",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "0.25rem"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
            ],
        )
        .base(
            "link",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-decoration", "none"),
                decl("border-radius", "0.25rem"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("background", "var(--fandhe-color-border)"),
                decl("width", "1px"),
                decl("align-self", "stretch"),
            ],
        )
        .base(
            "toggle-group",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "toggle-item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "1px solid transparent"),
                decl("border-radius", "0.25rem"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
            ],
        )
        // root が縦向きのとき列方向へ切り替える（本モジュール冒頭 rustdoc
        // 「レイアウト」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        // separator は toolbar 自身の向きと直交する aria-orientation を持つ
        // ため、その値そのものをセレクタに使って向き別の太さを出し分ける
        // （本モジュール冒頭 rustdoc「separator の向き別太さ」節参照）。
        .state(
            "separator",
            StateCondition::AttrEq("aria-orientation", "horizontal"),
            vec![
                decl("height", "1px"),
                decl("width", "100%"),
                decl("align-self", "auto"),
            ],
        )
        // 押下中の toggle-item を視覚的に強調する。
        .state(
            "toggle-item",
            StateCondition::AttrEq("data-state", "on"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("border-color", "var(--fandhe-color-accent)"),
            ],
        )
        // disabled でもフォーカス順序には残るため（headless 層の意図的な
        // 設計判断、`crates/headless-ui/src/toolbar.rs` モジュール doc
        // 「スコープ外」節参照）、視覚的にのみ操作不能を示す。
        .state(
            "button",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "toggle-item",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "button",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "link",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "toggle-item",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Toolbar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::action_bar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="root"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="button"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="link"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="separator"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="toggle-group"]"#));
        assert!(a.contains(r#"[data-scope="toolbar"][data-part="toggle-item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn root_switches_to_column_when_vertical() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="toolbar"][data-part="root"][data-orientation="vertical"]"#));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn separator_horizontal_aria_orientation_overrides_vertical_default() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toolbar"][data-part="separator"][aria-orientation="horizontal"]"#
        ));
        assert!(css.contains("height: 1px;"));
    }

    #[test]
    fn toggle_item_pressed_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="toggle-item"][data-state="on"]"#));
    }

    #[test]
    fn button_and_toggle_item_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="button"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="link"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="toolbar"][data-part="toggle-item"]:focus-visible {"#));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Orientation::Horizontal, "Toolbar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="toolbar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="toolbar""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_toolbar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Toolbar::new(0, 3, false, Orientation::Horizontal);
        assert_eq!(t.focused(), 0);

        let ssr_html = render(&t.button(0, false, vec![], vec![]));
        assert!(ssr_html.contains(r#"tabindex="0""#));

        assert!(dispatch(&mut t, "next", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-focused="1""#));

        let restored = Toolbar::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }
}
