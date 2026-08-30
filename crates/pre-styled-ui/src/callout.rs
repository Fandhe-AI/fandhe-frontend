//! Callout（イシュー #994）: 単一 recipe styled 部品。本文フロー中に置く
//! 補足情報を強調表示するための root/icon/text の 3 パーツ構成。
//!
//! [`crate::alert`] との責務差（本モジュールの中核的な設計判断）:
//!
//! - `alert` は `role="alert"`（WAI-ARIA live region）を全ステータス固定で
//!   付与する「ユーザーの操作に割り込んで伝えるべき通知」向けの部品。
//! - `callout` は本文中に静的に置かれる補足情報であり、live region ではない。
//!   そのため [`root`] は `role` を一切付与せず、支援技術への割り込み通知を
//!   発生させない。強調の意味づけは `color-palette` 軸（[`crate::recipe::ColorPalette`]）
//!   のみで表現し、`alert` のような緊急度の意味論（info/success/warning/error）
//!   は持たない。
//! - 緊急度の低い状態表示という点では `status`（未実装）とも近いが、`callout`
//!   はあくまで「本文の一部としての補足」であり、通知の生成・消去といった
//!   ライフサイクルを持たない静的な装飾部品である点で異なる。
//!
//! `icon` パーツは装飾要素であり、固有の `role`/`aria-*` は付与しない
//! （`alert::indicator` と同型）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="callout"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("callout");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "icon", "text"];

/// Callout の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalloutVariant {
    /// 淡色背景（既定）。
    #[default]
    Soft,
    /// 淡色背景 + 枠線。
    Surface,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for CalloutVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Surface => "surface",
            Self::Outline => "outline",
        }
    }
}

/// [`root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct CalloutProps {
    /// 見た目 variant（既定 `Soft`）。
    pub variant: CalloutVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606 と同じ名前空間）。
    pub palette: ColorPalette,
}

impl Default for CalloutProps {
    fn default() -> Self {
        CalloutProps {
            variant: CalloutVariant::Soft,
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// Callout の recipe（scope `"callout"`、[`SLOTS`] の 3 パーツ）。
///
/// axis 登録順を size → variant → color-palette に固定する（[`crate::badge`]
/// の recipe と同型）。[`SlotRecipe::variant_classes`] は axis の登録順で
/// クラスを連結するため、この順序が既定出力
/// `"fd-callout--size-md fd-callout--variant-soft fd-callout--color-palette-accent"`
/// を決定する。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("callout", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("gap", "0.75rem"),
                decl("border-radius", "var(--fandhe-radius-md)"),
            ],
        )
        .base("icon", vec![decl("flex-shrink", "0")])
        .base(
            "text",
            vec![
                decl("min-width", "0"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        // イシュー #1681: Xs は padding 0.25rem 刻みの等差進行を外挿。
        // font-size はトークン下限 xs を Sm と共有する。
        .variant(Size::Xs, "root", vec![decl("padding", "0.25rem 0.5rem")])
        .variant(
            Size::Xs,
            "text",
            vec![decl("font-size", "var(--fandhe-font-font-size-xs)")],
        )
        .variant(Size::Sm, "root", vec![decl("padding", "0.5rem 0.75rem")])
        .variant(
            Size::Sm,
            "text",
            vec![decl("font-size", "var(--fandhe-font-font-size-xs)")],
        )
        .variant(Size::Md, "root", vec![decl("padding", "0.75rem 1rem")])
        .variant(
            Size::Md,
            "text",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .variant(Size::Lg, "root", vec![decl("padding", "1rem 1.25rem")])
        .variant(
            Size::Lg,
            "text",
            vec![decl("font-size", "var(--fandhe-font-font-size-md)")],
        )
        .variant(Size::Xl, "root", vec![decl("padding", "1.25rem 1.5rem")])
        .variant(
            Size::Xl,
            "text",
            vec![decl("font-size", "var(--fandhe-font-font-size-lg)")],
        )
        .variant(
            CalloutVariant::Soft,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            CalloutVariant::Surface,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .variant(
            CalloutVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-palette)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(CalloutVariant::Soft)
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

/// Callout の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`role`/`aria-*` は一切付与しない（module doc
/// 参照。`alert::root` と異なり live region ではないため）。呼び出し側の
/// `class` は [`drop_class_attr`] で除去してから recipe クラスを合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::callout::{root, CalloutProps};
///
/// let node = root(&CalloutProps::default(), vec![], vec![]);
/// let html = render(&node);
/// assert!(!html.contains("role="));
/// assert!(html.contains("fd-callout--variant-soft"));
/// ```
#[must_use]
pub fn root<'a>(props: &CalloutProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// icon パーツ（`<span>`。装飾要素、固有の `role`/`aria-*` は付与しない）を
/// 組み立てる。
#[must_use]
pub fn icon<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("icon", "span", attrs, children)
}

/// text パーツ（`<div>`。補足情報の本文）を組み立てる。
///
/// `size` は [`root`] に渡したものと同じ値を渡す（[`recipe`] が
/// `text` slot 用に登録するフォントサイズ variant は
/// `[data-scope="callout"][data-part="text"].fd-callout--size-*` という
/// text 要素自身へのクラス付与を前提とした複合セレクタのため、`root` に
/// クラスを付けるだけでは text 要素のフォントサイズは変化しない。
/// 呼び出し側が両者へ同じ `size` を渡すことで整合を保つ）。
#[must_use]
pub fn text<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let class = recipe().variant_class(size);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("text", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as text_node};

    #[test]
    fn default_props_render_soft_md_accent_without_role() {
        let html = render(&root(&CalloutProps::default(), vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="callout" data-part="root" class="fd-callout--size-md fd-callout--variant-soft fd-callout--color-palette-accent"></div>"#
        );
        assert!(!html.contains("role="));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (CalloutVariant::Soft, "fd-callout--variant-soft"),
            (CalloutVariant::Surface, "fd-callout--variant-surface"),
            (CalloutVariant::Outline, "fd-callout--variant-outline"),
        ] {
            let props = CalloutProps {
                variant,
                ..CalloutProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-callout--size-md {class} fd-callout--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-callout--size-sm"),
            (Size::Md, "fd-callout--size-md"),
            (Size::Lg, "fd-callout--size-lg"),
        ] {
            let props = CalloutProps {
                size,
                ..CalloutProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-callout--variant-soft fd-callout--color-palette-accent\""
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-callout--color-palette-accent"),
            (ColorPalette::Info, "fd-callout--color-palette-info"),
            (ColorPalette::Success, "fd-callout--color-palette-success"),
            (ColorPalette::Warning, "fd-callout--color-palette-warning"),
            (ColorPalette::Danger, "fd-callout--color-palette-danger"),
            (ColorPalette::Neutral, "fd-callout--color-palette-neutral"),
        ] {
            let props = CalloutProps {
                palette,
                ..CalloutProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-callout--size-md fd-callout--variant-soft {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&icon(vec![], vec![]))
            .starts_with(r#"<span data-scope="callout" data-part="icon""#));
        assert!(render(&text(Size::Md, vec![], vec![]))
            .starts_with(r#"<div data-scope="callout" data-part="text""#));
    }

    #[test]
    fn text_size_variant_maps_to_expected_class() {
        for (size, class) in [
            (Size::Sm, "fd-callout--size-sm"),
            (Size::Md, "fd-callout--size-md"),
            (Size::Lg, "fd-callout--size-lg"),
        ] {
            let html = render(&text(size, vec![], vec![]));
            assert!(
                html.contains(&format!(r#"class="{class}""#)),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn composed_callout_snapshot() {
        let node = root(
            &CalloutProps::default(),
            vec![],
            vec![
                icon(vec![], vec![]),
                text(
                    Size::Md,
                    vec![],
                    vec![text_node("Heads up: this is supplementary info")],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="callout" data-part="root" class="fd-callout--size-md fd-callout--variant-soft fd-callout--color-palette-accent">"#,
                r#"<span data-scope="callout" data-part="icon"></span>"#,
                r#"<div data-scope="callout" data-part="text" class="fd-callout--size-md">Heads up: this is supplementary info</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &CalloutProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_text_children_is_escaped() {
        let html = render(&text(
            Size::Md,
            vec![],
            vec![text_node("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let payload = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            &CalloutProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));

        let html = render(&icon(vec![("data-testid", payload)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));

        let html = render(&text(Size::Md, vec![("data-testid", payload)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }
}
