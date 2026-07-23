//! styled NavList（headless ラッパー、イシュー #756、#716 最優先候補の消化）。
//!
//! `fandhe_frontend_headless_ui::nav_list`（イシュー #756）の Root / Heading /
//! List / Item / Link の 5 anatomy パーツを薄く再利用し、[`stylesheet`] で
//! 文書ナビの既定 CSS（list-style 除去・現在位置ハイライト）を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は [`crate::breadcrumb`]/[`crate::avatar`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`root` のみ再定義する理由）
//!
//! [`crate::breadcrumb`] と同型で、styled `root`（呼び出し側 `class` を
//! [`drop_class_attr`] で除去する唯一のパーツ）と headless の自由関数
//! `root` が名前衝突するため、それ以外のパーツ（[`heading`]/[`list`]/
//! [`item`]/[`link`]）のみを選択的に再エクスポートする。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（headless 層 → [`fandhe_frontend_core::render`]
//! の既定エスケープを必ず経由し、`raw_html()` の新規使用なし、`href` の URL
//! スキーム検証は headless 層が担う）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `fandhe-frontend-docs-site` はサイト骨格の自己完結不変条件
//!   （`site/assets/site.css`、`docs/design/docs-site-styled-ui-adoption.md`
//!   §3.4）を維持するため、本モジュールの styled `root`/`stylesheet` では
//!   なく headless 再エクスポート（[`heading`]/[`list`]/[`item`]/[`link`]）
//!   のみを直接使う。styled `root`（class 付与）は本クレートに直接依存する
//!   利用者（`examples/headless-pre-styled-ui` 等）向けの提供に留める。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::nav_list::{heading, item, link, list};

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/nav_list.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "heading", "list", "item", "link"];

/// この styled NavList の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("nav-list", SLOTS)
        .base(
            "heading",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "list",
            vec![
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .base(
            "link",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-decoration", "none"),
            ],
        )
        .state(
            "link",
            crate::recipe::StateCondition::AttrEq("aria-current", "page"),
            vec![
                decl(
                    "color",
                    "var(--fandhe-color-accent, var(--fandhe-color-fg))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
}

/// この styled NavList が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる（[`drop_class_attr`] により呼び出し側の
/// `class` は除去する）。実体は
/// [`fandhe_frontend_headless_ui::nav_list::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::nav_list;
///
/// let node = nav_list::root("Documentation", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="nav-list" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::nav_list::root(label, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_and_aria_label() {
        let html = render(&root("Documentation", vec![], vec![]));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Documentation""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn reexported_parts_render_expected_tags_without_role() {
        let heading_html = render(&heading(vec![], vec![text("Guides")]));
        assert!(heading_html.starts_with("<h2"));

        let list_html = render(&list(
            vec![],
            vec![item(
                vec![],
                vec![link("/docs", false, vec![], vec![text("Docs")])],
            )],
        ));
        assert!(list_html.contains("<ul"));
        assert!(list_html.contains("<li"));
        assert!(list_html.contains(r#"href="/docs""#));
        assert!(!list_html.contains("role="));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "Documentation",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="nav-list""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_from_caller_is_dropped() {
        let html = render(&root(
            "Documentation",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_current_state_selector() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[aria-current="page"]"#));
        assert!(a.contains("list-style: none"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_label_is_escaped() {
        let html = render(&root("\"><script>alert(1)</script>", vec![], vec![]));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
