//! Mark（イシュー #771）: 単一 recipe styled 部品。テキストハイライト
//! （`<mark>`）を `variant`/`colorPalette` の 2 軸で組み立てる
//! （[`crate::badge`] と同型の単一 recipe パターン）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="mark"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("mark");

/// Mark の見た目 variant（chakra-ui の Mark variant 準拠）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkVariant {
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 塗りつぶし。
    Solid,
    /// 背景なし・文字色のみ。
    Text,
    /// 装飾なし（ブラウザ既定の `<mark>` 表示をリセットした素の状態）。
    Plain,
}

impl VariantValue for MarkVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Solid => "solid",
            Self::Text => "text",
            Self::Plain => "plain",
        }
    }
}

/// [`mark`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct MarkProps {
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: MarkVariant,
    /// colorPalette 軸（既定 `Accent`）。[`crate::theme`] のセマンティック色
    /// から選択する。
    pub palette: ColorPalette,
}

impl Default for MarkProps {
    fn default() -> Self {
        MarkProps {
            variant: MarkVariant::Subtle,
            palette: ColorPalette::Accent,
        }
    }
}

/// Mark の recipe（scope `"mark"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_scale_declarations`] 経由の
/// `--fandhe-palette`/`-emphasized`/`-fg`/`-subtle`/`-muted`/`-fg-subtle`
/// （[`crate::badge::recipe`] と同型）を参照する（イシュー #1679 で 6 役割版へ
/// 移行）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("mark", &["root"])
        .base(
            "root",
            vec![
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("padding-inline", "0.25em"),
            ],
        )
        .variant(
            MarkVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            MarkVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            MarkVariant::Text,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            MarkVariant::Plain,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("padding-inline", "0"),
                decl("border-radius", "0"),
            ],
        )
        .default_variant(MarkVariant::Subtle)
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

/// Mark の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Mark 1 個（`<mark>`）を組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps};
///
/// let node = mark(&MarkProps::default(), vec![], vec![text("highlighted")]);
/// let html = render(&node);
/// assert!(html.starts_with("<mark"));
/// assert!(html.contains("highlighted"));
/// ```
#[must_use]
pub fn mark<'a>(props: &MarkProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "mark", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_subtle_accent() {
        let html = render(&mark(&MarkProps::default(), vec![], vec![text("hi")]));
        assert_eq!(
            html,
            r#"<mark data-scope="mark" data-part="root" class="fd-mark--variant-subtle fd-mark--color-palette-accent">hi</mark>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (MarkVariant::Subtle, "fd-mark--variant-subtle"),
            (MarkVariant::Solid, "fd-mark--variant-solid"),
            (MarkVariant::Text, "fd-mark--variant-text"),
            (MarkVariant::Plain, "fd-mark--variant-plain"),
        ] {
            let props = MarkProps {
                variant,
                ..MarkProps::default()
            };
            let html = render(&mark(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class} fd-mark--color-palette-accent\"")),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-mark--color-palette-accent"),
            (ColorPalette::Info, "fd-mark--color-palette-info"),
            (ColorPalette::Success, "fd-mark--color-palette-success"),
            (ColorPalette::Warning, "fd-mark--color-palette-warning"),
            (ColorPalette::Danger, "fd-mark--color-palette-danger"),
            (ColorPalette::Neutral, "fd-mark--color-palette-neutral"),
        ] {
            let props = MarkProps {
                palette,
                ..MarkProps::default()
            };
            let html = render(&mark(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"fd-mark--variant-subtle {class}\"")),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&mark(
            &MarkProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&mark(
            &MarkProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }
}
