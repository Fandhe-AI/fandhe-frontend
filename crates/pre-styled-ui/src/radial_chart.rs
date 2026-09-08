//! styled RadialChart（イシュー #2079、親 #2078、Phase 5 #2076、
//! ルート #2001）。
//!
//! shadcn/ui Charts の Radial（`chart-radial-simple`/`-label`/`-grid`/
//! `-text`/`-shape`/`-stacked`）相当の同心リング型グラフを、外部依存ゼロ・
//! [`crate::charts`] 基盤の SVG ノード木生成のみで実装する
//! （[`crate::pie_chart`]/[`crate::donut_chart`] と対をなす、
//! `docs/policy/intentional-non-adoption.md` §7 の保留解除の延長）。
//! 弧計算は [`crate::charts::pie`]（[`crate::charts::pie::annulus_sector_path`]/
//! [`crate::charts::pie::annulus_sector_rounded_path`]/
//! [`crate::charts::pie::annulus_full_ring_path`]）をそのまま再利用する。
//! ark-ui に対応する headless anatomy が存在しないため、[`crate::pie_chart`]/
//! [`crate::donut_chart`] と同じ判断で新規 anatomy `data-scope="radial-chart"`
//! を本クレートのみで定義する。
//!
//! # データモデル・配色
//!
//! 入力は既存 [`crate::charts::ChartData`]（`categories` × `series`）を
//! そのまま使う。
//!
//! - **リング = カテゴリ**（`categories` の index 0 が最内周）。
//! - **リング内のセグメント = 系列**（系列順に累積して積み上げる。
//!   simple 系は系列 1 本、stacked は系列 2 本以上が典型）。
//! - 角度写像は `θ(v) = start + sweep × v / domain_max`（`domain_max` は
//!   全リングの系列値合計の最大値）。最大リングがちょうど `sweep` を埋める。
//! - 配色: 系列が 1 本の場合はカテゴリ index で
//!   [`crate::charts::series_color_var`]（ただし `Series::color` が
//!   設定されていれば全リング同色）。系列が 2 本以上の場合は
//!   [`crate::charts::ChartData::series_color_var`]（系列ごとの
//!   `Series::color` 上書きを尊重）。
//! - `bar` パーツへ `data-series="<系列名>"` を付与する（既存語彙、
//!   [`fandhe_frontend_core::render`] の既定エスケープ経由）。hover /
//!   hit-area 用の `data-*` は本イシューでは付けない（#2128/#2132 の担当）。
//!
//! # 角度規約
//!
//! プロパティは**度数法・12 時方向 0°・時計回り正**（[`crate::charts::pie`]
//! のラジアン規約 `-π/2` 起点と整合。変換は `θ = -π/2 + deg.to_radians()`）。
//! [`RadialChartProps::start_angle_deg`]（既定 `0.0`）/
//! [`RadialChartProps::end_angle_deg`]（既定 `360.0`）は、双方有限・
//! `end > start`・`end - start <= 360.0` を満たさない場合
//! [`RadialChartError::InvalidAngleRange`] を返す。半円 stacked の代表値は
//! `start = -90.0`/`end = 90.0`（9 時 → 12 時 → 3 時）。recharts は 3 時
//! 起点・反時計回りで shadcn simple は 470° を掃くが、本実装は 360° 以内に
//! 閉じる（一周を超える多重巻きは表現しない）。
//!
//! # レイアウト（viewBox `0 0 100 100` 固定、[`crate::pie_chart`]/
//! [`crate::donut_chart`] と同一）
//!
//! 中心 `(50, 50)`・外径 [`OUTER_RADIUS`]（`45.0`）・内径
//! `r_inner = 45 × inner_ratio`（[`RadialChartProps::inner_ratio`]、既定
//! `0.3`。`0.0 < ratio < 1.0` かつ有限でなければ
//! [`RadialChartError::InvalidInnerRatio`]）。リング帯
//! `band = (45 - r_inner) / n`、リング厚 `thickness = band × (1 - RING_GAP)`
//! （[`RING_GAP`] = `0.2` の定数、内側から外側へ配置）。各セグメントは
//! [`crate::charts::pie::annulus_sector_path`]（角丸なし）または
//! [`crate::charts::pie::annulus_sector_rounded_path`]（角丸あり）で描画する。
//! 値 `0` のセグメントは描画しない（pie/donut と同じ契約）。
//!
//! ## 全周退化の共通規則
//!
//! `track`（トラック全体）と `bar`（各セグメント）の両方に同一規則を適用
//! する: 角度幅（`end - start`）がちょうど `2π`（360°）のとき、
//! [`crate::charts::pie::annulus_sector_path`]/
//! [`crate::charts::pie::annulus_sector_rounded_path`] は始点=終点の退化
//! arc を返すため、[`crate::charts::pie::annulus_full_ring_path`] +
//! `fill-rule="evenodd"` へ切り替える（donut と同じ判断）。この判定は
//! 角丸処理より**先**に行い、全周のときは角丸半径を無視する。内部ヘルパ
//! `ring_segment_path` が `track`/`bar` 双方の呼び出しを一元化する
//! （個々のセグメントの角度幅が全周の場合に自然に検知できるため、
//! `track` の全周判定〔`props` の角度範囲が丁度 360°〕と `bar` の全周判定
//! 〔単独セグメントが `domain_max` に到達し `props` の角度範囲も丁度 360°〕
//! を個別分岐せず単一の幅比較へ統合できる）。既定の simple
//! （`show_track: true` + 360°）では全リングの `track` がこの分岐を通る。
//!
//! 角度幅がちょうど `2π` でなくても、全周との差が
//! [`fmt_coord`](crate::charts::svg::fmt_coord) の丸め（小数第 2 位）に
//! よって始点・終点が同一座標へ退化しうる範囲（半径依存、
//! `degenerate_angle_threshold` 参照）に収まる場合も同じ全周分岐へ含める
//! （イシュー #2079 codex-review 指摘の回帰: カテゴリ値
//! `[100000, 99999]` のように `domain_max` に極めて近い値では、
//! 対応するバーの角度幅が全周との差 `1e-9` を大きく上回りつつも視覚上
//! 区別不能な範囲（例: `約 6.28e-5 rad`）に収まり、素のまま扱うと
//! 座標丸めで外周・内周の始終点が一致して弧が消える）。
//!
//! # バリアント対応
//!
//! | 名前 | props の組み合わせ |
//! |---|---|
//! | simple | 系列 1 本、`show_track: true`（既定）、他は既定 |
//! | label | simple + `show_labels: true` |
//! | grid | simple + `show_grid: true`（`show_track` との併用は禁止しない） |
//! | text | 系列 1 本・カテゴリ 1 件 + `center_text: Some(..)`（任意で角度範囲変更） |
//! | shape | text + `corner_radius > 0.0` |
//! | stacked | 系列 2 本以上・カテゴリ 1 件 + `start -90.0`/`end 90.0` + `center_text` |
//!
//! ラベル（label バリアント）はリング開始角の点（リング中心半径）に水平
//! 配置し、`text-anchor: start` とする。弧に沿った回転は**意図的に非対応**
//! （テキスト幅を計測できない・決定性優先、下記「意図的に合わせなかった
//! 点」参照）。グリッドはリング帯境界の同心円（`n + 1` 本）+ 30° 刻み
//! 12 本の放射スポークから成り、いずれも個数は固定で決定的。中央テキスト
//! は `center-value` を `(50, 50)`・`center-label` を `(50, 57)` に配置する
//! （半円時も同一）。
//!
//! # セキュリティ不変条件
//!
//! [`crate::pie_chart`]/[`crate::donut_chart`] モジュール doc「セキュリティ
//! 不変条件」節と同一（`raw_html()` 不使用・数値文字列化は
//! [`crate::charts::svg::fmt_coord`] に一元化・カテゴリ名/系列名/
//! `aria_label`/中央テキスト/呼び出し側 `attrs` は既定エスケープ経由・
//! `class` は [`crate::class_attr::drop_class_attr`] で単一化）。`fill` は
//! [`crate::charts::series_color_var`]/[`crate::charts::SeriesColor::var`]
//! の固定形のみで、`transform` 等の自由文字列属性は導入しない。
//!
//! # 意図的に合わせなかった点
//!
//! - 弧に沿ったラベル回転（テキスト幅を実行時計測できないため、水平配置
//!   固定で決定性を優先する）
//! - 中央の muted ディスク（shadcn text/shape の `PolarGrid` fill トリック
//!   相当）は導入しない
//! - hover / tooltip / hit-area 用の `data-*`（#2128/#2132 の担当）
//!
//! # 本イシューのスコープ外
//!
//! - `component-coverage-map.md`・docs サイト Themes ページ・Demo・nav
//!   登録（兄弟イシュー #2080 が担当）
//! - `examples/headless-pre-styled-ui` への反映（crates.io 公開後）

use crate::charts::pie::{
    annulus_full_ring_path, annulus_sector_path, annulus_sector_rounded_path,
};
use crate::charts::svg::{circle, line, svg_root, svg_text, ViewBox};
use crate::charts::{series_color_var, ChartData};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};
use std::f64::consts::{FRAC_PI_2, PI};

/// `data-scope="radial-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("radial-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "chart",
    "track",
    "bar",
    "label",
    "grid-circle",
    "grid-spoke",
    "center-value",
    "center-label",
];

/// viewBox に対する中心 X 座標（[`crate::pie_chart`]/[`crate::donut_chart`]
/// と同一定数）。
const CENTER_X: f64 = 50.0;
/// viewBox に対する中心 Y 座標。
const CENTER_Y: f64 = 50.0;
/// viewBox に対する外径。
const OUTER_RADIUS: f64 = 45.0;
/// リング帯厚に対するリング間ギャップの比率（帯の 20% を隙間として残す）。
const RING_GAP: f64 = 0.2;
/// 全周（1 周）を表す角度差（ラジアン）。
const FULL_CIRCLE: f64 = 2.0 * PI;
/// 浮動小数の丸め誤差を吸収する全周判定の許容誤差（ラジアン）。
const FULL_CIRCLE_EPSILON: f64 = 1e-9;
/// [`fmt_coord`](crate::charts::svg::fmt_coord) の丸め幅（小数第 2 位、
/// `0.01`）に対する安全マージン（座標単位）。[`degenerate_angle_threshold`]
/// が半径から導く動的許容誤差の分子として使う（イシュー #2079 codex-review
/// 指摘の回帰、`ring_segment_path` doc 参照）。
const COORD_ROUNDING_SAFETY: f64 = 0.01;
/// [`degenerate_angle_threshold`] が返す動的許容誤差の上限（ラジアン）。
/// 半径が極端に小さいリング（`inner_ratio` を `0` に近づけた場合等）で
/// `COORD_ROUNDING_SAFETY / r` が発散し、本来別々に描画すべき弧まで
/// 全周分岐へ誤って合流させてしまうのを防ぐキャップ。
const FULL_CIRCLE_EPSILON_CAP: f64 = 0.01;
/// グリッドの放射スポーク本数（30° 刻み固定）。
const GRID_SPOKE_COUNT: usize = 12;

/// [`chart`] へ既定で付与する `aria-label`。
const DEFAULT_ARIA_LABEL: &str = "radial chart";

/// 本モジュール・[`crate::pie_chart`]/[`crate::donut_chart`]が返す構築
/// エラーとは独立の専用エラー（イシュー #2079。既存 `PieChartError`/
/// `ChartError` は列挙網羅テストを持ち純追加原則にも反するため variant を
/// 追加しない）。
///
/// `Display` はユーザーデータの値そのものを含めず、検証に失敗した理由の
/// みを記述する（`.claude/rules/security.md`「機微情報の露出」対応、
/// [`crate::charts::pie::PieChartError`] と同型の判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadialChartError {
    /// いずれかの値が負（角度写像の分子として意味を持たない）。
    NegativeValue,
    /// 各値は [`ChartData::new`] が有限であることを検証済みだが、その和
    /// （リングごとの系列値合計）はオーバーフローして `inf` になりうる。
    /// `domain_max`（全リングの合計の最大値）が非有限の場合に返す
    /// （[`crate::charts::pie::segment_angles`] の同種チェックと同じ判断、
    /// イシュー #850 レビュー指摘の教訓）。放置すると `cumulative / inf`
    /// が常に `0.0` になり、全セグメントが幅 0 の空チャートを
    /// fail-closed ではなく silent に描画してしまう。
    NonFiniteValue,
    /// 全リングの系列値合計の最大値（`domain_max`）が `0`
    /// （描画すべき弧が存在しない）。
    ZeroTotal,
    /// [`RadialChartProps::start_angle_deg`]/
    /// [`RadialChartProps::end_angle_deg`] が非有限、`end <= start`、または
    /// `end - start > 360.0`。
    InvalidAngleRange,
    /// [`RadialChartProps::inner_ratio`] が非有限、または
    /// `0.0 < ratio < 1.0` の範囲外。
    InvalidInnerRatio,
    /// [`RadialChartProps::corner_radius`] が非有限、または負。
    InvalidCornerRadius,
}

impl std::fmt::Display for RadialChartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            RadialChartError::NegativeValue => "value must not be negative",
            RadialChartError::NonFiniteValue => {
                "sum of a ring's values must be finite (overflow to inf is rejected)"
            }
            RadialChartError::ZeroTotal => {
                "sum of the largest ring's values must be greater than 0"
            }
            RadialChartError::InvalidAngleRange => {
                "start_angle_deg/end_angle_deg must be finite with a sweep greater than 0.0 and at most 360.0"
            }
            RadialChartError::InvalidInnerRatio => {
                "inner_ratio must be finite and strictly between 0.0 and 1.0"
            }
            RadialChartError::InvalidCornerRadius => {
                "corner_radius must be finite and non-negative"
            }
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for RadialChartError {}

/// [`RadialChartProps::center_text`] が指定する中央テキスト（値・任意の
/// 補足ラベル）。
///
/// 呼び出し側が渡す文字列をそのまま描画する（合計・整形は行わない、
/// `.claude/rules/coding-rust.md`「UI 部品の責務境界」§3.25 の一般化）。
#[derive(Debug, Clone, Copy)]
pub struct RadialCenterText<'a> {
    /// 中央に大きく表示する値（例: `"1,260"`）。
    pub value: &'a str,
    /// 値の下に添える補足ラベル（例: `"visitors"`）。`None` なら描画しない。
    pub label: Option<&'a str>,
}

/// [`radial_chart`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct RadialChartProps<'a> {
    /// 寸法（既定 `Md`）。
    pub size: Size,
    /// `chart`（svg）へ付与する `aria-label`。`None` なら
    /// [`DEFAULT_ARIA_LABEL`]（`"radial chart"`）を使う。
    pub aria_label: Option<&'a str>,
    /// 開始角（度数法・12 時方向 0°・時計回り正、既定 `0.0`）。
    pub start_angle_deg: f64,
    /// 終了角（度数法、既定 `360.0`）。
    pub end_angle_deg: f64,
    /// 外径に対する内径の比率（既定 `0.3`）。
    pub inner_ratio: f64,
    /// `bar`（セグメント）弧端の角丸半径（viewBox 単位、既定 `0.0` =
    /// 角丸なし）。有限かつ `0.0` 以上でなければならない。
    pub corner_radius: f64,
    /// `true` なら各リングの全スイープ背景トラックを描画する（既定
    /// `true`）。
    pub show_track: bool,
    /// `true` ならリング開始角の点にカテゴリ名ラベルを描画する（既定
    /// `false`）。
    pub show_labels: bool,
    /// `true` なら極座標グリッド（同心円 + 放射スポーク）を描画する
    /// （既定 `false`）。
    pub show_grid: bool,
    /// 中央テキスト（既定 `None`）。
    pub center_text: Option<RadialCenterText<'a>>,
}

impl Default for RadialChartProps<'_> {
    fn default() -> Self {
        Self {
            size: Size::Md,
            aria_label: None,
            start_angle_deg: 0.0,
            end_angle_deg: 360.0,
            inner_ratio: 0.3,
            corner_radius: 0.0,
            show_track: true,
            show_labels: false,
            show_grid: false,
            center_text: None,
        }
    }
}

/// この styled RadialChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] の
/// みが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("radial-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("--fandhe-radial-chart-size", "16rem"),
            ],
        )
        .base(
            "chart",
            vec![
                decl("width", "var(--fandhe-radial-chart-size)"),
                decl("height", "var(--fandhe-radial-chart-size)"),
            ],
        )
        .base("track", vec![decl("fill", "var(--fandhe-color-bg-muted)")])
        .base(
            "bar",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "label",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "4px"),
                decl("text-anchor", "start"),
                decl("dominant-baseline", "central"),
                decl("paint-order", "stroke"),
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "grid-circle",
            vec![
                decl("fill", "none"),
                decl("stroke", "var(--fandhe-color-border)"),
                decl("stroke-width", "0.5"),
            ],
        )
        .base(
            "grid-spoke",
            vec![
                decl("stroke", "var(--fandhe-color-border)"),
                decl("stroke-width", "0.5"),
            ],
        )
        .base(
            "center-value",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "12px"),
                decl("font-weight", "var(--fandhe-font-font-weight-bold)"),
                decl("text-anchor", "middle"),
                decl("dominant-baseline", "central"),
            ],
        )
        .base(
            "center-label",
            vec![
                decl("fill", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "4px"),
                decl("text-anchor", "middle"),
                decl("dominant-baseline", "central"),
            ],
        )
        .size_variants(
            "root",
            &[
                (Size::Xs, vec![decl("--fandhe-radial-chart-size", "4rem")]),
                (Size::Sm, vec![decl("--fandhe-radial-chart-size", "10rem")]),
                (Size::Md, vec![decl("--fandhe-radial-chart-size", "16rem")]),
                (Size::Lg, vec![decl("--fandhe-radial-chart-size", "22rem")]),
                (Size::Xl, vec![decl("--fandhe-radial-chart-size", "28rem")]),
            ],
        )
}

/// この styled RadialChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// リング index `i`（`0` が最内周）の内径・外径を返す（内部ヘルパ）。
fn ring_radii(i: usize, r_inner_base: f64, band: f64, thickness: f64) -> (f64, f64) {
    let inner = r_inner_base + band * i as f64;
    (inner, inner + thickness)
}

/// 半径 `r` 上の弧の始点・終点が [`fmt_coord`](crate::charts::svg::fmt_coord)
/// の丸め（小数第 2 位、`0.01`）によって同一座標へ退化しうる最小角度差
/// （内部ヘルパ、`ring_segment_path` doc 参照）。
///
/// 弦長は角度差 `dθ` に対しおよそ `r × dθ` で近似できる。この弦長が丸め幅
/// （[`COORD_ROUNDING_SAFETY`]）を下回る角度差では、外周・内周の始点と
/// 終点が独立丸めにより同一座標に一致し、SVG `A`（elliptical arc）コマンド
/// が退化して弧が描画されなくなる（イシュー #2079 codex-review 指摘:
/// カテゴリ値 `[100000, 99999]` で後者の弧が全周との差 `約 6.28e-5 rad` に
/// もかかわらず既存の [`FULL_CIRCLE_EPSILON`]（`1e-9`）を外れ、最大値
/// `99.999%` を表すバーが消える）。
///
/// `r` が極端に小さいリング（`inner_ratio` を `0` に近づけた場合等）では
/// `COORD_ROUNDING_SAFETY / r` が発散し、本来別々に描画すべき弧まで
/// 全周分岐へ誤って合流させてしまうため、[`FULL_CIRCLE_EPSILON_CAP`]
/// で上限を設ける。`r <= 0.0`（呼び出し元の契約違反）は
/// [`FULL_CIRCLE_EPSILON`] へフォールバックする。
fn degenerate_angle_threshold(r: f64) -> f64 {
    if r > 0.0 {
        (COORD_ROUNDING_SAFETY / r).min(FULL_CIRCLE_EPSILON_CAP)
    } else {
        FULL_CIRCLE_EPSILON
    }
}

/// 環状セグメント（`track`/`bar` 共通）の `d` 属性値と `evenodd`
/// フラグを組み立てる（内部ヘルパ、モジュール doc「全周退化の共通規則」
/// 節）。
///
/// 角度幅（`end - start`）がちょうど全周（`2π`）、または全周との差が
/// 座標丸めによる退化を起こしうる範囲（[`degenerate_angle_threshold`]、
/// 外周・内周のうち半径が小さい側で判定。半径が小さいほど同じ角度差でも
/// 弦長が短くなり退化しやすいため）の場合、退化 arc を避けるため
/// [`annulus_full_ring_path`] + `evenodd` へ切り替える。この判定は角丸
/// 処理より先に行うため、全周（近傍）のときは `rc`（角丸半径）を無視する。
/// 座標単位で `0.01` 未満の差は元々視覚的に区別できないため、全周へ
/// まとめても見た目の退行にはならない。
fn ring_segment_path(r_outer: f64, r_inner: f64, start: f64, end: f64, rc: f64) -> (String, bool) {
    let epsilon = FULL_CIRCLE_EPSILON.max(degenerate_angle_threshold(r_inner.min(r_outer)));
    if (end - start - FULL_CIRCLE).abs() < epsilon {
        (
            annulus_full_ring_path(CENTER_X, CENTER_Y, r_outer, r_inner),
            true,
        )
    } else if rc > 0.0 {
        (
            annulus_sector_rounded_path(CENTER_X, CENTER_Y, r_outer, r_inner, start, end, rc),
            false,
        )
    } else {
        (
            annulus_sector_path(CENTER_X, CENTER_Y, r_outer, r_inner, start, end),
            false,
        )
    }
}

/// 度数法（12 時方向 0°・時計回り正）の角度をラジアン（12 時方向 `-π/2`・
/// 時計回り正）へ変換する（内部ヘルパ、モジュール doc「角度規約」節）。
fn deg_to_rad(deg: f64) -> f64 {
    -FRAC_PI_2 + deg.to_radians()
}

/// RadialChart 1 個を組み立てる（`root` > `chart`(svg) >
/// [`grid-circle`/`grid-spoke`] > (`track`, `bar`(系列順), [`label`])×リング
/// > [`center-value`, `center-label`]）。
///
/// # Errors
///
/// - `start_angle_deg`/`end_angle_deg` が非有限、`end <= start`、または
///   `end - start > 360.0` の場合 [`RadialChartError::InvalidAngleRange`]
/// - `inner_ratio` が非有限、または `0.0 < ratio < 1.0` の範囲外の場合
///   [`RadialChartError::InvalidInnerRatio`]
/// - `corner_radius` が非有限、または負の場合
///   [`RadialChartError::InvalidCornerRadius`]
/// - いずれかの値が負の場合 [`RadialChartError::NegativeValue`]
/// - リングごとの系列値合計がオーバーフローして非有限になった場合
///   [`RadialChartError::NonFiniteValue`]
/// - 全リングの系列値合計の最大値が `0` の場合
///   [`RadialChartError::ZeroTotal`]
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::radial_chart::{radial_chart, RadialChartProps};
///
/// let data = ChartData::new(
///     vec!["A".to_string(), "B".to_string()],
///     vec![Series::new("total", vec![80.0, 55.0])],
/// )
/// .unwrap();
/// let node = radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"role="img""#));
/// ```
pub fn radial_chart<'a>(
    props: &RadialChartProps<'a>,
    data: &ChartData,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, RadialChartError> {
    if !(props.start_angle_deg.is_finite() && props.end_angle_deg.is_finite()) {
        return Err(RadialChartError::InvalidAngleRange);
    }
    let sweep_deg = props.end_angle_deg - props.start_angle_deg;
    if !(sweep_deg > 0.0 && sweep_deg <= 360.0) {
        return Err(RadialChartError::InvalidAngleRange);
    }
    if !(props.inner_ratio.is_finite() && 0.0 < props.inner_ratio && props.inner_ratio < 1.0) {
        return Err(RadialChartError::InvalidInnerRatio);
    }
    if !(props.corner_radius.is_finite() && props.corner_radius >= 0.0) {
        return Err(RadialChartError::InvalidCornerRadius);
    }
    for series in data.series() {
        if series.values.iter().any(|&v| v < 0.0) {
            return Err(RadialChartError::NegativeValue);
        }
    }

    let categories = data.categories();
    let n = categories.len();
    let domain_max = (0..n)
        .map(|i| data.series().iter().map(|s| s.values[i]).sum::<f64>())
        .fold(0.0_f64, f64::max);
    // 各値は `ChartData::new` が有限であることを検証済みだが、リングごとの
    // 合計はオーバーフローして `inf` になりうる（`RadialChartError::NonFiniteValue`
    // doc 参照）。`domain_max <= 0.0` の判定だけでは `inf`（`0.0` より大きい）
    // をすり抜けるため、合計自体の有限性を個別にチェックする。
    if !domain_max.is_finite() {
        return Err(RadialChartError::NonFiniteValue);
    }
    if domain_max <= 0.0 {
        return Err(RadialChartError::ZeroTotal);
    }

    // `start_angle_deg`/`end_angle_deg` は極端な絶対値（例:
    // 1e10 と 1e10+360）を個別に検証済みでも許容する（`sweep_deg` は
    // 減算のみで求まるため丸め誤差の影響を受けない）。しかし
    // `deg_to_rad` を両者へ独立適用すると `to_radians()` の丸め誤差が
    // 各値の絶対値の大きさに応じて乗るため、本来ちょうど全周
    // （`sweep_deg == 360.0`）であっても `end_rad - start_rad` が
    // `FULL_CIRCLE_EPSILON` の許容誤差を外れうる（全周退化判定
    // `ring_segment_path` がすり抜け、`fmt_coord` で始点・終点が
    // 同一座標に丸まってトラック/最大値リングが描画されない）。
    // これを避けるため、開始角を `rem_euclid` で `[0, 360)` へ
    // 正規化してから変換し、終了角は検証済みの `sweep_deg` を直接
    // ラジアンへ変換して開始角に加算する（両者を独立変換しない）。
    let start_angle_deg_normalized = props.start_angle_deg.rem_euclid(360.0);
    let start_rad = deg_to_rad(start_angle_deg_normalized);
    let sweep_rad = sweep_deg.to_radians();
    let end_rad = start_rad + sweep_rad;
    let r_inner_base = OUTER_RADIUS * props.inner_ratio;
    let band = (OUTER_RADIUS - r_inner_base) / n as f64;
    let thickness = band * (1.0 - RING_GAP);
    let single_series = data.series().len() == 1;

    let mut children: Vec<Node> = Vec::new();

    // グリッド（同心円境界 → 放射スポーク、モジュール doc「レイアウト」節）。
    if props.show_grid {
        for i in 0..=n {
            let r = r_inner_base + band * i as f64;
            children.push(circle(
                CENTER_X,
                CENTER_Y,
                r,
                vec![("data-scope", "radial-chart"), ("data-part", "grid-circle")],
            ));
        }
        for spoke in 0..GRID_SPOKE_COUNT {
            let angle = deg_to_rad(spoke as f64 * (360.0 / GRID_SPOKE_COUNT as f64));
            let (x1, y1) =
                crate::charts::pie::point_on_circle(CENTER_X, CENTER_Y, r_inner_base, angle);
            let (x2, y2) =
                crate::charts::pie::point_on_circle(CENTER_X, CENTER_Y, OUTER_RADIUS, angle);
            children.push(line(
                x1,
                y1,
                x2,
                y2,
                vec![("data-scope", "radial-chart"), ("data-part", "grid-spoke")],
            ));
        }
    }

    for (i, category) in categories.iter().enumerate() {
        let (r_inner, r_outer) = ring_radii(i, r_inner_base, band, thickness);

        if props.show_track {
            let (d, evenodd) = ring_segment_path(r_outer, r_inner, start_rad, end_rad, 0.0);
            let mut track_attrs: Vec<(&str, &str)> = vec![
                ("data-scope", "radial-chart"),
                ("data-part", "track"),
                ("d", d.as_str()),
            ];
            if evenodd {
                track_attrs.push(("fill-rule", "evenodd"));
            }
            children.push(el("path", track_attrs, vec![]));
        }

        let mut cumulative = 0.0_f64;
        for (j, series) in data.series().iter().enumerate() {
            let value = series.values[i];
            if value <= 0.0 {
                continue;
            }
            let seg_start = start_rad + sweep_rad * (cumulative / domain_max);
            cumulative += value;
            let seg_end = start_rad + sweep_rad * (cumulative / domain_max);

            let fill = if single_series {
                match &series.color {
                    Some(color) => color.var().to_string(),
                    None => series_color_var(i),
                }
            } else {
                data.series_color_var(j)
            };

            let (d, evenodd) =
                ring_segment_path(r_outer, r_inner, seg_start, seg_end, props.corner_radius);
            let mut bar_attrs: Vec<(&str, &str)> = vec![
                ("data-scope", "radial-chart"),
                ("data-part", "bar"),
                ("data-series", series.name.as_str()),
                ("d", d.as_str()),
                ("fill", fill.as_str()),
            ];
            if evenodd {
                bar_attrs.push(("fill-rule", "evenodd"));
            }
            children.push(el("path", bar_attrs, vec![]));
        }

        if props.show_labels {
            let label_r = (r_inner + r_outer) / 2.0;
            let (lx, ly) =
                crate::charts::pie::point_on_circle(CENTER_X, CENTER_Y, label_r, start_rad);
            children.push(svg_text(
                lx,
                ly,
                vec![("data-scope", "radial-chart"), ("data-part", "label")],
                vec![text(category.as_str())],
            ));
        }
    }

    if let Some(center_text) = props.center_text {
        children.push(svg_text(
            CENTER_X,
            CENTER_Y,
            vec![
                ("data-scope", "radial-chart"),
                ("data-part", "center-value"),
            ],
            vec![text(center_text.value)],
        ));
        if let Some(label) = center_text.label {
            children.push(svg_text(
                CENTER_X,
                CENTER_Y + 7.0,
                vec![
                    ("data-scope", "radial-chart"),
                    ("data-part", "center-label"),
                ],
                vec![text(label)],
            ));
        }
    }

    let view_box = ViewBox::new(0.0, 0.0, 100.0, 100.0)
        .expect("固定 viewBox 100x100 は常に有効な正の寸法である");
    let aria_label_value = props.aria_label.unwrap_or(DEFAULT_ARIA_LABEL);
    let chart_node = svg_root(
        &view_box,
        vec![
            ("data-scope", "radial-chart"),
            ("data-part", "chart"),
            ("aria-label", aria_label_value),
        ],
        children,
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
    use crate::charts::data::SeriesColor;
    use crate::charts::Series;
    use fandhe_frontend_core::render;

    fn two_category_data() -> ChartData {
        ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![80.0, 55.0])],
        )
        .unwrap()
    }

    fn one_category_data() -> ChartData {
        ChartData::new(
            vec!["visitors".to_string()],
            vec![Series::new("total", vec![100.0])],
        )
        .unwrap()
    }

    // --- simple ---

    #[test]
    fn simple_renders_root_chart_and_tracks_and_bars_with_default_aria_label() {
        let node =
            radial_chart(&RadialChartProps::default(), &two_category_data(), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-scope="radial-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="radial-chart" data-part="chart""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="radial chart""#));
        assert_eq!(html.matches(r#"data-part="track""#).count(), 2);
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
        assert_eq!(html.matches(r#"data-series="total""#).count(), 2);
    }

    #[test]
    fn simple_default_track_sweep_renders_as_seamless_full_ring() {
        // 既定角度範囲（0..360°）ではトラックが常に全周となるため、
        // 退化 arc を避ける `evenodd` 分岐を通ることを固定する
        // （モジュール doc「全周退化の共通規則」節）。
        let html = render(
            &radial_chart(&RadialChartProps::default(), &two_category_data(), vec![]).unwrap(),
        );
        // 既定は 2 カテゴリ（= 2 リング）で `show_track: true` のため、
        // トラック 2 本の両方が `evenodd` 分岐を通る。加えて `two_category_data`
        // はリング A（値 80）が `domain_max`（80）と一致する単独系列であり、
        // その `bar` 自体の角度幅も配置された `props` の全スイープと一致する
        // ため同じ全周退化分岐を通る（`track`/`bar` 共通規則の実例）。
        assert_eq!(html.matches(r#"fill-rule="evenodd""#).count(), 3);
    }

    #[test]
    fn custom_aria_label_overrides_default() {
        let props = RadialChartProps {
            aria_label: Some("revenue split"),
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains(r#"aria-label="revenue split""#));
    }

    #[test]
    fn zero_value_segment_is_skipped() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![0.0, 50.0])],
        )
        .unwrap();
        let html = render(&radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 1);
    }

    #[test]
    fn series_color_override_applies_to_all_rings() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![80.0, 55.0])
                .with_color(SeriesColor::token("fg").unwrap())],
        )
        .unwrap();
        let html = render(&radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"fill="var(--fandhe-color-fg)""#).count(), 2);
    }

    // --- label ---

    #[test]
    fn label_renders_category_name_per_ring() {
        let props = RadialChartProps {
            show_labels: true,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="label""#).count(), 2);
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    // --- grid ---

    #[test]
    fn grid_renders_ring_boundary_circles_and_twelve_spokes() {
        let props = RadialChartProps {
            show_grid: true,
            show_track: false,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &two_category_data(), vec![]).unwrap());
        // n=2 リング → n+1=3 本の境界円。
        assert_eq!(html.matches(r#"data-part="grid-circle""#).count(), 3);
        assert_eq!(html.matches(r#"data-part="grid-spoke""#).count(), 12);
    }

    #[test]
    fn grid_is_absent_when_show_grid_is_false() {
        let html = render(
            &radial_chart(&RadialChartProps::default(), &two_category_data(), vec![]).unwrap(),
        );
        assert_eq!(html.matches(r#"data-part="grid-circle""#).count(), 0);
        assert_eq!(html.matches(r#"data-part="grid-spoke""#).count(), 0);
    }

    // --- text ---

    #[test]
    fn text_renders_center_value_and_label() {
        let props = RadialChartProps {
            center_text: Some(RadialCenterText {
                value: "1,260",
                label: Some("visitors"),
            }),
            end_angle_deg: 250.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &one_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="center-value""#).count(), 1);
        assert_eq!(html.matches(r#"data-part="center-label""#).count(), 1);
        assert!(html.contains(">1,260<"));
        assert!(html.contains(">visitors<"));
    }

    #[test]
    fn text_without_label_omits_center_label() {
        let props = RadialChartProps {
            center_text: Some(RadialCenterText {
                value: "1,260",
                label: None,
            }),
            end_angle_deg: 250.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &one_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="center-value""#).count(), 1);
        assert_eq!(html.matches(r#"data-part="center-label""#).count(), 0);
    }

    // --- shape ---

    #[test]
    fn shape_applies_corner_radius_to_bar_only() {
        // カテゴリ 1 件・系列 1 本・inner_ratio 既定 0.3（リング厚
        // 約 31.5）・角度範囲 0..250°（span がコーナー縮小の閾値を十分
        // 上回る）で角丸が発動することを固定する。
        let props = RadialChartProps {
            corner_radius: 4.0,
            end_angle_deg: 250.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &one_category_data(), vec![]).unwrap());
        assert_eq!(html.matches("A4,4,").count(), 4);
    }

    #[test]
    fn corner_radius_zero_produces_no_rounded_corners() {
        let props = RadialChartProps {
            corner_radius: 0.0,
            end_angle_deg: 250.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &one_category_data(), vec![]).unwrap());
        assert!(!html.contains("A4,4,"));
    }

    // --- stacked ---

    #[test]
    fn stacked_multi_series_renders_two_bars_with_matching_boundary() {
        let data = ChartData::new(
            vec!["visitors".to_string()],
            vec![
                Series::new("mobile", vec![60.0]),
                Series::new("desktop", vec![40.0]),
            ],
        )
        .unwrap();
        let props = RadialChartProps {
            start_angle_deg: -90.0,
            end_angle_deg: 90.0,
            center_text: Some(RadialCenterText {
                value: "100",
                label: Some("total"),
            }),
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
        assert!(html.contains(r#"data-series="mobile""#));
        assert!(html.contains(r#"data-series="desktop""#));
    }

    #[test]
    fn stacked_series_color_uses_chart_data_series_color_var() {
        let data = ChartData::new(
            vec!["visitors".to_string()],
            vec![
                Series::new("mobile", vec![60.0]),
                Series::new("desktop", vec![40.0]).with_color(SeriesColor::token("fg").unwrap()),
            ],
        )
        .unwrap();
        let props = RadialChartProps {
            start_angle_deg: -90.0,
            end_angle_deg: 90.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &data, vec![]).unwrap());
        assert!(html.contains(r#"var(--fandhe-color-chart-1)"#));
        assert!(html.contains(r#"var(--fandhe-color-fg)"#));
    }

    #[test]
    fn stacked_second_segment_start_matches_first_segment_end() {
        let data = ChartData::new(
            vec!["visitors".to_string()],
            vec![
                Series::new("mobile", vec![60.0]),
                Series::new("desktop", vec![40.0]),
            ],
        )
        .unwrap();
        let props = RadialChartProps {
            start_angle_deg: -90.0,
            end_angle_deg: 90.0,
            ..RadialChartProps::default()
        };
        let node = radial_chart(&props, &data, vec![]).unwrap();
        let html = render(&node);

        // 期待される境界角（mobile 60% 分だけ進んだ角度）における外周座標を
        // 独自に算出し、その座標が html 中に一度だけでなく mobile の終点・
        // desktop の始点として現れる（＝連続している）ことを固定する。
        // n=1（単一リング）・inner_ratio 既定 0.3 のため
        // r_outer=45・r_inner=13.5。
        let r_inner_base = OUTER_RADIUS * RadialChartProps::default().inner_ratio;
        let band = OUTER_RADIUS - r_inner_base; // n=1 リングのため band = 帯全体
        let thickness = band * (1.0 - RING_GAP);
        let r_outer = r_inner_base + thickness;
        let start_rad = deg_to_rad(-90.0);
        let end_rad = deg_to_rad(90.0);
        let boundary_rad = start_rad + (end_rad - start_rad) * 0.6;
        let (bx, by) =
            crate::charts::pie::point_on_circle(CENTER_X, CENTER_Y, r_outer, boundary_rad);
        let boundary_coord = format!(
            "{},{}",
            crate::charts::svg::fmt_coord(bx),
            crate::charts::svg::fmt_coord(by)
        );
        // annulus_sector_path は "M{x1},{y1} A...,{x2},{y2} L..." の形式で
        // 外周終点 (x2,y2) を `A` の最後の座標として出力するため、
        // desktop の `M{boundary_coord}` と mobile 側の `,{boundary_coord} L`
        // の双方が現れることを確認する。
        assert!(
            html.contains(&format!("M{boundary_coord} ")),
            "boundary_coord={boundary_coord} html={html}"
        );
        assert!(
            html.contains(&format!(",{boundary_coord} L")),
            "boundary_coord={boundary_coord} html={html}"
        );
    }

    // --- 検証・エラー ---

    #[test]
    fn negative_value_is_rejected() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![-1.0, 50.0])],
        )
        .unwrap();
        assert_eq!(
            radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap_err(),
            RadialChartError::NegativeValue
        );
    }

    #[test]
    fn all_zero_values_is_zero_total() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(
            radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap_err(),
            RadialChartError::ZeroTotal
        );
    }

    #[test]
    fn ring_total_overflow_is_rejected_as_non_finite() {
        // 個々の値は有限（`f64::MAX` 近傍）でも、同一リング内の系列値合計は
        // オーバーフローして `inf` になりうる。`domain_max <= 0.0` だけでは
        // `inf` をすり抜けるため、合計自体の有限性チェックで拒否される
        // ことを固定する（`charts::pie::segment_angles` の同種回帰テストと
        // 同型、イシュー #850 レビュー指摘の教訓）。
        let data = ChartData::new(
            vec!["A".to_string()],
            vec![
                Series::new("s1", vec![f64::MAX]),
                Series::new("s2", vec![f64::MAX]),
            ],
        )
        .unwrap();
        assert_eq!(
            radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap_err(),
            RadialChartError::NonFiniteValue
        );
    }

    #[test]
    fn angle_range_boundary_and_non_finite_values_are_rejected() {
        let cases = [
            (0.0, 0.0),
            (100.0, 50.0),
            (0.0, 361.0),
            (f64::NAN, 360.0),
            (0.0, f64::INFINITY),
        ];
        for (start, end) in cases {
            let props = RadialChartProps {
                start_angle_deg: start,
                end_angle_deg: end,
                ..RadialChartProps::default()
            };
            assert_eq!(
                radial_chart(&props, &two_category_data(), vec![]).unwrap_err(),
                RadialChartError::InvalidAngleRange,
                "start={start} end={end}"
            );
        }
    }

    #[test]
    fn full_360_degree_range_is_accepted() {
        let props = RadialChartProps {
            start_angle_deg: 0.0,
            end_angle_deg: 360.0,
            ..RadialChartProps::default()
        };
        assert!(radial_chart(&props, &two_category_data(), vec![]).is_ok());
    }

    #[test]
    fn full_360_degree_range_with_large_angle_offset_renders_as_seamless_full_ring() {
        // codex-review 指摘の回帰: start_angle_deg/end_angle_deg が
        // ちょうど 360° の範囲でも巨大な絶対値（例: 1e10 付近）だと、
        // 各値を独立に `deg_to_rad` すると `to_radians()` の丸め誤差で
        // `end_rad - start_rad` が全周判定の許容誤差
        // （`FULL_CIRCLE_EPSILON`）を外れうる（開始角の正規化と
        // 検証済み `sweep_deg` からの終了角組み立てで回避、モジュール
        // doc「全周退化の共通規則」節）。全周退化分岐を通らないと
        // `fmt_coord` で始点・終点が同一座標に丸まり、トラック/最大値
        // リングの `d` が描画不能な退化パスになる。
        let props = RadialChartProps {
            start_angle_deg: 10_000_000_000.0,
            end_angle_deg: 10_000_000_360.0,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"fill-rule="evenodd""#).count(), 3);
    }

    #[test]
    fn near_full_circle_bar_degenerate_by_coord_rounding_renders_as_seamless_full_ring() {
        // codex-review 指摘の回帰（イシュー #2079、未解決スレッド
        // PRRT_kwDOTarxgc6gb4Jv）: カテゴリ値を `[100000, 99999]` にすると
        // 後者（`domain_max` の `99.999%`）の弧が全周との差
        // `約 6.28e-5 rad` で `FULL_CIRCLE_EPSILON`（`1e-9`）の許容誤差を
        // 大きく外れ、全周判定分岐を通らない。しかしこの角度差は
        // `fmt_coord` の小数第 2 位丸めに対しては視覚上区別不能な範囲
        // （外周・内周の弦長が丸め幅 `0.01` を大きく下回る）であり、素の
        // まま `annulus_sector_path`/`annulus_sector_rounded_path` に渡すと
        // 独立丸めにより外周・内周の始点・終点が同一座標に退化し、
        // 最大値 `99.999%` を表すバーの SVG arc が消える。
        // `ring_segment_path` の動的許容誤差
        // （`degenerate_angle_threshold`）でこの近傍も全周分岐へ含める
        // ことで、2 カテゴリ双方の `track`（2 件）と `bar`（2 件）が
        // すべて `annulus_full_ring_path` + `evenodd` を経由することを
        // 固定する。
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![100_000.0, 99_999.0])],
        )
        .unwrap();
        let html = render(&radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"fill-rule="evenodd""#).count(), 4);
    }

    #[test]
    fn inner_ratio_boundary_and_non_finite_values_are_rejected() {
        for ratio in [0.0, 1.0, -0.1, 1.1, f64::NAN, f64::INFINITY] {
            let props = RadialChartProps {
                inner_ratio: ratio,
                ..RadialChartProps::default()
            };
            assert_eq!(
                radial_chart(&props, &two_category_data(), vec![]).unwrap_err(),
                RadialChartError::InvalidInnerRatio,
                "ratio={ratio}"
            );
        }
    }

    #[test]
    fn corner_radius_negative_or_non_finite_is_rejected() {
        for rc in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let props = RadialChartProps {
                corner_radius: rc,
                ..RadialChartProps::default()
            };
            assert_eq!(
                radial_chart(&props, &two_category_data(), vec![]).unwrap_err(),
                RadialChartError::InvalidCornerRadius,
                "rc={rc}"
            );
        }
    }

    // --- size / class / css ---

    #[test]
    fn size_variant_applies_root_class() {
        let node = radial_chart(
            &RadialChartProps {
                size: Size::Sm,
                ..RadialChartProps::default()
            },
            &two_category_data(),
            vec![],
        )
        .unwrap();
        assert!(render(&node).contains("radial-chart--size-sm"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(
            &radial_chart(
                &RadialChartProps::default(),
                &two_category_data(),
                vec![("class", "attacker-controlled")],
            )
            .unwrap(),
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn render_is_deterministic() {
        let a = render(
            &radial_chart(&RadialChartProps::default(), &two_category_data(), vec![]).unwrap(),
        );
        let b = render(
            &radial_chart(&RadialChartProps::default(), &two_category_data(), vec![]).unwrap(),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn css_output_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="radial-chart"][data-part="chart"]"#));
        assert!(!a.contains("color-palette"));
    }

    #[test]
    fn error_display_never_leaks_arbitrary_values() {
        for err in [
            RadialChartError::NegativeValue,
            RadialChartError::NonFiniteValue,
            RadialChartError::ZeroTotal,
            RadialChartError::InvalidAngleRange,
            RadialChartError::InvalidInnerRatio,
            RadialChartError::InvalidCornerRadius,
        ] {
            let message = err.to_string();
            assert!(!message.is_empty());
            assert!(!message.contains('<'));
        }
    }
}
