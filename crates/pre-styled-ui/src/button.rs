//! Button（イシュー #550）: 単一 recipe styled 部品。`<button type="button">`
//! を組み立てる。
//!
//! `loading: true` のとき [`crate::spinner::spinner_decorative`]（`role`/
//! `aria-label` を持たない装飾用途の Spinner）を子ノード先頭へ埋め込む
//! （呼び出し先の契約: Spinner は状態機械を要しない静的部品であり、Button の
//! 内部でのみ組み立てて返す。ボタン自身の `aria-busy` が既に読み上げ状態を
//! 伝えるため、公開 API の [`crate::spinner::spinner`] が持つ
//! `role="status"` + `aria-label` のライブリージョンを二重に埋め込まない）。
//! また `loading: true` のときは `disabled: true` と同様に `disabled` 属性・
//! `data-disabled`・`aria-disabled="true"` も付与し、読み込み中のクリック・
//! 暗黙 submit による重複アクションの発火を防ぐ（Medium severity のバグ
//! 指摘の是正、`aria-busy`/`data-loading` だけでは操作を止められないため）。
//! 呼び出し側 `attrs` は `class_attr::drop_class_attr` を経由して `class` を
//! 除去してから合成し、recipe が生成するクラスが常に唯一の `class` 属性値に
//! なる。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use crate::spinner::spinner_decorative;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_disabled, data_disabled, Anatomy};

/// `data-scope="button"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("button");

/// Button の見た目 variant（chakra-ui v3 準拠の最小サブセット）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// 塗りつぶし（既定）。
    #[default]
    Solid,
    /// 輪郭のみ。
    Outline,
    /// 背景なし・最小装飾。
    Ghost,
    /// 淡色背景。
    Subtle,
}

impl VariantValue for ButtonVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Subtle => "subtle",
        }
    }
}

/// [`button`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct ButtonProps {
    /// 見た目 variant（既定 `Solid`）。
    pub variant: ButtonVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Accent`、イシュー #606）。[`crate::theme`] の
    /// セマンティック色（`accent`/`info`/`success`/`warning`/`danger`）から
    /// 選択する。
    pub palette: ColorPalette,
    /// 無効化。`true` のとき `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` を付与する。
    pub disabled: bool,
    /// 読み込み中。`true` のとき `aria-busy="true"`・`data-loading` を付与し、
    /// [`crate::spinner::spinner_decorative`] を子ノード先頭へ埋め込む。
    /// [`Self::disabled`] と同様に `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` も付与し、読み込み中のクリック・暗黙 submit
    /// を止める。
    pub loading: bool,
}

impl Default for ButtonProps {
    fn default() -> Self {
        ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Md,
            palette: ColorPalette::Accent,
            disabled: false,
            loading: false,
        }
    }
}

/// Button の recipe（scope `"button"`、slot `"root"` のみ）。
///
/// 色は [`crate::recipe::palette_declarations`] が生成する
/// `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg`
/// （イシュー #606）経由で参照し、`var(--fandhe-color-accent)` 等の
/// セマンティック色を直接参照しない（`palette` variant の切り替えだけで
/// 全 variant の色が追従する）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("button", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "0.5rem"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("cursor", "pointer"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("padding", "0.25rem 0.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("padding", "0.5rem 1rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("padding", "0.75rem 1.5rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            ButtonVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
                decl("border", "none"),
            ],
        )
        .variant(
            ButtonVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "1px solid var(--fandhe-palette)"),
            ],
        )
        .variant(
            ButtonVariant::Ghost,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "none"),
            ],
        )
        .variant(
            ButtonVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
                decl("border", "none"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ButtonVariant::Solid)
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

/// Button の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Button 1 個を組み立てる。
///
/// `type="button"` を既定固定し、フォーム内の暗黙 submit（`type` 省略時の
/// HTML 既定値 `"submit"`）による事故を防ぐ（安全側既定、
/// `.claude/rules/security.md` セキュリティ設定ミス対策相当）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
///
/// let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
/// let html = render(&node);
/// assert!(html.contains(r#"type="button""#));
/// assert!(html.contains("Save"));
/// ```
#[must_use]
pub fn button<'a>(
    props: &ButtonProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);

    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), ("class", class.as_str())];
    if props.disabled || props.loading {
        merged.push(("disabled", ""));
        merged.extend(data_disabled(true));
        merged.push(aria_disabled(true));
    }
    if props.loading {
        merged.push(("aria-busy", "true"));
        merged.push(("data-loading", ""));
    }
    merged.extend(drop_class_attr(attrs));

    let mut node_children = Vec::with_capacity(children.len() + 1);
    if props.loading {
        node_children.push(spinner_decorative(Size::Sm));
    }
    node_children.extend(children);

    ANATOMY.part("root", "button", merged, node_children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_solid_md_type_button() {
        let node = button(&ButtonProps::default(), vec![], vec![text("Save")]);
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<button data-scope="button" data-part="root" type="button" "#,
                r#"class="fd-button--size-md fd-button--variant-solid fd-button--color-palette-accent">Save</button>"#,
            )
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ButtonVariant::Solid, "fd-button--variant-solid"),
            (ButtonVariant::Outline, "fd-button--variant-outline"),
            (ButtonVariant::Ghost, "fd-button--variant-ghost"),
            (ButtonVariant::Subtle, "fd-button--variant-subtle"),
        ] {
            let props = ButtonProps {
                variant,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md {class} fd-button--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-button--size-sm"),
            (Size::Md, "fd-button--size-md"),
            (Size::Lg, "fd-button--size-lg"),
        ] {
            let props = ButtonProps {
                size,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "{class} fd-button--variant-solid fd-button--color-palette-accent"
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    /// イシュー #606: `palette` の 5 値が期待どおりのクラス
    /// （`fd-button--color-palette-<value>`）へ写像されることを固定する。
    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-button--color-palette-accent"),
            (ColorPalette::Info, "fd-button--color-palette-info"),
            (ColorPalette::Success, "fd-button--color-palette-success"),
            (ColorPalette::Warning, "fd-button--color-palette-warning"),
            (ColorPalette::Danger, "fd-button--color-palette-danger"),
        ] {
            let props = ButtonProps {
                palette,
                ..ButtonProps::default()
            };
            let html = render(&button(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-button--size-md fd-button--variant-solid {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    /// イシュー #606: recipe の静的 CSS に `--fandhe-palette` 系の宣言と
    /// `var(--fandhe-radius-md)` の参照が含まれることを固定する。
    #[test]
    fn css_output_declares_palette_custom_properties_and_radius_token() {
        let out = css();
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-accent)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-info)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-success)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-warning)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
        assert!(out.contains("background: var(--fandhe-palette);"));
        assert!(out.contains("color: var(--fandhe-palette-fg);"));
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
    }

    #[test]
    fn disabled_adds_disabled_data_disabled_and_aria_disabled() {
        let props = ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_adds_aria_busy_data_loading_and_spinner_child() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-loading="""#));
        assert!(html.contains(r#"data-scope="spinner" data-part="root""#));
        // spinner は children の先頭に挿入される。
        let spinner_pos = html.find("data-scope=\"spinner\"").unwrap();
        let save_pos = html.find("Save").unwrap();
        assert!(spinner_pos < save_pos);
    }

    #[test]
    fn loading_also_disables_button_to_prevent_duplicate_actions() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn loading_spinner_is_decorative_and_does_not_break_button_name() {
        let props = ButtonProps {
            loading: true,
            ..ButtonProps::default()
        };
        let html = render(&button(&props, vec![], vec![text("Save")]));
        assert!(!html.contains(r#"role="status""#));
        assert!(!html.contains("aria-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![("class", "attacker-controlled"), ("id", "save-btn")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
        assert!(html.contains(r#"id="save-btn""#));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![],
            vec![text("<script>alert('xss')</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    }
}
