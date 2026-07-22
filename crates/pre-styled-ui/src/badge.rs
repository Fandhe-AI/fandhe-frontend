//! Badge（イシュー #550）: 単一 recipe styled 部品。ステータス表示・ラベル
//! 装飾のための `<span>` を組み立てる。装飾的テキストであり、追加の
//! `role`/`aria-*` は付与しない（chakra-ui v3 準拠の最小サブセット）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="badge"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("badge");

/// Badge の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    /// 塗りつぶし。
    Solid,
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for BadgeVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Subtle => "subtle",
            Self::Outline => "outline",
        }
    }
}

/// [`badge`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct BadgeProps {
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: BadgeVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色から選択する。
    pub palette: ColorPalette,
}

impl Default for BadgeProps {
    fn default() -> Self {
        BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Md,
            palette: ColorPalette::Accent,
        }
    }
}

/// Badge の recipe（scope `"badge"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_declarations`] 経由の
/// `--fandhe-palette`/`--fandhe-palette-fg`（イシュー #606）を参照する
/// （[`crate::button::recipe`] の rustdoc 参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("badge", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("padding", "0.0625rem 0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("padding", "0.125rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("padding", "0.25rem 0.625rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            BadgeVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            BadgeVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            BadgeVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(BadgeVariant::Subtle)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
    ] {
        recipe = recipe.variant(palette, "root", palette_declarations(palette));
    }
    recipe
}

/// Badge の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Badge 1 個を組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
///
/// let node = badge(&BadgeProps::default(), vec![], vec![text("New")]);
/// assert!(render(&node).contains("New"));
/// ```
#[must_use]
pub fn badge<'a>(props: &BadgeProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_subtle_md() {
        let node = badge(&BadgeProps::default(), vec![], vec![text("New")]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<span data-scope="badge" data-part="root" class="fd-badge--size-md fd-badge--variant-subtle fd-badge--color-palette-accent">New</span>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (BadgeVariant::Solid, "fd-badge--variant-solid"),
            (BadgeVariant::Subtle, "fd-badge--variant-subtle"),
            (BadgeVariant::Outline, "fd-badge--variant-outline"),
        ] {
            let props = BadgeProps {
                variant,
                ..BadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-badge--size-md {class} fd-badge--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラスへ写像されることを
    /// 固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-badge--color-palette-accent"),
            (ColorPalette::Info, "fd-badge--color-palette-info"),
            (ColorPalette::Success, "fd-badge--color-palette-success"),
            (ColorPalette::Warning, "fd-badge--color-palette-warning"),
            (ColorPalette::Danger, "fd-badge--color-palette-danger"),
        ] {
            let props = BadgeProps {
                palette,
                ..BadgeProps::default()
            };
            let html = render(&badge(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-badge--size-md fd-badge--variant-subtle {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    /// イシュー #606: recipe の静的 CSS に radii トークン参照が含まれることを
    /// 固定する。
    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&badge(
            &BadgeProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&badge(
            &BadgeProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
