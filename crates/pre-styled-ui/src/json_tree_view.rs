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
//! セマンティックトークンへマップする: `string` → success-fg-subtle /
//! `number` → info-fg-subtle / `bool` → warning-fg-subtle（+ semibold） /
//! `null` → fg-muted（+ semibold + italic） / `object`・`array`
//! （ブランチ要約、`data-kind` はコンテナ側の型を表す）→ italic のみ追加 /
//! `key` → fg（既定文字色のまま、配色分岐なし。medium）。
//!
//! # 参考サイト基準への調整（イシュー #1563）
//!
//! 参照元は ark-ui（`chakra-ui/ark` リポジトリの
//! `website/src/components/ui/primitives/json-tree-view.tsx` sva recipe /
//! `.storybook/modules/json-tree-view.module.css`。chakra-ui・Radix には
//! 対応部品が存在しない）。実測・是正した点:
//!
//! 1. **コントラスト不適合の是正**: 旧配色（`--fandhe-color-info`/`-warning`）は
//!    `theme.rs` の `DEFAULT_COLORS` 実測でライトテーマ時 `info`/`bg` =
//!    4.03:1・`warning`/`bg` = 3.64:1 となり、本文相当の 4.5:1 を満たさない
//!    （`theme.rs` の契約テストにも「大字・UI 部品 3:1」ペアとしてのみ登録
//!    されており、`font-size-sm` の地の文字色としては不適切）。`<palette>-fg-subtle`
//!    （ark の `fg.<palette>` 相当のテキスト用トークン）はライト 9.4〜10.1:1・
//!    ダーク 11.0〜13.3:1 を満たすため、`string`/`number`/`bool` の 3 色を
//!    `success-fg-subtle`/`info-fg-subtle`/`warning-fg-subtle` へ置換した。
//! 2. **monospace フォント**: 参照は `tree`/`key`/`value` 全体が `fontFamily: mono`。
//!    `tree` 側のフォントは [`crate::tree_view`]（#1578）の所掌のため、本
//!    モジュールは `key`/`value` の 2 パーツにのみ `font-family:
//!    var(--fandhe-font-font-mono)` を追加した（JSON 表示固有の関心）。
//! 3. **`key` の太さ**: 参照は `[data-kind="key"]` に `fontWeight: medium`。
//!    `key` パーツに `font-weight: medium` を追加した。
//! 4. **`bool`/`null` の強調**: 参照は `boolean` → `fontWeight: semibold`、
//!    `null`/`undefined` → `fontWeight: semibold` + `fontStyle: italic`。
//!    それぞれ追加した。
//! 5. **`object`/`array`（折りたたみ時の要約表示）の斜体**: 参照の
//!    `preview-text`（Object/Array 要約）→ `fontStyle: italic` に倣い、
//!    `data-kind="object"`/`"array"` に `font-style: italic` を追加した
//!    （色は既定の `fg-muted` を維持）。
//!
//! # 意図的非採用（参考サイト基準からの差分）
//!
//! - **`key` を緑色にしない**: 参照（Park UI 系デザイン）は `key` に
//!   `fg.success` 相当の緑を使うが、`value` の `string` にも同系統の
//!   `success-fg-subtle` を採用しているため、`key` まで緑にすると同一画面内
//!   で意味の異なる 2 箇所が同色になり判読性を損なう。当リポジトリでは
//!   `key` の既定文字色（fg）を維持する。
//! - **colon（`:`）区切りの装飾**: anatomy（構造）に colon パーツが存在せず
//!   （headless #1661 が anatomy 突合の対象）、CSS 生成コンテンツ（`::after`
//!   等）での擬似的な追加は #1661 が将来 colon パーツを追加した際の二重表示
//!   を招くため、本 PR では追加しない。
//! - **`tree`/`branch-control`/`item` の hover・フォーカスリング・
//!   トランジション・`font-size`**: [`crate::tree_view`] スコープ（別イシュー
//!   #1578）の所掌であり、`json-tree-view` スコープには持ち込まない
//!   （span への hover は行 hover と競合するため）。
//! - **`size`/`color-palette` variant**: 上記のとおり不採用を維持する
//!   （方針は変更なし）。

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
///
/// `data-kind` の出力元は headless-ui（`crates/headless-ui/src/
/// json_tree_view.rs` の `value` パーツ）。本モジュールは CSS セレクタ
/// として参照するのみで、属性を出力しない（イシュー #1063、
/// `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 A）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("json-tree-view", SLOTS)
        .base(
            "key",
            vec![
                // イシュー #1563: 参照（ark-ui）の `[data-kind="key"]` は
                // `fontWeight: medium`。色は緑にせず fg のまま維持する
                // （モジュール doc「意図的非採用」節参照）。
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-family", "var(--fandhe-font-font-mono)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .base(
            "value",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-family", "var(--fandhe-font-font-mono)"),
            ],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "string"),
            // イシュー #1563: 旧 `--fandhe-color-success`（ライト 4.54:1）から
            // `-fg-subtle`（ライト 9.4〜10.1:1）へ変更しコントラストを底上げ。
            vec![decl("color", "var(--fandhe-color-success-fg-subtle)")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "number"),
            // イシュー #1563: 旧 `--fandhe-color-info` はライト 4.03:1 で
            // 本文相当 4.5:1 未達だったため `-fg-subtle` へ置換。
            vec![decl("color", "var(--fandhe-color-info-fg-subtle)")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "bool"),
            // イシュー #1563: 旧 `--fandhe-color-warning` はライト 3.64:1 で
            // 本文相当 4.5:1 未達だったため `-fg-subtle` へ置換。参照の
            // `boolean` → `fontWeight: semibold` も追加。
            vec![
                decl("color", "var(--fandhe-color-warning-fg-subtle)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            ],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "null"),
            // イシュー #1563: 参照の `null`/`undefined` →
            // `fontWeight: semibold` + `fontStyle: italic` に倣う。色は
            // 既定の fg-muted を維持（null は「値が無い」ことを示す中立表現）。
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("font-style", "italic"),
            ],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "object"),
            // イシュー #1563: 参照の折りたたみ時要約（preview-text）→
            // `fontStyle: italic` に倣う。色は fg-muted 継承のまま。
            vec![decl("font-style", "italic")],
        )
        .state(
            "value",
            StateCondition::AttrEq("data-kind", "array"),
            vec![decl("font-style", "italic")],
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
        for kind in ["string", "number", "bool", "null", "object", "array"] {
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
        assert!(css.contains("var(--fandhe-color-success-fg-subtle)"));
        assert!(css.contains("var(--fandhe-color-info-fg-subtle)"));
        assert!(css.contains("var(--fandhe-color-warning-fg-subtle)"));
        assert!(css.contains("var(--fandhe-color-fg-muted)"));
        assert!(css.contains("var(--fandhe-color-fg)"));
        assert!(css.contains("var(--fandhe-font-font-mono)"));
        assert!(css.contains("var(--fandhe-font-font-weight-medium)"));
        assert!(css.contains("var(--fandhe-font-font-weight-semibold)"));
    }

    /// イシュー #1563: コントラスト是正のため旧トークン
    /// （`--fandhe-color-success`/`-info`/`-warning`、ライトテーマで
    /// 4.5:1 未達）を使わなくなったことを固定する回帰テスト。
    #[test]
    fn stylesheet_no_longer_uses_low_contrast_tokens() {
        let css = stylesheet();
        assert!(!css.contains("var(--fandhe-color-success)"));
        assert!(!css.contains("var(--fandhe-color-info)"));
        assert!(!css.contains("var(--fandhe-color-warning)"));
    }

    #[test]
    fn key_and_value_use_mono_font_family() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="json-tree-view"][data-part="key"] {
  color: var(--fandhe-color-fg);
  font-family: var(--fandhe-font-font-mono);"#
        ));
        assert!(css.contains(
            r#"[data-scope="json-tree-view"][data-part="value"] {
  color: var(--fandhe-color-fg-muted);
  font-family: var(--fandhe-font-font-mono);"#
        ));
    }

    /// イシュー #1563: `bool` は semibold、`null` は semibold + italic を
    /// 参照（ark-ui）基準で追加した宣言が出力に含まれることを固定する。
    #[test]
    fn bool_and_null_declare_weight_and_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="json-tree-view"][data-part="value"][data-kind="bool"] {
  color: var(--fandhe-color-warning-fg-subtle);
  font-weight: var(--fandhe-font-font-weight-semibold);
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="json-tree-view"][data-part="value"][data-kind="null"] {
  color: var(--fandhe-color-fg-muted);
  font-weight: var(--fandhe-font-font-weight-semibold);
  font-style: italic;
}"#
        ));
    }

    /// イシュー #1563: `object`/`array`（折りたたみ要約）は色を変えず
    /// 斜体のみを追加する（参照の `preview-text` 相当）。
    #[test]
    fn object_and_array_declare_italic_only() {
        let css = stylesheet();
        for kind in ["object", "array"] {
            assert!(css.contains(&format!(
                r#"[data-scope="json-tree-view"][data-part="value"][data-kind="{kind}"] {{
  font-style: italic;
}}"#
            )));
        }
    }

    /// イシュー #1563: セマンティックトークン var() 経由以外の生の色
    /// リテラル（16 進カラー・`rgb(`）を CSS 出力へ混入させない不変条件。
    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
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
