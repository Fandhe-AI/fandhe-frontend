//! `Runtime::apply_with_view_transition_named`（イシュー #2516、feature
//! `view-transition-preset`）の実ブラウザ統合テスト（`wasm-pack test
//! --headless --chrome`）。
//!
//! `document.startViewTransition` の記録用スタブ・機能非対応シャドウは
//! `runtime_view_transition_browser.rs` の同名ヘルパーと同じ意図の
//! 自己完結な複製である（テスト実行単位の制約上、他テストバイナリの
//! private 定義を import できないため、本リポジトリの既存流儀どおり
//! 複製する）。
//!
//! - 検証 A: `apply_with_view_transition_named(Slide)` 呼び出し後、
//!   `document.documentElement` の `data-fandhe-view-transition` 属性が
//!   `"slide"` になること。
//! - 検証 B: その後 `apply_with_view_transition()`（unnamed）を呼ぶと
//!   属性が除去される（named/unnamed 混在時の是正、`Self::
//!   apply_with_view_transition` 冒頭の属性除去が効くこと）。
//! - 検証 C: 機能非対応（`startViewTransition` が非関数）でも、呼び出し
//!   直後（同期）に属性が設定され DOM が更新済みであること
//!   （graceful degradation）。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "view-transition-preset")]

use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::view_transition_preset::{
    ViewTransitionPreset, VIEW_TRANSITION_PRESET_ATTR,
};
use fandhe_frontend_wasm_full::Runtime;
use js_sys::{Function, Reflect};
use std::cell::Cell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

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

/// `runtime_view_transition_browser.rs::ViewTransitionStub` の複製
/// （doc 参照。呼び出し回数の記録は本テストでは不要なため省く）。
/// update コールバックを同期的に即実行する（本テストは遅延・競合の
/// 検証を対象としないため）。
struct ViewTransitionStub {
    _closure: Closure<dyn FnMut(JsValue) -> JsValue>,
}

impl ViewTransitionStub {
    fn install(document: &Document) -> Self {
        let closure = Closure::wrap(Box::new(move |update: JsValue| -> JsValue {
            if let Some(update_fn) = update.dyn_ref::<Function>() {
                let _ = update_fn.call0(&JsValue::NULL);
            }
            JsValue::UNDEFINED
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        Reflect::set(
            &document_as_value(document),
            &JsValue::from_str("startViewTransition"),
            closure.as_ref().unchecked_ref(),
        )
        .expect("Reflect::set must not fail when shadowing a plain instance property");
        Self { _closure: closure }
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

/// `runtime_view_transition_browser.rs::NonFunctionViewTransitionShadow`
/// の複製（doc 参照）。
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

/// `runtime_view_transition_browser.rs::LabelState` と同型の最小 component。
struct LabelState {
    label: Cell<&'static str>,
    dirty: Vec<&'static str>,
}

impl LabelState {
    fn new() -> Self {
        Self {
            label: Cell::new("before"),
            dirty: Vec::new(),
        }
    }
}

impl Component for LabelState {
    type Action = ();
    fn update(&mut self, _action: Self::Action) {}
    fn view(&self) -> Node {
        el(
            "div",
            vec![("data-testid", "label-root")],
            vec![el(
                "span",
                vec![("data-testid", "label-view")],
                vec![text(self.label.get())],
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

fn document_element_attr(document: &Document) -> Option<String> {
    document
        .document_element()
        .and_then(|el| el.get_attribute(VIEW_TRANSITION_PRESET_ATTR))
}

/// 検証 A: named プリセット選択で属性が設定されること。
/// `wasm-full-feature-matrix-wiring`（CI、`view-transition-preset` 単体
/// 構成）でも実行できるよう、`view-transitions` feature 前提の検証 B
/// （unnamed 呼び出しでの除去）は別関数（下記）へ分離する。
#[wasm_bindgen_test]
fn named_preset_sets_attr() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "view-transition-preset-root");
    let _cleanup = RemoveOnDrop(placeholder);
    let _stub = ViewTransitionStub::install(&document);

    let runtime = Runtime::mount("view-transition-preset-root", LabelState::new())
        .expect("mount must succeed");

    runtime.apply_with_view_transition_named(ViewTransitionPreset::Slide);
    assert_eq!(
        document_element_attr(&document).as_deref(),
        Some("slide"),
        "named プリセット呼び出し後、data-fandhe-view-transition=\"slide\" が \
         document.documentElement へ設定されること"
    );
}

/// 検証 B: named プリセット選択後、unnamed 呼び出し
/// （`apply_with_view_transition`、feature `view-transitions`）で属性が
/// 除去されること（named/unnamed 混在時の残留を防ぐ是正）。
/// `wasm-full-feature-matrix-wiring` の `view-transition-preset` 単体
/// 構成には `view-transitions` を含めないため、両 feature を要求する
/// 本テストは別ゲートにする。
#[cfg(feature = "view-transitions")]
#[wasm_bindgen_test]
fn unnamed_call_clears_named_preset_attr() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "view-transition-preset-clear-root");
    let _cleanup = RemoveOnDrop(placeholder);
    let _stub = ViewTransitionStub::install(&document);

    let runtime = Runtime::mount("view-transition-preset-clear-root", LabelState::new())
        .expect("mount must succeed");

    runtime.apply_with_view_transition_named(ViewTransitionPreset::Slide);
    assert_eq!(document_element_attr(&document).as_deref(), Some("slide"));

    runtime.apply_with_view_transition();
    assert_eq!(
        document_element_attr(&document),
        None,
        "unnamed 呼び出し後、named プリセットの属性が除去されていること \
         （named/unnamed 混在時の残留を防ぐ是正）"
    );
}

/// 検証 C: 機能非対応でも同期的に属性が設定され DOM が更新されること。
#[wasm_bindgen_test]
fn named_preset_falls_back_synchronously_when_unsupported() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "view-transition-preset-shadow-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    let _shadow = NonFunctionViewTransitionShadow::install(&document);

    let runtime = Runtime::mount("view-transition-preset-shadow-root", LabelState::new())
        .expect("mount must succeed");

    runtime.component().label.set("after");
    runtime.apply_with_view_transition_named(ViewTransitionPreset::Wipe);

    assert_eq!(document_element_attr(&document).as_deref(), Some("wipe"));
    assert!(
        placeholder
            .query_selector("[data-testid='label-view']")
            .expect("query_selector must not fail")
            .expect("label-view must exist")
            .text_content()
            .as_deref()
            == Some("after"),
        "非対応ブラウザ相当のフォールバック経路でも、呼び出し直後（同期）に \
         DOM が新しい状態へ更新済みであること"
    );
}
