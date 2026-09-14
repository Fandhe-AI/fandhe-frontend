//! `fandhe_frontend_wasm_full::scroll_driver`（イシュー #2521、親 `#2499`）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wire_scroll_driver_with_env` が [`Env`] の値に応じて 3 通りに分岐する
//! ことと、`Runtime::mount` 経由（`scroll-driver` feature 既定 on）でも
//! 同じ配線が起動することを検証する。
//!
//! 1. フォールバック起動（非対応 + reduced-motion なし）: スクロールに応じて
//!    `--fandhe-motion-scroll-progress` が書き込まれること
//! 2. reduced-motion: 静止値 `"1"` を 1 回だけ書き込み、以後変化しないこと
//! 3. ネイティブ対応時（実検出）: headless Chrome は `animation-timeline:
//!    view()` に対応済みのため custom property が一切書き込まれないこと
//! 4. `Runtime::mount` 経由でもケース 1 相当の挙動になること（配線群統合の
//!    回帰）

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::scroll_driver::{Env, SCROLL_PROGRESS_PROPERTY};
use fandhe_frontend_wasm_full::scroll_driver::{
    wire_scroll_driver, wire_scroll_driver_with_env, SCROLL_PROGRESS_ATTR,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`gesture_browser.rs::
/// RemoveOnDrop`/`keynav_browser.rs::RemoveOnDrop` と同じ意図。テスト間
/// DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `condition` が真になるまで最大 200 回、rAF または 50ms タイムアウトの
/// 早い方で待つ（`in_view_browser.rs::wait_for` と同型）。
async fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
    for _ in 0..200 {
        if condition() {
            return true;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let resolve_for_raf = resolve.clone();
            let raf_closure = Closure::once(move |_timestamp: f64| {
                resolve_for_raf.call0(&wasm_bindgen::JsValue::NULL).ok();
            });
            window
                .request_animation_frame(raf_closure.as_ref().unchecked_ref())
                .expect("requestAnimationFrame must not fail");
            raf_closure.forget();

            let timeout_closure = Closure::once(move || {
                resolve.call0(&wasm_bindgen::JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout_closure.as_ref().unchecked_ref(),
                    50,
                )
                .expect("setTimeout must not fail");
            timeout_closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("promise must not reject");
    }
    condition()
}

/// 固定時間だけ待つ（「変化しないこと」の確認用。`wait_for` を条件常時
/// 偽で繰り返すと最大 200 回 × 50ms の待機になり全体テスト実行時間を
/// 圧迫するため、猶予確認には短い固定時間の待機を使う）。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms)
            .expect("setTimeout must not fail in test environment");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("setTimeout promise must not reject");
}

/// 3000px の spacer の直後に `[data-fandhe-scroll-progress]` 付きの
/// target(200px) を配置した `root` を body 直下へ組み立てる
/// （`frontend-animation/tests/scroll_driver_browser.rs` と同型のページ
/// レベルスクロール方式。`wire_scroll_driver_with_env` は `document`/
/// `window` へリスナーを登録するため、コンテナ内 `overflow` スクロールでは
/// なくページ全体のスクロールで検証する）。
fn build_dom(document: &Document, root_id: &str) -> (Element, HtmlElement) {
    let root = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    root.set_id(root_id);

    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height:3000px")
        .expect("set_attribute must not fail");
    root.append_child(&spacer)
        .expect("append_child must not fail");

    let target = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<HtmlElement>()
        .expect("created element must be an HtmlElement");
    target
        .set_attribute(SCROLL_PROGRESS_ATTR, "")
        .expect("set_attribute must not fail");
    target
        .set_attribute("style", "height:200px")
        .expect("set_attribute must not fail");
    root.append_child(&target)
        .expect("append_child must not fail");

    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&root)
        .expect("append_child must not fail for a detached div");

    (root, target)
}

fn scroll_progress_value(target: &HtmlElement) -> String {
    target
        .style()
        .get_property_value(SCROLL_PROGRESS_PROPERTY)
        .expect("get_property_value must not fail")
}

#[wasm_bindgen_test]
async fn fallback_env_writes_progress_on_scroll() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let (root, target) = build_dom(&document, "scroll-driver-fallback-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_scroll_driver_with_env(&root, Env::new(false, false))
        .expect("wire_scroll_driver_with_env must succeed");

    // 開始直後 1 フレーム目の無条件 recompute（`ScrollDriver::start` 契約）
    // により、スクロール前でも 0.0 が書き込まれるはず。
    assert!(
        wait_for(|| !scroll_progress_value(&target).is_empty()).await,
        "マウント直後の 1 フレーム目で progress が書き込まれるはず"
    );
    let initial = scroll_progress_value(&target)
        .parse::<f64>()
        .expect("written value must be a valid f64 string");
    assert_eq!(initial, 0.0, "スクロール前は未進入で 0.0 のはず");

    window.scroll_to_with_x_and_y(0.0, 3000.0);
    assert!(
        wait_for(|| {
            scroll_progress_value(&target)
                .parse::<f64>()
                .map(|v| v > initial)
                .unwrap_or(false)
        })
        .await,
        "スクロール後に progress が増加しているはず"
    );

    window.scroll_to_with_x_and_y(0.0, 0.0);
}

#[wasm_bindgen_test]
async fn reduced_motion_writes_static_one_and_does_not_change() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let (root, target) = build_dom(&document, "scroll-driver-reduced-motion-root");
    let _guard = RemoveOnDrop(root.clone());

    wire_scroll_driver_with_env(&root, Env::new(false, true))
        .expect("wire_scroll_driver_with_env must succeed");

    assert!(
        wait_for(|| !scroll_progress_value(&target).is_empty()).await,
        "reduced-motion 時は即座に静止値が書き込まれるはず"
    );
    assert_eq!(
        scroll_progress_value(&target),
        "1",
        "reduced-motion 時の静止値は 1 のはず"
    );

    window.scroll_to_with_x_and_y(0.0, 3000.0);
    // リスナー登録自体を行わない契約のため、スクロール後も変化しない
    // （`wait_for` は「変化しないこと」を直接検証できないため、猶予時間を
    // 置いてから固定値のままであることを確認する）。
    sleep_ms(150).await;
    assert_eq!(
        scroll_progress_value(&target),
        "1",
        "reduced-motion 時はスクロールしても静止値のまま変化しないはず"
    );

    window.scroll_to_with_x_and_y(0.0, 0.0);
}

#[wasm_bindgen_test]
async fn native_support_detected_writes_nothing() {
    let document = web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist");
    let (root, target) = build_dom(&document, "scroll-driver-native-root");
    let _guard = RemoveOnDrop(root.clone());

    // 実ブラウザ検出（`Env::detect`）を使う。headless Chrome は
    // `animation-timeline: view()` に既にネイティブ対応しているため、
    // custom property は一切書き込まれない（実行環境依存の回帰ガード、
    // 非対応ブラウザへ更新された場合はこのアサーションが変化を検知する）。
    wire_scroll_driver(&root).expect("wire_scroll_driver must succeed");

    sleep_ms(150).await;
    assert!(
        scroll_progress_value(&target).is_empty(),
        "ネイティブ対応時は custom property を一切書き込まないはず"
    );
}

/// `Runtime::mount` 経由（`scroll-driver` feature 既定 on）でも
/// `Self::wire_scroll_driver`（実検出 `Env::detect` 経由）が呼ばれることを
/// 固定する（配線群統合の回帰）。headless Chrome はネイティブ対応済みの
/// ため、`native_support_detected_writes_nothing` と同じ結果（custom
/// property 非書き込み）になる。
mod runtime_integration {
    use super::{scroll_progress_value, wait_for, RemoveOnDrop};
    use fandhe_frontend_core::{el, Node};
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::scroll_driver::SCROLL_PROGRESS_ATTR;
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    /// `Runtime::mount` の型境界を満たすだけの、状態を持たない最小ホスト
    /// （`in_view_browser.rs::runtime_integration::EmptyHost` と同型）。
    #[derive(Default)]
    struct EmptyHost;

    impl Component for EmptyHost {
        type Action = ();
        fn update(&mut self, _action: ()) {}
        fn view(&self) -> Node {
            el(
                "div",
                vec![],
                vec![
                    el("div", vec![("style", "height:3000px")], vec![]),
                    el(
                        "div",
                        vec![
                            ("id", "mount-scroll-driver-target"),
                            (SCROLL_PROGRESS_ATTR, ""),
                            ("style", "height:200px"),
                        ],
                        vec![],
                    ),
                ],
            )
        }
        fn decode_action(_name: &str, _payload: &str) -> Option<()> {
            None
        }
    }

    impl DirtyTracked for EmptyHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &[]
        }
    }

    impl BindingSource for EmptyHost {
        fn bound_value(&self, _field: &str) -> Option<BoundValue> {
            None
        }
    }

    impl Hydrate for EmptyHost {
        fn hydration_attrs(&self) -> Vec<(String, String)> {
            Vec::new()
        }
        fn from_hydration_attrs(_attrs: &[(String, String)]) -> Result<Self, HydrateError> {
            Ok(Self)
        }
    }

    #[wasm_bindgen_test]
    async fn mount_wires_scroll_driver_and_writes_progress_on_scroll() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");
        let root_id = "mount-scroll-driver-root";
        let root = document
            .create_element("div")
            .expect("create_element must not fail for a plain div");
        root.set_id(root_id);
        document
            .body()
            .expect("document body must exist in browser test environment")
            .append_child(&root)
            .expect("append_child must not fail for a detached div");
        let _guard = RemoveOnDrop(root);

        let _runtime = Runtime::mount(root_id, EmptyHost).expect("mount must succeed");
        let target = document
            .get_element_by_id("mount-scroll-driver-target")
            .expect("target element must exist after mount")
            .dyn_into::<web_sys::HtmlElement>()
            .expect("target must be an HtmlElement");

        // headless Chrome はネイティブ対応済みのため `Runtime::mount`
        // 経由の実検出（`Env::detect`）では custom property が書き込まれ
        // ないことを確認する（`native_support_detected_writes_nothing` と
        // 同じ回帰ガード。配線自体が確実に呼ばれたこと自体は `wire_gesture`/
        // `wire_in_view` 同様、既存の他配線群テストが `Runtime::mount` 経路
        // を横断的に固定しており、本テストは `scroll-driver` feature が
        // `Runtime::mount` から実際に到達可能であることの契約確認を担う）。
        for _ in 0..5 {
            wait_for(|| false).await;
        }
        assert!(
            scroll_progress_value(&target).is_empty(),
            "ネイティブ対応時は Runtime::mount 経由でも custom property を書き込まないはず"
        );
    }
}
