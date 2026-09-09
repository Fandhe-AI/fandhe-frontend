//! チャート凡例（イシュー #847、chakra-ui `charts/legend.md` 相当）。
//!
//! [`super::data::ChartData`] の系列一覧から `<ul>` ベースの凡例を組み立てる
//! （インタラクティブ legend——hover で対象系列を強調・click で表示トグル——は
//! JS/wasm ランタイム連携が必要なためスコープ外、`crates/pre-styled-ui/src/charts/mod.rs`
//! のスコープ外節参照）。
//!
//! # セキュリティ不変条件
//!
//! - タイトル・系列名/表示ラベルはすべて [`fandhe_frontend_core::text`]
//!   経由のテキストノードとして受け取り、`render()` の既定エスケープ
//!   （REQ-1）を必ず通る。
//! - マーカー・icon slot の色は [`ChartData::series_color_var`]（系列の
//!   [`super::data::SeriesColor`] 上書きが無ければ [`super::series_color_var`]
//!   の 6 色循環へフォールバック。いずれも `theme.rs` の `TokenName`
//!   allowlist を満たす固定形 `var(--fandhe-color-<name>)` のみを生成する）
//!   由来の値のみを `style` 属性へ埋め込み、呼び出し側の任意文字列を
//!   連結しない。
//! - `icon`（[`super::data::Series::icon`]）は利用者が `el()`/`text()`
//!   経由で組み立てたノード木のみを受け付ける（`Node` 型そのものが
//!   `raw_html()` を経由しない限り既定エスケープを保証する）。

use super::data::ChartData;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope。[`super::axis`]/[`super::grid`] とは別の
/// scope（凡例は SVG 外の通常 HTML であり、パーツ集合が異なるため）。
const SCOPE: &str = "chart-legend";

/// [`recipe`] に渡す slot 一覧。`icon`（末尾、イシュー #2077）は
/// [`super::data::Series::icon`] 指定時に `marker` の代わりに描画される
/// 代替スロット（shadcn/ui `ChartConfig.icon` 相当）。
const SLOTS: &[&str] = &["root", "title", "item", "marker", "label", "icon"];

/// [`legend`] の props。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegendProps {
    /// 凡例タイトル（省略可）。
    pub title: Option<String>,
}

/// Legend の recipe（scope `"chart-legend"`、[`SLOTS`] の 5 パーツ）。
///
/// # 参考サイト基準への調整（イシュー #1593）
///
/// `marker` の寸法をリテラル `0.75rem` からスケールトークン
/// `--fandhe-space-3`（同値）へ置換した。参照 4 サイトに対応部品が無いため
/// 内部整合のみを評価軸とした。
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
                // イシュー #1593: リテラル 0.75rem をスケールトークン
                // --fandhe-space-3（同値）へ置換。値は不変。
                decl("width", "var(--fandhe-space-3)"),
                decl("height", "var(--fandhe-space-3)"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "icon",
            vec![
                // marker と同寸のアイコン枠（イシュー #2077）。色は SVG
                // アイコン側の `fill="currentColor"`/`stroke="currentColor"`
                // へ継承させるため `color` を指定する（`legend()` が
                // インライン `style` で系列色を渡す）。
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex-shrink", "0"),
                decl("width", "var(--fandhe-space-3)"),
                decl("height", "var(--fandhe-space-3)"),
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
/// マーカー色は [`ChartData::series_color_var`] を系列インデックス順に
/// 割り当てる（系列に [`super::data::SeriesColor`] 上書きが無ければ
/// [`super::series_color_var`] の 6 色循環）。ラベルは
/// [`super::data::Series::display_label`]（`label` 未設定時は `name`）。
/// [`super::data::Series::icon`] が指定された系列は、マーカー（色付き丸）の
/// 代わりに `data-part="icon"` を描画する（shadcn/ui `ChartConfig.icon` と
/// 同じ「icon 指定時はマーカーを置換」意味論、イシュー #2077）。
#[must_use]
pub fn legend(data: &ChartData, props: &LegendProps) -> Node {
    let mut children: Vec<Node> = Vec::new();

    if let Some(title) = &props.title {
        // root は <ul> のため、直接の子として有効なのは <li>（および
        // <script>/<template>）のみ。<span> を直接の子にすると不正な
        // マークアップとなりリストのアクセシビリティチェックに失敗するため、
        // タイトルも <li> として描画する（`data-part="title"` は不変）。
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "title")],
            vec![text(title)],
        ));
    }

    for (i, series) in data.series().iter().enumerate() {
        let color = data.series_color_var(i);
        let marker = if let Some(icon) = &series.icon {
            let icon_style = format!("color: {color}");
            el(
                "span",
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "icon"),
                    ("style", icon_style.as_str()),
                    ("aria-hidden", "true"),
                ],
                vec![icon.clone()],
            )
        } else {
            let marker_style = format!("background: {color}");
            el(
                "span",
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "marker"),
                    ("style", marker_style.as_str()),
                    ("aria-hidden", "true"),
                ],
                vec![],
            )
        };
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(series.display_label())],
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

/// カテゴリ単位の凡例（shadcn `chart-pie-legend`、イシュー #2084）。
///
/// [`legend`] は [`ChartData::series`] を単位に 1 item を組み立てるが、
/// 円グラフ（[`crate::pie_chart`]/[`crate::donut_chart`]）は単一系列専用
/// （モジュール doc「単一系列専用」節、`pie_chart`/`donut_chart` の
/// モジュール doc参照）のため、`series` 単位の凡例ではカテゴリ 1 件ずつの
/// 対応が組めない。本関数はカテゴリ 1 件 = item 1 件とし、マーカー色は
/// [`super::series_color_var`]（カテゴリ index）で扇形の塗り色
/// （`pie_chart`/`donut_chart` の `segment` 塗り色）と一致させる。
///
/// `data` の系列は 1 本目（`series()[0]`）のみを参照し、`icon` slot は
/// 使わない（`icon` はカテゴリではなく系列単位の設定
/// [`super::data::Series::icon`] のため本関数の対象外）。マークアップは
/// [`legend`] と同じ scope/slot（`root`/`title`/`item`/`marker`/`label`）を
/// 共有するため CSS（[`css`]）は変更しない。
///
/// `data.series()` が空の場合は `item` を 1 件も持たない `root` のみを
/// 返す（fail-soft。呼び出し元は事前に [`ChartData`] の非空検証を通した
/// データを渡す前提、`pie_chart`/`donut_chart` と同型の判断）。
#[must_use]
pub fn category_legend(data: &ChartData, props: &LegendProps) -> Node {
    let mut children: Vec<Node> = Vec::new();

    if let Some(title) = &props.title {
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "title")],
            vec![text(title)],
        ));
    }

    let categories = data.categories();
    for (i, category) in categories.iter().enumerate() {
        let color = super::series_color_var(i);
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
            vec![text(category.as_str())],
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
        assert!(html.contains(r#"<li data-scope="chart-legend" data-part="title">Series</li>"#));
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

    #[test]
    fn category_legend_renders_one_item_per_category() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![Series::new("total", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert_eq!(html.matches(r#"data-part="item""#).count(), 3);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
        assert!(html.contains("var(--fandhe-color-chart-3)"));
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
        assert!(html.contains(">C<"));
        assert!(!html.contains(r#"data-part="icon""#));
    }

    #[test]
    fn category_legend_marker_color_matches_segment_color_cycle() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![1.0, 1.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert_eq!(
            crate::charts::series_color_var(0),
            "var(--fandhe-color-chart-1)"
        );
        assert!(html.contains(&format!(
            "background: {}",
            crate::charts::series_color_var(0)
        )));
        assert!(html.contains(&format!(
            "background: {}",
            crate::charts::series_color_var(1)
        )));
    }

    #[test]
    fn category_legend_xss_regression_category_name_is_escaped() {
        let payload = "</ul><script>alert(1)</script>";
        let data = ChartData::new(
            vec![payload.to_string()],
            vec![Series::new("total", vec![1.0])],
        )
        .unwrap();
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
