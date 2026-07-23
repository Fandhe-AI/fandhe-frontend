//! Code（イシュー #768）: 単一 slot styled 部品。インラインコード片表示の
//! ための `<code>` を組み立てる。
//!
//! chakra-ui v3 の `typography/code-block.md`（CodeBlock）は
//! `docs/design/component-coverage-map.md` で対象外確定済みであり、本
//! モジュールはインライン `<code>` のみを扱う。[`crate::kbd`] と同じ理由で
//! variant 軸を持たず、`class` 属性は付与しない（呼び出し側 `attrs` の
//! `class` は [`crate::class_attr::drop_class_attr`] で破棄する）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="code"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("code");

/// Code の recipe（scope `"code"`、slot `"root"` のみ、variant 軸なし）。
///
/// `font-family` は [`crate::kbd`] と同じ固定フォントスタック文字列を使う
/// （[`crate::theme`] に mono フォントトークンが存在しないため）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("code", &["root"]).base(
        "root",
        vec![
            decl(
                "font-family",
                "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
            ),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("border-radius", "var(--fandhe-radius-sm)"),
            decl("padding", "0.0625rem 0.375rem"),
            decl("font-size", "var(--fandhe-font-font-size-sm)"),
        ],
    )
}

/// Code の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Code 片 1 個（`<code>`）を組み立てる。呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] により破棄する（[`crate::kbd::kbd`]
/// と同型の契約一貫性）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::code::code;
///
/// let node = code(vec![], vec![text("cargo build")]);
/// assert!(render(&node).contains("cargo build"));
/// ```
#[must_use]
pub fn code<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "code", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_render_has_no_class_attribute() {
        let html = render(&code(vec![], vec![text("cargo build")]));
        assert_eq!(
            html,
            r#"<code data-scope="code" data-part="root">cargo build</code>"#
        );
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&code(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&code(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }
}
