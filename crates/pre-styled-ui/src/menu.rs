//! styled Menu（headless ラッパー第 1 弾、イシュー #551、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::menu`（イシュー #540）の Root / Trigger /
//! Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemGroup /
//! ItemGroupLabel / Separator 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::menu::Menu`] 状態機械をそのまま再エクスポート
//! し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外
//! 事項は [`crate::dialog`] の rustdoc と同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! `trigger`/`content` の開閉 `data-state`（open/closed）に応じた見た目の
//! 切り替えを [`state_css`] で追加する（[`crate::dialog`] と同じ手法）。
//!
//! # positioner のオーバーレイ配置（PR #575 Bugbot 指摘対応）
//!
//! `positioner` に `position: absolute` を設定し、開いた menu が通常のフローに
//! 残らずオーバーレイ表示になるようにする（[`crate::dialog`] の `positioner`・
//! [`crate::select`] の `positioner` と同じ配置責務）。`trigger`/`positioner` は
//! headless 側 `root`（`crates/headless-ui/src/menu.rs`）の子として並置される
//! 兄弟要素であり、`trigger` は `positioner` の祖先になれない。そのため
//! containing block を提供する `position: relative` は共通の祖先である `root`
//! に付与する（PR #575 Bugbot 指摘 1 対応、`trigger` への誤付与を修正）。

use crate::css::{decl, serialize_rule};
use crate::recipe::SlotRecipe;

pub use fandhe_frontend_headless_ui::menu::*;

/// headless `menu` anatomy の `data-part` 一覧（`crates/headless-ui/src/menu.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "indicator",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
    "item",
    "item-group",
    "item-group-label",
    "separator",
];

/// この styled Menu の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menu", SLOTS)
        .base("root", vec![decl("position", "relative")])
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
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        .base(
            "item",
            vec![
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border-muted)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
}

/// `data-state`（open/closed）に連動する CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ、イシュー #551 受け入れ条件）。
fn state_css() -> String {
    let mut out = String::new();
    if let Some(css) = serialize_rule(
        r#"[data-scope="menu"][data-part="trigger"][data-state="open"]"#,
        &[decl("border-color", "var(--fandhe-color-accent)")],
    ) {
        out.push_str(&css);
    }
    if let Some(css) = serialize_rule(
        r#"[data-scope="menu"][data-part="content"][data-state="closed"]"#,
        &[decl("visibility", "hidden")],
    ) {
        out.push_str(&css);
    }
    out
}

/// この styled Menu が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約）。base 規則（[`recipe`]）の後に `data-state` 連動規則
/// （[`state_css`]）を連結する。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css() + &state_css()
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
        assert!(a.contains(r#"[data-scope="menu"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        // PR #575 Bugbot 指摘対応: positioner がオーバーレイ配置になっている
        // ことを固定する（通常のフローに残ったままにならない）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // PR #575 Bugbot 指摘 1 対応: `trigger` と `positioner` は headless
        // `root` の下の兄弟要素であり、`trigger` は `positioner` の祖先には
        // なれない。そのため `position: relative` は共通祖先である `root`
        // に付与されていることを固定する（`trigger` への誤付与への回帰防止）。
        let css = stylesheet();
        assert!(
            css.contains("[data-scope=\"menu\"][data-part=\"root\"] {\n  position: relative;\n}\n")
        );
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="menu""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menu"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menu_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // 再エクスポートされた `Menu`（headless の Component/Hydrate 実装を
        // そのまま継承）経由で固定する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menu::default();
        assert_eq!(m.state(), OpenState::Closed);

        let ssr_html = render(&m.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut m, "open", ""));
        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Menu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
