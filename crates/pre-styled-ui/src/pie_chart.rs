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
//!
//! # 参考サイト基準への調整（イシュー #1596）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にチャート部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・データラベルのコントラスト）に限定する。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6` 固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由。`label` の `font-size: 6px` は viewBox ユーザー単位のため静的リテラルのまま） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | ラベルのコントラストはハローで是正（下記）。系列パレット自体の見直しはスコープ外 |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（扇形 SVG 描画のみ） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `label` slot に `dominant-baseline: central` を追加し、ラベルを
//!   扇形中心へ垂直方向にセンタリングした。従来は `text-anchor: middle`
//!   のみでベースライン調整が無く、狭い扇形ほど文字がベースライン基準で
//!   上側へ浮き扇形外へはみ出していた
//! - `label` slot へ背景色ハロー（`paint-order: stroke` /
//!   `stroke: var(--fandhe-color-bg)` / `stroke-width: 1` /
//!   `stroke-linejoin: round`）を追加した。dark モードでは `fill: var(--fandhe-color-fg)`
//!   が系列色の dark 値（`chart-1`/`chart-2` 等）に対して WCAG 4.5:1 を
//!   大きく下回り（`theme.rs` の light/dark トークン値からの概算）、light
//!   モードでも一部系列色で 4.5:1 未満だったため、系列色・ページ背景の
//!   どちらの上でも可読なハローで是正した（先例:
//!   [`crate::donut_chart`]（#1594）/ [`crate::area_chart`] `point` /
//!   `charts::tooltip` `datum`）。`paint-order: stroke` によりストローク
//!   を塗りの下へ回すため文字形は太らない
//! - `segment` slot に `stroke-linejoin: round` を追加した。各扇形 path は
//!   `M 中心 L 外周始点 A 外周弧 Z`（[`crate::charts::pie::sector_path`]）
//!   で閉じるため、**全セグメントが中心点を鋭角の共有頂点として持つ**。
//!   既定の miter では背景色ストローク（`stroke: var(--fandhe-color-bg)`）
//!   が中心から隣接セグメント側へ突き出し、描画順（後勝ち）に依存して
//!   背景色のスパイクが見えていた（単一全周セグメントの `<circle>` 分岐
//!   には結合部が無く無害）。donut（内周・外周の 4 頂点）より pie の方が
//!   中心 1 点に全セグメントが集まる分、症状が顕著だった
//!
//! 上記 3 点は兄弟部品 [`crate::donut_chart`]（#1594）で先行是正済みであり、
//! 本イシューはその引き継ぎとして pie 側に同型の是正を適用する。
//!
//! ## 意図的に合わせなかった点
//!
//! - `chart` slot への `overflow: visible` は、外径 45 + ストローク半幅
//!   0.5 が viewBox（100×100）内に収まるため不要
//! - `segment` slot への `vector-effect: non-scaling-stroke` は、兄弟部品
//!   [`crate::donut_chart`] と線幅の見え方が乖離するため見送る
//! - `Xs`（4rem）+ `show_labels` 時、`font-size: 6px` は実寸約 3.8px で
//!   判読が難しくなるが、ラベル表示は呼び出し側の選択であり本 PR では
//!   制約しない
//! - `label` の `font-weight` 引き上げ・`pointer-events: none` の付与は、
//!   効果が限定的で donut-chart（#1594）との整合を崩すため見送る
//! - `label` の `font-size` トークン化・系列パレット見直し等、上記 3 点を
//!   超える変更は双子部品（donut-chart）との整合を崩すため本 PR に含めない

use crate::charts::pie::{
    annulus_full_ring_path, annulus_sector_path, is_right_half, leader_line_path,
    outside_label_effective_outer_radius, outside_label_point, sector_path, segment_angles,
    PieChartError,
};
use crate::charts::svg::{circle, fmt_coord, svg_root, svg_text, ViewBox};
use crate::charts::{series_color_var, tooltip, ChartData};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="pie-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("pie-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "chart",
    "segment",
    "label",
    "label-line",
    "outside-label",
];

/// viewBox に対する中心 X 座標（固定、モジュール doc「幾何・角度計算」節）。
const CENTER_X: f64 = 50.0;
/// viewBox に対する中心 Y 座標（固定）。
const CENTER_Y: f64 = 50.0;
/// viewBox に対する外径（固定）。
const OUTER_RADIUS: f64 = 45.0;
/// ラベルを配置する半径（外径に対する比率。セグメント内側寄りに置く）。
const LABEL_RADIUS_RATIO: f64 = 0.6;

/// [`PieLabelPosition::Outside`] 使用時に縮小する外径の上限（引き出し線・
/// ラベルを viewBox 内に収めるため。モジュール doc「幾何上の制約」節、
/// イシュー #2084、shadcn `chart-pie-label` 突合）。ラベル文字列が短い場合
/// はこの値まで使う（[`crate::charts::pie::outside_label_effective_outer_radius`]
/// 参照）。
const OUTSIDE_LABEL_OUTER_RADIUS: f64 = 34.0;
/// [`OUTSIDE_LABEL_OUTER_RADIUS`] 縮小の下限（退化防止。長いラベル文字列で
/// クランプされた場合、テキストがはみ出しうる既知の限界はモジュール doc
/// `outside_label_effective_outer_radius` rustdoc 参照）。
const MIN_OUTSIDE_LABEL_OUTER_RADIUS: f64 = 15.0;
/// 外側ラベルの 1 文字あたり推定表示幅（`font-size: 5px` 相当、
/// `bar_chart` モジュールの `AVG_LABEL_CHAR_WIDTH` と同じ等幅想定近似
/// 設計。イシュー #2084 レビュー指摘）。
const AVG_LABEL_CHAR_WIDTH: f64 = 3.0;
/// viewBox の幅（固定 100×100、モジュール doc 参照）。
const VIEW_BOX_WIDTH: f64 = 100.0;
/// 引き出し線の放射方向の長さ（[`crate::charts::pie::leader_line_path`]）。
const LEADER_RADIAL_LEN: f64 = 4.0;
/// 引き出し線の水平方向の長さ。
const LEADER_HORIZONTAL_LEN: f64 = 6.0;
/// 引き出し線終端からラベルまでの追加余白。
const LEADER_LABEL_GAP: f64 = 1.5;
/// [`PieChartProps::stacked`]（shadcn `chart-pie-stacked`、イシュー #2084）
/// 時のリング間の隙間（viewBox 単位）。
const RING_GAP: f64 = 1.0;

/// [`chart`] へ既定で付与する `aria-label`（[`PieChartProps::aria_label`]
/// が `None` の場合に使う）。
const DEFAULT_ARIA_LABEL: &str = "pie chart";

/// [`PieLabelPosition::Outside`] 使用時の外径を、実際に描画される外側
/// ラベル文字列（[`PieLabelContent::Category`] ならカテゴリ名、
/// [`PieLabelContent::Value`] なら [`fmt_coord`] 後の値文字列）の最長推定
/// 表示幅を考慮して縮小する（[`crate::charts::pie::outside_label_effective_outer_radius`]
/// 参照。イシュー #2084 レビュー指摘。当初カテゴリ名のみを見ており
/// `label_content = Value` 表示時に値の桁数がはみ出すすり抜けがあった、
/// 同イシュー PR #2257 レビュー指摘で修正）。
fn resolve_outside_label_outer_radius(
    categories: &[String],
    values: &[f64],
    label_content: PieLabelContent,
) -> f64 {
    let max_label_len = match label_content {
        PieLabelContent::Category => categories.iter().map(|c| c.chars().count()).max(),
        PieLabelContent::Value => values.iter().map(|v| fmt_coord(*v).chars().count()).max(),
    }
    .unwrap_or(0);
    outside_label_effective_outer_radius(
        CENTER_X,
        VIEW_BOX_WIDTH,
        OUTSIDE_LABEL_OUTER_RADIUS,
        MIN_OUTSIDE_LABEL_OUTER_RADIUS,
        LEADER_RADIAL_LEN,
        LEADER_HORIZONTAL_LEN,
        LEADER_LABEL_GAP,
        AVG_LABEL_CHAR_WIDTH,
        max_label_len,
    )
}

/// セグメント間セパレータ（shadcn `chart-pie-separator-none` 突合、
/// イシュー #2084）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PieSeparator {
    /// 背景色ストロークで区切る（既定。イシュー #1596 以前からの挙動）。
    #[default]
    Line,
    /// セパレータなし（`segment` の `stroke` を `none` にする）。
    None,
}

impl VariantValue for PieSeparator {
    fn axis(self) -> &'static str {
        "separator"
    }

    fn value(self) -> &'static str {
        match self {
            PieSeparator::Line => "line",
            PieSeparator::None => "none",
        }
    }
}

/// セグメント上ラベルの内容（shadcn `chart-pie-label`/`chart-pie-label-list`
/// 突合、イシュー #2084）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PieLabelContent {
    /// カテゴリ名（既定。イシュー #1596 以前からの挙動）。
    #[default]
    Category,
    /// 値（[`crate::charts::svg::fmt_coord`] 固定書式、shadcn
    /// `chart-pie-label`。数値整形・単位付与はアプリ側の責務、
    /// `.claude/rules/coding-rust.md` §3.25 と同じ判断軸）。
    Value,
}

/// セグメント上ラベルの配置（shadcn `chart-pie-label`/`chart-pie-label-custom`
/// 突合、イシュー #2084）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PieLabelPosition {
    /// 扇形内側（既定。イシュー #1596 以前からの挙動）。
    #[default]
    Inside,
    /// 扇形外側・引き出し線付き（shadcn `chart-pie-label`）。外径を
    /// [`OUTSIDE_LABEL_OUTER_RADIUS`] へ縮小する（モジュール doc「幾何上の
    /// 制約」節）。[`PieChartProps::stacked`] と併用した場合、引き出し
    /// ラベルは最外周リングにのみ付く（他リングは [`Inside`] 相当の位置に
    /// 描画する。モジュール doc参照）。
    Outside,
}

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
    /// セグメント間セパレータ（既定 `Line`、イシュー #2084）。
    pub separator: PieSeparator,
    /// [`show_labels`](Self::show_labels) が `true` の場合のラベル内容
    /// （既定 `Category`、イシュー #2084）。
    pub label_content: PieLabelContent,
    /// [`show_labels`](Self::show_labels) が `true` の場合のラベル配置
    /// （既定 `Inside`、イシュー #2084）。
    pub label_position: PieLabelPosition,
    /// `true` なら複数系列をリング（多重円）として描画する（shadcn
    /// `chart-pie-stacked`、既定 `false`、イシュー #2084）。`true` の場合
    /// のみ複数系列を許容し（`false` は従来どおり単一系列専用）、系列
    /// index がリング（0 が最内周）に対応する。モジュール doc「stacked」節
    /// 参照。
    pub stacked: bool,
    /// `true`（既定）なら hit-area・`data-index`/`data-series` と `hidden`
    /// の SSR ツールチップ DOM（[`crate::charts::tooltip::layer`]）を
    /// 出力する（イシュー #2129、親 #2128）。`false` の場合は本イシュー
    /// 以前の出力とバイト一致する。
    pub show_tooltip: bool,
}

impl Default for PieChartProps<'_> {
    fn default() -> Self {
        Self {
            size: Size::Md,
            aria_label: None,
            show_labels: false,
            separator: PieSeparator::Line,
            label_content: PieLabelContent::Category,
            label_position: PieLabelPosition::Inside,
            stacked: false,
            show_tooltip: true,
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
                // イシュー #2129: `tooltip-layer`（`position: absolute`）の
                // 配置規則（#2130 が唯一のロケータとして使う契約）を成立
                // させるための末尾純追加。
                decl("position", "relative"),
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
                // イシュー #1596: 全セグメントが中心点を鋭角の共有頂点として
                // 持つため（`sector_path` が `M 中心 L 外周始点 A 外周弧 Z`
                // で閉じる）、既定の miter だと背景色セパレータが中心から
                // 隣接セグメント側へ突出して見える（donut #1594 と同型）。
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "label",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "6px"),
                decl("text-anchor", "middle"),
                // イシュー #1596: ラベルを扇形中心へ垂直センタリングする
                // （`text-anchor` のみでは水平方向しか揃わず、狭い扇形で
                // 文字がベースライン基準で上側へ浮き扇形外へはみ出していた）。
                decl("dominant-baseline", "central"),
                // イシュー #1596: 背景色ハローで系列色・ページ背景どちらの
                // 上でも可読性を確保する（dark モードで `fg` が系列色の
                // dark 値に対し WCAG 4.5:1 を大きく下回るための是正、
                // donut #1594 と同型）。`paint-order: stroke` でストロークを
                // 塗りの下へ回し、文字形が太って見えるのを防ぐ。
                decl("paint-order", "stroke"),
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "label-line",
            vec![
                // イシュー #2084: shadcn `chart-pie-label` の引き出し線
                // （`PieLabelPosition::Outside`）。
                decl("stroke", "var(--fandhe-color-fg-muted)"),
                decl("stroke-width", "0.5"),
                decl("fill", "none"),
            ],
        )
        .base(
            "outside-label",
            vec![
                // イシュー #2084: 扇形外側ラベル（`PieLabelPosition::Outside`）。
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "5px"),
                decl("text-anchor", "start"),
                decl("dominant-baseline", "central"),
            ],
        )
        // イシュー #1681: `crate::donut_chart::recipe` と同一の 6rem 刻み
        // 進行を共有する（size 値は donut と揃えている）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-pie-chart-size", "4rem")],
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
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-pie-chart-size", "28rem")],
        )
        .default_variant(Size::Md)
        // イシュー #2084: 外側ラベル使用時、水平方向終端（左半分）は
        // `text-anchor: end` へ切り替える（[`is_right_half`] が偽の場合、
        // `pie_chart` 本体が `data-align="end"` を付与する）。
        .state(
            "outside-label",
            StateCondition::AttrEq("data-align", "end"),
            vec![decl("text-anchor", "end")],
        )
        // イシュー #2084: shadcn `chart-pie-separator-none` 突合。
        .variant(PieSeparator::None, "segment", vec![decl("stroke", "none")])
}

/// この styled PieChart が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// 1 リング分のセグメント（+ ラベル）ノードを `out` へ積む（内部ヘルパ、
/// [`pie_chart`] の非 stacked/stacked 双方の描画経路が共有する）。
///
/// `r_inner <= 0.0` は pie 型セグメント（[`sector_path`]/[`circle`]）、
/// `r_inner > 0.0` は donut 型の環状セグメント（[`annulus_sector_path`]/
/// [`annulus_full_ring_path`]）を描く。`series_name` は
/// [`PieChartProps::stacked`] 時のみ `Some`（`data-series` 属性、既存 B-2
/// 語彙の共有）。`is_outermost` が `false` の場合、
/// [`PieLabelPosition::Outside`] でも [`PieLabelPosition::Inside`] 相当の
/// 位置（`(r_inner + r_outer) / 2`）へフォールバックする（モジュール doc
/// 「stacked」節、shadcn に stacked+外側ラベルの組み合わせは存在しないため
/// 独自に定めた規則）。
#[allow(clippy::too_many_arguments)]
fn render_ring<'a>(
    out: &mut Vec<Node>,
    categories: &[String],
    values: &[f64],
    angles: &[(f64, f64)],
    r_inner: f64,
    r_outer: f64,
    props: &PieChartProps<'a>,
    separator_class: &str,
    series_name: Option<&str>,
    is_outermost: bool,
) {
    // 非ゼロ値のセグメントがちょうど 1 個の場合（全周セグメント）は
    // sector_path/annulus_sector_path が始点=終点の退化 arc を返すため、
    // 代わりに circle/annulus_full_ring_path を描画する
    // （`crate::charts::pie` モジュール doc「境界規則」節）。
    let non_zero_count = values.iter().filter(|&&v| v > 0.0).count();
    let is_full_circle = non_zero_count == 1;

    for (i, (&(start, end), &value)) in angles.iter().zip(values.iter()).enumerate() {
        // 値 0 のセグメントは境界角が退化するため描画しない。
        if value <= 0.0 {
            continue;
        }
        let fill = series_color_var(i);

        let mut segment_attrs: Vec<(&str, &str)> =
            vec![("data-scope", "pie-chart"), ("data-part", "segment")];
        if let Some(name) = series_name {
            segment_attrs.push(("data-series", name));
        }
        if !separator_class.is_empty() {
            segment_attrs.push(("class", separator_class));
        }

        let segment_node = if r_inner <= 0.0 {
            if is_full_circle {
                segment_attrs.push(("fill", fill.as_str()));
                circle(CENTER_X, CENTER_Y, r_outer, segment_attrs)
            } else {
                let d = sector_path(CENTER_X, CENTER_Y, r_outer, start, end);
                segment_attrs.push(("d", d.as_str()));
                segment_attrs.push(("fill", fill.as_str()));
                el("path", segment_attrs, vec![])
            }
        } else if is_full_circle {
            let d = annulus_full_ring_path(CENTER_X, CENTER_Y, r_outer, r_inner);
            segment_attrs.push(("d", d.as_str()));
            segment_attrs.push(("fill", fill.as_str()));
            segment_attrs.push(("fill-rule", "evenodd"));
            el("path", segment_attrs, vec![])
        } else {
            let d = annulus_sector_path(CENTER_X, CENTER_Y, r_outer, r_inner, start, end);
            segment_attrs.push(("d", d.as_str()));
            segment_attrs.push(("fill", fill.as_str()));
            el("path", segment_attrs, vec![])
        };
        out.push(segment_node);

        if !props.show_labels {
            continue;
        }
        let mid = (start + end) / 2.0;
        let category = categories.get(i).map(String::as_str).unwrap_or_default();
        let label_text = match props.label_content {
            PieLabelContent::Category => category.to_string(),
            PieLabelContent::Value => fmt_coord(value),
        };

        let use_outside = props.label_position == PieLabelPosition::Outside && is_outermost;
        if use_outside {
            let leader_d = leader_line_path(
                CENTER_X,
                CENTER_Y,
                r_outer,
                mid,
                LEADER_RADIAL_LEN,
                LEADER_HORIZONTAL_LEN,
            );
            out.push(el(
                "path",
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "label-line"),
                    ("d", leader_d.as_str()),
                ],
                vec![],
            ));
            let (lx, ly) = outside_label_point(
                CENTER_X,
                CENTER_Y,
                r_outer,
                mid,
                LEADER_RADIAL_LEN,
                LEADER_HORIZONTAL_LEN,
                LEADER_LABEL_GAP,
            );
            let align = if is_right_half(mid) { "start" } else { "end" };
            out.push(svg_text(
                lx,
                ly,
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "outside-label"),
                    ("data-align", align),
                ],
                vec![text(label_text.as_str())],
            ));
        } else {
            let label_r = if r_inner <= 0.0 {
                r_outer * LABEL_RADIUS_RATIO
            } else {
                (r_inner + r_outer) / 2.0
            };
            let lx = CENTER_X + label_r * mid.cos();
            let ly = CENTER_Y + label_r * mid.sin();
            out.push(svg_text(
                lx,
                ly,
                vec![("data-scope", "pie-chart"), ("data-part", "label")],
                vec![text(label_text.as_str())],
            ));
        }
    }
}

/// PieChart 1 個を組み立てる（`root` > `chart`(svg) > `segment`(path/circle)
/// [+ `label-line`(path) + `outside-label`(text) | `label`(text)]）。
///
/// `data` はカテゴリ数 = セグメント数。[`PieChartProps::stacked`] が
/// `false`（既定）の場合系列数は必ず 1（モジュール doc「単一系列専用」節
/// 参照）、`true` の場合は系列 index がリング（0 が最内周）に対応する
/// （モジュール doc「stacked」節）。呼び出し側 `attrs` は `root` へ合成する
/// （`class` は [`drop_class_attr`] で除去してから recipe クラスへ一本化）。
///
/// # Errors
///
/// - `stacked == false` かつ `data.series().len() != 1` の場合
///   [`PieChartError::MultiSeries`]
/// - `stacked == true` かつ `data.series()` が空の場合
///   [`PieChartError::MultiSeries`]
/// - いずれかの系列の値に非有限・負値が含まれる、またはいずれかの系列の
///   合計が `0` の場合 [`crate::charts::pie::segment_angles`] のエラーを
///   そのまま返す
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
    let categories = data.categories();
    let recipe = recipe();
    let separator_class = if props.separator == PieSeparator::None {
        recipe.variant_class(PieSeparator::None)
    } else {
        String::new()
    };

    let mut nodes: Vec<Node> = Vec::new();

    if props.stacked {
        let series = data.series();
        if series.is_empty() {
            return Err(PieChartError::MultiSeries);
        }
        let ring_count = series.len();
        // 最外周リングが Outside ラベルを持つ場合、非 stacked 分岐と同様に
        // 外径を実際の外側ラベル文字列（カテゴリ名 or 値）の推定表示幅込み
        // で縮小してから band 計算する（引き出し線・outside-label が
        // viewBox 0..100 の外へはみ出すのを防ぐ。イシュー #2084 レビュー
        // 指摘、PR #2257 レビュー指摘で label_content = Value も考慮）。
        // outside label は最外周リング（`series` 末尾）のみが持つため
        // （`render_ring` の `is_outermost` 判定）、幅計算もその系列の値を
        // 参照する。
        let effective_outer_radius = if props.show_labels
            && props.label_position == PieLabelPosition::Outside
        {
            let outermost_values = series
                .last()
                .map(|s| s.values.as_slice())
                .unwrap_or_default();
            resolve_outside_label_outer_radius(categories, outermost_values, props.label_content)
        } else {
            OUTER_RADIUS
        };
        let band = effective_outer_radius / ring_count as f64;
        for (k, s) in series.iter().enumerate() {
            let angles = segment_angles(&s.values)?;
            let r_outer = band * (k as f64 + 1.0);
            let r_inner = if k == 0 {
                0.0
            } else {
                let inner_no_gap = band * k as f64;
                (inner_no_gap + RING_GAP)
                    .min(r_outer - 0.1)
                    .max(inner_no_gap)
            };
            render_ring(
                &mut nodes,
                categories,
                &s.values,
                &angles,
                r_inner,
                r_outer,
                props,
                &separator_class,
                Some(s.name.as_str()),
                k == ring_count - 1,
            );
        }
    } else {
        if data.series().len() != 1 {
            return Err(PieChartError::MultiSeries);
        }
        let values = &data.series()[0].values;
        let angles = segment_angles(values)?;
        let outer_radius = if props.show_labels && props.label_position == PieLabelPosition::Outside
        {
            resolve_outside_label_outer_radius(categories, values, props.label_content)
        } else {
            OUTER_RADIUS
        };
        render_ring(
            &mut nodes,
            categories,
            values,
            &angles,
            0.0,
            outer_radius,
            props,
            &separator_class,
            None,
            true,
        );
    }

    // イシュー #2129: hit-area・SSR ツールチップ DOM。非 stacked は唯一の
    // リングの幾何のみに基づく。stacked はリングごとに独立した非ゼロ
    // セグメント判定を行い、値を持つ全リングぶんの hit-area を生成する
    // （PR #2261 codex-review 指摘、threadId PRRT_kwDOTarxgc6guRrW:
    // 最外周のみを判定すると内周にのみ値を持つカテゴリの hit-area が
    // 欠落するため）。値 0 のセグメントは境界角が退化するため
    // `render_ring` と同様に hit-area も出さない（リング単位の判定）。
    let entries = if props.show_tooltip {
        // イシュー #2129 codex-review 指摘: pie の実描画色は
        // `series_color_var(i)`（`i` はカテゴリ index。リング＝系列を
        // またいで同一カテゴリは常に同色、上記 `render_ring` 参照。pie は
        // `Series::color` による個別上書きを持たない）であり、
        // `entries_from_chart_data` の既定（系列 index 基準）とは異なる。
        // `tooltip-indicator` の色を実際のスライス色に一致させるため
        // カテゴリ index 基準へ上書きする。
        let mut entries = tooltip::entries_from_chart_data(data);
        for entry in &mut entries {
            let color = crate::charts::SeriesColor::chart_slot(entry.index % 6 + 1)
                .expect("entry.index % 6 + 1 は常に 1..=6 の範囲内");
            for row in &mut entry.rows {
                row.color = color.clone();
            }
        }
        Some(entries)
    } else {
        None
    };
    if let Some(entries) = &entries {
        // イシュー #2129 codex-review 指摘（PR #2261、threadId
        // PRRT_kwDOTarxgc6guRrW）: stacked pie は各リングの非ゼロ
        // セグメントごとに hit-area を要る。当初は最外周リング
        // （`series` 末尾）の値のみを判定していたため、内周にのみ値を
        // 持つカテゴリ（例: category A が内周のみ非ゼロ、最外周は 0）の
        // hit-area が生成されず、生成済みの hidden tooltip DOM に対応する
        // 操作ターゲットが無い状態になっていた。stacked のときは全リング
        // を走査し、各リングの非ゼロセグメントごとに（同一カテゴリでも
        // 複数リングにまたがれば複数 hit-area を）そのリング自身の幾何
        // （`render_ring` と同じ per-ring `is_full_circle` 判定）で生成
        // する。ツールチップ本体は [`tooltip::entries_from_chart_data`]
        // によりカテゴリ単位（全系列の行を内包）で 1 個のみ生成される
        // ため、`data-index`（カテゴリ index）はリングをまたいで共有し、
        // `data-series` は付与しない（従来どおり）。
        if props.stacked {
            let series = data.series();
            let ring_count = series.len();
            let effective_outer_radius =
                if props.show_labels && props.label_position == PieLabelPosition::Outside {
                    let outermost_values = series
                        .last()
                        .map(|s| s.values.as_slice())
                        .unwrap_or_default();
                    resolve_outside_label_outer_radius(
                        categories,
                        outermost_values,
                        props.label_content,
                    )
                } else {
                    OUTER_RADIUS
                };
            let band = effective_outer_radius / ring_count as f64;
            for (k, s) in series.iter().enumerate() {
                let r_outer = band * (k as f64 + 1.0);
                let r_inner = if k == 0 {
                    0.0
                } else {
                    let inner_no_gap = band * k as f64;
                    (inner_no_gap + RING_GAP)
                        .min(r_outer - 0.1)
                        .max(inner_no_gap)
                };
                let angles = segment_angles(&s.values)?;
                let non_zero_count = s.values.iter().filter(|&&v| v > 0.0).count();
                let is_full_circle = non_zero_count == 1;
                for entry in entries {
                    let value = s.values.get(entry.index).copied().unwrap_or(0.0);
                    if value <= 0.0 {
                        continue;
                    }
                    let label = tooltip::hit_area_label(entry);
                    if is_full_circle && r_inner <= 0.0 {
                        nodes.push(tooltip::hit_area_circle(
                            CENTER_X,
                            CENTER_Y,
                            r_outer,
                            entry.index,
                            None,
                            &label,
                        ));
                        continue;
                    }
                    let (start, end) = angles[entry.index];
                    let d = if is_full_circle {
                        annulus_full_ring_path(CENTER_X, CENTER_Y, r_outer, r_inner)
                    } else if r_inner <= 0.0 {
                        sector_path(CENTER_X, CENTER_Y, r_outer, start, end)
                    } else {
                        annulus_sector_path(CENTER_X, CENTER_Y, r_outer, r_inner, start, end)
                    };
                    nodes.push(tooltip::hit_area_path(
                        &d,
                        entry.index,
                        None,
                        &label,
                        is_full_circle,
                    ));
                }
            }
        } else {
            let outer_values = data.series()[0].values.clone();
            let outer_radius = if props.show_labels
                && props.label_position == PieLabelPosition::Outside
            {
                resolve_outside_label_outer_radius(categories, &outer_values, props.label_content)
            } else {
                OUTER_RADIUS
            };
            let angles = segment_angles(&outer_values)?;
            let non_zero_count = outer_values.iter().filter(|&&v| v > 0.0).count();
            let is_full_circle = non_zero_count == 1;
            for entry in entries {
                let value = outer_values.get(entry.index).copied().unwrap_or(0.0);
                if value <= 0.0 {
                    continue;
                }
                let label = tooltip::hit_area_label(entry);
                if is_full_circle {
                    nodes.push(tooltip::hit_area_circle(
                        CENTER_X,
                        CENTER_Y,
                        outer_radius,
                        entry.index,
                        None,
                        &label,
                    ));
                    continue;
                }
                let (start, end) = angles[entry.index];
                let d = sector_path(CENTER_X, CENTER_Y, outer_radius, start, end);
                nodes.push(tooltip::hit_area_path(
                    &d,
                    entry.index,
                    None,
                    &label,
                    is_full_circle,
                ));
            }
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
        nodes,
    );

    let mut children = vec![chart_node];
    if let Some(entries) = &entries {
        children.push(tooltip::layer_from_entries(entries, None));
    }

    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));

    Ok(ANATOMY.part("root", "div", merged, children))
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
        // イシュー #2129: 全周セグメント時の hit-area も `hit_area_circle`
        // （`<circle>`）で描くため、既定（`show_tooltip: true`）ではセグメント
        // + hit-area の 2 件になる（`<path>` を経由しない点は不変）。
        assert_eq!(html.matches("<circle").count(), 2);
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

    #[test]
    fn recipe_includes_issue_1596_corrections() {
        // イシュー #1596: ラベルの垂直センタリング・背景色ハロー・
        // セパレータ線の miter 突出抑止が実出力に現れることを固定する
        // （黙って除外されていないことの確認、donut #1594 の
        // `recipe_includes_issue_1594_corrections` と同型）。
        let a = css();
        assert!(a.contains(r#"[data-scope="pie-chart"][data-part="label"]"#));
        assert!(a.contains("dominant-baseline: central"));
        assert!(a.contains("paint-order: stroke"));
        assert!(a.contains("stroke-linejoin: round"));
    }

    #[test]
    fn separator_none_adds_variant_class_and_default_omits_it() {
        let props = PieChartProps {
            separator: PieSeparator::None,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains("fd-pie-chart--separator-none"));

        let default_html =
            render(&pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap());
        assert!(!default_html.contains("fd-pie-chart--separator-none"));
    }

    #[test]
    fn label_content_value_uses_fmt_coord() {
        let props = PieChartProps {
            show_labels: true,
            label_content: PieLabelContent::Value,
            // イシュー #2129: ツールチップ DOM は `label_content` に関わらず
            // 常にカテゴリ名（`tooltip-label`）を出す（segment 上ラベルとは
            // 独立した契約）。本テストの関心はセグメント上ラベルのみのため
            // ツールチップ DOM を無効化し、既存のアサーションをそのまま保つ。
            show_tooltip: false,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains(">60<"));
        assert!(html.contains(">40<"));
        assert!(!html.contains(">A<"));
    }

    #[test]
    fn outside_labels_reserve_value_text_width_margin_when_label_content_is_value() {
        // 回帰テスト（PR #2257 codex-review P1）: `label_content = Value`
        // の場合、外径縮小の余白計算がカテゴリ名の文字数のみを見ており
        // 実際に描画される値の文字列幅を無視していた。カテゴリ名が短く
        // （"A"/"B"）値が大きい（50000）と、値の 5 桁がカテゴリ名 1 文字分
        // の余白（4.5 未満）を超えてはみ出す。修正後は Value 表示時に
        // `fmt_coord` 後の文字列幅を見込んで外径を縮小し、余白が確保される
        // ことを確認する。
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![50000.0, 50000.0])],
        )
        .unwrap();
        let props = PieChartProps {
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            label_content: PieLabelContent::Value,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &data, vec![]).unwrap());
        // 固定 34（カテゴリ名基準の余白のみ）のときの x=95.5 は使われて
        // いないこと。
        assert!(!html.contains(r#"x="95.5""#));
        let max_value_len = 5.0; // fmt_coord(50000.0) は "50000"（5 文字）。
        for x in extract_attr_values(&html, r#"data-part="outside-label""#, "x") {
            assert!(
                (0.0..=100.0).contains(&x),
                "outside-label x={x} は viewBox(0..100) の外"
            );
            let margin_to_edge = if x >= 50.0 { 100.0 - x } else { x };
            assert!(
                margin_to_edge >= max_value_len * AVG_LABEL_CHAR_WIDTH - 1e-9,
                "x={x} の余白 {margin_to_edge} は推定値文字幅未満"
            );
        }
    }

    #[test]
    fn label_position_outside_renders_leader_line_and_outside_label() {
        let props = PieChartProps {
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="label-line""#).count(), 2);
        assert_eq!(html.matches(r#"data-part="outside-label""#).count(), 2);
        assert!(!html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-align="start""#) || html.contains(r#"data-align="end""#));
        // 外径が OUTSIDE_LABEL_OUTER_RADIUS(34) へ縮小されていること。
        assert!(html.contains("34,34,0,"));
        assert!(!html.contains("45,45,0,"));
    }

    #[test]
    fn outside_labels_reserve_text_width_margin_for_longer_category_names() {
        // レビュー指摘（イシュー #2084 codex-review P1）: 等しい値の 2
        // カテゴリで固定 OUTSIDE_LABEL_OUTER_RADIUS(34) のみを使うと
        // outside-label の x 座標が 95.5/4.5 となり、viewBox 端までの余白
        // 4.5 では "Chrome" のような通常長のカテゴリ名（6 文字）が
        // はみ出す。修正後は文字幅を見込んで外径を縮小し、x 座標が
        // 推定文字幅ぶんの余白を viewBox 内に残すことを確認する。
        let data = ChartData::new(
            vec!["Chrome".to_string(), "Safari".to_string()],
            vec![Series::new("total", vec![50.0, 50.0])],
        )
        .unwrap();
        let props = PieChartProps {
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &data, vec![]).unwrap());
        // 固定 34 のときの x=95.5（右半分）は使われていないこと。
        assert!(!html.contains(r#"x="95.5""#));
        let max_category_len = 6.0; // "Chrome"/"Safari" いずれも 6 文字。
        for x in extract_attr_values(&html, r#"data-part="outside-label""#, "x") {
            assert!(
                (0.0..=100.0).contains(&x),
                "outside-label x={x} は viewBox(0..100) の外"
            );
            let margin_to_edge = if x >= 50.0 { 100.0 - x } else { x };
            assert!(
                margin_to_edge >= max_category_len * AVG_LABEL_CHAR_WIDTH - 1e-9,
                "x={x} の余白 {margin_to_edge} は推定文字幅未満"
            );
        }
    }

    #[test]
    fn default_position_inside_does_not_shrink_outer_radius() {
        let html =
            render(&pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap());
        assert!(!html.contains(r#"data-part="label-line""#));
        assert!(!html.contains(r#"data-part="outside-label""#));
        assert!(!html.contains(r#"data-align""#));
    }

    #[test]
    fn stacked_renders_one_ring_per_series_with_data_series_attribute() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                Series::new("2023", vec![60.0, 40.0]),
                Series::new("2024", vec![70.0, 30.0]),
            ],
        )
        .unwrap();
        let props = PieChartProps {
            stacked: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 4);
        assert!(html.contains(r#"data-series="2023""#));
        assert!(html.contains(r#"data-series="2024""#));
    }

    #[test]
    fn stacked_hit_area_covers_category_present_only_in_inner_ring() {
        // PR #2261 codex-review 指摘（イシュー #2129、threadId
        // PRRT_kwDOTarxgc6guRrW）の回帰テスト: stacked=true・カテゴリ
        // A/B、内周系列（0 番目、最内周）が [1, 0]（A のみ非ゼロ）、
        // 最外周系列が [0, 1]（B のみ非ゼロ）という構成では、最外周の値
        // のみを判定する旧ロジックだと A の hit-area が省略され、
        // 生成済みの hidden tooltip DOM に対応する操作ターゲットが
        // 無くなっていた。修正後は両カテゴリぶんの hit-area
        // （`data-part="hit-area"`）が生成され、それぞれ対応する
        // `data-index` を持つことを検証する。
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                Series::new("inner", vec![1.0, 0.0]),
                Series::new("outer", vec![0.0, 1.0]),
            ],
        )
        .unwrap();
        let props = PieChartProps {
            stacked: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="hit-area""#).count(), 2);
        assert!(html.contains(r#"data-index="0""#));
        assert!(html.contains(r#"data-index="1""#));
    }

    #[test]
    fn stacked_single_series_matches_non_stacked_ring_geometry() {
        let data = two_category_data();
        let stacked_props = PieChartProps {
            stacked: true,
            ..PieChartProps::default()
        };
        let stacked_html = render(&pie_chart(&stacked_props, &data, vec![]).unwrap());
        // n == 1 の stacked は非 stacked と同一のリング（r_inner=0,
        // r_outer=OUTER_RADIUS）に退化する。
        assert!(stacked_html.contains("45,45,0,") || stacked_html.contains("<circle"));
    }

    #[test]
    fn stacked_outside_labels_shrink_outermost_ring_within_view_box() {
        // レビュー指摘（イシュー #2084）: stacked + Outside ラベルの組み合わせで
        // 最外周リングの引き出し線・outside-label が viewBox(0..100) の外へ
        // はみ出していた不具合の回帰テスト。非 stacked 分岐と同様に
        // OUTSIDE_LABEL_OUTER_RADIUS(34) へ縮小した半径から band を計算する
        // ことを、生成された座標が viewBox 内に収まることで確認する。
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                Series::new("2023", vec![60.0, 40.0]),
                Series::new("2024", vec![70.0, 30.0]),
            ],
        )
        .unwrap();
        let props = PieChartProps {
            stacked: true,
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &data, vec![]).unwrap());
        // 最外周リングのみ Outside ラベルを持つ（モジュール doc「stacked」節）。
        assert_eq!(html.matches(r#"data-part="outside-label""#).count(), 2);
        assert_eq!(html.matches(r#"data-part="label-line""#).count(), 2);
        // 非 stacked 分岐と同じく 34（OUTSIDE_LABEL_OUTER_RADIUS）まで縮小されて
        // おり、OUTER_RADIUS(45) は使われていないこと。
        assert!(html.contains("34,34,0,") || html.contains("17,17,0,"));
        assert!(!html.contains("45,45,0,"));
        // outside-label の x 属性値が viewBox(0..100) を超えないこと。
        for x in extract_attr_values(&html, r#"data-part="outside-label""#, "x") {
            assert!(
                (0.0..=100.0).contains(&x),
                "outside-label x={x} は viewBox(0..100) の外"
            );
        }
    }

    /// `needle`（例: `data-part="outside-label"`）を含む `<text ...>` 開始タグ
    /// から `attr`（例: `x`）属性値を数値として抽出するテスト専用ヘルパ。
    fn extract_attr_values(html: &str, needle: &str, attr: &str) -> Vec<f64> {
        let mut out = Vec::new();
        for tag_start in html.match_indices("<text ").map(|(i, _)| i) {
            let tag_end = html[tag_start..]
                .find('>')
                .map(|off| tag_start + off)
                .unwrap_or(html.len());
            let tag = &html[tag_start..tag_end];
            if !tag.contains(needle) {
                continue;
            }
            let pat = format!(r#"{attr}=""#);
            if let Some(start) = tag.find(&pat) {
                let value_start = start + pat.len();
                if let Some(end_off) = tag[value_start..].find('"') {
                    let value_str = &tag[value_start..value_start + end_off];
                    if let Ok(value) = value_str.parse::<f64>() {
                        out.push(value);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn stacked_false_still_rejects_multi_series() {
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
    fn default_output_never_contains_new_2084_attributes() {
        // イシュー #2129: ツールチップ DOM の `tooltip-item` は系列表示名を
        // `data-series` として常に出す（stacked に関わらず）ため、
        // 本テストの本来の関心（segment への `data-series` 付与は
        // `stacked: true` 限定）を検証するにはツールチップ DOM を無効化する。
        let props = PieChartProps {
            show_tooltip: false,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(!html.contains("data-series"));
        assert!(!html.contains("data-align"));
        assert!(!html.contains("fd-pie-chart--separator-none"));
        assert!(!html.contains("label-line"));
        assert!(!html.contains("outside-label"));
    }
}
