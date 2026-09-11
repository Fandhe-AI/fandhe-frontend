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
//!    角度→座標変換は private ヘルパ `vertex` に一元化し、純 f64 算術
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
//!    `AXIS_LABEL_MARGIN` を差し引いた半径を `plot_radius` とする。
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
//! マークアップはすべて [`super::svg`]/`fandhe_frontend_core::el` 経由で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。カテゴリ名（軸ラベル）・
//! 系列名・`aria_label` はテキストノード/属性値として
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る。座標・半径・
//! `d` 属性はすべて [`ChartData::new`](super::data::ChartData::new)/
//! [`LinearScale::new`] が有限性検証済みの `f64` のみを
//! [`super::svg::fmt_coord`]/[`super::svg::PathBuilder`] へ渡すため、
//! 文字列注入経路を持たない。系列値そのものの数値表示（`axis-value`
//! tspan・`radius-label`）は同じく有限性検証済みの `f64` を
//! [`super::svg::fmt_value`] へ渡す（イシュー #2085 追補: `fmt_coord`
//! 固定小数第 2 位丸めの流用で小さい系列値が `"0"` へ収縮する codex-review
//! 指摘の是正。出力文字集合は `fmt_coord` と同じ `[0-9.-]` に閉じるため
//! 文字列注入経路は増えない）。
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
//!   （`polygon_d` が生成する閉多角形）は UA 既定の `stroke-width: 1` /
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
//!
//! # shadcn/ui Charts（radar）突合（イシュー #2085）
//!
//! shadcn/ui Charts（radar）の registry 14 種を突合し、静的に描画できる
//! 欠落バリアントを [`RadarChartProps`] の純追加で補完した（golden 純追加
//! 原則、`docs/design/shadcn-reference-adoption-policy.md` §8 規則 3）。
//! 既定値では本イシュー以前と**バイト同一の HTML**を出力する。
//!
//! | registry | shadcn の構成 | 本実装での対応 |
//! |---|---|---|
//! | `chart-radar-default` | `PolarGrid`（多角形）+ `PolarAngleAxis` + `Radar fillOpacity 0.6` | 既定（変更なし。`fill-opacity` は 0.2 を維持） |
//! | `chart-radar-dots` | `Radar dot={{ r: 4, fillOpacity: 1 }}` | `dots: true`（新 part `point`、半径 4） |
//! | `chart-radar-lines-only` | `PolarGrid radialLines={false}` + `Radar fillOpacity 0` | `fill: RadarFill::None` + `spokes: false` |
//! | `chart-radar-label-custom` | `PolarAngleAxis tick` カスタム（値 + カテゴリ名の 2 行） | `axis_label: RadarAxisLabel::ValueAndCategory`（`tspan` 2 行） |
//! | `chart-radar-grid-custom` | `PolarGrid radialLines={false} polarRadius={[90]}`（外周 1 本のみ） | `grid_rings: RadarGridRings::Outer` + `spokes: false` |
//! | `chart-radar-grid-fill` | `PolarGrid` を系列色で塗る | `grid_fill: RadarGridFill::Series` |
//! | `chart-radar-grid-none` | `PolarGrid` なし + dots | `grid: RadarGrid::None` + `dots: true` |
//! | `chart-radar-grid-circle` | `PolarGrid gridType="circle"` + dots | `grid: RadarGrid::Circle` + `dots: true` |
//! | `chart-radar-grid-circle-no-lines` | 同上 + `radialLines={false}` | `grid: RadarGrid::Circle` + `spokes: false` + `dots: true` |
//! | `chart-radar-grid-circle-fill` | `gridType="circle"` + 系列色塗り | `grid: RadarGrid::Circle` + `grid_fill: RadarGridFill::Series` |
//! | `chart-radar-multiple` | `Radar` ×2 | 既存対応（複数系列は変更なし） |
//! | `chart-radar-legend` | `Radar` ×2 + `ChartLegend` | radar 側変更なし。[`super::legend::legend`] を並べて合成する |
//! | `chart-radar-icons` | 同上 + `ChartConfig.icon` | radar 側変更なし。[`super::data::Series::with_icon`] + `legend` で合成する |
//! | `chart-radar-radius` | `PolarRadiusAxis angle={60} orientation="middle" axisLine={false}` | `radius_axis: true`（新 part `radius-label`、軸 0/1 中間角の静的近似） |
//!
//! ## 意図的に合わせなかった点
//!
//! - `fill-opacity` は shadcn の `0.6` ではなく既存の `0.2` を維持した
//!   （既存 golden の色味変更禁止・`super::area_chart` との統一）。
//! - `tickFormatter`・数値の書式（3 桁区切り等）はアプリ側整形の責務
//!   （`docs/policy/intentional-non-adoption.md` §3.23/§3.25）。
//! - グリッド外周 1 本（shadcn `polarRadius={[90]}`）の px 指定は非対応。
//!   [`RadarGridRings::Outer`] は `plot_radius` 固定の静的近似。
//! - 半径軸の角度指定（shadcn `angle` prop）は非対応。
//!   [`RadarChartProps::radius_axis`] は軸 0/1 中間角固定。
//! - `ChartTooltip`（indicator line / hideLabel 等）は静的表現が #2086、
//!   マウス追従の JS 配線は #2130、hit-area `data-*` は #2129 の担当。
//!   hover 強調（active 拡張・非 active 減光）の CSS 語彙
//!   （`root[data-has-active]`/`point[data-index]`/`point[data-active]`）
//!   は #2131 で追加済み。
//! - `ChartLegend`/icon は radar 部品へ内包せず [`super::legend`] との
//!   合成で表現する（chakra 方式、イシュー #2077）。凡例の系列トグルは
//!   #2132 の担当。
//! - shadcn の `margin` 調整（legend/icons 用の負マージン）は非対応。
//!   `AXIS_LABEL_MARGIN` は固定のまま。
//! - dots の半径は shadcn `dot.r`（4.0）を採用する（新規追加分のため
//!   参照値をそのまま使える。兄弟 `crate::line_chart::POINT_RADIUS`
//!   （2.5）は既存 golden 固定値であり揃えない）。一方 `point` の背景色
//!   ハロー（`stroke: var(--fandhe-color-bg)`）は line-chart `point` の
//!   先例（dark 時の隣接系列との識別性）を維持する。

use std::f64::consts::PI;

use super::data::ChartData;
use super::pie;
use super::scale::LinearScale;
use super::svg::{self, fmt_coord, fmt_value, svg_text, PathBuilder, ViewBox};
use super::{tooltip, ChartError};
use crate::css::decl;
use crate::recipe::{
    transition_declarations, MotionDuration, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// `data-scope="radar-chart"` の part 一覧（recipe と揃える）。イシュー
/// #2085 で `point`（dots）・`axis-value`（値付き軸ラベル）・`radius-label`
/// （半径軸目盛）を末尾へ純追加した。
const SLOTS: &[&str] = &[
    "root",
    "grid",
    "spoke",
    "axis-label",
    "series",
    "point",
    "axis-value",
    "radius-label",
];

/// [`RadarChartProps::dots`] 有効時の点マーカー半径（shadcn `chart-radar-dots`
/// の `dot.r` に合わせる。イシュー #2085 の新規追加分のため参照値をそのまま
/// 採用できる。兄弟 [`crate::line_chart::POINT_RADIUS`] とは値が異なる、
/// モジュール doc「意図的に合わせなかった点」参照）。
const DOT_RADIUS: f64 = 4.0;

/// [`RadarAxisLabel::ValueAndCategory`] の `tspan` 行間オフセット（下側
/// ラベルの 2 行目・上側ラベルの値行に使う、`em` 単位固定文字列）。
const AXIS_LABEL_LINE_DOWN: &str = "1.2em";
/// 上側ラベルの値行（1 行目、基準線より上方向）に使う `dy`。
const AXIS_LABEL_LINE_UP: &str = "-1.2em";
/// 水平軸（`middle` baseline）の値行に使う `dy`（カテゴリ行との重なりを
/// 避ける半行分のオフセット）。
const AXIS_LABEL_LINE_HALF_UP: &str = "-0.6em";

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

/// 同心グリッドの形状（イシュー #2085、shadcn `PolarGrid gridType`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarGrid {
    /// 正多角形のグリッド（既定。`chart-radar-default`）。
    #[default]
    Polygon,
    /// 正円のグリッド（`chart-radar-grid-circle`）。
    Circle,
    /// グリッドを描画しない（`chart-radar-grid-none`）。
    None,
}

/// グリッドの同心リング本数（イシュー #2085、shadcn `PolarGrid polarRadius`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarGridRings {
    /// [`LinearScale::ticks`] の 0 超 tick ごとに 1 本ずつ描く（既定）。
    #[default]
    Ticks,
    /// 外周（`plot_radius`）の 1 本のみ描く（`chart-radar-grid-custom` の
    /// 静的近似。shadcn `polarRadius={[90]}` の px 直接指定は非対応、
    /// モジュール doc「意図的に合わせなかった点」参照）。
    Outer,
}

/// グリッドの塗り（イシュー #2085、shadcn `chart-radar-grid-fill` 系）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarGridFill {
    /// 塗りなし（既定）。
    #[default]
    None,
    /// 先頭系列色でグリッドを塗る（`fill-opacity: 0.2`）。同心グリッドの
    /// 最外周リング 1 枚にのみ適用し、内側のリングは輪郭線のみを描く
    /// （イシュー #2085 追補、Cursor Bugbot 指摘: 各リングは中心からの
    /// 塗りつぶし円盤/多角形であり全リングへ適用すると内側ほど合成
    /// 不透明度が重なって意図した一様なウォッシュを超えてしまうため）。
    Series,
}

impl VariantValue for RadarGridFill {
    fn axis(self) -> &'static str {
        "grid-fill"
    }

    fn value(self) -> &'static str {
        match self {
            RadarGridFill::None => "none",
            RadarGridFill::Series => "series",
        }
    }
}

/// 系列ポリゴンの塗り（イシュー #2085、shadcn `chart-radar-lines-only`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarFill {
    /// 系列色で半透明に塗る（既定、`fill-opacity: 0.2`）。
    #[default]
    Solid,
    /// 塗りなし（輪郭のみ）。
    None,
}

/// 軸ラベルの内容（イシュー #2085、shadcn `chart-radar-label-custom`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarAxisLabel {
    /// カテゴリ名のみ（既定、従来どおり）。
    #[default]
    Category,
    /// 各系列の値（`/` 区切り）+ カテゴリ名の 2 行。
    ValueAndCategory,
}

/// [`root`] の描画パラメータ。
///
/// イシュー #2133 で `range`/`hidden_series`（`String`/`Vec<String>`）を
/// 純追加したため `Copy` は外れ `Clone` のみになった（0.x の破壊的変更）。
#[derive(Debug, Clone, PartialEq)]
pub struct RadarChartProps {
    /// `viewBox` の一辺の長さ（正方形、px 相当。既定 300.0）。
    pub size: f64,
    /// 同心グリッドの形状（イシュー #2085、既定 [`RadarGrid::Polygon`]）。
    pub grid: RadarGrid,
    /// グリッドの同心リング本数（イシュー #2085、既定 [`RadarGridRings::Ticks`]）。
    pub grid_rings: RadarGridRings,
    /// グリッドの塗り（イシュー #2085、既定 [`RadarGridFill::None`]）。
    pub grid_fill: RadarGridFill,
    /// スポーク（中心 → 各軸頂点の線）を描画するか（イシュー #2085、既定 `true`）。
    pub spokes: bool,
    /// 系列ポリゴンの塗り（イシュー #2085、既定 [`RadarFill::Solid`]）。
    pub fill: RadarFill,
    /// データ点マーカーを描画するか（イシュー #2085、既定 `false`）。
    pub dots: bool,
    /// 軸ラベルの内容（イシュー #2085、既定 [`RadarAxisLabel::Category`]）。
    pub axis_label: RadarAxisLabel,
    /// 半径軸（値目盛ラベル）を描画するか（イシュー #2085、既定 `false`）。
    pub radius_axis: bool,
    /// `true`（既定）なら hit-area・`data-index` と `hidden` の SSR
    /// ツールチップ DOM（[`super::tooltip::layer`]）を出力する（イシュー
    /// #2129、親 #2128。既存 `series` の `data-series` は不変、hit-area
    /// 自体には付与しない）。`false` の場合は本イシュー以前の出力と
    /// バイト一致する。
    pub show_tooltip: bool,
    /// 表示範囲の不透明な識別子（イシュー #2133、親 #2132）。`Some(v)` の
    /// とき root（`svg[data-part="root"]`）へ `data-range="<v>"` を出力
    /// する（既定 `None`＝非出力）。
    pub range: Option<String>,
    /// 非表示系列名の一覧（イシュー #2133）。系列名と完全一致する
    /// `series`/`point` へ値なし属性 `data-hidden` を付与する。データに
    /// 存在しない名前を指定してもエラーにしない（fail-soft）。
    pub hidden_series: Vec<String>,
}

impl Default for RadarChartProps {
    /// 既定値は #2085 以前の出力と完全に同一の HTML を生成する（golden
    /// 純追加原則）。
    fn default() -> Self {
        RadarChartProps {
            size: 300.0,
            grid: RadarGrid::default(),
            grid_rings: RadarGridRings::default(),
            grid_fill: RadarGridFill::default(),
            spokes: true,
            fill: RadarFill::default(),
            dots: false,
            axis_label: RadarAxisLabel::default(),
            radius_axis: false,
            show_tooltip: true,
            range: None,
            hidden_series: Vec::new(),
        }
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
        // イシュー #2085: dots（shadcn `chart-radar-dots`）の点マーカー。
        // line-chart の point base（`crate::line_chart::recipe`）と同じ背景色
        // ハローで dark 時の隣接系列との識別性を確保する。
        .base(
            "point",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        // イシュー #2085: `RadarAxisLabel::ValueAndCategory` の値行
        // （shadcn `fontWeight 500` 相当）。
        .base(
            "axis-value",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        // イシュー #2085: `radius_axis: true`（shadcn `PolarRadiusAxis`）の
        // 半径軸目盛ラベル。
        .base(
            "radius-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("fill", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #2085: `grid_fill: RadarGridFill::Series`（shadcn
        // `chart-radar-grid-fill`）。`grid` base の `fill: none` presentation
        // 属性より CSS が勝つため、インライン `fill` ではなく variant class
        // + `color` 属性 + `currentColor` で表現する（line_chart
        // `LineDots::Hollow` と同じ手法）。`default_variant` は登録しない
        // （登録すると全 radar-chart の grid class に無条件混入し HTML
        // golden が壊れる、area_chart `AreaFill::Gradient` と同じ判断）。
        .variant(
            RadarGridFill::Series,
            "grid",
            vec![decl("fill", "currentColor"), decl("fill-opacity", "0.2")],
        )
        // イシュー #2133: `hidden_series` で指定した系列の描画要素を
        // 非表示にする（末尾純追加、既存ブロックは不変）。
        .state(
            "series",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "point",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #2131: hover 強調（減光）の消費側。祖先の
        // `chart::tooltip::frame`（`<svg>` の親）が `data-has-active` を
        // 持つときに継承する `--fandhe-chart-inactive-opacity` を消費する
        // （`crate::charts::tooltip` モジュール doc「hover 強調」節参照。
        // wasm-full 側の付け外し配線は未実装のフォローアップ）。
        .state("point", StateCondition::Attr("data-index"), {
            let mut decls = vec![decl("opacity", "var(--fandhe-chart-inactive-opacity, 1)")];
            decls.extend(transition_declarations("opacity", MotionDuration::Fast));
            decls
        })
        // イシュー #2131: active な点は上記の減光を上書きしフル不透明へ
        // 戻し、系列色との識別性向上のため拡大する（`SlotRecipe::css` の
        // states 出力順契約により `[data-index]` 規則より後で上書き。
        // `chart::tooltip::datum:hover` 是正〔#1593〕と同じ色・線幅）。
        .state(
            "point",
            StateCondition::Attr("data-active"),
            vec![
                decl("opacity", "1"),
                decl("transform-box", "fill-box"),
                decl("transform-origin", "center"),
                decl("transform", "scale(1.5)"),
                decl("stroke", "var(--fandhe-color-fg)"),
                decl("stroke-width", "2"),
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
    let recipe = recipe();

    // グリッド（同心正多角形/正円）。`props.grid == RadarGrid::None` では
    // 一切描画しない（イシュー #2085、shadcn `chart-radar-grid-none`）。
    if props.grid != RadarGrid::None {
        let ring_radii: Vec<f64> = match props.grid_rings {
            // tick 0 は中心の 1 点に潰れ描画上意味を持たないため除外する
            // （#851 以来の既存挙動）。
            RadarGridRings::Ticks => value_scale
                .ticks(GRID_TICK_TARGET)?
                .into_iter()
                .filter(|t| *t > 0.0)
                .map(|tick| value_scale.scale(tick))
                .collect(),
            // shadcn `polarRadius={[90]}`（外周 1 本のみ）の静的近似。px
            // 直接指定は非対応、モジュール doc「意図的に合わせなかった点」
            // 参照。
            RadarGridRings::Outer => vec![plot_radius],
        };
        let grid_fill_class;
        let grid_fill_color;
        let grid_attrs_extra: Vec<(&str, &str)> = if props.grid_fill == RadarGridFill::Series {
            grid_fill_class = recipe.variant_class(RadarGridFill::Series);
            grid_fill_color = data.series_color_var(0);
            vec![
                ("class", grid_fill_class.as_str()),
                ("color", grid_fill_color.as_str()),
            ]
        } else {
            vec![]
        };
        // イシュー #2085 追補（Cursor Bugbot 指摘）: 各リングは中心から
        // 半径 r までの塗りつぶし形状（同心「環」ではなく同心「円盤/多角形」）
        // であるため、全リングへ一律 fill-opacity 0.2 を適用すると内側ほど
        // 塗りが重なり合成不透明度が意図（0.2 の一様なウォッシュ）を大きく
        // 超えてしまう。塗りは最外周リング 1 枚のみに適用し、内側のリングは
        // 輪郭線（`grid` base の stroke）のみを描く。`ring_radii` は
        // `RadarGridRings::Ticks`（`ticks()` の昇順出力を `value_scale.scale`
        // で写像、`scale` は単調増加）/`RadarGridRings::Outer`（要素 1 件）の
        // いずれも昇順であることを前提に、最終要素（インデックス最大）を
        // 最外周と判定する。
        let ring_count = ring_radii.len();
        for (idx, r) in ring_radii.into_iter().enumerate() {
            let is_outermost = idx + 1 == ring_count;
            let mut attrs: Vec<(&str, &str)> = vec![("data-scope", "radar-chart")];
            let node = match props.grid {
                RadarGrid::Polygon => {
                    let d = polygon_d(center, center, r, n);
                    attrs.push(("data-part", "grid"));
                    if is_outermost {
                        attrs.extend(grid_attrs_extra.iter().copied());
                    }
                    let mut path_attrs = attrs;
                    path_attrs.push(("d", d.as_str()));
                    el("path", path_attrs, vec![])
                }
                RadarGrid::Circle => {
                    attrs.push(("data-part", "grid"));
                    if is_outermost {
                        attrs.extend(grid_attrs_extra.iter().copied());
                    }
                    svg::circle(center, center, r, attrs)
                }
                RadarGrid::None => unreachable!("外側の if で RadarGrid::None を除外済み"),
            };
            children.push(node);
        }
    }

    // スポーク（中心 → 各軸の外周頂点）。`props.spokes == false` では描画
    // しない（イシュー #2085、shadcn `radialLines={false}`）。
    if props.spokes {
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
        let text_attrs = vec![
            ("data-scope", "radar-chart"),
            ("data-part", "axis-label"),
            ("text-anchor", anchor),
            ("dominant-baseline", baseline),
        ];
        match props.axis_label {
            RadarAxisLabel::Category => {
                children.push(svg_text(x, y, text_attrs, vec![text(category.as_str())]));
            }
            RadarAxisLabel::ValueAndCategory => {
                // shadcn `chart-radar-label-custom` の静的近似: 1 行目に
                // 各系列の当該軸の値を `/` 区切りで、2 行目にカテゴリ名を
                // 表示する。値行は `data-part="axis-value"` を持つ `tspan`
                // （font-weight を分ける）、カテゴリ行は `SLOTS` 未登録の
                // 素の `tspan`（親 axis-label の書式を継承、モジュール
                // 未登録 part を出力しない契約）。値は fmt_value（データ値
                // 用、イシュー #2085 追補）のみで文字列化し `/` で連結する
                // （文字集合 [0-9.-/] に閉じる）。座標用 fmt_coord の固定
                // 小数第 2 位丸めを流用すると小さい系列値（例:
                // [0.001, 0.002, 0.003]）が一律 "0" に潰れる不具合が
                // あった（codex-review 指摘）。
                let values: Vec<String> = data
                    .series()
                    .iter()
                    .map(|s| fmt_value(s.values[i]))
                    .collect();
                let value_line = values.join("/");
                let (value_dy, category_dy) = if theta.sin() > ANCHOR_EPSILON {
                    // 下側: 値行が基準線、カテゴリ行がその下（外側）。
                    (None, Some(AXIS_LABEL_LINE_DOWN))
                } else if theta.sin() < -ANCHOR_EPSILON {
                    // 上側: 値行が基準線より上、カテゴリ行が基準線に乗る。
                    (Some(AXIS_LABEL_LINE_UP), Some(AXIS_LABEL_LINE_DOWN))
                } else {
                    // 水平: 値行を半行上へ、カテゴリ行を半行下へ。
                    (Some(AXIS_LABEL_LINE_HALF_UP), Some(AXIS_LABEL_LINE_DOWN))
                };
                let x_str = fmt_coord(x);
                let mut value_attrs: Vec<(&str, &str)> = vec![
                    ("data-scope", "radar-chart"),
                    ("data-part", "axis-value"),
                    ("x", x_str.as_str()),
                ];
                if let Some(dy) = value_dy {
                    value_attrs.push(("dy", dy));
                }
                let value_tspan = el("tspan", value_attrs, vec![text(value_line)]);

                let mut category_attrs: Vec<(&str, &str)> = vec![("x", x_str.as_str())];
                if let Some(dy) = category_dy {
                    category_attrs.push(("dy", dy));
                }
                let category_tspan = el("tspan", category_attrs, vec![text(category.as_str())]);

                children.push(svg_text(
                    x,
                    y,
                    text_attrs,
                    vec![value_tspan, category_tspan],
                ));
            }
        }
    }

    // 系列ポリゴン + dots。
    for (series_idx, series) in data.series().iter().enumerate() {
        let color = data.series_color_var(series_idx);
        let mut points: Vec<(f64, f64)> = Vec::with_capacity(n);
        let mut builder = PathBuilder::new();
        for (i, &value) in series.values.iter().enumerate() {
            let r = value_scale.scale(value);
            let (x, y) = vertex(center, center, r, i, n);
            points.push((x, y));
            builder = if i == 0 {
                builder.move_to(x, y)
            } else {
                builder.line_to(x, y)
            };
        }
        let d = builder.close().build();
        let fill_attr = if props.fill == RadarFill::None {
            "none"
        } else {
            color.as_str()
        };
        let hidden = props.hidden_series.contains(&series.name);
        let mut series_attrs: Vec<(&str, &str)> = vec![
            ("data-scope", "radar-chart"),
            ("data-part", "series"),
            ("data-series", series.name.as_str()),
            ("d", d.as_str()),
            ("fill", fill_attr),
            ("stroke", color.as_str()),
        ];
        if hidden {
            series_attrs.push(("data-hidden", ""));
        }
        children.push(el("path", series_attrs, vec![]));

        if props.dots {
            for (point_idx, &(x, y)) in points.iter().enumerate() {
                let point_idx_str = point_idx.to_string();
                let mut point_attrs: Vec<(&str, &str)> =
                    vec![("data-scope", "radar-chart"), ("data-part", "point")];
                if props.show_tooltip {
                    // hit-area・`data-index` と同じゲート（scatter の
                    // point と同型、Cursor Bugbot 指摘「Hidden satellites
                    // lack shared identifiers」対応、イシュー #2133）。
                    point_attrs.push(("data-index", point_idx_str.as_str()));
                }
                // `series`（path）の `data-series` と同じく既存語彙、
                // `show_tooltip` に関わらず常に付与する。
                point_attrs.push(("data-series", series.name.as_str()));
                point_attrs.push(("fill", color.as_str()));
                if hidden {
                    point_attrs.push(("data-hidden", ""));
                }
                children.push(svg::circle(x, y, DOT_RADIUS, point_attrs));
            }
        }
    }

    // 半径軸（値目盛ラベル、shadcn `PolarRadiusAxis` の静的近似）。角度は
    // 軸 0 と軸 1 の中間（n=6 で shadcn `angle=60` と同じ右上方向）に固定
    // する。`angle` prop の任意指定は非対応（モジュール doc参照）。
    if props.radius_axis {
        let radius_theta = vertex_angle(0, n) + PI / (n as f64);
        for tick in value_scale
            .ticks(GRID_TICK_TARGET)?
            .into_iter()
            .filter(|t| *t > 0.0)
        {
            let r = value_scale.scale(tick);
            let x = center + r * radius_theta.cos();
            let y = center + r * radius_theta.sin();
            children.push(svg_text(
                x,
                y,
                vec![
                    ("data-scope", "radar-chart"),
                    ("data-part", "radius-label"),
                    ("text-anchor", "middle"),
                    ("dominant-baseline", "middle"),
                ],
                // `tick` は半径写像前のデータ値（ピクセル座標ではない）
                // のため、値表示用 fmt_value を使う（イシュー #2085 追補、
                // fmt_coord 流用による小さい値の "0" 収縮の是正）。
                vec![text(fmt_value(tick))],
            ));
        }
    }

    // イシュー #2129: hit-area・SSR ツールチップ DOM。各軸（カテゴリ）の
    // 頂点角 ± 半ステップの扇形を hit-area とする（`pie::sector_path` を
    // `plot_radius` で再利用、§2.7「radar」行）。既存 `series` の
    // `data-series` は不変、hit-area 自体には付与しない。
    let entries = if props.show_tooltip {
        Some(tooltip::entries_from_chart_data(data))
    } else {
        None
    };
    if let Some(entries) = &entries {
        let half_step = PI / n as f64;
        for entry in entries {
            let theta = vertex_angle(entry.index, n);
            let d = pie::sector_path(
                center,
                center,
                plot_radius,
                theta - half_step,
                theta + half_step,
            );
            let label = tooltip::hit_area_label(entry);
            children.push(tooltip::hit_area_path(&d, entry.index, None, &label, false));
        }
    }

    let mut root_attrs: Vec<(&str, &str)> = vec![
        ("data-scope", "radar-chart"),
        ("data-part", "root"),
        ("aria-label", aria_label),
    ];
    if let Some(range) = &props.range {
        root_attrs.push(("data-range", range.as_str()));
    }
    let root_node = svg::svg_root(&view_box, root_attrs, children);

    match entries {
        Some(entries) => Ok(tooltip::frame(vec![
            root_node,
            tooltip::layer_from_entries(&entries, None),
        ])),
        None => Ok(root_node),
    }
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
            root(
                &data,
                RadarChartProps {
                    size: 0.0,
                    ..RadarChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            root(
                &data,
                RadarChartProps {
                    size: f64::NAN,
                    ..RadarChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_plot_area_too_small() {
        let data = sample_data(4);
        // AXIS_LABEL_MARGIN (60.0) * 2 = 120.0 以下では plot_radius <= 0。
        assert_eq!(
            root(
                &data,
                RadarChartProps {
                    size: 60.0,
                    ..RadarChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
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

    /// 系列の [`crate::charts::data::SeriesColor`] 上書き（イシュー #2077）が
    /// ポリゴンの `stroke`/`fill` へ反映されることを固定する。
    #[test]
    fn root_reflects_series_color_override() {
        let categories: Vec<String> = (0..3).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![Series::new("s1", vec![10.0, 20.0, 30.0])
                .with_color(crate::charts::SeriesColor::token("info").unwrap())],
        )
        .unwrap();
        let html = render(&root(&data, RadarChartProps::default(), "label").unwrap());
        assert!(html.contains("var(--fandhe-color-info)"));
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

    // --- イシュー #2085: shadcn/ui Charts（radar）突合 ---

    /// #2085 着手前（`origin/main`）の `RadarChartProps::default()` 出力を
    /// バイトそのまま埋め込んだ golden fixture。新規 props はすべて既定値で
    /// #2085 以前の出力と完全に同一の HTML を生成する（golden 純追加原則、
    /// PR 本文の「純追加」主張の根拠）。
    const PRE_2085_SAMPLE5_HTML: &str = concat!(
        r#"<svg viewBox="0 0 300 300" role="img" data-scope="radar-chart" data-part="root" aria-label="sample5">"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,132 L167.12,144.44 L160.58,164.56 L139.42,164.56 L132.88,144.44 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,114 L184.24,138.88 L171.16,179.12 L128.84,179.12 L115.76,138.88 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,96 L201.36,133.31 L181.74,193.69 L118.26,193.69 L98.64,133.31 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,78 L218.48,127.75 L192.32,208.25 L107.68,208.25 L81.52,127.75 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,60 L235.6,122.19 L202.9,222.81 L97.1,222.81 L64.4,122.19 Z"></path>"#,
        r#"<line x1="150" y1="150" x2="150" y2="60" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="235.6" y2="122.19" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="202.9" y2="222.81" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="97.1" y2="222.81" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="64.4" y2="122.19" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<text x="150" y="54" data-scope="radar-chart" data-part="axis-label" text-anchor="middle" dominant-baseline="auto">axis0</text>"#,
        r#"<text x="241.3" y="120.33" data-scope="radar-chart" data-part="axis-label" text-anchor="start" dominant-baseline="auto">axis1</text>"#,
        r#"<text x="206.43" y="227.67" data-scope="radar-chart" data-part="axis-label" text-anchor="start" dominant-baseline="hanging">axis2</text>"#,
        r#"<text x="93.57" y="227.67" data-scope="radar-chart" data-part="axis-label" text-anchor="end" dominant-baseline="hanging">axis3</text>"#,
        r#"<text x="58.7" y="120.33" data-scope="radar-chart" data-part="axis-label" text-anchor="end" dominant-baseline="auto">axis4</text>"#,
        r#"<path data-scope="radar-chart" data-part="series" data-series="s1" d="M150,132 L184.24,138.88 L181.74,193.69 L107.68,208.25 L64.4,122.19 Z" fill="var(--fandhe-color-chart-1)" stroke="var(--fandhe-color-chart-1)"></path>"#,
        r#"</svg>"#,
    );

    /// showcase 相当の 2 系列 5 軸データの golden fixture（上記と同じ目的）。
    const PRE_2085_SHOWCASE_HTML: &str = concat!(
        r#"<svg viewBox="0 0 300 300" role="img" data-scope="radar-chart" data-part="root" aria-label="showcase demo">"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,130 L169.02,143.82 L161.76,166.18 L138.24,166.18 L130.98,143.82 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,110 L188.04,137.64 L173.51,182.36 L126.49,182.36 L111.96,137.64 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,90 L207.06,131.46 L185.27,198.54 L114.73,198.54 L92.94,131.46 Z"></path>"#,
        r#"<path data-scope="radar-chart" data-part="grid" d="M150,70 L226.08,125.28 L197.02,214.72 L102.98,214.72 L73.92,125.28 Z"></path>"#,
        r#"<line x1="150" y1="150" x2="150" y2="60" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="235.6" y2="122.19" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="202.9" y2="222.81" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="97.1" y2="222.81" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<line x1="150" y1="150" x2="64.4" y2="122.19" data-scope="radar-chart" data-part="spoke"></line>"#,
        r#"<text x="150" y="54" data-scope="radar-chart" data-part="axis-label" text-anchor="middle" dominant-baseline="auto">speed</text>"#,
        r#"<text x="241.3" y="120.33" data-scope="radar-chart" data-part="axis-label" text-anchor="start" dominant-baseline="auto">power</text>"#,
        r#"<text x="206.43" y="227.67" data-scope="radar-chart" data-part="axis-label" text-anchor="start" dominant-baseline="hanging">range</text>"#,
        r#"<text x="93.57" y="227.67" data-scope="radar-chart" data-part="axis-label" text-anchor="end" dominant-baseline="hanging">control</text>"#,
        r#"<text x="58.7" y="120.33" data-scope="radar-chart" data-part="axis-label" text-anchor="end" dominant-baseline="auto">armor</text>"#,
        r#"<path data-scope="radar-chart" data-part="series" data-series="mercury" d="M150,70 L207.06,131.46 L173.51,182.36 L97.1,222.81 L102.45,134.55 Z" fill="var(--fandhe-color-chart-1)" stroke="var(--fandhe-color-chart-1)"></path>"#,
        r#"<path data-scope="radar-chart" data-part="series" data-series="venus" d="M150,80 L235.6,122.19 L185.27,198.54 L126.49,182.36 L73.92,125.28 Z" fill="var(--fandhe-color-chart-2)" stroke="var(--fandhe-color-chart-2)"></path>"#,
        r#"</svg>"#,
    );

    #[test]
    fn default_props_html_is_byte_identical_to_pre_2085() {
        // イシュー #2129: 既定は `show_tooltip: true` へ変更されたため、
        // 本テストの本来の関心（#2085 以前のジオメトリが不変であること）を
        // 検証するには `show_tooltip: false`（opt-out、本イシュー以前の
        // 出力とバイト一致する契約）で確認する。
        let props = RadarChartProps {
            show_tooltip: false,
            ..RadarChartProps::default()
        };
        let data5 = sample_data(5);
        let html5 = render(&root(&data5, props.clone(), "sample5").unwrap());
        assert_eq!(html5, PRE_2085_SAMPLE5_HTML);

        let data2 = ChartData::new(
            vec![
                "speed".into(),
                "power".into(),
                "range".into(),
                "control".into(),
                "armor".into(),
            ],
            vec![
                Series::new("mercury", vec![80.0, 60.0, 40.0, 90.0, 50.0]),
                Series::new("venus", vec![70.0, 90.0, 60.0, 40.0, 80.0]),
            ],
        )
        .unwrap();
        let html2 = render(&root(&data2, props, "showcase demo").unwrap());
        assert_eq!(html2, PRE_2085_SHOWCASE_HTML);
    }

    #[test]
    fn default_props_output_has_no_new_parts() {
        // イシュー #2129: 同上の理由で `show_tooltip: false`（opt-out）を
        // 使う。hit-area は `fill="none"` を出力するため既定
        // （`show_tooltip: true`）ではこのアサーションが成立しない。
        let props = RadarChartProps {
            show_tooltip: false,
            ..RadarChartProps::default()
        };
        let data = sample_data(5);
        let html = render(&root(&data, props, "label").unwrap());
        assert!(!html.contains(r#"data-part="point""#));
        assert!(!html.contains(r#"data-part="axis-value""#));
        assert!(!html.contains(r#"data-part="radius-label""#));
        assert!(!html.contains("<circle"));
        assert!(!html.contains("class="));
        assert!(!html.contains("<tspan"));
        assert!(!html.contains(r#"fill="none""#));
    }

    #[test]
    fn grid_circle_renders_circle_elements() {
        let data = sample_data(5);
        let props = RadarChartProps {
            grid: RadarGrid::Circle,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        let circle_grid_count = html.matches(r#"<circle"#).count();
        assert!(circle_grid_count >= 1);
        assert_eq!(
            html.matches(r#"<path data-scope="radar-chart" data-part="grid""#)
                .count(),
            0
        );
    }

    #[test]
    fn grid_none_renders_no_grid_parts() {
        let data = sample_data(5);
        let props = RadarChartProps {
            grid: RadarGrid::None,
            dots: true,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="grid""#).count(), 0);
        // スポークは既定どおり残る（shadcn `chart-radar-grid-none` も
        // `PolarAngleAxis` の軸線は残る、モジュール doc §2 参照）。
        assert!(html.matches(r#"data-part="spoke""#).count() > 0);
    }

    #[test]
    fn grid_rings_outer_renders_single_ring() {
        let data = sample_data(5);
        let props = RadarChartProps {
            grid_rings: RadarGridRings::Outer,
            spokes: false,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="grid""#).count(), 1);
    }

    #[test]
    fn spokes_false_renders_no_spoke() {
        let data = sample_data(5);
        let props = RadarChartProps {
            spokes: false,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="spoke""#).count(), 0);
    }

    #[test]
    fn grid_fill_series_adds_variant_class_and_color() {
        let data = sample_data(5);
        let props = RadarChartProps {
            grid_fill: RadarGridFill::Series,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(html.contains("fd-radar-chart--grid-fill-series"));
        assert!(html.contains(r#"color="var(--fandhe-color-chart-1)""#));
    }

    #[test]
    fn grid_fill_series_variant_class_applies_to_outermost_ring_only() {
        // イシュー #2085 追補（Cursor Bugbot 指摘）: `grid_fill: Series` を
        // 全リングへ適用すると同心円盤/多角形の重なりで中心部の合成
        // 不透明度が意図した 0.2 のウォッシュを大きく超える。修正後は
        // variant class（塗り適用の目印）を持つ `grid` パーツが最外周
        // 1 枚のみになることを固定する。
        let data = sample_data(5);
        let props = RadarChartProps {
            grid_fill: RadarGridFill::Series,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        let filled_grid_count = html
            .matches(r#"data-part="grid" class="fd-radar-chart--grid-fill-series""#)
            .count();
        assert_eq!(filled_grid_count, 1);
    }

    #[test]
    fn fill_none_emits_fill_none_on_series() {
        let data = sample_data(5);
        let props = RadarChartProps {
            fill: RadarFill::None,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(html.contains(r#"data-part="series" data-series="s1" d="#));
        assert!(html.contains(r#"fill="none""#));
    }

    #[test]
    fn dots_render_one_point_per_vertex_per_series() {
        let n = 5;
        let categories: Vec<String> = (0..n).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![
                Series::new("s1", vec![10.0, 20.0, 30.0, 40.0, 50.0]),
                Series::new("s2", vec![15.0, 25.0, 35.0, 45.0, 55.0]),
            ],
        )
        .unwrap();
        let props = RadarChartProps {
            dots: true,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="point""#).count(), n * 2);
        assert!(html.contains(r#"r="4""#));
    }

    /// Cursor Bugbot 指摘（PR #2271「Hidden satellites lack shared
    /// identifiers」）: `point`（`dots: true`）に `data-hidden` は伝搬する
    /// が、series（path）が既に持つ `data-series` と、hit-area と同じ
    /// 語彙の `data-index` が欠けており、凡例トグルの共有セレクタから
    /// point だけを一緒に非表示・復元できなかった。両属性が付与される
    /// ことを固定する。
    #[test]
    fn dots_carry_data_series_and_data_index() {
        let n = 5;
        let categories: Vec<String> = (0..n).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![Series::new("s1", vec![10.0, 20.0, 30.0, 40.0, 50.0])],
        )
        .unwrap();
        let props = RadarChartProps {
            dots: true,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        let point_tags: Vec<_> = html.match_indices(r#"data-part="point""#).collect();
        assert_eq!(point_tags.len(), n);
        for (point_idx, (idx, _)) in point_tags.iter().enumerate() {
            let tag_end = html[*idx..].find('>').unwrap();
            let tag = &html[*idx..*idx + tag_end];
            assert!(
                tag.contains(&format!(r#"data-index="{point_idx}""#)),
                "point #{point_idx} に data-index が出力されること: {tag}"
            );
            assert!(
                tag.contains(r#"data-series="s1""#),
                "point #{point_idx} に data-series が出力されること: {tag}"
            );
        }
    }

    #[test]
    fn axis_label_value_and_category_renders_two_tspans_with_joined_values() {
        let categories: Vec<String> = (0..4).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![
                Series::new("s1", vec![80.0, 60.0, 40.0, 90.0]),
                Series::new("s2", vec![50.0, 30.0, 20.0, 10.0]),
            ],
        )
        .unwrap();
        let props = RadarChartProps {
            axis_label: RadarAxisLabel::ValueAndCategory,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(html.contains(">80/50<"));
        assert!(html.contains("<tspan"));
        assert!(html.contains(r#"data-part="axis-value""#));
    }

    #[test]
    fn axis_label_value_and_category_preserves_small_series_values() {
        // codex-review 指摘（イシュー #2085 追補）: fmt_coord（座標用、
        // 小数第 2 位丸め）の流用では [0.001, 0.002, 0.003] のような小さい
        // 系列値がすべて "0" に潰れて読み取れなくなる。fmt_value 導入後は
        // 有効数字が残ることを固定する。
        let categories: Vec<String> = (0..3).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![Series::new("s1", vec![0.001, 0.002, 0.003])],
        )
        .unwrap();
        let props = RadarChartProps {
            axis_label: RadarAxisLabel::ValueAndCategory,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(!html.contains(">0/axis0<"));
        assert!(html.contains(">0.001<"));
        assert!(html.contains(">0.002<"));
        assert!(html.contains(">0.003<"));
    }

    #[test]
    fn radius_axis_preserves_small_tick_values() {
        // codex-review 指摘（イシュー #2085 追補）: 半径軸目盛の値表示も
        // 同じ理由で "0" に潰れていた不具合の回帰防止。
        let categories: Vec<String> = (0..3).map(|i| format!("axis{i}")).collect();
        let data = ChartData::new(
            categories,
            vec![Series::new("s1", vec![0.001, 0.002, 0.003])],
        )
        .unwrap();
        let props = RadarChartProps {
            radius_axis: true,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        let radius_label_texts: Vec<&str> = html
            .match_indices(r#"data-part="radius-label""#)
            .map(|(pos, _)| {
                let after = &html[pos..];
                let open_end = after.find('>').expect("radius-label タグは閉じる");
                let rest = &after[open_end + 1..];
                let close = rest.find('<').expect("radius-label はテキストを持つ");
                &rest[..close]
            })
            .collect();
        assert!(!radius_label_texts.is_empty());
        assert!(
            radius_label_texts.iter().all(|t| *t != "0"),
            "radius-label に \"0\" へ収縮した値があってはならない: {radius_label_texts:?}"
        );
    }

    #[test]
    fn radius_axis_renders_one_label_per_positive_tick() {
        let data = sample_data(5);
        let props = RadarChartProps {
            radius_axis: true,
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        let expected = LinearScale::new((0.0, 50.0), (0.0, 90.0))
            .unwrap()
            .nice()
            .ticks(GRID_TICK_TARGET)
            .unwrap()
            .into_iter()
            .filter(|t| *t > 0.0)
            .count();
        assert_eq!(
            html.matches(r#"data-part="radius-label""#).count(),
            expected
        );
    }

    #[test]
    fn variants_are_deterministic() {
        let data = sample_data(6);
        let props = RadarChartProps {
            grid: RadarGrid::Circle,
            grid_rings: RadarGridRings::Outer,
            grid_fill: RadarGridFill::Series,
            spokes: false,
            fill: RadarFill::None,
            dots: true,
            axis_label: RadarAxisLabel::ValueAndCategory,
            radius_axis: true,
            size: 300.0,
            show_tooltip: true,
            range: None,
            hidden_series: Vec::new(),
        };
        let a = render(&root(&data, props.clone(), "label").unwrap());
        let b = render(&root(&data, props, "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn css_golden_prefix_is_unchanged() {
        let css = css();
        assert!(css.starts_with(
            "[data-scope=\"radar-chart\"][data-part=\"root\"] {\n  display: block;\n  max-width: 100%;\n}\n\n"
        ));
    }

    // イシュー #2133: 期間切替・凡例トグルの SSR 構造。

    #[test]
    fn range_none_omits_data_range() {
        let html = render(&root(&sample_data(3), RadarChartProps::default(), "range").unwrap());
        assert!(!html.contains("data-range"));
    }

    #[test]
    fn range_some_emits_data_range_on_root() {
        let props = RadarChartProps {
            range: Some("90d".to_string()),
            ..RadarChartProps::default()
        };
        let html = render(&root(&sample_data(3), props, "range").unwrap());
        assert!(html.contains(r#"data-range="90d""#));
    }

    #[test]
    fn hidden_series_adds_data_hidden_to_matching_series_only() {
        let data = ChartData::new(
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                Series::new("mercury", vec![1.0, 2.0, 3.0]),
                Series::new("venus", vec![4.0, 5.0, 6.0]),
            ],
        )
        .unwrap();
        let props = RadarChartProps {
            hidden_series: vec!["venus".to_string()],
            ..RadarChartProps::default()
        };
        let html = render(&root(&data, props, "hidden").unwrap());
        let venus_idx = html.find(r#"data-series="venus""#).unwrap();
        let venus_tag_end = html[venus_idx..].find('>').unwrap();
        assert!(html[venus_idx..venus_idx + venus_tag_end].contains("data-hidden"));
        let mercury_idx = html.find(r#"data-series="mercury""#).unwrap();
        let mercury_tag_end = html[mercury_idx..].find('>').unwrap();
        assert!(!html[mercury_idx..mercury_idx + mercury_tag_end].contains("data-hidden"));
    }

    #[test]
    fn hidden_series_unknown_name_is_fail_soft() {
        let props = RadarChartProps {
            hidden_series: vec!["does-not-exist".to_string()],
            ..RadarChartProps::default()
        };
        let result = root(&sample_data(3), props, "hidden");
        assert!(result.is_ok());
        assert!(!render(&result.unwrap()).contains("data-hidden"));
    }
}
