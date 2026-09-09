//! チャート凡例（イシュー #847、chakra-ui `charts/legend.md` 相当）。
//!
//! [`super::data::ChartData`] の系列一覧から `<ul>` ベースの凡例を組み立てる
//! （インタラクティブ legend——hover で対象系列を強調・click で表示トグル——は
//! JS/wasm ランタイム連携が必要なためスコープ外、`crates/pre-styled-ui/src/charts/mod.rs`
//! のスコープ外節参照。#2132 が実装を担う）。
//!
//! # shadcn/ui 突合（イシュー #2086）
//!
//! shadcn/ui Charts（tooltip ページの Legend 実演）は本モジュールに無かった
//! 3 点を持つ: (1) `hideIcon`（マーカー/icon を出さない）、(2) 中央揃え
//! （`justify-content: center` が既定）、(3) 角丸四角の色サンプル。
//!
//! | shadcn/ui | 対応 |
//! |---|---|
//! | `hideIcon` | [`LegendProps::hide_marker`] として純追加 |
//! | 中央揃え既定 | 採用しない（既定は chakra-ui 由来の `flex-start` を維持し churn を避ける）。[`LegendProps::align`] の [`LegendAlign::Center`] として opt-in 軸を追加 |
//! | 角丸四角の色サンプル | [`LegendProps::marker`] の [`LegendMarker::Square`] として opt-in 軸を追加（既定は chakra-ui 由来の円形 [`LegendMarker::Circle`]） |
//! | `verticalAlign`（top/bottom） | 採用しない。[`legend`] は独立したノードであり、チャート本体との上下位置は呼び出し側のノード合成順序が決める（CSS 軸を持たない）。上に置きたい場合は `legend()` の戻り値をチャート本体より先にノード木へ並べればよい |
//!
//! 参照競合の判定: 水平揃え・マーカー形状ともに chakra-ui の値を既定に
//! 据え置く（golden 純追加原則。既存消費者の見た目を再 churn させない）。
//! shadcn-ui 側の値は opt-in variant として追加した。
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
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope。[`super::axis`]/[`super::grid`] とは別の
/// scope（凡例は SVG 外の通常 HTML であり、パーツ集合が異なるため）。
const SCOPE: &str = "chart-legend";

/// [`recipe`] に渡す slot 一覧。`icon`（末尾、イシュー #2077）は
/// [`super::data::Series::icon`] 指定時に `marker` の代わりに描画される
/// 代替スロット（shadcn/ui `ChartConfig.icon` 相当）。
const SLOTS: &[&str] = &["root", "title", "item", "marker", "label", "icon"];

/// 凡例の水平揃え軸（shadcn/ui `verticalAlign` の水平版に相当する
/// `justify-content` 選択、イシュー #2086）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendAlign {
    /// 左寄せ（既定、chakra-ui 由来）。
    #[default]
    Start,
    /// 中央揃え（shadcn/ui の既定に相当する opt-in 軸）。
    Center,
    /// 右寄せ。
    End,
}

impl VariantValue for LegendAlign {
    fn axis(self) -> &'static str {
        "align"
    }

    fn value(self) -> &'static str {
        match self {
            LegendAlign::Start => "start",
            LegendAlign::Center => "center",
            LegendAlign::End => "end",
        }
    }
}

/// 凡例マーカーの形状軸（shadcn/ui の角丸四角サンプルに相当する opt-in 軸、
/// イシュー #2086）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendMarker {
    /// 円形（既定、chakra-ui 由来）。
    #[default]
    Circle,
    /// 角丸四角（shadcn/ui `rounded-[2px]` 相当）。
    Square,
}

impl VariantValue for LegendMarker {
    fn axis(self) -> &'static str {
        "marker"
    }

    fn value(self) -> &'static str {
        match self {
            LegendMarker::Circle => "circle",
            LegendMarker::Square => "square",
        }
    }
}

/// [`legend`] の props。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegendProps {
    /// 凡例タイトル（省略可）。
    pub title: Option<String>,
    /// `true` ならマーカー/icon slot を描画せずラベルのみを表示する
    /// （shadcn/ui `hideIcon` 相当、イシュー #2086。既定 `false`）。
    pub hide_marker: bool,
    /// 水平揃え（既定 [`LegendAlign::Start`]、イシュー #2086）。
    pub align: LegendAlign,
    /// マーカー形状（既定 [`LegendMarker::Circle`]、イシュー #2086）。
    pub marker: LegendMarker,
}

/// Legend の recipe（scope `"chart-legend"`、[`SLOTS`] の 5 パーツ）。
///
/// # 参考サイト基準への調整（イシュー #1593）
///
/// `marker` の寸法をリテラル `0.75rem` からスケールトークン
/// `--fandhe-space-3`（同値）へ置換した。参照 4 サイトに対応部品が無いため
/// 内部整合のみを評価軸とした。
///
/// # shadcn/ui 突合（イシュー #2086）
///
/// `align`（`root` slot、既定 [`LegendAlign::Start`]）・`marker`（`marker`
/// slot、既定 [`LegendMarker::Circle`]）の 2 軸を末尾へ純追加した。既存 5
/// ブロックは不変（golden 純追加原則、`tests/charts_parts_css.rs` 参照）。
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
        .variant(
            LegendAlign::Center,
            "root",
            vec![decl("justify-content", "center")],
        )
        .variant(
            LegendAlign::End,
            "root",
            vec![decl("justify-content", "flex-end")],
        )
        .default_variant(LegendAlign::Start)
        .variant(
            LegendMarker::Square,
            "marker",
            vec![decl("border-radius", "var(--fandhe-radius-sm)")],
        )
        .default_variant(LegendMarker::Circle)
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
///
/// [`LegendProps::hide_marker`] が `true` の場合、マーカー/icon slot は
/// 一切描画せずラベルのみの `<li>` を組み立てる（shadcn/ui `hideIcon` 相当、
/// イシュー #2086）。root へは [`LegendProps::align`]/[`LegendProps::marker`]
/// の variant class を常に emit する（`grid`/`radar_chart` 等と同じ「既定
/// variant も class として出力する」既存規約、`recipe().variant_classes`
/// 参照）。マーカー span 自身にも `marker` 軸の class を付与する（`root`
/// へ落とすだけでは `[data-part="marker"]` セレクタの規則が当たらないため）。
#[must_use]
pub fn legend(data: &ChartData, props: &LegendProps) -> Node {
    let recipe = recipe();
    let root_class = recipe.variant_classes(&[
        ("align", props.align.value()),
        ("marker", props.marker.value()),
    ]);
    let marker_class = recipe.variant_class(props.marker);
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
        let mut item_children: Vec<Node> = Vec::new();
        if !props.hide_marker {
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
                        ("class", marker_class.as_str()),
                        ("style", marker_style.as_str()),
                        ("aria-hidden", "true"),
                    ],
                    vec![],
                )
            };
            item_children.push(marker);
        }
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(series.display_label())],
        );
        item_children.push(label);
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "item")],
            item_children,
        ));
    }

    el(
        "ul",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "root"),
            ("role", "list"),
            ("class", root_class.as_str()),
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
/// [`LegendProps::hide_marker`]/`align`/`marker`（イシュー #2086）は
/// [`legend`] と同じ意味論で適用される（recipe・slot を共有するため）。
pub fn category_legend(data: &ChartData, props: &LegendProps) -> Node {
    let recipe = recipe();
    let root_class = recipe.variant_classes(&[
        ("align", props.align.value()),
        ("marker", props.marker.value()),
    ]);
    let marker_class = recipe.variant_class(props.marker);
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
        let mut item_children: Vec<Node> = Vec::new();
        if !props.hide_marker {
            let marker_style = format!("background: {color}");
            item_children.push(el(
                "span",
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "marker"),
                    ("class", marker_class.as_str()),
                    ("style", marker_style.as_str()),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ));
        }
        let label = el(
            "span",
            vec![("data-scope", SCOPE), ("data-part", "label")],
            vec![text(category.as_str())],
        );
        item_children.push(label);
        children.push(el(
            "li",
            vec![("data-scope", SCOPE), ("data-part", "item")],
            item_children,
        ));
    }

    el(
        "ul",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "root"),
            ("role", "list"),
            ("class", root_class.as_str()),
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
        // イシュー #2086: 既定でも root へ align/marker の既定 variant class を
        // 常に emit する（`grid`/`radar_chart` 等と同じ既存規約）。この class
        // 付与は意図した差分であり、root の直接の子は不変。
        assert!(html.starts_with(
            r#"<ul data-scope="chart-legend" data-part="root" role="list" class="fd-chart-legend--align-start fd-chart-legend--marker-circle">"#
        ));
        assert_eq!(html.matches(r#"data-part="item""#).count(), 2);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
        assert!(html.contains(">visits<"));
        assert!(html.contains(">signups<"));
    }

    #[test]
    fn legend_hide_marker_omits_marker_and_icon_slots() {
        let props = LegendProps {
            hide_marker: true,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(!html.contains(r#"data-part="marker""#));
        assert!(!html.contains(r#"data-part="icon""#));
        assert!(html.contains(">visits<"));
        assert!(html.contains(">signups<"));
    }

    #[test]
    fn legend_hide_marker_omits_icon_slot_when_series_has_icon() {
        use crate::charts::data::Series;
        use fandhe_frontend_core::text;

        let data = ChartData::new(
            vec!["Jan".to_string()],
            vec![Series::new("visits", vec![1.0]).with_icon(text("*"))],
        )
        .unwrap();
        let props = LegendProps {
            hide_marker: true,
            ..Default::default()
        };
        let html = render(&legend(&data, &props));
        assert!(!html.contains(r#"data-part="icon""#));
        assert!(!html.contains(r#"data-part="marker""#));
    }

    #[test]
    fn legend_align_center_and_end_emit_expected_class() {
        let center = LegendProps {
            align: LegendAlign::Center,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &center));
        assert!(html.contains("fd-chart-legend--align-center"));

        let end = LegendProps {
            align: LegendAlign::End,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &end));
        assert!(html.contains("fd-chart-legend--align-end"));
    }

    #[test]
    fn legend_marker_square_emits_expected_class_on_marker_span() {
        let props = LegendProps {
            marker: LegendMarker::Square,
            ..Default::default()
        };
        let html = render(&legend(&sample(), &props));
        assert!(html.contains(r#"data-part="marker" class="fd-chart-legend--marker-square""#));
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
            ..Default::default()
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
            ..Default::default()
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
