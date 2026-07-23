//! styled TagsInput（headless ラッパー、イシュー #744、親 #736/#520/#546）。
//!
//! `fandhe_frontend_headless_ui::tags_input`（イシュー #744）の
//! Label / Control / Input / Item / ItemPreview / ItemText / ItemInput /
//! ItemDeleteTrigger / ClearTrigger / HiddenInput 10 anatomy パーツをそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::pin_input`]/[`crate::number_input`] の rustdoc と同じ方針
//! に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`TagsInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::pin_input::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`label`]/[`control`]/[`input`]/[`item`]/[`item_preview`]/[`item_text`]/
//! [`item_input`]/[`item_delete_trigger`]/[`clear_trigger`]/[`hidden_input`]/
//! [`TagsInputAction`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::tags_input::TagsInput`] は**あえて**
//! 再エクスポートしない（[`crate::pin_input`]/[`crate::number_input`] の非再
//! エクスポートと同じ理由、PR #695 Bugbot 指摘の前例）。状態管理・hydration
//! が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::tags_input::TagsInput` を直接 import し、
//! 実際の描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `hidden-input` に CSS を付与しない理由
//!
//! [`crate::pin_input`] の `hidden-input` と同じ理由（`<input type="hidden">`
//! は UA 既定挙動として常にレンダリングされない）で、[`recipe`] は
//! `hidden-input` slot へ一切の CSS を登録しない
//! （`hidden_input_slot_has_no_css_rules` テストで固定）。
//!
//! # `size` variant
//!
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）に従い、
//! `size`（[`Size`]）は styled `root` へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-tags-input-*` の root スコープ CSS custom property
//! （通常の CSS 継承）経由で `item-preview`/`input` の寸法・書体を切り替える。
//! `base`/`variant` 規則の `var()` にはいずれも Md サイズ相当のフォールバック
//! 値を書き、styled `root` を経由しない headless 直接利用マークアップでも
//! 現行外観を維持する（fail-safe）。フォーム入力部品のため palette
//! （color-palette 軸）は本イシューでは提供しない（[`crate::number_input`]
//! の先例に倣う）。
//!
//! # フォーカスリング（実フォーカスを `input` 自身が受ける構成）
//!
//! [`crate::pin_input`] と同じく `input` は `<input type="text">` 自身が
//! 実フォーカスを受けるネイティブ要素であるため、
//! `StateCondition::FocusVisible`（`:focus-visible` 疑似クラス）を直接
//! `input` slot へ登録する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// `TagsInput` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::tags_input::TagsInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tags_input::{
    clear_trigger, control, hidden_input, input, item, item_delete_trigger, item_input,
    item_preview, item_text, label, TagsInputAction,
};

/// headless `tags_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/tags_input.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "item",
    "item-preview",
    "item-text",
    "item-input",
    "item-delete-trigger",
    "clear-trigger",
    "hidden-input",
];

/// この styled TagsInput の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tags-input", SLOTS)
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
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .base(
            "item-preview",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl(
                    "padding",
                    "var(--fandhe-tags-input-chip-padding-y, 0.125rem) var(--fandhe-tags-input-chip-padding-x, 0.5rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .state(
            "item-preview",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .base("item-text", vec![decl("white-space", "nowrap")])
        .base(
            "item-delete-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "1rem"),
                decl("height", "1rem"),
                decl("padding", "0"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
            ],
        )
        .state(
            "item-delete-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "input",
            vec![
                decl("flex", "1 1 auto"),
                decl("min-width", "var(--fandhe-tags-input-input-min-width, 6rem)"),
                decl("box-sizing", "border-box"),
                decl("border", "none"),
                decl("outline", "none"),
                decl("background", "transparent"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // 実フォーカスを `input` 自身が受けるため `:focus-visible` を直接
        // 登録する（モジュール rustdoc「フォーカスリング」節参照）。
        .state(
            "input",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "clear-trigger",
            vec![
                decl("align-self", "flex-start"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .state(
            "clear-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-xs)",
            )],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .default_variant(Size::Md)
}

/// この styled TagsInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::pin_input::stylesheet`]/[`crate::number_input::stylesheet`] と
/// 同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::tags_input::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tags_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = tags_input::root(Size::Md, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tags-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::tags_input::root(disabled, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tags-input"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_to_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="control"][data-invalid] {"#));
    }

    #[test]
    fn stylesheet_links_item_preview_to_highlighted_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="item-preview"][data-highlighted] {"#
        ));
    }

    #[test]
    fn stylesheet_links_input_to_focus_visible_outline() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="input"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_slot_has_no_css_rules() {
        // モジュール rustdoc「`hidden-input` に CSS を付与しない理由」参照。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-part="hidden-input"]"#));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains("fd-tags-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-tags-input--size-sm"),
            (Size::Md, "fd-tags-input--size-md"),
            (Size::Lg, "fd-tags-input--size-lg"),
        ] {
            let html = render(&root(size, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors_and_custom_properties() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-tags-input-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reflects_disabled_prop() {
        let html = render(&root(Size::Md, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_item_text_tag_payload_is_escaped_on_render() {
        let html = render(&item_text(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_delete_trigger_aria_label_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item_delete_trigger(PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_tags_input_state_machine() {
        // `TagsInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::tags_input::TagsInput;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = TagsInput::new(Vec::new(), Some(5));
        assert!(t.is_empty());

        let ssr_html = render(&t.root(false, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut t, "add", "rust"));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-max="5""#));

        let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), t.value());
    }
}
