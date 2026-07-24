//! 線形スケール（座標スケーリング、イシュー #846）。
//!
//! [`super::data::ChartData`] の値域（domain、データ空間）を SVG 描画領域
//! （range、ピクセル/ビューポート空間）へ線形写像する。後続チャート部品
//! （#847〜#851）は本モジュールの [`LinearScale`] のみを経由して座標変換を
//! 行い、独自の写像式を実装しない（決定性・一元化、`.claude/rules/coding-rust.md`）。
//!
//! tick 算出（[`LinearScale::ticks`]/[`LinearScale::nice`]）は d3
//! （`d3-scale`/`d3-array` の `ticks`/`tickStep`）が採用する「1-2-5 ステップ」
//! アルゴリズムを踏襲する。10 のべき乗を基準に、正規化したステップ幅を
//! `{1, 2, 5, 10}` のいずれか近い方へ丸めることで、軸ラベルとして読みやすい
//! 値（例: 20, 50, 100 であって 17, 43, 91 ではない）のみを生成する。

use super::ChartError;

/// 線形スケール本体（domain → range の写像パラメータ）。
///
/// [`LinearScale::new`] を経由した構築のみを公開し、`domain`/`range` は
/// ともに有限、かつ `domain` は非退化（`domain.0 != domain.1`）であることを
/// 保証する（[`ChartError::NonFiniteValue`]/[`ChartError::DegenerateDomain`]
/// で fail-closed に拒否する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
}

impl LinearScale {
    /// domain（データ空間の値域）・range（描画空間の値域）からスケールを
    /// 構築する。
    ///
    /// `domain`/`range` の各要素の大小関係は問わない（`domain.0 > domain.1`
    /// を軸反転として許容する。SVG の y 軸は下向きが正のため、データの
    /// 上下を反転させたい呼び出し元がこれを利用する）。
    ///
    /// # Errors
    ///
    /// - `domain`/`range` のいずれかの要素が `NaN`/`±inf` の場合
    ///   [`ChartError::NonFiniteValue`]
    /// - `domain.0 == domain.1`（データ空間の幅が 0）の場合
    ///   [`ChartError::DegenerateDomain`]（幅 0 では写像式が 0 除算になるため、
    ///   縮退規則を設けず構築時に拒否する）
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Result<Self, ChartError> {
        let finite = domain.0.is_finite()
            && domain.1.is_finite()
            && range.0.is_finite()
            && range.1.is_finite();
        if !finite {
            return Err(ChartError::NonFiniteValue);
        }
        if domain.0 == domain.1 {
            return Err(ChartError::DegenerateDomain);
        }
        Ok(LinearScale { domain, range })
    }

    /// domain を返す。
    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        self.domain
    }

    /// range を返す。
    #[must_use]
    pub fn range(&self) -> (f64, f64) {
        self.range
    }

    /// domain 上の値 `v` を range 上の座標へ線形写像する。
    ///
    /// `v` が domain の範囲外でも外挿する（クランプしない。範囲外描画の
    /// 抑制は呼び出し元＝チャート部品側の責務）。`v` が非有限の場合、結果も
    /// 非有限になる（本関数自体は検証しない。[`super::data::ChartData::new`]
    /// を経由した値のみを渡す契約を呼び出し元が担う）。
    #[must_use]
    pub fn scale(&self, v: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;
        let t = (v - d0) / (d1 - d0);
        r0 + t * (r1 - r0)
    }

    /// domain を 1-2-5 ステップの「nice」な境界へ外側方向に拡張した新しい
    /// スケールを返す（`range` は変更しない）。
    ///
    /// 内部的に目標 tick 本数 10 を基準としたステップ幅を算出し、`domain.0`
    /// 側は切り下げ、`domain.1` 側は切り上げる（軸反転時、すなわち
    /// `domain.0 > domain.1` の場合は大小を入れ替えて計算し、向きを保つ）。
    #[must_use]
    pub fn nice(&self) -> LinearScale {
        let (d0, d1) = self.domain;
        let (lo, hi) = if d0 <= d1 { (d0, d1) } else { (d1, d0) };
        let step = nice_step((hi - lo) / 10.0);
        let nice_lo = (lo / step).floor() * step;
        let nice_hi = (hi / step).ceil() * step;
        let domain = if d0 <= d1 {
            (nice_lo, nice_hi)
        } else {
            (nice_hi, nice_lo)
        };
        LinearScale {
            domain,
            range: self.range,
        }
    }

    /// domain 上に決定的な「nice」tick 値列を生成する（d3 `ticks` 相当）。
    ///
    /// `target` は生成したい tick 本数の目安（実際の本数は 1-2-5 ステップの
    /// 都合で前後する）。`target` の許容範囲は 1..=50 とし、この範囲外は
    /// 構築時に拒否する（無限ループ・過大メモリ割当の構造的排除、
    /// `.claude/rules/security.md` A04 対応）。
    ///
    /// # Errors
    ///
    /// `target` が 1..=50 の範囲外の場合 [`ChartError::InvalidTickTarget`]。
    pub fn ticks(&self, target: usize) -> Result<Vec<f64>, ChartError> {
        if target == 0 || target > 50 {
            return Err(ChartError::InvalidTickTarget);
        }
        let (d0, d1) = self.domain;
        let (lo, hi) = if d0 <= d1 { (d0, d1) } else { (d1, d0) };
        let step = nice_step((hi - lo) / target as f64);

        let first = (lo / step).ceil() * step;
        let last = (hi / step).floor() * step;

        // target<=50 かつ first/last は domain 由来のため本数は概ね target
        // 程度に収まるが、浮動小数点誤差の影響を排除するため上限を明示する
        // （無限ループ・過大割当の構造的排除、`.claude/rules/security.md` A04）。
        const MAX_TICKS: usize = 1000;
        let mut values = Vec::new();
        let mut i = 0usize;
        loop {
            let v = first + i as f64 * step;
            if v > last + step / 2.0 || i >= MAX_TICKS {
                break;
            }
            values.push(v);
            i += 1;
        }

        if d0 > d1 {
            values.reverse();
        }
        Ok(values)
    }
}

/// 生の刻み幅 `raw_step`（正の有限値）を、d3 `d3-array` の `tickStep` と
/// 同じアルゴリズムで「nice」な値（`{1, 2, 5, 10} × 10^k` のいずれか）へ
/// 丸める（内部ヘルパ）。
///
/// アルゴリズム: `raw_step` の桁（10 のべき乗、`magnitude = 10^floor(log10(raw_step))`）
/// を取り出し、正規化した誤差 `error = raw_step / magnitude`（1.0..=10.0 の
/// 範囲）を `sqrt(2)`/`sqrt(10)`/`sqrt(50)` の 3 閾値と比較して倍率
/// `{1, 2, 5, 10}` を選ぶ（単純な `{1,2,5,10}` の算術中間点ではなく幾何平均
/// 相当の閾値を使うことで、期待 tick 本数から実際の本数が体系的に増減しない
/// よう補正する、d3 が採用する既知の設計）。
fn nice_step(raw_step: f64) -> f64 {
    debug_assert!(raw_step.is_finite() && raw_step > 0.0);
    let magnitude = 10f64.powf(raw_step.log10().floor());
    let error = raw_step / magnitude;
    let multiplier = if error >= 50f64.sqrt() {
        10.0
    } else if error >= 10f64.sqrt() {
        5.0
    } else if error >= 2f64.sqrt() {
        2.0
    } else {
        1.0
    };
    multiplier * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_non_finite_domain_or_range() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                LinearScale::new((bad, 1.0), (0.0, 100.0)).unwrap_err(),
                ChartError::NonFiniteValue
            );
            assert_eq!(
                LinearScale::new((0.0, 1.0), (bad, 100.0)).unwrap_err(),
                ChartError::NonFiniteValue
            );
        }
    }

    #[test]
    fn new_rejects_degenerate_domain() {
        assert_eq!(
            LinearScale::new((5.0, 5.0), (0.0, 100.0)).unwrap_err(),
            ChartError::DegenerateDomain
        );
    }

    #[test]
    fn scale_maps_domain_to_range_linearly() {
        let s = LinearScale::new((0.0, 100.0), (0.0, 200.0)).unwrap();
        assert_eq!(s.scale(0.0), 0.0);
        assert_eq!(s.scale(100.0), 200.0);
        assert_eq!(s.scale(50.0), 100.0);
        // domain 外でも外挿する。
        assert_eq!(s.scale(150.0), 300.0);
    }

    #[test]
    fn scale_supports_inverted_range_for_svg_y_axis() {
        // SVG は y が下向き正のため、データの大小を上下反転させる典型例。
        let s = LinearScale::new((0.0, 100.0), (200.0, 0.0)).unwrap();
        assert_eq!(s.scale(0.0), 200.0);
        assert_eq!(s.scale(100.0), 0.0);
        assert_eq!(s.scale(50.0), 100.0);
    }

    #[test]
    fn ticks_rejects_out_of_range_target() {
        let s = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
        assert_eq!(s.ticks(0).unwrap_err(), ChartError::InvalidTickTarget);
        assert_eq!(s.ticks(51).unwrap_err(), ChartError::InvalidTickTarget);
        assert!(s.ticks(1).is_ok());
        assert!(s.ticks(50).is_ok());
    }

    #[test]
    fn ticks_known_domain_produces_expected_nice_values() {
        let s = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
        assert_eq!(
            s.ticks(5).unwrap(),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        );
    }

    #[test]
    fn ticks_negative_domain_produces_expected_nice_values() {
        let s = LinearScale::new((-50.0, 75.0), (0.0, 100.0)).unwrap();
        let ticks = s.ticks(5).unwrap();
        assert_eq!(ticks, vec![-40.0, -20.0, 0.0, 20.0, 40.0, 60.0]);
    }

    #[test]
    fn ticks_are_deterministic_across_repeated_calls() {
        let s = LinearScale::new((0.013, 0.987), (0.0, 100.0)).unwrap();
        assert_eq!(s.ticks(5).unwrap(), s.ticks(5).unwrap());
    }

    #[test]
    fn ticks_huge_domain_stays_within_iteration_cap() {
        let s = LinearScale::new((0.0, 1e12), (0.0, 100.0)).unwrap();
        let ticks = s.ticks(10).unwrap();
        assert!(ticks.len() <= 12);
    }

    #[test]
    fn nice_extends_domain_outward_to_1_2_5_boundaries() {
        let s = LinearScale::new((3.0, 97.0), (0.0, 100.0)).unwrap();
        let niced = s.nice();
        assert_eq!(niced.domain(), (0.0, 100.0));
        // range は不変。
        assert_eq!(niced.range(), (0.0, 100.0));
    }

    #[test]
    fn nice_preserves_inverted_domain_direction() {
        let s = LinearScale::new((97.0, 3.0), (0.0, 100.0)).unwrap();
        let niced = s.nice();
        assert_eq!(niced.domain(), (100.0, 0.0));
    }
}
