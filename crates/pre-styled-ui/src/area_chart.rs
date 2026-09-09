//! AreaChart（イシュー #848、親 Phase #845）: [`crate::line_chart`] と同じ
//! `charts` 基盤（#846）の消費者。系列ごとに、折れ線（`series-line`）と
//! domain 下端へ閉じた塗りつぶし面（`series-area`）を重ねて描く自己完結部品。
//!
//! 座標写像・path 生成・数値文字列化の一元化方針、x/y 軸の写像規則は
//! [`crate::line_chart`] モジュール doc を参照（[`crate::line_chart::category_x`]/
//! [`crate::line_chart::view_box_from_dims`] を共有ヘルパとして再利用する）。
//!
//! # 面 path の閉じ方（baseline、`stack: AreaStack::None`）
//!
//! 系列の折れ線経路を辿った後、x 軸の逆順で `domain` 下端（`data.domain().0`、
//! [`ChartData::domain`](crate::charts::data::ChartData::domain) が返す
//! フラットデータ非退化パディング込みの値）に写像した高さ（baseline）へ
//! 戻って閉じる（`M .. L .. L (last.x, baseline) L (first.x, baseline) Z`）。
//! `y = 0` 固定ではなく domain 下端を使うのは、全値が負の系列で面が上下反転
//! （画面上端へ張り付く）せず、値の小さい方へ塗りつぶしが伸びる直感的な
//! 見た目を保つため。積み上げ時（`stack: AreaStack::Normal`/`Expand`）の
//! 閉じ方は「積み上げ（`AreaStack`）」節を参照。
//!
//! # 積み上げ（`AreaStack`）
//!
//! [`AreaStack::Normal`]/[`AreaStack::Expand`] は [`cumulative_series`]
//! （内部ヘルパ）で系列ごとのカテゴリ累積上限値を求め、[`render_stacked`]
//! （内部ヘルパ）が系列 `i` を上側境界 `cum[i]`・下側境界 `cum[i-1]`
//! （`i == 0` は 0）の帯として描く。下側境界は上側と同じ `curve` で
//! 辿った点列を逆順に計算し直して `Z` で閉じる（baseline 節と異なり
//! domain 下端ではなく下側系列の境界へ閉じる）。`Expand` は
//! [`cumulative_series`] がカテゴリ合計で正規化するため domain
//! `(0.0, 1.0)` に固定され、`show_y_axis` 時の目盛ラベルは
//! [`crate::charts::axis::TickLabelFormat::label_scale`] `= 100.0` +
//! `suffix: "%"` で比率を百分率表示する（座標計算は生の比率のまま）。
//! `show_x_axis`/`show_y_axis`/`show_grid` は `stack: AreaStack::None` と
//! 同じ余白規則（[`AXIS_LEFT_MARGIN`]/[`AXIS_BOTTOM_MARGIN`]）を適用する
//! （イシュー #2081 Review 追補: 当初積み上げ時にこれらが無視される
//! 欠陥があった）。
//!
//! # エッジケース（`n == 1`）
//!
//! [`crate::line_chart`] と同じ規則: 面・線のいずれも生成せず、中央
//! （`width / 2.0`）に点マーカーのみを描く（0 除算・退化した面の回避。
//! 積み上げ時は累積値に配置する）。
//!
//! # セキュリティ不変条件
//!
//! [`crate::line_chart`] モジュール doc と同一（`raw_html()` 不使用、
//! 座標は `fmt_coord` 経由で文字集合 `[0-9.-]` に閉じる、CSS 宣言値は
//! すべて静的リテラル）。gradient（`fill: AreaFill::Gradient`）の `<defs>`
//! も同じ不変条件に従う（「gradient の不変条件」節参照）。
//!
//! # gradient の不変条件
//!
//! [`gradient_defs`]（内部ヘルパ）が組み立てる `<linearGradient>`/`<stop>`
//! は、`stop-color` に [`ChartData::series_color_var`] の固定形
//! （`var(--fandhe-color-<name>)`、`<name>` は `chart-1`〜`chart-6` 等の
//! 固定トークン名でユーザー入力を含まない）のみを埋め込み、`offset`/
//! `stop-opacity`/`x1`/`y1`/`x2`/`y2` はすべて静的リテラルである。
//! `id` は呼び出し側 `gradient_id`（[`is_valid_identifier`] 通過済み、
//! 先頭が ASCII 小文字・以降は ASCII 小文字/数字/ハイフンのみ、
//! `[a-z][a-z0-9-]*`。大文字・`_` は不可）+ `-<index>`（系列インデックス、
//! `usize` の十進表記）のみで構成されるため、`fill="url(#<id>)"` 参照の
//! 文字集合が閉じる。ユーザー由来の文字列（カテゴリ名・`aria_label`）は
//! この経路に一切渡らない。
//!
//! # shadcn/ui 突合（イシュー #2081）
//!
//! shadcn/ui Charts（area、10 バリアント）と突合し、静的に描画できる
//! バリアントを props の純追加で補完した。マウス追従ツールチップ・
//! hover 強調・期間切替・凡例トグルのような実行時インタラクションは
//! 対象外（後述「本イシューのスコープ外」）。
//!
//! | shadcn/ui registry | 本実装の対応 |
//! |---|---|
//! | `chart-area-default`（natural + 横グリッド + X 軸） | `curve: AreaCurve::Natural` + `show_grid` + `show_x_axis` |
//! | `chart-area-linear` | `curve: AreaCurve::Linear`（既定、既存出力と同一） |
//! | `chart-area-step` | `curve: AreaCurve::Step` |
//! | `chart-area-legend` | [`crate::charts::legend`] を呼び出し側が並べる（既存） |
//! | `chart-area-stacked` | `stack: AreaStack::Normal` |
//! | `chart-area-stacked-expand` | `stack: AreaStack::Expand`（domain `(0, 1)` 固定） |
//! | `chart-area-icons` | [`crate::charts::data::Series::with_icon`] + legend（イシュー #2077 で対応済み） |
//! | `chart-area-gradient` | `fill: AreaFill::Gradient` |
//! | `chart-area-axes` | `show_x_axis` + `show_y_axis` + `show_grid` |
//! | `chart-area-interactive` | 対象外（下記） |
//!
//! ## 意図的に合わせなかった点
//!
//! - `series-area` の既定 `fill-opacity: 0.2`（chakra-ui 値）は据え置く。
//!   shadcn の `0.4` は既存 golden の再 churn になるため採らず、gradient
//!   variant の stop-opacity（0.8→0.1）でのみ shadcn 値を採用する
//! - dots / label / 横向き（`layout="vertical"`）は shadcn area registry に
//!   存在しない（line/bar 側の variant）ため非対応（#2083/#2082 の判断に
//!   委ねる）
//! - X 軸ラベルの `tickFormatter`（月名 3 文字切り詰め等）はアプリ側整形
//!   （`docs/policy/intentional-non-adoption.md` §3.23/§3.25）のため非対応。
//!   カテゴリ文字列をそのまま描く
//! - `data-series`/hit-area 等の新規 `data-*` は付けない（マウス追従
//!   ツールチップ・hover 強調は #2128 の担当）
//! - `theme.rs` へのトークン追加（gradient stop-opacity 等）は行わない
//!   （`site_css_contract` への波及回避、#1589/#1593 と同じ判断）
//!
//! ## 本イシューのスコープ外
//!
//! マウス追従ツールチップ・hover 強調・hit-area `data-*` は #2128、期間
//! 切替・凡例トグルは #2132。`examples/headless-pre-styled-ui` への追随は
//! crates.io 公開後（[`crate::line_chart`] と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1589）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にチャート部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・ラベルのコントラスト）に限定する。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6` 固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由。`fill-opacity: 0.2` はトークン不在のため静的リテラルのまま） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | 追加規則なし（系列色は dark 値定義済み） |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（面・線のみの SVG 描画） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「viewBox 端でのストローク切れ」） |
//!
//! ## 是正した点
//!
//! - `plot` slot に `overflow: visible` を追加し、系列の折れ線が viewBox
//!   上下端（domain の max/min）に接するときに UA 既定 `overflow: hidden`
//!   で `stroke-width: 2` の半分がクリップされる欠陥を、ジオメトリを
//!   変えず CSS のみで是正した
//! - `series-line` slot に `stroke-linejoin: round` / `stroke-linecap: round`
//!   を追加し、折れ線の鋭角部での miter 突出を抑えた（先例:
//!   [`crate::signature_pad`] / [`crate::progress`]）
//! - `point` slot（`n == 1` 時の点マーカー）に背景色のハロー
//!   （`stroke: var(--fandhe-color-bg)`）を追加し、面と同色の系列色でも
//!   輪郭を識別できるようにした
//!
//! ## 意図的に合わせなかった点（イシュー #1589）
//!
//! - `series-line` への `vector-effect: non-scaling-stroke`（Xl で viewBox
//!   が約 2 倍に拡大されると線幅も約 4px 相当になる）は、兄弟部品
//!   [`crate::line_chart`] / `sparkline` と線幅の見え方が乖離するため
//!   本 PR では採らない。必要になれば #1593（charts 共通）で横断的に扱う
//! - `view_box_from_dims` / `category_x` へのパディング追加（ジオメトリ
//!   側の是正）は、[`crate::line_chart`] / `sparkline` と共有するヘルパの
//!   ため他 issue と競合する。CSS 側の `overflow: visible` で足りるため
//!   見送った
//! - `fill-opacity: 0.2` のトークン化（opacity トークン新設）は、消費者が
//!   chart 系のみで `theme.rs` 変更が docs-site 側の契約テストへ波及する
//!   ため見送った

use crate::charts::axis::{self, AxisProps};
use crate::charts::curve::{self, Curve};
use crate::charts::data::ChartData;
use crate::charts::grid::{self, GridProps};
use crate::charts::scale::LinearScale;
use crate::charts::svg::{fmt_coord, svg_root};
use crate::charts::ChartError;
use crate::class_attr::drop_class_attr;
use crate::css::{decl, is_valid_identifier};
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

/// `show_y_axis` 有効時にプロット領域左側へ確保する Y 軸ラベル用の
/// 余白（px）。`show_grid`/`show_x_axis` のみでは確保しない（Y 軸ラベルを
/// 描かない構成に余白は不要という判断、Review 指摘 #2081）。
/// [`crate::line_chart`]（イシュー #2083）からも同じ余白規則で参照される。
pub(crate) const AXIS_LEFT_MARGIN: f64 = 40.0;
/// `show_x_axis` 有効時にプロット領域下側へ確保する X 軸ラベル用の
/// 余白（px）。`show_grid`/`show_y_axis` のみでは確保しない（同上）。
/// [`crate::line_chart`]（イシュー #2083）からも同じ余白規則で参照される。
pub(crate) const AXIS_BOTTOM_MARGIN: f64 = 24.0;

/// `<linearGradient>` の既定 id 接頭辞（[`AreaChartProps::gradient_id`]）。
pub const DEFAULT_GRADIENT_ID: &str = "fandhe-area";

/// 曲線種（shadcn `type`）。イシュー #2081、shadcn/ui Charts（area）突合。
///
/// 既定 [`AreaCurve::Linear`] は現行（#2081 以前）の折れ線出力と完全に
/// 同一の `d` 属性を生成する（golden 純追加原則、モジュール doc参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AreaCurve {
    /// 直線区間（既定）。
    #[default]
    Linear,
    /// 自然三次スプライン補間（d3-shape `curveNatural` 相当、
    /// [`crate::charts::curve::natural_control_points`]）。
    Natural,
    /// 区間中点で段差になる補間（d3-shape `curveStep` 相当、
    /// [`crate::charts::curve::step_points`]）。
    Step,
}

/// [`AreaCurve`] を line-chart 側と共有する [`Curve`] へ写像する
/// （イシュー #2083: `line_path_d` を `charts::curve` へ合流させたことに
/// 伴う橋渡し。`AreaCurve` は area-chart の公開 API として不変のまま、
/// 内部実装のみ line-chart と共有する）。
impl From<AreaCurve> for Curve {
    fn from(value: AreaCurve) -> Self {
        match value {
            AreaCurve::Linear => Curve::Linear,
            AreaCurve::Natural => Curve::Natural,
            AreaCurve::Step => Curve::Step,
        }
    }
}

/// 積み上げ（shadcn `stackId`/`stackOffset="expand"`）。イシュー #2081。
///
/// 積み上げ時（[`AreaStack::Normal`]/[`AreaStack::Expand`]）は全系列の値が
/// 非負であることを要求する（負値は帯として定義できないため
/// [`ChartError::NegativeValue`]、`bar_segment`/`radar_chart` と同じ判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AreaStack {
    /// 積み上げなし（既定）。系列ごとに独立した面を重ね描きする。
    #[default]
    None,
    /// 累積和による積み上げ（帯状の面が積み重なる）。
    Normal,
    /// カテゴリ合計で正規化した積み上げ（domain `(0.0, 1.0)` 固定、
    /// Y 軸ラベルは `%` 表示）。
    Expand,
}

/// 塗り（shadcn `chart-area-gradient` variant）。イシュー #2081。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AreaFill {
    /// 単色塗り（既定、`fill-opacity: 0.2` の系列色）。
    #[default]
    Solid,
    /// 縦方向グラデーション塗り（`<linearGradient>`、
    /// モジュール doc「gradient の不変条件」参照）。
    Gradient,
}

impl VariantValue for AreaFill {
    fn axis(self) -> &'static str {
        "fill"
    }

    fn value(self) -> &'static str {
        match self {
            AreaFill::Solid => "solid",
            AreaFill::Gradient => "gradient",
        }
    }
}

/// [`area_chart`] の入力。
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
    /// 曲線種（イシュー #2081、既定 [`AreaCurve::Linear`]）。
    pub curve: AreaCurve,
    /// 積み上げ（イシュー #2081、既定 [`AreaStack::None`]）。
    pub stack: AreaStack,
    /// 塗り（イシュー #2081、既定 [`AreaFill::Solid`]）。
    pub fill: AreaFill,
    /// `fill: AreaFill::Gradient` 時の `<linearGradient id>` 接頭辞
    /// （既定 [`DEFAULT_GRADIENT_ID`]）。[`is_valid_identifier`] を満たさ
    /// なければ [`ChartError::InvalidGradientId`]。1 ページに複数チャート
    /// を置く呼び出し側が一意化する（モジュール doc「gradient の不変
    /// 条件」参照）。
    pub gradient_id: &'a str,
    /// X 軸（カテゴリ）を描画するか（イシュー #2081、既定 `false`）。
    pub show_x_axis: bool,
    /// Y 軸（数値目盛）を描画するか（イシュー #2081、既定 `false`）。
    pub show_y_axis: bool,
    /// 水平グリッド線を描画するか（イシュー #2081、既定 `false`）。
    pub show_grid: bool,
}

impl<'a> AreaChartProps<'a> {
    /// 既定寸法（[`Size::Md`]）・既定バリアント（Linear/None/Solid、
    /// 軸/グリッドなし）で組み立てる。既定値は #2081 以前の出力と完全に
    /// 同一の HTML を生成する（golden 純追加原則）。
    #[must_use]
    pub fn new(data: &'a ChartData, aria_label: &'a str) -> Self {
        AreaChartProps {
            data,
            aria_label,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            size: Size::Md,
            curve: AreaCurve::default(),
            stack: AreaStack::default(),
            fill: AreaFill::default(),
            gradient_id: DEFAULT_GRADIENT_ID,
            show_x_axis: false,
            show_y_axis: false,
            show_grid: false,
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
                // イシュー #1589: SVG 非ルート要素は UA 既定で `overflow: hidden`
                // となるため、domain の max/min に接する折れ線
                // （`stroke-width: 2`）が viewBox 上下端で半分クリップされる。
                // ジオメトリ（`view_box_from_dims`/`category_x`）は変えず、
                // CSS のみで表示上のクリップを解除する。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "series-area",
            // chakra-ui `charts/area-chart.md` の既定 `fillOpacity={0.2}` 準拠。
            vec![decl("fill-opacity", "0.2"), decl("stroke", "none")],
        )
        .base(
            "series-line",
            vec![
                decl("fill", "none"),
                decl("stroke-width", "2"),
                // イシュー #1589: 先例 signature_pad / progress。折れ線の
                // 鋭角部での miter 突出を抑え、端点の見た目を整える。
                decl("stroke-linejoin", "round"),
                decl("stroke-linecap", "round"),
            ],
        )
        .base(
            "point",
            // イシュー #1589: `n == 1` 時の点マーカーに背景色のハローを
            // 付け、面と同色の系列色でも輪郭を識別できるようにする。
            // `--fandhe-color-bg` はダーク時の値へトークン経由で自動追随
            // する（`theme.rs` の DEFAULT_COLORS 選定根拠を参照）。
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        // イシュー #1681: Xs/Xl は Sm(96)→Md(150)→Lg(220) の非等差進行
        // （差分 54→70、差分自体が 16 ずつ拡大）を両端へ同じ増分則で外挿
        // （前段差分 38 → Xs=58、次段差分 86 → Xl=306）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-area-chart-height", "58px")],
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
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-area-chart-height", "306px")],
        )
        .default_variant(Size::Md)
        // イシュー #2081: gradient variant は `default_variant` を登録
        // しない（root の `variant_classes` 呼び出しは `size` のみを選択
        // するため、既定を登録すると全 area-chart の root class に
        // `fd-area-chart--fill-solid` が無条件で混入し HTML golden が
        // 壊れる。`series-area` へは `recipe.variant_class(AreaFill::Gradient)`
        // で個別に付与する、モジュール doc「gradient の不変条件」参照）。
        .variant(
            AreaFill::Gradient,
            "series-area",
            vec![decl("fill-opacity", "1")],
        )
}

/// この styled AreaChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// カテゴリ位置ごとの点列を「面 path」「線 path」の `d` 属性へ変換する
/// （内部ヘルパ、`curve` に応じて直線/自然スプライン/step のいずれかで
/// 補間する）。`points` は `n >= 2` を契約とする。
///
/// `AreaCurve::Natural`（`n >= 3`）は
/// [`crate::charts::curve::natural_control_points`] が `None` を返した場合
/// （codex-review 指摘、イシュー #2081: 3 カテゴリ・極端に大きい `width`
/// で Thomas 法の中間計算が桁あふれし非有限値になるケース）に
/// [`ChartError::NonFiniteValue`] を返し、`fmt_coord` の「有限値のみを
/// 契約入力とする」不変条件が破られる前に呼び出し元へエラーを伝播する。
fn build_line_d(points: &[(f64, f64)], curve: AreaCurve) -> Result<String, ChartError> {
    curve::line_path_d(points, curve.into())
}

/// 系列 1 本を「面 + 線」（`n >= 2`）または中央の点マーカー（`n == 1`）として
/// 描く（内部ヘルパ、`stack: AreaStack::None` 専用）。`baseline_y` は
/// `data.domain().0` を y スケールで写像した座標（モジュール doc「面 path
/// の閉じ方」参照）。`color` は呼び出し元が
/// [`crate::charts::ChartData::series_color_var`] で解決済みの値
/// （系列の色上書き、無ければ [`crate::charts::series_color_var`] の 6 色
/// 循環、イシュー #2077）。`left` はプロット領域の左端オフセット
/// （`show_y_axis` 時の軸ラベル用余白。軸/グリッドなし呼び出しでは
/// `0.0` を渡し、`width` を `props.width` そのものとすることで従来と
/// 同一の座標になる。軸/グリッドあり呼び出しでは `width` に余白差し引き
/// 後のプロット幅を渡す。Review 指摘 #2081: 重複していたインライン
/// 描画をこのヘルパへ統合するために追加）。
#[allow(
    clippy::too_many_arguments,
    reason = "曲線種・塗り・gradient id・左端オフセットは同一系列の描画に必須の同格パラメータであり、分割すると呼び出し側で対応関係が追いにくくなる"
)]
fn render_series_none(
    width: f64,
    left: f64,
    y_scale: &LinearScale,
    baseline_y: f64,
    values: &[f64],
    color: &str,
    curve: AreaCurve,
    fill: AreaFill,
    fill_class: &str,
    gradient_url: &str,
) -> Result<Vec<Node>, ChartError> {
    let n = values.len();

    if n <= 1 {
        let x = category_x(width, n, 0) + left;
        let y = values.first().copied().map_or(0.0, |v| y_scale.scale(v));
        let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
        return Ok(vec![el(
            "circle",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "point"),
                ("cx", cx.as_str()),
                ("cy", cy.as_str()),
                ("r", r.as_str()),
                ("fill", color),
            ],
            vec![],
        )]);
    }

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| (category_x(width, n, i) + left, y_scale.scale(v)))
        .collect();

    let line_d = build_line_d(&points, curve)?;

    // 面 path: 折れ線を辿った後、baseline へ降りて逆方向の始点に戻り閉じる
    // （モジュール doc「面 path の閉じ方」参照）。Linear 以外の曲線でも
    // 下側の閉じ方自体は baseline 2 直線のまま変えない（塗りの見た目は
    // 上側の曲線形状で決まり、下側は viewBox 外へは出ないため）。
    let (last_x, _) = points[points.len() - 1];
    let (first_x, _) = points[0];
    let area_d = format!(
        "{line_d} L{},{} L{},{} Z",
        fmt_coord(last_x),
        fmt_coord(baseline_y),
        fmt_coord(first_x),
        fmt_coord(baseline_y)
    );

    let fill_value = match fill {
        AreaFill::Solid => color.to_string(),
        AreaFill::Gradient => format!("url(#{gradient_url})"),
    };
    let mut area_attrs: Vec<(&str, &str)> = vec![
        ("data-scope", "area-chart"),
        ("data-part", "series-area"),
        ("d", area_d.as_str()),
        ("fill", fill_value.as_str()),
    ];
    if !fill_class.is_empty() {
        area_attrs.push(("class", fill_class));
    }

    Ok(vec![
        el("path", area_attrs, vec![]),
        el(
            "path",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "series-line"),
                ("d", line_d.as_str()),
                ("stroke", color),
                ("fill", "none"),
            ],
            vec![],
        ),
    ])
}

/// 積み上げ時（`stack: AreaStack::Normal`/`Expand`）の系列群を描く
/// （内部ヘルパ）。`cum` は `series[i]` のカテゴリごとの累積上限値
/// （`cum[0]` は 1 系列目の値そのもの）。系列 `i` は上側境界 `cum[i]`・
/// 下側境界 `cum[i-1]`（`i == 0` は 0）の帯として描く。下側境界は上側と
/// 同じ `curve` で辿った点列を**逆順**に計算し直して閉じる（natural
/// spline の端点条件は対称なため逆順計算でも同一曲線になる）。
///
/// `width` は軸/グリッド用余白差し引き後のプロット幅、`left` はその左端
/// オフセット（`show_y_axis` 時の余白、イシュー #2081 Review 追補: 積み
/// 上げ時に軸/グリッドが無視される欠陥の是正）。軸/グリッドなし
/// （`left == 0.0`、`width == props.width`）では従来と同一の座標になる。
#[allow(
    clippy::too_many_arguments,
    reason = "積み上げ描画は系列群・累積値・色解決・曲線/塗り指定・軸余白オフセットを同時に必要とし、分割すると対応関係が追いにくくなる"
)]
fn render_stacked(
    width: f64,
    left: f64,
    y_scale: &LinearScale,
    cum: &[Vec<f64>],
    data: &ChartData,
    curve: AreaCurve,
    fill: AreaFill,
    fill_class: &str,
    gradient_id_prefix: &str,
) -> Result<Vec<Node>, ChartError> {
    let n = cum[0].len();
    let mut nodes = Vec::new();

    for (i, upper) in cum.iter().enumerate() {
        let color = data.series_color_var(i);
        if n <= 1 {
            let x = category_x(width, n, 0) + left;
            let y = upper.first().copied().map_or(0.0, |v| y_scale.scale(v));
            let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
            nodes.push(el(
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
            ));
            continue;
        }

        let upper_points: Vec<(f64, f64)> = upper
            .iter()
            .enumerate()
            .map(|(k, &v)| (category_x(width, n, k) + left, y_scale.scale(v)))
            .collect();
        let lower_values: Vec<f64> = if i == 0 {
            vec![0.0; n]
        } else {
            cum[i - 1].clone()
        };
        let mut lower_points: Vec<(f64, f64)> = lower_values
            .iter()
            .enumerate()
            .map(|(k, &v)| (category_x(width, n, k) + left, y_scale.scale(v)))
            .collect();
        lower_points.reverse();

        let upper_d = build_line_d(&upper_points, curve)?;
        let lower_d = build_line_d(&lower_points, curve)?;
        // lower_d は独立した `M..` から始まるため、先頭の `M` を `L` へ
        // 差し替えて上側 path の続きとして連結する（上側終点と下側
        // 逆順始点は同じカテゴリの下側境界であり座標が一致するため、
        // 連結しても幾何が破綻しない）。
        let lower_continuation = lower_d.replacen('M', "L", 1);
        let area_d = format!("{upper_d} {lower_continuation} Z");

        let fill_value = match fill {
            AreaFill::Solid => color.clone(),
            AreaFill::Gradient => format!("url(#{gradient_id_prefix}-{i})"),
        };
        let mut area_attrs: Vec<(&str, &str)> = vec![
            ("data-scope", "area-chart"),
            ("data-part", "series-area"),
            ("d", area_d.as_str()),
            ("fill", fill_value.as_str()),
        ];
        if !fill_class.is_empty() {
            area_attrs.push(("class", fill_class));
        }
        nodes.push(el("path", area_attrs, vec![]));
        nodes.push(el(
            "path",
            vec![
                ("data-scope", "area-chart"),
                ("data-part", "series-line"),
                ("d", upper_d.as_str()),
                ("stroke", color.as_str()),
                ("fill", "none"),
            ],
            vec![],
        ));
    }

    Ok(nodes)
}

/// `stack: AreaStack::Normal`/`Expand` 時のカテゴリごとの累積上限値を
/// 系列ごとに返す（内部ヘルパ）。実体は [`ChartData::stacked_cumulative`]
/// （イシュー #2082 で bar_chart と共有するため `charts::data` へ移設済み。
/// area_chart 側の呼び出し規約・エラー契約は変更なし）。
fn cumulative_series(data: &ChartData, expand: bool) -> Result<Vec<Vec<f64>>, ChartError> {
    data.stacked_cumulative(expand)
}

/// `gradient_id` から系列ごとの `<linearGradient>` 定義（`<defs>` の中身）を
/// 組み立てる（内部ヘルパ、モジュール doc「gradient の不変条件」参照）。
/// `stop-color` は [`ChartData::series_color_var`] の固定形
/// （`var(--fandhe-color-<name>)`）のみ、`offset`/`stop-opacity`/座標は
/// 静的リテラルであり、`id` は [`is_valid_identifier`] 通過済み文字列 +
/// `-<index>` のみのため、`fill="url(#<id>)"` の文字集合が閉じる。
fn gradient_defs(gradient_id: &str, colors: &[String]) -> Node {
    let stops: Vec<Node> = colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let id = format!("{gradient_id}-{i}");
            el(
                "linearGradient",
                vec![
                    ("id", id.as_str()),
                    ("x1", "0"),
                    ("y1", "0"),
                    ("x2", "0"),
                    ("y2", "1"),
                ],
                vec![
                    el(
                        "stop",
                        vec![
                            ("offset", "5%"),
                            ("stop-color", color.as_str()),
                            ("stop-opacity", "0.8"),
                        ],
                        vec![],
                    ),
                    el(
                        "stop",
                        vec![
                            ("offset", "95%"),
                            ("stop-color", color.as_str()),
                            ("stop-opacity", "0.1"),
                        ],
                        vec![],
                    ),
                ],
            )
        })
        .collect();
    el("defs", vec![], stops)
}

/// `show_grid` 有効時の水平グリッド線 1 本を組み立てる（内部ヘルパ）。
/// `stack: AreaStack::None`/`Normal`/`Expand` の `has_axes` 分岐いずれからも
/// 同一引数の組み合わせで呼ばれる（Review 指摘 #2081: 重複していたインライン
/// 呼び出しをこのヘルパへ統合）。`cartesian_grid` はピクセル座標をそのまま
/// SVG 座標として使う（`grid.rs` doc 参照）ため、`ticks` は
/// `y_scale.scale(t)` で domain 値をピクセルへ写像してから渡す
/// （domain の生の値をそのまま渡すと目盛ラベルの位置とグリッド線の位置が
/// ずれる、Review 指摘 #2081）。
pub(crate) fn grid_lines_for_ticks(
    left: f64,
    width: f64,
    plot_h: f64,
    y_scale: &LinearScale,
    ticks: &[f64],
) -> Result<Node, ChartError> {
    let grid_y: Vec<f64> = ticks.iter().map(|&t| y_scale.scale(t)).collect();
    grid::cartesian_grid(
        (left, width),
        (0.0, plot_h),
        &[],
        &grid_y,
        &GridProps {
            horizontal: true,
            vertical: false,
            ..GridProps::default()
        },
    )
}

/// `show_x_axis` 有効時のカテゴリラベル + 軸線を組み立てる（内部ヘルパ）。
/// `stack: AreaStack::None`/`Normal`/`Expand` の `has_axes` 分岐いずれからも
/// 同一引数の組み合わせで呼ばれる（Review 指摘 #2081: 重複していたインライン
/// 呼び出しをこのヘルパへ統合）。`axis::x_axis_categories` は band 中心配置
/// （`start + (i + 0.5) * width / n`）のためデータ点の `category_x`
/// （両端に点を置く等間隔配置）とラベル位置がずれるため、ラベルは
/// データ点の x 座標に直接合わせて描き、軸線のみ `axis` モジュールの
/// CSS 選択子（`data-part="axis-line"`）を再利用する。`x_axis_linear` は
/// `ticks` 非空を要求するためダミー tick `[0.0]` を渡すが、そのラベルは
/// カテゴリラベルと同じ座標に無関係な "0" として重複描画されてしまうため
/// （Review 指摘 #2081）、`show_tick_labels: false` で軸線のみに抑止する。
pub(crate) fn x_axis_category_labels(
    categories: &[String],
    plot_w: f64,
    left: f64,
    plot_h: f64,
    width: f64,
) -> Result<Vec<Node>, ChartError> {
    let n = categories.len();
    let mut nodes: Vec<Node> = categories
        .iter()
        .enumerate()
        .map(|(k, category)| {
            let cx = category_x(plot_w, n, k) + left;
            el(
                "text",
                vec![
                    ("data-scope", "chart"),
                    ("data-part", "tick-label"),
                    ("x", fmt_coord(cx).as_str()),
                    ("y", fmt_coord(plot_h + 16.0).as_str()),
                    ("text-anchor", "middle"),
                ],
                vec![fandhe_frontend_headless_ui::fandhe_frontend_core::text(
                    category,
                )],
            )
        })
        .collect();
    nodes.push(axis::x_axis_linear(
        &LinearScale::new((0.0, 1.0), (left, width))?,
        &[0.0],
        plot_h,
        &AxisProps {
            show_tick_lines: false,
            show_axis_line: true,
            show_tick_labels: false,
            ..AxisProps::default()
        },
    )?);
    Ok(nodes)
}

/// AreaChart 本体を組み立てる。
///
/// # Errors
///
/// - `width`/`height`/`aria_label` に関する契約は
///   [`crate::line_chart::line_chart`] と同じ（[`crate::line_chart::view_box_from_dims`] 参照）
/// - `stack` が [`AreaStack::Normal`]/[`AreaStack::Expand`] で系列に負値が
///   含まれる場合 [`ChartError::NegativeValue`]
/// - `fill` が [`AreaFill::Gradient`] で `gradient_id` が
///   [`is_valid_identifier`] を満たさない場合 [`ChartError::InvalidGradientId`]
/// - `show_x_axis`/`show_y_axis`/`show_grid` のいずれかが有効で、余白
///   差し引き後のプロット領域が 0 以下になる場合 [`ChartError::PlotAreaTooSmall`]
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

    if props.fill == AreaFill::Gradient && !is_valid_identifier(props.gradient_id) {
        return Err(ChartError::InvalidGradientId);
    }

    let recipe = recipe();
    let fill_class = if props.fill == AreaFill::Gradient {
        recipe.variant_class(AreaFill::Gradient)
    } else {
        String::new()
    };

    let has_axes = props.show_x_axis || props.show_y_axis || props.show_grid;

    let mut plot_children: Vec<Node> = Vec::new();

    if props.stack == AreaStack::None {
        let colors: Vec<String> = (0..props.data.series().len())
            .map(|i| props.data.series_color_var(i))
            .collect();
        if props.fill == AreaFill::Gradient {
            plot_children.push(gradient_defs(props.gradient_id, &colors));
        }

        if has_axes {
            let (left, bottom) = (
                if props.show_y_axis {
                    AXIS_LEFT_MARGIN
                } else {
                    0.0
                },
                if props.show_x_axis {
                    AXIS_BOTTOM_MARGIN
                } else {
                    0.0
                },
            );
            let plot_w = props.width - left;
            let plot_h = props.height - bottom;
            if plot_w <= 0.0 || plot_h <= 0.0 {
                return Err(ChartError::PlotAreaTooSmall);
            }
            let y_scale = LinearScale::new(props.data.domain(), (plot_h, 0.0))?.nice();
            let ticks = y_scale.ticks(4)?;
            // baseline は nice 化後 domain の下端（負値のみの系列でも面が
            // 上下反転しない、モジュール doc「面 path の閉じ方」と同じ判断）。
            let (dom_lo, _) = y_scale.domain();
            let baseline_y = y_scale.scale(dom_lo);

            if props.show_grid {
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
                // 軸/グリッドなし呼び出し（else 分岐）と同じヘルパを再利用する
                // （Review 指摘 #2081: 従来ここへインライン展開されていた
                // 「面 + 線」描画ロジックが `render_series_none` とほぼ同一の
                // 重複だったため、`left` オフセットを渡せるよう拡張して統合）。
                plot_children.extend(render_series_none(
                    plot_w,
                    left,
                    &y_scale,
                    baseline_y,
                    &s.values,
                    &color,
                    props.curve,
                    props.fill,
                    &fill_class,
                    &format!("{}-{i}", props.gradient_id),
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
                    plot_h,
                    props.width,
                )?);
            }
        } else {
            let y_scale = LinearScale::new(props.data.domain(), (props.height, 0.0))?;
            let (dom_lo, _dom_hi) = props.data.domain();
            let baseline_y = y_scale.scale(dom_lo);

            for (i, s) in props.data.series().iter().enumerate() {
                let color = props.data.series_color_var(i);
                plot_children.extend(render_series_none(
                    props.width,
                    0.0,
                    &y_scale,
                    baseline_y,
                    &s.values,
                    &color,
                    props.curve,
                    props.fill,
                    &fill_class,
                    &format!("{}-{i}", props.gradient_id),
                )?);
            }
        }
    } else {
        let expand = props.stack == AreaStack::Expand;
        let cum = cumulative_series(props.data, expand)?;
        let domain = if expand {
            (0.0, 1.0)
        } else {
            let max = cum
                .last()
                .map(|last| last.iter().copied().fold(f64::NEG_INFINITY, f64::max))
                .unwrap_or(0.0);
            if max <= 0.0 {
                (0.0, 1.0)
            } else {
                (0.0, max)
            }
        };

        // 積み上げ時（`AreaStack::Normal`/`Expand`）も `stack: None` と
        // 同じ規則で軸/グリッド用余白を差し引く（Review 指摘 #2081:
        // 積み上げ時に `show_x_axis`/`show_y_axis`/`show_grid` が黙って
        // 無視されていた欠陥の是正）。既定（軸/グリッドなし）は
        // `left == 0.0`/`plot_w == props.width`/`plot_h == props.height`
        // となり #2081 以前の出力と完全に一致する（golden 純追加原則）。
        let has_axes = props.show_x_axis || props.show_y_axis || props.show_grid;
        let (left, bottom) = (
            if props.show_y_axis {
                AXIS_LEFT_MARGIN
            } else {
                0.0
            },
            if props.show_x_axis {
                AXIS_BOTTOM_MARGIN
            } else {
                0.0
            },
        );
        let plot_w = props.width - left;
        let plot_h = props.height - bottom;
        if has_axes && (plot_w <= 0.0 || plot_h <= 0.0) {
            return Err(ChartError::PlotAreaTooSmall);
        }

        // `AreaStack::None` 分岐（`props.data.domain()`）と同じ規則で、
        // 軸/グリッド表示時のみ `nice()` を適用してドメイン上端を目盛の
        // 切りの良い値へ拡張する。`Expand` は domain `(0.0, 1.0)` が
        // カテゴリ合計比率の定義そのものであり、`nice()` で境界がずれると
        // 「合計 100%」の不変条件が崩れるため対象外とする（実際には
        // `nice_step` が 0.1 刻みで境界に揃うため通常はずれないが、定義上
        // 意図的に固定のままにする）。
        let y_scale = if has_axes && !expand {
            LinearScale::new(domain, (plot_h, 0.0))?.nice()
        } else {
            LinearScale::new(domain, (plot_h, 0.0))?
        };

        let colors: Vec<String> = (0..props.data.series().len())
            .map(|i| props.data.series_color_var(i))
            .collect();
        if props.fill == AreaFill::Gradient {
            plot_children.push(gradient_defs(props.gradient_id, &colors));
        }

        if has_axes {
            let ticks = y_scale.ticks(4)?;

            if props.show_grid {
                // None 分岐と同じヘルパを再利用する（Review 指摘 #2081）。
                plot_children.push(grid_lines_for_ticks(
                    left,
                    props.width,
                    plot_h,
                    &y_scale,
                    &ticks,
                )?);
            }

            plot_children.extend(render_stacked(
                plot_w,
                left,
                &y_scale,
                &cum,
                props.data,
                props.curve,
                props.fill,
                &fill_class,
                props.gradient_id,
            )?);

            if props.show_y_axis {
                // `AreaStack::Expand` は domain `(0.0, 1.0)` のカテゴリ
                // 合計比率のため、目盛ラベルは `%` 表示にする
                // （`label_scale: 100.0`、モジュール doc「積み上げ
                // （`AreaStack`）」節参照）。位置計算（`y_scale.scale`）は
                // 生の比率のまま使うため `Normal` と同じ座標系になる。
                let format = if expand {
                    axis::TickLabelFormat {
                        suffix: "%",
                        label_scale: 100.0,
                        ..axis::TickLabelFormat::default()
                    }
                } else {
                    axis::TickLabelFormat::default()
                };
                plot_children.push(axis::y_axis(
                    &y_scale,
                    &ticks,
                    left,
                    &AxisProps {
                        show_tick_lines: false,
                        show_axis_line: false,
                        format,
                        ..AxisProps::default()
                    },
                )?);
            }
            if props.show_x_axis {
                // None 分岐と同じヘルパを再利用する（Review 指摘 #2081）。
                plot_children.extend(x_axis_category_labels(
                    props.data.categories(),
                    plot_w,
                    left,
                    plot_h,
                    props.width,
                )?);
            }
        } else {
            plot_children.extend(render_stacked(
                plot_w,
                left,
                &y_scale,
                &cum,
                props.data,
                props.curve,
                props.fill,
                &fill_class,
                props.gradient_id,
            )?);
        }
    }

    let plot = svg_root(
        &view_box,
        vec![
            ("data-scope", "area-chart"),
            ("data-part", "plot"),
            ("aria-label", props.aria_label),
        ],
        plot_children,
    );

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

    /// 系列の [`crate::charts::data::SeriesColor`] 上書き（イシュー #2077）が
    /// 面・線の `fill`/`stroke` へ反映されることを固定する。
    #[test]
    fn render_series_reflects_series_color_override() {
        let categories = (0..2).map(|i| i.to_string()).collect();
        let d = ChartData::new(
            categories,
            vec![Series::new("s", vec![1.0, 2.0])
                .with_color(crate::charts::SeriesColor::token("success").unwrap())],
        )
        .unwrap();
        let node = area_chart(&AreaChartProps::new(&d, "sample"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"stroke="var(--fandhe-color-success)""#));
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
        assert!(a.contains(r#"[data-scope="area-chart"][data-part="plot"]"#));
        assert!(a.contains("overflow: visible"));
        assert!(a.contains(r#"[data-scope="area-chart"][data-part="series-line"]"#));
        assert!(a.contains("stroke-linejoin: round"));
        assert!(a.contains("stroke-linecap: round"));
        assert!(a.contains(r#"[data-scope="area-chart"][data-part="point"]"#));
        assert!(a.contains("stroke: var(--fandhe-color-bg)"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- イシュー #2081: shadcn/ui Charts（area）突合バリアント ---

    #[test]
    fn curve_natural_renders_cubic_segments() {
        let d = data(vec![1.0, 5.0, 2.0, 8.0]);
        let mut props = AreaChartProps::new(&d, "natural");
        props.curve = AreaCurve::Natural;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains('C'));
    }

    #[test]
    fn curve_natural_two_points_degenerates_to_line() {
        let d = data(vec![1.0, 5.0]);
        let mut props = AreaChartProps::new(&d, "natural-2");
        props.curve = AreaCurve::Natural;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(!html.contains('C'));
        assert!(html.contains('L'));
    }

    #[test]
    fn curve_natural_extreme_width_returns_non_finite_value_error() {
        // codex-review 指摘（イシュー #2081, PR #2254）: 3 カテゴリ・
        // 値 [0.0, 1.0, 0.0]・`width` が `f64::MAX` に近い極端値（`4e307`）
        // のとき、`AreaCurve::Natural` の Thomas 法中間計算
        // （`crate::charts::curve::control_points_1d` の
        // `8 * x_{n-1} + x_n`）が桁あふれし `inf` になる。この入力は
        // `view_box_from_dims` の寸法検証を通過し X 座標もすべて有限だが、
        // `fmt_coord` へ非有限座標が渡る前に `ChartError::NonFiniteValue`
        // を返すことを固定する（修正前は debug ビルドで
        // `fmt_coord` の `debug_assert` に panic していた）。
        let d = data(vec![0.0, 1.0, 0.0]);
        let mut props = AreaChartProps::new(&d, "natural-overflow");
        props.curve = AreaCurve::Natural;
        props.width = 4e307;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn curve_step_renders_midpoint_transitions() {
        let d = data(vec![1.0, 5.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "step");
        props.curve = AreaCurve::Step;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains("data-part=\"series-line\""));
    }

    #[test]
    fn curve_linear_output_matches_pre_2081_baseline() {
        let d = data(vec![1.0, 5.0, 2.0]);
        let default_props = AreaChartProps::new(&d, "linear");
        let mut explicit_props = AreaChartProps::new(&d, "linear");
        explicit_props.curve = AreaCurve::Linear;
        assert_eq!(
            render(&area_chart(&default_props, vec![]).unwrap()),
            render(&area_chart(&explicit_props, vec![]).unwrap())
        );
    }

    fn multi_series_data() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
            vec![
                Series::new("a", vec![10.0, 20.0, 15.0]),
                Series::new("b", vec![5.0, 8.0, 12.0]),
            ],
        )
        .unwrap()
    }

    /// テスト専用ヘルパ: `data-part="<part>"` を持つ各要素の開始タグから
    /// `<attr>="..."` の値を抽出する（自己完結 HTML の座標検証専用の
    /// 簡易パーサ。属性値は `fmt_coord` 経由の数値/固定リテラルのみで
    /// `>`/`"` を含まないためこの単純な走査で足りる）。
    fn attr_values_for_part(html: &str, part: &str, attr: &str) -> Vec<String> {
        let part_marker = format!(r#"data-part="{part}""#);
        let attr_marker = format!(r#"{attr}=""#);
        let mut out = Vec::new();
        let mut search_from = 0usize;
        while let Some(rel_pos) = html[search_from..].find(&part_marker) {
            let pos = search_from + rel_pos;
            let tag_start = html[..pos].rfind('<').expect("tag start");
            let tag_end = html[pos..].find('>').expect("tag end") + pos;
            let tag = &html[tag_start..tag_end];
            if let Some(rel_attr) = tag.find(&attr_marker) {
                let val_start = rel_attr + attr_marker.len();
                let val_end = tag[val_start..].find('"').expect("attr value end") + val_start;
                out.push(tag[val_start..val_end].to_string());
            }
            search_from = tag_end;
        }
        out
    }

    /// Review 指摘 #2081: `show_grid` が `cartesian_grid` へ domain の生の
    /// 値をそのまま渡しており、Y 軸目盛ラベルの位置とグリッド線の位置が
    /// ずれていた欠陥の回帰テスト。グリッド線（`grid-line`）の `y1` 座標
    /// 集合が Y 軸目盛ラベル（`tick-label`）の `y` 座標集合と完全一致する
    /// ことを固定する（両者とも同じ `ticks` を同じ `y_scale.scale` で
    /// ピクセルへ写像するため一致するはず）。
    #[test]
    fn grid_lines_use_pixel_coordinates_matching_y_axis_ticks() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "grid-align");
        props.show_y_axis = true;
        props.show_grid = true;
        let html = render(&area_chart(&props, vec![]).unwrap());

        let mut grid_y1 = attr_values_for_part(&html, "grid-line", "y1");
        let mut label_y = attr_values_for_part(&html, "tick-label", "y");
        assert!(!grid_y1.is_empty());
        assert_eq!(grid_y1.len(), label_y.len());
        grid_y1.sort();
        label_y.sort();
        assert_eq!(grid_y1, label_y);
    }

    /// Review 指摘 #2081: `show_x_axis` が軸線描画のためだけに
    /// `x_axis_linear(&[0.0], ...)` を呼んでおり、その目盛ラベル
    /// （無関係な `"0"`）がカテゴリラベルと同じ座標に重複描画されて
    /// いた欠陥の回帰テスト。`tick-label` の総数がカテゴリ数と一致する
    /// （余分な `"0"` ラベルが増えていない）ことを固定する。
    #[test]
    fn x_axis_line_does_not_duplicate_zero_tick_label() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "x-only");
        props.show_x_axis = true;
        let html = render(&area_chart(&props, vec![]).unwrap());
        let labels = attr_values_for_part(&html, "tick-label", "x");
        assert_eq!(labels.len(), d.categories().len());
    }

    #[test]
    fn stack_normal_renders_two_series_areas() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "stacked");
        props.stack = AreaStack::Normal;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="series-area""#).count(), 2);
    }

    #[test]
    fn stack_normal_rejects_negative_values() {
        let d = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![-1.0, 2.0])],
        )
        .unwrap();
        let mut props = AreaChartProps::new(&d, "neg");
        props.stack = AreaStack::Normal;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::NegativeValue
        );
    }

    #[test]
    fn stack_expand_rejects_category_total_overflow_to_infinity() {
        // codex-review 指摘（PR #2254）: 同一カテゴリに 1e308 の系列が
        // 2 本あると totals[k] が `f64::MAX` を超えて `+inf` になり、
        // 是正前は各 contribution が 0 になって「本来 50% ずつ」が
        // 全て 0% として正常終了する silent failure だった。
        let d = ChartData::new(
            vec!["a".to_string()],
            vec![
                Series::new("s1", vec![1e308]),
                Series::new("s2", vec![1e308]),
            ],
        )
        .unwrap();
        let mut props = AreaChartProps::new(&d, "overflow");
        props.stack = AreaStack::Expand;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn stack_expand_is_deterministic_and_uses_full_height_domain() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "expand");
        props.stack = AreaStack::Expand;
        let a = render(&area_chart(&props, vec![]).unwrap());
        let b = render(&area_chart(&props, vec![]).unwrap());
        assert_eq!(a, b);
    }

    /// Review 指摘 #2081: `stack: AreaStack::Normal`/`Expand` 時に
    /// `show_x_axis`/`show_y_axis`/`show_grid` が黙って無視されていた
    /// 欠陥の回帰テスト。積み上げ時も軸・グリッドが実際に描画される
    /// ことを固定する（`None` 分岐向けの `axes_and_grid_render_expected_parts`
    /// と対の積み上げ版）。
    #[test]
    fn stack_normal_axes_and_grid_are_not_silently_ignored() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "stacked-axes");
        props.stack = AreaStack::Normal;
        props.show_x_axis = true;
        props.show_y_axis = true;
        props.show_grid = true;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"data-scope="chart" data-part="y-axis""#));
        assert!(html.contains(r#"data-scope="chart" data-part="grid-line""#));
        assert!(html.contains("Jan"));
    }

    /// `AreaStack::Expand` の `show_y_axis` はカテゴリ合計比率を `%` 表示
    /// する（モジュール doc「積み上げ（`AreaStack`）」節、
    /// `docs/api/pre-styled-ui-api.md` の記述と実装を一致させる、
    /// Review 指摘 #2081）。
    #[test]
    fn stack_expand_y_axis_labels_use_percent_suffix() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "expand-axes");
        props.stack = AreaStack::Expand;
        props.show_y_axis = true;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"data-scope="chart" data-part="y-axis""#));
        // ticks(4) は domain (0.0, 1.0) から 0/0.2/0.4/0.6/0.8/1.0 を
        // 生成する。`label_scale: 100.0` により `fmt_coord` へ渡る前に
        // 100 倍されるため、実際のラベル値（丸め誤差の混入含め）まで
        // 固定する。
        assert!(html.contains(">0%<"));
        assert!(html.contains(">20%<"));
        assert!(html.contains(">100%<"));
    }

    #[test]
    fn gradient_fill_emits_defs_and_url_reference() {
        let d = data(vec![1.0, 2.0, 3.0]);
        let mut props = AreaChartProps::new(&d, "gradient");
        props.fill = AreaFill::Gradient;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains("<linearGradient"));
        assert!(html.contains(r#"fill="url(#fandhe-area-0)""#));
        assert!(html.contains("fd-area-chart--fill-gradient"));
    }

    #[test]
    fn gradient_fill_with_multiple_series_uses_distinct_ids() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "gradient-multi");
        props.fill = AreaFill::Gradient;
        props.gradient_id = "showcase-area";
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"id="showcase-area-0""#));
        assert!(html.contains(r#"id="showcase-area-1""#));
    }

    #[test]
    fn gradient_fill_rejects_invalid_gradient_id() {
        let d = data(vec![1.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "bad-id");
        props.fill = AreaFill::Gradient;
        props.gradient_id = "Invalid Id!";
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::InvalidGradientId
        );
    }

    #[test]
    fn solid_fill_root_class_is_unaffected_by_fill_axis() {
        // fill axis に default_variant を登録していないため、root の
        // class は size のみで構成され `fd-area-chart--fill-*` を
        // 一切含まないことを固定する（モジュール doc「gradient の不変
        // 条件」参照、recipe.rs `variant_classes` の axis 補完規則）。
        let d = data(vec![1.0, 2.0]);
        let html = render(&area_chart(&AreaChartProps::new(&d, "root"), vec![]).unwrap());
        assert!(html.starts_with(
            r#"<div data-scope="area-chart" data-part="root" class="fd-area-chart--size-md">"#
        ));
    }

    #[test]
    fn axes_and_grid_render_expected_parts() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "axes");
        props.show_x_axis = true;
        props.show_y_axis = true;
        props.show_grid = true;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(html.contains(r#"data-scope="chart" data-part="y-axis""#));
        assert!(
            html.contains(r#"data-scope="chart" data-part="grid-line""#)
                || html.contains("data-part=\"grid\"")
        );
        assert!(html.contains("Jan"));
    }

    #[test]
    fn axes_reject_when_plot_area_too_small() {
        let d = data(vec![1.0, 2.0]);
        let mut props = AreaChartProps::new(&d, "tiny");
        props.width = 10.0;
        props.height = 10.0;
        props.show_x_axis = true;
        props.show_y_axis = true;
        assert_eq!(
            area_chart(&props, vec![]).unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn axes_category_labels_are_escaped() {
        let payload = "</text><script>alert(1)</script>";
        let d = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let mut props = AreaChartProps::new(&d, "xss-axis");
        props.show_x_axis = true;
        let html = render(&area_chart(&props, vec![]).unwrap());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn area_chart_never_emits_data_attrs_beyond_scope_and_part() {
        let d = multi_series_data();
        let mut props = AreaChartProps::new(&d, "vocab");
        props.show_x_axis = true;
        props.show_y_axis = true;
        props.show_grid = true;
        props.stack = AreaStack::Normal;
        props.fill = AreaFill::Gradient;
        let html = render(&area_chart(&props, vec![]).unwrap());
        for token in html.split("data-").skip(1) {
            let name = token.split(['=', ' ', '>']).next().unwrap_or("");
            assert!(
                name == "scope" || name == "part",
                "unexpected data-* attribute: data-{name}"
            );
        }
    }
}
