//! RadarChart（SVG レーダーチャート、イシュー #851・親 Phase #845）。
//!
//! chakra-ui `charts/radar-chart.md`（recharts `RadarChart` 相当）を、
//! [`super::data::ChartData`]（カテゴリ = 軸、系列 = ポリゴン）+
//! [`super::scale::LinearScale`]（半径写像）+ [`super::svg`]（マークアップ
//! 生成）の 3 層のみで外部依存ゼロ・決定的に再構成する。
//!
//! # レイアウト規則（決定的。本モジュールが唯一の正）
//!
//! 1. **頂点角度**: 軸数 `n`・軸 index `i`（0 始まり）に対し
//!    `θ_i = -π/2 + i · 2π / n`（12 時方向開始・時計回り、chakra-ui/recharts
//!    既定と同じ見え方）。頂点座標は `(cx + r·cos θ_i, cy + r·sin θ_i)`。
//!    角度→座標変換は private ヘルパ [`vertex`] に一元化し、純 f64 算術
//!    （`f64::sin`/`f64::cos`）のみで入力から一意に決まる。文字列化は
//!    [`super::svg::fmt_coord`] のみを経由する（[`crate::charts`] モジュール
//!    doc 不変条件 2）。
//! 2. **軸数の下限**: 軸（`categories`）が 3 未満では多角形が定義できない
//!    ため [`ChartError::TooFewAxes`] として構築前に拒否する。
//! 3. **負値の拒否**: 半径写像は `0` を起点とするため、系列値に負値が
//!    含まれる場合 [`ChartError::NegativeValue`] として拒否する。
//! 4. **半径スケール**: domain `(0.0, 全系列中の最大値)` → range
//!    `(0.0, plot_radius)` の [`LinearScale`]（`.nice()` 適用）。全値 0 の
//!    退化は domain を `(0.0, 1.0)` へ拡張して回避する。
//! 5. **プロット領域**: `viewBox` は `size × size` の正方形。軸ラベル用に
//!    [`AXIS_LABEL_MARGIN`] を差し引いた半径を `plot_radius` とする。
//!    `plot_radius` が 0 以下になる場合 [`ChartError::PlotAreaTooSmall`]
//!    （[`super::bar_chart`] の `PlotAreaTooSmall` と同型の fail-closed 判断、
//!    `ViewBox::new` は寸法の正値のみを検証し、ラベル余白差し引き後までは
//!    検証しないため）。
//!
//! # a11y
//!
//! [`super::svg::svg_root`] が既定付与する `role="img"` に加え、呼び出し側
//! 必須の `aria_label` 引数を出力する（`bar_chart`/`scatter_chart` と同型の
//! alt 必須判断）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`super::svg`]/[`fandhe_frontend_core::el`] 経由で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。カテゴリ名（軸ラベル）・
//! 系列名・`aria_label` はテキストノード/属性値として
//! [`fandhe_frontend_core::render`] の既定エスケープを必ず通る。座標・半径・
//! `d` 属性はすべて [`ChartData::new`](super::data::ChartData::new)/
//! [`LinearScale::new`] が有限性検証済みの `f64` のみを
//! [`super::svg::fmt_coord`]/[`super::svg::PathBuilder`] へ渡すため、
//! 文字列注入経路を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 凡例・ツールチップ（#847）。
//! - ホバーインタラクション・アニメーション（意図的非採用、
//!   `docs/policy/intentional-non-adoption.md`）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`bar_chart` の先例と同じ判断）。
//!
//! # `data-series` 語彙（イシュー #1063）
//!
//! `data-series`（系列ポリゴン要素へ付与、値は系列名）は
//! `fandhe-frontend-headless-ui` に対応部品を持たない pre-styled-only 語彙
//! である（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B、
//! [`super::scatter_chart`] と共通）。現在の recipe に CSS 消費者はなく、
//! 利用者側 CSS/JS が任意でフックするための識別子に留まる。
//!
//! # 参考サイト基準への調整（イシュー #1597）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にレーダー
//! チャート部品が存在しないため、評価軸は**内部整合のみ**（`--fandhe-*`
//! トークン適用・ダーク時の軸/グリッドの可読性・系列色の識別性・ラベルの
//! コントラスト）に限定する。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 非該当（`RadarChartProps::size` は viewBox 一辺の px 相当長で
//!   `Size` variant 軸ではない。新設は 0.x 破壊的変更＝minor バンプ対象で
//!   「内部整合のみ」の評価軸を超えるため非採用） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6`
//!   固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない
//!   pre-styled-only 部品） |
//! | ダークモード | 系列ポリゴンの輪郭を太く・丸めて識別性を上げた（下記
//!   「是正した点」）。系列パレット自体の見直しはスコープ外 |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（ポリゴン SVG 描画のみ） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `series` slot に `stroke-width: 2` / `stroke-linejoin: round` を
//!   追加した。兄弟部品 [`crate::line_chart`]（#1595）/
//!   [`crate::area_chart`]（#1589）の `series-line` は `stroke-width: 2` +
//!   `stroke-linejoin: round` を持つが、radar の `series`
//!   （[`polygon_d`] が生成する閉多角形）は UA 既定の `stroke-width: 1` /
//!   `stroke-linejoin: miter` のままで、`fill-opacity: 0.2` の薄い塗りに
//!   対し輪郭が系列識別の主要素であるにもかかわらず兄弟部品より細く、
//!   鋭角頂点（値の谷）で miter が尖って突出していた。輪郭幅・結合方式を
//!   兄弟部品と揃えて系列の識別性を上げた
//! - `axis-label` slot に `font-family: var(--fandhe-font-font-body)` を
//!   追加した。[`super::axis`] の `tick-label` は同トークンを明示するが、
//!   radar の軸ラベルは書体指定が欠けていた（SVG テキストは祖先から
//!   `font-family` を継承するため描画欠陥ではないが、`charts::axis` の
//!   軸ラベルとのトークン整合を取った）
//!
//! ## 意図的に合わせなかった点
//!
//! - `grid`/`spoke` の `stroke: var(--fandhe-color-border)` は維持した。
//!   `charts::grid` の `grid-line`（#1866 で `border-muted` を意図的に
//!   維持）と異なり、レーダーの同心多角形は目盛ラベルを持たない値スケール
//!   そのもの（軸線 + 目盛の役割）であり `charts::axis` の
//!   `axis-line`/`tick-line`（#1593 で `border` へ統一）と同格と判断した。
//!   `border-muted` 化すると dark モードで環が背景に沈み値スケールが
//!   読めなくなる
//! - `series` の `fill-opacity: 0.2` は維持した（area-chart と同じ
//!   chakra/recharts 既定準拠）
//! - `root` への `overflow: visible` は不要（ポリゴン最大半径は
//!   `plot_radius` 以下、`AXIS_LABEL_MARGIN` 60 単位の余白があるため
//!   `stroke-width: 2` でも viewBox 内に収まる）
//! - `axis-label` へのハロー（`paint-order: stroke`）は不要（プロット外側
//!   のページ背景上に配置され、`fg-muted` は light/dark とも WCAG 4.5:1 を
//!   十分に上回るコントラストを持つ）
//! - `series` への `vector-effect: non-scaling-stroke` は非採用（兄弟部品
//!   との線幅の見え方乖離回避、#1593/#1595/#1596 と同じ判断）
//! - 系列パレット（`chart-1〜6`）の dark 近接見直しはスコープ外（#1866/#1867
//!   と同じ判断）

use std::f64::consts::PI;

use super::data::ChartData;
use super::scale::LinearScale;
use super::svg::{self, svg_text, PathBuilder, ViewBox};
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// `data-scope="radar-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &["root", "grid", "spoke", "axis-label", "series"];

/// 軸ラベル用に確保する半径方向の余白（px 相当。[`super::bar_chart`] の
/// `CATEGORY_LABEL_SPACE` と同型の判断）。
///
/// `AXIS_LABEL_MARGIN - AXIS_LABEL_OFFSET` が「ラベル アンカー点から
/// `viewBox` 外周までの実利用可能幅」（`root` 内の式変形を参照。
/// `plot_radius = size / 2 - AXIS_LABEL_MARGIN` かつラベルは半径
/// `plot_radius + AXIS_LABEL_OFFSET` に配置するため、`size` に依存せず
/// 一定値になる）。side ラベル（`text-anchor` `start`/`end`）はこの幅の
/// 方向へ全体が伸びるため、幅が狭いと通常の長さのカテゴリ名でも
/// `viewBox` をはみ出してクリップまたはレイアウトへ食い込む
/// （Cursor Bugbot 指摘、イシュー #851 追補）。本モジュールはテキスト幅を
/// 測定する手段を持たない（外部依存ゼロ・決定的レンダリングの制約）ため、
/// `font-size xs`（≒0.75rem/12px、1 文字あたり概ね 7〜8px と仮定）で
/// 本モジュールが実際に描画する最長ラベル（doctest/テストの `"control"`
/// 7 文字、既定 `size` 300.0 でも size に依存せず一定）が収まる下限として
/// 54px（`AXIS_LABEL_MARGIN` 60.0 − `AXIS_LABEL_OFFSET` 6.0）を確保する
/// （下記 `axis_label_side_budget_fits_longest_known_label` が固定する契約）。
/// これより著しく長いカテゴリ名を使う場合は呼び出し側で `size` を大きくする
/// か短縮する必要がある（本モジュールはテキスト幅を計測できないため
/// 自動対応しない）。
const AXIS_LABEL_MARGIN: f64 = 60.0;

/// `plot_radius` の外側、軸ラベルを配置する追加オフセット（px 相当。
/// [`AXIS_LABEL_MARGIN`] のドキュメント参照）。
const AXIS_LABEL_OFFSET: f64 = 6.0;

/// グリッド（同心正多角形）の目安本数（[`LinearScale::ticks`] の `target`）。
const GRID_TICK_TARGET: usize = 4;

/// `text-anchor` 分岐のしきい値（`cos(θ)` がこの絶対値未満なら中央揃え、
/// 浮動小数点誤差を吸収する）。
const ANCHOR_EPSILON: f64 = 1e-6;

/// [`root`] の描画パラメータ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarChartProps {
    /// `viewBox` の一辺の長さ（正方形、px 相当。既定 300.0）。
    pub size: f64,
}

impl Default for RadarChartProps {
    fn default() -> Self {
        RadarChartProps { size: 300.0 }
    }
}

/// 軸 index `i`（`0..n`）の頂点角度（ラジアン、12 時方向開始・時計回り）を
/// 返す（モジュール doc「レイアウト規則」節 1 の式そのもの）。
#[must_use]
fn vertex_angle(i: usize, n: usize) -> f64 {
    -PI / 2.0 + (i as f64) * 2.0 * PI / (n as f64)
}

/// 中心 `(cx, cy)`・半径 `r`・軸 index `i`（`0..n`）から頂点座標を返す
/// （角度→座標変換の唯一の実装箇所、モジュール doc「レイアウト規則」節参照）。
#[must_use]
fn vertex(cx: f64, cy: f64, r: f64, i: usize, n: usize) -> (f64, f64) {
    let theta = vertex_angle(i, n);
    (cx + r * theta.cos(), cy + r * theta.sin())
}

/// `n` 頂点の正多角形を閉じた `path` の `d` 属性値へ組み立てる。
#[must_use]
fn polygon_d(cx: f64, cy: f64, r: f64, n: usize) -> String {
    let mut builder = PathBuilder::new();
    for i in 0..n {
        let (x, y) = vertex(cx, cy, r, i, n);
        builder = if i == 0 {
            builder.move_to(x, y)
        } else {
            builder.line_to(x, y)
        };
    }
    builder.close().build()
}

/// この RadarChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
///
/// `series` パーツの塗りは半透明固定（`fill-opacity: 0.2`）とし、動的な
/// 透過度を CSS 値へ流し込む経路は作らない（色自体はインライン `fill`
/// 属性、[`crate::charts::bar_chart`] と同型の「variant を持たない静的
/// 部品」判断）。輪郭（`stroke-width`/`stroke-linejoin`）は兄弟部品
/// （`line_chart`/`area_chart` の `series-line`）と揃えた静的値であり
/// 系列ごとに変化しない（モジュール doc「参考サイト基準への調整」節参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("radar-chart", SLOTS)
        .base(
            "root",
            vec![decl("display", "block"), decl("max-width", "100%")],
        )
        .base(
            "grid",
            vec![
                decl("stroke", "var(--fandhe-color-border)"),
                decl("fill", "none"),
            ],
        )
        .base("spoke", vec![decl("stroke", "var(--fandhe-color-border)")])
        .base(
            "axis-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("fill", "var(--fandhe-color-fg-muted)"),
                // イシュー #1597: charts::axis の tick-label と同じトークンで
                // 軸ラベルの書体指定を統一する（SVG テキストは祖先から
                // font-family を継承するため描画欠陥の修正ではなく、charts
                // 共通軸ラベルとのトークン整合）。
                decl("font-family", "var(--fandhe-font-font-body)"),
            ],
        )
        .base(
            "series",
            vec![
                decl("fill-opacity", "0.2"),
                // イシュー #1597: 兄弟部品 line-chart（#1595）/area-chart
                // （#1589）の series-line と輪郭幅・結合方式を揃え、薄い
                // 塗り（fill-opacity 0.2）に対する系列識別の主要素である
                // 輪郭を太く・鋭角頂点での miter 突出を防ぐ。
                decl("stroke-width", "2"),
                decl("stroke-linejoin", "round"),
            ],
        )
}

/// この RadarChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// RadarChart 本体を組み立てる。
///
/// `data.categories()` を軸、`data.series()` を系列ポリゴンとして描画する。
/// `aria_label` は `svg_root` の `role="img"` に対する代替テキストとして
/// 必須（モジュール doc「a11y」節参照）。
///
/// # Errors
///
/// - 軸数（`data.categories().len()`）が 3 未満の場合 [`ChartError::TooFewAxes`]
/// - いずれかの系列値が負の場合 [`ChartError::NegativeValue`]
/// - `props.size` が非有限・0 以下の場合 [`ChartError::NonFiniteValue`]
/// - `props.size` からラベル余白を差し引いた `plot_radius` が 0 以下の場合
///   [`ChartError::PlotAreaTooSmall`]
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::charts::radar_chart::{root, RadarChartProps};
///
/// let data = ChartData::new(
///     vec!["speed".into(), "power".into(), "range".into(), "control".into()],
///     vec![Series::new("mercury", vec![80.0, 60.0, 40.0, 90.0])],
/// )
/// .unwrap();
/// let node = root(&data, RadarChartProps::default(), "stat comparison").unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="radar-chart" data-part="series""#));
/// ```
pub fn root(
    data: &ChartData,
    props: RadarChartProps,
    aria_label: &str,
) -> Result<Node, ChartError> {
    let axes = data.categories();
    let n = axes.len();
    if n < 3 {
        return Err(ChartError::TooFewAxes);
    }
    if data
        .series()
        .iter()
        .any(|s| s.values.iter().any(|&v| v < 0.0))
    {
        return Err(ChartError::NegativeValue);
    }
    if !props.size.is_finite() || props.size <= 0.0 {
        return Err(ChartError::NonFiniteValue);
    }
    let view_box =
        ViewBox::new(0.0, 0.0, props.size, props.size).map_err(|_| ChartError::NonFiniteValue)?;

    let plot_radius = props.size / 2.0 - AXIS_LABEL_MARGIN;
    if plot_radius <= 0.0 {
        return Err(ChartError::PlotAreaTooSmall);
    }
    let center = props.size / 2.0;

    let max_value = data
        .series()
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .fold(f64::NEG_INFINITY, f64::max);
    let domain_max = if max_value <= 0.0 { 1.0 } else { max_value };
    let value_scale = LinearScale::new((0.0, domain_max), (0.0, plot_radius))?.nice();

    let mut children: Vec<Node> = Vec::new();

    // グリッド（同心正多角形）。tick 0 は中心の 1 点に潰れ描画上意味を
    // 持たないため除外する。
    for tick in value_scale
        .ticks(GRID_TICK_TARGET)?
        .into_iter()
        .filter(|t| *t > 0.0)
    {
        let r = value_scale.scale(tick);
        let d = polygon_d(center, center, r, n);
        children.push(el(
            "path",
            vec![
                ("data-scope", "radar-chart"),
                ("data-part", "grid"),
                ("d", d.as_str()),
            ],
            vec![],
        ));
    }

    // スポーク（中心 → 各軸の外周頂点）。
    for i in 0..n {
        let (x, y) = vertex(center, center, plot_radius, i, n);
        children.push(svg::line(
            center,
            center,
            x,
            y,
            vec![("data-scope", "radar-chart"), ("data-part", "spoke")],
        ));
    }

    // 軸ラベル。`text-anchor` は象限（cos(θ) の符号）で決定的に分岐する。
    for (i, category) in axes.iter().enumerate() {
        let theta = vertex_angle(i, n);
        let (x, y) = vertex(center, center, plot_radius + AXIS_LABEL_OFFSET, i, n);
        let anchor = if theta.cos() > ANCHOR_EPSILON {
            "start"
        } else if theta.cos() < -ANCHOR_EPSILON {
            "end"
        } else {
            "middle"
        };
        // 垂直方向のアラインメントも `sin(θ)`（象限の上下）で決定的に分岐する。
        // 既定のアルファベティックベースラインは常にテキストが `y` 座標から
        // 上方向へ伸びるため、下側（bottom、`sin(θ) > 0`）の軸ラベルはプロット
        // 内部（スポーク・グリッドリング・外周付近の系列）へ向かって重なって
        // しまう（Cursor Bugbot 指摘、イシュー #851 追補）。下側ラベルは
        // `hanging`（`y` 座標から下方向、プロットの外側へ伸びる）へ、上側
        // （`sin(θ) < 0`）は既定の `auto`（上方向、プロットの外側へ伸びる）の
        // ままとし、水平軸上（`sin(θ) ≈ 0`）は `middle` で中央揃えにする。
        let baseline = if theta.sin() > ANCHOR_EPSILON {
            "hanging"
        } else if theta.sin() < -ANCHOR_EPSILON {
            "auto"
        } else {
            "middle"
        };
        children.push(svg_text(
            x,
            y,
            vec![
                ("data-scope", "radar-chart"),
                ("data-part", "axis-label"),
                ("text-anchor", anchor),
                ("dominant-baseline", baseline),
            ],
            vec![text(category.as_str())],
        ));
    }

    // 系列ポリゴン。
    for (series_idx, series) in data.series().iter().enumerate() {
        let color = series_color_var(series_idx);
        let mut builder = PathBuilder::new();
        for (i, &value) in series.values.iter().enumerate() {
            let r = value_scale.scale(value);
            let (x, y) = vertex(center, center, r, i, n);
            builder = if i == 0 {
                builder.move_to(x, y)
            } else {
                builder.line_to(x, y)
            };
        }
        let d = builder.close().build();
        children.push(el(
            "path",
            vec![
                ("data-scope", "radar-chart"),
                ("data-part", "series"),
                ("data-series", series.name.as_str()),
                ("d", d.as_str()),
                ("fill", color.as_str()),
                ("stroke", color.as_str()),
            ],
            vec![],
        ));
    }

    Ok(svg::svg_root(
        &view_box,
        vec![
            ("data-scope", "radar-chart"),
            ("data-part", "root"),
            ("aria-label", aria_label),
        ],
        children,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample_data(n: usize) -> ChartData {
        let categories: Vec<String> = (0..n).map(|i| format!("axis{i}")).collect();
        let values: Vec<f64> = (0..n).map(|i| 10.0 * (i as f64 + 1.0)).collect();
        ChartData::new(categories, vec![Series::new("s1", values)]).unwrap()
    }

    #[test]
    fn root_rejects_fewer_than_three_axes() {
        for n in [0usize, 1, 2] {
            if n == 0 {
                // ChartData::new 自体が空カテゴリを EmptyData として拒否するため、
                // TooFewAxes の対象は 1・2 軸のみ（`ChartData::new` 経由での
                // 到達可能な最小値）。
                continue;
            }
            let data = sample_data(n);
            assert_eq!(
                root(&data, RadarChartProps::default(), "label").unwrap_err(),
                ChartError::TooFewAxes
            );
        }
    }

    #[test]
    fn root_accepts_exactly_three_axes() {
        let data = sample_data(3);
        assert!(root(&data, RadarChartProps::default(), "label").is_ok());
    }

    #[test]
    fn root_rejects_negative_values() {
        let data = ChartData::new(
            vec!["a".into(), "b".into(), "c".into()],
            vec![Series::new("s1", vec![1.0, -2.0, 3.0])],
        )
        .unwrap();
        assert_eq!(
            root(&data, RadarChartProps::default(), "label").unwrap_err(),
            ChartError::NegativeValue
        );
    }

    #[test]
    fn root_rejects_non_positive_or_non_finite_size() {
        let data = sample_data(4);
        assert_eq!(
            root(&data, RadarChartProps { size: 0.0 }, "label").unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            root(&data, RadarChartProps { size: f64::NAN }, "label").unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_plot_area_too_small() {
        let data = sample_data(4);
        // AXIS_LABEL_MARGIN (60.0) * 2 = 120.0 以下では plot_radius <= 0。
        assert_eq!(
            root(&data, RadarChartProps { size: 60.0 }, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn axis_label_side_budget_fits_longest_known_label() {
        // side ラベル（`text-anchor` start/end）のアンカー点から `viewBox`
        // 外周までの実利用可能幅は `AXIS_LABEL_MARGIN - AXIS_LABEL_OFFSET`
        // （`size` に依存せず一定、`AXIS_LABEL_MARGIN` doc 参照）。本モジュール
        // が実際に描画する最長ラベル（doctest の "control", 7 文字）が、
        // `font-size xs` での 1 文字あたり想定幅（7.5px、doc 記載の 7〜8px
        // 仮定の中央値）で収まることを固定する回帰テスト（Cursor Bugbot 指摘、
        // イシュー #851 追補。この余白を将来縮小する変更は本テストで検知する）。
        const ASSUMED_CHAR_WIDTH_PX: f64 = 7.5;
        let longest_label_chars = "control".len() as f64;
        let side_budget = AXIS_LABEL_MARGIN - AXIS_LABEL_OFFSET;
        assert!(
            side_budget >= longest_label_chars * ASSUMED_CHAR_WIDTH_PX,
            "side_budget={side_budget} は最長ラベル想定幅 {}px を下回ってはならない",
            longest_label_chars * ASSUMED_CHAR_WIDTH_PX
        );
    }

    #[test]
    fn vertex_golden_coordinates_for_square_axes() {
        // n=4: 12 時・3 時・6 時・9 時方向の単位円上の座標を手計算で固定する
        // （モジュール doc「頂点角度」節の式の golden 検証）。
        let (x0, y0) = vertex(0.0, 0.0, 1.0, 0, 4);
        assert!((x0 - 0.0).abs() < 1e-9);
        assert!((y0 - (-1.0)).abs() < 1e-9);

        let (x1, y1) = vertex(0.0, 0.0, 1.0, 1, 4);
        assert!((x1 - 1.0).abs() < 1e-9);
        assert!((y1 - 0.0).abs() < 1e-9);

        let (x2, y2) = vertex(0.0, 0.0, 1.0, 2, 4);
        assert!((x2 - 0.0).abs() < 1e-9);
        assert!((y2 - 1.0).abs() < 1e-9);

        let (x3, y3) = vertex(0.0, 0.0, 1.0, 3, 4);
        assert!((x3 - (-1.0)).abs() < 1e-9);
        assert!((y3 - 0.0).abs() < 1e-9);
    }

    #[test]
    fn vertex_is_deterministic_for_n_3_5_6() {
        for n in [3usize, 5, 6] {
            for i in 0..n {
                let a = vertex(10.0, 20.0, 5.0, i, n);
                let b = vertex(10.0, 20.0, 5.0, i, n);
                assert_eq!(a, b);
            }
        }
    }

    #[test]
    fn root_renders_expected_part_counts() {
        let data = sample_data(5);
        let html = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        assert_eq!(html.matches(r#"data-part="spoke""#).count(), 5);
        assert_eq!(html.matches(r#"data-part="axis-label""#).count(), 5);
        assert_eq!(html.matches(r#"data-part="series""#).count(), 1);
        assert!(html.matches(r#"data-part="grid""#).count() >= 1);
    }

    #[test]
    fn axis_label_dominant_baseline_avoids_plot_overlap() {
        // n=4: i=0 は 12 時（上、`sin(θ) = -1`）、i=1 は 3 時（右、`sin(θ) = 0`）、
        // i=2 は 6 時（下、`sin(θ) = 1`）、i=3 は 9 時（左、`sin(θ) = 0`）。
        // 下側ラベルのみプロット外側（下方向）へ伸びる `hanging` を持つことを
        // 固定する（Cursor Bugbot 指摘「Bottom radar labels overlap plot」の
        // 回帰、イシュー #851 追補）。
        let data = sample_data(4);
        let html = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        let labels: Vec<&str> = html
            .split(r#"data-part="axis-label""#)
            .skip(1)
            .map(|rest| rest.split('>').next().unwrap_or(""))
            .collect();
        assert_eq!(labels.len(), 4);
        assert!(labels[0].contains(r#"dominant-baseline="auto""#));
        assert!(labels[1].contains(r#"dominant-baseline="middle""#));
        assert!(labels[2].contains(r#"dominant-baseline="hanging""#));
        assert!(labels[3].contains(r#"dominant-baseline="middle""#));
    }

    #[test]
    fn root_renders_role_img_and_aria_label() {
        let data = sample_data(4);
        let html = render(&root(&data, RadarChartProps::default(), "radar demo").unwrap());
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="radar demo""#));
        assert!(html.contains(r#"data-scope="radar-chart" data-part="root""#));
    }

    #[test]
    fn root_handles_all_zero_values_domain() {
        let data = ChartData::new(
            vec!["a".into(), "b".into(), "c".into()],
            vec![Series::new("s1", vec![0.0, 0.0, 0.0])],
        )
        .unwrap();
        let html = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        assert!(html.contains(r#"data-part="series""#));
    }

    #[test]
    fn root_is_deterministic() {
        let data = sample_data(6);
        let a = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        let b = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn root_escapes_category_series_name_and_aria_label() {
        let data = ChartData::new(
            vec![
                "<script>alert(1)</script>".to_string(),
                "b".to_string(),
                "c".to_string(),
            ],
            vec![Series::new(
                "<img src=x onerror=alert(1)>",
                vec![1.0, 2.0, 3.0],
            )],
        )
        .unwrap();
        let html =
            render(&root(&data, RadarChartProps::default(), "<script>xss</script>").unwrap());
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn css_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="radar-chart"][data-part="grid"]"#));
        assert!(a.contains(r#"[data-scope="radar-chart"][data-part="series"]"#));
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let css = css();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn recipe_includes_issue_1597_corrections() {
        // イシュー #1597: series の輪郭（兄弟部品との整合）・axis-label の
        // font-family（charts::axis とのトークン整合）が実出力に含まれる
        // ことを確認する（モジュール doc「参考サイト基準への調整」節参照）。
        let css = css();
        assert!(css.contains("stroke-width: 2"));
        assert!(css.contains("stroke-linejoin: round"));
        assert!(css.contains("font-family: var(--fandhe-font-font-body)"));
    }
}
