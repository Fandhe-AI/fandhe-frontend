//! LineChart（イシュー #848、親 Phase #845）: `charts` 基盤（#846）の最初の
//! 消費者。系列ごとの折れ線を SVG ノード木として描画する自己完結部品。
//! #847（軸/グリッド）は本モジュールへ統合済み（`show_x_axis`/`show_y_axis`/
//! `show_grid`、イシュー #2083）。
//!
//! # shadcn/ui Charts（line）突合（イシュー #2083）
//!
//! shadcn/ui の line-chart registry は 10 バリアントを持つが、実行時
//! インタラクション（マウス追従ツールチップ・hover 強調・`activeDot`・
//! 期間切替・凡例トグル）を除く静的に描画できるバリアントを本イシューで
//! props の純追加として補完した。
//!
//! | registry | 本実装での対応 |
//! |---|---|
//! | `chart-line-default` | `curve: Curve::Natural` + `show_grid` + `show_x_axis` |
//! | `chart-line-linear` | `curve: Curve::Linear`（既定、既存出力と同一） |
//! | `chart-line-step` | `curve: Curve::Step` |
//! | `chart-line-multiple` | 複数系列（既存対応、変更なし） |
//! | `chart-line-dots` | `dots: LineDots::Filled` |
//! | `chart-line-dots-custom` | `dots: LineDots::Hollow`（任意アイコン形状は非対応、静的近似） |
//! | `chart-line-dots-colors` | `color_by_category: true`（点の色をカテゴリ index で `chart-1〜6` 循環） |
//! | `chart-line-label` | `label: LineLabel::Value` |
//! | `chart-line-label-custom` | `label: LineLabel::Category` |
//! | `chart-line-interactive` | 対象外（#2132、期間切替・凡例トグル） |
//!
//! ## 意図的に合わせなかった点
//!
//! - `tickFormatter`（値の 3 文字切詰）はアプリ側整形の責務
//!   （`docs/policy/intentional-non-adoption.md` §3.23/§3.25）
//! - 任意アイコンによるカスタム点形状は非対応。`LineDots::Hollow` を
//!   静的近似として提供する（`Series::icon` は凡例専用のまま不変）
//! - `activeDot`（hover 時の点拡大）・マウス追従ツールチップ・hit-area
//!   `data-*` は #2128 のスコープ
//! - 点ごとの任意色は非対応。`color_by_category` によるトークン循環
//!   （`chart-1〜6`）で代替する
//! - shadcn の左右 margin（`margin: { left: 12, right: 12 }`）は
//!   `overflow: visible`（本モジュール既存の是正）で端点ラベルの見切れを
//!   回避しており、余白は `label`/`show_x_axis`/`show_y_axis` 有効時の
//!   上下方向のみ確保する
//! - 積み上げ・横向きは shadcn line registry に存在しないため非対応
//! - 凡例は呼び出し側が [`crate::charts::legend`] を並べる想定のまま
//!   （`Series::with_icon` で icon 表示可、イシュー #2077）
//! - `-interactive`（期間切替）・凡例トグルは #2132
//!
//! # 座標写像・数値文字列化の一元化
//!
//! 座標写像は [`crate::charts::scale::LinearScale`]、`path` の `d` 属性は
//! [`crate::charts::svg::PathBuilder`]、数値の決定的文字列化は
//! [`crate::charts::svg::fmt_coord`]（`PathBuilder`/`circle`/`svg_root` 内部
//! 経由）のみを通す。本モジュール自身は独自の数値フォーマット・座標計算式を
//! 実装しない（`.claude/rules/coding-rust.md` 決定性の一元化）。
//!
//! # x/y 軸の写像規則
//!
//! - x 軸: カテゴリ index `i`（`0..n`）を [`category_x`] で等間隔配置する。
//!   `n == 1`（単一カテゴリ）は `i * width / (n - 1)` が 0 除算になるため
//!   特別扱いし、`width / 2.0` の中央 1 点のみへ配置する。
//! - y 軸: [`ChartData::domain`](crate::charts::data::ChartData::domain)
//!   （フラットデータの非退化パディング込み）を
//!   `LinearScale::new(domain, (height, 0.0))`（SVG の y 下向き正のため range
//!   を反転）で写像する。軸/グリッド無効時（既定）は `nice()` を適用しない
//!   （#2083 以前と同一の domain）。軸/グリッド有効時（`show_x_axis`/
//!   `show_y_axis`/`show_grid` のいずれか）は [`crate::area_chart`] と
//!   同じ規則で `nice()` を適用し目盛の切りの良い値へ拡張する。
//!
//! # エッジケース（golden テスト対象、`tests/charts_line_area_sparkline.rs`）
//!
//! | 入力 | 挙動 |
//! |------|------|
//! | 単一カテゴリ（`n == 1`） | `path` を生成せず、中央に [`point`] マーカーのみ描く |
//! | 負値・負のみの系列 | domain がそのまま負域を含み、`fmt_coord` の `-` 付き座標で決定的出力 |
//! | フラット（全値同値） | `ChartData::domain` のパディングにより中央水平線 |
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/
//! [`ANATOMY`] 経由のノード木 API のみで組み立て、`raw_html()`・SVG/HTML
//! 文字列の直接組み立ては一切行わない（REQ-1）。`aria_label`・呼び出し側
//! `attrs`（`data-testid` 等）は `fandhe_frontend_core::render` の既定
//! エスケープを必ず経由する。`path`/`circle` の座標・寸法は [`fmt_coord`]
//! （文字集合 `[0-9.-]` に閉じる）のみを経由し、任意文字列混入経路を持たない。
//! CSS 宣言値はすべてコンパイル時静的リテラルであり、動的値を CSS へ流し込む
//! 経路はない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 凡例・ツールチップ・積み上げは非対応のまま（軸・グリッド・曲線補間は
//!   #2083 で本モジュールへ統合済み）。
//! - マウス追従ツールチップ・hover 強調・`activeDot`・hit-area `data-*` は
//!   #2128、期間切替・凡例トグルは #2132。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::qr_code`] の先例と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1595）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にチャート部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・ラベルのコントラスト）に限定する
//! （兄弟部品 [`crate::area_chart`]（#1589）と同じ判断）。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6` 固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由。生の色リテラルなし） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | 追加規則なし（系列色・`--fandhe-color-bg` は dark 値定義済み） |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（線のみの SVG 描画） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `plot` slot に `overflow: visible` を追加し、domain の max/min に接する
//!   折れ線が UA 既定 `overflow: hidden` で viewBox 上下端において
//!   `stroke-width: 2` の半分をクリップされる欠陥を、ジオメトリを変えず
//!   CSS のみで是正した（先例: [`crate::area_chart`] #1589）
//! - `series-line` slot に `stroke-linejoin: round` / `stroke-linecap: round`
//!   を追加し、折れ線の鋭角部での miter 突出を抑えた
//!   （先例: [`crate::signature_pad`] / [`crate::progress`] / [`crate::area_chart`]）
//! - `point` slot（`n == 1` 時の点マーカー）に背景色のハロー
//!   （`stroke: var(--fandhe-color-bg)`）を追加し、背景・隣接系列色との
//!   識別性を高めた
//!
//! ## 意図的に合わせなかった点（イシュー #1595）
//!
//! - `series-line` への `vector-effect: non-scaling-stroke` は、#1863（area-chart）
//!   が「必要なら #1593 で横断的に」と先送りしたが #1593 は非採用のまま完了
//!   したため、area-chart / sparkline との線幅の見え方の整合を保つべく
//!   本 PR でも非採用とする
//! - `view_box_from_dims` / `category_x`（[`crate::area_chart`]/[`crate::sparkline`]
//!   と共有するヘルパ）へのパディング追加は #1599（sparkline）と競合する
//!   ため見送った
//! - `theme.rs` への opacity 等トークン新設は、消費者が chart 系のみで
//!   契約テストへ波及するため見送った

use crate::area_chart::AXIS_LEFT_MARGIN;
use crate::area_chart::{grid_lines_for_ticks, x_axis_category_labels, AXIS_BOTTOM_MARGIN};
use crate::charts::axis::{self, AxisProps};
use crate::charts::curve::{self, Curve};
use crate::charts::data::ChartData;
use crate::charts::drop_range_attr;
use crate::charts::scale::LinearScale;
use crate::charts::svg::{fmt_coord, svg_root, svg_text, ViewBox, ViewBoxError};
use crate::charts::{tooltip, ChartError};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    transition_declarations, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="line-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("line-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。`value-label`（イシュー #2083）は
/// [`LineLabel`] 有効時の値/カテゴリラベルが使う。
const SLOTS: &[&str] = &["root", "plot", "series-line", "point", "value-label"];

/// `viewBox` 幅の既定値（chakra `charts/line-chart.md` の代表例に近い横長比率）。
pub const DEFAULT_WIDTH: f64 = 300.0;
/// `viewBox` 高さの既定値。
pub const DEFAULT_HEIGHT: f64 = 150.0;

/// 単一カテゴリ（`n == 1`）時に描く点マーカー、および [`LineDots`] 有効時の
/// データ点マーカーの半径（`viewBox` 座標系）。shadcn の `r=3`（dots）/
/// `r=5`（dots-colors）へは合わせず、`n == 1` golden を固定する既存値を
/// 共有する（イシュー #2083、golden 純追加原則）。
const POINT_RADIUS: f64 = 2.5;

/// [`LineLabel`] 有効時、データ点の上へ確保するラベルの垂直オフセット
/// （shadcn `LabelList offset={12}` 相当、イシュー #2083）。
const LABEL_OFFSET: f64 = 12.0;
/// [`LineLabel`] 有効時にプロット領域上側へ確保する余白（shadcn
/// `margin: { top: 20 }` 相当、イシュー #2083。bar_chart の `LABEL_MARGIN`
/// と同値）。
const LABEL_TOP_MARGIN: f64 = 20.0;

/// データ点マーカーの表示（shadcn `dot`、イシュー #2083）。
///
/// 既定 [`LineDots::None`] は `n == 1`（単一カテゴリ）時の既存マーカーのみ
/// 描く挙動（#2083 以前と同一）を保つ。`n >= 2` で有効化すると各データ点に
/// 円マーカーを追加で描く。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineDots {
    /// マーカーなし（既定。`n == 1` の中央点マーカーは従来どおり描く）。
    #[default]
    None,
    /// 系列色で塗りつぶした点（shadcn `chart-line-dots`）。
    Filled,
    /// 背景色で塗り系列色の輪郭を持つ点（shadcn `chart-line-dots-custom`
    /// の静的近似。任意アイコン形状は非対応、モジュール doc「意図的に
    /// 合わせなかった点」参照）。
    Hollow,
}

impl VariantValue for LineDots {
    fn axis(self) -> &'static str {
        "dots"
    }

    fn value(self) -> &'static str {
        match self {
            LineDots::None => "none",
            LineDots::Filled => "filled",
            LineDots::Hollow => "hollow",
        }
    }
}

/// データ点上の値ラベル（shadcn `LabelList`、イシュー #2083）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineLabel {
    /// ラベルなし（既定）。
    #[default]
    None,
    /// 値をそのまま表示（shadcn `chart-line-label`）。
    Value,
    /// カテゴリ名を表示（shadcn `chart-line-label-custom`。config への
    /// 表示名写像はアプリ側整形の責務、モジュール doc参照）。
    Category,
}

/// [`line_chart`] の入力。
pub struct LineChartProps<'a> {
    /// 描画するチャートデータ（[`ChartData::new`](crate::charts::data::ChartData::new)
    /// を経由して構築済みであることが不変条件。空データ・非有限値混入は
    /// 構築時に拒否済みのため本関数へは到達しない）。
    pub data: &'a ChartData,
    /// `svg` 要素へ付与する `aria-label`（必須。装飾ではなくデータ可視化のため
    /// スクリーンリーダー向け説明を必須とする、`.claude/rules/security.md`
    /// 以前に a11y 上の必須要件）。
    pub aria_label: &'a str,
    /// `viewBox` 幅（描画座標系。CSS 表示寸法は [`Size`] variant が別途制御する）。
    pub width: f64,
    /// `viewBox` 高さ。
    pub height: f64,
    /// root へ付与する寸法 variant（svg の CSS 表示高さを切替える、
    /// [`crate::qr_code`] と同型）。
    pub size: Size,
    /// 曲線種（イシュー #2083、既定 [`Curve::Linear`]）。既定値は #2083
    /// 以前の出力と完全に同一の `d` 属性を生成する（golden 純追加原則）。
    pub curve: Curve,
    /// データ点マーカー（イシュー #2083、既定 [`LineDots::None`]）。
    pub dots: LineDots,
    /// 値/カテゴリラベル（イシュー #2083、既定 [`LineLabel::None`]）。
    pub label: LineLabel,
    /// データ点の色をカテゴリ index で `chart-1〜6`循環へ切り替えるか
    /// （shadcn `chart-line-dots-colors`、イシュー #2083、既定 `false`）。
    /// `false` のときは系列色（[`ChartData::series_color_var`]）のまま。
    pub color_by_category: bool,
    /// X 軸（カテゴリ）を描画するか（イシュー #2083、既定 `false`）。
    pub show_x_axis: bool,
    /// Y 軸（数値目盛）を描画するか（イシュー #2083、既定 `false`）。
    pub show_y_axis: bool,
    /// 水平グリッド線を描画するか（イシュー #2083、既定 `false`）。
    pub show_grid: bool,
    /// `true`（既定）なら hit-area・`data-index`/`data-series` と `hidden`
    /// の SSR ツールチップ DOM（[`crate::charts::tooltip::layer`]）を
    /// 出力する（イシュー #2129、親 #2128）。`false` の場合は本イシュー
    /// 以前の出力とバイト一致する。
    pub show_tooltip: bool,
    /// 表示範囲の不透明な識別子（イシュー #2133、親 #2132）。`Some(v)` の
    /// とき root へ `data-range="<v>"` を出力する（既定 `None`＝非出力）。
    /// 期間→カテゴリ集合の写像は定義しない（アプリ/wasm-full〔#2134〕の
    /// 責務、`crate::charts` モジュール doc「期間切替・凡例トグルの SSR
    /// 構造」節参照）。呼び出し側 `attrs` に同名キーがあっても
    /// [`crate::charts::drop_range_attr`] で除去してから合成する。
    pub range: Option<&'a str>,
    /// 非表示系列名の一覧（イシュー #2133）。[`super::data::Series::name`]
    /// と完全一致する系列の `series-line`/`point`/`value-label` へ値なし
    /// 属性 `data-hidden` を付与する。スケール/domain の算出には影響しない
    /// （SSR は全範囲・全系列を出力する設計、モジュール doc参照）。
    /// データに存在しない名前を指定してもエラーにしない（fail-soft）。
    pub hidden_series: &'a [&'a str],
}

impl<'a> LineChartProps<'a> {
    /// 既定寸法（`DEFAULT_WIDTH`/`DEFAULT_HEIGHT`・[`Size::Md`]）・既定
    /// バリアント（Linear/None/None、軸/グリッドなし）で組み立てる。既定値は
    /// #2083 以前の出力と完全に同一の HTML を生成する（golden 純追加原則）。
    #[must_use]
    pub fn new(data: &'a ChartData, aria_label: &'a str) -> Self {
        LineChartProps {
            data,
            aria_label,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            size: Size::Md,
            curve: Curve::default(),
            dots: LineDots::default(),
            label: LineLabel::default(),
            color_by_category: false,
            show_x_axis: false,
            show_y_axis: false,
            show_grid: false,
            show_tooltip: true,
            range: None,
            hidden_series: &[],
        }
    }
}

/// `width`/`height` から `viewBox` を構築する（内部ヘルパ、[`crate::area_chart`]/
/// [`crate::sparkline`] も共有する）。
///
/// [`ViewBoxError`] を [`ChartError`] へ写像する: `NonFinite` →
/// [`ChartError::NonFiniteValue`]、`NonPositiveSize`（幅/高さ 0 以下）→
/// [`ChartError::DegenerateDomain`]（描画領域が退化しているという意味的な
/// 近さから、`charts` 基盤 #846 が定義する既存バリアントを転用する判断。
/// 本クレートは基盤モジュールへ新規バリアントを追加しない）。
pub(crate) fn view_box_from_dims(width: f64, height: f64) -> Result<ViewBox, ChartError> {
    ViewBox::new(0.0, 0.0, width, height).map_err(|e| match e {
        ViewBoxError::NonFinite => ChartError::NonFiniteValue,
        ViewBoxError::NonPositiveSize => ChartError::DegenerateDomain,
    })
}

/// カテゴリ index `i`（`0..n`）を x 座標へ等間隔で写像する（内部ヘルパ、
/// [`crate::area_chart`]/[`crate::sparkline`] も共有する）。
///
/// `n <= 1` の場合は `i * width / (n - 1)` が 0 除算になるため、描画領域の
/// 中央 1 点（`width / 2.0`）へ配置する特別扱いをする（モジュール doc
/// 「x/y 軸の写像規則」参照）。
pub(crate) fn category_x(width: f64, n: usize, i: usize) -> f64 {
    if n <= 1 {
        width / 2.0
    } else {
        (i as f64) * width / ((n - 1) as f64)
    }
}

/// この styled LineChart の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("line-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("--fandhe-line-chart-height", "150px"),
                // イシュー #2129: `tooltip-layer`（`position: absolute`）の
                // 配置規則（#2130 が唯一のロケータとして使う契約）を成立
                // させるための末尾純追加。
                decl("position", "relative"),
            ],
        )
        .base(
            "plot",
            vec![
                decl("display", "block"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-line-chart-height, auto)"),
                // イシュー #1595: SVG 非ルート要素は UA 既定で `overflow: hidden`
                // となるため、domain の max/min に接する折れ線
                // （`stroke-width: 2`）が viewBox 上下端で半分クリップされる。
                // ジオメトリ（`view_box_from_dims`/`category_x`）は変えず、
                // CSS のみで表示上のクリップを解除する（先例: area_chart #1589）。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "series-line",
            vec![
                decl("fill", "none"),
                decl("stroke-width", "2"),
                // イシュー #1595: 先例 signature_pad / progress / area_chart。
                // 折れ線の鋭角部での miter 突出を抑え、端点の見た目を整える。
                decl("stroke-linejoin", "round"),
                decl("stroke-linecap", "round"),
            ],
        )
        .base(
            "point",
            // イシュー #1595: `n == 1` 時の点マーカーに背景色のハローを
            // 付け、背景・隣接系列色との識別性を高める。
            // `--fandhe-color-bg` はダーク時の値へトークン経由で自動追随
            // する（`theme.rs` の DEFAULT_COLORS 選定根拠を参照）。
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        .base(
            "value-label",
            // イシュー #2083: bar_chart `value-label` と同一 3 宣言
            // （shadcn `fill-foreground`/`fontSize 12` 相当）。
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("fill", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #1681: `crate::area_chart::recipe` と同一の高さ値・
        // 導出根拠（差分 54→70 の拡大則を外挿）を共有する。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-line-chart-height", "58px")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-line-chart-height", "96px")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-line-chart-height", "150px")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-line-chart-height", "220px")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-line-chart-height", "306px")],
        )
        .default_variant(Size::Md)
        // イシュー #2083: `dots: LineDots::Hollow` variant は `default_variant`
        // を登録しない（area_chart `AreaFill::Gradient` と同じ判断: 登録すると
        // 全 line-chart の root class に無条件混入し HTML golden が壊れる）。
        // 既存 `point` base の `stroke: var(--fandhe-color-bg)` が presentation
        // 属性の `stroke` に勝つため、輪郭点は variant class + `color`
        // presentation 属性（系列色）+ `currentColor` で表現する
        // （bar_chart `bar[data-active]` と同じ手法）。
        .variant(
            LineDots::Hollow,
            "point",
            vec![
                decl("fill", "var(--fandhe-color-bg)"),
                decl("stroke", "currentColor"),
                decl("stroke-width", "2"),
            ],
        )
        // イシュー #2133: `hidden_series` で指定した系列の描画要素を非表示に
        // する（SSR はスケール/domain を変えず全系列を出力したまま、CSS の
        // みで隠す。末尾純追加、既存ブロックは不変）。
        .state(
            "series-line",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "point",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "value-label",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #2131: hover 強調（減光）の起点。`root` は
        // `tooltip-layer`（`position: absolute`）の配置に必要な
        // `position: relative` を既に持つ祖先であり、ここで
        // `--fandhe-chart-inactive-opacity` を宣言して `point` へ継承
        // させる（`crate::charts::tooltip` モジュール doc「hover 強調」節
        // 参照。wasm-full 側の `data-has-active` 付け外し配線は未実装の
        // フォローアップ）。
        .state(
            "root",
            StateCondition::Attr("data-has-active"),
            vec![decl("--fandhe-chart-inactive-opacity", "0.4")],
        )
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

/// この styled LineChart が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// [`render_series`] の描画パラメータ（内部ヘルパ、イシュー #2083:
/// `curve`/`dots`/`label`/`color_by_category` の追加で引数が増えたため
/// 構造体へまとめる。`recipe`/`categories` は系列を跨いで共有する参照）。
struct SeriesRenderCtx<'a> {
    left: f64,
    curve: Curve,
    dots: LineDots,
    label: LineLabel,
    color_by_category: bool,
    categories: &'a [String],
    recipe: &'a SlotRecipe,
    /// 系列名（[`super::data::Series::name`]）。`show_series_attr` が
    /// `true` のとき `series-line`/`point`/`value-label` へ `data-series`
    /// として付与する（イシュー #2133）。
    series_name: &'a str,
    /// `true` なら `data-series` を付与する（`LineChartProps::show_tooltip`
    /// と同じゲート。イシュー #2129 の契約「`false` の出力は #2129 以前と
    /// バイト一致」を保つため、tooltip 用の `data-index`/`data-series`
    /// 語彙と同じ opt-in にする、イシュー #2133）。
    show_series_attr: bool,
    /// `true` なら `series-line`/`point`/`value-label` へ値なし属性
    /// `data-hidden` を付与する（イシュー #2133、
    /// `LineChartProps::hidden_series` に系列名が含まれる場合）。
    hidden: bool,
}

/// [`SeriesRenderCtx::show_series_attr`]/[`SeriesRenderCtx::hidden`] から
/// `data-series`/`data-hidden` の追加属性列を組み立てる（内部ヘルパ）。
fn series_extra_attrs<'a>(ctx: &SeriesRenderCtx<'a>) -> Vec<(&'a str, &'a str)> {
    let mut extra: Vec<(&str, &str)> = Vec::new();
    if ctx.show_series_attr {
        extra.push(("data-series", ctx.series_name));
    }
    if ctx.hidden {
        extra.push(("data-hidden", ""));
    }
    extra
}

/// データ点 1 個の上へ値/カテゴリラベルを描く（内部ヘルパ、shadcn
/// `LabelList position="top" offset={12}` 相当、イシュー #2083）。
/// `text_value` は [`LineLabel::Value`] なら [`fmt_coord`] 済みの値文字列、
/// [`LineLabel::Category`] ならカテゴリ名（[`text`] ノード経由で既定
/// エスケープを通る）。
fn value_label<'a>(
    x: f64,
    y: f64,
    text_value: String,
    extra_attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut attrs = vec![
        ("data-scope", "line-chart"),
        ("data-part", "value-label"),
        ("text-anchor", "middle"),
    ];
    attrs.extend(extra_attrs);
    svg_text(x, y - LABEL_OFFSET, attrs, vec![text(text_value)])
}

/// 系列 1 本を折れ線 `path`（`n >= 2`）または中央の点マーカー（`n == 1`）
/// として描く（内部ヘルパ）。`color` は呼び出し元（[`line_chart`]）が
/// [`crate::charts::ChartData::series_color_var`] で解決済みの値
/// （系列の色上書き、無ければ [`crate::charts::series_color_var`] の 6 色
/// 循環、イシュー #2077）。`n == 1` の既存マーカーは `ctx.dots ==
/// LineDots::None` でも従来どおり描く（golden 固定、イシュー #2083）。
/// `n == 1` は単一カテゴリの中央点という特別扱いのため、`ctx.dots ==
/// LineDots::Hollow`・`ctx.color_by_category` は適用されず、常に系列色の
/// `fill` 塗り点として描かれる（`n >= 2` の各点にのみ適用される。golden
/// 固定を優先した意図的な非対称、`single_category_renders_point_not_path`
/// テスト参照）。
///
/// # Errors
///
/// `ctx.curve` が [`Curve::Natural`] で自然スプラインの中間計算が桁あふれ
/// した場合 [`ChartError::NonFiniteValue`]（[`curve::line_path_d`] 参照）。
fn render_series(
    width: f64,
    y_scale: &LinearScale,
    values: &[f64],
    color: &str,
    ctx: &SeriesRenderCtx<'_>,
) -> Result<Vec<Node>, ChartError> {
    let n = values.len();
    let mut nodes: Vec<Node> = Vec::new();
    let extra_attrs = series_extra_attrs(ctx);

    if n <= 1 {
        let v = values.first().copied().unwrap_or(0.0);
        let x = category_x(width, n, 0) + ctx.left;
        let y = y_scale.scale(v);
        let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
        let mut point_attrs: Vec<(&str, &str)> = vec![
            ("data-scope", "line-chart"),
            ("data-part", "point"),
            ("cx", cx.as_str()),
            ("cy", cy.as_str()),
            ("r", r.as_str()),
            ("fill", color),
        ];
        point_attrs.extend(extra_attrs.clone());
        nodes.push(el("circle", point_attrs, vec![]));
        if ctx.label != LineLabel::None {
            let text_value = match ctx.label {
                LineLabel::Value => fmt_coord(v),
                LineLabel::Category => ctx.categories.first().cloned().unwrap_or_default(),
                LineLabel::None => unreachable!("上の if で LineLabel::None を除外済み"),
            };
            nodes.push(value_label(x, y, text_value, extra_attrs.clone()));
        }
        return Ok(nodes);
    }

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| (category_x(width, n, i) + ctx.left, y_scale.scale(v)))
        .collect();

    let d = curve::line_path_d(&points, ctx.curve)?;
    let mut line_attrs: Vec<(&str, &str)> = vec![
        ("data-scope", "line-chart"),
        ("data-part", "series-line"),
        ("d", d.as_str()),
        ("stroke", color),
        ("fill", "none"),
    ];
    line_attrs.extend(extra_attrs.clone());
    nodes.push(el("path", line_attrs, vec![]));

    if ctx.dots != LineDots::None {
        for (i, &(x, y)) in points.iter().enumerate() {
            let point_color = if ctx.color_by_category {
                crate::charts::series_color_var(i)
            } else {
                color.to_string()
            };
            let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
            let mut point_attrs: Vec<(&str, &str)> = vec![
                ("data-scope", "line-chart"),
                ("data-part", "point"),
                ("cx", cx.as_str()),
                ("cy", cy.as_str()),
                ("r", r.as_str()),
            ];
            let hollow_class;
            if ctx.dots == LineDots::Hollow {
                hollow_class = ctx.recipe.variant_class(LineDots::Hollow);
                point_attrs.push(("class", hollow_class.as_str()));
                point_attrs.push(("color", point_color.as_str()));
            } else {
                point_attrs.push(("fill", point_color.as_str()));
            }
            point_attrs.extend(extra_attrs.clone());
            nodes.push(el("circle", point_attrs, vec![]));
        }
    }

    if ctx.label != LineLabel::None {
        for (i, &(x, y)) in points.iter().enumerate() {
            let text_value = match ctx.label {
                LineLabel::Value => fmt_coord(values[i]),
                LineLabel::Category => ctx.categories.get(i).cloned().unwrap_or_default(),
                LineLabel::None => unreachable!("上の if で LineLabel::None を除外済み"),
            };
            nodes.push(value_label(x, y, text_value, extra_attrs.clone()));
        }
    }

    Ok(nodes)
}

/// LineChart 本体を組み立てる。
///
/// # Errors
///
/// - `props.width`/`props.height` が非有限の場合 [`ChartError::NonFiniteValue`]、
///   0 以下の場合 [`ChartError::DegenerateDomain`]（[`view_box_from_dims`] 参照）
/// - `props.curve` が [`Curve::Natural`] で自然スプラインの中間計算が桁あふれ
///   した場合 [`ChartError::NonFiniteValue`]（[`curve::line_path_d`] 参照）
/// - `props.show_x_axis`/`props.show_y_axis`/`props.show_grid`/`props.label`
///   のいずれかが有効で、余白差し引き後のプロット領域が 0 以下になる場合
///   [`ChartError::PlotAreaTooSmall`]
///
/// `props.data` は呼び出し側が [`ChartData::new`](crate::charts::data::ChartData::new)
/// を経由して構築済みであるため、上記以外のエラーは発生しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
/// )
/// .unwrap();
/// let node = line_chart(&LineChartProps::new(&data, "monthly visits"), vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="line-chart" data-part="root""#));
/// assert!(html.contains("<path"));
/// ```
///
/// `dots`/`label` を有効化した例（イシュー #2083）:
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps, LineDots, LineLabel};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
/// )
/// .unwrap();
/// let mut props = LineChartProps::new(&data, "monthly visits");
/// props.dots = LineDots::Filled;
/// props.label = LineLabel::Value;
/// let html = render(&line_chart(&props, vec![]).unwrap());
/// assert!(html.contains(r#"data-part="point""#));
/// assert!(html.contains(r#"data-part="value-label""#));
/// ```
pub fn line_chart<'a>(
    props: &LineChartProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, ChartError> {
    let view_box = view_box_from_dims(props.width, props.height)?;

    // 余白（イシュー #2083）: `show_y_axis`/`show_x_axis` は area_chart と
    // 同じ規則で左/下を確保する。`label` は shadcn `margin: { top: 20 }`
    // 相当で上のみ確保する（モジュール doc「意図的に合わせなかった点」の
    // 左右 margin 非採用と対）。
    let left = if props.show_y_axis {
        AXIS_LEFT_MARGIN
    } else {
        0.0
    };
    let bottom = if props.show_x_axis {
        AXIS_BOTTOM_MARGIN
    } else {
        0.0
    };
    let top = if props.label != LineLabel::None {
        LABEL_TOP_MARGIN
    } else {
        0.0
    };
    let plot_w = props.width - left;
    let plot_h = props.height - bottom - top;
    let has_margin = left != 0.0 || bottom != 0.0 || top != 0.0;
    if has_margin && (plot_w <= 0.0 || plot_h <= 0.0) {
        return Err(ChartError::PlotAreaTooSmall);
    }

    let has_axes = props.show_x_axis || props.show_y_axis || props.show_grid;
    let y_scale = if has_axes {
        LinearScale::new(props.data.domain(), (top + plot_h, top))?.nice()
    } else {
        LinearScale::new(props.data.domain(), (top + plot_h, top))?
    };

    let recipe = recipe();
    let mut plot_children: Vec<Node> = Vec::new();

    let ticks = if has_axes {
        y_scale.ticks(4)?
    } else {
        Vec::new()
    };

    if props.show_grid {
        // area_chart と同じヘルパを再利用する（イシュー #2083:
        // `grid_lines_for_ticks` は水平線のみ描くため第 3 引数 `plot_h` は
        // 実質未使用、`top` オフセットは `ticks` を `y_scale.scale` 済みの
        // 座標へ変換する時点で反映済み）。
        plot_children.push(grid_lines_for_ticks(
            left,
            props.width,
            plot_h,
            &y_scale,
            &ticks,
        )?);
    }

    for (i, s) in props.data.series().iter().enumerate() {
        let color = props.data.series_color_var(i);
        let render_ctx = SeriesRenderCtx {
            left,
            curve: props.curve,
            dots: props.dots,
            label: props.label,
            color_by_category: props.color_by_category,
            categories: props.data.categories(),
            recipe: &recipe,
            series_name: s.name.as_str(),
            show_series_attr: props.show_tooltip,
            hidden: props.hidden_series.contains(&s.name.as_str()),
        };
        plot_children.extend(render_series(
            plot_w,
            &y_scale,
            &s.values,
            &color,
            &render_ctx,
        )?);
    }

    if props.show_y_axis {
        plot_children.push(axis::y_axis(
            &y_scale,
            &ticks,
            left,
            &AxisProps {
                show_tick_lines: false,
                show_axis_line: false,
                ..AxisProps::default()
            },
        )?);
    }
    if props.show_x_axis {
        plot_children.extend(x_axis_category_labels(
            props.data.categories(),
            plot_w,
            left,
            top + plot_h,
            props.width,
        )?);
    }

    let entries = if props.show_tooltip {
        Some(tooltip::entries_from_chart_data(props.data))
    } else {
        None
    };
    if let Some(entries) = &entries {
        let n = props.data.categories().len();
        for entry in entries {
            let (band_left, band_right) = category_band(plot_w, n, entry.index);
            let label = tooltip::hit_area_label(entry);
            plot_children.push(tooltip::hit_area_rect(
                left + band_left,
                top,
                band_right - band_left,
                plot_h,
                entry.index,
                None,
                &label,
            ));
        }
    }

    let plot = svg_root(
        &view_box,
        vec![
            ("data-scope", "line-chart"),
            ("data-part", "plot"),
            ("aria-label", props.aria_label),
        ],
        plot_children,
    );

    let mut children = vec![plot];
    if let Some(entries) = &entries {
        children.push(tooltip::layer_from_entries(entries, None));
    }

    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    if let Some(range) = props.range {
        merged.push(("data-range", range));
    }
    merged.extend(drop_range_attr(drop_class_attr(attrs)));
    Ok(ANATOMY.part("root", "div", merged, children))
}

/// カテゴリ `i`（`0..n`）の帯型 hit-area の `[left, right)` を、隣接
/// [`category_x`] の中点で区切って算出する（内部ヘルパ、[`crate::sparkline`]/
/// [`crate::area_chart`] も同型のロジックを持つ）。
fn category_band(width: f64, n: usize, i: usize) -> (f64, f64) {
    let left = if i == 0 {
        0.0
    } else {
        (category_x(width, n, i - 1) + category_x(width, n, i)) / 2.0
    };
    let right = if n == 0 || i + 1 >= n {
        width
    } else {
        (category_x(width, n, i) + category_x(width, n, i + 1)) / 2.0
    };
    (left, right)
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

    /// 系列の [`crate::charts::data::SeriesColor`] 上書き（イシュー #2077）が
    /// 折れ線の `stroke` へ反映されることを固定する（凡例と同一の `var()`
    /// を共有する一元性の確認）。
    #[test]
    fn render_series_reflects_series_color_override() {
        let categories = (0..2).map(|i| i.to_string()).collect();
        let d = ChartData::new(
            categories,
            vec![Series::new("s", vec![1.0, 2.0])
                .with_color(crate::charts::SeriesColor::token("neutral").unwrap())],
        )
        .unwrap();
        let node = line_chart(&LineChartProps::new(&d, "sample"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"stroke="var(--fandhe-color-neutral)""#));
    }

    #[test]
    fn renders_root_and_plot_with_aria_label() {
        let d = data(vec![1.0, 2.0, 3.0]);
        let node = line_chart(&LineChartProps::new(&d, "sample chart"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.starts_with(r#"<div data-scope="line-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="line-chart" data-part="plot""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="sample chart""#));
    }

    #[test]
    fn single_category_renders_point_not_path() {
        let d = data(vec![5.0]);
        let node = line_chart(&LineChartProps::new(&d, "single"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="point""#));
        assert!(!html.contains("<path"));
    }

    #[test]
    fn multi_category_renders_path_not_point() {
        let d = data(vec![1.0, 2.0]);
        let node = line_chart(&LineChartProps::new(&d, "multi"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains("<path"));
        assert!(!html.contains(r#"data-part="point""#));
    }

    #[test]
    fn negative_values_produce_negative_coordinates_deterministically() {
        let d = data(vec![-10.0, 0.0, 10.0]);
        let node = line_chart(&LineChartProps::new(&d, "negative"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains('-'));
    }

    #[test]
    fn flat_data_renders_horizontal_line() {
        let d = data(vec![5.0, 5.0, 5.0]);
        let node1 = line_chart(&LineChartProps::new(&d, "flat"), vec![]).unwrap();
        let node2 = line_chart(&LineChartProps::new(&d, "flat"), vec![]).unwrap();
        assert_eq!(render(&node1), render(&node2));
    }

    #[test]
    fn same_input_produces_same_output_deterministically() {
        let d = data(vec![1.0, 4.0, 2.0, 8.0]);
        let a = render(&line_chart(&LineChartProps::new(&d, "det"), vec![]).unwrap());
        let b = render(&line_chart(&LineChartProps::new(&d, "det"), vec![]).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn non_finite_width_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "bad");
        props.width = f64::NAN;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn non_positive_height_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "bad");
        props.height = 0.0;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::DegenerateDomain
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "class-test");
        props.size = Size::Lg;
        let html = render(&line_chart(&props, vec![("class", "attacker")]).unwrap());
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker"));
        assert!(html.contains("fd-line-chart--size-lg"));
    }

    #[test]
    fn xss_payload_in_aria_label_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><img src=x onerror=alert(1)>";
        let node = line_chart(&LineChartProps::new(&d, payload), vec![]).unwrap();
        let html = render(&node);
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><script>alert(1)</script>";
        let html = render(
            &line_chart(
                &LineChartProps::new(&d, "attrs-test"),
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
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="plot"]"#));
        assert!(a.contains("overflow: visible"));
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="series-line"]"#));
        assert!(a.contains("stroke-linejoin: round"));
        assert!(a.contains("stroke-linecap: round"));
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="point"]"#));
        assert!(a.contains("stroke: var(--fandhe-color-bg)"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // イシュー #2083（shadcn/ui Charts（line）突合）: 既定 props（Linear/
    // None/None、軸/グリッドなし）は #2083 以前の出力と完全に同一の HTML を
    // 生成する（golden 純追加原則）ことを固定する。
    #[test]
    fn default_props_output_matches_pre_2083_baseline() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let html = render(&line_chart(&LineChartProps::new(&d, "baseline"), vec![]).unwrap());
        assert!(!html.contains(r#"data-part="value-label""#));
        assert!(!html.contains("dots-hollow"));
        assert!(!html.contains(r#"data-part="grid-line""#));
        assert!(!html.contains(r#"data-part="y-axis""#));
        assert!(!html.contains(r#"data-part="axis-line""#));
    }

    #[test]
    fn curve_natural_produces_cubic_segments() {
        let d = data(vec![1.0, 4.0, 2.0, 8.0]);
        let mut props = LineChartProps::new(&d, "natural");
        props.curve = Curve::Natural;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains(" C"));
    }

    #[test]
    fn curve_natural_two_points_degenerates_to_line() {
        let d = data(vec![1.0, 4.0]);
        let mut props = LineChartProps::new(&d, "natural-2");
        props.curve = Curve::Natural;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(!html.contains(" C"));
        assert!(html.contains(" L"));
    }

    #[test]
    fn curve_step_produces_expected_line_to_count() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "step");
        props.curve = Curve::Step;
        let html = render(&line_chart(&props, vec![]).unwrap());
        // step_points: 2 区間 * 2 点 + 終点 1 = 5 点、先頭は move_to のため
        // `L` は 5 回。
        assert_eq!(html.matches(" L").count(), 5);
    }

    #[test]
    fn dots_filled_renders_point_per_category() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "dots");
        props.dots = LineDots::Filled;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="point""#).count(), 3);
        assert!(!html.contains("dots-hollow"));
    }

    #[test]
    fn dots_hollow_uses_variant_class_and_color_attr_without_fill() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "dots-hollow");
        props.dots = LineDots::Hollow;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains("fd-line-chart--dots-hollow"));
        assert!(html.contains("color=\"var(--fandhe-color-"));
        // Hollow 点は `fill` presentation 属性を持たない（CSS の
        // variant 宣言 `fill: var(--fandhe-color-bg)` に任せる）。
        let point_start = html.find(r#"data-part="point""#).unwrap();
        let point_tag_end = html[point_start..].find('>').unwrap() + point_start;
        assert!(!html[point_start..point_tag_end].contains("fill="));
    }

    #[test]
    fn color_by_category_uses_category_index_color_for_second_point() {
        let d = data(vec![1.0, 4.0]);
        let mut props = LineChartProps::new(&d, "color-by-category");
        props.dots = LineDots::Filled;
        props.color_by_category = true;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains("chart-2"));
    }

    #[test]
    fn label_value_renders_formatted_value_text() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "label-value");
        props.label = LineLabel::Value;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="value-label""#).count(), 3);
        assert!(html.contains(">4<"));
    }

    #[test]
    fn label_category_renders_category_text() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "label-category");
        props.label = LineLabel::Category;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains(">1<"));
    }

    #[test]
    fn axes_and_grid_render_expected_parts() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "axes");
        props.show_x_axis = true;
        props.show_y_axis = true;
        props.show_grid = true;
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"data-part="grid-line""#));
        assert!(html.contains(r#"data-part="y-axis""#));
        assert!(html.contains(r#"data-part="tick-label""#));
        assert!(html.contains(r#"data-part="axis-line""#));
    }

    #[test]
    fn axes_reject_when_plot_area_too_small() {
        let d = data(vec![1.0, 4.0]);
        let mut props = LineChartProps::new(&d, "too-small");
        props.width = 10.0;
        props.show_y_axis = true;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn curve_natural_extreme_width_returns_non_finite_value_error() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "extreme");
        props.curve = Curve::Natural;
        props.width = 4e307;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn line_chart_never_emits_data_attrs_beyond_scope_and_part() {
        let d = data(vec![1.0, 4.0, 2.0]);
        let mut props = LineChartProps::new(&d, "vocab");
        props.dots = LineDots::Filled;
        props.label = LineLabel::Value;
        props.show_x_axis = true;
        props.show_y_axis = true;
        props.show_grid = true;
        let html = render(&line_chart(&props, vec![]).unwrap());
        // `color` は presentation 属性であり `data-*` 語彙ではないため対象外。
        assert!(!html.contains("data-state"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-active"));
    }

    #[test]
    fn stylesheet_includes_value_label_and_dots_hollow_selectors() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="line-chart"][data-part="value-label"]"#));
        assert!(css.contains(
            r#"[data-scope="line-chart"][data-part="point"].fd-line-chart--dots-hollow"#
        ));
    }

    // イシュー #2133: 期間切替・凡例トグルの SSR 構造。

    #[test]
    fn range_none_omits_data_range() {
        let d = data(vec![1.0, 2.0]);
        let html = render(&line_chart(&LineChartProps::new(&d, "range"), vec![]).unwrap());
        assert!(!html.contains("data-range"));
    }

    #[test]
    fn range_some_emits_data_range_on_root() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "range");
        props.range = Some("90d");
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.starts_with(
            r#"<div data-scope="line-chart" data-part="root" class="fd-line-chart--size-md" data-range="90d">"#
        ));
    }

    #[test]
    fn caller_data_range_attr_is_dropped_not_duplicated() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "range");
        props.range = Some("real");
        let html = render(&line_chart(&props, vec![("data-range", "fake")]).unwrap());
        assert_eq!(html.matches("data-range").count(), 1);
        assert!(html.contains(r#"data-range="real""#));
        assert!(!html.contains("fake"));
    }

    #[test]
    fn hidden_series_adds_data_hidden_to_matching_series_elements_only() {
        let d = ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![
                Series::new("visits", vec![1.0, 2.0]),
                Series::new("signups", vec![3.0, 4.0]),
            ],
        )
        .unwrap();
        let mut props = LineChartProps::new(&d, "hidden");
        props.hidden_series = &["signups"];
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"data-series="visits""#));
        assert!(html.contains(r#"data-series="signups" data-hidden="""#));
        // visits 側の要素には data-hidden が付かないことを固定する。
        let visits_path_start = html.find(r#"data-series="visits""#).unwrap();
        let visits_fragment = &html[visits_path_start..visits_path_start + 40];
        assert!(!visits_fragment.contains("data-hidden"));
    }

    #[test]
    fn hidden_series_unknown_name_is_fail_soft() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "hidden");
        props.hidden_series = &["does-not-exist"];
        let result = line_chart(&props, vec![]);
        assert!(result.is_ok());
        let html = render(&result.unwrap());
        assert!(!html.contains("data-hidden"));
    }

    #[test]
    fn hidden_series_does_not_change_geometry() {
        // SSR はスケール/domain を hidden_series の有無で変えない（モジュール
        // doc「スケール」節）。座標（`d=`/`cx=`/`cy=`）はビットごとに同一で、
        // 差分は `data-hidden` 属性のみになることを固定する。
        let d = data(vec![1.0, 4.0, 2.0]);
        let baseline = render(&line_chart(&LineChartProps::new(&d, "geo"), vec![]).unwrap());
        let mut props = LineChartProps::new(&d, "geo");
        props.hidden_series = &["s"];
        let hidden = render(&line_chart(&props, vec![]).unwrap());
        let baseline_no_series_attr = baseline.replace(r#" data-series="s""#, "");
        let hidden_no_attrs = hidden
            .replace(r#" data-series="s""#, "")
            .replace(r#" data-hidden="""#, "");
        assert_eq!(baseline_no_series_attr, hidden_no_attrs);
    }
}
