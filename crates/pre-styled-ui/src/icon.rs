//! Icon（イシュー #770）: 単一 recipe styled 部品。インライン SVG の寸法
//! （`size` variant）・配色（`currentColor` 継承）を統一する `<svg>` ラッパー
//! （chakra-ui `data-display/icon` 相当）。
//!
//! SVG 本体（`path`/`circle` 等の子ノード）は本モジュールでは持たず、
//! 呼び出し側が [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]
//! でノード木として構築したものを [`icon`] の `children` 引数へ渡す。この
//! 子ノードの属性（`d`・`fill` 個別上書き等）にも
//! `fandhe_frontend_core::render` の既定エスケープと URL 属性検証
//! （`xlink:href` は [`fandhe_frontend_core::URL_ATTRS`] に収載済み）が
//! そのまま適用される。本モジュール自身は外部リソースを一切参照しない
//! （`href`/`xlink:href` を自ら出力しない）。
//!
//! `size`（[`crate::recipe::Size`]、寸法スケール）のみを variant として持ち、
//! `color-palette` 軸は提供しない（アイコンの配色は `color: currentColor`
//! 継承により祖先要素の文字色にそのまま追従させる設計判断であり、
//! [`crate::spinner`]・[`crate::badge`] のようにアイコン自身が状態/意味を
//! 持つ palette 軸を必要としない。将来個別のアクセント色が必要になれば
//! 非破壊的に追加できる）。
//!
//! # イシュー #1561 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`Icon`、`iconRecipe` の `size` 軸: xs=3=0.75rem/
//! sm=4=1rem/md=5=1.25rem/lg=6=1.5rem/xl=7=1.75rem/2xl=8=2rem、既定
//! `inherit`。`variant`/`colorPalette` 軸なし。ark-ui / Radix Themes /
//! Radix Primitives には対応部品がない。Radix `AccessibleIcon` は #1066 で
//! 代替検証済みの別論点）とスクリーンショット
//! （`docs/design/reference-screenshots/chakra-icon-{1,2,3}.png`・
//! `themes-icon.png`）を比較した結果を記録する。
//!
//! - **サイズ**: [`crate::recipe::Size`] の Xs〜Xl の実寸を、#1681 時点の
//!   等差外挿（Xs=0.5rem/Sm=1rem/Md=1.5rem/Lg=2rem/Xl=2.5rem）から chakra
//!   の同名段の実寸（Xs=0.75rem/Sm=1rem/Md=1.25rem/Lg=1.5rem/Xl=1.75rem）
//!   へ是正した（color-swatch #1558 と同型の判断）。`SlotRecipe::size_variants`
//!   経由にすることで既定 `Md` を構造的に保証する
//!   （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §4）。chakra の `2xl`（2rem）は共通語彙 `recipe::Size`（5 段）に
//!   存在しないため採らない。chakra の既定 `inherit`（寸法宣言なしで
//!   祖先フォントサイズへ追従）は「size 軸を持つ styled 部品は既定が
//!   必ず `md`」という規約（同文書 §4）と衝突するため意図的に合わせず、
//!   既定 `Md`（1.25rem 固定）を維持する。
//! - **バリアント**: chakra `Icon` は `variant` 軸を持たない。増減なし。
//! - **色**: `color: currentColor` 継承のみ（変更なし）。chakra の
//!   `colorPalette`/`color` は recipe 外のスタイル prop であり、本部品も
//!   palette 軸は非提供のまま（既存の設計判断を維持）。生の色リテラルは
//!   持ち込まない。
//! - **状態（`data-*`）**: 増減なし。`data-scope`/`data-part` のみ。
//! - **ダーク**: 非適用（意図的）。`currentColor` 継承のため祖先の
//!   文字色トークン再定義に自動追従し、部品固有の色宣言を持たない。
//! - **フォーカス**: 非適用（意図的）。`focusable="false"` の
//!   非インタラクティブ `<svg>` であり #1424 の適用対象外。
//! - **余白・角丸・影**: 変更なし。参照にも無い。chakra base の
//!   `line-height: 1em` は、本実装の root が常に `<svg>`（インライン
//!   内容を持たない）であるため line box に影響せず、追加を見送る。
//! - **hover / disabled / transition**: 非適用（意図的）。
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 が
//!   「表示専用には hover を付けない」と明記しており、disabled 概念・
//!   遷移対象もない。
//!
//! ## スコープ外（変更しない点）
//!
//! [`crate::spinner`] の size 実寸（現在 icon と同じ旧等差外挿）は #1567
//! が担当し、本イシューでは触らない。chakra `2xl`/`inherit` 段の追加は
//! 共通語彙 `recipe::Size` の拡張論点であり本イシュー外（親 #1420 配下で
//! 別途提案）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, role, Anatomy};

/// `data-scope="icon"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("icon");

/// Icon の recipe（scope `"icon"`、slot `"root"` のみ）。
///
/// `Size` 軸の寸法は chakra-ui `iconRecipe` の同名段の実寸（イシュー
/// #1561）に整合させる。#1681 時点の等差外挿（0.5rem 刻み）から
/// chakra 同名段（xs=0.75rem/sm=1rem/md=1.25rem/lg=1.5rem/xl=1.75rem）へ
/// 是正済み。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("icon", &["root"])
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("flex-shrink", "0"),
                decl("color", "currentColor"),
                decl("vertical-align", "middle"),
            ],
        )
        // イシュー #1561: #1681 時点の 0.5rem 刻み等差外挿
        // （Xs=0.5rem/Sm=1rem/Md=1.5rem/Lg=2rem/Xl=2.5rem）から chakra-ui
        // `iconRecipe` の同名段の実寸へ是正。`SlotRecipe::size_variants`
        // 経由にすることで既定 `Md` を構造的に保証する
        // （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §4）。chakra の `2xl`（2rem）/`inherit`（既定・寸法宣言なし）は
        // 共通語彙 `recipe::Size`（5 段）・§4 の「既定は必ず `md`」規約と
        // 衝突するため採らない（モジュール冒頭「イシュー #1561 の参照
        // サイト比較」参照）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![decl("width", "0.75rem"), decl("height", "0.75rem")],
                ),
                (
                    Size::Sm,
                    vec![decl("width", "1rem"), decl("height", "1rem")],
                ),
                (
                    Size::Md,
                    vec![decl("width", "1.25rem"), decl("height", "1.25rem")],
                ),
                (
                    Size::Lg,
                    vec![decl("width", "1.5rem"), decl("height", "1.5rem")],
                ),
                (
                    Size::Xl,
                    vec![decl("width", "1.75rem"), decl("height", "1.75rem")],
                ),
            ],
        )
}

/// Icon の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// [`icon`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct IconProps<'a> {
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
    /// アクセシブルネーム。`Some` なら `role="img"` + `aria-label` を付与し
    /// スクリーンリーダーへ意味のあるアイコンとして伝える。`None`（既定）
    /// なら装飾用途とみなし `aria-hidden="true"` を付与する
    /// （[`crate::spinner::spinner_decorative`] と同型の判断）。
    pub label: Option<&'a str>,
    /// `viewBox` 属性値（既定 `"0 0 24 24"`）。既定エスケープを経由する。
    pub view_box: &'a str,
}

impl<'a> Default for IconProps<'a> {
    fn default() -> Self {
        IconProps {
            size: Size::Md,
            label: None,
            view_box: "0 0 24 24",
        }
    }
}

/// Icon 1 個を組み立てる（`<svg>`。`children` は呼び出し側が構築する
/// `path` 等の SVG ノード木、本モジュール doc 参照）。
///
/// `fill="currentColor"` を固定で付与し（呼び出し側の色制御は祖先の
/// `color` プロパティ経由に一本化する）、`focusable="false"` で IE 系の
/// 既定フォーカス挙動を抑止する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{el, render};
/// use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
///
/// let node = icon(
///     &IconProps::default(),
///     vec![],
///     vec![el("path", vec![("d", "M12 2L2 22h20z")], vec![])],
/// );
/// let html = render(&node);
/// assert!(html.contains("<svg"));
/// assert!(html.contains(r#"aria-hidden="true""#));
/// ```
#[must_use]
pub fn icon<'a>(
    props: &IconProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    // `merged` の型を `Vec<(&'a str, &'a str)>` と明示しない（推論に任せる）:
    // ローカル変数 `class` の借用寿命は `'a` より短いため、明示すると
    // 「`class` は `'a` まで生存しない」という借用エラーになる（badge/card
    // 等の既存 styled 部品と同じ回避策、`crate::badge::badge` 参照）。
    let mut merged: Vec<(&str, &str)> = vec![
        ("class", class.as_str()),
        ("viewBox", props.view_box),
        ("fill", "currentColor"),
        ("focusable", "false"),
    ];
    match props.label {
        Some(label) => {
            merged.push(role("img"));
            merged.push(aria_label(label));
        }
        None => merged.push(aria_hidden(true)),
    }
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "svg", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, render};

    #[test]
    fn default_props_render_md_size_decorative() {
        let node = icon(&IconProps::default(), vec![], vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<svg data-scope="icon" data-part="root" class="fd-icon--size-md" viewBox="0 0 24 24" fill="currentColor" focusable="false" aria-hidden="true"></svg>"#
        );
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-icon--size-xs"),
            (Size::Sm, "fd-icon--size-sm"),
            (Size::Md, "fd-icon--size-md"),
            (Size::Lg, "fd-icon--size-lg"),
            (Size::Xl, "fd-icon--size-xl"),
        ] {
            let props = IconProps {
                size,
                ..IconProps::default()
            };
            let html = render(&icon(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(r#"class="{class}""#)),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_md_is_default_variant() {
        // イシュー #1561: `size_variants` 経由でも既定が `Md`
        // （1.25rem）のまま保たれることを固定する（前掲 rustdoc の
        // 「既定は必ず `md`」規約の裏付け）。
        let html_default = render(&icon(&IconProps::default(), vec![], vec![]));
        let props_md = IconProps {
            size: Size::Md,
            ..IconProps::default()
        };
        let html_md = render(&icon(&props_md, vec![], vec![]));
        assert_eq!(html_default, html_md);
        assert!(html_default.contains(r#"class="fd-icon--size-md""#));
    }

    #[test]
    fn css_has_no_raw_color_literals() {
        // イシュー #1561: 色は `currentColor` 継承のみで、生の色リテラル
        // （16 進数・`rgb(`/`rgba(`）を持ち込まないことを固定する
        // （モジュール冒頭「イシュー #1561 の参照サイト比較」の「色」節）。
        let css = css();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn label_some_switches_to_role_img_and_aria_label() {
        let props = IconProps {
            label: Some("Search"),
            ..IconProps::default()
        };
        let html = render(&icon(&props, vec![], vec![]));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="Search""#));
        assert!(!html.contains("aria-hidden"));
    }

    #[test]
    fn view_box_and_label_are_escaped() {
        let props = IconProps {
            label: Some("\"><script>alert(1)</script>"),
            view_box: "0 0 24 24\"><script>alert(1)</script>",
            ..IconProps::default()
        };
        let html = render(&icon(&props, vec![], vec![]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn children_svg_path_attrs_are_escaped() {
        let node = icon(
            &IconProps::default(),
            vec![],
            vec![el(
                "path",
                vec![("d", "M0 0\"><script>alert(1)</script>")],
                vec![],
            )],
        );
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn children_xlink_href_javascript_scheme_is_not_output() {
        // Icon 自身は外部リソースを参照しないが、children 経由で渡された
        // `xlink:href` にも core の URL_ATTRS 検証がそのまま適用されることを
        // 固定する（本モジュール doc の「外部リソース非参照」契約の裏付け）。
        let node = icon(
            &IconProps::default(),
            vec![],
            vec![el(
                "use",
                vec![("xlink:href", "javascript:alert(1)")],
                vec![],
            )],
        );
        let html = render(&node);
        assert!(!html.contains("xlink:href"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&icon(
            &IconProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn css_output_declares_size_and_currentcolor() {
        // イシュー #1561: chakra-ui `iconRecipe` 同名段の実寸へ是正済み
        // （0.75rem/1rem/1.25rem/1.5rem/1.75rem）。
        let out = css();
        assert!(out.contains("color: currentColor;"));
        assert!(out.contains("width: 0.75rem;"));
        assert!(out.contains("width: 1rem;"));
        assert!(out.contains("width: 1.25rem;"));
        assert!(out.contains("width: 1.5rem;"));
        assert!(out.contains("width: 1.75rem;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
