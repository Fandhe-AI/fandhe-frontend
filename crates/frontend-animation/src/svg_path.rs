//! SVG path drawing アニメーション（イシュー #2519、親 #2508）。
//!
//! `SVGGeometryElement.getTotalLength()` から得た全長で
//! `stroke-dasharray`/`stroke-dashoffset` の初期値を計算・書き込み、
//! [`crate::animate::animate`]（WAAPI 薄いラッパ、#2398）で
//! `stroke-dashoffset` を全長 → 0 へアニメーションする。値の計算・DOM
//! 書き込みまでが本モジュールの責務であり、`data-*` 属性の走査・opt-in
//! 判定は `fandhe-frontend-wasm-full`（`svg-path` feature）の責務である
//! （3 層構成、`docs/design/motion-reference-adoption-policy.md` §4・
//! `docs/design/animation-core-architecture.md` §6.2）。
//!
//! # `fill: "forwards"` が必須の理由
//!
//! WAAPI の既定 `fill` は `"none"` であり、アニメーション終了後は
//! 適用前の値（＝全長、未描画状態）へ戻る。描画済み状態を保持するため
//! [`default_options`] は必ず `fill: Some("forwards")` を返す。
//!
//! # ロケータ契約（fail-closed）
//!
//! [`total_length`]/[`apply_initial_style`] は対象要素が
//! `SVGGeometryElement`/`SVGElement` にキャストできない場合 `Err` を返す
//! （`dom_target.rs` と同じ「対象外要素は静かに無視できるよう Result で
//! 伝播する」方針）。
//!
//! # reduced-motion 方針
//!
//! [`draw_path`] は `prefers-reduced-motion: reduce` を検出すると DOM を
//! 一切書き換えず `Ok(None)` を返す（SSR 出力のフル表示〔全長 0 の
//! `stroke-dashoffset` 相当〕をそのまま活かす。検出失敗時も `true`
//! （reduced 側）へ fail-closed、`confetti.rs`/`scroll_driver.rs` と同方針）。
//!
//! # `getTotalLength()` の JS 例外（codex-review P1 是正）
//!
//! 非表示要素（`display: none` 等）に対する `getTotalLength()` は仕様上
//! `InvalidStateError` を送出しうる。`web_sys::SvgGeometryElement::
//! get_total_length()` は戻り値が `f32`（`Result` を返さない）ため、その
//! まま呼ぶと JS 例外が Rust 側で捕捉されず wasm インスタンスの異常終了
//! を招き、mount/hydrate 全体・後続配線を巻き込む。[`total_length`] は
//! `js_sys::Function::call0`（`catch` 属性付きで例外を `Result` 化する）
//! 経由で呼び出し、対象要素 1 件の失敗を `Err` として局所化する。
//!
//! # `pathLength` 補正（codex-review P1 是正）
//!
//! `getTotalLength()` は `pathLength` 属性を無視した実際の幾何学的長さを
//! 返すが、`stroke-dasharray`/`stroke-dashoffset` の距離解釈は `pathLength`
//! 属性が指定されていればその値を基準にする（SVG 仕様）。`pathLength`
//! 指定パスに実長をそのまま使うと dasharray/dashoffset の基準がずれ、
//! アニメーション完了時にも線が欠けて見える。[`apply_initial_style`]/
//! [`draw`] は呼び出し側（[`draw_path_with_reduced_motion`]）が
//! `pathLength` 属性の有無を検査して補正済みの長さを渡す契約とする。
//!
//! # アニメーション開始失敗時の復元（codex-review P1 是正）
//!
//! [`draw_path_with_reduced_motion`] は初期スタイル適用後に `animate()`
//! が `Err` を返した場合、書き込み前の `stroke-dasharray`/
//! `stroke-dashoffset` インライン値へ復元してから `Err` を伝播する
//! （非表示のまま放置しない）。

use crate::animate::AnimateOptions;

/// `stroke-dasharray`/`stroke-dashoffset` の初期値（同一値）を
/// `"{total_length}px"` 形式の CSS 値文字列として返す（DOM 非依存の純粋
/// 関数）。
#[must_use]
pub fn initial_dash_values(total_length: f64) -> (String, String) {
    let value = format!("{total_length}px");
    (value.clone(), value)
}

/// 描画アニメーションの固定既定値。
///
/// `data-*` 属性経由の duration/easing カスタマイズはスコープ外（YAGNI、
/// `confetti::ConfettiConfig::default()` と同じ判断）。
#[must_use]
pub fn default_options() -> AnimateOptions {
    AnimateOptions {
        duration_ms: 800.0,
        easing: Some("ease-in-out".to_string()),
        fill: Some("forwards".to_string()),
        iterations: Some(1.0),
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::{default_options, initial_dash_values};
    use crate::animate::{animate, AnimateOptions, AnimationHandle, WaapiKeyframe};
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, SvgElement, SvgGeometryElement};

    /// `element.getTotalLength()`（`SVGGeometryElement`）を `f64` で返す。
    ///
    /// `getTotalLength()` 自体が JS 例外（非表示要素等での
    /// `InvalidStateError`）を送出しうるため、`js_sys::Function::call0`
    /// （`catch` 属性付き）経由で呼び出し例外を `Err` として捕捉する
    /// （モジュール doc「`getTotalLength()` の JS 例外」参照）。
    ///
    /// # Errors
    ///
    /// `element` が `SVGGeometryElement` にキャストできない場合、
    /// メソッド取得に失敗した場合、`getTotalLength()` 呼び出し自体が
    /// 例外を送出した場合、戻り値が数値でなかった場合。
    pub fn total_length(element: &Element) -> Result<f64, JsValue> {
        let geometry = element
            .dyn_ref::<SvgGeometryElement>()
            .ok_or_else(|| JsValue::from_str("element is not an SVGGeometryElement"))?;
        let geometry_js: &JsValue = geometry.as_ref();
        let get_total_length =
            js_sys::Reflect::get(geometry_js, &JsValue::from_str("getTotalLength"))?
                .dyn_into::<js_sys::Function>()?;
        get_total_length
            .call0(geometry_js)?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("getTotalLength() did not return a number"))
    }

    /// `pathLength` 属性が指定されている場合、その値（`SvgAnimatedNumber`
    /// の `baseVal()`）を返す。未指定・不正値の場合は `None`（呼び出し側で
    /// `total_length` の実測値へフォールバックする、モジュール doc
    /// 「`pathLength` 補正」参照）。
    fn path_length_override(element: &Element) -> Option<f64> {
        let geometry = element.dyn_ref::<SvgGeometryElement>()?;
        if !geometry.has_attribute("pathLength") {
            return None;
        }
        let value = f64::from(geometry.path_length().base_val());
        (value.is_finite() && value > 0.0).then_some(value)
    }

    /// `pathLength` 属性が指定されていればその値、なければ `real_length`
    /// （`total_length` の実測値）を、dasharray/dashoffset の距離基準
    /// として返す。
    fn effective_length(element: &Element, real_length: f64) -> f64 {
        path_length_override(element).unwrap_or(real_length)
    }

    /// `stroke-dasharray`/`stroke-dashoffset` の初期値を書き込む。
    ///
    /// # Errors
    ///
    /// `element` が `SVGElement`（`.style()` を持つ）にキャストできない場合。
    pub fn apply_initial_style(element: &Element, total_length: f64) -> Result<(), JsValue> {
        let svg = element
            .dyn_ref::<SvgElement>()
            .ok_or_else(|| JsValue::from_str("element is not an SVGElement"))?;
        let (dasharray, dashoffset) = initial_dash_values(total_length);
        let style = svg.style();
        style.set_property("stroke-dasharray", &dasharray)?;
        style.set_property("stroke-dashoffset", &dashoffset)?;
        Ok(())
    }

    /// `stroke-dashoffset` を `total_length` → `0` へ WAAPI でアニメーション
    /// する（[`apply_initial_style`] 済みであることを前提とする）。
    ///
    /// # Errors
    ///
    /// `element.animate()` の呼び出し自体が失敗した場合。
    pub fn draw(
        element: &Element,
        total_length: f64,
        options: &AnimateOptions,
    ) -> Result<AnimationHandle, JsValue> {
        let (from, _) = initial_dash_values(total_length);
        let keyframes = [
            WaapiKeyframe {
                offset: 0.0,
                // `options.easing`（`AnimateOptions` 側の全体 easing）と
                // 二重適用しない（codex-review P2 是正）: WAAPI は
                // per-keyframe easing を options 側の easing の内側で
                // 合成適用するため、両方に同じ値を設定すると意図した
                // 曲線に対しイージングが 2 重にかかる。唯一の区間の
                // easing は options 側へ一本化し、ここは `None` とする。
                easing: None,
                properties: vec![("strokeDashoffset".to_string(), from)],
            },
            WaapiKeyframe {
                offset: 1.0,
                easing: None,
                properties: vec![("strokeDashoffset".to_string(), "0px".to_string())],
            },
        ];
        animate(element, &keyframes, options)
    }

    /// `window.matchMedia("(prefers-reduced-motion: reduce)")` を照会する。
    /// 失敗時は `true`（reduced 側）へ fail-closed（モジュール doc 参照）。
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

    /// [`draw_path`] の本体。`reduced_motion` を呼び出し側が明示できる
    /// （テスト用の注入口、`confetti::fire_with_reduced_motion` と同型）。
    ///
    /// `element` が `SVGGeometryElement`/`SVGElement` のいずれかでない場合は
    /// 何も書き換えず `Ok(None)` を返す（著者マークアップの誤りとして
    /// 静かに無視する、`confetti.rs` と同じ fail-closed 方針）。
    ///
    /// # Errors
    ///
    /// `element.animate()` の呼び出し自体が失敗した場合。
    pub fn draw_path_with_reduced_motion(
        element: &Element,
        options: &AnimateOptions,
        reduced_motion: bool,
    ) -> Result<Option<AnimationHandle>, JsValue> {
        if reduced_motion {
            return Ok(None);
        }
        let Ok(real_length) = total_length(element) else {
            return Ok(None);
        };
        let length = effective_length(element, real_length);
        let Some(svg) = element.dyn_ref::<SvgElement>() else {
            return Ok(None);
        };
        let style = svg.style();
        // animate() 開始失敗時に元の表示へ戻すため、書き込み前の値を控える
        // （codex-review P1 是正、モジュール doc「アニメーション開始失敗時の
        // 復元」参照）。未設定なら空文字（`remove_property` 相当で復元）。
        let previous_dasharray = style
            .get_property_value("stroke-dasharray")
            .unwrap_or_default();
        let previous_dashoffset = style
            .get_property_value("stroke-dashoffset")
            .unwrap_or_default();
        if apply_initial_style(element, length).is_err() {
            return Ok(None);
        }
        match draw(element, length, options) {
            Ok(handle) => Ok(Some(handle)),
            Err(err) => {
                restore_style_property(&style, "stroke-dasharray", &previous_dasharray);
                restore_style_property(&style, "stroke-dashoffset", &previous_dashoffset);
                Err(err)
            }
        }
    }

    /// `style` の `property` を `previous`（空文字なら未設定）へ戻す。
    /// 復元自体の失敗は元の呼び出しの成否に影響させない（best-effort）。
    fn restore_style_property(
        style: &web_sys::CssStyleDeclaration,
        property: &str,
        previous: &str,
    ) {
        let _ = if previous.is_empty() {
            style.remove_property(property).map(|_| ())
        } else {
            style.set_property(property, previous)
        };
    }

    /// `element` の `stroke-dashoffset` を全長 → 0 へ描画アニメーションする
    /// 公開の一枚看板 API（モジュール doc 参照）。
    ///
    /// # Errors
    ///
    /// `element.animate()` の呼び出し自体が失敗した場合。
    pub fn draw_path(element: &Element) -> Result<Option<AnimationHandle>, JsValue> {
        draw_path_with_reduced_motion(element, &default_options(), detect_reduced_motion())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::{
    apply_initial_style, draw, draw_path, draw_path_with_reduced_motion, total_length,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_dash_values_formats_px_and_matches() {
        let (dasharray, dashoffset) = initial_dash_values(123.5);
        assert_eq!(dasharray, "123.5px");
        assert_eq!(dashoffset, "123.5px");
    }

    #[test]
    fn initial_dash_values_zero_length() {
        let (dasharray, dashoffset) = initial_dash_values(0.0);
        assert_eq!(dasharray, "0px");
        assert_eq!(dashoffset, "0px");
    }

    #[test]
    fn default_options_uses_fill_forwards() {
        let options = default_options();
        assert_eq!(options.fill.as_deref(), Some("forwards"));
        assert_eq!(options.iterations, Some(1.0));
        assert!(options.duration_ms > 0.0);
    }
}
