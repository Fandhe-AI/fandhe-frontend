//! List（イシュー #771）: slot recipe styled 部品。root（`<ul>`/`<ol>`）/
//! item（`<li>`）/ indicator（装飾マーカー用 `<span>`）の 3 パーツで構成する
//! リスト表示。
//!
//! [`ListType`] は [`crate::heading::HeadingLevel`] と同じ「variant クラス
//! ではなくレンダリングするタグそのものを選ぶ」方式で `<ul>`/`<ol>` を選択
//! する。colorPalette 軸は付与しない（中立部品。`indicator` の色は呼び出し
//! 側が children/attrs で指定する）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, aria_hidden, Anatomy};

/// `data-scope="list"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("list");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ、[`crate::card`] 前例と同型）。
const SLOTS: &[&str] = &["root", "item", "indicator"];

/// root がレンダリングする HTML 要素（`<ul>`/`<ol>`。[`crate::heading::HeadingLevel`]
/// と同型のタグ選択方式、variant クラスではない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    /// 順序なしリスト（既定）。
    #[default]
    Unordered,
    /// 順序付きリスト。
    Ordered,
}

impl ListType {
    /// この種別に対応する HTML タグ名。
    fn tag(self) -> &'static str {
        match self {
            Self::Unordered => "ul",
            Self::Ordered => "ol",
        }
    }
}

/// List の見た目 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListVariant {
    /// 既定のマーカー表示（既定）。
    #[default]
    Marker,
    /// マーカーなし（`indicator` によるカスタムマーカー用）。
    Plain,
}

impl VariantValue for ListVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Marker => "marker",
            Self::Plain => "plain",
        }
    }
}

/// List の recipe（scope `"list"`、[`SLOTS`] の 3 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("list", SLOTS)
        .base("root", vec![decl("margin", "0")])
        .base(
            "item",
            vec![decl("margin-block", "0.25rem"), decl("line-height", "1.5")],
        )
        .base("indicator", vec![decl("display", "inline-block")])
        .variant(
            ListVariant::Marker,
            "root",
            vec![
                decl("list-style", "revert"),
                decl("padding-inline-start", "1.5rem"),
            ],
        )
        .variant(
            ListVariant::Plain,
            "root",
            vec![
                decl("list-style", "none"),
                decl("padding-inline-start", "0"),
            ],
        )
        .default_variant(ListVariant::Marker)
}

/// List の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<ul>`/`<ol>`）を組み立てる。`list_type` がレンダリングする
/// タグを、`variant` がマーカー表示を決める（両者は独立）。`ol` の
/// `start`/`reversed` は呼び出し側 `attrs` をそのまま透過する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
///
/// let node = list::root(ListType::default(), ListVariant::default(), vec![], vec![]);
/// assert!(render(&node).starts_with("<ul"));
/// ```
#[must_use]
pub fn root<'a>(
    list_type: ListType,
    variant: ListVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("variant", variant.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", list_type.tag(), merged, children)
}

/// item パーツ（`<li>`）を組み立てる。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "li", attrs, children)
}

/// indicator パーツ（`<span aria-hidden="true">`）を組み立てる。装飾用
/// カスタムマーカーであり、スクリーンリーダーへ意味を持たせないため常に
/// `aria-hidden="true"` を固定する（呼び出し側がこれを外すオプションは
/// 設けない。[`crate::skeleton::skeleton`] と同じ fail-closed 判断）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs: Vec<(&str, &str)> = attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("aria-hidden"))
        .collect();
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_renders_ul_with_marker_variant() {
        let html = render(&root(
            ListType::default(),
            ListVariant::default(),
            vec![],
            vec![],
        ));
        assert_eq!(
            html,
            r#"<ul data-scope="list" data-part="root" class="fd-list--variant-marker"></ul>"#
        );
    }

    #[test]
    fn list_type_enumeration_maps_to_expected_tags() {
        for (list_type, tag) in [(ListType::Unordered, "ul"), (ListType::Ordered, "ol")] {
            let html = render(&root(list_type, ListVariant::default(), vec![], vec![]));
            assert!(
                html.starts_with(&format!("<{tag} ")),
                "list_type={list_type:?} -> {html}"
            );
            assert!(
                html.ends_with(&format!("</{tag}>")),
                "list_type={list_type:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (ListVariant::Marker, "fd-list--variant-marker"),
            (ListVariant::Plain, "fd-list--variant-plain"),
        ] {
            let html = render(&root(ListType::default(), variant, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&item(vec![], vec![text("one")]))
            .starts_with(r#"<li data-scope="list" data-part="item""#));
        let html = render(&indicator(vec![], vec![]));
        assert!(html.starts_with(r#"<span data-scope="list" data-part="indicator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn ordered_list_start_and_reversed_attrs_pass_through() {
        let html = render(&root(
            ListType::Ordered,
            ListVariant::default(),
            vec![("start", "3"), ("reversed", "reversed")],
            vec![],
        ));
        assert!(html.contains(r#"start="3""#));
        assert!(html.contains(r#"reversed="reversed""#));
    }

    #[test]
    fn caller_supplied_aria_hidden_on_indicator_is_dropped_case_insensitively() {
        for key in ["aria-hidden", "Aria-Hidden", "ARIA-HIDDEN"] {
            let html = render(&indicator(vec![(key, "false")], vec![]));
            assert_eq!(html.matches("aria-hidden=").count(), 1, "html={html}");
            assert!(html.contains(r#"aria-hidden="true""#), "html={html}");
            assert!(!html.contains(r#"aria-hidden="false""#), "html={html}");
        }
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            ListType::default(),
            ListVariant::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_item_children_is_escaped() {
        let html = render(&item(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_plain_list_style_reset() {
        let out = css();
        assert!(out.contains("list-style: none;"));
    }
}
