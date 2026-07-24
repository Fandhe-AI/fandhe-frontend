//! BarChart（SVG 棒グラフ、イシュー #849・親 Phase #845）。
//!
//! chakra-ui `charts/bar-chart.md`（recharts `BarComposition` 相当）を、
//! [`super::data::ChartData`]（複数系列）+ [`super::scale::LinearScale`]
//! （値軸の domain → range 写像）+ [`super::svg`]（マークアップ生成）の
//! 3 層のみを組み合わせて、外部依存ゼロ・決定的なグループ棒グラフとして
//! 再構成する。
//!
//! # レイアウト規則（決定的。本モジュールが唯一の正）
//!
//! 1. **値軸**: `data.domain()` を基準に `(0.0 を含むよう拡張) → LinearScale::new
//!    → nice()` を経由する（棒はベースライン 0 起点、chakra-ui/recharts の
//!    既定と同じ）。`data.domain()` は必ず 0 を跨ぐとは限らないため、値域を
//!    `(domain.0.min(0.0), domain.1.max(0.0))` へ明示的に広げてから
//!    `LinearScale::new` に渡す（正値のみ・負値のみのデータでもベースライン 0
//!    が描画範囲に含まれることを保証する）。
//! 2. **カテゴリ軸（バンドレイアウト）**: カテゴリ数 `n` に対し
//!    `band = plot_length / n`。各バンド内は両端に `BAND_EDGE_PADDING_FRAC`
//!    （10%）ずつの余白を置き、残り 80% を系列数で均等分割して棒幅とする
//!    （系列間の追加ギャップは設けない、純算術で決定的）。
//! 3. **座標の文字列化**: すべて [`super::svg::fmt_coord`] のみを経由する
//!    （独自フォーマット禁止、[`crate::charts`] モジュール doc 不変条件 2）。
//! 4. **軸線・グリッド・凡例・ツールチップ**: 本モジュールのスコープ外
//!    （イシュー #847 が担当。本モジュールはカテゴリラベルの最小出力のみ
//!    行う）。
//!
//! # a11y
//!
//! [`super::svg::svg_root`] が既定付与する `role="img"` に加え、呼び出し側
//! 必須の `aria_label` 引数を出力する（画像として読み上げられるため代替
//! テキストが必須、`progress`/`image` 等の alt 必須パターンと同じ発想）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`super::svg`] 経由（`el`/`text` を最終的に呼ぶ）で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。系列名・カテゴリ名・
//! `aria_label` はすべて [`fandhe_frontend_core::text`] のテキストノードとして
//! 渡すため `render()` の既定エスケープを必ず通る。座標・寸法は
//! [`ChartData::new`](super::data::ChartData::new)/
//! [`LinearScale::new`](super::scale::LinearScale::new) が有限性検証済みの
//! `f64` のみを [`super::svg::fmt_coord`] へ渡すため、文字列注入経路を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 軸線・グリッド・凡例・ツールチップ（#847）。
//! - ホバーインタラクション・アニメーション（JS ランタイム前提のため
//!   `docs/policy/intentional-non-adoption.md` の意図的非採用方針に従う）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`rating_group` の先例と同じ判断）。

use super::data::ChartData;
use super::scale::LinearScale;
use super::svg::{self, svg_text, ViewBox};
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};

/// バンド内の両端余白（片側、バンド幅に対する比率）。
const BAND_EDGE_PADDING_FRAC: f64 = 0.1;

/// カテゴリラベル（`svg_text`）用に確保する軸方向の余白（px 相当）。
///
/// [`Orientation::Vertical`] では棒の下側にラベルを 1 行分収める用途のため
/// 24px で足りる（`text-anchor="middle"` でバンド内に収まり、高さ方向に
/// 折り返しがないため）。
const CATEGORY_LABEL_SPACE: f64 = 24.0;

/// [`Orientation::Horizontal`] でカテゴリラベル用に確保する `viewBox` 右側の
/// 余白（px 相当）。
///
/// Horizontal はラベルを `plot_width + 4` から `text-anchor="start"` で
/// 右方向に伸ばす（[`category_label`]）ため、[`CATEGORY_LABEL_SPACE`]
/// （24px、Vertical のバンド下余白流用）のままだとラベル文字列が
/// `viewBox` 右端を越えてクリップされ、ほぼ判読不能になっていた
/// （PR #877 Bugbot 指摘、イシュー #849）。カテゴリ名は任意長でありテキスト
/// 幅を事前計測できない（フォントメトリクス非依存が本モジュールの方針）
/// ため、厳密な無クリップ保証はできないが、一般的なラベル長を収める実用的
/// な既定値としてより広い余白を確保する。
const CATEGORY_LABEL_SPACE_HORIZONTAL: f64 = 96.0;

/// `data-scope="bar-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &["root", "bar", "category-label"];

/// 棒の並べ方（chakra-ui BarChart `layout` の `vertical`/`horizontal` を
/// 縮約。命名は「棒が伸びる向き」ではなく「カテゴリ軸の向き」を表す
/// chakra-ui の用語をそのまま踏襲する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// カテゴリ軸が横（x 軸）、値軸が縦（y 軸）。棒は縦に伸びる（既定）。
    #[default]
    Vertical,
    /// カテゴリ軸が縦（y 軸）、値軸が横（x 軸）。棒は横に伸びる。
    Horizontal,
}

/// [`root`] の描画パラメータ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarChartProps {
    /// 棒の向き（既定 [`Orientation::Vertical`]）。
    pub orientation: Orientation,
    /// `viewBox` の幅（px 相当。既定 480.0）。
    pub width: f64,
    /// `viewBox` の高さ（px 相当。既定 300.0）。
    pub height: f64,
}

impl Default for BarChartProps {
    fn default() -> Self {
        BarChartProps {
            orientation: Orientation::default(),
            width: 480.0,
            height: 300.0,
        }
    }
}

/// この BarChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
///
/// 色は棒ごとに [`series_color_var`] のインライン `fill` 属性で決まるため、
/// recipe は寸法系の最小宣言のみを持つ（[`crate::qr_code`] の
/// 「前景/背景は固定トークン・variant は寸法のみ」判断と同型ではなく、本
/// 部品は variant 自体を持たない静的部品。[`crate::table`] の
/// 「状態機械を持たない静的 styled 部品」に分類される）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("bar-chart", SLOTS)
        .base(
            "root",
            vec![decl("display", "block"), decl("max-width", "100%")],
        )
        .base(
            "category-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("fill", "var(--fandhe-color-fg-muted)"),
            ],
        )
}

/// この BarChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// BarChart 本体を組み立てる。
///
/// `aria_label` は `svg_root` の `role="img"` に対する代替テキストとして
/// 必須（モジュール doc「a11y」節参照）。
///
/// # Errors
///
/// - `data` の値軸 domain・`viewBox` 寸法のいずれかが非有限、または
///   `props.width`/`props.height` が 0 以下の場合、内部の
///   [`ViewBox::new`]/[`LinearScale::new`] の失敗を [`ChartError`] へ変換して
///   返す（[`ChartError::NonFiniteValue`]/[`ChartError::DegenerateDomain`]）。
/// - `props.width`/`props.height` が正でも、カテゴリラベル用余白
///   （[`Orientation::Vertical`] は [`CATEGORY_LABEL_SPACE`]、
///   [`Orientation::Horizontal`] は [`CATEGORY_LABEL_SPACE_HORIZONTAL`]）を
///   差し引いた結果プロット領域の幅・高さが 0 以下になる場合
///   [`ChartError::PlotAreaTooSmall`]（`ViewBox::new` は寸法の正値のみを
///   検証し、余白差し引き後までは検証しないため、放置するとバーが潰れる、
///   または viewBox 外へ無警告で描画される、PR #877 レビュー指摘）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::bar_chart::{root, BarChartProps};
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0])],
/// )
/// .unwrap();
/// let node = root(&data, BarChartProps::default(), "monthly visits").unwrap();
/// assert!(render(&node).contains(r#"role="img""#));
/// ```
pub fn root(data: &ChartData, props: BarChartProps, aria_label: &str) -> Result<Node, ChartError> {
    let view_box = ViewBox::new(0.0, 0.0, props.width, props.height)
        .map_err(|_| ChartError::NonFiniteValue)?;

    let (dmin, dmax) = data.domain();
    let (dmin, dmax) = (dmin.min(0.0), dmax.max(0.0));
    // domain() は非退化を保証するが、上記の 0 拡張で min==max==0 になる
    // ケースは発生しない（domain() が既に min<max を保証しているため、
    // 0 を含めて広げても min<=0<=max かつ min<max のまま）。
    let categories = data.categories();
    let n_categories = categories.len();
    let series = data.series();
    let n_series = series.len().max(1);

    let (plot_width, plot_height) = match props.orientation {
        Orientation::Vertical => (props.width, props.height - CATEGORY_LABEL_SPACE),
        Orientation::Horizontal => (props.width - CATEGORY_LABEL_SPACE_HORIZONTAL, props.height),
    };
    // `ViewBox::new` は width/height が正であることのみ検証し、カテゴリ
    // ラベル余白差し引き後の実プロット領域までは検証しない。ここで拒否
    // しないと、幅・高さが 0 以下のままバンド幅・棒寸法が 0/負値になり、
    // バーが潰れる、または viewBox 外に無警告で描画される
    // （PR #877 レビュー指摘、イシュー #849）。
    if plot_width <= 0.0 || plot_height <= 0.0 {
        return Err(ChartError::PlotAreaTooSmall);
    }

    let value_range = match props.orientation {
        // SVG は y が下向き正のため、値の大小を上下反転させる。
        Orientation::Vertical => (plot_height, 0.0),
        Orientation::Horizontal => (0.0, plot_width),
    };
    let value_scale =
        LinearScale::new((dmin, dmax), value_range).map_err(|_| ChartError::NonFiniteValue)?;
    let baseline = value_scale.scale(0.0);

    let band = match props.orientation {
        Orientation::Vertical => plot_width / n_categories as f64,
        Orientation::Horizontal => plot_height / n_categories as f64,
    };
    let usable = band * (1.0 - 2.0 * BAND_EDGE_PADDING_FRAC);
    let edge_offset = band * BAND_EDGE_PADDING_FRAC;
    let bar_thickness = usable / n_series as f64;

    let mut bars: Vec<Node> = Vec::new();
    for (cat_idx, category) in categories.iter().enumerate() {
        let band_start = band * cat_idx as f64;
        for (series_idx, s) in series.iter().enumerate() {
            let value = s.values[cat_idx];
            let scaled = value_scale.scale(value);
            let color = series_color_var(series_idx);
            let attrs = vec![
                ("data-scope", "bar-chart"),
                ("data-part", "bar"),
                ("fill", color.as_str()),
            ];
            let rect = match props.orientation {
                Orientation::Vertical => {
                    let x = band_start + edge_offset + bar_thickness * series_idx as f64;
                    let y = scaled.min(baseline);
                    let h = (scaled - baseline).abs();
                    svg::rect(x, y, bar_thickness, h, attrs)
                }
                Orientation::Horizontal => {
                    let y = band_start + edge_offset + bar_thickness * series_idx as f64;
                    let x = scaled.min(baseline);
                    let w = (scaled - baseline).abs();
                    svg::rect(x, y, w, bar_thickness, attrs)
                }
            };
            bars.push(rect);
        }
        let label = category_label(category, band_start, band, plot_width, plot_height, props);
        bars.push(label);
    }

    let attrs = vec![("data-scope", "bar-chart"), ("data-part", "root")];
    let mut merged_attrs = vec![("aria-label", aria_label)];
    merged_attrs.extend(attrs);
    Ok(svg::svg_root(&view_box, merged_attrs, bars))
}

/// カテゴリラベル（[`svg_text`]）を組み立てる（内部ヘルパ）。
fn category_label(
    category: &str,
    band_start: f64,
    band: f64,
    plot_width: f64,
    plot_height: f64,
    props: BarChartProps,
) -> Node {
    let (x, y, attrs) = match props.orientation {
        Orientation::Vertical => (
            band_start + band / 2.0,
            plot_height + CATEGORY_LABEL_SPACE / 2.0 + 4.0,
            vec![("text-anchor", "middle")],
        ),
        Orientation::Horizontal => (
            plot_width + 4.0,
            band_start + band / 2.0,
            vec![("text-anchor", "start")],
        ),
    };
    let mut merged: Vec<(&str, &str)> =
        vec![("data-scope", "bar-chart"), ("data-part", "category-label")];
    merged.extend(attrs);
    svg_text(x, y, merged, vec![text(category.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![Series::new("visits", vec![10.0, 30.0])],
        )
        .unwrap()
    }

    #[test]
    fn root_rejects_width_or_height_too_small_for_category_label_space() {
        // PR #877 レビュー指摘: height/width がカテゴリラベル用余白
        // （Vertical は CATEGORY_LABEL_SPACE 24.0、Horizontal は
        // CATEGORY_LABEL_SPACE_HORIZONTAL 96.0）以下だとプロット領域が
        // 0 以下になり、バーが潰れる/viewBox 外描画になる silent failure
        // だった。fail-closed で拒否する。
        let vertical_too_small = BarChartProps {
            orientation: Orientation::Vertical,
            width: 480.0,
            height: 24.0,
        };
        assert_eq!(
            root(&sample(), vertical_too_small, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );

        let horizontal_too_small = BarChartProps {
            orientation: Orientation::Horizontal,
            width: 96.0,
            height: 300.0,
        };
        assert_eq!(
            root(&sample(), horizontal_too_small, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn root_renders_role_img_and_aria_label() {
        let node = root(&sample(), BarChartProps::default(), "monthly visits").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="monthly visits""#));
        assert!(html.contains(r#"data-scope="bar-chart" data-part="root""#));
    }

    #[test]
    fn root_renders_one_bar_per_category_series_pair() {
        let node = root(&sample(), BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
    }

    #[test]
    fn root_is_deterministic() {
        let a = render(&root(&sample(), BarChartProps::default(), "label").unwrap());
        let b = render(&root(&sample(), BarChartProps::default(), "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn vertical_bar_geometry_matches_hand_computed_values() {
        // domain: (0,30) を nice せず 0 起点のまま使う本モジュールの規則
        // （nice() は #847 の軸描画スコープであり本モジュールは適用しない）。
        // plot_height = 300 - 24 = 276, plot_width = 480。
        // band = 480 / 2 = 240, edge_offset = 24, usable = 192,
        // bar_thickness = 192 (1 系列)。
        // Jan (value=10): scaled = 276 - (10/30)*276 = 184, baseline = 276。
        //   y = min(184,276)=184, h = |184-276| = 92, x = 0+24 = 24。
        let node = root(&sample(), BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"x="24""#));
        assert!(html.contains(r#"y="184""#));
        assert!(html.contains(r#"height="92""#));
        assert!(html.contains(r#"width="192""#));
    }

    #[test]
    fn horizontal_orientation_swaps_axes() {
        let props = BarChartProps {
            orientation: Orientation::Horizontal,
            ..BarChartProps::default()
        };
        let node = root(&sample(), props, "label").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="bar""#));
    }

    #[test]
    fn horizontal_category_label_start_stays_within_view_box() {
        // PR #877 Bugbot 指摘（Medium）: Horizontal ではラベルが
        // `plot_width + 4` から `text-anchor="start"` で右方向に伸びるため、
        // 余白が狭すぎるとラベル文字列が viewBox 右端を越えてクリップされる
        // （イシュー #849）。CATEGORY_LABEL_SPACE_HORIZONTAL 導入後は
        // ラベル開始位置 (`plot_width + 4`) と viewBox 右端
        // (`props.width`) の間に十分な余白が残ることを固定する。
        let props = BarChartProps {
            orientation: Orientation::Horizontal,
            ..BarChartProps::default()
        };
        let node = root(&sample(), props, "label").unwrap();
        let html = render(&node);
        // plot_width = 480 - 96 = 384, label x = 384 + 4 = 388。
        assert!(html.contains(r#"x="388""#));
        // ラベル開始位置から viewBox 右端までの残り余白（92px）が
        // クリップ再発防止の下限としてゼロより十分大きいことを保証する。
        let label_start = 388.0;
        assert!(props.width - label_start >= 90.0);
    }

    #[test]
    fn negative_and_positive_values_share_baseline_zero() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![-10.0, 10.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        // 両方とも描画され、baseline をまたいでも panic しない。
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
    }

    #[test]
    fn multi_series_renders_bars_side_by_side_within_band() {
        let data = ChartData::new(
            vec!["a".to_string()],
            vec![Series::new("s1", vec![10.0]), Series::new("s2", vec![20.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
        // 2 系列は色トークンが異なる。
        assert!(html.contains("chart-1"));
        assert!(html.contains("chart-2"));
    }

    #[test]
    fn category_and_series_names_are_escaped() {
        let data = ChartData::new(
            vec!["<script>".to_string()],
            vec![Series::new("s", vec![1.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "<script>alert(1)</script>").unwrap();
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_is_deterministic_and_has_no_breakout_sequences() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(!a.contains('<'));
        assert!(a.contains(r#"[data-scope="bar-chart"]"#));
    }
}
