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
//!   明示的な Rust 純関数として吸収する）。
//! - [`scale`]: 線形スケール（domain → range 写像）・1-2-5 nice tick 算出。
//! - [`svg`]: SVG ノード木生成ヘルパー（`viewBox`・座標文字列化・`path` の
//!   `d` 属性組み立て）。後続チャート部品はここを経由してのみ SVG を組み立てる。
//! - [`axis`]: X/Y 軸（chakra-ui `charts/axes.md` 相当、イシュー #847）。
//! - [`grid`]: CartesianGrid（chakra-ui `charts/cartesian-grid.md` 相当、
//!   イシュー #847）。
//! - [`legend`]: 凡例（chakra-ui `charts/legend.md` 相当、イシュー #847）。
//! - [`tooltip`]: データ点のツールチップ表示（chakra-ui `charts/tooltip.md`
//!   相当、イシュー #847。[`crate::tooltip`] とは別物、[`tooltip`] モジュール
//!   doc 参照）。
//!
//! # 本モジュールの不変条件（[`crate`] クレート全体の不変条件を継承、
//! `.claude/rules/coding-rust.md`）
//!
//! 1. **`raw_html()` を使用しない**: マークアップはすべて
//!    [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/`text` 経由の
//!    ノード木 API のみで組み立てる（REQ-1、[`svg`] モジュール doc 参照）。
//! 2. **数値の決定的文字列化**: 座標・tick 値は [`svg::fmt_coord`] にのみ
//!    実装を一元化する（`{:.2}` 最近接偶数丸め → 末尾ゼロ除去、出力文字集合
//!    `[0-9.-]` に閉じる）。呼び出し元（後続チャート部品）はこの関数のみを
//!    経由し、独自の数値フォーマットを実装しない。
//! 3. **fail-closed な数値検証**: `NaN`/`±inf` は [`data::ChartData`]・
//!    [`scale::LinearScale`] の構築時に [`ChartError`] として拒否し、
//!    フォーマット段（[`svg::fmt_coord`]）へ到達させない。
//! 4. **外部依存ゼロ**: 本モジュールは `fandhe_frontend_headless_ui`
//!    （既存のクレート依存）のみを使用し、新規クレート依存を追加しない
//!    （REQ-3 不変）。

pub mod axis;
pub mod data;
pub mod grid;
pub mod legend;
pub mod scale;
pub mod svg;
pub mod tooltip;

pub use data::{ChartData, Series};
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
    /// [`data::ChartData::sort_by_series`] に、存在しない系列名が渡された。
    UnknownSeriesName,
}

impl std::fmt::Display for ChartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ChartError::SeriesLengthMismatch => "series value count does not match category count",
            ChartError::EmptyData => "categories and series must not be empty",
            ChartError::NonFiniteValue => "value must be finite (NaN/inf is rejected)",
            ChartError::DegenerateDomain => "domain must have non-zero width (min != max)",
            ChartError::InvalidTickTarget => "tick target must be in range 1..=50",
            ChartError::UnknownSeriesName => "sort_by_series: no series with the given name",
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
        ] {
            let message = err.to_string();
            assert!(!message.is_empty());
            assert!(!message.contains('<'));
        }
    }
}
