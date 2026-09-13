//! フレームループから tick を受け取る `Driver`（イシュー #2378）。
//!
//! `animation-core-architecture.md` §2.1 の pull 型設計を採用する:
//! 本 crate はフレームループ自体を持たず、呼び出し側（`fandhe-frontend-animation`
//! の rAF コールバック等）が 1 フレームごとに [`Driver::tick`] を 1 回呼ぶ。
//!
//! # `tick` の契約
//!
//! 戻り値は「直前の `tick` 呼び出しからの経過秒（有限・`>= 0`）」である。
//! まだ計測を開始していない・供給が尽きた等の理由で経過時間を返せない
//! 場合は `None` を返す。契約違反（NaN・負値）の値検証は本 trait では
//! 行わない — [`crate::spring::Spring::at`] / [`crate::keyframes::Keyframes::at`]
//! 側が既に NaN・負を丸めて扱うため、ここに検証を重ねると全実装
//! （信頼できる `performance.now()` 差分を返す Web 実装を含む）へ
//! 無駄なコストを強制することになる。
pub trait Driver {
    /// 直前の呼び出しからの経過秒を返す。供給が尽きたら `None`。
    fn tick(&mut self) -> Option<f64>;
}

/// テスト用の参照実装: あらかじめ積んだ delta 列を FIFO で払い出すだけの `Driver`。
///
/// 実時間・rAF に依存しない決定的なテストのために、呼び出し側が明示的に
/// `tick` の戻り値列を用意する。`test-utils` feature 経由で crate 外
/// （`fandhe-frontend-animation` 等）のテストコードからも利用できる。
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone, Default)]
pub struct ManualDriver {
    deltas: std::collections::VecDeque<f64>,
}

#[cfg(any(test, feature = "test-utils"))]
impl ManualDriver {
    /// 払い出す delta 列（秒）を積んで生成する。
    pub fn new(deltas: impl IntoIterator<Item = f64>) -> Self {
        Self {
            deltas: deltas.into_iter().collect(),
        }
    }

    /// 末尾に delta を追加で積む。
    pub fn push(&mut self, delta: f64) {
        self.deltas.push_back(delta);
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Driver for ManualDriver {
    fn tick(&mut self) -> Option<f64> {
        self.deltas.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::easing::Easing;
    use crate::keyframes::Keyframes;
    use crate::spring::{Spring, SpringConfig};
    use crate::target::{RecordingTarget, Target};
    use crate::timeline::Timeline;

    #[test]
    fn manual_driver_pops_deltas_in_fifo_order() {
        let mut driver = ManualDriver::new([1.0, 2.0, 3.0]);
        assert_eq!(driver.tick(), Some(1.0));
        assert_eq!(driver.tick(), Some(2.0));
        assert_eq!(driver.tick(), Some(3.0));
    }

    #[test]
    fn manual_driver_returns_none_once_exhausted() {
        let mut driver = ManualDriver::new([1.0]);
        assert_eq!(driver.tick(), Some(1.0));
        assert_eq!(driver.tick(), None);
        // 枯渇後も再度 None を返し続ける（panic しない）。
        assert_eq!(driver.tick(), None);
    }

    #[test]
    fn manual_driver_resumes_after_push() {
        let mut driver = ManualDriver::new([1.0]);
        assert_eq!(driver.tick(), Some(1.0));
        assert_eq!(driver.tick(), None);
        driver.push(2.0);
        assert_eq!(driver.tick(), Some(2.0));
    }

    #[test]
    fn manual_driver_new_with_empty_iter_ticks_none_immediately() {
        let mut driver = ManualDriver::new(std::iter::empty());
        assert_eq!(driver.tick(), None);
    }

    /// spring を `ManualDriver` で駆動し、`RecordingTarget` への書き込みが
    /// 積んだ tick 数と一致し、先頭が `from`・終盤が `to` へ収束することを
    /// 確認する（Driver は `t` を積算しない — 積算は呼び出し側の責務、という
    /// 不変条件を rAF 側の結線と同じ形で固定する）。
    #[test]
    fn spring_driven_by_manual_driver_writes_monotone_settle() {
        const DT: f64 = 1.0 / 60.0;
        const TICKS: usize = 300;
        let mut driver = ManualDriver::new(std::iter::repeat_n(DT, TICKS));
        let spring = Spring::new(SpringConfig::default(), 0.0, 100.0, 0.0)
            .expect("有効な spring パラメータのため None にならない");
        let mut target = RecordingTarget::new();

        let mut t = 0.0;
        while let Some(dt) = driver.tick() {
            t += dt;
            target.write(spring.at(t).value);
        }

        let values = target.into_values();
        assert_eq!(values.len(), TICKS);
        assert_eq!(values[0], spring.at(DT).value);
        // 十分な tick 数（300 * 1/60s = 5s）を与えたため最終値は to に収束する。
        assert_eq!(*values.last().unwrap(), 100.0);
    }

    /// keyframes を `ManualDriver` で駆動し、進行度 `t/duration` を [`Keyframes::at`]
    /// へ渡すサンプリングが tick 数と一致し、末尾がキーフレーム末尾値へ揃うことを
    /// 確認する。
    #[test]
    fn keyframes_driven_by_manual_driver_samples_in_order() {
        const DURATION: f64 = 1.0;
        const DT: f64 = 0.1;
        const TICKS: usize = 12; // duration を超えて進めても clamp されることも確認する。

        let kf = Keyframes::evenly(vec![0.0, 10.0, 0.0], vec![Easing::Linear, Easing::Linear])
            .expect("3 値・2 easing は有効な構成");
        let mut driver = ManualDriver::new(std::iter::repeat_n(DT, TICKS));
        let mut target = RecordingTarget::new();

        let mut t = 0.0;
        while let Some(dt) = driver.tick() {
            t += dt;
            target.write(kf.at(t / DURATION));
        }

        let values = target.into_values();
        assert_eq!(values.len(), TICKS);
        // 末尾キーフレームの値（0.0）で揃う（t が duration を超えても clamp される）。
        assert_eq!(*values.last().unwrap(), 0.0);
        // 中間（offset 0.5 付近）で最大値付近を通過する。
        let max = values.iter().cloned().fold(f64::MIN, f64::max);
        assert!((max - 10.0).abs() < 1e-9, "max={max}");
    }

    /// `Timeline<Keyframes<f64>>` を `ManualDriver` で駆動し、各セグメントの
    /// 進行度が時系列順に更新されることを確認する（driver → timeline →
    /// keyframes という 3 モジュール結合の end-to-end 経路）。
    #[test]
    fn timeline_of_keyframes_driven_end_to_end() {
        let kf_a = Keyframes::evenly(vec![0.0, 1.0], vec![Easing::Linear]).unwrap();
        let kf_b = Keyframes::evenly(vec![1.0, 2.0], vec![Easing::Linear]).unwrap();

        let mut timeline = Timeline::new();
        timeline.add(kf_a, 0.5).unwrap();
        timeline.add(kf_b, 0.5).unwrap();
        let total_duration = timeline.duration();
        assert_eq!(total_duration, 1.0);

        const DT: f64 = 0.1;
        const TICKS: usize = 10;
        let mut driver = ManualDriver::new(std::iter::repeat_n(DT, TICKS));
        let mut target: RecordingTarget<Vec<f64>> = RecordingTarget::new();

        let mut t = 0.0;
        while let Some(dt) = driver.tick() {
            t += dt;
            let sampled: Vec<f64> = timeline
                .progress_at(t)
                .map(|(progress, kf)| kf.at(progress))
                .collect();
            target.write(sampled);
        }

        let values = target.into_values();
        assert_eq!(values.len(), TICKS);
        // t = duration() 到達時点で両セグメントとも末尾値に揃う
        // （dt 積算の浮動小数点誤差を許容する）。
        let last = values.last().unwrap();
        assert!((last[0] - 1.0).abs() < 1e-9, "last[0]={}", last[0]);
        assert!((last[1] - 2.0).abs() < 1e-9, "last[1]={}", last[1]);
        // セグメント 0 が終わる（t < 0.5）前はセグメント 1 の progress が 0
        // （キーフレーム先頭値のまま）であることを最初の tick で確認する。
        assert_eq!(values[0][1], 1.0);
    }
}
