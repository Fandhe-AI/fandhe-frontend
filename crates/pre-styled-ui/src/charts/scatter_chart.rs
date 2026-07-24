//! ScatterChart（SVG 散布図、イシュー #851・親 Phase #845）。
//!
//! chakra-ui `charts/scatter-chart.md`（recharts `ScatterChart` 相当）を、
//! 外部依存ゼロ・決定的な SVG ノード木生成のみで再構成する。[`super::data::ChartData`]
//! はカテゴリ軸 + 系列値の形状（棒/折れ線向け）であり、散布図が必要とする
//! `(x, y)` 数値ペアの集合を表現できないため、本モジュールは独自に
//! [`ScatterSeries`]/[`ScatterData`] を定義する（`data.rs` は変更しない。
//! `bar_chart`/`radar_chart` 等 [`super::data::ChartData`] を使う並行実装との
//! 競合面を最小化する判断）。
//!
//! # レイアウト規則（決定的。本モジュールが唯一の正）
//!
//! 1. **2 軸線形スケール**: 全系列・全点を横断した x/y それぞれの
//!    `(min, max)` を算出し、[`super::scale::LinearScale::new`] → `nice()`
//!    を経由して `viewBox` の描画領域へ写像する（x: `(0, width)`、
//!    y: `(height, 0)` で SVG の下向き正の y 軸を反転する）。
//! 2. **退化 domain（`min == max`）**: [`super::data::ChartData::domain`] の
//!    片側パディング方針と同型に、`v` を中心とした対称区間 `(v - 1.0, v +
//!    1.0)` へ拡張してから `LinearScale::new` へ渡す（1 点のみ・全点同一
//!    座標のデータでも `ChartError::DegenerateDomain` を誘発しない）。
//! 3. **座標の文字列化**: すべて [`super::svg::fmt_coord`] のみを経由する
//!    （独自フォーマット禁止、[`crate::charts`] モジュール doc 不変条件 2）。
//! 4. **軸線・グリッド・凡例・ツールチップ**: 本モジュールのスコープ外
//!    （イシュー #847 が担当）。
//!
//! # a11y
//!
//! [`super::svg::svg_root`] が既定付与する `role="img"` に加え、呼び出し側
//! 必須の `aria_label` 引数を出力する（`bar_chart` と同型の alt 必須判断）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`super::svg`] 経由（`el`/`text` を最終的に呼ぶ）で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。系列名・`aria_label` は
//! テキストノード/属性値として [`fandhe_frontend_core::render`] の既定
//! エスケープを必ず通る。座標・半径は [`ScatterData::new`]/
//! [`super::scale::LinearScale::new`] が有限性検証済みの `f64` のみを
//! [`super::svg::fmt_coord`] へ渡すため、文字列注入経路を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 軸線・グリッド・凡例・ツールチップ（#847）。
//! - ホバーインタラクション・アニメーション（意図的非採用、
//!   `docs/policy/intentional-non-adoption.md`）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`bar_chart` の先例と同じ判断）。

use super::scale::LinearScale;
use super::svg::{self, ViewBox};
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// `data-scope="scatter-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &["root", "point"];

/// 1 系列分の散布点集合。
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterSeries {
    /// 系列名（`data-series` 属性値として出力する。凡例は #847 のスコープ）。
    pub name: String,
    /// `(x, y)` 座標列。
    pub points: Vec<(f64, f64)>,
}

impl ScatterSeries {
    /// 新しい系列を組み立てる（検証なしの薄いコンストラクタ。値の検証は
    /// [`ScatterData::new`] が一括で行う）。
    #[must_use]
    pub fn new(name: impl Into<String>, points: Vec<(f64, f64)>) -> Self {
        ScatterSeries {
            name: name.into(),
            points,
        }
    }
}

/// 散布図のデータモデル（系列の集合。カテゴリ軸を持たない、
/// [`super::data::ChartData`] とは独立した形状）。
///
/// [`ScatterData::new`] を経由した構築のみを公開し、以下を不変条件として
/// 保証する。
///
/// 1. 系列は 1 件以上、かつ全系列合計で点が 1 件以上。
/// 2. 全ての座標が有限（`NaN`/`±inf` を含まない）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterData {
    series: Vec<ScatterSeries>,
}

impl ScatterData {
    /// 系列群から散布図データを構築する。
    ///
    /// # Errors
    ///
    /// - `series` が空、または全系列合計で点が 0 件の場合
    ///   [`ChartError::EmptyData`]
    /// - いずれかの座標が `NaN`/`±inf` の場合 [`ChartError::NonFiniteValue`]
    pub fn new(series: Vec<ScatterSeries>) -> Result<Self, ChartError> {
        if series.is_empty() || series.iter().all(|s| s.points.is_empty()) {
            return Err(ChartError::EmptyData);
        }
        for s in &series {
            if s.points
                .iter()
                .any(|(x, y)| !x.is_finite() || !y.is_finite())
            {
                return Err(ChartError::NonFiniteValue);
            }
        }
        Ok(ScatterData { series })
    }

    /// 系列一覧（挿入順）を返す。
    #[must_use]
    pub fn series(&self) -> &[ScatterSeries] {
        &self.series
    }

    /// 全系列・全点を横断した x/y それぞれの値域 `(min, max)` を返す。
    ///
    /// [`ScatterData::new`] の不変条件により全系列合計で点は必ず 1 件以上
    /// かつ全値有限であるため、本関数は必ず有限な値域を返す。`min == max`
    /// の退化は [`super::data::ChartData::domain`] と同型に対称区間へ
    /// 拡張する（モジュール doc「レイアウト規則」節参照）。
    #[must_use]
    fn domain(&self) -> ((f64, f64), (f64, f64)) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for s in &self.series {
            for &(x, y) in &s.points {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        let x_domain = if min_x == max_x {
            (min_x - 1.0, max_x + 1.0)
        } else {
            (min_x, max_x)
        };
        let y_domain = if min_y == max_y {
            (min_y - 1.0, max_y + 1.0)
        } else {
            (min_y, max_y)
        };
        (x_domain, y_domain)
    }
}

/// [`root`] の描画パラメータ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterChartProps {
    /// `viewBox` の幅（px 相当。既定 480.0）。
    pub width: f64,
    /// `viewBox` の高さ（px 相当。既定 300.0）。
    pub height: f64,
    /// 点マーカーの半径（px 相当。既定 4.0）。
    pub point_radius: f64,
}

impl Default for ScatterChartProps {
    fn default() -> Self {
        ScatterChartProps {
            width: 480.0,
            height: 300.0,
            point_radius: 4.0,
        }
    }
}

/// この ScatterChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
///
/// 色は点ごとに [`series_color_var`] のインライン `fill` 属性で決まるため、
/// recipe は寸法系・視認性向上の最小宣言のみを持つ（[`crate::charts::bar_chart`]
/// と同型の「variant を持たない静的部品」判断）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("scatter-chart", SLOTS)
        .base(
            "root",
            vec![decl("display", "block"), decl("max-width", "100%")],
        )
        .base(
            "point",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1px"),
            ],
        )
}

/// この ScatterChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// ScatterChart 本体を組み立てる。
///
/// `aria_label` は `svg_root` の `role="img"` に対する代替テキストとして
/// 必須（モジュール doc「a11y」節参照）。
///
/// # Errors
///
/// - `props.width`/`props.height` が 0 以下、または非有限の場合
///   （[`ViewBox::new`] の失敗を変換して）[`ChartError::NonFiniteValue`]
/// - `props.point_radius` が非有限、または 0 以下の場合
///   [`ChartError::NonFiniteValue`]
/// - x/y いずれかの domain 算出後の [`LinearScale::new`] が失敗した場合、
///   その失敗をそのまま返す（[`ChartData::domain`] 同型の退化パディングに
///   より通常は発生しない）
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
///     root, ScatterChartProps, ScatterData, ScatterSeries,
/// };
///
/// let data = ScatterData::new(vec![ScatterSeries::new(
///     "a",
///     vec![(1.0, 2.0), (3.0, 4.0)],
/// )])
/// .unwrap();
/// let node = root(&data, ScatterChartProps::default(), "scatter demo").unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="scatter-chart" data-part="point""#));
/// ```
pub fn root(
    data: &ScatterData,
    props: ScatterChartProps,
    aria_label: &str,
) -> Result<Node, ChartError> {
    if !props.point_radius.is_finite() || props.point_radius <= 0.0 {
        return Err(ChartError::NonFiniteValue);
    }
    let view_box = ViewBox::new(0.0, 0.0, props.width, props.height)
        .map_err(|_| ChartError::NonFiniteValue)?;

    let (x_domain, y_domain) = data.domain();
    let x_scale = LinearScale::new(x_domain, (0.0, props.width))?.nice();
    let y_scale = LinearScale::new(y_domain, (props.height, 0.0))?.nice();

    let mut points: Vec<Node> = Vec::new();
    for (series_idx, series) in data.series().iter().enumerate() {
        let fill = series_color_var(series_idx);
        for &(x, y) in &series.points {
            let cx = x_scale.scale(x);
            let cy = y_scale.scale(y);
            points.push(svg::circle(
                cx,
                cy,
                props.point_radius,
                vec![
                    ("data-scope", "scatter-chart"),
                    ("data-part", "point"),
                    ("data-series", series.name.as_str()),
                    ("fill", fill.as_str()),
                ],
            ));
        }
    }

    Ok(svg::svg_root(
        &view_box,
        vec![
            ("data-scope", "scatter-chart"),
            ("data-part", "root"),
            ("aria-label", aria_label),
        ],
        points,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn sample_data() -> ScatterData {
        ScatterData::new(vec![
            ScatterSeries::new("a", vec![(0.0, 0.0), (10.0, 20.0)]),
            ScatterSeries::new("b", vec![(5.0, 5.0)]),
        ])
        .unwrap()
    }

    #[test]
    fn scatter_data_rejects_empty_series() {
        assert_eq!(ScatterData::new(vec![]).unwrap_err(), ChartError::EmptyData);
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![])]).unwrap_err(),
            ChartError::EmptyData
        );
    }

    #[test]
    fn scatter_data_rejects_non_finite_points() {
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![(f64::NAN, 0.0)])]).unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![(0.0, f64::INFINITY)])])
                .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_non_positive_point_radius() {
        let data = sample_data();
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    point_radius: 0.0,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    point_radius: f64::NAN,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_non_positive_view_box() {
        let data = sample_data();
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    width: 0.0,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_renders_role_img_and_aria_label() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "scatter demo").unwrap());
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="scatter demo""#));
        assert!(html.contains(r#"data-scope="scatter-chart" data-part="root""#));
    }

    #[test]
    fn root_renders_one_point_per_coordinate() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert_eq!(
            html.matches(r#"data-part="point""#).count(),
            3,
            "系列 a に 2 点、系列 b に 1 点で合計 3 点"
        );
    }

    #[test]
    fn root_cycles_series_color_var_across_series() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
    }

    #[test]
    fn root_is_deterministic() {
        let data = sample_data();
        let a = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        let b = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn root_handles_degenerate_single_point_domain() {
        let data = ScatterData::new(vec![ScatterSeries::new("a", vec![(3.0, 3.0)])]).unwrap();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert!(html.contains(r#"data-part="point""#));
    }

    #[test]
    fn root_escapes_series_name_and_aria_label() {
        let data = ScatterData::new(vec![ScatterSeries::new(
            "<script>alert(1)</script>",
            vec![(0.0, 0.0)],
        )])
        .unwrap();
        let html =
            render(&root(&data, ScatterChartProps::default(), "<script>xss</script>").unwrap());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="scatter-chart"][data-part="root"]"#));
        assert!(a.contains(r#"[data-scope="scatter-chart"][data-part="point"]"#));
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let css = css();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
