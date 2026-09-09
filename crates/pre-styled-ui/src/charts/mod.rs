//! charts 基盤（座標スケーリング・SVG ノード木生成・`ChartData` モデル、
//! イシュー #846・親 Phase #845）。
//!
//! chakra-ui の charts 群（16 項目）は recharts（外部 JS ランタイム）依存の
//! ため `docs/policy/intentional-non-adoption.md` §7 で「保留」区分だった。
//! 保留解除トリガーは「外部依存ゼロを維持したまま SVG ノード木生成のみで
//! 実装できる設計の確立」であり、本モジュールがその基盤を提供する。
//! 個々のチャート部品（Area/Bar/Line/Pie 等、#848〜#851）はいずれも本モジュールに
//! 依存するが、本モジュール自体はそれらの上位部品を持たない（配置先判断は
//! `docs/design/charts-foundation-design.md` 参照）。軸/グリッド/凡例/
//! ツールチップ（[`axis`]/[`grid`]/[`legend`]/[`tooltip`]、#847）は
//! [`data`]/[`scale`]/[`svg`] の最初の消費者であり、後続チャート部品はこれらを
//! 自身のデータ系列と同じ座標系の上に重ねて使う想定である。
//!
//! # 構成
//!
//! - [`data`]: `ChartData`/`Series` モデルと集計・ソート API
//!   （chakra-ui `useChart` の getTotal/getMin/getMax/getValuePercent 相当を
//!   明示的な Rust 純関数として吸収する）。`Series` は `label`/`color`/
//!   `icon` の系列設定（shadcn/ui `ChartConfig` 相当、イシュー #2077）を
//!   任意で保持し、[`legend`]・line/area/bar/radar の各消費者が
//!   [`ChartData::series_color_var`]/[`data::Series::display_label`] を
//!   経由して共有する。
//! - [`curve`]: 曲線補間ジオメトリ（natural spline / step、イシュー
//!   #2081/#2083）。[`crate::area_chart`] の `AreaCurve::Natural`/
//!   `AreaCurve::Step` と [`crate::line_chart`] の [`Curve::Natural`]/
//!   [`Curve::Step`] が共有する純関数。
//! - [`scale`]: 線形スケール（domain → range 写像）・1-2-5 nice tick 算出。
//! - [`svg`]: SVG ノード木生成ヘルパー（`viewBox`・座標文字列化・`path` の
//!   `d` 属性組み立て）。後続チャート部品はここを経由してのみ SVG を組み立てる。
//! - [`scatter_chart`]: 2 軸線形スケール + 点マーカーの SVG 散布図
//!   （イシュー #851）。
//! - [`radar_chart`]: 正多角形グリッド + 系列ポリゴンの SVG レーダーチャート
//!   （頂点角度の決定的算出、イシュー #851）。
//! - [`axis`]: X/Y 軸（chakra-ui `charts/axes.md` 相当、イシュー #847）。
//! - [`grid`]: CartesianGrid（chakra-ui `charts/cartesian-grid.md` 相当、
//!   イシュー #847）。
//! - [`legend`]: 凡例（chakra-ui `charts/legend.md` 相当、イシュー #847）。
//! - [`tooltip`]: データ点のツールチップ表示（chakra-ui `charts/tooltip.md`
//!   相当、イシュー #847。[`crate::tooltip`] とは別物、[`tooltip`] モジュール
//!   doc 参照）。
//! - [`bar_chart`]: 縦/横 orientation の SVG 棒グラフ（イシュー #849）。
//! - [`bar_list`]: ランキング型バーリスト（HTML、イシュー #849）。
//! - [`bar_segment`]: 構成比バー（HTML、100% 積み上げ、イシュー #849）。
//! - [`pie`]: 円弧ジオメトリ（角度計算・sector/annulus path 生成、イシュー
//!   #850）。[`crate::pie_chart`]/[`crate::donut_chart`]（styled 層）が
//!   本モジュールを経由して円グラフ・ドーナツグラフの `d` 属性を組み立てる。
//!   角丸端の環状セクタ（[`pie::annulus_sector_rounded_path`]、イシュー
//!   #2079）は [`crate::radial_chart`]（同心リング型グラフ）の shape/text
//!   バリアントが消費する。
//!
//! # 本モジュールの不変条件（[`crate`] クレート全体の不変条件を継承、
//! `.claude/rules/coding-rust.md`）
//!
//! 1. **`raw_html()` を使用しない**: マークアップはすべて
//!    [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/`text` 経由の
//!    ノード木 API のみで組み立てる（REQ-1、[`svg`] モジュール doc 参照）。
//! 2. **数値の決定的文字列化**: SVG のピクセル座標・寸法は [`svg::fmt_coord`]
//!    にのみ実装を一元化する（`{:.2}` 丸め → 末尾ゼロ除去、出力文字集合
//!    `[0-9.-]` に閉じる）。系列データ値そのもの（軸ラベル・目盛の数値表示
//!    等、ピクセル座標ではない値）は [`svg::fmt_value`]（イシュー #2085
//!    追補。`fmt_coord` と同じ丸め規則を共有しつつ、絶対値が小さい場合の
//!    有効数字保持のため精度を動的に引き上げる。`|v| >= 0.01` では
//!    `fmt_coord` とバイト同一）を経由する。呼び出し元（後続チャート部品）
//!    は「ピクセル座標なら `fmt_coord`、データ値なら `fmt_value`」の 2 択
//!    のみとし、独自の数値フォーマットを実装しない。
//! 3. **fail-closed な数値検証**: `NaN`/`±inf` は [`data::ChartData`]・
//!    [`scale::LinearScale`] の構築時に [`ChartError`] として拒否し、
//!    フォーマット段（[`svg::fmt_coord`]/[`svg::fmt_value`]）へ到達させない。
//! 4. **外部依存ゼロ**: 本モジュールは `fandhe_frontend_headless_ui`
//!    （既存のクレート依存）のみを使用し、新規クレート依存を追加しない
//!    （REQ-3 不変）。

pub mod axis;
pub mod bar_chart;
pub mod bar_list;
pub mod bar_segment;
pub mod curve;
pub mod data;
pub mod grid;
pub mod legend;
pub mod pie;
pub mod radar_chart;
pub mod scale;
pub mod scatter_chart;
pub mod svg;
pub mod tooltip;

pub use curve::Curve;
pub use data::{ChartData, Series, SeriesColor};
pub use scale::LinearScale;

/// charts 基盤全体（[`data`]/[`scale`]）が返す構築エラー。
///
/// `Display` にはユーザーデータの値そのものを含めず、検証に失敗した理由
/// （不正な形状・非有限値の混入等）のみを記述する（`.claude/rules/security.md`
/// 「機微情報の露出」対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartError {
    /// [`data::ChartData::new`] で、系列の値数がカテゴリ数と一致しない。
    SeriesLengthMismatch,
    /// カテゴリ・系列のいずれかが空（1 件もない）データが渡された。
    EmptyData,
    /// 値・domain・range のいずれかに `NaN`/`±inf` が含まれる。
    NonFiniteValue,
    /// [`scale::LinearScale::new`] で domain の幅が 0（`min == max`）。
    DegenerateDomain,
    /// [`scale::LinearScale::ticks`] の `target` が許容範囲（1..=50）外。
    InvalidTickTarget,
    /// [`data::ChartData::sort_by_series`]/[`bar_list::root`]/
    /// [`bar_segment::root`] に、存在しない系列名が渡された。`Display`
    /// メッセージは特定 API 名を含めない汎用文言に統一する（PR #877 Bugbot
    /// 指摘、イシュー #849。`bar_list`/`bar_segment` も同 variant を返す
    /// ようになったため、`sort_by_series` 名指しは実際の発生元と乖離した
    /// 誤ったメッセージになっていた）。
    UnknownSeriesName,
    /// [`bar_list`]/[`bar_segment`]/[`radar_chart::root`] に、比率描画では
    /// 意味を持たない負値が系列中に含まれていた（イシュー #849/#851）。
    NegativeValue,
    /// [`bar_segment`] で対象系列の合計が 0（構成比が定義できない）
    /// （イシュー #849）。全セグメント幅 0% の silent failure を避けるため
    /// 構築時に拒否する（`bar_segment` モジュール doc 参照）。
    ZeroTotal,
    /// [`radar_chart::root`] に、軸（`categories`）が 3 未満のデータが渡され
    /// 多角形が定義できない（イシュー #851）。
    TooFewAxes,
    /// [`bar_chart::root`] で `viewBox` からカテゴリラベル用余白を
    /// 差し引いたプロット領域の幅・高さが 0 以下になる
    /// （`props.width`/`props.height` が小さすぎる）
    /// （PR #877 レビュー指摘、イシュー #849）。[`radar_chart::root`] でも
    /// `viewBox` から軸ラベル用余白を差し引いたプロット半径が 0 以下になる
    /// 場合（`props.size` が小さすぎる、イシュー #851）に同じ variant を返す。
    /// `ViewBox::new` は寸法の有限性・正値のみを検証し、ラベル余白差し引き後の
    /// 実描画領域までは検証しないため、放置するとバー/ポリゴンが潰れる、
    /// または viewBox 外に無警告で描画される silent failure になる。
    PlotAreaTooSmall,
    /// [`crate::area_chart::AreaChartProps::gradient_id`] が
    /// `is_valid_identifier`（英小文字始まり、以降英数小文字/ハイフン）を
    /// 満たさない（イシュー #2081）。1 ページに複数チャートを置く場合の
    /// `<linearGradient id>` 一意化を呼び出し側の責務とするための
    /// fail-closed 検証（`crate::area_chart` モジュール doc「gradient の
    /// 不変条件」参照）。
    InvalidGradientId,
    /// [`bar_chart::BarChartProps::corner_radius`] が非有限、または負値
    /// （イシュー #2082、shadcn/ui Charts（bar）突合）。角丸半径は fail-closed
    /// に拒否し、サイレントな不正描画（負の半径による自己交差 path 等）を
    /// 作らない。
    InvalidCornerRadius,
    /// [`bar_chart::BarChartProps::active_index`] がカテゴリ数以上
    /// （イシュー #2082）。範囲外インデックスをサイレントに無視せず
    /// fail-closed に拒否する。
    IndexOutOfRange,
}

impl std::fmt::Display for ChartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ChartError::SeriesLengthMismatch => "series value count does not match category count",
            ChartError::EmptyData => "categories and series must not be empty",
            ChartError::NonFiniteValue => "value must be finite (NaN/inf is rejected)",
            ChartError::DegenerateDomain => "domain must have non-zero width (min != max)",
            ChartError::InvalidTickTarget => "tick target must be in range 1..=50",
            ChartError::UnknownSeriesName => "no series with the given name",
            ChartError::NegativeValue => "value must be non-negative for ratio-based rendering",
            ChartError::ZeroTotal => "series total must be non-zero to compute a ratio",
            ChartError::TooFewAxes => "radar chart requires at least 3 axes",
            ChartError::PlotAreaTooSmall => {
                "width/height must leave a positive plot area after reserving label space"
            }
            ChartError::InvalidGradientId => {
                "gradient id must be a lowercase identifier ([a-z][a-z0-9-]*)"
            }
            ChartError::InvalidCornerRadius => "corner radius must be finite and non-negative",
            ChartError::IndexOutOfRange => "index must be within the category count",
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for ChartError {}

/// 系列インデックスから系列配色トークン（`theme.rs` の `chart-1`〜`chart-6`、
/// イシュー #846）の `var()` 参照を返す。
///
/// 6 色を超える系列数では循環（`index % 6`）して再利用する（chakra-ui の
/// `colorPalette` ローテーション相当）。トークン名は [`crate::theme::color_var`]
/// の allowlist（英数小文字・ハイフンのみ）を必ず満たす固定文字列から構築する
/// ため `expect` で確定させる（構築時に必ず成功することがコードから自明で
/// あり、`unwrap()`/`panic!` 回避規約の例外条件を満たす、`theme.rs` の
/// `Theme::default()` と同型の判断）。
///
/// # 観察記録（イシュー #1593、charts 共通 4 パーツのスタイル調整）
///
/// dark モードで `chart-1`（`#63b3ed`）と `chart-6`（`#76e4f7`）がやや近接
/// して見えるが、`theme.rs` の 6 色パレット自体の見直しは本イシューの
/// スコープ外とした。`theme.rs` の変更は
/// `crates/docs-site/tests/site_css_contract.rs` 等の契約テストへ波及し、
/// charts 以外の他部品にも影響するため（先例の PR #1863/#1864 も同じ判断で
/// `theme.rs` を不変としている）。
///
/// # 意図的に合わせなかった点（イシュー #2077、shadcn/ui `ChartConfig` 突合）
///
/// shadcn/ui の `--chart-1`〜`--chart-5`（5 段階）に対し、本フレームワーク
/// の `chart-1`〜`chart-6`（6 段階）は変更しない。Phase 0（イシュー
/// #2005、`docs/design/color-token-system.md` §9.2）で「6 は shadcn 5 の
/// スーパーセット（5 段を包含し 1 段多い）であり変更不要」と既に決定済み
/// であり、5 へ削ると `chart-6` を参照する既存の利用者コード・golden の
/// 出力が壊れる（golden 純追加原則に反する）。系列ごとの色上書きは
/// [`data::SeriesColor`]（[`data::Series::with_color`]）が個別に提供する
/// ため、6 段循環に閉じない任意色の指定は既にでき、段階数を追随させる
/// 実利は無い。
#[must_use]
pub fn series_color_var(index: usize) -> String {
    const SLOT_COUNT: usize = 6;
    let slot = index % SLOT_COUNT + 1;
    crate::theme::color_var(&format!("chart-{slot}"))
        .expect("chart-<1..=6> は theme.rs の TokenName allowlist を満たす固定文字列")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_color_var_cycles_through_six_slots() {
        assert_eq!(series_color_var(0), "var(--fandhe-color-chart-1)");
        assert_eq!(series_color_var(5), "var(--fandhe-color-chart-6)");
        assert_eq!(series_color_var(6), "var(--fandhe-color-chart-1)");
        assert_eq!(series_color_var(11), "var(--fandhe-color-chart-6)");
    }

    #[test]
    fn series_color_var_is_deterministic() {
        for i in 0..20 {
            assert_eq!(series_color_var(i), series_color_var(i));
        }
    }

    #[test]
    fn chart_error_display_never_leaks_arbitrary_values() {
        // Display は固定メッセージのみを返し、値そのものを含めないことを固定する
        // （`.claude/rules/security.md` 「機微情報の露出」対応）。
        for err in [
            ChartError::SeriesLengthMismatch,
            ChartError::EmptyData,
            ChartError::NonFiniteValue,
            ChartError::DegenerateDomain,
            ChartError::InvalidTickTarget,
            ChartError::UnknownSeriesName,
            ChartError::NegativeValue,
            ChartError::ZeroTotal,
            ChartError::TooFewAxes,
            ChartError::PlotAreaTooSmall,
            ChartError::InvalidGradientId,
            ChartError::InvalidCornerRadius,
            ChartError::IndexOutOfRange,
        ] {
            let message = err.to_string();
            assert!(!message.is_empty());
            assert!(!message.contains('<'));
        }
    }
}
