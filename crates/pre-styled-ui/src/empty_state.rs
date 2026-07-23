//! EmptyState（イシュー #765）: slot recipe styled 部品。indicator/title/
//! description/actions を持つ空状態レイアウトコンテナ。
//!
//! [`crate::card`] と同じく中立的なレイアウトコンテナであり、`role`/
//! `aria-*` は付与しない（`.claude/rules/coding-rust.md` 準拠のプレーンな
//! HTML を尊重する方針）。特定のセマンティック色を持つ意味論を持たないため
//! `color-palette` 軸は提供しない（[`crate::card`] rustdoc と同型の判断）。
//! `title` を見出し要素（`<h1>`〜`<h6>`）にせず `<div>` とするのは、
//! `fandhe-frontend-docs-site` の showcase が `.docs-content h3` 等の
//! セレクタで見出しを拾うテスト・スタイルを持ち、部品埋め込み位置に応じて
//! 見出しレベルが変わり得る呼び出し文脈では固定レベルの見出し要素を強制
//! しない方が安全という判断（[`crate::alert::title`] と同型の判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

use crate::recipe::{Size, SlotRecipe, VariantValue};

/// `data-scope="empty-state"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("empty-state");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "content",
    "indicator",
    "title",
    "description",
    "actions",
];

/// [`root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct EmptyStateProps {
    /// サイズ variant（既定 `Md`。root の padding を切り替える）。
    pub size: Size,
}

impl Default for EmptyStateProps {
    fn default() -> Self {
        EmptyStateProps { size: Size::Md }
    }
}

/// EmptyState の recipe（scope `"empty-state"`、[`SLOTS`] の 6 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("empty-state", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2, 0.5rem)"),
                decl("text-align", "center"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("font-size", "2rem"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "title",
            vec![decl(
                "font-weight",
                "var(--fandhe-font-font-weight-semibold)",
            )],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "actions",
            vec![
                decl("display", "flex"),
                decl("gap", "var(--fandhe-space-2, 0.5rem)"),
                decl("margin-top", "var(--fandhe-space-2, 0.5rem)"),
            ],
        )
        .variant(Size::Sm, "root", vec![decl("padding", "2rem")])
        .variant(Size::Md, "root", vec![decl("padding", "3rem")])
        .variant(Size::Lg, "root", vec![decl("padding", "4rem")])
        .default_variant(Size::Md)
}

/// EmptyState の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<div>`）を組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`crate::class_attr::drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
///
/// let node = empty_state::root(&EmptyStateProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="empty-state" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &EmptyStateProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// content パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// indicator パーツ（`<span>`）を組み立てる。アイコン等を子ノードとして
/// 受け取る（本クレートは外部リソース・アイコンフォントを参照しない方針の
/// ため、具体的な意匠は呼び出し側が children として渡す）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "span", attrs, children)
}

/// title パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "div", attrs, children)
}

/// description パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "div", attrs, children)
}

/// actions パーツ（`<div>`）を組み立てる。ボタン等の操作導線を並べる。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("actions", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(html.contains("fd-empty-state--size-md"));
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-empty-state--size-sm"),
            (Size::Md, "fd-empty-state--size-md"),
            (Size::Lg, "fd-empty-state--size-lg"),
        ] {
            let props = EmptyStateProps { size };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="content""#));
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<span data-scope="empty-state" data-part="indicator""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="description""#));
        assert!(render(&actions(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="actions""#));
    }

    #[test]
    fn composed_empty_state_snapshot() {
        let node = root(
            &EmptyStateProps::default(),
            vec![],
            vec![content(
                vec![],
                vec![
                    indicator(vec![], vec![]),
                    title(vec![], vec![text("No results")]),
                    description(vec![], vec![text("Try a different search.")]),
                    actions(vec![], vec![]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="empty-state" data-part="root" class="fd-empty-state--size-md">"#,
                r#"<div data-scope="empty-state" data-part="content">"#,
                r#"<span data-scope="empty-state" data-part="indicator"></span>"#,
                r#"<div data-scope="empty-state" data-part="title">No results</div>"#,
                r#"<div data-scope="empty-state" data-part="description">Try a different search.</div>"#,
                r#"<div data-scope="empty-state" data-part="actions"></div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn root_has_no_role_attribute() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(!html.contains("role="));
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &EmptyStateProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_and_description_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let html = render(&description(
            vec![],
            vec![text("<script>alert(2)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_padding_and_muted_fg_tokens() {
        let out = css();
        assert!(out.contains("padding: 3rem;"));
        assert!(out.contains("color: var(--fandhe-color-fg-muted);"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
