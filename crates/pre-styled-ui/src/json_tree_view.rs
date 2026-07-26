//! styled JsonTreeView（headless ラッパー、イシュー #829、
//! `docs/policy/intentional-non-adoption.md` §7・
//! `docs/design/component-coverage-map.md` の保留解除）。
//!
//! `fandhe_frontend_headless_ui::json_tree_view`（イシュー #829）の
//! [`JsonValue`]・[`TreeView`]・[`render_json`]・[`expanded_to_depth`]・
//! `key`/`value` の 2 anatomy パーツをそのまま再エクスポートし（`pub use
//! ...::*`、[`crate::tree_view`] と同型の薄い委譲）、[`stylesheet`] で
//! 型別配色の既定 CSS を追加提供する。
//!
//! # tree_view（#753）styled recipe との関係
//!
//! JsonTreeView の構造部（root/tree/branch/branch-control/branch-indicator/
//! branch-content/branch-indent-guide/item/item-indicator、
//! `data-scope="tree-view"`）は headless 層が [`fandhe_frontend_headless_ui::tree_view`]
//! の既存パーツ関数をそのまま呼ぶため、[`crate::tree_view::stylesheet`]
//! （インデント・開閉・選択・focus-visible の CSS）がそのまま適用される。
//! 本モジュールの [`stylesheet`] は JSON 固有の 2 パーツ（`key`/`value`、
//! `data-scope="json-tree-view"`）の型別配色のみを追加する。呼び出し側は
//! 両方の `stylesheet()` を併用する必要がある（`docs-site` showcase の
//! 呼び出し例を参照）。
//!
//! # `size`/`color-palette` variant を提供しない（[`crate::tree_view`] と同型の判断、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」§3 参照）
//!
//! JsonTreeView も [`crate::tree_view`] と同じくナビゲーション/コレクション
//! 表示部品であり、寸法スケール（`size`）や選択状態のアクセント色
//! （`color-palette`）を適用する明確な基準がないため、意図的に variant を
//! 提供しない。
//!
//! # 型別配色（受け入れ条件）
//!
//! [`fandhe_frontend_headless_ui::json_tree_view::JsonValue::kind`] が返す
//! 6 種の `data-kind` 値へ [`StateCondition::AttrEq`] で反応し、既定 Theme の
//! セマンティックトークンへマップする: `string` → success / `number` → info /
//! `bool` → warning / `null` → fg-muted / `object`・`array`（ブランチ要約）→
//! fg-muted / `key` → fg（既定文字色のまま、配色分岐なし）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数・variant 型を再定義しない（規約 B-1）。variant 軸
// も提供せず（規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタ
// のみに依存する（規約 B-3）。headless 側 `json_tree_view` モジュールが
// 持つ `pub use`（`TreeView`）は下記の明示再エクスポート名（`TreeViewAction`
// 等）と衝突しないことを確認済み（イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::json_tree_view::*;
// `TreeView` の `Component::Action`（dispatch 対象）・`OpenState` はいずれも
// [`fandhe_frontend_headless_ui::tree_view`]/`state` 由来で上記 glob
// 再エクスポートでは到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui`
// のみに依存して呼び出せることを保証するための明示再エクスポート
// （[`crate::tree_view`] と同じ判断、イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{MultiSelectAction, OpenState, SingleSelectAction};
pub use fandhe_frontend_headless_ui::tree_view::TreeViewAction;

/// headless `json-tree-view` anatomy の `data-part` 一覧（`crates/headless-ui/src/json_tree_view.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。[`crate::tree_view::SLOTS`]
/// と同じ理由でずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れる）。
const SLOTS: &[&str] = &["key", "value"];

/// この styled JsonTreeView の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("json-tree-view", SLOTS)
        .base("key", vec![decl("color", "var(--fandhe-color-fg)")])
        .base("value", vec![decl("color", "var(--fandhe-color-fg-muted)")])
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "string"),
            vec![decl("color", "var(--fandhe-color-success)")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "number"),
            vec![decl("color", "var(--fandhe-color-info)")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "bool"),
            vec![decl("color", "var(--fandhe-color-warning)")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "null"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
}

/// この styled JsonTreeView が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tree_view::stylesheet`] と同じ契約: 同一プロセス内の複数回呼び出し
/// は常にバイト単位で同一の文字列を返す）。[`crate::tree_view::stylesheet`]
/// との併用が前提（モジュール doc §tree_view styled recipe との関係参照）。
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
        assert!(a.contains(r#"[data-scope="json-tree-view"][data-part="key"]"#));
        assert!(a.contains(r#"[data-scope="json-tree-view"][data-part="value"]"#));
    }

    #[test]
    fn stylesheet_declares_selector_per_kind() {
        let css = stylesheet();
        for kind in ["string", "number", "bool", "null"] {
            assert!(
                css.contains(&format!(
                    r#"[data-scope="json-tree-view"][data-part="value"][data-kind="{kind}"]"#
                )),
                "kind={kind} 用のセレクタが出力に見当たらない: css={css}"
            );
        }
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_uses_theme_token_vars() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-color-success)"));
        assert!(css.contains("var(--fandhe-color-info)"));
        assert!(css.contains("var(--fandhe-color-warning)"));
        assert!(css.contains("var(--fandhe-color-fg-muted)"));
        assert!(css.contains("var(--fandhe-color-fg)"));
    }

    #[test]
    fn reexported_render_json_renders_with_json_tree_view_anatomy_attrs() {
        let tree = TreeView::default();
        let data = JsonValue::Object(vec![(
            "name".to_string(),
            JsonValue::String("Ada".to_string()),
        )]);
        let html = render(&render_json(&tree, &data));
        assert!(html.contains(r#"data-scope="json-tree-view""#));
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains(r#"data-kind="string""#));
    }

    #[test]
    fn reexported_expanded_to_depth_and_dispatch_round_trip() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let data = JsonValue::Object(vec![(
            "a".to_string(),
            JsonValue::Array(vec![JsonValue::Null]),
        )]);
        let mut tree = expanded_to_depth(&data, 1);
        assert!(tree.is_expanded(""));

        assert!(dispatch(&mut tree, "select", "/a"));
        let hydrate_html = render(&render_for_hydration(&tree));
        assert!(hydrate_html.contains("data-hydrate-expanded="));
        assert!(hydrate_html.contains("data-hydrate-selected="));

        let restored = TreeView::from_hydration_attrs(&tree.hydration_attrs()).unwrap();
        assert_eq!(restored, tree);
    }
}
