//! styled DonutChart（イシュー #850、親 Phase #845）。
//!
//! chakra-ui `charts/donut-chart.md`（recharts `PieChart`/`Pie`
//! `innerRadius`/`outerRadius` 依存）相当のドーナツグラフを、外部依存ゼロ・
//! [`crate::charts`] 基盤の SVG ノード木生成のみで実装する（[`crate::pie_chart`]
//! と対をなす、`docs/policy/intentional-non-adoption.md` §7 の保留解除）。
//!
//! anatomy・角度計算・a11y 契約・`size` variant の設計方針は
//! [`crate::pie_chart`] モジュール doc を参照（本モジュールは環状（annulus）
//! セグメントを描画する点のみが異なる）。ark-ui に対応する headless
//! anatomy は存在しないため、[`crate::pie_chart`] と同じ判断で新規 anatomy
//! `data-scope="donut-chart"` を本クレートのみで定義する。
//!
//! # 内径（`inner_ratio`）
//!
//! [`DonutChartProps::inner_ratio`]（既定 `0.6`）は外径に対する内径の比率
//! （`r_inner = r_outer * inner_ratio`）。`0.0 < ratio < 1.0` の範囲・有限
//! 値であることを構築時に検証し、外れる場合は
//! [`crate::charts::pie::PieChartError::InvalidInnerRatio`] を返す（`0.0`
//! は内径 0（すなわち通常の pie）、`1.0` は内外径が一致し面積 0 の環となる
//! 退化構成であり、いずれも意味を持たないため拒否する）。
//!
//! # 全周セグメントの描画（[`crate::charts::pie`] モジュール doc「境界規則」）
//!
//! 非ゼロ値のセグメントが 1 個のみの場合、環状の扇形 path は始点=終点で
//! 退化するため、半周ずつ 2 本の annulus path（外周半周 arc + 内周半周
//! arc、[`crate::charts::pie::annulus_sector_path`] を 2 回呼ぶ）に分割して
//! 描画し、リング全体を隙間なく表現する。
//!
//! # セキュリティ不変条件
//!
//! [`crate::pie_chart`] モジュール doc「セキュリティ不変条件」節と同一
//! （`raw_html()` 不使用・数値文字列化は [`crate::charts::svg::fmt_coord`]
//! に一元化・既定エスケープ経由・`class` 単一化）。
//!
//! # 本イシューのスコープ外
//!
//! [`crate::pie_chart`] モジュール doc「本イシューのスコープ外」節と同一
//! （Legend/Tooltip・アニメーション・中央テキスト等）。中央テキスト
//! （chakra の "With Centered Text" 例）は呼び出し側が [`root`] の子ノード
//! として `chart` と並べて配置することで代替可能であり、本 API は
//! `root` の子ノードを [`chart`] 単体に固定しない設計とする（下記
//! [`donut_chart`] 実装参照）。

use crate::charts::pie::{annulus_sector_path, segment_angles, PieChartError};
use crate::charts::svg::{svg_root, svg_text, ViewBox};
use crate::charts::{series_color_var, ChartData};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};
use std::f64::consts::PI;

/// `data-scope="donut-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("donut-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "chart", "segment", "label"];

/// viewBox に対する中心 X 座標（[`crate::pie_chart`] と同一定数）。
const CENTER_X: f64 = 50.0;
/// viewBox に対する中心 Y 座標。
const CENTER_Y: f64 = 50.0;
/// viewBox に対する外径。
const OUTER_RADIUS: f64 = 45.0;

/// [`chart`] へ既定で付与する `aria-label`。
const DEFAULT_ARIA_LABEL: &str = "donut chart";

/// [`donut_chart`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct DonutChartProps<'a> {
    /// 寸法（既定 `Md`）。
    pub size: Size,
    /// `chart`（svg）へ付与する `aria-label`。`None` なら
    /// [`DEFAULT_ARIA_LABEL`]（`"donut chart"`）を使う。
    pub aria_label: Option<&'a str>,
    /// `true` ならカテゴリ名ラベルをセグメント上に描画する（既定 `false`）。
    pub show_labels: bool,
    /// 外径に対する内径の比率（既定 `0.6`）。`0.0 < ratio < 1.0` の範囲・
    /// 有限値であること（モジュール doc「内径」節参照）。
    pub inner_ratio: f64,
}

impl Default for DonutChartProps<'_> {
    fn default() -> Self {
        Self {
            size: Size::Md,
            aria_label: None,
            show_labels: false,
            inner_ratio: 0.6,
        }
    }
}

/// この styled DonutChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("donut-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("--fandhe-donut-chart-size", "16rem"),
            ],
        )
        .base(
            "chart",
            vec![
                decl("width", "var(--fandhe-donut-chart-size)"),
                decl("height", "var(--fandhe-donut-chart-size)"),
            ],
        )
        .base(
            "segment",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        .base(
            "label",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "6px"),
                decl("text-anchor", "middle"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-donut-chart-size", "10rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-donut-chart-size", "16rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-donut-chart-size", "22rem")],
        )
        .default_variant(Size::Md)
}

/// この styled DonutChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// DonutChart 1 個を組み立てる（`root` > `chart`(svg) > `segment`(path)
/// [+ `label`(text)]）。[`crate::pie_chart::pie_chart`] と同型の契約。
///
/// # Errors
///
/// - `data.series().len() != 1` の場合 [`PieChartError::MultiSeries`]
/// - `inner_ratio` が非有限、または `0.0 < ratio < 1.0` の範囲外の場合
///   [`PieChartError::InvalidInnerRatio`]
/// - 系列の値に非有限・負値が含まれる、または合計が `0` の場合
///   [`crate::charts::pie::segment_angles`] のエラーをそのまま返す
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps};
///
/// let data = ChartData::new(
///     vec!["A".to_string(), "B".to_string()],
///     vec![Series::new("total", vec![60.0, 40.0])],
/// )
/// .unwrap();
/// let node = donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"role="img""#));
/// ```
pub fn donut_chart<'a>(
    props: &DonutChartProps<'a>,
    data: &ChartData,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, PieChartError> {
    if data.series().len() != 1 {
        return Err(PieChartError::MultiSeries);
    }
    if !(props.inner_ratio.is_finite() && 0.0 < props.inner_ratio && props.inner_ratio < 1.0) {
        return Err(PieChartError::InvalidInnerRatio);
    }
    let categories = data.categories();
    let values = &data.series()[0].values;
    let angles = segment_angles(values)?;
    let r_inner = OUTER_RADIUS * props.inner_ratio;

    // 非ゼロ値のセグメントがちょうど 1 個の場合（全周セグメント）は
    // annulus path が始点=終点の退化 arc を返すため、半周ずつ 2 本へ
    // 分割してリング全体を描画する（モジュール doc「全周セグメントの
    // 描画」節）。
    let non_zero_count = values.iter().filter(|&&v| v > 0.0).count();
    let is_full_circle = non_zero_count == 1;

    let mut segment_and_label_nodes: Vec<Node> = Vec::new();
    for (i, (&(start, end), &value)) in angles.iter().zip(values.iter()).enumerate() {
        // 値 0 のセグメントは境界角が退化するため描画しない。
        if value <= 0.0 {
            continue;
        }
        let fill = series_color_var(i);
        if is_full_circle {
            let mid = start + PI;
            for (half_start, half_end) in [(start, mid), (mid, end)] {
                let d = annulus_sector_path(
                    CENTER_X,
                    CENTER_Y,
                    OUTER_RADIUS,
                    r_inner,
                    half_start,
                    half_end,
                );
                segment_and_label_nodes.push(el(
                    "path",
                    vec![
                        ("data-scope", "donut-chart"),
                        ("data-part", "segment"),
                        ("d", d.as_str()),
                        ("fill", fill.as_str()),
                    ],
                    vec![],
                ));
            }
        } else {
            let d = annulus_sector_path(CENTER_X, CENTER_Y, OUTER_RADIUS, r_inner, start, end);
            segment_and_label_nodes.push(el(
                "path",
                vec![
                    ("data-scope", "donut-chart"),
                    ("data-part", "segment"),
                    ("d", d.as_str()),
                    ("fill", fill.as_str()),
                ],
                vec![],
            ));
        }

        if props.show_labels {
            let mid = (start + end) / 2.0;
            // ラベル半径は外径・内径の中間（`(r_inner + r_outer) / 2`）付近に置く。
            let label_r = (r_inner + OUTER_RADIUS) / 2.0;
            let lx = CENTER_X + label_r * mid.cos();
            let ly = CENTER_Y + label_r * mid.sin();
            let category = categories.get(i).map(String::as_str).unwrap_or_default();
            let label_node = svg_text(
                lx,
                ly,
                vec![("data-scope", "donut-chart"), ("data-part", "label")],
                vec![text(category)],
            );
            segment_and_label_nodes.push(label_node);
        }
    }

    let view_box = ViewBox::new(0.0, 0.0, 100.0, 100.0)
        .expect("固定 viewBox 100x100 は常に有効な正の寸法である");
    let aria_label_value = props.aria_label.unwrap_or(DEFAULT_ARIA_LABEL);
    let chart_node = svg_root(
        &view_box,
        vec![
            ("data-scope", "donut-chart"),
            ("data-part", "chart"),
            ("aria-label", aria_label_value),
        ],
        segment_and_label_nodes,
    );

    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));

    Ok(ANATOMY.part("root", "div", merged, vec![chart_node]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::Series;
    use fandhe_frontend_core::render;

    fn two_category_data() -> ChartData {
        ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![60.0, 40.0])],
        )
        .unwrap()
    }

    #[test]
    fn renders_root_chart_and_segments_with_default_aria_label() {
        let node = donut_chart(&DonutChartProps::default(), &two_category_data(), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-scope="donut-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="donut-chart" data-part="chart""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="donut chart""#));
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
        // 環状 path の `d` 属性は `M...A...L...A...Z` の形（外周 arc + 内周
        // arc の 2 本）を持つ。1 セグメントあたり `A` が 2 回出現すること
        // で annulus_sector_path が実際に呼ばれていることを固定する。
        assert_eq!(html.matches("A45,45,0,").count(), 2);
        assert!(html.contains("L"));
    }

    #[test]
    fn custom_aria_label_overrides_default() {
        let props = DonutChartProps {
            aria_label: Some("revenue split"),
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains(r#"aria-label="revenue split""#));
    }

    #[test]
    fn show_labels_renders_category_text_nodes() {
        let props = DonutChartProps {
            show_labels: true,
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="label""#).count(), 2);
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    #[test]
    fn zero_value_segment_is_skipped() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![Series::new("total", vec![50.0, 0.0, 50.0])],
        )
        .unwrap();
        let html = render(&donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
    }

    #[test]
    fn single_non_zero_segment_splits_into_two_half_arcs() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![100.0, 0.0])],
        )
        .unwrap();
        let html = render(&donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
    }

    #[test]
    fn multi_series_is_rejected() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                Series::new("s1", vec![1.0, 2.0]),
                Series::new("s2", vec![3.0, 4.0]),
            ],
        )
        .unwrap();
        assert_eq!(
            donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::MultiSeries
        );
    }

    #[test]
    fn inner_ratio_boundary_and_non_finite_values_are_rejected() {
        for ratio in [0.0, 1.0, -0.1, 1.1, f64::NAN, f64::INFINITY] {
            let props = DonutChartProps {
                inner_ratio: ratio,
                ..DonutChartProps::default()
            };
            assert_eq!(
                donut_chart(&props, &two_category_data(), vec![]).unwrap_err(),
                PieChartError::InvalidInnerRatio,
                "ratio={ratio}"
            );
        }
    }

    #[test]
    fn zero_total_propagates_geometry_error() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(
            donut_chart(&DonutChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::ZeroTotal
        );
    }

    #[test]
    fn size_variant_applies_root_class() {
        let node = donut_chart(
            &DonutChartProps {
                size: Size::Sm,
                ..DonutChartProps::default()
            },
            &two_category_data(),
            vec![],
        )
        .unwrap();
        assert!(render(&node).contains("donut-chart--size-sm"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(
            &donut_chart(
                &DonutChartProps::default(),
                &two_category_data(),
                vec![("class", "attacker-controlled")],
            )
            .unwrap(),
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn css_output_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="donut-chart"][data-part="chart"]"#));
        assert!(!a.contains("color-palette"));
    }
}
