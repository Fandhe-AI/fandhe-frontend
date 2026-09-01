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
//!
//! # 参考サイト基準への調整（イシュー #1511）
//!
//! chakra-ui v3 Textarea / Radix Themes text-area と視覚比較し、Phase 0
//! で確定した共通基盤（[`crate::recipe::focus_ring_declarations`]・
//! [`crate::recipe::disabled_declarations`]・
//! [`crate::recipe::transition_declarations`]・#1678 の
//! `--fandhe-size-control-padding-x/font-size-*` トークン）へ移行した。
//! [`crate::input`]（イシュー #1482）の差分を踏襲するが、以下の点で
//! textarea 固有の事情により差分がある。
//!
//! - **固定 `height` を採らない（意図的差分）**: [`crate::input`] は
//!   chakra v3 Input の固定高（h-8〜h-12）に合わせ `height` +
//!   水平 padding のみで表現するが、`textarea` は複数行部品であり
//!   `rows` 属性・内容量に応じて高さが伸縮するのが自然な挙動である。
//!   chakra v3 Textarea・Radix Themes text-area のいずれも固定高を
//!   宣言せず、縦 padding は入力欄と同じ理由で高さ計算に含める。本
//!   モジュールも `--fandhe-size-control-height-*` トークンは使わず、
//!   既存の縦 padding（rem 固定値、参照サイト比較で概ね妥当と判断し
//!   維持）+ `--fandhe-size-control-padding-x-*`（水平のみ）+
//!   `--fandhe-size-control-font-size-*` で表現する。
//! - **フォーカス・トランジション・disabled・角丸**: [`crate::input`] と
//!   同型（canonical ヘルパへの移行）。
//! - **hover（意図的非採用）**: hover 背景は付与しない。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` の判定基準
//!   （hover はインタラクティブ slot = `cursor: pointer` を持つ slot のみ）
//!   に対しテキスト入力は `cursor: text` であり対象外。chakra v3
//!   Textarea・Radix Themes text-area も hover 背景変化を持たない。
//! - **readonly（意図的非採用）**: `data-readonly` への視覚宣言は追加しない。
//!   ネイティブ `<textarea readonly>` は選択・キャレット操作可能なため
//!   [`crate::input`] と同じ判断（参照サイトも readonly の独自装飾を
//!   持たない）。
//! - **size / variant の網羅性**: 既存の xs〜xl 5 段・outline/subtle/
//!   flushed 3 variant を維持し、参照サイト名（chakra の solid/surface/
//!   ghost、Radix の classic/soft 等）は持ち込まない（本リポジトリ既存
//!   語彙優先）。
//! - **placeholder 色（意図的非採用）**: [`crate::recipe::SlotRecipe`] の
//!   [`crate::recipe::StateCondition`] に `::placeholder` 経路がなく、
//!   本イシューの対応範囲では生セレクタ経路を新設しない既存設計を維持
//!   する（recipe 基盤の拡張は本イシューのスコープ外）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, transition_declarations, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
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
    let mut base = vec![
        decl("box-sizing", "border-box"),
        decl("width", "100%"),
        decl("font", "inherit"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("background", "var(--fandhe-color-bg)"),
        // input #1482・date-input #1469・button #1447 が確立した Forms
        // 家族の標準角丸（旧 `--fandhe-radius-sm` から変更、イシュー #1511）。
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("resize", "vertical"),
    ];
    base.extend(transition_declarations(
        "border-color, background",
        MotionDuration::Fast,
    ));

    SlotRecipe::new("field", SLOTS)
        .base("textarea", base)
        .state(
            "textarea",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "textarea",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "textarea",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
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
        // size（イシュー #1678 の `--fandhe-size-control-padding-x/
        // font-size-*` トークンへ移行、イシュー #1511）。複数行部品のため
        // input と異なり `height` トークンは使わず、縦 padding は既存の
        // rem 固定値を維持する（モジュール rustdoc「固定 `height` を
        // 採らない」節参照）。
        .variant(
            Size::Xs,
            "textarea",
            vec![
                decl(
                    "padding",
                    "0.125rem var(--fandhe-size-control-padding-x-xs, 0.625rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "textarea",
            vec![
                decl(
                    "padding",
                    "0.25rem var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Md,
            "textarea",
            vec![
                decl(
                    "padding",
                    "0.375rem var(--fandhe-size-control-padding-x-md, 1rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "textarea",
            vec![
                decl(
                    "padding",
                    "0.5rem var(--fandhe-size-control-padding-x-lg, 1.25rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "textarea",
            vec![
                decl(
                    "padding",
                    "0.625rem var(--fandhe-size-control-padding-x-xl, 1.5rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-lg))",
                ),
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
    fn stylesheet_uses_canonical_focus_ring_declarations() {
        // イシュー #1511: focus ring がリテラル値ではなく canonical ヘルパ
        // （`focus_ring_declarations`）由来のトークン参照であることを固定。
        let out = css();
        assert!(out.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(out.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn stylesheet_uses_motion_token_transition() {
        // イシュー #1511: transition がリテラル秒数ではなく motion トークン
        // （`transition_declarations`）由来であることを固定。
        let out = css();
        assert!(out.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(out.contains("transition-property: border-color, background;"));
    }

    #[test]
    fn stylesheet_size_variants_use_control_tokens() {
        // イシュー #1511: 各 size が #1678 の control トークン（padding-x/
        // font-size のみ、複数行部品のため height は使わない）へ移行した
        // ことを固定。
        let out = css();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                out.contains(&format!("--fandhe-size-control-padding-x-{suffix}")),
                "padding-x token missing for {suffix} -> {out}"
            );
            assert!(
                out.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "font-size token missing for {suffix} -> {out}"
            );
            assert!(
                !out.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "height token unexpectedly present for {suffix} (textarea is multi-line, no fixed height) -> {out}"
            );
        }
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
