//! Quote（イシュー #995）: variant を持たない最小静的部品。短いインライン
//! 引用（`<q>`）を既定スタイルで組み立てる。[`crate::em`] / [`crate::link_overlay`]
//! と同型の「variant 軸を持たない slot recipe」パターンに従う。
//!
//! [`crate::blockquote`]（`<figure>`/`<blockquote>`/`<figcaption>` の
//! ブロック引用）との役割差: `quote` は文中に埋め込む短いインライン引用
//! （Radix Themes の Quote と同じ位置付け）であり、ブロックレベルの構造・
//! 出典表示（`caption` パーツ）を持たない。ブロック引用が必要な場合は
//! [`crate::blockquote`] を使う。
//!
//! ブラウザ既定の引用符（`q::before`/`q::after` の生成コンテンツ）は本
//! recipe が意図的に上書きしない（[`crate::recipe::SlotRecipe`] は宣言のみを
//! 持つため擬似要素規則は元々表現できない）。`font-style: italic` の 1 宣言
//! のみを base として持たせる。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="quote"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("quote");

/// Quote の recipe（scope `"quote"`、slot `"root"` のみ、variant 軸なし）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("quote", &["root"]).base("root", vec![decl("font-style", "italic")])
}

/// Quote の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Quote 1 個（`<q>`）を組み立てる。variant 軸を持たないため `class` 属性は
/// 付与しない（呼び出し側 `attrs` の `class` は他 styled 部品との一貫性の
/// ため [`drop_class_attr`] で除去する。[`crate::em::em`] と同型の判断）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::quote::quote;
///
/// let node = quote(vec![], vec![text("to be or not to be")]);
/// let html = render(&node);
/// assert!(html.starts_with("<q"));
/// assert!(html.contains("to be or not to be"));
/// ```
#[must_use]
pub fn quote<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "q", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_q_tag_with_scope_and_part() {
        let html = render(&quote(vec![], vec![text("important")]));
        assert_eq!(
            html,
            r#"<q data-scope="quote" data-part="root">important</q>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped() {
        let html = render(&quote(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&quote(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let html = render(&quote(
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_italic() {
        let out = css();
        assert!(out.contains("font-style: italic;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }
}
