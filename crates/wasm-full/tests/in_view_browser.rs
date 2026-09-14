//! `fandhe_frontend_wasm_full::in_view`（イシュー #2396、親 #2394）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/in_view.rs`（native）は純粋層（`in_view_once_from_attr`
//! 等）のみを検証済みである。本ファイルは実 `IntersectionObserver` 配線が
//! 実 DOM（headless Chromium）上で以下を満たすことを検証する。
//!
//! 1. 初期非交差の要素に `data-in-view` が付かないこと
//! 2. スクロールで可視域へ入ると `data-in-view` が付くこと
//! 3. スクロールを戻すと（`data-in-view-once` なし要素は）再び外れること
//! 4. `data-in-view-once="true"` の要素は一度可視域へ入った後スクロールを
//!    戻しても `data-in-view` が残ること（`unobserve` 済みの回帰固定）
//! 5. `Runtime::mount` 経由（`in-view` feature 既定 on）でも同じ挙動になる
//!    こと（配線群統合の回帰）
//!
//! 決定性を優先し、ブラウザ既定ビューポートに依存しない固定サイズの
//! スクロールコンテナ方式を採る（`message_scroller_browser.rs::
//! simulate_user_scroll` と同型）。`IntersectionObserver` の既定 root
//! （viewport）判定でも、`overflow-y:auto` のクリッピング祖先を通じて
//! 対象要素の交差状態がコンテナの `scroll_top` で決定的に切り替わる
//! （lazy-load 画像がスクロールコンテナ内で機能するのと同じ挙動）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::in_view::wire_in_view;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のスクロールコンテナを document body へ 1 個生成する。
/// `height:100px;overflow-y:auto` のコンテナ内に spacer(300px) → target →
/// spacer(300px) を配置し、`scroll_top == 0` では target が非交差、
/// `scroll_top == 300` で交差するレイアウトを組み立てる。
///
/// `id` を一意にすることで、同一テストバイナリ内の複数テストケースが
/// 要素を奪い合わないようにする（`headless_avatar_browser.rs::
/// create_container` と同じ意図）。
fn create_scroll_fixture(document: &Document, id_prefix: &str, once: bool) -> (Element, Element) {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(&format!("{id_prefix}-container"));
    container
        .set_attribute("style", "height:100px;overflow-y:auto")
        .expect("set_attribute must not fail");

    let top_spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    top_spacer
        .set_attribute("style", "height:300px")
        .expect("set_attribute must not fail");
    container
        .append_child(&top_spacer)
        .expect("append_child must not fail");

    let target = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    target.set_id(&format!("{id_prefix}-target"));
    target
        .set_attribute("data-in-view", "")
        .expect("set_attribute must not fail");
    if once {
        target
            .set_attribute("data-in-view-once", "true")
            .expect("set_attribute must not fail");
    }
    target
        .set_attribute("style", "height:20px")
        .expect("set_attribute must not fail");
    container
        .append_child(&target)
        .expect("append_child must not fail");

    let bottom_spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    bottom_spacer
        .set_attribute("style", "height:300px")
        .expect("set_attribute must not fail");
    container
        .append_child(&bottom_spacer)
        .expect("append_child must not fail");

    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");

    (container, target)
}

/// `condition` が成立するまで最大 2 秒（10ms x 200 回）ポーリングする
/// （`headless_avatar_browser.rs::wait_for` と同型）。
async fn wait_for(mut condition: impl FnMut() -> bool) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    for _ in 0..200 {
        if condition() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&wasm_bindgen::JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10,
                )
                .expect("setTimeout must not fail");
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("timeout promise must resolve");
    }
}

#[wasm_bindgen_test]
async fn scroll_into_and_out_of_view_toggles_attribute() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let (container, target) = create_scroll_fixture(&document, "in-view-toggle", false);

    wire_in_view(&container).expect("wire_in_view must not fail");

    // ケース 1: 初期非交差では `data-in-view` が付かない。
    wait_for(|| !target.has_attribute("data-in-view")).await;

    // ケース 2: スクロールで可視域へ入ると付く。
    container.set_scroll_top(300);
    wait_for(|| target.has_attribute("data-in-view")).await;

    // ケース 3: スクロールを戻すと（once なし要素は）再び外れる。
    container.set_scroll_top(0);
    wait_for(|| !target.has_attribute("data-in-view")).await;
}

#[wasm_bindgen_test]
async fn once_attribute_keeps_state_after_scrolling_back() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let (container, target) = create_scroll_fixture(&document, "in-view-once", true);

    wire_in_view(&container).expect("wire_in_view must not fail");

    container.set_scroll_top(300);
    wait_for(|| target.has_attribute("data-in-view")).await;

    // ケース 4: once 指定の要素はスクロールを戻しても `data-in-view` が
    // 残る（unobserve 済みで再評価されない回帰固定）。
    // 「外れない」ことは有限時間内には確認できないため、`IntersectionObserver`
    // の再発火が起きるであろう猶予（数フレーム分）だけ待ってから
    // 付いたままであることを確認する。
    container.set_scroll_top(0);
    let mut ticks = 0;
    wait_for(|| {
        ticks += 1;
        ticks >= 20
    })
    .await;
    assert!(
        target.has_attribute("data-in-view"),
        "once 指定の要素は unobserve 済みのため data-in-view が残り続けること"
    );
}

/// `Runtime::mount` 経由（`in-view` feature 既定 on）でも
/// `Self::wire_in_view` が呼ばれ、同じ挙動になることを固定する
/// （配線群統合の回帰）。
mod runtime_integration {
    use super::wait_for;
    use fandhe_frontend_core::{el, Node};
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen_test::*;

    /// `Runtime::mount` の型境界（`Component` + `DirtyTracked` +
    /// `BindingSource` + `'static`）を満たすだけの、状態を持たない
    /// 最小ホスト。`in_view` は `dispatch` チャネルを使わない属性専用配線
    /// のため、本ホストは常に no-op を返す。
    #[derive(Default)]
    struct EmptyHost;

    impl Component for EmptyHost {
        type Action = ();
        fn update(&mut self, _action: ()) {}
        fn view(&self) -> Node {
            el(
                "div",
                vec![("style", "height:100px;overflow-y:auto")],
                vec![
                    el("div", vec![("style", "height:300px")], vec![]),
                    el(
                        "div",
                        vec![
                            ("id", "mount-in-view-target"),
                            ("data-in-view", ""),
                            ("style", "height:20px"),
                        ],
                        vec![],
                    ),
                    el("div", vec![("style", "height:300px")], vec![]),
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
    async fn mount_wires_in_view_and_toggles_attribute_on_scroll() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");
        let root_id = "mount-in-view-root";
        let root = document
            .create_element("div")
            .expect("create_element must not fail for a plain div");
        root.set_id(root_id);
        document
            .body()
            .expect("document body must exist in browser test environment")
            .append_child(&root)
            .expect("append_child must not fail for a detached div");

        let runtime = Runtime::mount(root_id, EmptyHost).expect("mount must succeed");
        let target = document
            .get_element_by_id("mount-in-view-target")
            .expect("target element must exist after mount");

        wait_for(|| !target.has_attribute("data-in-view")).await;

        runtime.root().set_scroll_top(300);
        wait_for(|| target.has_attribute("data-in-view")).await;
    }
}
