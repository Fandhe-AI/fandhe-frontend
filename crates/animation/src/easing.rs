//! CSS 相当の決定論的イージング関数（cubic-bezier / steps）とサンプリング。
//!
//! `fandhe-frontend-animation`（Web アダプタ）がフレーム進捗 `t` に適用し、
//! `linear()` 用の数値列生成は #2381（spring 近似 easing プリセット）が
//! [`sample`] を呼び出す。本モジュールは CSS 文字列を一切生成しない
//! （数値計算のみ。`Declaration::value` の `&'static str` 契約は変更しない）。

/// CSS `cubic-bezier(x1, y1, x2, y2)` タイミング関数。
///
/// x1/x2 は CSS 仕様上 `[0, 1]` に限定される（y1/y2 はオーバーシュート
/// 表現のため範囲外を許容）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl CubicBezier {
    /// CSS `ease` キーワード相当。
    pub const EASE: Self = Self {
        x1: 0.25,
        y1: 0.1,
        x2: 0.25,
        y2: 1.0,
    };
    /// CSS `ease-in` キーワード相当。
    pub const EASE_IN: Self = Self {
        x1: 0.42,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
    /// CSS `ease-out` キーワード相当。
    pub const EASE_OUT: Self = Self {
        x1: 0.0,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };
    /// CSS `ease-in-out` キーワード相当。
    pub const EASE_IN_OUT: Self = Self {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };

    /// x1/x2 が `[0,1]` の外、またはいずれかの引数が非有限値のとき `None`。
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Option<Self> {
        if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
            return None;
        }
        if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            return None;
        }
        Some(Self { x1, y1, x2, y2 })
    }

    fn bezier(a0: f64, a1: f64, a2: f64, s: f64) -> f64 {
        // 3 次ベジェ曲線 B(s) = 3(1-s)^2 s a0 + 3(1-s) s^2 a1 + s^3 a2
        // （端点は (0,0)/(1,1) 固定のため a0/a2 は制御点、a1 は終点側の係数）
        let s2 = s * s;
        let s3 = s2 * s;
        let mt = 1.0 - s;
        let mt2 = mt * mt;
        3.0 * mt2 * s * a0 + 3.0 * mt * s2 * a1 + s3 * a2
    }

    fn bezier_derivative(a0: f64, a1: f64, s: f64) -> f64 {
        // 上記 B(s) の导 dB/ds（端点 (0,0)/(1,1) 固定の特殊形）
        let s2 = s * s;
        3.0 * (1.0 - 4.0 * s + 3.0 * s2) * a0 + 3.0 * (2.0 * s - 3.0 * s2) * a1 + 3.0 * s2
    }

    /// `t` を `[0, 1]` へ clamp し、`x(s) = t` を解いた `s` から `y(s)` を返す。
    ///
    /// WebKit `UnitBezier` と同型: Newton 法（最大 8 反復・許容誤差 `1e-7`）
    /// を試み、収束しなければ二分探索へフォールバックする（決定的・
    /// 超越関数不要）。
    pub fn evaluate(&self, t: f64) -> f64 {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        if t == 0.0 || t == 1.0 {
            return t;
        }

        let mut s = t;
        let mut converged = false;
        for _ in 0..8 {
            let x = Self::bezier(self.x1, self.x2, 1.0, s) - t;
            if x.abs() < 1e-7 {
                converged = true;
                break;
            }
            let dx = Self::bezier_derivative(self.x1, self.x2, s);
            if dx.abs() < 1e-12 {
                break;
            }
            s -= x / dx;
        }

        if !converged || !(0.0..=1.0).contains(&s) {
            // Newton 法が発散・範囲外に出た場合の決定的フォールバック
            let mut lo = 0.0_f64;
            let mut hi = 1.0_f64;
            s = t;
            for _ in 0..60 {
                let x = Self::bezier(self.x1, self.x2, 1.0, s);
                if (x - t).abs() < 1e-7 {
                    break;
                }
                if x < t {
                    lo = s;
                } else {
                    hi = s;
                }
                s = (lo + hi) / 2.0;
            }
        }

        // y はオーバーシュート表現のため [0,1] へは clamp しない（t のみ clamp）
        Self::bezier(self.y1, self.y2, 1.0, s)
    }
}

/// CSS `steps()` の jump 位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepPosition {
    /// `jump-start`（CSS `start` の別名）。
    JumpStart,
    /// `jump-end`（CSS `end` の別名、既定）。
    JumpEnd,
    /// `jump-none`。
    JumpNone,
    /// `jump-both`。
    JumpBoth,
}

/// CSS `steps(count, position)` タイミング関数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Steps {
    count: u32,
    position: StepPosition,
}

impl Steps {
    /// `count == 0`、または `JumpNone` かつ `count < 2` のとき `None`
    /// （CSS Easing 仕様上の不正値）。
    pub fn new(count: u32, position: StepPosition) -> Option<Self> {
        if count == 0 {
            return None;
        }
        if position == StepPosition::JumpNone && count < 2 {
            return None;
        }
        Some(Self { count, position })
    }

    /// CSS Easing Level 1 の step easing function アルゴリズムに従う評価。
    ///
    /// `t` は `[0, 1]` へ clamp する。分母は `JumpNone` で `count - 1`、
    /// `JumpBoth` で `count + 1`、それ以外は `count`。
    pub fn evaluate(&self, t: f64) -> f64 {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let count = f64::from(self.count);

        let mut step = (t * count).floor();
        if matches!(
            self.position,
            StepPosition::JumpStart | StepPosition::JumpBoth
        ) {
            step += 1.0;
        }
        if t >= 1.0 {
            let max_step = match self.position {
                StepPosition::JumpNone => count - 1.0,
                StepPosition::JumpBoth => count + 1.0,
                StepPosition::JumpStart | StepPosition::JumpEnd => count,
            };
            step = max_step;
        }

        let denominator = match self.position {
            StepPosition::JumpNone => count - 1.0,
            StepPosition::JumpBoth => count + 1.0,
            StepPosition::JumpStart | StepPosition::JumpEnd => count,
        };

        (step / denominator).clamp(0.0, 1.0)
    }
}

/// 恒等イージング（線形）。既定 easing・サンプリングの自明ケースに使う。
pub fn linear(t: f64) -> f64 {
    t
}

/// `easing` を `t = i / (n - 1)`（`i = 0..n`）で等間隔に `n` 点サンプリングする。
///
/// `n < 2` は両端を含む区間分割が定義できないため空の `Vec` を返す。
pub fn sample(easing: impl Fn(f64) -> f64, n: usize) -> Vec<f64> {
    if n < 2 {
        return Vec::new();
    }
    let last = (n - 1) as f64;
    (0..n).map(|i| easing(i as f64 / last)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn cubic_bezier_known_values() {
        let eio = CubicBezier::EASE_IN_OUT;
        assert!(approx(eio.evaluate(0.5), 0.5, 1e-6));
        assert!(approx(eio.evaluate(0.25), 0.129_162, 1e-5));
        assert!(approx(eio.evaluate(0.75), 0.870_838, 1e-5));

        assert!(approx(CubicBezier::EASE.evaluate(0.5), 0.802_403, 1e-5));
        assert!(approx(CubicBezier::EASE_IN.evaluate(0.5), 0.315_357, 1e-5));
        assert!(approx(CubicBezier::EASE_OUT.evaluate(0.5), 0.684_643, 1e-5));

        let standard = CubicBezier::new(0.4, 0.0, 0.2, 1.0).unwrap();
        assert!(approx(standard.evaluate(0.5), 0.775_561, 1e-5));
        let emphasized = CubicBezier::new(0.2, 0.0, 0.0, 1.0).unwrap();
        assert!(approx(emphasized.evaluate(0.5), 0.877_834, 1e-5));
    }

    #[test]
    fn cubic_bezier_identity_and_endpoints() {
        let identity = CubicBezier::new(0.0, 0.0, 1.0, 1.0).unwrap();
        for t in [0.0, 0.1, 0.37, 0.5, 0.9, 1.0] {
            assert!(approx(identity.evaluate(t), t, 1e-6));
        }
        assert_eq!(CubicBezier::EASE.evaluate(0.0), 0.0);
        assert_eq!(CubicBezier::EASE.evaluate(1.0), 1.0);
    }

    #[test]
    fn cubic_bezier_clamps_out_of_range_t() {
        assert_eq!(CubicBezier::EASE.evaluate(-1.0), 0.0);
        assert_eq!(CubicBezier::EASE.evaluate(2.0), 1.0);
    }

    #[test]
    fn cubic_bezier_symmetry() {
        for t in [0.1, 0.3, 0.5, 0.7, 0.9] {
            let a = CubicBezier::EASE_IN.evaluate(t);
            let b = 1.0 - CubicBezier::EASE_OUT.evaluate(1.0 - t);
            assert!(approx(a, b, 1e-5), "t={t} a={a} b={b}");
        }
    }

    #[test]
    fn cubic_bezier_rejects_invalid_x() {
        assert!(CubicBezier::new(1.5, 0.0, 0.5, 1.0).is_none());
        assert!(CubicBezier::new(0.5, 0.0, -0.1, 1.0).is_none());
        assert!(CubicBezier::new(f64::NAN, 0.0, 0.5, 1.0).is_none());
        // y は範囲外（オーバーシュート）でも許容する
        assert!(CubicBezier::new(0.5, -0.5, 0.5, 1.5).is_some());
    }

    #[test]
    fn steps_jump_end() {
        let s = Steps::new(4, StepPosition::JumpEnd).unwrap();
        assert_eq!(s.evaluate(0.0), 0.0);
        assert_eq!(s.evaluate(0.24), 0.0);
        assert_eq!(s.evaluate(0.25), 0.25);
        assert_eq!(s.evaluate(1.0), 1.0);
    }

    #[test]
    fn steps_jump_start() {
        let s = Steps::new(4, StepPosition::JumpStart).unwrap();
        assert_eq!(s.evaluate(0.0), 0.25);
        assert_eq!(s.evaluate(1.0), 1.0);
    }

    #[test]
    fn steps_jump_none() {
        let s = Steps::new(4, StepPosition::JumpNone).unwrap();
        assert_eq!(s.evaluate(0.0), 0.0);
        assert!(approx(s.evaluate(0.5), 2.0 / 3.0, 1e-9));
        assert_eq!(s.evaluate(1.0), 1.0);
    }

    #[test]
    fn steps_jump_both() {
        let s = Steps::new(4, StepPosition::JumpBoth).unwrap();
        assert!(approx(s.evaluate(0.0), 0.2, 1e-9));
        assert_eq!(s.evaluate(1.0), 1.0);
    }

    #[test]
    fn steps_rejects_invalid_count() {
        assert!(Steps::new(0, StepPosition::JumpEnd).is_none());
        assert!(Steps::new(1, StepPosition::JumpNone).is_none());
    }

    #[test]
    fn sample_linear() {
        assert_eq!(sample(linear, 5), vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn sample_length_and_endpoints() {
        let easing = |t: f64| CubicBezier::EASE_IN_OUT.evaluate(t);
        let points = sample(easing, 3);
        assert_eq!(points.len(), 3);
        assert!(approx(points[0], easing(0.0), 1e-12));
        assert!(approx(points[2], easing(1.0), 1e-12));
    }

    #[test]
    fn sample_below_minimum_is_empty() {
        assert!(sample(linear, 0).is_empty());
        assert!(sample(linear, 1).is_empty());
    }

    #[test]
    fn evaluate_is_deterministic() {
        let bezier = CubicBezier::new(0.3, 0.7, 0.6, 0.2).unwrap();
        assert_eq!(bezier.evaluate(0.37), bezier.evaluate(0.37));
        let steps = Steps::new(6, StepPosition::JumpBoth).unwrap();
        assert_eq!(steps.evaluate(0.42), steps.evaluate(0.42));
    }
}
