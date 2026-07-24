//! `ChartData` モデル（イシュー #846）。
//!
//! chakra-ui `charts/use-chart.md` の `useChart`（data/series/集計/フォーマット
//! を束ねる React hook）を、JS ランタイムを持たない本フレームワークでは
//! 「明示的な Rust 構造体 + 決定的純関数」として吸収する。本モジュールの
//! 型・関数は状態を持たず、同一入力に対して常に同一出力を返す（決定性、
//! `.claude/rules/coding-rust.md` の AI 開発・保守前提）。
//!
//! 後続の各チャート部品（Bar/Line/Area/Pie 等、#848〜#851）は本モジュールの
//! [`ChartData`] を入力として受け取り、[`crate::charts::scale::LinearScale`]・
//! [`crate::charts::svg`] へ橋渡しする想定であり、本モジュール自体は SVG・
//! マークアップを一切生成しない（関心の分離）。

use super::ChartError;

/// 1 系列（1 本のグラフ線・1 色に対応するデータ列）。
///
/// `values` は [`ChartData::new`] を経由して構築された場合に限り、常に
/// 有限（`NaN`/`±inf` を含まない）かつ [`ChartData::categories`] と同じ
/// 長さであることが保証される。フィールドを直接構築した場合はこの保証が
/// 及ばないため、本モジュールの集計関数（[`total`]/[`min`]/[`max`]/
/// [`value_percent`]）は空・非有限値混入のいずれも panic せず fail-closed
/// に扱う。
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// 系列名（凡例・ツールチップ表示用。styled 層が既定エスケープ経由で
    /// テキストノード化する、[`crate::charts`] モジュール doc の不変条件 1）。
    pub name: String,
    /// カテゴリ順に対応する値列。
    pub values: Vec<f64>,
}

impl Series {
    /// 新しい系列を組み立てる（検証なしの薄いコンストラクタ）。値の検証は
    /// [`ChartData::new`] が一括で行う。
    #[must_use]
    pub fn new(name: impl Into<String>, values: Vec<f64>) -> Self {
        Series {
            name: name.into(),
            values,
        }
    }
}

/// チャート全体のデータモデル（カテゴリ軸 + 複数系列）。
///
/// [`ChartData::new`] を経由した構築のみを公開し、以下を不変条件として
/// 保証する（フィールドは非公開、アクセサ経由でのみ参照可能）。
///
/// 1. `categories` は 1 件以上。
/// 2. `series` は 1 件以上。
/// 3. 各系列の `values.len() == categories.len()`。
/// 4. 全ての値が有限（`NaN`/`±inf` を含まない）。
#[derive(Debug, Clone, PartialEq)]
pub struct ChartData {
    categories: Vec<String>,
    series: Vec<Series>,
}

/// [`ChartData::sort_by_series`] のソート方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// 値の昇順（小さい順）。
    Ascending,
    /// 値の降順（大きい順）。
    Descending,
}

impl ChartData {
    /// カテゴリ・系列群からチャートデータを構築する。
    ///
    /// # Errors
    ///
    /// - `categories`/`series` のいずれかが空の場合 [`ChartError::EmptyData`]
    /// - いずれかの系列の値数がカテゴリ数と一致しない場合
    ///   [`ChartError::SeriesLengthMismatch`]
    /// - いずれかの値が `NaN`/`±inf` の場合 [`ChartError::NonFiniteValue`]
    pub fn new(categories: Vec<String>, series: Vec<Series>) -> Result<Self, ChartError> {
        if categories.is_empty() || series.is_empty() {
            return Err(ChartError::EmptyData);
        }
        for s in &series {
            if s.values.len() != categories.len() {
                return Err(ChartError::SeriesLengthMismatch);
            }
            if s.values.iter().any(|v| !v.is_finite()) {
                return Err(ChartError::NonFiniteValue);
            }
        }
        Ok(ChartData { categories, series })
    }

    /// カテゴリ軸の並び（挿入順）を返す。
    #[must_use]
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    /// 系列一覧（挿入順）を返す。
    #[must_use]
    pub fn series(&self) -> &[Series] {
        &self.series
    }

    /// 名前が一致する系列を検索する（内部ヘルパ）。
    fn find_series(&self, name: &str) -> Option<&Series> {
        self.series.iter().find(|s| s.name == name)
    }

    /// 全系列・全カテゴリを横断した値域 `(min, max)` を返す。
    ///
    /// [`crate::charts::scale::LinearScale`] の domain 入力として使う想定
    /// （複数系列を同一スケールに重ねて描画する軸共有チャート、#847/#849 等）。
    /// [`ChartData::new`] の不変条件により `categories`/`series` は必ず
    /// 1 件以上かつ全値有限であるため、本関数は必ず有限な `(min, max)` を
    /// 返す（空データによる panic は構造的に発生しない）。
    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for s in &self.series {
            for &v in &s.values {
                min = min.min(v);
                max = max.max(v);
            }
        }
        (min, max)
    }

    /// 指定した系列名の値を基準に、カテゴリと全系列の値を安定ソートした
    /// 新しい [`ChartData`] を返す（元の `self` は変更しない）。
    ///
    /// 複数カテゴリが同値を持つ場合、元の挿入順を保つ（`Vec::sort_by` の
    /// 安定ソート特性、決定性の一部）。
    ///
    /// # Errors
    ///
    /// `name` に一致する系列が存在しない場合 [`ChartError::UnknownSeriesName`]。
    pub fn sort_by_series(&self, name: &str, direction: SortDirection) -> Result<Self, ChartError> {
        let key_series = self
            .find_series(name)
            .ok_or(ChartError::UnknownSeriesName)?;

        let mut order: Vec<usize> = (0..self.categories.len()).collect();
        order.sort_by(|&a, &b| {
            let (va, vb) = (key_series.values[a], key_series.values[b]);
            let ord = va.partial_cmp(&vb).expect(
                "ChartData::new が全値の有限性を検証済みのため partial_cmp は必ず Some を返す",
            );
            match direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });

        let categories = order.iter().map(|&i| self.categories[i].clone()).collect();
        let series = self
            .series
            .iter()
            .map(|s| {
                let values = order.iter().map(|&i| s.values[i]).collect();
                Series::new(s.name.clone(), values)
            })
            .collect();

        Ok(ChartData { categories, series })
    }
}

/// 系列の合計値（chakra-ui `useChart` の `getTotal` 相当）。
#[must_use]
pub fn total(series: &Series) -> f64 {
    series.values.iter().sum()
}

/// 系列の最小値（chakra-ui `useChart` の `getMin` 相当）。
///
/// `values` が空の場合は `None`（`ChartData::new` を経由した系列では
/// 発生しないが、[`Series::new`] 直接構築の空系列を渡された場合も panic
/// せず fail-closed に扱う）。
#[must_use]
pub fn min(series: &Series) -> Option<f64> {
    series
        .values
        .iter()
        .copied()
        .fold(None, |acc, v| Some(acc.map_or(v, |m: f64| m.min(v))))
}

/// 系列の最大値（chakra-ui `useChart` の `getMax` 相当）。
///
/// 空系列に対する挙動は [`min`] と同様。
#[must_use]
pub fn max(series: &Series) -> Option<f64> {
    series
        .values
        .iter()
        .copied()
        .fold(None, |acc, v| Some(acc.map_or(v, |m: f64| m.max(v))))
}

/// 系列合計に対する `value` の割合（パーセント、0..=100 目安）を返す
/// （chakra-ui `useChart` の `getValuePercent` 相当）。
///
/// 合計が 0（全値 0、または空系列）の場合は `NaN` を生まず `0.0` を返す
/// （[`crate::charts`] の「フォーマット段への非有限値の到達を防ぐ」不変条件、
/// 呼び出し元が [`crate::charts::svg::fmt_coord`] へそのまま渡しても安全）。
#[must_use]
pub fn value_percent(series: &Series, value: f64) -> f64 {
    let t = total(series);
    if t == 0.0 {
        0.0
    } else {
        value / t * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
            vec![
                Series::new("visits", vec![10.0, 30.0, 20.0]),
                Series::new("signups", vec![1.0, 2.0, 3.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn new_rejects_empty_categories_or_series() {
        assert_eq!(
            ChartData::new(vec![], vec![Series::new("a", vec![])]).unwrap_err(),
            ChartError::EmptyData
        );
        assert_eq!(
            ChartData::new(vec!["a".to_string()], vec![]).unwrap_err(),
            ChartError::EmptyData
        );
    }

    #[test]
    fn new_rejects_length_mismatch() {
        let err = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0])],
        )
        .unwrap_err();
        assert_eq!(err, ChartError::SeriesLengthMismatch);
    }

    #[test]
    fn new_rejects_non_finite_values() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = ChartData::new(vec!["a".to_string()], vec![Series::new("s", vec![bad])])
                .unwrap_err();
            assert_eq!(err, ChartError::NonFiniteValue);
        }
    }

    #[test]
    fn total_min_max_value_percent_known_values() {
        let data = sample();
        let visits = &data.series()[0];
        assert_eq!(total(visits), 60.0);
        assert_eq!(min(visits), Some(10.0));
        assert_eq!(max(visits), Some(30.0));
        assert!((value_percent(visits, 30.0) - 50.0).abs() < 1e-9);
    }

    #[test]
    fn value_percent_zero_total_returns_zero_not_nan() {
        let zeros = Series::new("z", vec![0.0, 0.0]);
        assert_eq!(value_percent(&zeros, 0.0), 0.0);
    }

    #[test]
    fn min_max_on_empty_series_return_none() {
        let empty = Series::new("e", vec![]);
        assert_eq!(min(&empty), None);
        assert_eq!(max(&empty), None);
    }

    #[test]
    fn domain_spans_all_series() {
        let data = sample();
        assert_eq!(data.domain(), (1.0, 30.0));
    }

    #[test]
    fn sort_by_series_is_stable_and_reorders_all_series() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![
                Series::new("key", vec![2.0, 1.0, 1.0]),
                Series::new("other", vec![20.0, 10.0, 11.0]),
            ],
        )
        .unwrap();

        let sorted = data
            .sort_by_series("key", SortDirection::Ascending)
            .unwrap();
        // b, c は同値 1.0 のため元の挿入順（b → c）を保つ（安定ソート）。
        assert_eq!(sorted.categories(), &["b", "c", "a"]);
        assert_eq!(sorted.series()[0].values, vec![1.0, 1.0, 2.0]);
        assert_eq!(sorted.series()[1].values, vec![10.0, 11.0, 20.0]);

        let desc = data
            .sort_by_series("key", SortDirection::Descending)
            .unwrap();
        assert_eq!(desc.categories(), &["a", "b", "c"]);
    }

    #[test]
    fn sort_by_series_unknown_name_is_error() {
        let data = sample();
        assert_eq!(
            data.sort_by_series("missing", SortDirection::Ascending)
                .unwrap_err(),
            ChartError::UnknownSeriesName
        );
    }

    #[test]
    fn sort_by_series_does_not_mutate_original() {
        let data = sample();
        let before = data.clone();
        let _ = data.sort_by_series("visits", SortDirection::Descending);
        assert_eq!(data, before);
    }
}
