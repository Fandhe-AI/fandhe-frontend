//! Heading（イシュー #771）: 単一 recipe styled 部品。h1〜h6 の見出し要素を
//! variant として組み立てる。text/em/mark/blockquote/list と同じく、
//! headless 状態機械を要しない静的部品（badge/skeleton と同型、
//! `docs/design/component-coverage-map.md` heading 行）。
//!
//! # 意味論レベルと視覚サイズの独立（chakra-ui の `as` prop 相当）
//!
//! [`HeadingLevel`]（レンダリングするタグ h1〜h6、文書構造上の意味論的階層）
//! と [`HeadingSize`]（`font-size`/`line-height` の視覚サイズ）は独立した
//! 軸である。文書構造上は h2 が正しくても見た目は大きく（あるいはその逆）
//! したいケースを許すため、呼び出し側は両方を独立に選択できる。
//! [`HeadingLevel`] は variant 軸ではなく、レンダリングするタグそのものを
//! 選ぶ引数として渡す（`recipe::VariantValue` を実装しない。タグ選択は
//! `crate::list::ListType` と同型の方式）。
//!
//! # サイズトークンの縮約（対象外事項、PR 本文参照）
//!
//! chakra-ui の Heading は `xs`〜`7xl` の 9 段階を持つが、[`crate::theme`]
//! のテーマトークンは `font-size-xs`〜`font-size-4xl` の 8 段階までしか
//! 持たない。本実装は `xs`〜`4xl` の 8 段階（chakra の `5xl`〜`7xl` は非採用）
//! へ縮約する。テーマトークンの拡張は本イシューのスコープ外。
//!
//! # colorPalette 軸を持たない理由
//!
//! テキストは前景色トークンを継承する中立部品であり、ステータス色を持たない
//! （[`crate::card`]・[`crate::skeleton`] が同じ判断をした根拠と同型）。
//!
//! ## 参考サイト基準との 7 軸比較（イシュー #1434）
//!
//! chakra-ui / Radix Themes の Heading 相当部品とサイズ・バリアント・色・
//! 状態・ダーク・フォーカス・余白 / hover / disabled / transition の
//! 7 軸で比較した結果（スクショは
//! `docs/design/reference-screenshots/{chakra,radixt}-heading-*.png`）:
//!
//! - **サイズ軸**: chakra は `xs`〜`7xl` の 9 段階、Radix Themes は
//!   `1`〜`9` の 9 段階を持つ。旧実装は `sm`〜`xl4` の 7 段階までしか
//!   なく、両サイトが持つ最小段（chakra `xs`）に対応する段が欠落していた。
//!   [`crate::theme`] のテーマトークンは既に `font-size-xs` を持つため、
//!   本イシューで [`HeadingSize::Xs`] を追加しテーマトークン全 8 段
//!   （`xs`〜`4xl`）を網羅する形にした。上端（chakra `5xl`〜`7xl` /
//!   Radix `size 8`〜`9` 相当）はテーマトークンが `4xl` までのため
//!   引き続き非採用（前節「サイズトークンの縮約」参照。再評価トリガーは
//!   「複数部品で 4xl 超の要求が出た時点」）。
//! - **バリアント軸**: 両サイト共に Heading へ `variant`（solid/subtle 等）
//!   prop を持たない。当部品も軸を追加しない。
//! - **色**: 両サイト共に前景色を継承する中立部品として実装されており、
//!   `colorPalette` 相当の軸を持たない。当部品も一致（前節参照）。
//! - **状態（data-*）**: heading は headless 状態機械を持たない静的部品
//!   であり、両サイトの Heading も操作状態（`data-state` 等）を持たない。
//!   一致。
//! - **ダーク**: 色宣言を持たず本文色に自動追従するため、両サイトと同様
//!   ライト / ダーク双方で自動的に一致する。
//! - **フォーカス / hover / disabled / transition**: heading は非
//!   インタラクティブな表示専用 slot であり、両サイトの Heading もこれら
//!   の状態を持たない。本フレームワークでも hover / disabled / transition
//!   はインタラクティブ slot のみに適用する方針
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）
//!   であり、フォーカスリングもフォーカス対象部品限定
//!   （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）
//!   のため付与しない（一致）。
//! - **余白・letter-spacing**: `margin: 0`（余白は利用者責務）は chakra
//!   の marginless 方針と一致。大サイズ（`xl3`/`xl4`）でのネガティブ
//!   letter-spacing 強化（chakra 4xl+ 相当）は、収集済みスクリーンショット
//!   の解像度・構図では参照サイトの正確なトラッキング値を実測で裏付け
//!   できなかったため、根拠不十分な意匠変更を避け**本イシューでは見送る**
//!   （安全側の判断）。一律 `-0.01em` の現状維持。再評価は実機での
//!   フォント計測ツールを用いた比較が可能になった時点で行う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="heading"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("heading");

/// Heading がレンダリングする HTML 要素（意味論レベル、chakra-ui の `as` prop
/// 相当）。variant クラスではなく、実際にレンダリングするタグ名を選択する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingLevel {
    /// `<h1>`。
    H1,
    /// `<h2>`（既定）。
    #[default]
    H2,
    /// `<h3>`。
    H3,
    /// `<h4>`。
    H4,
    /// `<h5>`。
    H5,
    /// `<h6>`。
    H6,
}

impl HeadingLevel {
    /// この意味論レベルに対応する HTML タグ名。
    fn tag(self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
        }
    }
}

/// Heading の視覚サイズ variant（`font-size`/`line-height`。chakra-ui の
/// `size` prop 相当。テーマトークンの範囲に合わせ `xs`〜`4xl` の 8 段階へ
/// 縮約する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingSize {
    /// 極小サイズ（イシュー #1434 で追加。参考サイト基準との 7 軸比較で
    /// chakra-ui `xs` / Radix Themes 最小段に対応する段が欠落していたため
    /// 追加した。モジュール rustdoc「参考サイト基準との 7 軸比較」節参照）。
    Xs,
    /// 小サイズ。
    Sm,
    /// 中サイズ。
    Md,
    /// 大サイズ。
    Lg,
    /// 特大サイズ（既定）。
    #[default]
    Xl,
    /// 2 段階特大。
    Xl2,
    /// 3 段階特大。
    Xl3,
    /// 4 段階特大（テーマトークンが持つ最大サイズ）。
    Xl4,
}

impl VariantValue for HeadingSize {
    fn axis(self) -> &'static str {
        "size"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            // `is_valid_identifier`（`crates/pre-styled-ui/src/css.rs`）は先頭文字に
            // ASCII 小文字のみを許容し数字始まりを拒否するため、chakra-ui の
            // `2xl`/`3xl`/`4xl` 表記ではなく `xl2`/`xl3`/`xl4`（enum variant 名と
            // 一致する表記）を使う。クラス名として出力可能にするための表記選択。
            Self::Xl2 => "xl2",
            Self::Xl3 => "xl3",
            Self::Xl4 => "xl4",
        }
    }
}

/// [`heading`] の設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct HeadingProps {
    /// 視覚サイズ variant（既定 `Xl`）。
    pub size: HeadingSize,
}

/// Heading の recipe（scope `"heading"`、slot `"root"` のみ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("heading", &["root"])
        .base(
            "root",
            vec![
                decl("margin", "0"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("letter-spacing", "-0.01em"),
            ],
        )
        .variant(
            HeadingSize::Xs,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("line-height", "1.3"),
            ],
        )
        .variant(
            HeadingSize::Sm,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "1.25"),
            ],
        )
        .variant(
            HeadingSize::Md,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-md)"),
                decl("line-height", "1.3"),
            ],
        )
        .variant(
            HeadingSize::Lg,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("line-height", "1.3"),
            ],
        )
        .variant(
            HeadingSize::Xl,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xl)"),
                decl("line-height", "1.3"),
            ],
        )
        .variant(
            HeadingSize::Xl2,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-2xl)"),
                decl("line-height", "1.25"),
            ],
        )
        .variant(
            HeadingSize::Xl3,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-3xl)"),
                decl("line-height", "1.2"),
            ],
        )
        .variant(
            HeadingSize::Xl4,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-4xl)"),
                decl("line-height", "1.15"),
            ],
        )
        .default_variant(HeadingSize::Xl)
}

/// Heading の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Heading 1 個を組み立てる。`level` がレンダリングするタグ（h1〜h6）を、
/// `props.size` が視覚サイズ variant を決める（両者は独立、モジュール冒頭
/// rustdoc 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
///
/// let node = heading(HeadingLevel::H1, &HeadingProps::default(), vec![], vec![text("Title")]);
/// let html = render(&node);
/// assert!(html.starts_with("<h1"));
/// assert!(html.contains("Title"));
/// ```
#[must_use]
pub fn heading<'a>(
    level: HeadingLevel,
    props: &HeadingProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", level.tag(), merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_h2_xl_size() {
        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![],
            vec![text("Title")],
        ));
        assert_eq!(
            html,
            r#"<h2 data-scope="heading" data-part="root" class="fd-heading--size-xl">Title</h2>"#
        );
    }

    #[test]
    fn level_enumeration_maps_to_expected_tags() {
        for (level, tag) in [
            (HeadingLevel::H1, "h1"),
            (HeadingLevel::H2, "h2"),
            (HeadingLevel::H3, "h3"),
            (HeadingLevel::H4, "h4"),
            (HeadingLevel::H5, "h5"),
            (HeadingLevel::H6, "h6"),
        ] {
            let html = render(&heading(level, &HeadingProps::default(), vec![], vec![]));
            assert!(
                html.starts_with(&format!("<{tag} ")),
                "level={level:?} -> {html}"
            );
            assert!(
                html.ends_with(&format!("</{tag}>")),
                "level={level:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (HeadingSize::Xs, "fd-heading--size-xs"),
            (HeadingSize::Sm, "fd-heading--size-sm"),
            (HeadingSize::Md, "fd-heading--size-md"),
            (HeadingSize::Lg, "fd-heading--size-lg"),
            (HeadingSize::Xl, "fd-heading--size-xl"),
            (HeadingSize::Xl2, "fd-heading--size-xl2"),
            (HeadingSize::Xl3, "fd-heading--size-xl3"),
            (HeadingSize::Xl4, "fd-heading--size-xl4"),
        ] {
            let props = HeadingProps { size };
            let html = render(&heading(HeadingLevel::default(), &props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_all_size_font_tokens() {
        let out = css();
        for token in [
            "font-size-xs",
            "font-size-sm",
            "font-size-md",
            "font-size-lg",
            "font-size-xl",
            "font-size-2xl",
            "font-size-3xl",
            "font-size-4xl",
        ] {
            assert!(
                out.contains(&format!("var(--fandhe-font-{token})")),
                "missing {token} in {out}"
            );
        }
    }
}
