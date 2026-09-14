//! confetti パーティクルの決定的物理演算（イシュー #2533）。
//!
//! Motion+ `components/confetti` を Rust へ再実装する。canvas 描画・
//! `requestAnimationFrame` 駆動は本 crate の責務ではなく
//! `fandhe-frontend-animation::confetti`（Web アダプタ、イシュー #2533）が
//! 担う。本モジュールは「1 フレーム分の `dt` を受け取り、パーティクル群の
//! 位置・速度・回転・寿命を進める」計算のみを提供する（DOM に一切触れない、
//! `docs/design/animation-core-architecture.md` §2.1 の pull 型設計を踏襲）。
//!
//! # 可否判定（イシュー #2533 実装計画 §0 の要約）
//!
//! `docs/policy/intentional-non-adoption.md` §3.22 の headless-ui 入力系
//! 部品（ImageCropper/SignaturePad/AngleSlider）canvas 非採用は「ポインタ
//! 座標ストリームの非決定性・canvas ピクセル出力の機械検証困難性」が
//! 理由であり、本モジュールはいずれにも該当しない: (1) confetti は
//! `headless-ui` 部品ではなく `fandhe-animation`/`fandhe-frontend-animation`
//! の装飾エフェクトであり対象範囲外、(2) アプリが消費する永続状態・出力
//! データを持たない、(3) 本モジュールは固定シード・固定 `dt` 列に対して
//! 常に同一のパーティクル軌跡を返す（`tests` 参照）ため決定的検証が最初
//! から成立する。詳細は `docs/design/motion-reference-adoption-policy.md`
//! §4「confetti」行を参照。
//!
//! # 決定性・乱数の性質
//!
//! [`ConfettiSim::spawn`] はシード付き自作 PRNG（splitmix64、[`SplitMix64`]）
//! のみで初期速度・角度・色を決定する。**暗号学的乱数ではない**: 見た目の
//! ばらつき用途にのみ使い、認証・トークン生成等セキュリティ上の予測不能性
//! を要する用途へ転用しない（本モジュールの利用範囲でそのような用途は
//! 想定していない）。`rand` crate は追加しない（`fandhe-animation` は
//! 外部依存ゼロ、REQ-3）。同一シード・同一 `dt` 列であれば常に同一の
//! パーティクル軌跡を返す。

use crate::interpolate::{Rgba, Vec2};

/// パーティクル数の下限・上限（著者の誤設定による自己 DoS 防止、
/// security.md A04）。将来 `data-*` 属性経由のカスタマイズを追加する場合も
/// この範囲を超えないよう fail-closed に clamp すること。
const MIN_PARTICLE_COUNT: u32 = 1;
const MAX_PARTICLE_COUNT: u32 = 500;

/// 発火継続時間（秒）の下限・上限。`spring.rs::SpringConfig::from_duration_bounce`
/// の `[0.01, 10.0]` clamp と同じ方針（巨大値でフレームループが張り付く事態を
/// 防ぐ）。
const MIN_DURATION_SECS: f64 = 0.1;
const MAX_DURATION_SECS: f64 = 10.0;

/// drag（空気抵抗係数）の下限・上限。負値は速度を反転させてしまうため
/// 許容しない。
const MIN_DRAG: f64 = 0.0;
const MAX_DRAG: f64 = 10.0;

/// 1 回の [`ConfettiSim::step`] が進める `dt` の上限（秒）。バックグラウンド
/// タブ復帰時など `requestAnimationFrame` の delta が数秒に達すると、
/// clamp なしでは全パーティクルが 1 フレームで画面外へテレポートする
/// （`fandhe-frontend-animation::raf_driver::RafDriver` は `dt` を検証しない
/// ため、本モジュール側で防御する）。
const MAX_STEP_DT: f64 = 1.0 / 30.0;

/// 単一パーティクルの物理状態。
///
/// `Copy` にして `ConfettiSim` が内部 `Vec<Particle>` を毎フレーム
/// 上書き・再利用できるようにする（フレームごとの新規アロケーションを
/// 避ける）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    /// 現在位置。呼び出し側（`fire()`）が指定する [`ConfettiConfig::origin`]
    /// と同じ座標系（Web アダプタでは canvas ピクセル座標）。
    pub position: Vec2,
    /// 速度（座標系の単位 / 秒）。
    pub velocity: Vec2,
    /// 現在の回転角（ラジアン）。
    pub rotation: f64,
    /// 角速度（ラジアン / 秒）。
    pub angular_velocity: f64,
    /// パーティクルの色。
    pub color: Rgba,
    /// 寿命。`1.0`（生成直後）から `0.0`（消滅）へ減衰する。`0.0` 以下は
    /// 描画対象外として扱う（[`ConfettiSim::particles`] は寿命尽きた要素も
    /// 含めて返すため、書き込み先〔`Target`〕側で `life <= 0.0` を除外する）。
    pub life: f64,
}

/// confetti 発火の設定値。
///
/// 本イシューのスコープは固定デフォルト値のみ（`data-*` 属性経由の
/// カスタマイズは対象外、実装計画 §2.5・§6 の YAGIN 判断）。将来
/// カスタマイズを追加する場合は必ず [`ConfettiSim::spawn`] の clamp 経路を
/// 通すこと（著者の誤設定による自己 DoS を防ぐ、security.md A04）。
#[derive(Debug, Clone, PartialEq)]
pub struct ConfettiConfig {
    /// 発生源の座標。Web アダプタでは canvas 内のピクセル座標
    /// （`fire()` が `getBoundingClientRect()` から算出する）。
    pub origin: Vec2,
    /// 生成するパーティクル数（[`MIN_PARTICLE_COUNT`]..=[`MAX_PARTICLE_COUNT`]
    /// へ [`ConfettiSim::spawn`] が clamp する）。
    pub particle_count: u32,
    /// パーティクルの色候補（生成時にこの中から等確率で選ぶ）。空の場合は
    /// 既定色 5 色（[`DEFAULT_COLORS`]）へフォールバックする。
    pub colors: Vec<Rgba>,
    /// 初速の角度ばらつき（ラジアン、鉛直上方向 `-90°` を中心に左右へ
    /// 均等に広がる）。
    pub spread_radians: f64,
    /// 初速の大きさの範囲（`(最小, 最大)`、座標系の単位 / 秒）。
    pub speed_range: (f64, f64),
    /// 重力加速度（座標系の単位 / 秒²、正値で `position.y` 増加方向 =
    /// canvas 座標系の下方向へ加速する）。
    pub gravity: f64,
    /// 空気抵抗係数（[`MIN_DRAG`]..=[`MAX_DRAG`] へ clamp する）。
    /// `step` 内で `velocity *= (1.0 - drag * dt).clamp(0.0, 1.0)` として
    /// 適用する（負の乗数による速度反転を防ぐ）。
    pub drag: f64,
    /// 発火継続時間（秒、[`MIN_DURATION_SECS`]..=[`MAX_DURATION_SECS`] へ
    /// clamp する）。全パーティクルの寿命はこの時間で `1.0` → `0.0` へ
    /// 線形減衰する。
    pub duration_secs: f64,
}

/// [`ConfettiConfig::colors`] が空の場合のフォールバック 5 色
/// （赤・青・黄・緑・紫。Motion+ の実装は参照していない独自選定であり、
/// 一般的な confetti UI で見られる基本色相を素朴に列挙したもの）。
const DEFAULT_COLORS: [Rgba; 5] = [
    Rgba::rgb(0.92, 0.26, 0.21),
    Rgba::rgb(0.13, 0.59, 0.95),
    Rgba::rgb(0.99, 0.76, 0.03),
    Rgba::rgb(0.30, 0.69, 0.31),
    Rgba::rgb(0.61, 0.15, 0.69),
];

impl Default for ConfettiConfig {
    /// 固定デフォルト値。`origin` は原点 `(0, 0)` のため、Web アダプタ
    /// （`fire()`）が canvas サイズから算出した値で上書きすることを前提と
    /// する。
    fn default() -> Self {
        Self {
            origin: Vec2::default(),
            particle_count: 150,
            colors: DEFAULT_COLORS.to_vec(),
            spread_radians: std::f64::consts::FRAC_PI_3,
            speed_range: (200.0, 500.0),
            gravity: 700.0,
            drag: 0.9,
            duration_secs: 2.5,
        }
    }
}

/// splitmix64 相当の最小 PRNG（決定的・非暗号学的、モジュール doc 参照）。
///
/// `rand` crate を追加しない代わりに、確立されたアルゴリズム
/// （Vigna, "Further scramblings of Marsaglia's xorshift generators")
/// の定数をそのまま用いる。実装自体は本モジュール向けにゼロから書いた
/// もので、外部コードの転写ではない。
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `[0.0, 1.0)` の一様乱数。
    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / (1u64 << 53) as f64;
        (self.next_u64() >> 11) as f64 * SCALE
    }
}

/// パーティクル角速度の絶対値上限（ラジアン / 秒）。視覚的にちらつき
/// すぎない範囲として固定する。
const MAX_ANGULAR_VELOCITY: f64 = std::f64::consts::TAU * 2.0;

/// confetti パーティクル群のシミュレーション状態。
///
/// [`ConfettiSim::spawn`] で生成し、フレームごとに [`ConfettiSim::step`]
/// を呼んで進める（pull 型、`fandhe-animation` 全体の設計方針）。
#[derive(Debug, Clone)]
pub struct ConfettiSim {
    particles: Vec<Particle>,
    elapsed: f64,
    gravity: f64,
    drag: f64,
    duration_secs: f64,
}

impl ConfettiSim {
    /// `config` からパーティクル群を生成する。`seed` は呼び出し側
    /// （Web アダプタの `fire()`）が渡す（`performance.now()` 由来等）。
    ///
    /// `particle_count`/`duration_secs`/`drag` は本関数内で必ず clamp
    /// する（[`MIN_PARTICLE_COUNT`] 等の doc 参照）。`colors` が空・
    /// `speed_range` が非有限/逆転している場合も安全側の既定値へ
    /// フォールバックする（著者の誤設定で panic しない、security.md A04）。
    #[must_use]
    pub fn spawn(config: &ConfettiConfig, seed: u64) -> Self {
        let particle_count = config
            .particle_count
            .clamp(MIN_PARTICLE_COUNT, MAX_PARTICLE_COUNT);
        let duration_secs = if config.duration_secs.is_finite() {
            config
                .duration_secs
                .clamp(MIN_DURATION_SECS, MAX_DURATION_SECS)
        } else {
            ConfettiConfig::default().duration_secs
        };
        let drag = if config.drag.is_finite() {
            config.drag.clamp(MIN_DRAG, MAX_DRAG)
        } else {
            ConfettiConfig::default().drag
        };
        let gravity = if config.gravity.is_finite() {
            config.gravity
        } else {
            0.0
        };
        let spread_radians = if config.spread_radians.is_finite() {
            config.spread_radians
        } else {
            0.0
        };

        let (speed_min, speed_max) = {
            let (a, b) = config.speed_range;
            let a = if a.is_finite() { a.max(0.0) } else { 0.0 };
            let b = if b.is_finite() { b.max(0.0) } else { 0.0 };
            if a <= b {
                (a, b)
            } else {
                (b, a)
            }
        };

        let colors: Vec<Rgba> = if config.colors.is_empty() {
            DEFAULT_COLORS.to_vec()
        } else {
            config.colors.clone()
        };

        let mut rng = SplitMix64::new(seed);
        let mut particles = Vec::with_capacity(particle_count as usize);
        for _ in 0..particle_count {
            let jitter = (rng.next_f64() - 0.5) * spread_radians;
            let angle = -std::f64::consts::FRAC_PI_2 + jitter;
            let speed = speed_min + rng.next_f64() * (speed_max - speed_min);
            let velocity = Vec2 {
                x: angle.cos() * speed,
                y: angle.sin() * speed,
            };
            let color_index = (rng.next_f64() * colors.len() as f64) as usize % colors.len();
            let rotation = rng.next_f64() * std::f64::consts::TAU;
            let angular_velocity = (rng.next_f64() - 0.5) * MAX_ANGULAR_VELOCITY;
            particles.push(Particle {
                position: config.origin,
                velocity,
                rotation,
                angular_velocity,
                color: colors[color_index],
                life: 1.0,
            });
        }

        Self {
            particles,
            elapsed: 0.0,
            gravity,
            drag,
            duration_secs,
        }
    }

    /// 経過秒 `dt` だけ物理状態を進める。
    ///
    /// `dt` が有限でない・`0.0` 以下の場合は no-op（`spring.rs`/`driver.rs`
    /// と同じ防御方針）。`dt` は [`MAX_STEP_DT`] へ clamp してから適用する
    /// （バックグラウンドタブ復帰時の巨大 delta 対策、モジュール doc 参照）。
    pub fn step(&mut self, dt: f64) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(MAX_STEP_DT);
        self.elapsed += dt;

        let drag_factor = (1.0 - self.drag * dt).clamp(0.0, 1.0);
        let life_decay = dt / self.duration_secs;

        for particle in &mut self.particles {
            particle.velocity.y += self.gravity * dt;
            particle.velocity.x *= drag_factor;
            particle.velocity.y *= drag_factor;
            particle.position.x += particle.velocity.x * dt;
            particle.position.y += particle.velocity.y * dt;
            particle.rotation += particle.angular_velocity * dt;
            particle.life = (particle.life - life_decay).max(0.0);
        }
    }

    /// 現在のパーティクル群（生成順）。寿命尽きた要素（`life <= 0.0`）も
    /// 含めて返す（書き込み先が描画スキップを判断する契約、[`Particle::life`]
    /// doc 参照）。
    #[must_use]
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// 発火継続時間に達した、または全パーティクルの寿命が尽きたら `true`。
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration_secs || self.particles.iter().all(|p| p.life <= 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> ConfettiConfig {
        ConfettiConfig {
            origin: Vec2 { x: 10.0, y: 20.0 },
            particle_count: 32,
            ..ConfettiConfig::default()
        }
    }

    #[test]
    fn spawn_same_seed_produces_identical_particles() {
        let config = sample_config();
        let a = ConfettiSim::spawn(&config, 42);
        let b = ConfettiSim::spawn(&config, 42);
        assert_eq!(a.particles(), b.particles());
    }

    #[test]
    fn spawn_different_seed_produces_different_particles() {
        let config = sample_config();
        let a = ConfettiSim::spawn(&config, 1);
        let b = ConfettiSim::spawn(&config, 2);
        assert_ne!(a.particles(), b.particles());
    }

    #[test]
    fn step_same_dt_sequence_produces_identical_trajectory() {
        let config = sample_config();
        let mut a = ConfettiSim::spawn(&config, 7);
        let mut b = ConfettiSim::spawn(&config, 7);
        for _ in 0..10 {
            a.step(1.0 / 60.0);
            b.step(1.0 / 60.0);
        }
        assert_eq!(a.particles(), b.particles());
    }

    #[test]
    fn gravity_increases_downward_velocity() {
        let config = ConfettiConfig {
            particle_count: 1,
            spread_radians: 0.0,
            speed_range: (0.0, 0.0),
            gravity: 500.0,
            drag: 0.0,
            ..ConfettiConfig::default()
        };
        let mut sim = ConfettiSim::spawn(&config, 1);
        let before = sim.particles()[0].velocity.y;
        sim.step(0.1);
        let after = sim.particles()[0].velocity.y;
        assert!(after > before, "重力により y 速度は増加するはず");
    }

    #[test]
    fn drag_reduces_speed_without_gravity() {
        let config = ConfettiConfig {
            particle_count: 1,
            spread_radians: 0.0,
            speed_range: (100.0, 100.0),
            gravity: 0.0,
            drag: 5.0,
            ..ConfettiConfig::default()
        };
        let mut sim = ConfettiSim::spawn(&config, 1);
        let speed_before = sim.particles()[0].velocity.y.abs();
        for _ in 0..5 {
            sim.step(1.0 / 60.0);
        }
        let speed_after = sim.particles()[0].velocity.y.abs();
        assert!(
            speed_after < speed_before,
            "drag により速度が減衰するはず（before={speed_before}, after={speed_after}）"
        );
    }

    #[test]
    fn step_ignores_nan_and_negative_dt() {
        let config = sample_config();
        let mut sim = ConfettiSim::spawn(&config, 3);
        let before = sim.particles().to_vec();
        sim.step(f64::NAN);
        sim.step(-1.0);
        sim.step(0.0);
        assert_eq!(sim.particles(), before.as_slice());
    }

    #[test]
    fn step_clamps_huge_dt_to_max_step() {
        // バックグラウンドタブ復帰等の巨大 delta でもテレポートしない
        // （MAX_STEP_DT でクランプされる）ことを、通常 dt 1 回分との
        // 位置変化量が同一であることで確認する。
        let config = ConfettiConfig {
            particle_count: 1,
            spread_radians: 0.0,
            speed_range: (100.0, 100.0),
            gravity: 0.0,
            drag: 0.0,
            ..ConfettiConfig::default()
        };
        let mut a = ConfettiSim::spawn(&config, 9);
        let mut b = ConfettiSim::spawn(&config, 9);
        a.step(MAX_STEP_DT);
        b.step(1000.0);
        assert_eq!(a.particles(), b.particles());
    }

    #[test]
    fn particle_count_clamps_to_max() {
        let config = ConfettiConfig {
            particle_count: u32::MAX,
            ..ConfettiConfig::default()
        };
        let sim = ConfettiSim::spawn(&config, 1);
        assert_eq!(sim.particles().len(), MAX_PARTICLE_COUNT as usize);
    }

    #[test]
    fn particle_count_clamps_to_min() {
        let config = ConfettiConfig {
            particle_count: 0,
            ..ConfettiConfig::default()
        };
        let sim = ConfettiSim::spawn(&config, 1);
        assert_eq!(sim.particles().len(), MIN_PARTICLE_COUNT as usize);
    }

    #[test]
    fn duration_clamps_nan_and_out_of_range() {
        // sim.step(MAX_DURATION_SECS) は 1 回の呼び出しが内部で
        // dt.min(MAX_STEP_DT) へさらに clamp されるため、1 回だけ渡しても
        // elapsed は MAX_STEP_DT 分（約 0.033 秒）しか進まない。これでは
        // duration_secs が 999.0 のまま clamp されなくても is_finished()
        // が常に false になり、clamp 挙動を判別できない（恒真テストに
        // なる）。MAX_STEP_DT 刻みで MAX_DURATION_SECS 分の時間が経過する
        // まで繰り返し step し、実際に経過した elapsed で clamp 後の
        // duration_secs（MAX_DURATION_SECS）へ到達したことを確認する。
        let too_long = ConfettiConfig {
            duration_secs: 999.0,
            ..ConfettiConfig::default()
        };
        let mut sim = ConfettiSim::spawn(&too_long, 1);
        let steps = (MAX_DURATION_SECS / MAX_STEP_DT).ceil() as u32 + 1;
        for _ in 0..steps {
            sim.step(MAX_STEP_DT);
        }
        assert!(
            sim.is_finished(),
            "duration_secs は MAX_DURATION_SECS へ clamp され、\
             MAX_DURATION_SECS 分経過後は発火完了しているはず"
        );

        let nan = ConfettiConfig {
            duration_secs: f64::NAN,
            ..ConfettiConfig::default()
        };
        let sim = ConfettiSim::spawn(&nan, 1);
        // NaN は既定値へフォールバックする（panic しない）。
        assert!(!sim.is_finished());
    }

    #[test]
    fn empty_colors_falls_back_to_default_palette() {
        let config = ConfettiConfig {
            colors: Vec::new(),
            particle_count: 8,
            ..ConfettiConfig::default()
        };
        let sim = ConfettiSim::spawn(&config, 1);
        for particle in sim.particles() {
            assert!(DEFAULT_COLORS.contains(&particle.color));
        }
    }

    #[test]
    fn is_finished_after_full_duration() {
        let config = ConfettiConfig {
            duration_secs: 0.1,
            particle_count: 4,
            ..ConfettiConfig::default()
        };
        let mut sim = ConfettiSim::spawn(&config, 1);
        assert!(!sim.is_finished());
        for _ in 0..10 {
            sim.step(1.0 / 60.0);
        }
        assert!(sim.is_finished());
    }

    #[test]
    fn particles_start_at_origin_with_full_life() {
        let config = ConfettiConfig {
            origin: Vec2 { x: 5.0, y: 6.0 },
            particle_count: 4,
            ..ConfettiConfig::default()
        };
        let sim = ConfettiSim::spawn(&config, 1);
        for particle in sim.particles() {
            assert_eq!(particle.position, Vec2 { x: 5.0, y: 6.0 });
            assert_eq!(particle.life, 1.0);
        }
    }
}
