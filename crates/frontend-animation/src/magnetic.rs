//! magnetic pull オフセット計算・DOM 書き込み（イシュー #2550）。
//!
//! # 責務境界
//!
//! Motion+ の magnetic ボタン（ポインタに追従して吸い付く CTA）に相当する
//! 効果のうち、「ポインタ座標 → オフセット」の純粋な計算（[`compute_pull`]）
//! と、そのオフセットを DOM の CSS カスタムプロパティへ書き込む薄いヘルパ
//! （[`write_offset`]）のみを本モジュールが担う。ポインタイベントの購読・
//! opt-in 属性判定・`prefers-reduced-motion` 検出は `fandhe-frontend-wasm-full`
//! の `magnetic` モジュール（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）の責務であり、本モジュールは
//! 持たない。
//!
//! # rAF/spring を使わない設計判断（重要な逸脱の明記）
//!
//! [`crate::raf_driver::RafDriver`]/[`crate::raf_driver::AnimationLoop`] や
//! `fandhe_animation` の spring ソルバは使わない。`pointermove` は高頻度
//! （ブラウザのイベントループに従い数十 Hz 程度）で発火するイベントであり、
//! そのたびに [`compute_pull`] で計算した値を [`write_offset`] で直接
//! 書き込むだけで、実用上十分に滑らかな追従感が得られる。「バネで戻る」
//! 感触は rAF ループでの物理積分ではなく、既存の CSS motion トークン
//! （`--fandhe-motion-duration-*`/`--fandhe-motion-easing-standard`、
//! `Theme::to_css` の既定出力）による CSS `transition` に委ねる設計とした
//! （呼び出し側の CSS 側の責務、本モジュールは値の書き込みのみ）。
//!
//! 利点: (1) 新規の 2 軸 spring 積分・`AnimationLoop` の自己参照構造
//! （use-after-free 対策込み）を持ち込まずに済み実装・テストコストが小さい、
//! (2) `--fandhe-motion-duration-*` は `prefers-reduced-motion: reduce` 下で
//! `Theme::to_css` が一括 0 化する既存機構がそのまま効くため、個別の
//! `@media` 追加が不要になる（`docs/design/pricing-tiers-morph` 系 block と
//! 同じ理由）。
//!
//! この判断が「継続的な状態更新を rAF で行わない」という他の本クレート
//! モジュール（`scroll_driver`/`hold_to_confirm` 等）との一貫性から外れる
//! ことは意図的な逸脱であり、後日の再評価が必要になった場合の根拠として
//! 本 doc に明記する（`pricing_tiers_morph.rs` が自らの設計逸脱をモジュール
//! doc に明記した先例と同型）。
//!
//! # `prefers-reduced-motion: reduce` のフェイルセーフ方向
//!
//! [`detect_reduced_motion`] は `window.matchMedia` 自体の呼び出しが失敗
//! する場合、安全側（**配線抑制**）へ倒す（`crate::confetti::
//! detect_reduced_motion`/`crate::scroll_driver::Env::detect` と同じ方針、
//! security.md A05）。

/// ポインタ座標に対する引力の強さ（0.0〜1.0 の係数）。値が大きいほど
/// ポインタの動きに対してオフセットが強く反応する。固定デフォルト値のみ
/// （`data-*` 属性経由のカスタマイズは confetti と同じ理由で YAGNI として
/// スコープ外にする）。
pub const MAGNETIC_STRENGTH: f64 = 0.3;

/// オフセットの最大距離（CSS ピクセル）。円形境界として `(dx, dy)` の
/// ベクトル長をこの値へ clamp する（片軸ごとの clamp ではない）。
pub const MAGNETIC_MAX_PULL_PX: f64 = 12.0;

/// ポインタ座標 `(pointer_x, pointer_y)` と対象要素の中心座標
/// `(center_x, center_y)` から、`(dx, dy)`（CSS ピクセル、要素へ適用する
/// `translate()` オフセット）を計算する。
///
/// - `dx = (pointer_x - center_x) * strength`、`dy` も同様。
/// - ベクトル長（`(dx, dy)` のユークリッド長）が `max_pull` を超える場合は
///   方向を保ったまま `max_pull` へ clamp する（円形境界）。
/// - 入力に非有限値（NaN/inf）が混入した場合は `(0.0, 0.0)` へ fail-safe
///   する（`fandhe_animation::confetti::ConfettiSim`/`SpringConfig` 等、
///   既存コードの「非有限入力は安全側」方針を踏襲）。
/// - `max_pull` が 0 以下、または非有限の場合も `(0.0, 0.0)` を返す。
#[must_use]
pub fn compute_pull(
    pointer_x: f64,
    pointer_y: f64,
    center_x: f64,
    center_y: f64,
    max_pull: f64,
    strength: f64,
) -> (f64, f64) {
    if !pointer_x.is_finite()
        || !pointer_y.is_finite()
        || !center_x.is_finite()
        || !center_y.is_finite()
        || !max_pull.is_finite()
        || !strength.is_finite()
        || max_pull <= 0.0
    {
        return (0.0, 0.0);
    }

    let dx = (pointer_x - center_x) * strength;
    let dy = (pointer_y - center_y) * strength;
    let length = dx.hypot(dy);
    if length <= max_pull || length == 0.0 {
        return (dx, dy);
    }
    let scale = max_pull / length;
    (dx * scale, dy * scale)
}

/// `HtmlElement` の style へ `(x, y)`（CSS ピクセル）を 2 つの CSS カスタム
/// プロパティ（`--fandhe-motion-magnetic-x`/`-y`）として書き込む。
///
/// [`crate::dom_target::DomTarget`] は `Target<f64>` 経由の 1 値書き込み
/// 用途（rAF ループでの毎フレーム書き込み）を想定した設計のため、
/// 本モジュールのような「イベントごとの 1 回書き込みで 2 値をまとめて
/// 書く」用途には合わない。`CSSStyleDeclaration.setProperty` を直接
/// 2 回呼ぶ薄いヘルパとして独立に定義する。
///
/// # セキュリティ（A03: CSS injection）
///
/// `CSSStyleDeclaration.setProperty(name, value)`（2 引数 API）のみを使う
/// （[`crate::dom_target`] モジュール doc と同じ不変条件）。`name` は本
/// 関数が持つ固定のリテラル文字列であり、呼び出し側から差し替えられない。
/// `value` はここで生成した `f64` を `format!("{value}px")` した文字列
/// のみであり、DOM 由来の信頼できない文字列を混ぜない。
#[cfg(target_arch = "wasm32")]
pub fn write_offset(element: &web_sys::HtmlElement, x: f64, y: f64) {
    let style = element.style();
    let _ = style.set_property("--fandhe-motion-magnetic-x", &format!("{x}px"));
    let _ = style.set_property("--fandhe-motion-magnetic-y", &format!("{y}px"));
}

/// `window.matchMedia("(prefers-reduced-motion: reduce)")` を照会する。
/// 呼び出し自体が失敗する（API 不在・例外）場合は `true`（抑制側）へ
/// fail-closed に倒す（モジュール doc「`prefers-reduced-motion: reduce` の
/// フェイルセーフ方向」節参照）。
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn detect_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|list| list.matches())
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::{compute_pull, MAGNETIC_MAX_PULL_PX, MAGNETIC_STRENGTH};

    #[test]
    fn compute_pull_is_zero_at_center() {
        assert_eq!(compute_pull(50.0, 50.0, 50.0, 50.0, 12.0, 0.3), (0.0, 0.0));
    }

    #[test]
    fn compute_pull_scales_by_strength_within_bound() {
        // dx = (60 - 50) * 0.5 = 5.0, dy = (55 - 50) * 0.5 = 2.5;
        // length = sqrt(5^2 + 2.5^2) ≈ 5.59 < max_pull(12) なので clamp なし。
        let (dx, dy) = compute_pull(60.0, 55.0, 50.0, 50.0, 12.0, 0.5);
        assert!((dx - 5.0).abs() < 1e-9);
        assert!((dy - 2.5).abs() < 1e-9);
    }

    #[test]
    fn compute_pull_clamps_to_max_pull_preserving_direction() {
        // dx = (150 - 50) * 0.3 = 30.0, dy = 0.0; length = 30.0 > max_pull(12)。
        let (dx, dy) = compute_pull(150.0, 50.0, 50.0, 50.0, 12.0, 0.3);
        assert!(
            (dx - 12.0).abs() < 1e-9,
            "dx should clamp to max_pull: {dx}"
        );
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn compute_pull_clamps_diagonal_vector_to_circular_boundary() {
        // dx = dy = (100 - 0) * 1.0 = 100.0 each; length = 100*sqrt(2)。
        let (dx, dy) = compute_pull(100.0, 100.0, 0.0, 0.0, 12.0, 1.0);
        let length = dx.hypot(dy);
        assert!(
            (length - 12.0).abs() < 1e-9,
            "length should clamp to max_pull: {length}"
        );
        assert!(
            (dx - dy).abs() < 1e-9,
            "direction (dx == dy) should be preserved"
        );
    }

    #[test]
    fn compute_pull_is_zero_when_input_contains_nan() {
        assert_eq!(
            compute_pull(f64::NAN, 50.0, 50.0, 50.0, 12.0, 0.3),
            (0.0, 0.0)
        );
    }

    #[test]
    fn compute_pull_is_zero_when_input_contains_infinity() {
        assert_eq!(
            compute_pull(f64::INFINITY, 50.0, 50.0, 50.0, 12.0, 0.3),
            (0.0, 0.0)
        );
    }

    #[test]
    fn compute_pull_is_zero_when_max_pull_is_zero() {
        assert_eq!(compute_pull(100.0, 50.0, 50.0, 50.0, 0.0, 0.3), (0.0, 0.0));
    }

    #[test]
    fn compute_pull_is_zero_when_max_pull_is_negative() {
        assert_eq!(compute_pull(100.0, 50.0, 50.0, 50.0, -1.0, 0.3), (0.0, 0.0));
    }

    #[test]
    fn default_constants_are_positive_and_finite() {
        assert!(MAGNETIC_STRENGTH.is_finite() && MAGNETIC_STRENGTH > 0.0);
        assert!(MAGNETIC_MAX_PULL_PX.is_finite() && MAGNETIC_MAX_PULL_PX > 0.0);
    }
}
