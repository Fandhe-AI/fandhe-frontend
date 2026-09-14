//! `Runtime::apply_with_view_transition`（イシュー #2400、feature
//! `view-transitions`）の実ブラウザ統合テスト（`wasm-pack test --headless
//! --chrome`）。
//!
//! `document.startViewTransition` の記録用スタブ（[`ViewTransitionStub`]）・
//! 機能非対応シャドウ（[`NonFunctionViewTransitionShadow`]）は
//! `nav_browser.rs` の同名ヘルパーと同じ意図の自己完結な複製である
//! （テスト実行単位の制約上、他テストバイナリの private 定義を import
//! できないため、本リポジトリの既存流儀どおり複製する）。
//!
//! - 検証 A: 機能検出成功時、`apply_with_view_transition` が
//!   `document.startViewTransition` をちょうど 1 回呼び、update コールバック
//!   （実ブラウザでは非同期になり得るため [`wait_until`] で待つ）実行後に
//!   `root` 配下が現在の状態から再構築されていること。
//! - 検証 B: 機能非対応（`startViewTransition` が非関数）時、呼び出し直後
//!   （同期）に DOM が更新済みであること（graceful degradation）。
//!
//! feature 無効時に `nav.rs` の router 経由 `startViewTransition` が
//! 無条件で動作し続けることの証跡は、コンパイル契約
//! （`cargo check -p fandhe-frontend-wasm-full --no-default-features
//! --features wasm-bindgen-exports --target wasm32-unknown-unknown`）で
//! 別途示す（本ファイルの対象外）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "view-transitions")]

use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use js_sys::{Function, Reflect};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// `nav_browser.rs::create_placeholder` と同じ意図（テスト間の document
/// 汚染を避けるための一意 id 付きプレースホルダ）。
fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// `nav_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn document_as_value(document: &Document) -> JsValue {
    JsValue::from(document.clone())
}

fn document_as_object(document: &Document) -> js_sys::Object {
    document.clone().unchecked_into::<js_sys::Object>()
}

/// `nav_browser.rs::ViewTransitionStub` の複製（doc 参照）。
struct ViewTransitionStub {
    call_count: Rc<Cell<u32>>,
    _closure: Closure<dyn FnMut(JsValue) -> JsValue>,
}

impl ViewTransitionStub {
    fn install(document: &Document) -> Self {
        let call_count = Rc::new(Cell::new(0u32));
        let counter = call_count.clone();
        let closure = Closure::wrap(Box::new(move |update: JsValue| -> JsValue {
            counter.set(counter.get() + 1);
            if let Some(update_fn) = update.dyn_ref::<Function>() {
                let update_fn = update_fn.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = update_fn.call0(&JsValue::NULL);
                });
            }
            JsValue::UNDEFINED
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        Reflect::set(
            &document_as_value(document),
            &JsValue::from_str("startViewTransition"),
            closure.as_ref().unchecked_ref(),
        )
        .expect("Reflect::set must not fail when shadowing a plain instance property");
        Self {
            call_count,
            _closure: closure,
        }
    }

    fn calls(&self) -> u32 {
        self.call_count.get()
    }
}

impl Drop for ViewTransitionStub {
    fn drop(&mut self) {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("document must exist");
        let _ = Reflect::delete_property(
            &document_as_object(&document),
            &JsValue::from_str("startViewTransition"),
        );
    }
}

/// `nav_browser.rs::NonFunctionViewTransitionShadow` の複製（doc 参照）。
struct NonFunctionViewTransitionShadow;

impl NonFunctionViewTransitionShadow {
    fn install(document: &Document) -> Self {
        Reflect::set(
            &document_as_value(document),
            &JsValue::from_str("startViewTransition"),
            &JsValue::from_str("not-a-function"),
        )
        .expect("Reflect::set must not fail when shadowing a plain instance property");
        Self
    }
}

impl Drop for NonFunctionViewTransitionShadow {
    fn drop(&mut self) {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("document must exist");
        let _ = Reflect::delete_property(
            &document_as_object(&document),
            &JsValue::from_str("startViewTransition"),
        );
    }
}

const FRAME_FALLBACK_TIMEOUT_MS: i32 = 100;

/// `nav_browser.rs::next_animation_frame` の複製（doc 参照。壁時計
/// フォールバック付きの 1 フレーム待機）。
async fn next_animation_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");

        let raf_resolve = resolve.clone();
        let raf_callback = Closure::once_into_js(move || {
            let _ = raf_resolve.call0(&JsValue::NULL);
        });
        window
            .request_animation_frame(raf_callback.unchecked_ref())
            .expect("requestAnimationFrame must not fail in test environment");

        let timeout_callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_callback.unchecked_ref(),
                FRAME_FALLBACK_TIMEOUT_MS,
            )
            .expect("setTimeout must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("requestAnimationFrame/setTimeout promise must not reject");
}

/// `nav_browser.rs::wait_until` の複製（doc 参照）。
async fn wait_until<F: Fn() -> bool>(condition: F, max_frames: u32) -> bool {
    for _ in 0..max_frames {
        if condition() {
            return true;
        }
        next_animation_frame().await;
    }
    condition()
}

/// テスト専用の最小 component。`label` を切り替える度に
/// `data-testid="label-view"` の内容が変わる（束縛点を使わない、
/// `apply_with_view_transition` が常に `state.view()` からサブツリー全体を
/// 再構築することを可視化するための最小構成）。
///
/// `label` は `Cell`（内部可変性）で持つ: `Runtime::component()` は
/// `Ref<'_, C>`（`&C` 相当）しか返さず、`Runtime` は本テスト専用のアプリ内
/// アクション経由の可変アクセス手段を公開していないため、`&self` から
/// 直接書き換えられる形にして [`apply_with_view_transition`] 単体の
/// 挙動（現在の `view()` から常に全再構築する）をテストの他要素
/// （イベント dispatch・束縛点・keyed list 更新経路）と独立に検証する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Label {
    Before,
    After,
}

struct LabelState {
    label: Cell<Label>,
    dirty: Vec<&'static str>,
}

impl LabelState {
    fn new() -> Self {
        Self {
            label: Cell::new(Label::Before),
            dirty: Vec::new(),
        }
    }

    fn set_label(&self, label: Label) {
        self.label.set(label);
    }
}

impl Component for LabelState {
    type Action = ();

    fn update(&mut self, _action: Self::Action) {
        // 本テストは `set_label`（`Cell` への直接書き込み）のみで状態を
        // 変更するため到達しない。`Component` トレイトの充足のためだけに
        // 存在する。
    }

    fn view(&self) -> Node {
        let text_content = match self.label.get() {
            Label::Before => "before",
            Label::After => "after",
        };
        el(
            "div",
            vec![("data-testid", "label-root")],
            vec![el(
                "span",
                vec![("data-testid", "label-view")],
                vec![text(text_content)],
            )],
        )
    }

    fn decode_action(_name: &str, _payload: &str) -> Option<Self::Action> {
        None
    }
}

impl DirtyTracked for LabelState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for LabelState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// 検証 A: 機能検出成功時、`apply_with_view_transition` が
/// `document.startViewTransition` を 1 回呼び、update コールバック
/// （非同期になり得る）実行後に `root` 配下が新しい状態から再構築されて
/// いること。
#[wasm_bindgen_test]
async fn apply_with_view_transition_uses_stub_and_rebuilds_subtree() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "view-transition-stub-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    let stub = ViewTransitionStub::install(&document);

    let runtime =
        Runtime::mount("view-transition-stub-root", LabelState::new()).expect("mount must succeed");

    assert!(
        placeholder
            .query_selector("[data-testid='label-view']")
            .expect("query_selector must not fail")
            .expect("label-view must exist")
            .text_content()
            .as_deref()
            == Some("before"),
        "初期状態は before であること"
    );

    runtime.component().set_label(Label::After);
    runtime.apply_with_view_transition();

    assert_eq!(
        stub.calls(),
        1,
        "document.startViewTransition がちょうど 1 回呼ばれること"
    );

    assert!(
        wait_until(
            || placeholder
                .query_selector("[data-testid='label-view']")
                .ok()
                .flatten()
                .and_then(|el| el.text_content())
                .as_deref()
                == Some("after"),
            60,
        )
        .await,
        "update コールバック実行後、root 配下が after を含む新規ノードへ \
         再構築されていること"
    );
}

/// 検証 B: 機能非対応（`startViewTransition` が非関数）時、呼び出し直後
/// （同期）に DOM が更新済みであること（graceful degradation）。
#[wasm_bindgen_test]
fn apply_with_view_transition_falls_back_synchronously_when_unsupported() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "view-transition-shadow-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    let _shadow = NonFunctionViewTransitionShadow::install(&document);

    let runtime = Runtime::mount("view-transition-shadow-root", LabelState::new())
        .expect("mount must succeed");

    runtime.component().set_label(Label::After);
    runtime.apply_with_view_transition();

    assert!(
        placeholder
            .query_selector("[data-testid='label-view']")
            .expect("query_selector must not fail")
            .expect("label-view must exist")
            .text_content()
            .as_deref()
            == Some("after"),
        "非対応ブラウザ相当のフォールバック経路では、呼び出し直後（同期）に \
         DOM が新しい状態へ更新済みであること"
    );
}
