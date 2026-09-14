//! confetti（canvas ベース）発火のオーケストレーション（イシュー #2533）。
//!
//! `fandhe_animation::confetti::ConfettiSim`（決定的物理演算）を
//! [`crate::raf_driver::RafDriver`]/[`crate::raf_driver::AnimationLoop`]
//! （requestAnimationFrame 駆動）+ [`crate::canvas_target::CanvasTarget`]
//! （canvas 2D 描画）へ結線し、著者が静的に配置した `<canvas>` 要素へ
//! 1 回の発火を実行する。`data-*` 属性からの自動トリガー配線
//! （クリック → `fire()` 呼び出し）は `fandhe-frontend-wasm-full`
//! （`confetti` feature、イシュー #2533 実装計画 §2.6）の責務であり、
//! 本モジュールは呼び出されて 1 回発火するところまでを担う。
//!
//! # `AnimationLoop` の所有権契約
//!
//! [`fire`] は起動した [`crate::raf_driver::AnimationLoop`] を
//! `Ok(Some(..))` で返す（環境非対応・`prefers-reduced-motion: reduce`・
//! canvas サイズ 0 のいずれかで発火しない場合は `Ok(None)`）。**呼び出し側
//! がこのハンドルを保持し続ける必要がある**（`AnimationLoop::drop` が
//! `stop()` を呼びループを止めるため、即座に破棄すると 1 フレームも
//! 進まない）。再クリック等で新しい発火に差し替える場合は、呼び出し側の
//! 保持スロット（例: `Rc<RefCell<Option<AnimationLoop>>>`）へ新しい
//! ハンドルを代入すること——この代入は `fire` の外側（次のクリックイベント
//! ハンドラの呼び出しフレーム）で起きるため、`step` クロージャの実行中に
//! 自分自身を drop する use-after-free（`raf_driver.rs` の `AnimationLoop`
//! doc が警告する構造）を踏まない。**`step` クロージャ自身が保持スロットを
//! 捕捉して `None` を代入する設計は行わない**（それは `step` 実行中の
//! 自己 drop に該当し安全でない）。発火完了（`ConfettiSim::is_finished`）
//! 後は `step` が `false` を返して `AnimationLoop` 自身のループを止めるが、
//! ハンドル自体は呼び出し側が次に差し替える・スコープを抜けるまで生存する。
//!
//! # `prefers-reduced-motion: reduce` のフェイルセーフ方向
//!
//! `window.matchMedia` 自体の呼び出しが失敗する場合は安全側（**発火抑制**）
//! へ倒す。`crate::scroll_driver::Env::detect` の
//! `reduced_motion: unwrap_or(true)` と同じ方針（security.md A05）。

use fandhe_animation::confetti::ConfettiConfig;
use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::raf_driver::AnimationLoop;

// native（非 wasm32）ビルドでは `fire_internal` 自体がコンパイルされない
// ため（下記 `#[cfg(target_arch = "wasm32")]`）、その内部でのみ使う型は
// ここでは無条件 import せず、同じ cfg でガードする（未使用 import 警告を
// 避ける、`raf_driver.rs`/`scroll_driver.rs` と同じ方針）。
#[cfg(target_arch = "wasm32")]
use crate::canvas_target::CanvasTarget;
#[cfg(target_arch = "wasm32")]
use crate::raf_driver::RafDriver;
#[cfg(target_arch = "wasm32")]
use fandhe_animation::confetti::ConfettiSim;
#[cfg(target_arch = "wasm32")]
use fandhe_animation::driver::Driver;
#[cfg(target_arch = "wasm32")]
use fandhe_animation::interpolate::Vec2;
#[cfg(target_arch = "wasm32")]
use fandhe_animation::target::Target;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// `reduced_motion` が `true`（`prefers-reduced-motion: reduce` 検出、
/// または検出失敗時のフェイルセーフ）なら発火しない。DOM 非依存の純粋
/// 関数（native `cargo test` で真偽表を検証できる）。
#[must_use]
pub fn should_fire(reduced_motion: bool) -> bool {
    !reduced_motion
}

/// `window.matchMedia("(prefers-reduced-motion: reduce)")` を照会する。
/// 呼び出し自体が失敗する（API 不在・例外）場合は `true`（reduced 側）へ
/// fail-closed に倒す（モジュール doc 参照）。
#[cfg(target_arch = "wasm32")]
fn detect_reduced_motion() -> bool {
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

/// `canvas` へ confetti を 1 回発火する。
///
/// - `canvas` が `<canvas>` 要素でない・`getContext("2d")` が失敗する場合は
///   `Err` を返す（著者マークアップの誤りとして呼び出し側へ伝播する）。
/// - `prefers-reduced-motion: reduce`・canvas の描画領域が 0（非表示等）・
///   `requestAnimationFrame` が使えない環境（`RafDriver::new` が `None`）の
///   いずれかに該当する場合は、キャンバス生成・ループ起動を一切行わず
///   `Ok(None)` を返す（発火抑制、モジュール doc 参照）。
///
/// wasm32 以外のターゲット（native/SSR）では JS ホストが存在せず本体の
/// 呼び出し自体が panic するため、`RafDriver::new`/`AnimationLoop::start`
/// と同じ方針で JS 呼び出しを `cfg(target_arch = "wasm32")` でガードし、
/// native では常に `Ok(None)` を返す。
///
/// # Errors
///
/// `canvas` が `<canvas>` 要素でない、または 2D コンテキストの取得に
/// 失敗した場合。
pub fn fire(canvas: &Element, config: ConfettiConfig) -> Result<Option<AnimationLoop>, JsValue> {
    #[cfg(target_arch = "wasm32")]
    {
        fire_with_reduced_motion(canvas, config, detect_reduced_motion())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = canvas;
        let _ = config;
        Ok(None)
    }
}

/// [`fire`] の本体。`reduced_motion` を呼び出し側が明示できる（テスト用の
/// 注入口、`crate::scroll_driver::Env::new` と同じ意図——headless Chrome の
/// 実際の `prefers-reduced-motion` 判定結果に依存せず、抑制側の分岐を
/// 決定的に検証できる）。
///
/// `canvas` が `<canvas>` 要素かどうかの検証を `reduced_motion` の分岐より
/// 先に行う（著者マークアップの誤り＝`Err` を、実行環境の
/// `prefers-reduced-motion` 状態に左右されず常に検出できるようにするため。
/// この順序は native/CI 双方の環境非依存性を意図した設計であり、実装時の
/// 偶然の並びではない）。
///
/// # Errors
///
/// `canvas` が `<canvas>` 要素でない、または 2D コンテキストの取得に
/// 失敗した場合。
pub fn fire_with_reduced_motion(
    canvas: &Element,
    config: ConfettiConfig,
    reduced_motion: bool,
) -> Result<Option<AnimationLoop>, JsValue> {
    #[cfg(target_arch = "wasm32")]
    {
        fire_internal(canvas, config, reduced_motion)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = canvas;
        let _ = config;
        let _ = reduced_motion;
        Ok(None)
    }
}

#[cfg(target_arch = "wasm32")]
fn fire_internal(
    canvas: &Element,
    config: ConfettiConfig,
    reduced_motion: bool,
) -> Result<Option<AnimationLoop>, JsValue> {
    let Some(canvas) = canvas.dyn_ref::<HtmlCanvasElement>() else {
        return Err(JsValue::from_str(
            "fandhe-frontend-animation::confetti::fire: element is not a <canvas>",
        ));
    };

    if !should_fire(reduced_motion) {
        return Ok(None);
    }

    let rect = canvas.get_bounding_client_rect();
    let width = rect.width();
    let height = rect.height();
    let has_positive_area = width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0;
    if !has_positive_area {
        // 非表示（`display: none` 等）や未レイアウトの canvas は描画領域を
        // 持たないため、発火せず抜ける（著者マークアップのエラーではない
        // ため `Err` にはしない）。
        return Ok(None);
    }

    // 描画バッファ解像度を CSS 表示サイズへ合わせる（`devicePixelRatio` に
    // よるぼやけ対策はスコープ外、実装計画 §6 の YAGNI 判断）。
    canvas.set_width(width.round().max(1.0) as u32);
    canvas.set_height(height.round().max(1.0) as u32);

    let context = canvas
        .get_context("2d")?
        .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
        .ok_or_else(|| {
            JsValue::from_str("fandhe-frontend-animation::confetti::fire: 2d context unavailable")
        })?;

    let Some(mut driver) = RafDriver::new() else {
        return Ok(None);
    };

    // 見た目のばらつき用途のみの非暗号学的シード（モジュール doc・
    // `fandhe_animation::confetti` doc 参照）。`performance.now()` の
    // ビットパターンをそのまま使う。
    let seed = web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now().to_bits())
        .unwrap_or(0);

    let config = ConfettiConfig {
        origin: Vec2 {
            x: width / 2.0,
            y: height,
        },
        ..config
    };
    let mut sim = ConfettiSim::spawn(&config, seed);
    let mut target = CanvasTarget::new(context, width, height);

    let step = move || {
        let Some(dt) = driver.tick() else {
            // 初回 tick は基準時刻の記録のみ（`RafDriver::tick` の契約、
            // `raf_driver.rs` doc 参照）。継続する。
            return true;
        };
        sim.step(dt);
        target.write(sim.particles());
        if sim.is_finished() {
            // 最終フレームで描画を消す（空スライスを書き込むと
            // `CanvasTarget::write` が `clear_rect` のみ実行する）。
            target.write(&[]);
            return false;
        }
        true
    };

    Ok(Some(AnimationLoop::start(step)))
}

#[cfg(test)]
mod tests {
    use super::should_fire;

    #[test]
    fn should_fire_true_when_not_reduced() {
        assert!(should_fire(false));
    }

    #[test]
    fn should_fire_false_when_reduced() {
        assert!(!should_fire(true));
    }
}
