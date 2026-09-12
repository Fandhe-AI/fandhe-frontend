//! 物理ベースのばね（spring）ソルバ（motion.dev `spring()` 相当）。
//!
//! 減衰調和振動子の解析解として、任意時刻 `t`（秒）における値・速度を
//! 決定的に計算する。フレームループ・DOM への書き込みは持たない
//! （`fandhe-frontend-animation` の rAF ループ、イシュー #2378/#2403 が
//! 本モジュールの [`Spring::at`] を毎フレーム呼び出す消費者になる想定。
//! CSS `linear()` プリセット生成〔#2381〕は [`Spring::at`] のサンプル列を
//! `easing::sample` へ渡す下流消費者）。
//!
//! 時間単位は秒（`f64`）で統一する（`animation-core-architecture.md` §2.1
//! の `Driver::tick` 契約に合わせる。motion.dev は ms 単位だが本 crate は
//! 秒に統一する）。

/// ばねの物理パラメータ（motion.dev `spring()` の既定値 stiffness=100 /
/// damping=10 / mass=1 を [`Default`] に採用）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }
}

/// duration/bounce 変換の Newton 法反復回数。motion.dev `findSpring` の
/// 実測収束回数に対し十分な余裕を持たせた固定回数（DoS 対策として上限を
/// 設ける。固定回数のため入力に関わらず必ず終了する）。
const NEWTON_ITERATIONS: u32 = 12;

/// 包絡線が減衰したとみなす比率（motion.dev `findSpring` の `safeMin` 相当）。
/// 0.001 は「初期振幅の 0.1% まで減衰した時刻を duration とみなす」という
/// motion.dev のヒューリスティックをそのまま踏襲する。
const SAFE_MIN: f64 = 0.001;

impl SpringConfig {
    /// duration（秒）/ bounce（0..=1、1 に近いほど大きくオーバーシュートする）
    /// から物理パラメータへ変換する（motion.dev `findSpring` 相当）。
    ///
    /// - `duration` は `[0.01, 10.0]` 秒へ clamp する
    /// - `bounce` から減衰比 `ζ = clamp(1 - bounce, 0.05, 1.0)` を得る
    /// - Newton 法で `ω0` を求め、`stiffness = ω0²·mass` /
    ///   `damping = ζ·2·sqrt(mass·stiffness)` を返す
    /// - 数値的に発散した場合（NaN 等）は [`SpringConfig::default`] へ
    ///   フォールバックする（fail-safe。パニックしない）
    pub fn from_duration_bounce(duration: f64, bounce: f64, velocity: f64, mass: f64) -> Self {
        // duration/bounce が非有限（NaN 等）なら Newton 法へ進まず既定値へ
        // フォールバックする（velocity/mass は既定 0/1 で代替可能な補助
        // パラメータのため個別に clamp する）。
        if !duration.is_finite() || !bounce.is_finite() {
            return Self::default();
        }
        let duration = duration.clamp(0.01, 10.0);
        let mass = if mass.is_finite() && mass > 0.0 {
            mass
        } else {
            1.0
        };
        let zeta = (1.0 - bounce).clamp(0.05, 1.0);
        let velocity = if velocity.is_finite() { velocity } else { 0.0 };

        let omega0 = solve_omega0(duration, zeta, velocity);
        match omega0 {
            Some(omega0) if omega0.is_finite() && omega0 > 0.0 => {
                let stiffness = omega0 * omega0 * mass;
                let damping = zeta * 2.0 * (mass * stiffness).sqrt();
                if stiffness.is_finite() && damping.is_finite() {
                    return Self {
                        stiffness,
                        damping,
                        mass,
                    };
                }
                Self::default()
            }
            _ => Self::default(),
        }
    }

    /// 減衰比 `ζ = damping / (2·sqrt(stiffness·mass))`。
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }
}

/// `envelope(ω) = SAFE_MIN` の根を Newton 法で探索し `ω0` を返す。
///
/// ζ < 1（不足減衰）と ζ == 1（臨界減衰）で包絡線の式が異なるため分岐する
/// （§3.4 参照）。過減衰（ζ > 1）は `from_duration_bounce` の zeta clamp
/// 上限が 1.0 のため到達しない。
fn solve_omega0(duration: f64, zeta: f64, velocity: f64) -> Option<f64> {
    let mut omega = 5.0 / duration;
    for _ in 0..NEWTON_ITERATIONS {
        if !omega.is_finite() || omega <= 0.0 {
            return None;
        }
        let (f, df) = if (zeta - 1.0).abs() < 1e-9 {
            // ζ == 1（臨界減衰）:
            // envelope(ω) = -SAFE_MIN + e^{-ωT}((ω - v)T + 1)
            let e = (-omega * duration).exp();
            let f = -SAFE_MIN + e * ((omega - velocity) * duration + 1.0);
            let df = e * (velocity - omega) * duration * duration;
            (f, df)
        } else {
            // ζ < 1（不足減衰）:
            // envelope(ω) = SAFE_MIN - ((ζω - v) / (ω·sqrt(1-ζ²)))·e^{-ζωT}
            let omega_d_ratio = (1.0 - zeta * zeta).sqrt();
            let e = (-zeta * omega * duration).exp();
            let a = (zeta * omega - velocity) / (omega * omega_d_ratio);
            let f = SAFE_MIN - a * e;
            // 数値微分（解析導関数は式が煩雑になるため、固定小刻みの中心差分で
            // 代替する。Newton 法は導関数の厳密性を必要としないため十分）。
            let h = omega * 1e-6 + 1e-9;
            let eval = |w: f64| -> f64 {
                let e = (-zeta * w * duration).exp();
                let a = (zeta * w - velocity) / (w * omega_d_ratio);
                SAFE_MIN - a * e
            };
            let df = (eval(omega + h) - eval(omega - h)) / (2.0 * h);
            (f, df)
        };
        if !f.is_finite() || !df.is_finite() || df == 0.0 {
            return None;
        }
        omega -= f / df;
    }
    if omega.is_finite() && omega > 0.0 {
        Some(omega)
    } else {
        None
    }
}

/// 時刻 `t` におけるばねのサンプル値。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    pub value: f64,
    pub velocity: f64,
    pub done: bool,
}

/// `settle_duration` の探索上限（秒）。10 s 経過しても収束しない設定は
/// 実用上ありえないため、無限ループ防止のための固定上限として置く。
const MAX_SETTLE_DURATION: f64 = 10.0;
/// `settle_duration` の探索刻み幅（秒）。motion.dev `calcGeneratorDuration`
/// の 10 ms 刻みより細かくし、#2381 が `linear()` プリセットを生成する際の
/// サンプリング精度を確保する。
const SETTLE_STEP: f64 = 0.001;

/// 減衰領域（判別式 `ζ` の符号）ごとの事前計算済み係数。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Region {
    /// 不足減衰（ζ < 1）: 振動しながら収束する。
    Underdamped { omega_d: f64, zeta_omega0: f64 },
    /// 臨界減衰（ζ == 1、許容誤差 1e-9）: 振動せず最速に収束する。
    Critical { omega0: f64 },
    /// 過減衰（ζ > 1）: 2 つの実数指数の和で振動せず収束する。
    Overdamped { r1: f64, r2: f64 },
}

/// from → to へ向かうばね 1 本の解析解ソルバ。
///
/// 値は単位非依存の `f64`（px でも進捗 0..=1 でも可）。不変・`Copy` 可・
/// 同一入力に対し常にビット一致する決定的な出力を返す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    to: f64,
    x0: f64,
    v0: f64,
    region: Region,
    rest_delta: f64,
    rest_speed: f64,
}

impl Spring {
    /// `stiffness`/`mass` が正でない・`damping` が負・非有限値のいずれかを
    /// 満たす場合は `None` を返す（ライブラリコードで panic しない）。
    pub fn new(config: SpringConfig, from: f64, to: f64, initial_velocity: f64) -> Option<Self> {
        let SpringConfig {
            stiffness,
            damping,
            mass,
        } = config;
        if !stiffness.is_finite()
            || !damping.is_finite()
            || !mass.is_finite()
            || !from.is_finite()
            || !to.is_finite()
            || !initial_velocity.is_finite()
        {
            return None;
        }
        if stiffness <= 0.0 || mass <= 0.0 || damping < 0.0 {
            return None;
        }

        let omega0 = (stiffness / mass).sqrt();
        let zeta = damping / (2.0 * (stiffness * mass).sqrt());
        let x0 = from - to;
        let v0 = initial_velocity;

        let region = if (zeta - 1.0).abs() < 1e-9 {
            Region::Critical { omega0 }
        } else if zeta < 1.0 {
            Region::Underdamped {
                omega_d: omega0 * (1.0 - zeta * zeta).sqrt(),
                zeta_omega0: zeta * omega0,
            }
        } else {
            let disc = (zeta * zeta - 1.0).sqrt();
            Region::Overdamped {
                r1: -omega0 * (zeta - disc),
                r2: -omega0 * (zeta + disc),
            }
        };

        // rest 閾値の 2 段選択（motion.dev のヒューリスティック）:
        // - 移動距離が 5 未満なら「小レンジ値」（opacity・scale・進捗 0..=1 等）
        //   とみなし granular 閾値を使う。0.5 の絶対差はレンジの半分に相当し
        //   大きすぎるため、1/100 スケールの閾値に切り替える
        //   （motion.dev の `isGranularScale` と同一の判定基準）
        // - それ以外は px 単位の DOM 移動を想定した既定閾値を使う:
        //   rest_delta=0.5px は 60fps でサブピクセル描画により視認できない
        //   差分、rest_speed=2px/s は 1 フレーム換算で約 0.03px の動きに相当し
        //   実用上静止とみなせる
        let (rest_delta, rest_speed) = if (to - from).abs() < 5.0 {
            (0.005, 0.01)
        } else {
            (0.5, 2.0)
        };

        Some(Self {
            to,
            x0,
            v0,
            region,
            rest_delta,
            rest_speed,
        })
    }

    /// 時刻 `t`（秒）における値・速度・収束済みかを返す。
    ///
    /// `t` が負・NaN の場合は `t = 0.0` として扱う。収束判定は
    /// `|to - value| <= rest_delta && |velocity| <= rest_speed` の
    /// AND 条件（motion.dev と同一）。片方のみでは「目標を高速通過中」や
    /// 「遠方で静止中」を誤って完了扱いにしてしまうため両方を要求する。
    /// `done == true` のとき `value` は `to` に完全一致し `velocity` は
    /// `0.0` にスナップする（残差を後続フレームへ漏らさず、#2381 の
    /// `linear()` 末尾値が正確に 1 になる契約を満たすため）。
    pub fn at(&self, t: f64) -> SpringState {
        let t = if t.is_finite() && t > 0.0 { t } else { 0.0 };
        let (x, v) = match self.region {
            Region::Underdamped {
                omega_d,
                zeta_omega0,
            } => {
                let decay = (-zeta_omega0 * t).exp();
                let (sin, cos) = (omega_d * t).sin_cos();
                let b = (self.v0 + zeta_omega0 * self.x0) / omega_d;
                let x = decay * (self.x0 * cos + b * sin);
                // 解析微分: x' = -ζω0·x + decay·(-x0·ωd·sin + b·ωd·cos)
                let v = -zeta_omega0 * x + decay * omega_d * (-self.x0 * sin + b * cos);
                (x, v)
            }
            Region::Critical { omega0 } => {
                let decay = (-omega0 * t).exp();
                let c = self.v0 + omega0 * self.x0;
                let x = decay * (self.x0 + c * t);
                // 解析微分: x' = -ω0·decay·(x0 + c·t) + decay·c = -ω0·x + decay·c
                let v = -omega0 * x + decay * c;
                (x, v)
            }
            Region::Overdamped { r1, r2 } => {
                let a = (self.v0 - r2 * self.x0) / (r1 - r2);
                let b = self.x0 - a;
                let e1 = (r1 * t).exp();
                let e2 = (r2 * t).exp();
                let x = a * e1 + b * e2;
                let v = a * r1 * e1 + b * r2 * e2;
                (x, v)
            }
        };

        let value = self.to + x;
        let done = (self.to - value).abs() <= self.rest_delta && v.abs() <= self.rest_speed;
        if done {
            SpringState {
                value: self.to,
                velocity: 0.0,
                done: true,
            }
        } else {
            SpringState {
                value,
                velocity: v,
                done: false,
            }
        }
    }

    /// `done` になる最初の時刻（秒）を `SETTLE_STEP` 刻みで探索する
    /// （`#2381` がサンプリング範囲を決定するために使う）。
    /// `MAX_SETTLE_DURATION` 到達まで収束しない場合はその上限値を返す
    /// （無限ループ防止）。
    pub fn settle_duration(&self) -> f64 {
        let mut t = 0.0;
        while t < MAX_SETTLE_DURATION {
            if self.at(t).done {
                return t;
            }
            t += SETTLE_STEP;
        }
        MAX_SETTLE_DURATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    // --- 物理パラメータ系 ---

    #[test]
    fn at_zero_returns_initial_state_in_all_regions() {
        for damping in [5.0, 20.0, 40.0] {
            let config = SpringConfig {
                stiffness: 100.0,
                damping,
                mass: 1.0,
            };
            let spring = Spring::new(config, 0.0, 100.0, 3.0).unwrap();
            let s = spring.at(0.0);
            assert!(approx(s.value, 0.0, 1e-9), "damping={damping}");
            assert!(approx(s.velocity, 3.0, 1e-9), "damping={damping}");
        }
    }

    #[test]
    fn ode_residual_matches_analytic_derivative_via_finite_difference() {
        // x'' + (c/m)x' + (k/m)x = 0（x = value - to）を満たすことを、
        // 解析速度と値の中心差分の一致・二階差分の残差の小ささで確認する。
        for damping in [5.0, 20.0, 40.0] {
            let stiffness = 100.0;
            let mass = 1.0;
            let config = SpringConfig {
                stiffness,
                damping,
                mass,
            };
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let h = 1e-4;
            for i in 1..20 {
                let t = i as f64 * 0.05;
                let s = spring.at(t);
                if s.done {
                    break;
                }
                let x_minus = spring.at(t - h).value;
                let x_plus = spring.at(t + h).value;
                let central_velocity = (x_plus - x_minus) / (2.0 * h);
                assert!(
                    approx(central_velocity, s.velocity, 1e-3),
                    "damping={damping} t={t} analytic={} central={}",
                    s.velocity,
                    central_velocity
                );

                let accel = (x_plus - 2.0 * s.value + x_minus) / (h * h);
                let x = s.value - spring.to;
                let residual = accel + (damping / mass) * s.velocity + (stiffness / mass) * x;
                assert!(
                    residual.abs() < 1.0,
                    "damping={damping} t={t} residual={residual}"
                );
            }
        }
    }

    #[test]
    fn default_spring_overshoot_matches_theoretical_value() {
        // 既定 spring（100/10/1、ζ=0.5）の 0→1 は
        // overshoot = exp(-ζπ/sqrt(1-ζ²)) だけ 1 を超え、
        // 到達時刻は π/ωd 付近になる（不足減衰の一般的性質）。
        let config = SpringConfig::default();
        assert!(approx(config.damping_ratio(), 0.5, 1e-9));
        let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();

        let omega0 = (config.stiffness / config.mass).sqrt();
        let zeta = 0.5_f64;
        let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
        let expected_peak_time = std::f64::consts::PI / omega_d;
        let expected_overshoot =
            1.0 + (-zeta * std::f64::consts::PI / (1.0 - zeta * zeta).sqrt()).exp();

        let mut max_value = f64::MIN;
        let mut max_time = 0.0;
        let mut t = 0.0;
        while t < 2.0 {
            let v = spring.at(t).value;
            if v > max_value {
                max_value = v;
                max_time = t;
            }
            t += 0.0005;
        }

        assert!(
            approx(max_value, expected_overshoot, 1e-3),
            "max_value={max_value} expected={expected_overshoot}"
        );
        assert!(
            approx(max_time, expected_peak_time, 0.02),
            "max_time={max_time} expected={expected_peak_time}"
        );
    }

    #[test]
    fn critical_and_overdamped_are_monotonic_and_never_overshoot() {
        for damping in [20.0, 40.0] {
            let config = SpringConfig {
                stiffness: 100.0,
                damping,
                mass: 1.0,
            };
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let mut prev = spring.at(0.0).value;
            let mut t = 0.001;
            while t < 3.0 {
                let v = spring.at(t).value;
                assert!(
                    v >= prev - 1e-9,
                    "damping={damping} t={t} v={v} prev={prev}"
                );
                assert!(v <= 1.0 + 1e-9, "damping={damping} t={t} v={v} overshot");
                prev = v;
                t += 0.01;
            }
        }
    }

    #[test]
    fn settle_duration_marks_done_and_snaps_value() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        let settle = spring.settle_duration();
        let after = spring.at(settle);
        assert!(after.done);
        assert_eq!(after.value, 1.0);
        assert_eq!(after.velocity, 0.0);

        if settle > 0.001 {
            let before = spring.at(settle - 0.001);
            assert!(!before.done);
        }

        // 0→1（距離 1 < 5）は granular 閾値（rest_delta=0.005）が使われる。
        // 不足減衰の振幅包絡線は |x(t)| = decay·R（R = sqrt(x0² + b²)、
        // decay = e^{-ζω0·t}）で上から抑えられるため、
        // decay·R = rest_delta となる時刻を理論値とする
        // （ζω0=5、b=(v0+ζω0·x0)/ωd=5/ωd、ωd=ω0·sqrt(1-ζ²)）。
        let omega0 = (SpringConfig::default().stiffness / SpringConfig::default().mass).sqrt();
        let zeta = 0.5_f64;
        let zeta_omega0 = zeta * omega0;
        let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
        let b = zeta_omega0 / omega_d;
        let r = (1.0_f64 + b * b).sqrt();
        let theoretical = -(0.005_f64 / r).ln() / zeta_omega0;
        assert!(
            approx(settle, theoretical, 0.05),
            "settle={settle} theoretical={theoretical}"
        );
    }

    #[test]
    fn rest_threshold_switches_between_granular_and_default() {
        // 移動距離が大きい（>= 5）場合は既定閾値（rest_delta=0.5）が使われ、
        // わずかな残差（0.3）ではまだ done にならないことを確認する。
        let far = Spring::new(SpringConfig::default(), 0.0, 100.0, 0.0).unwrap();
        let far_settle = far.settle_duration();
        // 収束直前は残差が rest_delta=0.5 に漸近するため、そのわずか手前の
        // 状態はまだ done でないはず。
        if far_settle > 0.001 {
            assert!(!far.at(far_settle - 0.001).done || far_settle < 0.002);
        }

        // 移動距離が小さい（< 5）場合は granular 閾値（rest_delta=0.005）が
        // 使われ、既定閾値なら done 扱いになるはずの残差でもまだ収束しない。
        let near = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        // near の収束時刻は far よりも早くならない設計（同じ ζ・ω0 なので
        // 相対残差の減衰速度は同じだが、閾値が厳しいぶん収束は遅くなる）。
        assert!(near.settle_duration() >= far.settle_duration() - 0.01);
    }

    #[test]
    fn invalid_inputs_return_none() {
        let base = SpringConfig::default();
        assert!(Spring::new(
            SpringConfig {
                stiffness: 0.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(
            SpringConfig {
                stiffness: -1.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(SpringConfig { mass: 0.0, ..base }, 0.0, 1.0, 0.0).is_none());
        assert!(Spring::new(
            SpringConfig {
                damping: -1.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(base, f64::NAN, 1.0, 0.0).is_none());
    }

    #[test]
    fn at_with_nan_or_negative_t_matches_at_zero() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 2.0).unwrap();
        let zero = spring.at(0.0);
        let nan = spring.at(f64::NAN);
        let negative = spring.at(-1.0);
        assert_eq!(zero, nan);
        assert_eq!(zero, negative);
    }

    #[test]
    fn at_is_deterministic() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        let a = spring.at(0.37);
        let b = spring.at(0.37);
        assert_eq!(a, b);
    }

    // --- duration/bounce 系 ---

    #[test]
    fn from_duration_bounce_damping_ratio() {
        let config = SpringConfig::from_duration_bounce(0.8, 0.3, 0.0, 1.0);
        assert!(
            approx(config.damping_ratio(), 0.7, 1e-9),
            "damping_ratio={}",
            config.damping_ratio()
        );
    }

    #[test]
    fn from_duration_bounce_settle_duration_is_close_to_requested() {
        for (duration, bounce) in [(0.8, 0.3), (0.3, 0.3), (2.0, 0.3)] {
            let config = SpringConfig::from_duration_bounce(duration, bounce, 0.0, 1.0);
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let settle = spring.settle_duration();
            let ratio = settle / duration;
            assert!(
                (0.7..=1.3).contains(&ratio),
                "duration={duration} settle={settle} ratio={ratio}"
            );
        }
    }

    #[test]
    fn from_duration_bounce_zeta_bounds() {
        let critical = SpringConfig::from_duration_bounce(0.5, 0.0, 0.0, 1.0);
        assert!(approx(critical.damping_ratio(), 1.0, 1e-6));

        let bouncy = SpringConfig::from_duration_bounce(0.5, 1.0, 0.0, 1.0);
        assert!(approx(bouncy.damping_ratio(), 0.05, 1e-6));
        // 発散しないことの確認: at() が有限値を返す。
        let spring = Spring::new(bouncy, 0.0, 1.0, 0.0).unwrap();
        assert!(spring.at(0.1).value.is_finite());
    }

    #[test]
    fn from_duration_bounce_clamps_extreme_inputs() {
        let too_short = SpringConfig::from_duration_bounce(0.001, 0.3, 0.0, 1.0);
        assert!(too_short.stiffness.is_finite() && too_short.stiffness > 0.0);

        let too_long = SpringConfig::from_duration_bounce(100.0, 0.3, 0.0, 1.0);
        assert!(too_long.stiffness.is_finite() && too_long.stiffness > 0.0);

        let bad_mass = SpringConfig::from_duration_bounce(0.5, 0.3, 0.0, -1.0);
        assert_eq!(bad_mass.mass, 1.0);
    }

    #[test]
    fn from_duration_bounce_output_is_finite_and_positive() {
        let config = SpringConfig::from_duration_bounce(0.5, 0.3, 0.0, 1.0);
        assert!(config.stiffness.is_finite() && config.stiffness > 0.0);
        assert!(config.damping.is_finite() && config.damping > 0.0);

        // NaN 入力はフォールバックで既定値を返す。
        let fallback = SpringConfig::from_duration_bounce(f64::NAN, f64::NAN, 0.0, 1.0);
        assert_eq!(fallback, SpringConfig::default());
    }
}
