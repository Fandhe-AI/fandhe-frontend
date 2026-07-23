//! styled Breadcrumb（headless ラッパー、イシュー #755、#716 追加候補の消化）。
//!
//! `fandhe_frontend_headless_ui::breadcrumb`（イシュー #755）の Root / List /
//! Item / Link / CurrentLink / Separator / Ellipsis 7 anatomy パーツを薄く
//! 再利用し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・
//! スコープ外事項は [`crate::avatar`]/[`crate::card`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`root` のみ再定義する理由）
//!
//! [`crate::avatar`] と同型で、styled `root`（`size`/`variant` クラス付与の
//! ため本モジュールで再定義）と headless の自由関数 `root` が名前衝突する
//! ため、それ以外のパーツ（[`list`]/[`item`]/[`link`]/[`current_link`]/
//! [`separator`]/[`ellipsis`]）・[`BreadcrumbItem`] のみを選択的に
//! 再エクスポートする（`root` を除く headless anatomy 関数一式）。
//!
//! # variant（size/variant）について
//!
//! Breadcrumb は 2 軸の variant を持つ（chakra-ui Breadcrumb の
//! size/variant を最小構成へ縮約）:
//!
//! - `size`（[`crate::recipe::Size`]）: `root` の `font-size` を切り替える。
//! - [`BreadcrumbVariant`]（`Plain`（既定）/`Underline`）: `link` の
//!   `text-decoration` を切り替える。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `link`/`current-link` への伝搬は `root` の variant 宣言が登録する
//! root スコープの CSS custom property（`--fandhe-breadcrumb-link-text-decoration`
//! 等）の通常の CSS 継承で行い、[`recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない（[`crate::switch`] と同型のパターン）。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の URL スキーム検証は headless
//!   層（`crates/headless-ui/src/breadcrumb.rs` rustdoc 参照）が担う。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//! - styled [`root`] は headless
//!   [`fandhe_frontend_headless_ui::breadcrumb::root`] へ委譲するため、
//!   呼び出し側 `attrs` の `data-scope`/`data-part` 偽装除去（headless
//!   anatomy の fail-closed 挙動）をそのまま継承する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `docs/design/docs-site-styled-ui-adoption.md` §3.1/§3.2（Link リスト /
//!   LinkOverlay）の再評価は Link 系実装イシューのスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
pub use fandhe_frontend_headless_ui::breadcrumb::{
    current_link, ellipsis, item, link, list, separator, BreadcrumbItem,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/breadcrumb.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "list",
    "item",
    "link",
    "current-link",
    "separator",
    "ellipsis",
];

/// `link` の見た目（chakra-ui Breadcrumb の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreadcrumbVariant {
    /// 下線なし（既定）。
    #[default]
    Plain,
    /// 常時下線表示。
    Underline,
}

impl VariantValue for BreadcrumbVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Underline => "underline",
        }
    }
}

/// この styled Breadcrumb の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("breadcrumb", SLOTS)
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "0.375rem"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-breadcrumb-font-size, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "0.375rem"),
            ],
        )
        .base(
            "link",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "text-decoration",
                    "var(--fandhe-breadcrumb-link-text-decoration, none)",
                ),
            ],
        )
        .base(
            "current-link",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("color", "var(--fandhe-color-fg-subtle)"),
            ],
        )
        .base(
            "ellipsis",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("color", "var(--fandhe-color-fg-subtle)"),
            ],
        )
        .variant(
            crate::recipe::Size::Sm,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            crate::recipe::Size::Md,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            crate::recipe::Size::Lg,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-font-size",
                "var(--fandhe-font-font-size-lg)",
            )],
        )
        .default_variant(crate::recipe::Size::Md)
        .variant(
            BreadcrumbVariant::Plain,
            "root",
            vec![decl("--fandhe-breadcrumb-link-text-decoration", "none")],
        )
        .variant(
            BreadcrumbVariant::Underline,
            "root",
            vec![decl(
                "--fandhe-breadcrumb-link-text-decoration",
                "underline",
            )],
        )
        .default_variant(BreadcrumbVariant::Plain)
}

/// この styled Breadcrumb が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`variant` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::breadcrumb::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = breadcrumb::root(Size::Md, BreadcrumbVariant::default(), None, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="breadcrumb" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: crate::recipe::Size,
    variant: BreadcrumbVariant,
    aria_label_value: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value()), ("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::breadcrumb::root(aria_label_value, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    // --- anatomy ---

    #[test]
    fn root_outputs_scope_and_part_with_default_aria_label() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="breadcrumb""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="breadcrumb""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn reexported_parts_render_expected_tags() {
        let html = render(&list(
            vec![],
            vec![item(
                vec![],
                vec![link("/docs", vec![], vec![text("Docs")])],
            )],
        ));
        assert!(html.contains("<ol"));
        assert!(html.contains("<li"));
        assert!(html.contains(r#"href="/docs""#));

        let current_html = render(&current_link(vec![], vec![text("Breadcrumb")]));
        assert!(current_html.contains(r#"aria-current="page""#));

        let sep_html = render(&separator(vec![], vec![text("/")]));
        assert!(sep_html.contains(r#"role="presentation""#));

        let ellipsis_html = render(&ellipsis(vec![]));
        assert!(ellipsis_html.contains(r#"data-part="ellipsis""#));
    }

    // --- variant クラス ---

    #[test]
    fn default_variant_is_md_and_plain() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::default(),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-breadcrumb--size-md"));
        assert!(html.contains("fd-breadcrumb--variant-plain"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (crate::recipe::Size::Sm, "fd-breadcrumb--size-sm"),
            (crate::recipe::Size::Md, "fd-breadcrumb--size-md"),
            (crate::recipe::Size::Lg, "fd-breadcrumb--size-lg"),
        ] {
            let html = render(&root(size, BreadcrumbVariant::Plain, None, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (BreadcrumbVariant::Plain, "fd-breadcrumb--variant-plain"),
            (
                BreadcrumbVariant::Underline,
                "fd-breadcrumb--variant-underline",
            ),
        ] {
            let html = render(&root(
                crate::recipe::Size::Md,
                variant,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_variant_selectors_and_tokens() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--size-"));
        assert!(a.contains("--variant-"));
        assert!(a.contains("--fandhe-breadcrumb-link-text-decoration"));
        assert!(a.contains("var(--fandhe-color-fg)"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            BreadcrumbVariant::Plain,
            None,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn link_children_script_payload_is_escaped() {
        let html = render(&link(
            "/docs",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn javascript_scheme_href_is_dropped() {
        let html = render(&link("javascript:alert(1)", vec![], vec![]));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("href="));
    }
}
