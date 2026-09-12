//! オフセット付きキーフレーム列の補間（イシュー #2376、Motion の
//! `animate()` 配列キーフレーム `[0, 1, 0]` + `ease: [...]` 相当）。
//!
//! `fandhe-frontend-animation`（#2417）が `Driver::tick` の進捗（`[0, 1]`
//! に正規化済みの経過時間）を [`Keyframes::at`] へ渡し、任意時刻の値を
//! 得る想定。区間の端点間補間は [`Interpolate`] trait（#2374）に委譲し、
//! 区間ごとの easing は [`Easing`]（#2373）を使う。本モジュールは数値
//! 計算のみを担い、DOM・CSS 文字列の生成は行わない。
//!
//! # 契約
//!
//! - `t` は `[0, 1]` へ clamp し、NaN は `0.0` として扱う（`timeline`・
//!   `easing` モジュールと同じ規約）
//! - 区間外（`t` が先頭 offset 未満・末尾 offset 超過）は端点の値を保持
//!   する（Web Animations API の fill 挙動と同じ）
//! - 構築時（[`Keyframes::new`]/[`Keyframes::evenly`]）に不変条件を検証し
//!   `Result` で返す。`at` 側は検証済み不変条件の上に立つため `unwrap`/
//!   `expect`/パニックする索引を一切使わない

use crate::easing::Easing;
use crate::interpolate::Interpolate;

/// 1 キーフレーム。`offset` は `0.0..=1.0` に正規化された時刻。
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<T> {
    /// 正規化時刻（`0.0..=1.0`）。
    pub offset: f64,
    /// この時刻での値。
    pub value: T,
}

/// [`Keyframes::new`]/[`Keyframes::evenly`] の検証エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyframesError {
    /// キーフレームが 0 件。
    Empty,
    /// `frames[index].offset` が非有限、または `[0, 1]` の範囲外。
    InvalidOffset {
        /// 不正な offset を持つキーフレームの索引。
        index: usize,
    },
    /// `frames[index].offset` が直前のキーフレームより小さい（降順）。
    OffsetNotAscending {
        /// 降順になった側のキーフレームの索引。
        index: usize,
    },
    /// `easings` の長さが区間数（`frames.len() - 1`）と一致しない。
    EasingCountMismatch {
        /// 期待する区間数。
        expected: usize,
        /// 実際に渡された easing 数。
        actual: usize,
    },
}

impl std::fmt::Display for KeyframesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "keyframes must not be empty"),
            Self::InvalidOffset { index } => {
                write!(
                    f,
                    "keyframe offset at index {index} must be finite and within [0, 1]"
                )
            }
            Self::OffsetNotAscending { index } => {
                write!(
                    f,
                    "keyframe offset at index {index} must not be less than the previous offset"
                )
            }
            Self::EasingCountMismatch { expected, actual } => write!(
                f,
                "expected {expected} easing(s) for {expected} segment(s), got {actual}"
            ),
        }
    }
}

impl std::error::Error for KeyframesError {}

/// 検証済みキーフレーム列。区間ごとに異なる easing を持てる。
///
/// 不変条件（構築後は常に成立）: `frames` は 1 件以上・offset は非減少・
/// 各 offset は有限かつ `[0, 1]`、`easings.len() == frames.len() - 1`。
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframes<T> {
    frames: Vec<Keyframe<T>>,
    easings: Vec<Easing>,
}

impl<T> Keyframes<T> {
    /// キーフレーム列と区間ごとの easing から構築する。
    ///
    /// `easings` が空なら全区間 [`Easing::Linear`] を補う。それ以外は
    /// 長さが区間数（`frames.len() - 1`）と一致していなければならない。
    pub fn new(frames: Vec<Keyframe<T>>, easings: Vec<Easing>) -> Result<Self, KeyframesError> {
        if frames.is_empty() {
            return Err(KeyframesError::Empty);
        }
        for (index, frame) in frames.iter().enumerate() {
            if !frame.offset.is_finite() || !(0.0..=1.0).contains(&frame.offset) {
                return Err(KeyframesError::InvalidOffset { index });
            }
            if index > 0 && frame.offset < frames[index - 1].offset {
                return Err(KeyframesError::OffsetNotAscending { index });
            }
        }

        let expected_segments = frames.len() - 1;
        let easings = if easings.is_empty() {
            vec![Easing::Linear; expected_segments]
        } else if easings.len() == expected_segments {
            easings
        } else {
            return Err(KeyframesError::EasingCountMismatch {
                expected: expected_segments,
                actual: easings.len(),
            });
        };

        Ok(Self { frames, easings })
    }

    /// 値の列から等間隔 offset（`i / (n - 1)`。`n == 1` は `0.0`）で構築する
    /// 糖衣（Motion の `[a, b, c]` 配列キーフレーム相当）。
    pub fn evenly(values: Vec<T>, easings: Vec<Easing>) -> Result<Self, KeyframesError> {
        let last = values.len().saturating_sub(1).max(1) as f64;
        let frames = values
            .into_iter()
            .enumerate()
            .map(|(i, value)| Keyframe {
                offset: if last == 0.0 { 0.0 } else { i as f64 / last },
                value,
            })
            .collect();
        Self::new(frames, easings)
    }

    /// 登録済みキーフレーム列。
    pub fn frames(&self) -> &[Keyframe<T>] {
        &self.frames
    }

    /// 区間ごとの easing（`frames().len() - 1` 件）。
    pub fn easings(&self) -> &[Easing] {
        &self.easings
    }
}

impl<T: Interpolate> Keyframes<T> {
    /// 時刻 `t` における値を返す。
    ///
    /// `t` は NaN なら `0.0`、それ以外は `[0, 1]` へ clamp する。先頭
    /// offset 未満・末尾 offset 超過は端点の値を保持する。
    pub fn at(&self, t: f64) -> T
    where
        T: Clone,
    {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };

        // hi: offset <= t を満たす最後の索引の 1 つ後（先頭が t 超過なら 0）。
        let hi = self.frames.partition_point(|f| f.offset <= t);

        if hi == 0 {
            // t は先頭 offset 未満: 先頭値を保持する。
            return self.frames[0].value.clone();
        }
        if hi == self.frames.len() {
            // t は末尾 offset 以上（同一 offset の末尾ブロックを含む）: 末尾値を保持する。
            return self.frames[self.frames.len() - 1].value.clone();
        }

        let lo = hi - 1;
        let (o_lo, o_hi) = (self.frames[lo].offset, self.frames[hi].offset);
        // hi は「offset <= t」を満たさない最初の索引のため o_hi > t、
        // かつ lo は o_lo <= t を満たすため o_lo <= t < o_hi。よって
        // o_hi - o_lo > 0（同一 offset の区間は partition_point がまたいで
        // 飛ばすためここには現れない＝0 除算しない）。
        let local = (t - o_lo) / (o_hi - o_lo);
        let eased = self.easings[lo].evaluate(local);
        self.frames[lo]
            .value
            .interpolate(&self.frames[hi].value, eased)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::easing::CubicBezier;

    fn evenly_linear(values: &[f64]) -> Keyframes<f64> {
        Keyframes::evenly(values.to_vec(), Vec::new()).unwrap()
    }

    #[test]
    fn endpoints_return_first_and_last_value() {
        let kf = evenly_linear(&[0.0, 1.0, 0.0]);
        assert_eq!(kf.at(0.0), 0.0);
        assert_eq!(kf.at(1.0), 0.0);
    }

    #[test]
    fn segment_boundary_returns_exact_middle_value() {
        let kf = evenly_linear(&[0.0, 1.0, 0.0]);
        assert_eq!(kf.at(0.5), 1.0);
    }

    #[test]
    fn linear_interpolation_within_segment() {
        let kf = evenly_linear(&[0.0, 1.0, 0.0]);
        assert_eq!(kf.at(0.25), 0.5);
        assert_eq!(kf.at(0.75), 0.5);
    }

    #[test]
    fn per_segment_easing_differs_from_linear() {
        let linear_kf = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: 0.0,
                },
                Keyframe {
                    offset: 1.0,
                    value: 1.0,
                },
            ],
            vec![Easing::Linear],
        )
        .unwrap();
        let eased_kf = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: 0.0,
                },
                Keyframe {
                    offset: 1.0,
                    value: 1.0,
                },
            ],
            vec![Easing::CubicBezier(CubicBezier::EASE_IN)],
        )
        .unwrap();

        let expected = CubicBezier::EASE_IN.evaluate(0.5);
        assert_eq!(eased_kf.at(0.5), expected);
        assert_ne!(linear_kf.at(0.5), eased_kf.at(0.5));
    }

    #[test]
    fn out_of_range_t_clamps_to_endpoints() {
        let kf = evenly_linear(&[0.0, 1.0, 0.0]);
        assert_eq!(kf.at(-1.0), 0.0);
        assert_eq!(kf.at(2.0), 0.0);
    }

    #[test]
    fn nan_t_is_treated_as_zero() {
        let kf = evenly_linear(&[10.0, 20.0]);
        assert_eq!(kf.at(f64::NAN), 10.0);
    }

    #[test]
    fn explicit_offsets_hold_endpoints_outside_range() {
        let kf = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.2,
                    value: 1.0,
                },
                Keyframe {
                    offset: 0.8,
                    value: 2.0,
                },
            ],
            Vec::new(),
        )
        .unwrap();
        assert_eq!(kf.at(0.0), 1.0);
        assert_eq!(kf.at(1.0), 2.0);
    }

    #[test]
    fn zero_width_segment_is_a_step() {
        let kf = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: 0.0,
                },
                Keyframe {
                    offset: 0.5,
                    value: 1.0,
                },
                Keyframe {
                    offset: 0.5,
                    value: 2.0,
                },
                Keyframe {
                    offset: 1.0,
                    value: 3.0,
                },
            ],
            Vec::new(),
        )
        .unwrap();
        assert_eq!(kf.at(0.5), 2.0);
        // t=0.49 は第 1 区間 [0.0, 0.5) の補間中（0.0 と 1.0 の間）。
        assert!(kf.at(0.49) > 0.0 && kf.at(0.49) < 1.0);
    }

    #[test]
    fn single_frame_is_constant() {
        let kf = evenly_linear(&[42.0]);
        assert_eq!(kf.at(0.0), 42.0);
        assert_eq!(kf.at(0.5), 42.0);
        assert_eq!(kf.at(1.0), 42.0);
    }

    #[test]
    fn empty_frames_is_rejected() {
        assert_eq!(
            Keyframes::<f64>::new(Vec::new(), Vec::new()),
            Err(KeyframesError::Empty)
        );
    }

    #[test]
    fn invalid_offset_is_rejected() {
        assert_eq!(
            Keyframes::new(
                vec![Keyframe {
                    offset: f64::NAN,
                    value: 0.0
                }],
                Vec::new()
            ),
            Err(KeyframesError::InvalidOffset { index: 0 })
        );
        assert_eq!(
            Keyframes::new(
                vec![Keyframe {
                    offset: 1.5,
                    value: 0.0
                }],
                Vec::new()
            ),
            Err(KeyframesError::InvalidOffset { index: 0 })
        );
    }

    #[test]
    fn descending_offset_is_rejected() {
        assert_eq!(
            Keyframes::new(
                vec![
                    Keyframe {
                        offset: 0.5,
                        value: 0.0
                    },
                    Keyframe {
                        offset: 0.2,
                        value: 1.0
                    },
                ],
                Vec::new()
            ),
            Err(KeyframesError::OffsetNotAscending { index: 1 })
        );
    }

    #[test]
    fn easing_count_mismatch_is_rejected() {
        assert_eq!(
            Keyframes::new(
                vec![
                    Keyframe {
                        offset: 0.0,
                        value: 0.0
                    },
                    Keyframe {
                        offset: 0.5,
                        value: 1.0
                    },
                    Keyframe {
                        offset: 1.0,
                        value: 2.0
                    },
                ],
                vec![Easing::Linear],
            ),
            Err(KeyframesError::EasingCountMismatch {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn non_scalar_value_type_interpolates() {
        use crate::interpolate::Vec3;

        let kf = Keyframes::evenly(
            vec![
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Vec3 {
                    x: 10.0,
                    y: 20.0,
                    z: 30.0,
                },
            ],
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            kf.at(0.5),
            Vec3 {
                x: 5.0,
                y: 10.0,
                z: 15.0
            }
        );
    }

    #[test]
    fn at_is_deterministic() {
        let kf = evenly_linear(&[0.0, 1.0, 0.0]);
        assert_eq!(kf.at(0.37), kf.at(0.37));
    }
}
