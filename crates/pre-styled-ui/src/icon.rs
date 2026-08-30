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

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, aria_label, role, Anatomy};

/// `data-scope="icon"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("icon");

/// Icon の recipe（scope `"icon"`、slot `"root"` のみ）。
///
/// `Size` 軸の寸法は [`crate::spinner`] の `size` variant と同じスケール
/// （1rem/1.5rem/2rem）を採用し、styled 部品間で寸法感覚を揃える。
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
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の 0.5rem 刻み等差進行を外挿。
        .variant(
            Size::Xs,
            "root",
            vec![decl("width", "0.5rem"), decl("height", "0.5rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("width", "1rem"), decl("height", "1rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("width", "1.5rem"), decl("height", "1.5rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("width", "2rem"), decl("height", "2rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("width", "2.5rem"), decl("height", "2.5rem")],
        )
        .default_variant(Size::Md)
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
            (Size::Sm, "fd-icon--size-sm"),
            (Size::Md, "fd-icon--size-md"),
            (Size::Lg, "fd-icon--size-lg"),
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
        let out = css();
        assert!(out.contains("color: currentColor;"));
        assert!(out.contains("width: 1rem;"));
        assert!(out.contains("width: 2rem;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
