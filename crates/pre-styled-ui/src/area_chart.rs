//! AreaChart（イシュー #848、親 Phase #845）: [`crate::line_chart`] と同じ
//! `charts` 基盤（#846）の消費者。系列ごとに、折れ線（`series-line`）と
//! domain 下端へ閉じた塗りつぶし面（`series-area`）を重ねて描く自己完結部品。
//!
//! chakra-ui `charts/area-chart.md` は `stackId`（積み上げ）・`curveType`
//! （曲線補間）等を提供するが、これらは #847 以降のスコープ外とする
//! （[`crate::line_chart`] モジュール doc「本イシューのスコープ外」と同じ
//! 判断）。系列ごとに独立した面を重ね描きする素朴な構成のみを提供する。
//!
//! 座標写像・path 生成・数値文字列化の一元化方針、x/y 軸の写像規則は
//! [`crate::line_chart`] モジュール doc を参照（[`crate::line_chart::category_x`]/
//! [`crate::line_chart::view_box_from_dims`] を共有ヘルパとして再利用する）。
//!
//! # 面 path の閉じ方（baseline）
//!
//! 系列の折れ線経路を辿った後、x 軸の逆順で `domain` 下端（`data.domain().0`、
//! [`ChartData::domain`](crate::charts::data::ChartData::domain) が返す
//! フラットデータ非退化パディング込みの値）に写像した高さ（baseline）へ
//! 戻って閉じる（`M .. L .. L (last.x, baseline) L (first.x, baseline) Z`）。
//! `y = 0` 固定ではなく domain 下端を使うのは、全値が負の系列で面が上下反転
//! （画面上端へ張り付く）せず、値の小さい方へ塗りつぶしが伸びる直感的な
//! 見た目を保つため。
//!
//! # エッジケース（`n == 1`）
//!
//! [`crate::line_chart`] と同じ規則: 面・線のいずれも生成せず、中央
//! （`width / 2.0`）に点マーカーのみを描く（0 除算・退化した面の回避）。
//!
//! # セキュリティ不変条件
//!
//! [`crate::line_chart`] モジュール doc と同一（`raw_html()` 不使用、
//! 座標は `fmt_coord` 経由で文字集合 `[0-9.-]` に閉じる、CSS 宣言値は
//! すべて静的リテラル）。
//!
//! # 本イシューのスコープ外
//!
//! 積み上げ（`stackId`）・曲線補間・軸/グリッド/凡例/ツールチップは #847
//! 以降。`examples/headless-pre-styled-ui` への追随は crates.io 公開後
//! （[`crate::line_chart`] と同じ判断）。

use crate::charts::data::ChartData;
use crate::charts::scale::LinearScale;
use crate::charts::svg::{fmt_coord, svg_root, PathBuilder};
use crate::charts::{series_color_var, ChartError};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::line_chart::{category_x, view_box_from_dims};
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="area-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("area-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。単一カテゴリ（`n == 1`）時の点
/// マーカー用に `point` を含める（[`crate::line_chart`] の `SLOTS` と同じ
/// エッジケース規則、モジュール doc「エッジケース」参照）。
const SLOTS: &[&str] = &["root", "plot", "series-area", "series-line", "point"];

/// `viewBox` 幅の既定値（[`crate::line_chart::DEFAULT_WIDTH`] と同値）。
pub const DEFAULT_WIDTH: f64 = 300.0;
/// `viewBox` 高さの既定値。
pub const DEFAULT_HEIGHT: f64 = 150.0;

/// 単一カテゴリ時に描く点マーカーの半径（[`crate::line_chart`] と同値）。
const POINT_RADIUS: f64 = 2.5;

/// [`area_chart`] の入力。フィールドの意味は [`crate::line_chart::LineChartProps`]
/// と同一。
pub struct AreaChartProps<'a> {
    /// 描画するチャートデータ。
    pub data: &'a ChartData,
    /// `svg` 要素へ付与する `aria-label`（必須）。
    pub aria_label: &'a str,
    /// `viewBox` 幅。
    pub width: f64,
    /// `viewBox` 高さ。
    pub height: f64,
    /// root へ付与する寸法 variant。
    pub size: Size,
}

impl<'a> AreaChartProps<'a> {
    /// 既定寸法（[`Size::Md`]）で組み立てる。
    #[must_use]
    pub fn new(data: &'a ChartData, aria_label: &'a str) -> Self {
        AreaChartProps {
            data,
            aria_label,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            size: Size::Md,
        }
    }
}

/// この styled AreaChart の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("area-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("--fandhe-area-chart-height", "150px"),
            ],
        )
        .base(
            "plot",
            vec![
                decl("display", "block"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-area-chart-height, auto)"),
            ],
        )
        .base(
            "series-area",
            // chakra-ui `charts/area-chart.md` の既定 `fillOpacity={0.2}` 準拠。
            vec![decl("fill-opacity", "0.2"), decl("stroke", "none")],
        )
        .base(
            "series-line",
            vec![decl("fill", "none"), decl("stroke-width", "2")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-area-chart-height", "96px")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-area-chart-height", "150px")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-area-chart-height", "220px")],
        )
        .default_variant(Size::Md)
}

/// この styled AreaChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// 系列 1 本を「面 + 線」（`n >= 2`）または中央の点マーカー（`n == 1`）として
/// 描く（内部ヘルパ）。`baseline_y` は `data.domain().0` を y スケールで
/// 写像した座標（モジュール doc「面 path の閉じ方」参照）。
fn render_series(
    width: f64,
    y_scale: &LinearScale,
    baseline_y: f64,
    values: &[f64],
    series_index: usize,
) -> Vec<Node> {
    let n = values.len();
    let color = series_color_var(series_index);

    if n <= 1 {
        let x = category_x(width, n, 0);
        let y = values.first().copied().map_or(0.0, |v| y_scale.scale(v));
        let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
        return vec![el(
            "circle",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "point"),
                ("cx", cx.as_str()),
                ("cy", cy.as_str()),
                ("r", r.as_str()),
                ("fill", color.as_str()),
            ],
            vec![],
        )];
    }

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| (category_x(width, n, i), y_scale.scale(v)))
        .collect();

    // 面 path: 折れ線を順方向に辿った後、baseline へ降りて逆方向の始点に戻り
    // 閉じる（モジュール doc「面 path の閉じ方」参照）。
    let mut area_builder = PathBuilder::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        area_builder = if i == 0 {
            area_builder.move_to(x, y)
        } else {
            area_builder.line_to(x, y)
        };
    }
    let (last_x, _) = points[points.len() - 1];
    let (first_x, _) = points[0];
    let area_d = area_builder
        .line_to(last_x, baseline_y)
        .line_to(first_x, baseline_y)
        .close()
        .build();

    let mut line_builder = PathBuilder::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        line_builder = if i == 0 {
            line_builder.move_to(x, y)
        } else {
            line_builder.line_to(x, y)
        };
    }
    let line_d = line_builder.build();

    vec![
        el(
            "path",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "series-area"),
                ("d", area_d.as_str()),
                ("fill", color.as_str()),
            ],
            vec![],
        ),
        el(
            "path",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "series-line"),
                ("d", line_d.as_str()),
                ("stroke", color.as_str()),
                ("fill", "none"),
            ],
            vec![],
        ),
    ]
}

/// AreaChart 本体を組み立てる。
///
/// # Errors
///
/// [`crate::line_chart::line_chart`] と同じ契約
/// （[`crate::line_chart::view_box_from_dims`] 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::area_chart::{area_chart, AreaChartProps};
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
/// )
/// .unwrap();
/// let node = area_chart(&AreaChartProps::new(&data, "monthly visits"), vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-part="series-area""#));
/// ```
pub fn area_chart<'a>(
    props: &AreaChartProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, ChartError> {
    let view_box = view_box_from_dims(props.width, props.height)?;
    let y_scale = LinearScale::new(props.data.domain(), (props.height, 0.0))?;
    let (dom_lo, _dom_hi) = props.data.domain();
    let baseline_y = y_scale.scale(dom_lo);

    let plot_children: Vec<Node> = props
        .data
        .series()
        .iter()
        .enumerate()
        .flat_map(|(i, s)| render_series(props.width, &y_scale, baseline_y, &s.values, i))
        .collect();

    let plot = svg_root(
        &view_box,
        vec![
            ("data-scope", "area-chart"),
            ("data-part", "plot"),
            ("aria-label", props.aria_label),
        ],
        plot_children,
    );

    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    Ok(ANATOMY.part("root", "div", merged, vec![plot]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn data(values: Vec<f64>) -> ChartData {
        let categories = (0..values.len()).map(|i| i.to_string()).collect();
        ChartData::new(categories, vec![Series::new("s", values)]).unwrap()
    }

    #[test]
    fn renders_root_and_plot_with_aria_label() {
        let d = data(vec![1.0, 2.0, 3.0]);
        let node = area_chart(&AreaChartProps::new(&d, "sample"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.starts_with(r#"<div data-scope="area-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="area-chart" data-part="plot""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="sample""#));
    }

    #[test]
    fn multi_category_renders_area_and_line_paths() {
        let d = data(vec![1.0, 5.0, 2.0]);
        let node = area_chart(&AreaChartProps::new(&d, "multi"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="series-area""#));
        assert!(html.contains(r#"data-part="series-line""#));
        assert!(html.contains('Z'));
    }

    #[test]
    fn single_category_renders_point_not_path() {
        let d = data(vec![5.0]);
        let node = area_chart(&AreaChartProps::new(&d, "single"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="point""#));
        assert!(!html.contains("<path"));
    }

    #[test]
    fn negative_only_values_stay_deterministic_with_negative_coordinates() {
        let d = data(vec![-10.0, -30.0, -20.0]);
        let node1 = area_chart(&AreaChartProps::new(&d, "neg"), vec![]).unwrap();
        let node2 = area_chart(&AreaChartProps::new(&d, "neg"), vec![]).unwrap();
        assert_eq!(render(&node1), render(&node2));
        assert!(render(&node1).contains('-'));
    }

    #[test]
    fn flat_data_renders_deterministically() {
        let d = data(vec![5.0, 5.0, 5.0]);
        let a = render(&area_chart(&AreaChartProps::new(&d, "flat"), vec![]).unwrap());
        let b = render(&area_chart(&AreaChartProps::new(&d, "flat"), vec![]).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn non_finite_height_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "bad");
        props.height = f64::INFINITY;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn non_positive_width_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "bad");
        props.width = -1.0;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::DegenerateDomain
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let d = data(vec![1.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "class-test");
        props.size = Size::Sm;
        let html = render(&area_chart(&props, vec![("class", "attacker")]).unwrap());
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker"));
        assert!(html.contains("fd-area-chart--size-sm"));
    }

    #[test]
    fn xss_payload_in_aria_label_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&area_chart(&AreaChartProps::new(&d, payload), vec![]).unwrap());
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><script>alert(1)</script>";
        let html = render(
            &area_chart(
                &AreaChartProps::new(&d, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_expected_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="area-chart"][data-part="series-area"]"#));
        assert!(a.contains("fill-opacity: 0.2"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
