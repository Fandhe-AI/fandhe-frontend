//! Card（イシュー #550）: slot recipe styled 部品。root/header/body/footer/
//! title/description の 6 パーツで構成する装飾的コンテナ。
//!
//! 純粋なレイアウトコンテナであり、`role`/`aria-*` は付与しない
//! （`.claude/rules/coding-rust.md` 準拠のプレーンな HTML を尊重する方針）。
//! 組み立ての自由度を [`crate::alert`] や headless 層のパーツ関数群と同型に
//! 保つため、コンビニ関数（全部入り `card(...)`）は提供せず、各パーツを
//! 個別に呼び出して組み立てる契約とする（呼び出し例は各関数の rustdoc
//! `# Examples` を参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="card"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("card");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &["root", "header", "body", "footer", "title", "description"];

/// Card の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// 影付き（背景と分離感を強調）。
    Elevated,
    /// 輪郭のみ（既定）。
    #[default]
    Outline,
    /// 淡色背景。
    Subtle,
}

impl VariantValue for CardVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Elevated => "elevated",
            Self::Outline => "outline",
            Self::Subtle => "subtle",
        }
    }
}

/// Card の recipe（scope `"card"`、[`SLOTS`] の 6 パーツ）。
///
/// 中立的なレイアウトコンテナであり、Button/Badge/Spinner/Alert と異なり
/// colorPalette 軸は付与しない（イシュー #606。Card は特定のセマンティック
/// 色を持つ意味論を持たず、`header`/`footer` の枠線色等は既存どおり
/// `--fandhe-color-*` を直接参照する）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("card", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        .base(
            "header",
            vec![
                decl("padding", "1rem"),
                decl(
                    "border-bottom",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
            ],
        )
        .base("body", vec![decl("padding", "1rem")])
        .base(
            "footer",
            vec![
                decl("padding", "1rem"),
                decl("border-top", "1px solid var(--fandhe-color-border-muted)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
            ],
        )
        .base(
            "description",
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        .variant(
            CardVariant::Elevated,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
            ],
        )
        .variant(
            CardVariant::Outline,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
            ],
        )
        .variant(
            CardVariant::Subtle,
            "root",
            vec![decl("background", "var(--fandhe-color-bg-subtle)")],
        )
        .default_variant(CardVariant::Outline)
}

/// Card の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`variant` に応じたクラスを付与する唯一のパーツ
/// （`class_attr::drop_class_attr` により呼び出し側の `class` は除去してから
/// 合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
///
/// let node = card::root(CardVariant::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="card" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(variant: CardVariant, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// header パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "div", attrs, children)
}

/// body パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("body", "div", attrs, children)
}

/// footer パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "div", attrs, children)
}

/// title パーツ（`<h3>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "h3", attrs, children)
}

/// description パーツ（`<p>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "p", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_outline() {
        let html = render(&root(CardVariant::default(), vec![], vec![]));
        assert!(html.contains("fd-card--variant-outline"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (CardVariant::Elevated, "fd-card--variant-elevated"),
            (CardVariant::Outline, "fd-card--variant-outline"),
            (CardVariant::Subtle, "fd-card--variant-subtle"),
        ] {
            let html = render(&root(variant, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&header(vec![], vec![]))
            .starts_with(r#"<div data-scope="card" data-part="header""#));
        assert!(
            render(&body(vec![], vec![])).starts_with(r#"<div data-scope="card" data-part="body""#)
        );
        assert!(render(&footer(vec![], vec![]))
            .starts_with(r#"<div data-scope="card" data-part="footer""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<h3 data-scope="card" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<p data-scope="card" data-part="description""#));
    }

    #[test]
    fn composed_card_snapshot() {
        let node = root(
            CardVariant::Elevated,
            vec![],
            vec![
                header(vec![], vec![title(vec![], vec![text("Title")])]),
                body(vec![], vec![text("Body")]),
                footer(vec![], vec![text("Footer")]),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="card" data-part="root" class="fd-card--variant-elevated">"#,
                r#"<div data-scope="card" data-part="header">"#,
                r#"<h3 data-scope="card" data-part="title">Title</h3>"#,
                r#"</div>"#,
                r#"<div data-scope="card" data-part="body">Body</div>"#,
                r#"<div data-scope="card" data-part="footer">Footer</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            CardVariant::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// イシュー #606: recipe の静的 CSS に radii/shadow トークン参照が
    /// 含まれることを固定する。
    #[test]
    fn css_output_declares_radius_and_shadow_tokens() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(out.contains("box-shadow: var(--fandhe-shadow-sm);"));
    }
}
