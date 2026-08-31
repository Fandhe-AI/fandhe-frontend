//! Mark（イシュー #771）: 単一 recipe styled 部品。テキストハイライト
//! （`<mark>`）を `variant`/`colorPalette` の 2 軸で組み立てる
//! （[`crate::badge`] と同型の単一 recipe パターン）。
//!
//! # イシュー #1439 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`typography/mark.md`）とスクリーンショット比較した結果を
//! 記録する（Radix Themes に Mark 相当なし、ark-ui は headless utility の
//! みで独自スタイルを持たない）。
//!
//! - **サイズ**: 軸を新設しない。
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4(c)
//!   の保有判定基準により、mark は size 軸を持たない Typography 周辺部品
//!   と確定済み。
//! - **バリアント**: `subtle`（既定）/`solid`/`text`/`plain` の 4 値は
//!   chakra と一致しており追加なし。
//! - **subtle（既定）の是正**: 旧実装は `background:
//!   var(--fandhe-color-bg-subtle)`（中立背景固定）+
//!   `color: var(--fandhe-palette)`（色付き文字）だったが、chakra は
//!   `bg: colorPalette.subtle`（palette 連動の淡色背景）+
//!   `color: inherit`（文字色は本文を継承）と定義している。本 issue で
//!   `background: var(--fandhe-palette-subtle)` + `color: inherit` へ
//!   是正した。[`crate::code`]（#1432）の subtle は
//!   `--fandhe-palette-fg-subtle`（色付き文字）を採るが、これは chakra
//!   側で Code と Mark の subtle 定義そのものが異なる（Code は色付き fg・
//!   Mark は inherit）ことの反映であり、両部品間の不整合ではない。
//! - **text の是正**: 旧実装は `background: transparent` +
//!   `color: var(--fandhe-palette)`（色付き文字）だったが、chakra の
//!   Mark text は `fontWeight: medium` のみを持ち色は inherit のため、
//!   `background: transparent` + `color: inherit` +
//!   `font-weight: var(--fandhe-font-font-weight-medium)` へ是正した。
//! - **solid/plain**: chakra と一致しており変更なし。
//! - **既定 palette の是正**: 旧既定 [`ColorPalette::Accent`] は chakra の
//!   既定 colorPalette（`gray`）と乖離していたため、[`ColorPalette::Neutral`]
//!   へ是正した（[`crate::code`] #1432・kbd #1721 と同一判断。#1711 が
//!   Phase 1 部品側の宿題として残した「subtle 系配色の 6 役割移行」を
//!   本 issue で消化する）。
//! - **`data-*` 状態**: `data-scope`/`data-part` のみを持つ静的部品であり
//!   変更なし。
//! - **ダーク**: 全宣言が `--fandhe-*` トークン参照のみ（生色リテラル
//!   なし）のため `write_dark_declarations` の一元機構に自動追従する。
//! - **状態（hover/disabled/transition）・フォーカスリング**: 適用しない
//!   （意図的）。mark は非インタラクティブな表示専用部品であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （hover はインタラクティブ slot のみ）・
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   （フォーカスリングはフォーカス対象部品のみ）のいずれの適用対象にも
//!   当たらない（[`crate::code`]・[`crate::highlight`] と同一判断）。
//! - **`white-space: nowrap`**: chakra base はこれを持つが、意図的に
//!   非採用とする。日本語文中の複数語・長句ハイライトで折り返し不能に
//!   なり本文レイアウトを壊すため（フレームワークの国際化前提を優先）。

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
    /// colorPalette 軸（既定 `Neutral`。イシュー #1439 で chakra の既定
    /// colorPalette（`gray`）へ合わせ `Accent` から是正）。[`crate::theme`]
    /// のセマンティック色から選択する。
    pub palette: ColorPalette,
}

impl Default for MarkProps {
    fn default() -> Self {
        MarkProps {
            variant: MarkVariant::Subtle,
            palette: ColorPalette::Neutral,
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
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "inherit"),
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
                decl("color", "inherit"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
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
        .default_variant(ColorPalette::Neutral);

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
    fn default_props_render_subtle_neutral() {
        let html = render(&mark(&MarkProps::default(), vec![], vec![text("hi")]));
        assert_eq!(
            html,
            r#"<mark data-scope="mark" data-part="root" class="fd-mark--variant-subtle fd-mark--color-palette-neutral">hi</mark>"#
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
                html.contains(&format!("class=\"{class} fd-mark--color-palette-neutral\"")),
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
