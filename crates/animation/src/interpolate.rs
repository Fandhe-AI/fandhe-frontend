//! 2 値と進捗 `t` から補間値を返す `Interpolate` trait と対象値型。
//!
//! `fandhe-frontend-animation`（Web アダプタ）が算出した補間値を DOM/Web
//! Animations API へ適用する。後続の keyframes（#2376）は区間端点間の
//! 補間に、spring（#2375）は減衰振動の現在値算出に、それぞれ本 trait を
//! `T: Interpolate` 境界として要求する。
//!
//! # 契約
//!
//! - `t` は `[0, 1]` に clamp しない（spring・overshoot 系イージングが
//!   範囲外の `t` を渡すため、`docs/design/animation-core-architecture.md`
//!   §3.1）。範囲外の `t` は外挿として自然に定義される
//! - 本クレートの他モジュールと同じく `unsafe` ゼロ・パニックしない
//!   （`docs/design/animation-core-architecture.md` §3.2 の `Vec<T>` 契約
//!   を含む）

/// `t = 0.0` で `self`、`t = 1.0` で `other` を返す 2 点間補間。
pub trait Interpolate {
    /// `self` と `other` を進捗 `t` で補間する。
    ///
    /// `t` は `[0, 1]` に clamp しない。範囲外の `t` は外挿として扱う。
    fn interpolate(&self, other: &Self, t: f64) -> Self;
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t as f32
    }
}

/// 2 次元ベクトル。成分ごとに `f64::interpolate` を適用する。
///
/// 外部 crate（glam 等）は導入しない（`core` 外部依存ゼロ方針を
/// `fandhe-animation` へも適用、`docs/design/animation-core-architecture.md`
/// §3.3）。型消去したスライス演算も採らず、フィールドを直接持つ。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Interpolate for Vec2 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
        }
    }
}

/// 3 次元ベクトル（成分ごと線形）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Interpolate for Vec3 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
            z: self.z.interpolate(&other.z, t),
        }
    }
}

/// 4 次元ベクトル（成分ごと線形）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Interpolate for Vec4 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
            z: self.z.interpolate(&other.z, t),
            w: self.w.interpolate(&other.w, t),
        }
    }
}

/// RGBA 色（成分ごと線形、正規化範囲 `0.0..=1.0` を想定・`clamp` しない）。
///
/// CSS の色補間が持つ premultiplied alpha・色空間（oklab 等）指定は本イシュー
/// の対象外（成分ごとの素な線形補間のみ）。高度化が必要になった場合は
/// `docs/policy/intentional-non-adoption.md` の評価軸に沿って別途評価する
/// （出典: 本イシューの実装計画。スコープ外事項として記録）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgba {
    /// 不透明色（`a = 1.0`）を作る。RGB 専用の別型は設けない。
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl Interpolate for Rgba {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            r: self.r.interpolate(&other.r, t),
            g: self.g.interpolate(&other.g, t),
            b: self.b.interpolate(&other.b, t),
            a: self.a.interpolate(&other.a, t),
        }
    }
}

/// 単位四元数（回転表現）。`interpolate` は slerp（球面線形補間）。
///
/// # slerp を選んだ理由
///
/// 成分ごと線形（nlerp）は角速度が一定にならず、回転角が大きいほど
/// 中間点付近で見かけの速度が落ちる。slerp は等角速度補間であり、
/// glam / three.js / Web Animations の回転補間と挙動が揃う参照実装。
/// `t` の範囲外は角度の外挿として自然に定義される。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quat {
    /// 無回転を表す恒等四元数。
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
            w: self.w * s,
        }
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }

    fn neg(&self) -> Self {
        self.scale(-1.0)
    }

    fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    /// 長さで正規化する。長さ 0（非正規な入力）は NaN を発生させず自身を返す。
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            return *self;
        }
        self.scale(1.0 / len)
    }
}

impl Interpolate for Quat {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        let mut dot = self.dot(other);
        // 四元数 q と -q は同じ回転を表す。最短経路を取るため dot < 0 なら符号反転する。
        let other = if dot < 0.0 {
            dot = -dot;
            other.neg()
        } else {
            *other
        };

        // ほぼ同一の入力では sin(theta) がゼロに近づき slerp の除算が不安定になる
        // ため、成分線形 + 正規化（nlerp）へフォールバックする。
        if dot > 1.0 - 1e-6 {
            let lerped = Self {
                x: self.x.interpolate(&other.x, t),
                y: self.y.interpolate(&other.y, t),
                z: self.z.interpolate(&other.z, t),
                w: self.w.interpolate(&other.w, t),
            };
            return lerped.normalize();
        }

        // 丸め誤差で dot がわずかに [-1, 1] を超えることがあるため acos 前に clamp する。
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let a = (theta * (1.0 - t)).sin() / sin_theta;
        let b = (theta * t).sin() / sin_theta;
        self.scale(a).add(&other.scale(b)).normalize()
    }
}

/// 4x4 行列（列優先、CSS `matrix3d()` / WebGL と同順）。
///
/// # 成分ごと線形を選んだ理由・限界
///
/// 平行移動・スケール・せん断のみの行列では成分ごと線形は正確だが、
/// 回転を含む行列では中間値が縮む（例: 180° 回転同士の中点がゼロ行列へ
/// 潰れる）。CSS Transforms 仕様の matrix 補間は decompose
/// （translate/scale/skew/perspective/quaternion）→ 各要素補間 →
/// recompose だが、本イシュー時点では未実装。回転を伴う用途は
/// [`Vec3`]（translate/scale）+ [`Quat`] を個別に補間して合成することを
/// 推奨する。分解ベース補間の導入は別途評価する（out-of-scope-tracking、
/// ユーザー承認なしでの Issue 化はしない）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub m: [f64; 16],
}

impl Interpolate for Mat4 {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            m: self.m.interpolate(&other.m, t),
        }
    }
}

impl<T: Interpolate, const N: usize> Interpolate for [T; N] {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        std::array::from_fn(|i| self[i].interpolate(&other[i], t))
    }
}

impl<T: Interpolate + Clone> Interpolate for Vec<T> {
    /// 要素ごとに補間する。長さが異なる場合は短い方の長さで打ち切る
    /// （パニックしない。長さの整合は呼び出し側の責務）。
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        self.iter()
            .zip(other)
            .map(|(a, b)| a.interpolate(b, t))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn f64_endpoints_and_extrapolation() {
        assert_eq!(0.0_f64.interpolate(&10.0, 0.0), 0.0);
        assert_eq!(0.0_f64.interpolate(&10.0, 1.0), 10.0);
        assert_eq!(0.0_f64.interpolate(&10.0, 0.5), 5.0);
        assert_eq!(0.0_f64.interpolate(&10.0, 1.5), 15.0);
        assert_eq!(0.0_f64.interpolate(&10.0, -0.5), -5.0);
    }

    #[test]
    fn f32_midpoint() {
        assert_eq!(0.0_f32.interpolate(&10.0, 0.5), 5.0_f32);
    }

    #[test]
    fn vec2_component_wise() {
        let a = Vec2 { x: 0.0, y: 0.0 };
        let b = Vec2 { x: 10.0, y: 20.0 };
        assert_eq!(a.interpolate(&b, 0.5), Vec2 { x: 5.0, y: 10.0 });
    }

    #[test]
    fn vec3_component_wise() {
        let a = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let b = Vec3 {
            x: 2.0,
            y: 4.0,
            z: 6.0,
        };
        assert_eq!(
            a.interpolate(&b, 0.5),
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }

    #[test]
    fn vec4_endpoints() {
        let a = Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        let b = Vec4 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0,
        };
        assert_eq!(a.interpolate(&b, 0.0), a);
        assert_eq!(a.interpolate(&b, 1.0), b);
    }

    #[test]
    fn rgba_component_wise_including_alpha() {
        let a = Rgba::rgb(0.0, 0.0, 0.0);
        let b = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.0,
        };
        let mid = a.interpolate(&b, 0.5);
        assert!(approx(mid.r, 0.5, 1e-9));
        assert!(approx(mid.g, 0.5, 1e-9));
        assert!(approx(mid.b, 0.5, 1e-9));
        assert!(approx(mid.a, 0.5, 1e-9));
    }

    #[test]
    fn quat_identity_to_z_90_midpoint_is_z_45() {
        let identity = Quat::IDENTITY;
        // Z 軸 90° 回転: w = cos(45°), z = sin(45°)
        let angle = std::f64::consts::FRAC_PI_2;
        let z90 = Quat {
            x: 0.0,
            y: 0.0,
            z: (angle / 2.0).sin(),
            w: (angle / 2.0).cos(),
        };
        let mid = identity.interpolate(&z90, 0.5);
        let expected_half_angle = std::f64::consts::PI / 8.0; // 45° の半分
        assert!(approx(mid.w, expected_half_angle.cos(), 1e-9));
        assert!(approx(mid.z, expected_half_angle.sin(), 1e-9));
        assert!(approx(mid.x, 0.0, 1e-9));
        assert!(approx(mid.y, 0.0, 1e-9));
    }

    #[test]
    fn quat_result_is_unit_length() {
        let a = Quat::IDENTITY;
        let angle = std::f64::consts::FRAC_PI_3;
        let b = Quat {
            x: (angle / 2.0).sin(),
            y: 0.0,
            z: 0.0,
            w: (angle / 2.0).cos(),
        };
        for t in [0.0, 0.25, 0.5, 0.75, 1.0, -0.3, 1.3] {
            let r = a.interpolate(&b, t);
            assert!(approx(r.length(), 1.0, 1e-9), "t={t} length={}", r.length());
        }
    }

    #[test]
    fn quat_takes_shortest_path_regardless_of_sign() {
        let a = Quat::IDENTITY;
        let angle = std::f64::consts::FRAC_PI_2;
        let b = Quat {
            x: 0.0,
            y: 0.0,
            z: (angle / 2.0).sin(),
            w: (angle / 2.0).cos(),
        };
        let b_negated = b.neg();
        let r1 = a.interpolate(&b, 0.5);
        let r2 = a.interpolate(&b_negated, 0.5);
        assert!(approx(r1.x, r2.x, 1e-9));
        assert!(approx(r1.y, r2.y, 1e-9));
        assert!(approx(r1.z, r2.z, 1e-9));
        assert!(approx(r1.w, r2.w, 1e-9));
    }

    #[test]
    fn quat_nearly_identical_inputs_do_not_produce_nan() {
        let a = Quat::IDENTITY;
        let b = Quat {
            x: 1e-10,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
        .normalize();
        let r = a.interpolate(&b, 0.5);
        assert!(r.x.is_finite());
        assert!(r.y.is_finite());
        assert!(r.z.is_finite());
        assert!(r.w.is_finite());
    }

    #[test]
    fn quat_identity_to_identity_is_identity() {
        let r = Quat::IDENTITY.interpolate(&Quat::IDENTITY, 0.5);
        assert!(approx(r.w, 1.0, 1e-9));
        assert!(approx(r.x, 0.0, 1e-9));
        assert!(approx(r.y, 0.0, 1e-9));
        assert!(approx(r.z, 0.0, 1e-9));
    }

    #[test]
    fn mat4_translation_midpoint() {
        let mut a = [0.0; 16];
        let mut b = [0.0; 16];
        // 単位行列に平行移動成分（列優先: 第 4 列が translate）を持たせる
        for m in [&mut a, &mut b] {
            m[0] = 1.0;
            m[5] = 1.0;
            m[10] = 1.0;
            m[15] = 1.0;
        }
        a[12] = 0.0; // translate.x
        b[12] = 10.0;
        a[13] = 0.0; // translate.y
        b[13] = 20.0;

        let mat_a = Mat4 { m: a };
        let mat_b = Mat4 { m: b };
        let mid = mat_a.interpolate(&mat_b, 0.5);
        assert!(approx(mid.m[12], 5.0, 1e-9));
        assert!(approx(mid.m[13], 10.0, 1e-9));
        assert!(approx(mid.m[0], 1.0, 1e-9));
    }

    #[test]
    fn array_element_wise() {
        let a = [0.0_f64, 1.0, 2.0];
        let b = [10.0_f64, 11.0, 12.0];
        assert_eq!(a.interpolate(&b, 0.5), [5.0, 6.0, 7.0]);
    }

    #[test]
    fn vec_same_length_element_wise() {
        let a = vec![0.0_f64, 1.0];
        let b = vec![10.0_f64, 21.0];
        assert_eq!(a.interpolate(&b, 0.5), vec![5.0, 11.0]);
    }

    #[test]
    fn vec_mismatched_length_truncates_without_panic() {
        let a = vec![0.0_f64, 1.0, 2.0];
        let b = vec![10.0_f64, 11.0];
        let result = a.interpolate(&b, 0.5);
        assert_eq!(result.len(), 2);
        assert_eq!(result, vec![5.0, 6.0]);
    }

    #[test]
    fn evaluate_is_deterministic() {
        let a = Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let b = Vec3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        };
        assert_eq!(a.interpolate(&b, 0.37), a.interpolate(&b, 0.37));
    }
}
