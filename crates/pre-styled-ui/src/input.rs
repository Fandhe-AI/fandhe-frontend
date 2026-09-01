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
//!
//! # 参考サイト基準への調整（イシュー #1482）
//!
//! chakra-ui v3 Input / Radix Themes text-field と視覚比較し、Phase 0
//! で確定した共通基盤（[`crate::recipe::focus_ring_declarations`]・
//! [`crate::recipe::disabled_declarations`]・
//! [`crate::recipe::transition_declarations`]・#1678 の
//! `--fandhe-size-control-height/padding-x/font-size-*` トークン）へ
//! 移行した。date-input #1469（[`crate::date_input`]）・button #1447
//! （[`crate::button`]）と同型。
//!
//! - **hover（意図的非採用）**: hover 背景は付与しない。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` の判定基準
//!   （hover はインタラクティブ slot = `cursor: pointer` を持つ slot のみ）
//!   に対しテキスト入力は `cursor: text` であり対象外。chakra v3 Input・
//!   Radix Themes text-field も hover 背景変化を持たない。
//! - **readonly（意図的非採用）**: `data-readonly` への視覚宣言は追加しない。
//!   ネイティブ `<input readonly>` は選択・キャレット操作が可能なため
//!   テキストカーソル（既定の `cursor: text`）のままが適切であり、参照
//!   3 サイトも readonly の独自装飾を持たない（[`crate::date_input`] の
//!   `segment` へ付けた `cursor: default` は非ネイティブ `<span>` セグメント
//!   固有の事情であり、ネイティブ `<input>` である本パーツには適用しない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, transition_declarations, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless `field` の `FieldProps`/`FieldIds`/`error_text` を再エクスポートし、
// 呼び出し側が `fandhe-frontend-pre-styled-ui` 単独依存でアクセシビリティ
// props と、`invalid` 時に `field::input` の `aria-describedby` が参照する
// error id を持つ `error_text` パーツを組み立てられるようにする（#685 の
// エスケープハッチと同型の判断）。`error_text` 自体は見た目を持たない
// headless パーツであり、本モジュールは薄い再エクスポートのみを担う
// （二重実装を作らない）。
pub use fandhe_frontend_headless_ui::field::{error_text, FieldIds, FieldProps};

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
    let mut base = vec![
        decl("box-sizing", "border-box"),
        decl("width", "100%"),
        decl("font", "inherit"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("background", "var(--fandhe-color-bg)"),
        // date-input #1469・button #1447 が確立した Forms 家族の標準角丸
        // （旧 `--fandhe-radius-sm` から変更、イシュー #1482）。
        decl("border-radius", "var(--fandhe-radius-md)"),
    ];
    base.extend(transition_declarations(
        "border-color, background",
        MotionDuration::Fast,
    ));

    SlotRecipe::new("field", SLOTS)
        .base("input", base)
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
            disabled_declarations(),
        )
        .state(
            "input",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // size（イシュー #1678 の `--fandhe-size-control-height/padding-x/
        // font-size-*` トークンへ移行、イシュー #1482）。固定高を持つ
        // chakra v3 Input（h-8〜h-12）に合わせ、縦 padding は廃止して
        // `height` + 水平 padding のみで表現する。
        .variant(
            Size::Xs,
            "input",
            vec![
                decl("height", "var(--fandhe-size-control-height-xs, 2rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-xs, 0.625rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "input",
            vec![
                decl("height", "var(--fandhe-size-control-height-sm, 2.25rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Md,
            "input",
            vec![
                decl("height", "var(--fandhe-size-control-height-md, 2.5rem)"),
                decl("padding", "0 var(--fandhe-size-control-padding-x-md, 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "input",
            vec![
                decl("height", "var(--fandhe-size-control-height-lg, 2.75rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-lg, 1.25rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "input",
            vec![
                decl("height", "var(--fandhe-size-control-height-xl, 3rem)"),
                decl(
                    "padding",
                    "0 var(--fandhe-size-control-padding-x-xl, 1.5rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl))",
                ),
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
    fn stylesheet_uses_canonical_focus_ring_declarations() {
        // イシュー #1482: focus ring がリテラル値ではなく canonical ヘルパ
        // （`focus_ring_declarations`）由来のトークン参照であることを固定。
        let out = css();
        assert!(out.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(out.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn stylesheet_uses_motion_token_transition() {
        // イシュー #1482: transition がリテラル秒数ではなく motion トークン
        // （`transition_declarations`）由来であることを固定。
        let out = css();
        assert!(out.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(out.contains("transition-property: border-color, background;"));
    }

    #[test]
    fn stylesheet_size_variants_use_control_tokens() {
        // イシュー #1482: 各 size が #1678 の control トークンへ移行した
        // ことを固定（button #1447 と同型の 3 点セット）。
        let out = css();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                out.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "height token missing for {suffix} -> {out}"
            );
            assert!(
                out.contains(&format!("--fandhe-size-control-padding-x-{suffix}")),
                "padding-x token missing for {suffix} -> {out}"
            );
            assert!(
                out.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "font-size token missing for {suffix} -> {out}"
            );
        }
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
            (Size::Xs, "fd-field--size-xs"),
            (Size::Sm, "fd-field--size-sm"),
            (Size::Md, "fd-field--size-md"),
            (Size::Lg, "fd-field--size-lg"),
            (Size::Xl, "fd-field--size-xl"),
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
    fn invalid_input_describedby_target_id_is_rendered_by_error_text() {
        // showcase.rs（docs-site）の invalid デモが `field::input` の
        // `aria-describedby` 参照先へ `field::error_text` を併設する構成
        // （PR #783 Bugbot 指摘の再発防止）。再エクスポートした `error_text`
        // が `input` と同じ `FieldProps` から一貫した id を導出することを
        // 固定する。
        let mut field = default_field("f");
        field.invalid = true;
        let input_html = render(&input(&InputProps::default(), &field, vec![]));
        assert!(input_html.contains(r#"aria-describedby="f-error-text""#));
        let error_html = render(&error_text(&field, vec![], vec![]));
        assert!(error_html.contains(r#"id="f-error-text""#));
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
