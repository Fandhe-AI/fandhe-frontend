//! styled Textarea（イシュー #737、親 #736、祖父トラッキング #726）。
//!
//! [`crate::input`] と同型の薄い委譲層。
//! `fandhe_frontend_headless_ui::field::textarea`（#538/#602）が出力する
//! `data-scope="field"` `data-part="textarea"` へ `variant`/`size` variant
//! クラスと既定 CSS を重ねる。設計方針・状態機械を持たない理由・`field`
//! scope を共有する理由は [`crate::input`] rustdoc を参照（本モジュールは
//! 重複を避けるため差分のみ記す）。
//!
//! # `autoresize` フック（headless 宣言的属性への styled 側の応答）
//!
//! headless `field::textarea` の `autoresize: bool` 引数は SSR 時点で
//! `data-autoresize=""` 存在属性のみを出力する宣言的フックであり、実際の
//! 高さ調整は CSR/wasm 層またはスタイルの責務（`crates/headless-ui/src/field.rs`
//! rustdoc 参照）。本モジュールは `[data-autoresize]` 状態規則として
//! `field-sizing: content` + `resize: none` を登録し、この宣言的フックへ
//! styled 層として応答する。`autoresize` が `false` のときは通常どおり
//! `resize: vertical`（base 規則）のみが効く。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

pub use fandhe_frontend_headless_ui::field::{FieldIds, FieldProps};

/// この styled Textarea が扱う slot。
const SLOTS: &[&str] = &["textarea"];

/// Textarea の見た目 variant（[`crate::input::InputVariant`] と同じ語彙）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextareaVariant {
    /// 枠線あり（既定）。
    #[default]
    Outline,
    /// 淡色背景・枠線なし。
    Subtle,
    /// 下線のみ。
    Flushed,
}

impl VariantValue for TextareaVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Subtle => "subtle",
            Self::Flushed => "flushed",
        }
    }
}

/// [`textarea`] の見た目設定。
#[derive(Debug, Clone, Copy)]
pub struct TextareaProps {
    /// 見た目 variant（既定 `Outline`）。
    pub variant: TextareaVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for TextareaProps {
    fn default() -> Self {
        TextareaProps {
            variant: TextareaVariant::Outline,
            size: Size::Md,
        }
    }
}

/// この styled Textarea の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("field", SLOTS)
        .base(
            "textarea",
            vec![
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("font", "inherit"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("resize", "vertical"),
                decl("transition", "border-color 0.15s, background 0.15s"),
            ],
        )
        .state(
            "textarea",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "textarea",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "textarea",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // headless `autoresize` フック（モジュール rustdoc「`autoresize` フック」
        // 節参照）への styled 側の応答。`field-sizing` は対応ブラウザでのみ
        // 効き、非対応ブラウザでは `resize: none` のみが効いて base の
        // `resize: vertical` を上書きする（グレースフルデグレード）。
        .state(
            "textarea",
            StateCondition::Attr("data-autoresize"),
            vec![decl("field-sizing", "content"), decl("resize", "none")],
        )
        .variant(
            Size::Xs,
            "textarea",
            vec![
                decl("padding", "0.125rem 0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "textarea",
            vec![
                decl("padding", "0.25rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Md,
            "textarea",
            vec![
                decl("padding", "0.375rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "textarea",
            vec![
                decl("padding", "0.5rem 1rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Xl,
            "textarea",
            vec![
                decl("padding", "0.625rem 1.25rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            TextareaVariant::Outline,
            "textarea",
            vec![decl("border", "1px solid var(--fandhe-color-border)")],
        )
        .variant(
            TextareaVariant::Subtle,
            "textarea",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border", "1px solid transparent"),
            ],
        )
        .variant(
            TextareaVariant::Flushed,
            "textarea",
            vec![
                decl("border", "0"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(TextareaVariant::Outline)
}

/// この styled Textarea が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `textarea` パーツを組み立てる。`variant`/`size` に応じたクラスを
/// 付与し（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）、アクセシビリティ配線・`autoresize` フックは
/// [`fandhe_frontend_headless_ui::field::textarea`] へそのまま委譲する。
///
/// `children` はテキストコンテンツ（`<textarea>` の初期値、`fandhe_frontend_core::text`
/// 経由で既定エスケープされる）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};
///
/// let field = FieldProps {
///     id: "bio",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let node = textarea::textarea(
///     &TextareaProps::default(),
///     &field,
///     true,
///     vec![],
///     vec![text("hello")],
/// );
/// assert!(render(&node).contains(r#"data-scope="field" data-part="textarea""#));
/// ```
#[must_use]
pub fn textarea<'a>(
    props: &TextareaProps,
    field: &FieldProps<'_>,
    autoresize: bool,
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
    fandhe_frontend_headless_ui::field::textarea(field, autoresize, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

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

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="field"][data-part="textarea"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_links_autoresize_to_field_sizing() {
        let out = css();
        assert!(out.contains(
            r#"[data-scope="field"][data-part="textarea"][data-autoresize] {
  field-sizing: content;
  resize: none;
}"#
        ));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="textarea""#));
    }

    #[test]
    fn default_variant_is_outline_and_md() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-field--variant-outline"));
        assert!(html.contains("fd-field--size-md"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TextareaVariant::Outline, "fd-field--variant-outline"),
            (TextareaVariant::Subtle, "fd-field--variant-subtle"),
            (TextareaVariant::Flushed, "fd-field--variant-flushed"),
        ] {
            let field = default_field("f");
            let props = TextareaProps {
                variant,
                ..TextareaProps::default()
            };
            let html = render(&textarea(&props, &field, false, vec![], vec![]));
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
            let props = TextareaProps {
                size,
                ..TextareaProps::default()
            };
            let html = render(&textarea(&props, &field, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn autoresize_true_emits_data_autoresize_attribute() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            true,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-autoresize=""#));
    }

    #[test]
    fn autoresize_false_omits_data_autoresize_attribute() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-autoresize"));
    }

    #[test]
    fn invalid_and_disabled_flags_propagate_from_field_props() {
        let mut field = default_field("f");
        field.invalid = true;
        field.disabled = true;
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
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
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn children_text_payload_is_escaped_on_render() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn extra_attrs_attribute_breakout_payload_is_escaped() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![("placeholder", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let field = default_field("f");
        let html = render(&textarea(
            &TextareaProps::default(),
            &field,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="textarea""#));
        assert!(!html.contains("attacker"));
    }
}
