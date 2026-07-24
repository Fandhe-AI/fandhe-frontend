//! チャート凡例（イシュー #847、chakra-ui `charts/legend.md` 相当）。
//!
//! [`super::data::ChartData`] の系列一覧から `<ul>` ベースの凡例を組み立てる
//! （インタラクティブ legend——hover で対象系列を強調・click で表示トグル——は
//! JS/wasm ランタイム連携が必要なためスコープ外、`crates/pre-styled-ui/src/charts/mod.rs`
//! のスコープ外節参照）。
//!
//! # セキュリティ不変条件
//!
//! - タイトル・系列名はすべて [`fandhe_frontend_core::text`] 経由のテキスト
//!   ノードとして受け取り、`render()` の既定エスケープ（REQ-1）を必ず通る。
//! - マーカーの色は [`super::series_color_var`]（`theme.rs` の `TokenName`
//!   allowlist を満たす固定文字列のみを生成する）由来の値のみを `style`
//!   属性へ埋め込み、呼び出し側の任意文字列を連結しない。

use super::data::ChartData;
use super::series_color_var;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope。[`super::axis`]/[`super::grid`] とは別の
/// scope（凡例は SVG 外の通常 HTML であり、パーツ集合が異なるため）。
const SCOPE: &str = "chart-legend";

/// [`recipe`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "title", "item", "marker", "label"];

/// [`legend`] の props。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegendProps {
    /// 凡例タイトル（省略可）。
    pub title: Option<String>,
}

/// Legend の recipe（scope `"chart-legend"`、[`SLOTS`] の 5 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new(SCOPE, SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-4)"),
                decl("list-style", "none"),
                decl("padding", "0"),
                decl("margin", "0"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin-right", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "marker",
            vec![
                decl("display", "inline-block"),
                decl("width", "0.75rem"),
                decl("height", "0.75rem"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("flex-shrink", "0"),
            ],
        )
}

/// Legend の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// `data` の系列一覧から凡例を組み立てる（`<ul data-scope="chart-legend"
/// data-part="root">` を root とする）。
///
/// マーカー色は [`super::series_color_var`] を系列インデックス順に割り当てる
/// （[`super::series_color_var`] と同じ 6 色循環）。
#[must_use]
pub fn legend(data: &ChartData, props: &LegendProps) -> Node {
    let mut children: Vec<Node> = Vec::new();

    if let Some(title) = &props.title {
        children.push(el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "title")],
            vec![text(title)],
        ));
    }

    for (i, series) in data.series().iter().enumerate() {
        let color = series_color_var(i);
        let marker_style = format!("background: {color}");
        let marker = el(
            "span",
            vec![
                ("data-scope", SCOPE),
                ("data-part", "marker"),
                ("style", marker_style.as_str()),
                ("aria-hidden", "true"),
            ],
            vec![],
        );
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(&series.name)],
        );
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "item")],
            vec![marker, label],
        ));
    }

    el(
        "ul",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "root"),
            ("role", "list"),
        ],
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![
                Series::new("visits", vec![1.0, 2.0]),
                Series::new("signups", vec![3.0, 4.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn legend_renders_one_item_per_series_with_marker_and_label() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        assert!(html.starts_with(r#"<ul data-scope="chart-legend" data-part="root" role="list">"#));
        assert_eq!(html.matches(r#"data-part="item""#).count(), 2);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
        assert!(html.contains(">visits<"));
        assert!(html.contains(">signups<"));
    }

    #[test]
    fn legend_omits_title_span_when_none() {
        let html = render(&legend(&sample(), &LegendProps::default()));
        assert!(!html.contains(r#"data-part="title""#));
    }

    #[test]
    fn legend_renders_title_when_present() {
        let props = LegendProps {
            title: Some("Series".to_string()),
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-part="title""#));
        assert!(html.contains(">Series<"));
    }

    #[test]
    fn xss_regression_title_and_series_name_are_escaped() {
        let payload = "</ul><script>alert(1)</script>";
        let data =
            ChartData::new(vec!["a".to_string()], vec![Series::new(payload, vec![1.0])]).unwrap();
        let props = LegendProps {
            title: Some(payload.to_string()),
        };
        let html = render(&legend(&data, &props));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_output_is_closed_charset() {
        let out = css();
        assert!(!out.contains('<'));
        assert!(out.contains("data-part=\"marker\""));
    }
}
