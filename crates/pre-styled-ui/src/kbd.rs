//! Kbd（イシュー #768）: 単一 slot styled 部品。キーボード入力・ショート
//! カット表示のための `<kbd>` を組み立てる。
//!
//! [`crate::spinner`]（#550）と同様に状態機械を要しない静的マークアップ
//! 部品だが、variant 軸を持たない点が異なる（受け入れ条件が Size/
//! ColorPalette を [`crate::tag`] のみに限定しているため）。variant が
//! ないため `class` 属性は付与せず、CSS は
//! `[data-scope="kbd"][data-part="root"]` の base 宣言のみで表現する
//! （[`crate::alert`] の非 root パーツと同型）。呼び出し側 `attrs` の
//! `class` は他部品との契約一貫性のため
//! [`crate::class_attr::drop_class_attr`] で破棄する。
//!
//! variant 軸の追加（chakra-ui の `raised`/`outline`/`subtle`/`plain`）は
//! 非破壊で後から可能（イシュー #768 計画 §9 参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="kbd"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("kbd");

/// Kbd の recipe（scope `"kbd"`、slot `"root"` のみ、variant 軸なし）。
///
/// `font-family` は [`crate::theme`] に mono フォントトークンが存在しない
/// ため固定のフォントスタック文字列を直接宣言する（存在しないトークンを
/// 新設しない方針、イシュー #768 計画 §3.2）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("kbd", &["root"]).base(
        "root",
        vec![
            decl(
                "font-family",
                "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
            ),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("border", "1px solid var(--fandhe-color-border)"),
            decl("border-bottom-width", "2px"),
            decl("border-radius", "var(--fandhe-radius-sm)"),
            decl("padding", "0.0625rem 0.375rem"),
            decl("font-size", "var(--fandhe-font-font-size-xs)"),
        ],
    )
}

/// Kbd の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Kbd 1 個（`<kbd>`）を組み立てる。呼び出し側 `attrs` の `class` は
/// [`crate::class_attr::drop_class_attr`] により破棄する（variant を持たず
/// クラスを自ら付与しない部品でも、他部品との `class` 破棄契約を一貫させ、
/// 呼び出し側が誤って動的クラスを合成する余地を残さないため）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::kbd::kbd;
///
/// let node = kbd(vec![], vec![text("Ctrl")]);
/// assert!(render(&node).contains("Ctrl"));
/// ```
#[must_use]
pub fn kbd<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "kbd", drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_render_has_no_class_attribute() {
        let html = render(&kbd(vec![], vec![text("Ctrl")]));
        assert_eq!(html, r#"<kbd data-scope="kbd" data-part="root">Ctrl</kbd>"#);
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&kbd(vec![("class", "attacker-controlled")], vec![]));
        assert!(!html.contains("class="));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&kbd(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-sm);"));
    }
}
