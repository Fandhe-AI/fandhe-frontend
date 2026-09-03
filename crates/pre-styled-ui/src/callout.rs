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
//!
//! # 参考サイト基準への調整（イシュー #1556）
//!
//! 参照軸のうち Radix Themes `Callout` のみが直接対応する部品である
//! （chakra-ui は `Alert` のみで Callout を持たず、Radix Primitives /
//! ark-ui には存在しない）。同部品の視覚基準に照らし、[`crate::alert`]
//! （イシュー #1553）と同じ設計判断で以下を是正・追加した。
//!
//! - **色**: 生の中立色（`--fandhe-color-bg-subtle`）+ 素の `--fandhe-palette`
//!   だった Soft/Surface の配色を、[`crate::recipe::palette_scale_declarations`]
//!   経由の 6 役割トークン（`--fandhe-palette-subtle`/`-fg-subtle`/`-muted`）へ
//!   移行した。本文サイズ（Md = `font-size-sm`）で 4.5:1 の WCAG コントラスト
//!   を満たすため（素の palette 色は満たさない、`crate::theme` 参照）。
//! - **サイズ**: padding・gap・角丸・font-size を root の
//!   `--fandhe-callout-*` custom property へ一本化し、5 段（Xs〜Xl）すべてで
//!   連動させた（従来は padding が生リテラルで角丸が固定、`text` slot 側の
//!   font-size と root の `size` を呼び出し側が揃える必要があった）。
//! - **`text()` の破壊的変更**: 上記一本化に伴い `text()` から `size`
//!   引数を削除した（root の `font-size` を継承するだけで揃うため、
//!   「root と text に同じ size を渡す」という旧設計の footgun を解消）。
//! - **余白・角丸**: `0.75rem 1rem` 等の生値をトークン
//!   （`--fandhe-space-*`/`--fandhe-radius-*`）化した。
//! - **アイコン整列**: icon の高さを行高に揃え、1 行目の中央に配置する
//!   （`height: calc(1em * line-height)`）。
//! - **意図的に追随しない点**:
//!   - hover / disabled / transition / `:focus-visible`: 表示専用・
//!     非フォーカス要素であり、参照サイトも状態遷移を持たない
//!     （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）。
//!   - `data-*` 状態: headless 側部品を持たない（`wrap_state.rs` バケット
//!     「pre-styled only」）。
//!   - Radix `highContrast`: トークン体系にない軸のため見送り。
//!   - Radix の grid レイアウト（複数 Text の縦積み）: anatomy が text
//!     1 パーツのため flex のまま。
//!   - 影: Radix Callout は影を持たない。

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
/// を決定する。size 軸は [`SlotRecipe::size_variants`] を最初に呼ぶことで
/// 最初に登録される axis になる（同メソッドは呼び出し末尾で必ず
/// `Size::Md` を既定へ戻すため、後続の `default_variant` 呼び出し順に
/// 依存せず size の既定は常に `Md` になる）。
///
/// `border: 1px solid transparent` を base 側に置き、`Surface`/`Outline`
/// variant は `border-color` のみを上書きする（variant 切替でボックス高さが
/// ±1px ぶれないようにするため。[`crate::alert`] の是正と同じ動機）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("callout", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("box-sizing", "border-box"),
                decl("gap", "var(--fandhe-callout-gap, var(--fandhe-space-3))"),
                decl(
                    "padding",
                    "var(--fandhe-callout-padding, var(--fandhe-space-4))",
                ),
                decl("border", "1px solid transparent"),
                decl(
                    "border-radius",
                    "var(--fandhe-callout-radius, var(--fandhe-radius-lg))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-callout-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "icon",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("flex-shrink", "0"),
                decl(
                    "height",
                    "calc(1em * var(--fandhe-font-line-height-normal))",
                ),
            ],
        )
        .base("text", vec![decl("min-width", "0")])
        // イシュー #1556: padding・gap・角丸・font-size を root の
        // `--fandhe-callout-*` custom property へ一本化。Sm/Md が
        // font-size を共有するのは Radix Themes size 1/2 がともに
        // `font-size-2`（14px）であることに忠実な意図的判断（alert は
        // chakra 基準で Sm = xs だが、callout は Radix Themes のみを
        // 参照するため揃えない）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-callout-padding", "var(--fandhe-space-2)"),
                        decl("--fandhe-callout-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-callout-radius", "var(--fandhe-radius-sm)"),
                        decl(
                            "--fandhe-callout-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-callout-padding", "var(--fandhe-space-3)"),
                        decl("--fandhe-callout-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-callout-radius", "var(--fandhe-radius-md)"),
                        decl(
                            "--fandhe-callout-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-callout-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-callout-gap", "var(--fandhe-space-3)"),
                        decl("--fandhe-callout-radius", "var(--fandhe-radius-lg)"),
                        decl(
                            "--fandhe-callout-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-callout-padding", "var(--fandhe-space-5)"),
                        decl("--fandhe-callout-gap", "var(--fandhe-space-4)"),
                        decl("--fandhe-callout-radius", "var(--fandhe-radius-xl)"),
                        decl(
                            "--fandhe-callout-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-callout-padding", "var(--fandhe-space-6)"),
                        decl("--fandhe-callout-gap", "var(--fandhe-space-4)"),
                        decl("--fandhe-callout-radius", "var(--fandhe-radius-2xl)"),
                        decl(
                            "--fandhe-callout-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
        .variant(
            CalloutVariant::Soft,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            CalloutVariant::Surface,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border-color", "var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            CalloutVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border-color", "var(--fandhe-palette-muted)"),
            ],
        )
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
/// イシュー #1556: font-size は [`root`] の `--fandhe-callout-font-size`
/// custom property から継承するのみで決まるため、本関数は `size` 引数を
/// 取らない（旧設計は `root` と `text` へ同じ `size` を渡す必要があり、
/// 揃え漏れで文字サイズが崩れる footgun があった）。
#[must_use]
pub fn text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("text", "div", attrs, children)
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
            (Size::Xs, "fd-callout--size-xs"),
            (Size::Sm, "fd-callout--size-sm"),
            (Size::Md, "fd-callout--size-md"),
            (Size::Lg, "fd-callout--size-lg"),
            (Size::Xl, "fd-callout--size-xl"),
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
        assert!(render(&text(vec![], vec![]))
            .starts_with(r#"<div data-scope="callout" data-part="text""#));
    }

    #[test]
    fn composed_callout_snapshot() {
        let node = root(
            &CalloutProps::default(),
            vec![],
            vec![
                icon(vec![], vec![]),
                text(
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
                r#"<div data-scope="callout" data-part="text">Heads up: this is supplementary info</div>"#,
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
        let html = render(&text(vec![], vec![text_node("<script>alert(1)</script>")]));
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

        let html = render(&text(vec![("data-testid", payload)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    /// イシュー #1556: 参考サイト基準への調整で palette 6 役割トークンへ
    /// 移行したことを固定する（生の `--fandhe-color-bg-subtle`/
    /// `--fandhe-palette` 単体参照へ後退しないこと）。
    #[test]
    fn css_output_declares_palette_role_tokens_and_size_custom_properties() {
        let out = css();
        assert!(out.contains("background: var(--fandhe-palette-subtle);"));
        assert!(out.contains("color: var(--fandhe-palette-fg-subtle);"));
        assert!(out.contains("border-color: var(--fandhe-palette-muted);"));
        assert!(out.contains("--fandhe-callout-padding: var(--fandhe-space-2);"));
        assert!(out.contains("--fandhe-callout-radius: var(--fandhe-radius-2xl);"));
        assert!(!out.contains("--fandhe-color-bg-subtle"));
    }
}
