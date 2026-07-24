//! styled PieChart（イシュー #850、親 Phase #845）。
//!
//! chakra-ui `charts/pie-chart.md`（recharts `PieChart`/`Pie`/`Cell` 依存）
//! 相当の円グラフを、外部依存ゼロ・[`crate::charts`] 基盤の SVG ノード木
//! 生成のみで実装する（`docs/policy/intentional-non-adoption.md` §7 の
//! 保留解除、[`crate::charts`] モジュール doc「保留解除トリガー」参照）。
//!
//! ark-ui には対応する headless anatomy が存在しないため、[`crate::marquee`]/
//! [`crate::stat`] と同型の判断で headless-ui は変更せず、本クレートのみで
//! 新規 anatomy `data-scope="pie-chart"` を定義する。
//!
//! # anatomy（4 パーツ）
//!
//! - `root`（`<div>`）: 寸法 variant のクラスを持つ唯一のパーツ。
//! - `chart`（`<svg>`、[`crate::charts::svg::svg_root`] 経由）: `viewBox`
//!   `"0 0 100 100"` 固定・`role="img"`（`svg_root` が既定付与）・
//!   `aria-label`（既定 `"pie chart"`、[`PieChartProps::aria_label`] で上書き
//!   可能）。
//! - `segment`（`<path>`。全周セグメントは `<circle>`、後述）: 系列 1 本の
//!   各カテゴリに対応する扇形。塗り色は [`crate::charts::series_color_var`]
//!   （`chart-1`〜`chart-6` トークン循環）。
//! - `label`（`<text>`）: [`PieChartProps::show_labels`] が `true` の場合
//!   のみ出力するカテゴリ名ラベル（中間角位置、既定エスケープ経由の
//!   テキストノード、REQ-1）。
//!
//! # 幾何・角度計算
//!
//! 境界角の算出・丸め規則・境界規則（値 `0` セグメントのスキップ・単一
//! 全周セグメントの特別扱い）は [`crate::charts::pie`] モジュール doc を
//! 参照。固定寸法として中心 `(50, 50)`・外径 `r = 45`（viewBox
//! `"0 0 100 100"` に対する定数、[`root`]/[`chart`] doc 参照）を用いる。
//!
//! # 単一系列専用（多系列は fail-closed で拒否）
//!
//! 円グラフは「全体に対する各カテゴリの割合」を表す性質上、複数系列を
//! 同時に扇形へ写像する意味を持たない。[`pie_chart`] は
//! `data.series().len() != 1` の場合 [`PieChartError::MultiSeries`] を返す
//! （構築時 fail-closed、[`crate::charts::data::ChartData`] 自体は複数系列を
//! 許容する汎用モデルであるため、本モジュール側で追加検証する）。
//!
//! # `size` variant（寸法のみ）
//!
//! [`crate::recipe::Size`]（既定 `Md`）のみを `root` へ付与し、
//! `--fandhe-pie-chart-size` の root スコープ custom property（通常の CSS
//! 継承により `chart` へ伝わる）経由で寸法を切り替える（[`crate::qr_code`]
//! と同型）。`color-palette` 軸は提供しない（セグメント配色はチャート共通
//! パレットの循環で決まるため、[`crate::qr_code`] と同型の判断）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは `raw_html()` を使用しない。マークアップは `d`/`cx`/`cy`/
//! `r`/`x`/`y` 属性の数値文字列化を [`crate::charts::svg::fmt_coord`] のみに
//! 一元化した [`crate::charts::pie`]/[`crate::charts::svg`] のヘルパー経由
//! でのみ組み立て、任意文字列を SVG 属性値へ直接結合する経路を持たない。
//! カテゴリ名ラベル・`aria_label`・呼び出し側 `attrs` はすべて
//! `fandhe_frontend_core::render` の既定エスケープを経由する（REQ-1）。
//! `class` 属性は [`crate::class_attr::drop_class_attr`] により常に単一化
//! する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - Legend / Tooltip（#847）。
//! - アニメーション（chakra は `isAnimationActive={false}` を推奨例として
//!   おり非対応で整合）。
//! - `paddingAngle`・`startAngle`/`endAngle` の任意指定・カスタム shape・
//!   中央テキスト（呼び出し側 children での代替は本 API のスコープ外）。
//! - `examples/headless-pre-styled-ui` への反映は crates.io 公開後に別途
//!   （[`crate::qr_code`]/[`crate::rating_group`] の先例と同じ判断）。

use crate::charts::pie::{sector_path, segment_angles, PieChartError};
use crate::charts::svg::{circle, svg_root, svg_text, ViewBox};
use crate::charts::{series_color_var, ChartData};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="pie-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("pie-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "chart", "segment", "label"];

/// viewBox に対する中心 X 座標（固定、モジュール doc「幾何・角度計算」節）。
const CENTER_X: f64 = 50.0;
/// viewBox に対する中心 Y 座標（固定）。
const CENTER_Y: f64 = 50.0;
/// viewBox に対する外径（固定）。
const OUTER_RADIUS: f64 = 45.0;
/// ラベルを配置する半径（外径に対する比率。セグメント内側寄りに置く）。
const LABEL_RADIUS_RATIO: f64 = 0.6;

/// [`chart`] へ既定で付与する `aria-label`（[`PieChartProps::aria_label`]
/// が `None` の場合に使う）。
const DEFAULT_ARIA_LABEL: &str = "pie chart";

/// [`pie_chart`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct PieChartProps<'a> {
    /// 寸法（既定 `Md`）。
    pub size: Size,
    /// `chart`（svg）へ付与する `aria-label`。`None` なら
    /// [`DEFAULT_ARIA_LABEL`]（`"pie chart"`）を使う。
    pub aria_label: Option<&'a str>,
    /// `true` ならカテゴリ名ラベルをセグメント上に描画する（既定 `false`）。
    pub show_labels: bool,
}

impl Default for PieChartProps<'_> {
    fn default() -> Self {
        Self {
            size: Size::Md,
            aria_label: None,
            show_labels: false,
        }
    }
}

/// この styled PieChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("pie-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("--fandhe-pie-chart-size", "16rem"),
            ],
        )
        .base(
            "chart",
            vec![
                decl("width", "var(--fandhe-pie-chart-size)"),
                decl("height", "var(--fandhe-pie-chart-size)"),
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
            vec![decl("--fandhe-pie-chart-size", "10rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-pie-chart-size", "16rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-pie-chart-size", "22rem")],
        )
        .default_variant(Size::Md)
}

/// この styled PieChart が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// PieChart 1 個を組み立てる（`root` > `chart`(svg) > `segment`(path/circle)
/// [+ `label`(text)]）。
///
/// `data` はカテゴリ数 = セグメント数、系列数は必ず 1（モジュール doc
/// 「単一系列専用」節参照）。呼び出し側 `attrs` は `root` へ合成する
/// （`class` は [`drop_class_attr`] で除去してから recipe クラスへ一本化）。
///
/// # Errors
///
/// - `data.series().len() != 1` の場合 [`PieChartError::MultiSeries`]
/// - 系列の値に非有限・負値が含まれる、または合計が `0` の場合
///   [`crate::charts::pie::segment_angles`] のエラーをそのまま返す
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
///
/// let data = ChartData::new(
///     vec!["A".to_string(), "B".to_string()],
///     vec![Series::new("total", vec![60.0, 40.0])],
/// )
/// .unwrap();
/// let node = pie_chart(&PieChartProps::default(), &data, vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"role="img""#));
/// ```
pub fn pie_chart<'a>(
    props: &PieChartProps<'a>,
    data: &ChartData,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, PieChartError> {
    if data.series().len() != 1 {
        return Err(PieChartError::MultiSeries);
    }
    let categories = data.categories();
    let values = &data.series()[0].values;
    let angles = segment_angles(values)?;

    // 非ゼロ値のセグメントがちょうど 1 個の場合（全周セグメント）は
    // sector_path が始点=終点の退化 arc を返すため、代わりに <circle> を
    // 描画する（`crate::charts::pie` モジュール doc「境界規則」節）。
    let non_zero_count = values.iter().filter(|&&v| v > 0.0).count();
    let is_full_circle = non_zero_count == 1;

    let mut segment_and_label_nodes: Vec<Node> = Vec::new();
    for (i, (&(start, end), &value)) in angles.iter().zip(values.iter()).enumerate() {
        // 値 0 のセグメントは境界角が退化するため描画しない。
        if value <= 0.0 {
            continue;
        }
        let fill = series_color_var(i);
        let segment_node = if is_full_circle {
            circle(
                CENTER_X,
                CENTER_Y,
                OUTER_RADIUS,
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "segment"),
                    ("fill", fill.as_str()),
                ],
            )
        } else {
            let d = sector_path(CENTER_X, CENTER_Y, OUTER_RADIUS, start, end);
            el(
                "path",
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "segment"),
                    ("d", d.as_str()),
                    ("fill", fill.as_str()),
                ],
                vec![],
            )
        };
        segment_and_label_nodes.push(segment_node);

        if props.show_labels {
            let mid = (start + end) / 2.0;
            let lx = CENTER_X + OUTER_RADIUS * LABEL_RADIUS_RATIO * mid.cos();
            let ly = CENTER_Y + OUTER_RADIUS * LABEL_RADIUS_RATIO * mid.sin();
            let category = categories.get(i).map(String::as_str).unwrap_or_default();
            let label_node = svg_text(
                lx,
                ly,
                vec![("data-scope", "pie-chart"), ("data-part", "label")],
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
            ("data-scope", "pie-chart"),
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
        let node = pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-scope="pie-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="pie-chart" data-part="chart""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="pie chart""#));
        assert!(html.contains(r#"viewBox="0 0 100 100""#));
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
    }

    #[test]
    fn custom_aria_label_overrides_default() {
        let props = PieChartProps {
            aria_label: Some("revenue split"),
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains(r#"aria-label="revenue split""#));
        assert!(!html.contains("pie chart"));
    }

    #[test]
    fn show_labels_renders_category_text_nodes() {
        let props = PieChartProps {
            show_labels: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="label""#).count(), 2);
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    #[test]
    fn default_hides_labels() {
        let html =
            render(&pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap());
        assert!(!html.contains(r#"data-part="label""#));
    }

    #[test]
    fn zero_value_segment_is_skipped() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![Series::new("total", vec![50.0, 0.0, 50.0])],
        )
        .unwrap();
        let html = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
    }

    #[test]
    fn single_non_zero_segment_renders_as_circle() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![100.0, 0.0])],
        )
        .unwrap();
        let html = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches("<circle").count(), 1);
        assert!(!html.contains("<path"));
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
            pie_chart(&PieChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::MultiSeries
        );
    }

    #[test]
    fn zero_total_propagates_geometry_error() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(
            pie_chart(&PieChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::ZeroTotal
        );
    }

    #[test]
    fn size_variant_applies_root_class() {
        let node = pie_chart(
            &PieChartProps {
                size: Size::Lg,
                ..PieChartProps::default()
            },
            &two_category_data(),
            vec![],
        )
        .unwrap();
        assert!(render(&node).contains("pie-chart--size-lg"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(
            &pie_chart(
                &PieChartProps::default(),
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
        assert!(a.contains(r#"[data-scope="pie-chart"][data-part="chart"]"#));
        assert!(!a.contains("color-palette"));
    }
}
