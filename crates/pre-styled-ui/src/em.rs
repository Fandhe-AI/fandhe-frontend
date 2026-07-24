//! Em（イシュー #771）: variant を持たない最小静的部品。強調テキスト
//! （`<em>`）を既定スタイルで組み立てる。[`crate::link_overlay`] と同型の
//! 「variant 軸を持たない slot recipe」パターンに従う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="em"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("em");

/// Em の recipe（scope `"em"`、slot `"root"` のみ、variant 軸なし）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("em", &["root"]).base(
        "root",
        vec![
            decl("font-style", "italic"),
            decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        ],
    )
}

/// Em の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Em 1 個（`<em>`）を組み立てる。variant 軸を持たないため `class` 属性は
/// 付与しない（呼び出し側 `attrs` の `class` は他 styled 部品との一貫性の
/// ため [`drop_class_attr`] で除去する。[`crate::link_overlay::root`] と
/// 同型の判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::em::em;
///
/// let node = em(vec![], vec![text("important")]);
/// let html = render(&node);
/// assert!(html.starts_with("<em"));
/// assert!(html.contains("important"));
/// ```
#[must_use]
pub fn em<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "em", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_em_tag_with_scope_and_part() {
        let html = render(&em(vec![], vec![text("important")]));
        assert_eq!(
            html,
            r#"<em data-scope="em" data-part="root">important</em>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped() {
        let html = render(&em(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&em(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&em(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_italic_and_font_weight() {
        let out = css();
        assert!(out.contains("font-style: italic;"));
        assert!(out.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
