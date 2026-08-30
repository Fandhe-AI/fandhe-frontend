//! styled PasswordInput（headless ラッパー、イシュー #740、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::password_input`（イシュー #740）の Label /
//! Control / Input / VisibilityTrigger / Indicator 5 anatomy パーツ
//! （headless 側は Root を含む 6 パーツ）をそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::switch`]/[`crate::select`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`PasswordInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::avatar::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::password_input::PasswordInput`] は
//! **あえて**再エクスポートしない（[`crate::switch`] の `Switch` 非再
//! エクスポートと同じ理由、イシュー #708 の判断を踏襲）。`PasswordInput` は
//! `.root(props, attrs, children)` 等の inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size`/`palette`
//! variant クラスを一切付与しない未スタイルの実体である。本モジュールが
//! `PasswordInput` を丸ごと再エクスポートすると、呼び出し側が（styled 層の
//! つもりで）`password_input_instance.root(...)` を呼んでしまい、`size`/
//! `palette` が付与されず見た目が静かに崩れる事故を誘発する。状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::password_input::PasswordInput` を直接
//! import し、実際の描画は本モジュールの styled [`root`]（および再
//! エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` 語彙について
//!
//! headless 層は表示切替を `"visible"`/`"hidden"` 語彙で表現する
//! （`crates/headless-ui/src/password_input.rs` 参照）。[`recipe`] の
//! `control`/`visibility-trigger` への状態連動規則もこの語彙に合わせて
//! `data-state="visible"` を条件とする。
//!
//! # `control` の `focus-within` リング（イシュー #740、`crate::radio_group`
//! と同じ判断）
//!
//! [`crate::recipe::StateCondition::FocusWithin`] を `control` へ登録する。
//! `input`（実フォーカスを受けるネイティブ `<input>`）は `control` の子孫
//! であり、hidden-input パターン（Switch 等）と異なり実際に視覚要素が
//! フォーカスを受けるため、`:focus-within` で祖先の枠へリングを伝播できる
//! （`data-focus-visible` の付け外し配線は不要）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-password-input-height`/`-font-size`/`-padding-x` の root
//! スコープ custom property（CSS の通常のプロパティ継承により `control`/
//! `input` へ伝わる）経由で寸法を切り替える。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、表示中の `visibility-trigger` の色・
//! `control` のフォーカスリング色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・Accent
//! パレット相当のフォールバック値を書き、styled `root` を経由しない headless
//! 直接利用マークアップでも現行外観を維持する（fail-safe、`crate::lib`
//! rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - クライアント側の click → dispatch 配線（`fandhe-frontend-wasm-full`）。
//! - `examples/headless-pre-styled-ui` への PasswordInput 追加（#608/#609 と
//!   同じ後続分離、crates.io 版依存のため公開後にしか追随できない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// `PasswordInput` 状態機械・headless 自由関数 `root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::password_input::PasswordInput` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::password_input::{
    control, indicator, input, label, visibility_trigger, PasswordAutocomplete,
    PasswordInputAction, PasswordInputProps,
};

/// headless `password_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/password_input.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "visibility-trigger",
    "indicator",
];

/// この styled PasswordInput の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("password-input", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-password-input-height, 2.5rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-password-input-padding-x, 0.75rem)",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transition", "border-color 0.15s"),
            ],
        )
        .state(
            "control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "control",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .state(
            "control",
            StateCondition::FocusWithin,
            vec![
                decl(
                    "outline",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "input",
            vec![
                decl("flex", "1"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("outline", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("padding", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-password-input-font-size, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .base(
            "visibility-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("padding", "0 0 0 var(--fandhe-space-2)"),
            ],
        )
        .state(
            "visibility-trigger",
            StateCondition::AttrEq("data-state", "visible"),
            vec![decl(
                "color",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        .state(
            "visibility-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-password-input-height", "1.5rem"),
                decl("--fandhe-password-input-padding-x", "0.25rem"),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-password-input-height", "2rem"),
                decl("--fandhe-password-input-padding-x", "0.5rem"),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-password-input-height", "2.5rem"),
                decl("--fandhe-password-input-padding-x", "0.75rem"),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-password-input-height", "3rem"),
                decl("--fandhe-password-input-padding-x", "1rem"),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-password-input-height", "3.5rem"),
                decl("--fandhe-password-input-padding-x", "1.25rem"),
                decl(
                    "--fandhe-password-input-font-size",
                    "var(--fandhe-font-font-size-xl)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled PasswordInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は
/// [`fandhe_frontend_headless_ui::password_input::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::password_input::{PasswordAutocomplete, PasswordInputProps};
/// use fandhe_frontend_pre_styled_ui::password_input;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let props = PasswordInputProps {
///     id: "login-password",
///     disabled: false,
///     invalid: false,
///     required: false,
///     autocomplete: PasswordAutocomplete::CurrentPassword,
/// };
/// let node = password_input::root(Size::Md, ColorPalette::Accent, false, &props, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="password-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    visible: bool,
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::password_input::root(visible, props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::password_input::PasswordInput;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    fn default_props(id: &str) -> PasswordInputProps<'_> {
        PasswordInputProps {
            id,
            disabled: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="password-input"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_to_invalid_and_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="control"][data-invalid] {
  border-color: var(--fandhe-color-danger);
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="control"][data-disabled] {
  cursor: not-allowed;
  opacity: 0.5;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_visibility_trigger_to_visible_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="password-input"][data-part="visibility-trigger"][data-state="visible"] {
  color: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-password-input--size-md"));
        assert!(html.contains("fd-password-input--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let props = default_props("pw");
        for (size, class) in [
            (Size::Xs, "fd-password-input--size-xs"),
            (Size::Sm, "fd-password-input--size-sm"),
            (Size::Md, "fd-password-input--size-md"),
            (Size::Lg, "fd-password-input--size-lg"),
            (Size::Xl, "fd-password-input--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                &props,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let props = default_props("pw");
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-password-input--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-password-input--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-password-input--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-password-input--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-password-input--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-password-input--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, &props, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let props = default_props("pw");
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &props,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let props = default_props("pw");
        let html = render(&label(
            &props,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_input_never_outputs_value_attribute() {
        let props = default_props("pw");
        let html = render(&input(false, &props, vec![]));
        assert!(!html.contains("value="));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_password_input_state_machine() {
        // `PasswordInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`PasswordInput` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ検証
        // する。
        let mut p = PasswordInput::default();
        assert!(!p.is_visible());

        let props = default_props("pw");
        let ssr_html = render(&p.root(&props, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="hidden""#));

        assert!(dispatch(&mut p, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-visible="visible""#));

        let restored = PasswordInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }
}
