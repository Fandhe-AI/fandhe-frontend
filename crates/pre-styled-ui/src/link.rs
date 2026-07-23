//! styled Link（headless ラッパー、イシュー #756、#716 追加候補の消化）。
//!
//! `fandhe_frontend_headless_ui::link`（イシュー #756）の唯一の anatomy パーツ
//! `root` を薄く再利用し、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::breadcrumb`]/[`crate::avatar`] の
//! rustdoc と同じ方針に従う。
//!
//! # variant（chakra-ui Link の `variant` を最小構成へ縮約）
//!
//! [`LinkVariant`]（`Plain`（既定）/`Underline`）が `text-decoration` を
//! 切り替える。クラスは唯一の anatomy パーツ `root` へ付与する。
//!
//! # `current` 状態の装飾
//!
//! [`crate::recipe::StateCondition::AttrEq`] で `aria-current="page"` を
//! 条件にした装飾（フォント太字化）を [`recipe`] に登録する。
//! `fandhe_frontend_headless_ui::link::root` は `current` 引数が `true` の
//! ときのみ `aria-current="page"` を出力する契約（headless 層 rustdoc
//! 参照）であるため、本 styled 層は追加の bool 引数を持たず CSS 側の状態
//! セレクタのみで表現する。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。`href` の URL スキーム検証は headless
//!   層（`crates/headless-ui/src/link.rs` rustdoc 参照）が担う。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/link.rs` の
/// `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root"];

/// `root` の見た目（chakra-ui Link の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkVariant {
    /// 下線なし（既定）。
    #[default]
    Plain,
    /// 常時下線表示。
    Underline,
}

impl VariantValue for LinkVariant {
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

/// この styled Link の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] の
/// みが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("link", SLOTS)
        .base(
            "root",
            vec![
                decl(
                    "color",
                    "var(--fandhe-color-accent, var(--fandhe-color-fg))",
                ),
                decl(
                    "text-decoration",
                    "var(--fandhe-link-text-decoration, none)",
                ),
                decl("cursor", "pointer"),
            ],
        )
        .variant(
            LinkVariant::Plain,
            "root",
            vec![decl("--fandhe-link-text-decoration", "none")],
        )
        .variant(
            LinkVariant::Underline,
            "root",
            vec![decl("--fandhe-link-text-decoration", "underline")],
        )
        .default_variant(LinkVariant::Plain)
        .state(
            "root",
            StateCondition::AttrEq("aria-current", "page"),
            vec![decl("font-weight", "var(--fandhe-font-font-weight-medium)")],
        )
}

/// この styled Link が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。`variant` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は [`fandhe_frontend_headless_ui::link::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::link::{self, LinkVariant};
///
/// let node = link::root("/docs", false, false, LinkVariant::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="link" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    href: &'a str,
    external: bool,
    current: bool,
    variant: LinkVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::link::root(href, external, current, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            "/docs",
            false,
            false,
            LinkVariant::Plain,
            vec![],
            vec![text("Docs")],
        ));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/docs""#));
    }

    #[test]
    fn external_true_adds_target_and_rel() {
        let html = render(&root(
            "https://example.com",
            true,
            false,
            LinkVariant::Plain,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn current_true_adds_aria_current() {
        let html = render(&root(
            "/docs",
            false,
            true,
            LinkVariant::Plain,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn default_variant_is_plain() {
        let html = render(&root(
            "/docs",
            false,
            false,
            LinkVariant::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-link--variant-plain"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (LinkVariant::Plain, "fd-link--variant-plain"),
            (LinkVariant::Underline, "fd-link--variant-underline"),
        ] {
            let html = render(&root("/docs", false, false, variant, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "/docs",
            false,
            false,
            LinkVariant::Plain,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            "/docs",
            false,
            false,
            LinkVariant::Plain,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_expected_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--variant-"));
        assert!(a.contains(r#"[aria-current="page"]"#));
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
            "/docs",
            false,
            false,
            LinkVariant::Plain,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(
            "/docs",
            false,
            false,
            LinkVariant::Plain,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
