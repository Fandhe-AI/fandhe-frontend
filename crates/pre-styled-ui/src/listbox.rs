//! styled Listbox（headless ラッパー、イシュー #750、親 #520/#546/#748）。
//!
//! `fandhe_frontend_headless_ui::listbox`（イシュー #750）の Label / Content /
//! ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator / ValueText
//! 8 anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::select`] の
//! rustdoc と同じ方針に従う。
//!
//! # [`crate::select`] との責務境界
//!
//! headless 層と同じく、styled Select はポップアップ型（trigger/positioner
//! を持つ）であるのに対し、styled Listbox は常時展開（`content` が常に
//! 表示される、`hidden`/`positioner`/`trigger` を一切持たない）。「常に
//! 見えているリストから選ぶ」用途には本モジュールを、「クリックで開閉する
//! ドロップダウン」用途には [`crate::select`] を使う（詳細は
//! `fandhe_frontend_headless_ui::listbox` module doc 参照）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Listbox`/
//! `MultiListbox` 型・headless `root` を再エクスポートしない理由）
//!
//! `size` variant クラス付与のため styled [`root`]（[`crate::select::root`]
//! と同型）を本モジュールで新設する。headless 自由関数 `root` と名前が
//! 衝突するため、`pub use ...::*` ではなく必要な識別子のみを選択的に再
//! エクスポートする。状態機械 [`fandhe_frontend_headless_ui::listbox::Listbox`]/
//! [`fandhe_frontend_headless_ui::listbox::MultiListbox`] は**あえて**
//! 再エクスポートしない（[`crate::select`]/[`crate::switch`]/[`crate::menu`]
//! の状態機械非再エクスポートと同じ理由）。状態管理・hydration が必要な
//! 呼び出し側は `fandhe_frontend_headless_ui::listbox::{Listbox, MultiListbox}`
//! を直接 import し、実際の描画は本モジュールの styled [`root`]（および
//! 再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `item`（選択有無、`data-state` を再利用）・`root`（disabled）の
//! `data-*` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]、[`crate::select`] と同じ機構）。
//!
//! # ハイライト表示（SSR 静的表現）
//!
//! `item` の `data-highlighted`（[`crate::select`] と同じ virtual focus
//! パターン、イシュー #581/#599）は選択済み `item[data-state="open"]` とは
//! 背景色を変えて視覚的に区別する。`content` 自身が DOM フォーカスを受ける
//! （headless module doc 参照）ため `:focus-visible` は `content` slot へ
//! 登録する（[`crate::select`] の `trigger` に相当）。
//!
//! # `color-palette` 軸を提供しない判断
//!
//! [`crate::select`]/[`crate::menu`]/[`crate::tags_input`] の既存判断に
//! 追随し、`size` variant のみを提供する（chakra 固有の `variant`
//! （subtle/solid/plain）展開は out-of-scope として PR 本文で別イシュー化を
//! 提案する）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root`・状態機械 `Listbox`/`MultiListbox` はあえて再
// エクスポートしない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。
// 未スタイル・variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::listbox` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::listbox::{
    content, item, item_group, item_group_label, item_indicator, item_text, label, value_text,
};
// `root`/`item`/`item_indicator` 等の状態引数はいずれも headless
// `state`/`OpenState` 由来で上記選択的再エクスポートでは到達しない。呼び出し
// 側が `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証
// するための明示再エクスポート（[`crate::select`] の `OpenState` 再
// エクスポートと同じ理由）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `listbox` anatomy の `data-part` 一覧（`crates/headless-ui/src/listbox.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
    "value-text",
];

/// この styled Listbox の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("listbox", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
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
                decl("overflow-y", "auto"),
                decl(
                    "max-height",
                    "var(--fandhe-listbox-content-max-height, 16rem)",
                ),
                decl(
                    "padding",
                    "var(--fandhe-listbox-content-padding, var(--fandhe-space-2))",
                ),
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
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "padding",
                    "var(--fandhe-listbox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-text",
            vec![decl("flex", "1"), decl("min-width", "0")],
        )
        .base(
            "value-text",
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        // 選択済み item の見た目の切り替え（headless `data-state` 語彙の再利用）。
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // virtual focus の highlight 表示（`item` は実 DOM フォーカスを受けない
        // ため `:focus-visible` ではなく `data-highlighted` で表現する。既存の
        // 選択済み表示（背景 `bg-muted`）とは異なる強度にして視覚的に区別する、
        // モジュール rustdoc 参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // disabled item は減光 + cursor: not-allowed。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        // `content` 自身が DOM フォーカスを受けるため（headless module doc
        // 参照）、キーボード操作時のみのフォーカスリングを `content` へ登録する
        // （[`crate::select`] の `trigger` に相当）。
        .state(
            "content",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "-2px"),
            ],
        )
        // `size` variant（root スコープの CSS custom property。Md はフォールバック
        // 値と同一の現行外観を維持する。[`crate::select`] の `size` variant と
        // 同型の判断）。
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-1)"),
                decl("--fandhe-listbox-content-max-height", "12rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-listbox-content-max-height", "16rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-listbox-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-listbox-content-padding", "var(--fandhe-space-3)"),
                decl("--fandhe-listbox-content-max-height", "20rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Listbox が生成する静的 CSS 全量を返す（決定的。[`crate::select::stylesheet`]
/// と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::listbox::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::listbox::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = listbox::root(Size::Md, OpenState::Closed, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="listbox" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    selection_state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::listbox::root(selection_state, disabled, merged, children)
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
        assert!(a.contains(r#"[data-scope="listbox"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="listbox""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                false,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-listbox--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_fallback_values() {
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-listbox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(css.contains("max-height: var(--fandhe-listbox-content-max-height, 16rem);"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn item_highlighted_and_disabled_states_are_styled_distinctly() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="listbox"][data-part="item"][data-disabled] {"#));
    }

    #[test]
    fn content_has_focus_visible_ring_since_it_receives_dom_focus() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="content"]:focus-visible {"#));
    }

    #[test]
    fn root_disabled_state_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="listbox"][data-part="root"][data-disabled] {"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_listbox_state_machine() {
        // SSR / hydration 両経路の動作確認: 本モジュールから状態機械を再
        // エクスポートしないため、エスケープハッチ経由で直接 import する
        // （モジュール冒頭の rustdoc「選択的 re-export」節参照）。
        use fandhe_frontend_headless_ui::listbox::Listbox;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut l = Listbox::default();
        let ssr_html = render(&l.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut l, "select", "apple"));
        let hydrate_html = render(&render_for_hydration(&l));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Listbox::from_hydration_attrs(&l.hydration_attrs()).unwrap();
        assert_eq!(restored.selected(), Some("apple"));
    }
}
