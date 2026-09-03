//! Badge（イシュー #550。イシュー #1555 で参照サイト基準（chakra-ui/Radix
//! Themes）へスタイル調整済み）: 単一 recipe styled 部品。ステータス表示・
//! ラベル装飾のための `<span>` を組み立てる。装飾的テキストであり、追加の
//! `role`/`aria-*` は付与しない（chakra-ui v3 準拠の最小サブセット）。
//!
//! # イシュー #1555 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（Badge、`variant`（`solid`/`subtle`(既定)/`outline`/`surface`/
//! `plain`）+ `size`（`xs`〜`lg`）+ `colorPalette`、既定 `gray`）・Radix
//! Themes（Badge、`variant`（`soft`(既定)/`solid`/`surface`/`outline`）+
//! `size`（1〜3）+ `color`）とスクリーンショット
//! （`docs/design/reference-screenshots/{chakra,radixt}-badge-*.png`）を
//! 比較した結果を記録する。
//!
//! - **サイズ**: 共通 [`crate::recipe::Size`] の 5 段（Xs〜Xl、#1681 の
//!   進行則）を維持する。chakra の 4 段・Radix の 3 段は共通語彙の 5 段へ
//!   丸める（avatar/kbd/code と同じ判断）。padding の生値（4px 格子外）は
//!   kbd（#1436）/code（#1717）が「badge/tag/code と同一進行則」として
//!   据え置いた先例に揃え、本イシューでは変更しない（トークン化は横断課題）。
//! - **バリアント**: [`BadgeVariant`] へ `Surface` を追加し 4 値へ拡張した
//!   （`Solid`/`Subtle`(既定)/`Outline`/`Surface`）。`surface` は chakra・
//!   Radix Themes 双方に存在し、本リポジトリ既存語彙
//!   `ButtonVariant::Surface`（#1448）と同名で持ち込める。chakra `plain` は
//!   最小サブセット方針（badge #768 以来、code/kbd/avatar で見送り済み）を
//!   継続する。
//! - **色**: `Subtle` を `--fandhe-color-bg-subtle`（中立色）+
//!   `--fandhe-palette`（文字）の旧 3 役割配色から
//!   `--fandhe-palette-subtle`（背景）+ `--fandhe-palette-fg-subtle`
//!   （文字）の 6 役割 palette（[`crate::recipe::palette_scale_declarations`]）
//!   へ移行した（chakra `subtle` = `colorPalette.subtle` 背景 +
//!   `colorPalette.fg` 文字、Radix `soft` = accent-3 背景 + accent-11 文字
//!   に相当）。`Outline` は文字色を palette 非連動の `--fandhe-palette` から
//!   `--fandhe-palette-fg-subtle` へ、枠線を palette 非連動の
//!   `--fandhe-color-border` から `--fandhe-palette-muted` へそれぞれ
//!   移行した（code/kbd が Phase 1 で既に済ませていた移行に追随。
//!   `Outline` = `--fandhe-palette-fg-subtle` 文字 + `--fandhe-palette-muted`
//!   枠線という配色パターンは code.rs/kbd.rs と一致する）。新設 `Surface` は
//!   `--fandhe-palette-subtle` 背景 + `--fandhe-palette-fg-subtle` 文字 +
//!   `--fandhe-palette-muted` 枠線。`Solid` は不変。既定 palette は
//!   `Accent` を維持する（Radix Themes の accent 既定と一致し、badge は
//!   ステータス表示部品のため中立色既定〔avatar/kbd/code〕は採らない）。
//! - **状態（`data-*`）**: 増減なし。headless を持たない静的部品で
//!   `data-scope`/`data-part` のみ（[`ANATOMY`]）。
//! - **ダーク**: 追加宣言はすべてトークン参照のため
//!   `write_dark_declarations` へ自動追従する。コントラストは `theme.rs`
//!   の既存テストが固定済みでありトークン追加はない。
//! - **フォーカス**: 非適用（意図的）。フォーカス不能な `<span>` であり
//!   #1424 の適用対象外。
//! - **余白・角丸・影**: base へ `gap`（アイコン併用時、chakra
//!   `gap: 1` 相当）・`white-space: nowrap`・
//!   `font-variant-numeric: tabular-nums`・
//!   `line-height: var(--fandhe-font-line-height-tight)` を追加した
//!   （chakra base の圧縮行高・折り返し禁止に合わせる）。角丸は
//!   `--fandhe-radius-sm`（密なインライン部品、#1423）を維持し影は付けない。
//!   chakra の `user-select: none` はテキスト選択・コピーを妨げるため
//!   採らない。
//! - **hover / disabled / transition**: 非適用（意図的）。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 が
//!   「表示専用（badge/alert/card/stat 等）には hover を付けない」と
//!   明記しており、disabled 概念・遷移対象もない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
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
    /// 淡色背景 + 輪郭（イシュー #1555。`ButtonVariant::Surface`〔#1448〕と
    /// 同名。chakra-ui/Radix Themes 双方に存在する variant で、`Subtle` の
    /// 塗りに `Outline` の枠線を重ねた見た目）。
    Surface,
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
            Self::Surface => "surface",
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
/// 色は [`crate::recipe::palette_scale_declarations`] 経由の 6 役割 palette
/// （`--fandhe-palette`/`--fandhe-palette-fg`/`--fandhe-palette-subtle`/
/// `--fandhe-palette-muted`/`--fandhe-palette-fg-subtle`、イシュー #606/#1679）
/// を参照する（[`crate::button::recipe`] の rustdoc 参照）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("badge", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("white-space", "nowrap"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        // イシュー #1681: Xs は padding（垂直 2 倍刻み・水平 0.125rem 刻み）
        // を外挿。font-size はトークン下限 xs を Sm と共有する（より小さい
        // トークンが存在しないため）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("padding", "0.03125rem 0.25rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
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
            Size::Xl,
            "root",
            vec![
                decl("padding", "0.5rem 0.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
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
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            BadgeVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            BadgeVariant::Surface,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
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
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
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
            (BadgeVariant::Surface, "fd-badge--variant-surface"),
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
            (ColorPalette::Neutral, "fd-badge--color-palette-neutral"),
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

    /// イシュー #1555: `Subtle`/`Surface` が 6 役割 palette の淡色トークン
    /// （`--fandhe-palette-subtle`）を消費することを固定する。
    #[test]
    fn css_output_declares_palette_subtle_token() {
        let out = css();
        assert!(out.contains("background: var(--fandhe-palette-subtle);"));
        assert!(out.contains("color: var(--fandhe-palette-fg-subtle);"));
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
