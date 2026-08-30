//! Code（イシュー #768、#1432 で参照サイト基準へ調整）: 単一 recipe styled
//! 部品。インラインコード片表示のための `<code>` を `variant`/`size`/
//! `colorPalette` の 3 軸で組み立てる（[`crate::mark`] と同型の単一 recipe
//! パターン）。
//!
//! chakra-ui v3 の `typography/code-block.md`（CodeBlock）は
//! `docs/design/component-coverage-map.md` で対象外確定済みであり、本
//! モジュールはインライン `<code>` のみを扱う。`class` は
//! [`crate::class_attr::drop_class_attr`] で呼び出し側の指定を破棄してから
//! recipe 由来のクラスへ差し替える（badge/tag/mark と同一契約）。
//!
//! # イシュー #1432 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`typography/code.md`、`xs/sm/md/lg` size + `solid/subtle/
//! surface/outline/plain` variant + colorPalette 連動）・Radix Themes
//! （`Code`、`1`〜`9` size + `classic/solid/soft/surface/outline/ghost`
//! variant + accent 色連動）とスクリーンショット比較した結果を記録する。
//!
//! - **サイズ**: 軸を新設した（旧実装は固定 1 段）。共通 [`Size`]（Xs〜Xl
//!   の 5 段）を使う。Radix の `1`〜`9` 9 段は本リポジトリの共通語彙
//!   （5 段）へ丸め、9 段の精密さは意図的に採用しない（badge/tag と同じ
//!   進行則を再利用し部品間の一貫性を優先するため）。
//! - **バリアント**: `Solid`/`Subtle`（既定）/`Outline` の最小 3 値
//!   （badge/tag と同一語彙のサブセット）を新設した。chakra の `surface`/
//!   `plain`、Radix の `classic`/`ghost` は badge（#768）以来の最小
//!   サブセット方針により見送る。
//! - **色**: [`ColorPalette`] 軸（6 値）を新設した。`Subtle` は
//!   `--fandhe-palette-subtle` 背景 + `--fandhe-palette-fg-subtle` 文字
//!   （#1711 が Phase 1 部品側の宿題として残した「subtle 系配色の 6 役割
//!   移行」を本 issue で消化する）。既定 palette は chakra Code の既定
//!   colorPalette（`gray`）に合わせ [`ColorPalette::Neutral`] とし、
//!   旧実装の中立灰色系の見た目からの乖離を避ける。
//! - **状態（hover/disabled/transition）・フォーカスリング**: 適用しない
//!   （意図的）。code は非インタラクティブな表示専用部品であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （hover はインタラクティブ slot のみ）・
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   （フォーカスリングはフォーカス対象部品のみ）のいずれの適用対象にも
//!   当たらない。
//! - **ダーク**: 全宣言が `--fandhe-*` トークン参照のみ（生色リテラル
//!   なし）のため `write_dark_declarations` の一元機構に自動追従する。
//! - **余白・角丸**: padding/font-size を badge/tag と同一の size 進行則
//!   （#1681）へ載せ替えた。角丸は既存の `--fandhe-radius-sm` を維持。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="code"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("code");

/// Code の見た目 variant（[`crate::tag::TagVariant`] と同型の 3 値。
/// chakra の `surface`/`plain`、Radix の `classic`/`ghost` は最小サブセット
/// 方針により見送り、[`crate` 冒頭 rustdoc](self) の 7 軸チェック参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeVariant {
    /// 塗りつぶし。
    Solid,
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for CodeVariant {
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

/// [`code`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct CodeProps {
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: CodeVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Neutral`。chakra Code の既定 colorPalette
    /// `gray` に合わせる）。
    pub palette: ColorPalette,
}

impl Default for CodeProps {
    fn default() -> Self {
        CodeProps {
            variant: CodeVariant::Subtle,
            size: Size::Md,
            palette: ColorPalette::Neutral,
        }
    }
}

/// Code の recipe（scope `"code"`、slot `"root"` のみ、variant 3 軸）。
///
/// `font-family` は旧実装から維持する固定 mono フォントスタック文字列
/// （[`crate::theme`] に mono フォントトークンが存在しないため）。
///
/// size 進行則は badge/tag（#1681）と同一の padding/font-size 刻みを流用
/// する。variant 別の配色:
///
/// - **Subtle（既定）**: `--fandhe-palette-subtle` 背景 +
///   `--fandhe-palette-fg-subtle` 文字（chakra `subtle` に相当）。
/// - **Solid**: `--fandhe-palette` 背景 + `--fandhe-palette-fg` 文字
///   （chakra `solid` に相当）。
/// - **Outline**: 背景なし + `--fandhe-palette-fg-subtle` 文字 +
///   `--fandhe-palette-muted` の 1px 枠線。badge/tag の `Outline` は
///   枠線に palette 非連動の `--fandhe-color-border` を使うが、chakra/
///   Radix の Code `outline` はいずれも colorPalette 連動の枠線色（chakra:
///   `colorPalette.7`、Radix: accent 系）であるため、本部品では意図的に
///   palette 連動の `--fandhe-palette-muted` を採用する（badge/tag との
///   差分は参照サイト対応を優先した結果であり、統一漏れではない）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("code", &["root"])
        .base(
            "root",
            vec![
                decl(
                    "font-family",
                    "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        // イシュー #1681: badge/tag の recipe と同一進行則。
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
            CodeVariant::Solid,
            "root",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            CodeVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            CodeVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(CodeVariant::Subtle)
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

/// Code の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Code 片 1 個（`<code>`）を組み立てる。`variant`/`size`/`palette` に
/// 応じたクラスを付与する（呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] により破棄してから合成する、
/// [`crate::mark::mark`] と同型の契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::code::{code, CodeProps};
///
/// let node = code(&CodeProps::default(), vec![], vec![text("cargo build")]);
/// assert!(render(&node).contains("cargo build"));
/// ```
#[must_use]
pub fn code<'a>(props: &CodeProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "code", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_render_has_expected_class_attribute() {
        let html = render(&code(
            &CodeProps::default(),
            vec![],
            vec![text("cargo build")],
        ));
        assert_eq!(
            html,
            r#"<code data-scope="code" data-part="root" class="fd-code--size-md fd-code--variant-subtle fd-code--color-palette-neutral">cargo build</code>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (CodeVariant::Solid, "fd-code--variant-solid"),
            (CodeVariant::Subtle, "fd-code--variant-subtle"),
            (CodeVariant::Outline, "fd-code--variant-outline"),
        ] {
            let props = CodeProps {
                variant,
                ..CodeProps::default()
            };
            let html = render(&code(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-code--size-md {class} fd-code--color-palette-neutral\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-code--color-palette-accent"),
            (ColorPalette::Info, "fd-code--color-palette-info"),
            (ColorPalette::Success, "fd-code--color-palette-success"),
            (ColorPalette::Warning, "fd-code--color-palette-warning"),
            (ColorPalette::Danger, "fd-code--color-palette-danger"),
            (ColorPalette::Neutral, "fd-code--color-palette-neutral"),
        ] {
            let props = CodeProps {
                palette,
                ..CodeProps::default()
            };
            let html = render(&code(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-code--size-md fd-code--variant-subtle {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-code--size-xs"),
            (Size::Sm, "fd-code--size-sm"),
            (Size::Md, "fd-code--size-md"),
            (Size::Lg, "fd-code--size-lg"),
            (Size::Xl, "fd-code--size-xl"),
        ] {
            let props = CodeProps {
                size,
                ..CodeProps::default()
            };
            let html = render(&code(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-code--variant-subtle fd-code--color-palette-neutral\""
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&code(
            &CodeProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&code(
            &CodeProps::default(),
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
