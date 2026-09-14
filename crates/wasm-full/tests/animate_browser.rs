//! `fandhe_frontend_animation::animate`（`element.animate()` WAAPI 薄いラッパ、
//! イシュー #2398）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `fandhe-frontend-animation` は wasm クレートでも native `cargo test` へ
//! 持ち込まない 2 層構成（`crates/wasm-full/src/stagger_index.rs`・
//! `content_height.rs` と同型）を採るため、pure 層（`easing_to_css`・
//! `keyframes_to_waapi`）は `crates/frontend-animation/src/animate.rs` 内の
//! native `#[cfg(test)]` で検証済みであり、本ファイルは wasm32 層
//! （`animate`/`AnimationHandle::finished`）を `fandhe-frontend-wasm-full`
//! （既に cdylib+rlib・`wasm-bindgen-futures`〔dev-dependency〕・`browser-test`
//! ジョブ枠を持つ）から呼ぶ実ブラウザ経路で検証する（`frontend-animation` を
//! 新規に cdylib 化するより最小の diff、実装計画参照）。

#![cfg(target_arch = "wasm32")]
// `fandhe-frontend-animation` は `animate` feature（既定 on）が有効な場合のみ
// 依存として解決される optional 依存（`Cargo.toml` `animate = ["dep:fandhe-
// frontend-animation"]`）のため、feature matrix の `--no-default-features`
// 構成（イシュー #2328 baseline ジョブ）でもコンパイルできるよう本ファイル
// 全体を feature ゲートする（`position_browser.rs` の個別テスト単位ゲートと
// 異なり、本ファイルは `animate` 専用のため全体ゲートで足りる）。
#![cfg(feature = "animate")]

// `fandhe-animation`（`Keyframes`/`Keyframe`）は本クレートの直接依存には
// 置かず、`fandhe_frontend_animation::fandhe_animation`（イシュー #2398、
// `crates/frontend-animation/src/lib.rs` の再エクスポート）経由で参照する。
// `structure.toml` の `[directories.animation].allowed_dependents` は
// `["frontend-animation"]` のみを宣言しており、wasm-full からの直接依存
// エッジは新設しない（`Runtime` 経由で `fandhe_frontend_animation` crate
// 自体を再エクスポートする既存パターン、lib.rs 参照）。
use fandhe_frontend_animation::animate::{
    animate, keyframes_to_waapi, AnimateOptions, WaapiKeyframe,
};
use fandhe_frontend_animation::fandhe_animation::keyframes::{Keyframe, Keyframes};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Document;

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用の detached でない（`getComputedStyle` が有効値を返すよう
/// document に接続した）div を 1 個生成する
/// （`tooltip_delay_browser.rs::create_placeholder` と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> web_sys::Element {
    let element = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    element.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&element)
        .expect("append_child must not fail for a detached div");
    element
}

/// テスト末尾で要素を document から確実に除去する RAII ガード
/// （`tooltip_delay_browser.rs::RemoveOnDrop` と同じ意図）。
struct RemoveOnDrop(web_sys::Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

#[wasm_bindgen_test]
async fn animate_finished_resolves_and_applies_forwards_fill() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window
        .document()
        .expect("document must exist in browser test environment");
    let element = create_placeholder(&document, "animate-browser-target");
    let _guard = RemoveOnDrop(element.clone());

    // 要素の既定 opacity は 1（未適用時と区別が付かない値）のため、1 → 0 へ
    // アニメートして「fill: "forwards" が効いていなければ 1 のまま」との
    // 差が assert で検出できるようにする（Bugbot 指摘: 旧版は 0 → 1 へ
    // アニメートしており、fill 未適用でも既定値 1 と一致し常に pass していた）。
    let frames = vec![
        WaapiKeyframe {
            offset: 0.0,
            easing: Some("linear".to_string()),
            properties: vec![("opacity".to_string(), "1".to_string())],
        },
        WaapiKeyframe {
            offset: 1.0,
            easing: None,
            properties: vec![("opacity".to_string(), "0".to_string())],
        },
    ];
    let options = AnimateOptions {
        // CI 実行時間を節約するため短く保つ（他ブラウザテストと同じ方針）。
        duration_ms: 20.0,
        easing: None,
        fill: Some("forwards".to_string()),
        iterations: None,
    };

    let handle = animate(&element, &frames, &options)
        .expect("element.animate() must not throw for a valid keyframes/options pair");
    handle
        .finished()
        .await
        .expect("finished Promise must resolve for a normal (non-cancelled) animation");

    let computed = window
        .get_computed_style(&element)
        .expect("getComputedStyle must not throw")
        .expect("getComputedStyle must return a value for an attached element");
    let opacity = computed
        .get_property_value("opacity")
        .expect("get_property_value must not throw for a known CSS property");
    assert_eq!(
        opacity, "0",
        "fill: \"forwards\" の効果で終了状態（opacity: 0）が維持されているはず（要素の既定 opacity は 1 のため、0 ならフィル適用の証拠になる）"
    );
}

/// `duration_ms` ミリ秒待つ小さなヘルパ（`headless_avatar_browser.rs::wait_for`
/// と同じ `setTimeout` ベースの待機パターン）。
async fn sleep_ms(duration_ms: i32) {
    use wasm_bindgen::closure::Closure;

    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&wasm_bindgen::JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                duration_ms,
            )
            .expect("setTimeout must not fail");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("setTimeout Promise must resolve");
}

/// codex-review 指摘（PR #2475）の回帰テスト: `keyframes_to_waapi` は
/// `Keyframes::at` の端点保持契約（区間外は端点値を保持する）を維持する
/// ため、先頭・末尾 offset が 0.0/1.0 でない入力を offset 0.0/1.0 の
/// keyframe で補完する。本テストはその補完済み出力を実際の
/// `element.animate()` へ渡し、要素の元のスタイル（underlying value）へ
/// 値がにじみ出さず、アニメーション開始直後から一貫して指定値が保持される
/// ことをブラウザで確認する（補完がなければ WAAPI は欠けた端点を
/// underlying value で補うため、開始直後の値は要素に設定したインライン
/// `opacity: 0.9`（この後で設定する）に近い値になってしまう。W3C Web
/// Animations §5.3.4）。
#[wasm_bindgen_test]
async fn keyframes_to_waapi_padded_endpoints_hold_constant_value_from_the_start() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window
        .document()
        .expect("document must exist in browser test environment");
    let element = create_placeholder(&document, "animate-browser-padding-target");
    let _guard = RemoveOnDrop(element.clone());

    // 要素の「元のスタイル」（underlying value）を補完先の値（0.0）とは
    // はっきり異なる値にしておく。補完が効いていなければ、開始直後の
    // 実効値はこの値に近くなるはず。
    let html_element: web_sys::HtmlElement = element
        .clone()
        .dyn_into()
        .expect("placeholder div must be an HtmlElement");
    html_element
        .style()
        .set_property("opacity", "0.9")
        .expect("set_property must not fail for a known CSS property");

    // 単一 keyframe（offset 0.5、値 0.0）は先頭・末尾どちらも欠けているため、
    // `keyframes_to_waapi` により offset 0.0/0.5/1.0 の 3 keyframe（すべて
    // 値 0.0）へ補完される（`keyframes_to_waapi_single_frame_at_non_endpoint_offset_pads_both_ends`
    // の native テストと同じ入力形状）。
    let keyframes = Keyframes::<f64>::new(
        vec![Keyframe {
            offset: 0.5,
            value: 0.0,
        }],
        vec![],
    )
    .expect("single in-range offset must be a valid Keyframes construction");
    let frames: Vec<WaapiKeyframe> = keyframes_to_waapi(&keyframes, "opacity", |v| v.to_string());
    assert_eq!(
        frames.len(),
        3,
        "leading と trailing の両方が補完され 3 keyframe になるはず"
    );

    let options = AnimateOptions {
        // CI 実行時間を抑えつつ、開始直後のサンプリングに十分な余裕を持たせる。
        duration_ms: 300.0,
        easing: Some("linear".to_string()),
        fill: Some("both".to_string()),
        iterations: None,
    };

    let handle = animate(&element, &frames, &options)
        .expect("element.animate() must not throw for a padded keyframes/options pair");

    // duration の 1 割未満の時点でサンプリングする（補完なしなら underlying
    // value からの変化途中でまだ 0 に到達していないはずの早いタイミング）。
    sleep_ms(20).await;
    let computed = window
        .get_computed_style(&element)
        .expect("getComputedStyle must not throw")
        .expect("getComputedStyle must return a value for an attached element");
    let opacity_early = computed
        .get_property_value("opacity")
        .expect("get_property_value must not throw for a known CSS property");
    assert_eq!(
        opacity_early, "0",
        "先頭 offset 0.0 が補完されているため、開始直後から一貫して opacity: 0 が保持されるはず（underlying value の 0.9 へにじみ出ていないことの確認）"
    );

    handle
        .finished()
        .await
        .expect("finished Promise must resolve for a normal (non-cancelled) animation");

    let computed = window
        .get_computed_style(&element)
        .expect("getComputedStyle must not throw")
        .expect("getComputedStyle must return a value for an attached element");
    let opacity_final = computed
        .get_property_value("opacity")
        .expect("get_property_value must not throw for a known CSS property");
    assert_eq!(
        opacity_final, "0",
        "末尾 offset 1.0 も補完されているため、終了後も opacity: 0 が維持されるはず"
    );
}
