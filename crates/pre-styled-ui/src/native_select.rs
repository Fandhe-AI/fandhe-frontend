//! styled NativeSelect（イシュー #737、親 #736、祖父トラッキング #726）。
//!
//! [`crate::input`] と同型の薄い委譲層。
//! `fandhe_frontend_headless_ui::field::select`（#538/#602）が出力する
//! `data-scope="field"` `data-part="select"` へ `variant`/`size` variant
//! クラスと既定 CSS を重ねる。設計方針・状態機械を持たない理由・`field`
//! scope を共有する理由は [`crate::input`] rustdoc を参照（本モジュールは
//! 重複を避けるため差分のみ記す）。
//!
//! # ネイティブ矢印を維持する（`appearance: none` を使わない）設計判断
//!
//! chakra-ui の `NativeSelect` はカスタム矢印アイコン（`NativeSelect.Indicator`）
//! を重ねるため `appearance: none` でブラウザ既定の矢印を消す構成が一般的だが、
//! 本イシューは「ブラウザネイティブ挙動を尊重する」という本クレートの
//! 設計原則（indicator パーツはスコープ外、モジュール rustdoc・PR 本文
//! 参照）に従い、ネイティブの矢印・開閉挙動をそのまま残す最小サブセットと
//! する。`appearance` プロパティの宣言自体を持たない。
//!
//! # ネイティブ `readonly` を出力しない理由（headless 層への委譲）
//!
//! `<select readonly>` は HTML 仕様上無効な属性のため、headless
//! `field::select`（イシュー #602）はネイティブ `readonly` を出力しない
//! （`data-readonly` は他コントロール同様に出力する）。本モジュールは
//! この判断を再実装せず、そのまま委譲する。
//!
//! # `option` 子ノード
//!
//! `core` は `select` ショートカットタグを意図的に持たない
//! （`crates/core/src/tags.rs` 冒頭 doc 参照）ため、呼び出し側が
//! `fandhe_frontend_core::el("option", ..., ...)` で組み立てて `children` に
//! 渡す（headless `field::select` rustdoc と同じ契約）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

pub use fandhe_frontend_headless_ui::field::{FieldIds, FieldProps};

/// この styled NativeSelect が扱う slot。
const SLOTS: &[&str] = &["select"];

/// NativeSelect の見た目 variant（chakra-ui `NativeSelect` の `variant`
/// 相当。`Flushed` の代わりに枠なしの `Plain` を持つ点が
/// [`crate::input::InputVariant`]/[`crate::textarea::TextareaVariant`] との
/// 差異）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeSelectVariant {
    /// 枠線あり（既定）。
    #[default]
    Outline,
    /// 淡色背景・枠線なし。
    Subtle,
    /// 枠・背景なし（装飾なしの最小サブセット）。
    Plain,
}

impl VariantValue for NativeSelectVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Subtle => "subtle",
            Self::Plain => "plain",
        }
    }
}

/// [`native_select`] の見た目設定。
#[derive(Debug, Clone, Copy)]
pub struct NativeSelectProps {
    /// 見た目 variant（既定 `Outline`）。
    pub variant: NativeSelectVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for NativeSelectProps {
    fn default() -> Self {
        NativeSelectProps {
            variant: NativeSelectVariant::Outline,
            size: Size::Md,
        }
    }
}

/// この styled NativeSelect の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみ
/// が呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("field", SLOTS)
        .base(
            "select",
            vec![
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("font", "inherit"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("transition", "border-color 0.15s, background 0.15s"),
            ],
        )
        .state(
            "select",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "select",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "select",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .variant(
            Size::Xs,
            "select",
            vec![
                decl("padding", "0.125rem 0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "select",
            vec![
                decl("padding", "0.25rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Md,
            "select",
            vec![
                decl("padding", "0.375rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "select",
            vec![
                decl("padding", "0.5rem 1rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Xl,
            "select",
            vec![
                decl("padding", "0.625rem 1.25rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            NativeSelectVariant::Outline,
            "select",
            vec![decl("border", "1px solid var(--fandhe-color-border)")],
        )
        .variant(
            NativeSelectVariant::Subtle,
            "select",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border", "1px solid transparent"),
            ],
        )
        .variant(
            NativeSelectVariant::Plain,
            "select",
            vec![
                decl("background", "transparent"),
                decl("border", "1px solid transparent"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(NativeSelectVariant::Outline)
}

/// この styled NativeSelect が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `select` パーツを組み立てる。`variant`/`size` に応じたクラスを
/// 付与し（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）、アクセシビリティ配線は
/// [`fandhe_frontend_headless_ui::field::select`] へそのまま委譲する。
///
/// `children` は `fandhe_frontend_core::el("option", ..., ...)` で組み立てた
/// `<option>` 要素列を渡す（モジュール rustdoc「`option` 子ノード」参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, render, text};
/// use fandhe_frontend_pre_styled_ui::native_select::{
///     self, FieldIds, FieldProps, NativeSelectProps,
/// };
///
/// let field = FieldProps {
///     id: "country",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let option = el("option", vec![("value", "jp")], vec![text("Japan")]);
/// let node = native_select::native_select(
///     &NativeSelectProps::default(),
///     &field,
///     vec![],
///     vec![option],
/// );
/// assert!(render(&node).contains(r#"data-scope="field" data-part="select""#));
/// ```
#[must_use]
pub fn native_select<'a>(
    props: &NativeSelectProps,
    field: &FieldProps<'_>,
    extra_attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(extra_attrs));
    fandhe_frontend_headless_ui::field::select(field, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, render, text};

    fn default_field(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    fn option(value: &str, label: &str) -> Node {
        el("option", vec![("value", value)], vec![text(label)])
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="field"][data-part="select"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_never_declares_appearance() {
        // ネイティブ矢印を維持する設計判断（モジュール rustdoc 参照）の回帰。
        let out = css();
        assert!(!out.contains("appearance"));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![option("jp", "Japan")],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="select""#));
        assert!(html.contains(r#"<option value="jp">Japan</option>"#));
    }

    #[test]
    fn default_variant_is_outline_and_md() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-field--variant-outline"));
        assert!(html.contains("fd-field--size-md"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (NativeSelectVariant::Outline, "fd-field--variant-outline"),
            (NativeSelectVariant::Subtle, "fd-field--variant-subtle"),
            (NativeSelectVariant::Plain, "fd-field--variant-plain"),
        ] {
            let field = default_field("f");
            let props = NativeSelectProps {
                variant,
                ..NativeSelectProps::default()
            };
            let html = render(&native_select(&props, &field, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-field--size-xs"),
            (Size::Sm, "fd-field--size-sm"),
            (Size::Md, "fd-field--size-md"),
            (Size::Lg, "fd-field--size-lg"),
            (Size::Xl, "fd-field--size-xl"),
        ] {
            let field = default_field("f");
            let props = NativeSelectProps {
                size,
                ..NativeSelectProps::default()
            };
            let html = render(&native_select(&props, &field, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn readonly_does_not_emit_native_attribute_but_keeps_data_readonly() {
        let mut field = default_field("f");
        field.readonly = true;
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(!html.contains(r#" readonly="""#));
        assert!(html.contains(r#"data-readonly=""#));
    }

    #[test]
    fn invalid_and_disabled_flags_propagate_from_field_props() {
        let mut field = default_field("f");
        field.invalid = true;
        field.disabled = true;
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid=""#));
        assert!(html.contains(r#"data-disabled=""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"disabled=""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn option_children_text_payload_is_escaped_on_render() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![],
            vec![option("x", "<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn extra_attrs_attribute_breakout_payload_is_escaped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let field = default_field("f");
        let html = render(&native_select(
            &NativeSelectProps::default(),
            &field,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="select""#));
        assert!(!html.contains("attacker"));
    }
}
