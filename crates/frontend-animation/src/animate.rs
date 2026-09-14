//! `element.animate()`（Web Animations API、以下 WAAPI）の薄いラッパ（イシュー #2398）。
//!
//! `fandhe-animation::keyframes::Keyframes<T>`（#2376）を WAAPI keyframes 形式へ
//! 変換し、`Element::animate()` を呼び出して `finished` Promise を待つところまでを
//! 担う。値の計算（イージング・補間・keyframes）は `fandhe-animation` の責務、
//! `data-*` 属性からの自動トリガー配線は `fandhe-frontend-wasm-full` の責務であり、
//! 本モジュールはその中間（Rust 値 → WAAPI 呼び出し）のみを担う。
//!
//! # 契約
//!
//! - `WaapiKeyframe`/`AnimateOptions` の文字列フィールド（`properties`/`easing`/
//!   `fill` 等）は、本フレームワークの型付き API（`easing_to_css`・呼び出し側の
//!   `to_css_value`）を経由した信頼できる値を渡すことを前提とする。未検証の
//!   外部入力文字列をそのまま渡さないこと（`js_sys::Reflect::set` で JS
//!   オブジェクトのプロパティ値として設定されるのみで DOM 文字列注入経路には
//!   乗らないが、想定外の値はブラウザに無視されるか `TypeError` を招く）
//! - 独立 transform（`translate`/`rotate`/`scale` 個別プロパティ）・色空間拡張
//!   （oklab 等）・複数 `Keyframes<T>` の単一呼び出しへのマージはスコープ外
//!   （YAGNI、必要な場合は `WaapiKeyframe.properties` を直接組み立てる）

use fandhe_animation::easing::{Easing, StepPosition};
use fandhe_animation::interpolate::Interpolate;
use fandhe_animation::keyframes::Keyframes;

/// WAAPI へ渡す 1 keyframe。
///
/// `easing` は「次のキーフレームへの遷移に使う easing」（WAAPI の仕様どおり、
/// 末尾キーフレームは `None`）。`properties` は `(CSS プロパティ名, CSS 値文字列)`
/// の列（呼び出し側が値を文字列化して渡す。プロパティ名の妥当性検証は行わず
/// 素通しする）。
#[derive(Debug, Clone, PartialEq)]
pub struct WaapiKeyframe {
    /// 正規化時刻（`0.0..=1.0`）。
    pub offset: f64,
    /// このキーフレームから次への遷移に使う easing（CSS easing 文字列）。
    pub easing: Option<String>,
    /// `(CSS プロパティ名, CSS 値文字列)` の列。
    pub properties: Vec<(String, String)>,
}

/// `element.animate()` の第 2 引数（薄いラッパのため duration のみ必須）。
///
/// `direction`/`composite`/`delay`/`iterationStart`/`playbackRate` は
/// スコープ外（YAGNI、必要になれば別 issue で追加する）。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimateOptions {
    /// アニメーション全体の長さ（ミリ秒）。
    pub duration_ms: f64,
    /// 全区間共通 easing のフォールバック（WAAPI 既定 `"linear"`）。
    pub easing: Option<String>,
    /// `"none"` / `"forwards"` / `"backwards"` / `"both"` / `"auto"`。
    pub fill: Option<String>,
    /// 繰り返し回数（`f64::INFINITY` 可）。
    pub iterations: Option<f64>,
}

/// [`fandhe_animation::easing::Easing`]（#2373）を CSS easing 文字列へ変換する。
///
/// `fandhe-animation::easing` は「CSS 文字列を一切生成しない」不変条件を持つため
/// （`easing.rs` モジュール冒頭コメント参照）、変換はアダプタ層である本クレートの
/// 責務として実装する。
pub fn easing_to_css(easing: Easing) -> String {
    match easing {
        Easing::Linear => "linear".to_string(),
        Easing::CubicBezier(bezier) => {
            format!(
                "cubic-bezier({}, {}, {}, {})",
                bezier.x1(),
                bezier.y1(),
                bezier.x2(),
                bezier.y2()
            )
        }
        Easing::Steps(steps) => {
            let position = match steps.position() {
                StepPosition::JumpStart => "jump-start",
                StepPosition::JumpEnd => "jump-end",
                StepPosition::JumpNone => "jump-none",
                StepPosition::JumpBoth => "jump-both",
            };
            format!("steps({}, {position})", steps.count())
        }
    }
}

/// [`Keyframes<T>`]（#2376）を単一 CSS プロパティの WAAPI keyframes 列へ変換する。
///
/// 複数プロパティを同一 keyframe に束ねたい場合は呼び出し側が
/// `WaapiKeyframe.properties` を直接組み立てる（本関数は単一プロパティの糖衣の
/// み、複数 `Keyframes<T>` のマージはスコープ外、YAGNI）。
pub fn keyframes_to_waapi<T>(
    keyframes: &Keyframes<T>,
    property: &str,
    to_css_value: impl Fn(&T) -> String,
) -> Vec<WaapiKeyframe>
where
    T: Interpolate + Clone,
{
    let frames = keyframes.frames();
    let easings = keyframes.easings();
    frames
        .iter()
        .enumerate()
        .map(|(i, frame)| WaapiKeyframe {
            offset: frame.offset,
            easing: easings.get(i).map(|e| easing_to_css(*e)),
            properties: vec![(property.to_string(), to_css_value(&frame.value))],
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::{AnimateOptions, WaapiKeyframe};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    // `web-sys` の `Element::animate`/`Animation`/`KeyframeAnimationOptions` は
    // `#[cfg(web_sys_unstable_apis)]` ゲート付きで、有効化にはワークスペース
    // 全体への `RUSTFLAGS='--cfg web_sys_unstable_apis'` 適用が要る（共有
    // `CARGO_TARGET_DIR` 運用・他クレートへのビルドフラグ波及を招くため不採用、
    // `crates/wasm-full/src/nav.rs`「判断10」と同方針）。安定版 wasm-bindgen
    // のみで完結する duck-typing extern バインディングで代替する。ソーステキスト
    // 上に `unsafe` トークンを含まない（マクロ展開後のグルーコードのみが
    // `unsafe` を含む、`docs/policy/unsafe-boundary.md` の許容境界と同区分）。
    #[wasm_bindgen]
    extern "C" {
        type AnimatableElement;

        #[wasm_bindgen(method, catch, js_name = animate)]
        fn animate_js(
            this: &AnimatableElement,
            keyframes: &JsValue,
            options: &JsValue,
        ) -> Result<WaapiAnimation, JsValue>;

        type WaapiAnimation;

        #[wasm_bindgen(method, getter, js_name = finished)]
        fn finished_promise(this: &WaapiAnimation) -> js_sys::Promise;
    }

    fn build_keyframes_js(frames: &[WaapiKeyframe]) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for frame in frames {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("offset"),
                &JsValue::from_f64(frame.offset),
            );
            if let Some(easing) = &frame.easing {
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("easing"),
                    &JsValue::from_str(easing),
                );
            }
            for (key, value) in &frame.properties {
                let _ =
                    js_sys::Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value));
            }
            arr.push(&obj);
        }
        arr
    }

    fn build_options_js(options: &AnimateOptions) -> js_sys::Object {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("duration"),
            &JsValue::from_f64(options.duration_ms),
        );
        if let Some(easing) = &options.easing {
            let _ = js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("easing"),
                &JsValue::from_str(easing),
            );
        }
        if let Some(fill) = &options.fill {
            let _ =
                js_sys::Reflect::set(&obj, &JsValue::from_str("fill"), &JsValue::from_str(fill));
        }
        if let Some(iterations) = options.iterations {
            let _ = js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("iterations"),
                &JsValue::from_f64(iterations),
            );
        }
        obj
    }

    /// `element.animate(keyframes, options)` を呼び出す。
    ///
    /// # Errors
    ///
    /// `element.animate()` の呼び出し自体が throw した場合、その `JsValue` を返す。
    pub fn animate(
        element: &web_sys::Element,
        keyframes: &[WaapiKeyframe],
        options: &AnimateOptions,
    ) -> Result<AnimationHandle, JsValue> {
        let el = element.clone().unchecked_into::<AnimatableElement>();
        let anim = el.animate_js(
            &build_keyframes_js(keyframes).into(),
            &build_options_js(options).into(),
        )?;
        Ok(AnimationHandle(anim))
    }

    /// `element.animate()` が返す `Animation` の薄いハンドル。
    ///
    /// `cancel()`/`pause()` 等の追加操作はスコープ外（YAGNI、必要になれば別 issue）。
    pub struct AnimationHandle(WaapiAnimation);

    impl AnimationHandle {
        /// `finished` Promise を待つ。
        ///
        /// # Errors
        ///
        /// Promise が reject された場合、その `JsValue` を返す。
        pub async fn finished(&self) -> Result<(), JsValue> {
            wasm_bindgen_futures::JsFuture::from(self.0.finished_promise()).await?;
            Ok(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::{animate, AnimationHandle};

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_animation::easing::{CubicBezier, Steps};
    use fandhe_animation::keyframes::Keyframe;

    #[test]
    fn easing_to_css_linear() {
        assert_eq!(easing_to_css(Easing::Linear), "linear");
    }

    #[test]
    fn easing_to_css_cubic_bezier() {
        let bezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0).unwrap();
        assert_eq!(
            easing_to_css(Easing::CubicBezier(bezier)),
            "cubic-bezier(0.4, 0, 0.2, 1)"
        );
    }

    #[test]
    fn easing_to_css_steps_all_positions() {
        let cases = [
            (StepPosition::JumpStart, "steps(4, jump-start)"),
            (StepPosition::JumpEnd, "steps(4, jump-end)"),
            (StepPosition::JumpBoth, "steps(4, jump-both)"),
        ];
        for (position, expected) in cases {
            let steps = Steps::new(4, position).unwrap();
            assert_eq!(easing_to_css(Easing::Steps(steps)), expected);
        }
        let jump_none = Steps::new(4, StepPosition::JumpNone).unwrap();
        assert_eq!(
            easing_to_css(Easing::Steps(jump_none)),
            "steps(4, jump-none)"
        );
    }

    #[test]
    fn keyframes_to_waapi_two_frames_with_segment_easing() {
        let keyframes = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: 0.0_f64,
                },
                Keyframe {
                    offset: 1.0,
                    value: 1.0_f64,
                },
            ],
            vec![Easing::CubicBezier(CubicBezier::EASE_IN)],
        )
        .unwrap();

        let waapi = keyframes_to_waapi(&keyframes, "opacity", |v| v.to_string());
        assert_eq!(waapi.len(), 2);
        assert_eq!(waapi[0].offset, 0.0);
        assert_eq!(
            waapi[0].easing.as_deref(),
            Some(easing_to_css(Easing::CubicBezier(CubicBezier::EASE_IN)).as_str())
        );
        assert_eq!(
            waapi[0].properties,
            vec![("opacity".to_string(), "0".to_string())]
        );
        assert_eq!(waapi[1].offset, 1.0);
        assert_eq!(waapi[1].easing, None);
        assert_eq!(
            waapi[1].properties,
            vec![("opacity".to_string(), "1".to_string())]
        );
    }

    #[test]
    fn keyframes_to_waapi_three_frames_two_segment_easings() {
        let keyframes = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: 0.0_f64,
                },
                Keyframe {
                    offset: 0.5,
                    value: 1.0_f64,
                },
                Keyframe {
                    offset: 1.0,
                    value: 0.0_f64,
                },
            ],
            vec![Easing::Linear, Easing::Linear],
        )
        .unwrap();

        let waapi = keyframes_to_waapi(&keyframes, "opacity", |v| v.to_string());
        assert_eq!(waapi.len(), 3);
        assert_eq!(waapi[0].easing.as_deref(), Some("linear"));
        assert_eq!(waapi[1].easing.as_deref(), Some("linear"));
        assert_eq!(waapi[2].easing, None);
    }

    #[test]
    fn keyframes_to_waapi_single_frame_has_no_easing() {
        let keyframes = Keyframes::new(
            vec![Keyframe {
                offset: 0.0,
                value: 0.0_f64,
            }],
            vec![],
        )
        .unwrap();

        let waapi = keyframes_to_waapi(&keyframes, "opacity", |v| v.to_string());
        assert_eq!(waapi.len(), 1);
        assert_eq!(waapi[0].easing, None);
    }

    #[test]
    fn keyframes_to_waapi_non_scalar_value_uses_supplied_formatter() {
        use fandhe_animation::interpolate::Vec3;

        let keyframes = Keyframes::new(
            vec![
                Keyframe {
                    offset: 0.0,
                    value: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                Keyframe {
                    offset: 1.0,
                    value: Vec3 {
                        x: 10.0,
                        y: 20.0,
                        z: 30.0,
                    },
                },
            ],
            vec![],
        )
        .unwrap();

        let waapi = keyframes_to_waapi(&keyframes, "transform", |v| {
            format!("translate3d({}px, {}px, {}px)", v.x, v.y, v.z)
        });
        assert_eq!(
            waapi[1].properties,
            vec![(
                "transform".to_string(),
                "translate3d(10px, 20px, 30px)".to_string()
            )]
        );
    }
}
