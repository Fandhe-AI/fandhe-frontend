//! Kbd（イシュー #768、#1436 で参照サイト基準へ調整）: 単一 recipe styled
//! 部品。キーボード入力・ショートカット表示のための `<kbd>` を
//! `variant`/`size`/`colorPalette` の 3 軸で組み立てる（[`crate::code`] と
//! 同型の単一 recipe パターン）。`class` は
//! [`crate::class_attr::drop_class_attr`] で呼び出し側の指定を破棄してから
//! recipe 由来のクラスへ差し替える（badge/tag/code/mark と同一契約）。
//!
//! # イシュー #1436 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`typography/kbd.md`、`sm/md/lg` size + `raised`(既定)/
//! `outline`/`subtle`/`plain` variant + colorPalette 連動、既定 `gray`）・
//! Radix Themes（`Kbd`、`1`〜`9` size のみ・variant なし・中立色固定）と
//! スクリーンショット比較した結果を記録する。
//!
//! - **サイズ**: 軸を新設した（旧実装は固定 1 段）。共通 [`Size`]（Xs〜Xl
//!   の 5 段）を使い、badge/tag/code と同一の padding/font-size 進行則
//!   （#1681）に載せる。既定は `Md`（chakra Kbd の既定 `md` と一致）。
//!   Radix の `1`〜`9` 9 段は本リポジトリの共通語彙（5 段）へ丸め、9 段の
//!   精密さは意図的に採用しない（badge/tag/code と同じ判断）。
//! - **バリアント**: `Raised`（既定）/`Subtle`/`Outline` の 3 値を新設した。
//!   `Raised` は現行の意匠（枠線 + `border-bottom-width: 2px` によるキー
//!   押下風の立体表現 + 淡色背景）を継承し、chakra の既定 `raised` に
//!   相当する。`Subtle`/`Outline` は既存語彙（badge/tag/code）と同型
//!   （立体表現なしの淡色背景／背景なし + 枠線）。`Raised` は既存 3 値
//!   語彙に対応物がない kbd 固有の物理的意匠の名前だが、#768 時点の
//!   rustdoc が将来軸として既に予約していた語であり、参照サイト名を
//!   そのまま持ち込む判断ではないためこの語を採用する。chakra の `plain`
//!   は最小サブセット方針（badge #768 以来、code #1432 でも見送った
//!   先例）により見送る。
//! - **色**: [`ColorPalette`] 軸（6 値）を新設した。既定 palette は chakra
//!   Kbd の既定 colorPalette（`gray`）に合わせ [`ColorPalette::Neutral`]
//!   とし、旧実装の中立灰色系の見た目からの乖離を避ける（#1711 が
//!   Phase 1 部品側の宿題として残した「subtle 系配色の 6 役割移行」を
//!   本 issue で消化する）。
//! - **状態（hover/disabled/transition）・フォーカスリング**: 適用しない
//!   （意図的）。kbd は非インタラクティブな表示専用部品であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （hover はインタラクティブ slot のみ）・
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   （フォーカスリングはフォーカス対象部品のみ）のいずれの適用対象にも
//!   当たらない（code #1717 と同一判断）。
//! - **ダーク**: 全宣言が `--fandhe-*` トークン参照のみ（生色リテラル
//!   なし）のため `write_dark_declarations` の一元機構に自動追従する。
//! - **余白・角丸・影**: padding/font-size を badge/tag/code と同一の size
//!   進行則（#1681）へ載せ替えた。角丸は既存の `--fandhe-radius-sm` を
//!   維持。`Raised` の立体表現（`border-bottom-width: 2px`）は影の代替
//!   として維持する（参照サイトのキー押下風意匠を尊重）。
//! - **`data-*`**: 状態を持たない静的部品のため増減なし。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="kbd"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("kbd");

/// Kbd の見た目 variant（[`crate::code::CodeVariant`] と同型の 3 値だが、
/// `Solid` の代わりに kbd 固有の物理的意匠 `Raised` を既定に持つ。
/// chakra の `plain` は最小サブセット方針により見送り、
/// [`crate` 冒頭 rustdoc](self) の 7 軸チェック参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KbdVariant {
    /// キー押下風の立体表現（枠線 + 下枠強調）+ 淡色背景（既定）。
    #[default]
    Raised,
    /// 淡色背景のみ（立体表現なし）。
    Subtle,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for KbdVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Raised => "raised",
            Self::Subtle => "subtle",
            Self::Outline => "outline",
        }
    }
}

/// [`kbd`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct KbdProps {
    /// 見た目 variant（既定 `Raised`）。
    pub variant: KbdVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// colorPalette 軸（既定 `Neutral`。chakra Kbd の既定 colorPalette
    /// `gray` に合わせる）。
    pub palette: ColorPalette,
}

impl Default for KbdProps {
    fn default() -> Self {
        KbdProps {
            variant: KbdVariant::Raised,
            size: Size::Md,
            palette: ColorPalette::Neutral,
        }
    }
}

/// Kbd の recipe（scope `"kbd"`、slot `"root"` のみ、variant 3 軸）。
///
/// `font-family` は旧実装から維持する固定 mono フォントスタック文字列
/// （[`crate::theme`] に mono フォントトークンが存在しないため）。
///
/// size 進行則は badge/tag/code（#1681）と同一の padding/font-size 刻みを
/// 流用する。variant 別の配色:
///
/// - **Raised（既定）**: `--fandhe-palette-subtle` 背景 +
///   `--fandhe-palette-fg-subtle` 文字 + `--fandhe-palette-muted` の
///   1px 枠線（下辺のみ 2px、キー押下風の立体表現）。
/// - **Subtle**: `--fandhe-palette-subtle` 背景 +
///   `--fandhe-palette-fg-subtle` 文字（立体表現なし。code `Subtle` と
///   同型）。
/// - **Outline**: 背景なし + `--fandhe-palette-fg-subtle` 文字 +
///   `--fandhe-palette-muted` の 1px 枠線（code `Outline` と同型）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("kbd", &["root"])
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
        // イシュー #1681: badge/tag/code の recipe と同一進行則。
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
            KbdVariant::Raised,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
                decl("border-bottom-width", "2px"),
            ],
        )
        .variant(
            KbdVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            KbdVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border", "1px solid var(--fandhe-palette-muted)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(KbdVariant::Raised)
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

/// Kbd の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Kbd 1 個（`<kbd>`）を組み立てる。`variant`/`size`/`palette` に応じた
/// クラスを付与する（呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] により破棄してから合成する、
/// [`crate::code::code`] と同型の契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::kbd::{kbd, KbdProps};
///
/// let node = kbd(&KbdProps::default(), vec![], vec![text("Ctrl")]);
/// assert!(render(&node).contains("Ctrl"));
/// ```
#[must_use]
pub fn kbd<'a>(props: &KbdProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("color-palette", props.palette.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "kbd", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_render_has_expected_class_attribute() {
        let html = render(&kbd(&KbdProps::default(), vec![], vec![text("Ctrl")]));
        assert_eq!(
            html,
            r#"<kbd data-scope="kbd" data-part="root" class="fd-kbd--size-md fd-kbd--variant-raised fd-kbd--color-palette-neutral">Ctrl</kbd>"#
        );
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (KbdVariant::Raised, "fd-kbd--variant-raised"),
            (KbdVariant::Subtle, "fd-kbd--variant-subtle"),
            (KbdVariant::Outline, "fd-kbd--variant-outline"),
        ] {
            let props = KbdProps {
                variant,
                ..KbdProps::default()
            };
            let html = render(&kbd(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-kbd--size-md {class} fd-kbd--color-palette-neutral\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-kbd--color-palette-accent"),
            (ColorPalette::Info, "fd-kbd--color-palette-info"),
            (ColorPalette::Success, "fd-kbd--color-palette-success"),
            (ColorPalette::Warning, "fd-kbd--color-palette-warning"),
            (ColorPalette::Danger, "fd-kbd--color-palette-danger"),
            (ColorPalette::Neutral, "fd-kbd--color-palette-neutral"),
        ] {
            let props = KbdProps {
                palette,
                ..KbdProps::default()
            };
            let html = render(&kbd(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-kbd--size-md fd-kbd--variant-raised {class}\""
                )),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-kbd--size-xs"),
            (Size::Sm, "fd-kbd--size-sm"),
            (Size::Md, "fd-kbd--size-md"),
            (Size::Lg, "fd-kbd--size-lg"),
            (Size::Xl, "fd-kbd--size-xl"),
        ] {
            let props = KbdProps {
                size,
                ..KbdProps::default()
            };
            let html = render(&kbd(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-kbd--variant-raised fd-kbd--color-palette-neutral\""
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&kbd(
            &KbdProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&kbd(
            &KbdProps::default(),
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
