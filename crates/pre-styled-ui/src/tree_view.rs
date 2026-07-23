//! styled TreeView（headless ラッパー、イシュー #753、親トラッキング #748/#520）。
//!
//! `fandhe_frontend_headless_ui::tree_view`（イシュー #753）の Root / Label /
//! Tree / Branch / BranchControl / BranchIndicator / BranchText /
//! BranchContent / BranchIndentGuide / Item / ItemText / ItemIndicator の 12
//! anatomy パーツと [`fandhe_frontend_headless_ui::tree_view::TreeView`]
//! 状態機械・[`fandhe_frontend_headless_ui::tree_view::TreeNode`] コレクション
//! をそのまま再エクスポートし（`pub use ...::*`、[`crate::tooltip`]/
//! [`crate::popover`] と同型の名前衝突なし薄い委譲）、[`stylesheet`] で既定
//! CSS を追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::dialog`]/
//! [`crate::tooltip`] の rustdoc と同じ方針に従う（`data-scope`/`data-part`
//! セレクタへの CSS 適用のみで、パーツ関数へ手を加えない）。
//!
//! # `size`/`color-palette` variant を提供しない（ナビゲーション/コレクション
//! 表示部品、`crate::lib` rustdoc「複合部品の variant 統一方針」§3 参照）
//!
//! TreeView はオーバーレイの配置・寸法がコンテンツ起因の部品ではないが、
//! ツリー構造の階層表示という性質上、寸法スケール（`size`）や選択状態の
//! アクセント色（`color-palette`）を適用する明確な基準がない。
//! [`crate::popover`]/[`crate::tooltip`] の非提供判断（variant 統一方針 §3
//! 「オーバーレイの配置・寸法がコンテンツ/positioning 起因の popover/tooltip
//! には提供しない」）と同型の理由で、本モジュールも意図的に variant を
//! 提供しない。
//!
//! # インデントは CSS custom property（受け入れ条件）
//!
//! `branch-content` の `padding-inline-start` へ
//! `var(--fandhe-tree-view-indent, 1rem)` を設定する。DOM ネスト（[`headless
//! TreeView::render_nodes`](fandhe_frontend_headless_ui::tree_view::TreeView::render_nodes)
//! が組み立てる `branch > branch-content > root > branch/item` の再帰構造）
//! により、深さ分のインデントが親子の `padding-inline-start` の重ね掛けで
//! 自然に累積する（CSS のみで完結し、深さごとの数値計算・追加の CSS 変数を
//! 持たない）。`branch-indent-guide` は同じ custom property を
//! `border-inline-start` の位置基準として使い、縦のガイド線を描く。
//!
//! # 選択・開閉状態の CSS 反映
//!
//! - 展開状態: `branch`/`branch-control`/`branch-indicator`/`branch-content`
//!   の `data-state`（`"open"`/`"closed"`）へ [`recipe::StateCondition::AttrEq`]
//!   で反応する。
//! - 選択状態: `branch`/`item` の `data-selected` 存在属性へ
//!   [`recipe::StateCondition::Attr`] で反応する。
//! - disabled: `branch`/`item` の `data-disabled` 存在属性へ反応する
//!   （[`crate::tags_input`] 等と同型）。
//!
//! # キーボード操作系スタイル
//!
//! `branch-control`/`item` はクリック対象（`item` は `tabindex` 経由の
//! フォーカス対象になりうる。実 DOM 配線は headless モジュール doc
//! §out-of-scope 参照）であり、キーボード操作時のみのフォーカスリング
//! （`:focus-visible`）を [`recipe::StateCondition::FocusVisible`] 経由で
//! 登録する（[`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] と同じ判断）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::tree_view::*;
// `branch`/`item` 等の `state`/`selected`/`disabled` 引数・`TreeView` の
// `Component::Action`（dispatch 対象）・`OpenState` はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（[`crate::tooltip`] と同じ判断、イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{MultiSelectAction, OpenState, SingleSelectAction};

/// headless `tree-view` anatomy の `data-part` 一覧（`crates/headless-ui/src/tree_view.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "tree",
    "branch",
    "branch-control",
    "branch-indicator",
    "branch-text",
    "branch-content",
    "branch-indent-guide",
    "item",
    "item-text",
    "item-indicator",
];

/// この styled TreeView の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tree-view", SLOTS)
        .base(
            "tree",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "branch-control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "branch-indicator",
            vec![
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base("branch-text", vec![decl("color", "var(--fandhe-color-fg)")])
        // イシュー #753 受け入れ条件: インデントは CSS custom property。
        .base(
            "branch-content",
            vec![decl(
                "padding-inline-start",
                "var(--fandhe-tree-view-indent, 1rem)",
            )],
        )
        .base(
            "branch-indent-guide",
            vec![
                decl(
                    "border-inline-start",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
                decl(
                    "margin-inline-start",
                    "calc(var(--fandhe-tree-view-indent, 1rem) / 2)",
                ),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-indicator",
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        // 展開状態の見た目切り替え（branch-indicator の回転表示）。
        .state(
            "branch-indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(90deg)")],
        )
        // 選択状態の見た目切り替え（branch/item 共通）。
        .state(
            "branch-control",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent)"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent)"),
            ],
        )
        // disabled の見た目切り替え（branch/item 共通、`crate::tags_input` と同型）。
        .state(
            "branch-control",
            StateCondition::Attr("data-disabled"),
            vec![
                decl("opacity", "0.5"),
                decl("cursor", "not-allowed"),
                decl("pointer-events", "none"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![
                decl("opacity", "0.5"),
                decl("cursor", "not-allowed"),
                decl("pointer-events", "none"),
            ],
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "branch-control",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "item",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled TreeView が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約: 同一プロセス内の複数回呼び出し
/// は常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tree-view"][data-part="branch-content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn branch_content_indent_uses_css_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-content"]"#));
        assert!(css.contains("padding-inline-start: var(--fandhe-tree-view-indent, 1rem);"));
    }

    #[test]
    fn branch_indent_guide_uses_border_and_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="branch-indent-guide"]"#));
        assert!(css.contains("border-inline-start:"));
        assert!(css.contains("var(--fandhe-tree-view-indent, 1rem)"));
    }

    #[test]
    fn stylesheet_links_branch_indicator_to_open_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tree-view"][data-part="branch-indicator"][data-state="open"]"#
        ));
        assert!(css.contains("transform: rotate(90deg);"));
    }

    #[test]
    fn stylesheet_links_selected_state_for_branch_control_and_item() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-selected]"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"][data-selected]"#));
    }

    #[test]
    fn stylesheet_links_disabled_state_for_branch_control_and_item() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"][data-disabled]"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"][data-disabled]"#));
    }

    #[test]
    fn branch_control_and_item_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="tree-view"][data-part="branch-control"]:focus-visible {"#)
        );
        assert!(css.contains(r#"[data-scope="tree-view"][data-part="item"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_render_nodes_renders_full_tree_markup() {
        let nodes = vec![
            TreeNode::new("src", "src").with_children(vec![TreeNode::new("a.rs", "a.rs")]),
            TreeNode::new("readme.md", "readme.md"),
        ];
        let tree_view = TreeView::default();
        let rendered = tree_view.render_nodes(&nodes);
        let html = rendered.iter().map(render).collect::<Vec<_>>().join("");
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains("src"));
        assert!(html.contains("a.rs"));
        assert!(html.contains("readme.md"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_tree_view_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = TreeView::default();
        assert_eq!(t.selected(), None);
        assert!(!t.is_expanded("src"));

        let ssr_html = render(&branch_indicator(
            OpenState::Closed,
            vec![],
            vec![text("+")],
        ));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "expand", "src"));
        assert!(dispatch(&mut t, "select", "a.rs"));

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains("data-hydrate-expanded="));
        assert!(hydrate_html.contains("data-hydrate-selected="));

        let restored = TreeView::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert!(restored.is_expanded("src"));
        assert_eq!(restored.selected(), Some("a.rs"));
    }
}
