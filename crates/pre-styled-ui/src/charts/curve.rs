//! 曲線補間ジオメトリ（イシュー #2081、shadcn/ui Charts（area）突合）。
//!
//! [`crate::area_chart`] の `curve: AreaCurve::Natural`/`AreaCurve::Step`
//! バリアントが消費する純関数のみを置く。本モジュール自体は SVG ノード木を
//! 組み立てず、`(x, y)` 座標列を受け取って別の `(x, y)` 座標列・制御点列を
//! 返すだけの決定的なジオメトリ計算に閉じる（[`super::svg::PathBuilder`]
//! への変換は呼び出し元 [`crate::area_chart`] が行う）。line-chart 側の
//! 同種補間（#2083）が先に本モジュールを必要とした場合は重複実装せず
//! ここへ合流する。
//!
//! # natural spline（[`natural_control_points`]）
//!
//! d3-shape `curveNatural` と同じ「自然三次スプライン」アルゴリズム
//! （3 重対角行列を Thomas 法で解き、各区間の 3 次 Bézier 制御点を求める）
//! を x 列・y 列それぞれに独立して適用する。境界条件は自然（両端の
//! 二階微分 = 0）。3 点未満（区間 1 個未満）では定義できないため、
//! 呼び出し元が `points.len() >= 3` を保証する契約とする
//! （[`crate::area_chart`] は `n == 2` を直線へ退化させ本関数を呼ばない）。
//!
//! # step（[`step_points`]）
//!
//! d3-shape `curveStep`（区間中点で段差になる版）相当。各区間 `k` の中点
//! `m_k = (x_k + x_{k+1}) / 2` を経由し、`(m_k, y_k) → (m_k, y_{k+1})` の
//! 2 点を追加した後、最終点 `(x_{n-1}, y_{n-1})` で締める。

/// 1 次元数列 `v`（`x` 列または `y` 列）に対する natural cubic spline の
/// 制御点対 `(a, b)` を返す（内部ヘルパ、d3-shape `natural.js` の
/// `controlPoints` を直訳）。`v.len() >= 3` を契約とする（呼び出し元
/// [`natural_control_points`] が保証する）。
fn control_points_1d(v: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = v.len() - 1;
    let mut a = vec![0.0; n];
    let mut b = vec![0.0; n];
    let mut r = vec![0.0; n];

    a[0] = 0.0;
    b[0] = 2.0;
    r[0] = v[0] + 2.0 * v[1];
    for i in 1..n.saturating_sub(1) {
        a[i] = 1.0;
        b[i] = 4.0;
        r[i] = 4.0 * v[i] + 2.0 * v[i + 1];
    }
    a[n - 1] = 2.0;
    b[n - 1] = 7.0;
    r[n - 1] = 8.0 * v[n - 1] + v[n];

    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m;
        r[i] -= m * r[i - 1];
    }

    a[n - 1] = r[n - 1] / b[n - 1];
    for i in (0..n - 1).rev() {
        a[i] = (r[i] - a[i + 1]) / b[i];
    }
    b[n - 1] = (v[n] + a[n - 1]) / 2.0;
    for i in 0..n - 1 {
        b[i] = 2.0 * v[i + 1] - a[i + 1];
    }

    (a, b)
}

/// natural spline の区間ごとの 3 次 Bézier 制御点対を返す。
///
/// 戻り値は `points.len() - 1` 個の要素を持ち、各要素 `(cp1, cp2)` は
/// `points[i]` から `points[i + 1]` への区間の制御点（呼び出し元は
/// `PathBuilder::cubic_to(cp1.0, cp1.1, cp2.0, cp2.1, points[i+1].0, points[i+1].1)`
/// として使う）。
///
/// `points.len() < 3` の場合は区間が定義できないため空配列を返す
/// （呼び出し元契約違反の silent no-op。呼び出し元 [`crate::area_chart`]
/// は `n == 2` を直線へ退化させ本関数を呼ばない契約のため、通常経路では
/// 到達しない）。
#[must_use]
pub(crate) fn natural_control_points(points: &[(f64, f64)]) -> Vec<((f64, f64), (f64, f64))> {
    if points.len() < 3 {
        return Vec::new();
    }

    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let (ax, bx) = control_points_1d(&xs);
    let (ay, by) = control_points_1d(&ys);

    (0..points.len() - 1)
        .map(|i| ((ax[i], ay[i]), (bx[i], by[i])))
        .collect()
}

/// step（区間中点で段差になる）補間の line-to 座標列を返す。
///
/// 先頭点（`points[0]`）は呼び出し元が既に `move_to` 済みの前提で含めず、
/// 各区間の中点往復 2 点 + 最終点で締める列を返す（モジュール doc
/// 「step」節参照）。
#[must_use]
pub(crate) fn step_points(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for w in points.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        let mid = (x0 + x1) / 2.0;
        out.push((mid, y0));
        out.push((mid, y1));
    }
    if let Some(&last) = points.last() {
        out.push(last);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_control_points_returns_one_pair_per_segment() {
        let points = vec![(0.0, 0.0), (1.0, 2.0), (2.0, 0.0), (3.0, 2.0)];
        let cps = natural_control_points(&points);
        assert_eq!(cps.len(), points.len() - 1);
    }

    #[test]
    fn natural_control_points_is_deterministic() {
        let points = vec![(0.0, 1.0), (1.0, 5.0), (2.0, 2.0), (3.0, 8.0), (4.0, 3.0)];
        let a = natural_control_points(&points);
        let b = natural_control_points(&points);
        assert_eq!(a, b);
    }

    #[test]
    fn natural_control_points_symmetric_for_symmetric_input() {
        // y が線対称（山型）な入力では、区間の制御点も対称に現れることを
        // 固定する（自然境界条件の対称性チェック、浮動小数の桁落ちを
        // 考慮し許容誤差付きで比較する）。
        let points = vec![(0.0, 0.0), (1.0, 4.0), (2.0, 0.0)];
        let cps = natural_control_points(&points);
        assert_eq!(cps.len(), 2);
        let ((cp1a, cp1b), (cp2a, cp2b)) = (cps[0], cps[1]);
        assert!((cp1a.0 - (2.0 - cp2b.0)).abs() < 1e-9);
        assert!((cp1a.1 - cp2b.1).abs() < 1e-9);
        assert!((cp1b.0 - (2.0 - cp2a.0)).abs() < 1e-9);
        assert!((cp1b.1 - cp2a.1).abs() < 1e-9);
    }

    #[test]
    fn natural_control_points_below_three_points_returns_empty() {
        assert_eq!(natural_control_points(&[]), Vec::new());
        assert_eq!(natural_control_points(&[(0.0, 0.0)]), Vec::new());
        assert_eq!(
            natural_control_points(&[(0.0, 0.0), (1.0, 1.0)]),
            Vec::new()
        );
    }

    #[test]
    fn step_points_visits_midpoint_before_and_after_each_transition() {
        let points = vec![(0.0, 0.0), (10.0, 5.0), (20.0, 2.0)];
        let out = step_points(&points);
        assert_eq!(
            out,
            vec![
                (5.0, 0.0),
                (5.0, 5.0),
                (15.0, 5.0),
                (15.0, 2.0),
                (20.0, 2.0)
            ]
        );
    }

    #[test]
    fn step_points_is_deterministic() {
        let points = vec![(0.0, 0.0), (1.0, 3.0), (2.0, 1.0)];
        assert_eq!(step_points(&points), step_points(&points));
    }
}
