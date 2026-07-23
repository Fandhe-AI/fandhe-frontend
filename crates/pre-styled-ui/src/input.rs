//! styled Input（イシュー #737、親 #736、祖父トラッキング #726）。
//!
//! `fandhe_frontend_headless_ui::field::input`（#538/#602）が出力する
//! `data-scope="field"` `data-part="input"` の 1 パーツへ、`variant`/`size`
//! variant クラスと既定 CSS を重ねる薄い委譲層である。
//!
//! # 状態機械を持たない理由
//!
//! ark-ui/chakra-ui の `Input` はブラウザネイティブの `<input>` 挙動
//! （フォーカス・入力・フォーム送信）をそのまま尊重する部品であり、
//! headless [`fandhe_frontend_headless_ui::field`] 自身も「props から決定的に
//! マークアップを組み立てる純粋関数群」（状態機械なし、`field.rs` モジュール
//! doc 参照）として実装されている。本モジュールもその設計をそのまま継承し、
//! [`crate::checkbox`]/[`crate::switch`] のような開閉・選択状態を持たない。
//!
//! # `field` scope を共有する理由（recipe scope の設計判断）
//!
//! [`crate::recipe::SlotRecipe`] が生成する CSS セレクタは
//! `[data-scope="<scope>"][data-part="<slot>"]` 固定であり、headless
//! `field::input` が実際にレンダリングする `data-scope="field"` と一致させる
//! 必要がある。そのため本モジュールの recipe scope は独自の `"input"` では
//! なく `"field"` とし、slot を `"input"` のみ宣言する（[`crate::textarea`]/
//! [`crate::native_select`] も同じ scope を共有するが、slot が相互排他
//! （`"input"`/`"textarea"`/`"select"`）なのでセレクタ・宣言は衝突しない）。
//!
//! アクセシビリティ配線（`id`・ネイティブ `disabled`/`required`/`readonly`・
//! `aria-invalid`・`aria-describedby`・`data-*` フラグ）は headless
//! `field::input` へすべて委譲し、本モジュールは見た目（variant クラス・
//! 既定 CSS）の登録のみを担う。二重実装によるドリフトを作らない。
//!
//! `color-palette` 軸は提供しない（[`crate::lib`] 「複合部品の variant
//! 統一方針」§3 参照: palette は選択・チェック状態を示す部品向けで、フォーム
//! 入力はアクセントを focus ring のトークン参照でのみ使う）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless `field` の `FieldProps`/`FieldIds` を再エクスポートし、呼び出し側が
// `fandhe-frontend-pre-styled-ui` 単独依存でアクセシビリティ props を組み立て
// られるようにする（#685 のエスケープハッチと同型の判断）。
pub use fandhe_frontend_headless_ui::field::{FieldIds, FieldProps};

/// この styled Input が扱う slot（[`crate::recipe::SlotRecipe::new`] の
/// `slots` 引数、モジュール rustdoc「`field` scope を共有する理由」参照）。
const SLOTS: &[&str] = &["input"];

/// Input の見た目 variant（chakra-ui `Input` の `variant` 相当）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputVariant {
    /// 枠線あり（既定）。
    #[default]
    Outline,
    /// 淡色背景・枠線なし。
    Subtle,
    /// 下線のみ。
    Flushed,
}

impl VariantValue for InputVariant {
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

/// [`input`] の見た目設定（アクセシビリティ props は別引数 [`FieldProps`] で
/// 渡す。ark-ui/chakra-ui が見た目 props とフォーム状態 props を分離する
/// 構成に合わせる）。
#[derive(Debug, Clone, Copy)]
pub struct InputProps {
    /// 見た目 variant（既定 `Outline`）。
    pub variant: InputVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for InputProps {
    fn default() -> Self {
        InputProps {
            variant: InputVariant::Outline,
            size: Size::Md,
        }
    }
}

/// この styled Input の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("field", SLOTS)
        .base(
            "input",
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
        // イシュー #737 受け入れ条件: invalid/disabled/focus-visible の
        // 視覚状態は headless `field::input` が出力する data-* 存在属性・
        // 実フォーカスへそのまま連動させる（checkbox control と同型の視覚
        // 言語、モジュール rustdoc 参照）。
        .state(
            "input",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "input",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .variant(
            Size::Sm,
            "input",
            vec![
                decl("padding", "0.25rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Md,
            "input",
            vec![
                decl("padding", "0.375rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "input",
            vec![
                decl("padding", "0.5rem 1rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            InputVariant::Outline,
            "input",
            vec![decl("border", "1px solid var(--fandhe-color-border)")],
        )
        .variant(
            InputVariant::Subtle,
            "input",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border", "1px solid transparent"),
            ],
        )
        .variant(
            InputVariant::Flushed,
            "input",
            vec![
                decl("border", "0"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(InputVariant::Outline)
}

/// この styled Input が生成する静的 CSS 全量を返す（決定的。
/// [`crate::checkbox::stylesheet`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `input` パーツを組み立てる。`variant`/`size` に応じたクラスを
/// 付与し（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）、アクセシビリティ配線は
/// [`fandhe_frontend_headless_ui::field::input`] へそのまま委譲する。
///
/// `extra_attrs` には `type`/`name`/`value`/`placeholder` 等、呼び出し側が
/// 必要とする追加属性を渡す。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
///
/// let field = FieldProps {
///     id: "email",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let node = input::input(&InputProps::default(), &field, vec![("type", "email")]);
/// assert!(render(&node).contains(r#"data-scope="field" data-part="input""#));
/// ```
#[must_use]
pub fn input<'a>(
    props: &InputProps,
    field: &FieldProps<'_>,
    extra_attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(extra_attrs));
    fandhe_frontend_headless_ui::field::input(field, merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

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
        assert!(a.contains(r#"[data-scope="field"][data-part="input"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_links_input_to_invalid_disabled_and_focus_visible() {
        let out = css();
        assert!(out.contains(r#"[data-scope="field"][data-part="input"][data-invalid] {"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="input"][data-disabled] {"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="input"]:focus-visible {"#));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let field = default_field("f");
        let html = render(&input(&InputProps::default(), &field, vec![]));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="input""#));
    }

    #[test]
    fn default_variant_is_outline_and_md() {
        let field = default_field("f");
        let html = render(&input(&InputProps::default(), &field, vec![]));
        assert!(html.contains("fd-field--variant-outline"));
        assert!(html.contains("fd-field--size-md"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (InputVariant::Outline, "fd-field--variant-outline"),
            (InputVariant::Subtle, "fd-field--variant-subtle"),
            (InputVariant::Flushed, "fd-field--variant-flushed"),
        ] {
            let field = default_field("f");
            let props = InputProps {
                variant,
                ..InputProps::default()
            };
            let html = render(&input(&props, &field, vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-field--size-sm"),
            (Size::Md, "fd-field--size-md"),
            (Size::Lg, "fd-field--size-lg"),
        ] {
            let field = default_field("f");
            let props = InputProps {
                size,
                ..InputProps::default()
            };
            let html = render(&input(&props, &field, vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn invalid_and_disabled_flags_propagate_from_field_props() {
        let mut field = default_field("f");
        field.invalid = true;
        field.disabled = true;
        let html = render(&input(&InputProps::default(), &field, vec![]));
        assert!(html.contains(r#"data-invalid=""#));
        assert!(html.contains(r#"data-disabled=""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"disabled=""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let field = default_field("f");
        let html = render(&input(
            &InputProps::default(),
            &field,
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn extra_attrs_attribute_breakout_payload_is_escaped() {
        let field = default_field("f");
        let html = render(&input(
            &InputProps::default(),
            &field,
            vec![("value", "\" onmouseover=\"alert(1)")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_field_props_id_payload_is_escaped_on_render() {
        let payload_id = "x\" onmouseover=\"alert(1)";
        let field = default_field(payload_id);
        let html = render(&input(&InputProps::default(), &field, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let field = default_field("f");
        let html = render(&input(
            &InputProps::default(),
            &field,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
        ));
        assert!(html.contains(r#"data-scope="field""#));
        assert!(html.contains(r#"data-part="input""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn xss_payload_in_children_context_placeholder_value_is_escaped() {
        let field = default_field("f");
        let html = render(&input(
            &InputProps::default(),
            &field,
            vec![("placeholder", "<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
