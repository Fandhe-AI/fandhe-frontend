//! confetti パーティクル群を canvas 2D へ描画する
//! [`fandhe_animation::target::Target`] 実装（イシュー #2533）。
//!
//! `fandhe_animation::confetti::ConfettiSim::particles()` はフレームごとに
//! `&[Particle]`（複数値）を返すため、既存の [`crate::dom_target::DomTarget`]
//! （`Target<f64>`、単一値）とは異なる形状の `Target<&[Particle]>` を新設する。
//!
//! # セキュリティ（A03: CSS/JS injection）
//!
//! [`CanvasTarget::write`] が `CanvasRenderingContext2d` へ渡す値は
//! `Particle` の数値フィールド（`position`/`rotation`）と、本クレート内で
//! 組み立てる `rgba(r, g, b, a)` 形式の色文字列（各成分は `0..=255`/
//! `0.0..=1.0` へ clamp 済みの数値のみを埋め込み、DOM/`data-*` 由来の
//! 文字列を混ぜない）のみである。`set_fill_style_str` は CSS プロパティ値
//! 単体として解釈され、`dom_target.rs` の `setProperty` と同じく複数宣言を
//! 注入する経路を持たない。

use fandhe_animation::confetti::Particle;
use fandhe_animation::interpolate::Rgba;
use fandhe_animation::target::Target;
use web_sys::CanvasRenderingContext2d;

/// パーティクルを矩形として描画する際の一辺の長さ（ピクセル）。
const PARTICLE_SIZE: f64 = 8.0;

/// `CanvasRenderingContext2d` へパーティクル群を描画する `Target` 実装。
///
/// `write` は毎フレーム呼ばれる（[`fandhe_animation::target::Target`] の
/// 契約）ため、まず `clear_rect` で前フレームの描画を消してから現在の
/// パーティクル群を描き直す（差分更新ではなく全消去・全再描画。
/// パーティクル数が [`fandhe_animation::confetti`] の上限 500 に clamp
/// されているため許容できるコストと判断する）。
pub struct CanvasTarget {
    context: CanvasRenderingContext2d,
    width: f64,
    height: f64,
}

impl CanvasTarget {
    /// `context`（`canvas.get_context("2d")` の戻り値）と、`clear_rect` に
    /// 使う canvas の描画バッファサイズ（`width`/`height` 属性値）を渡して
    /// 生成する。
    pub fn new(context: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        Self {
            context,
            width,
            height,
        }
    }

    /// `Rgba`（各成分 `0.0..=1.0` を想定、範囲外は clamp）を
    /// `rgba(r, g, b, a)` 形式の CSS 色文字列へ変換する。
    fn color_to_css(color: Rgba) -> String {
        let to_u8 = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let alpha = color.a.clamp(0.0, 1.0);
        format!(
            "rgba({}, {}, {}, {alpha})",
            to_u8(color.r),
            to_u8(color.g),
            to_u8(color.b),
        )
    }
}

impl Target<&[Particle]> for CanvasTarget {
    fn write(&mut self, particles: &[Particle]) {
        self.context.clear_rect(0.0, 0.0, self.width, self.height);
        for particle in particles {
            if particle.life <= 0.0 {
                continue;
            }
            // `save`/`translate`/`rotate`/`restore` の戻り値（Result）は
            // 描画1件の失敗で他パーティクルの描画を止めないよう無視する
            // （`Target::write` は panic/Result 伝播しない契約、
            // `fandhe_animation::target::Target` doc 参照）。
            self.context.save();
            let _ = self
                .context
                .translate(particle.position.x, particle.position.y);
            let _ = self.context.rotate(particle.rotation);
            self.context
                .set_fill_style_str(&Self::color_to_css(particle.color));
            self.context.set_global_alpha(particle.life.clamp(0.0, 1.0));
            self.context.fill_rect(
                -PARTICLE_SIZE / 2.0,
                -PARTICLE_SIZE / 2.0,
                PARTICLE_SIZE,
                PARTICLE_SIZE,
            );
            self.context.restore();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CanvasTarget;
    use fandhe_animation::interpolate::Rgba;

    #[test]
    fn color_to_css_formats_full_opacity() {
        let css = CanvasTarget::color_to_css(Rgba::rgb(1.0, 0.0, 0.5));
        assert_eq!(css, "rgba(255, 0, 128, 1)");
    }

    #[test]
    fn color_to_css_clamps_out_of_range_components() {
        let css = CanvasTarget::color_to_css(Rgba {
            r: 1.5,
            g: -0.5,
            b: 0.0,
            a: 2.0,
        });
        assert_eq!(css, "rgba(255, 0, 0, 1)");
    }
}
