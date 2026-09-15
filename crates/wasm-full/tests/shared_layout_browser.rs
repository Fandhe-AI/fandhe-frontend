//! `crate::shared_layout`（イシュー #2536、feature `layout-animation`）の
//! `data-*` 配線を実ブラウザ上で駆動する統合テスト（`wasm-pack test
//! --headless --chrome`）。`layout_flip_browser.rs`（同一要素の並べ替え）
//! とは対象が異なり、本ファイルは「別要素が同じ役割を引き継ぐ」ケース
//! （`Runtime::rerender`/`apply_with_view_transition` が `root` サブツリー
//! を丸ごと再構築し、`data-fandhe-layout-id` の値が同じ新要素が旧要素とは
//! 別の DOM インスタンスになる）を検証する。
//!
//! `document.startViewTransition` の記録用スタブ・機能非対応シャドウは
//! `runtime_view_transition_browser.rs` と同じ意図の自己完結な複製
//! （同ファイル doc 参照）。

#![cfg(target_arch = "wasm32")]
// `Runtime::apply_with_view_transition`（feature `view-transitions`）経由で
// 共有レイアウト遷移を駆動するため、`layout-animation` 単体構成（CI の
// wasm-full feature matrix `-wiring` ジョブ、イシュー #2578）では
// `apply_with_view_transition` 自体が存在せずコンパイルできない。両
// feature を要求する。
#![cfg(all(feature = "layout-animation", feature = "view-transitions"))]

use fandhe_frontend_core::{el, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::shared_layout::LAYOUT_ID_ATTR;
use fandhe_frontend_wasm_full::Runtime;
use js_sys::{Function, Reflect};
use std::cell::Cell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// `runtime_view_transition_browser.rs::create_placeholder` と同じ意図。
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

/// `runtime_view_transition_browser.rs::ViewTransitionStub` の複製。
struct ViewTransitionStub {
    _closure: Closure<dyn FnMut(JsValue) -> JsValue>,
}

impl ViewTransitionStub {
    fn install(document: &Document) -> Self {
        let closure = Closure::wrap(Box::new(move |update: JsValue| -> JsValue {
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

/// `runtime_view_transition_browser.rs::NonFunctionViewTransitionShadow` の複製。
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

/// `runtime_view_transition_browser.rs::next_animation_frame` の複製。
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

/// `runtime_view_transition_browser.rs::wait_until` の複製。
async fn wait_until<F: Fn() -> bool>(condition: F, max_frames: u32) -> bool {
    for _ in 0..max_frames {
        if condition() {
            return true;
        }
        next_animation_frame().await;
    }
    condition()
}

/// テスト専用の最小 component。`state`（`Before`/`After`）に応じて、
/// [`LAYOUT_ID_ATTR`]="indicator" を持つ要素の位置・サイズ（絶対配置の
/// inline `style`）を変える。`view()` は毎回 `data-testid` を持つ**新規
/// ノード**を構築する（束縛点・keyed list を使わないため、`Runtime::
/// rerender`/`apply_with_view_transition` の全再描画経路でのみ更新が
/// 反映される。`shared_layout`（別要素引き継ぎ）の対象範囲を検証する
/// ための最小構成）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    Before,
    After,
}

struct IndicatorState {
    position: Cell<Position>,
    dirty: Vec<&'static str>,
}

impl IndicatorState {
    fn new() -> Self {
        Self {
            position: Cell::new(Position::Before),
            dirty: Vec::new(),
        }
    }
}

impl Component for IndicatorState {
    type Action = ();

    fn update(&mut self, _action: Self::Action) {}

    fn view(&self) -> Node {
        let style = match self.position.get() {
            Position::Before => "position:absolute;left:0px;top:0px;width:50px;height:50px",
            Position::After => "position:absolute;left:150px;top:80px;width:80px;height:80px",
        };
        el(
            "div",
            vec![("data-testid", "indicator-root")],
            vec![el(
                "span",
                vec![
                    ("data-testid", "indicator"),
                    (LAYOUT_ID_ATTR, "indicator"),
                    ("style", style),
                ],
                vec![],
            )],
        )
    }

    fn decode_action(_name: &str, _payload: &str) -> Option<Self::Action> {
        None
    }
}

impl DirtyTracked for IndicatorState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for IndicatorState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

fn indicator_transform(placeholder: &Element) -> String {
    placeholder
        .query_selector("[data-testid='indicator']")
        .expect("query_selector must not fail")
        .expect("indicator must exist")
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("indicator must be an HtmlElement")
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail")
}

/// dispatch/rerender 経路（VT を介さない）は常に共有レイアウト遷移を
/// 起動する: `rerender()` 直後は補正 `transform` が書き込まれており、
/// 十分な時間待てば解除される。
#[wasm_bindgen_test]
async fn rerender_path_applies_shared_flip_between_differing_elements() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "shared-layout-rerender-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("shared-layout-rerender-root", IndicatorState::new())
        .expect("mount must succeed");
    assert!(
        indicator_transform(&placeholder).is_empty(),
        "初期表示には補正 transform がないはず"
    );

    runtime.component().position.set(Position::After);
    runtime.rerender();

    let transform_after_rerender = indicator_transform(&placeholder);
    assert!(
        !transform_after_rerender.is_empty(),
        "rerender 直後は別要素間の共有レイアウト遷移が補正 transform を \
         書き込んでいるはず"
    );

    assert!(
        wait_until(|| indicator_transform(&placeholder).is_empty(), 240).await,
        "十分な時間待てば FLIP が収束し transform が解除されているはず"
    );
}

/// View Transitions が使える場合は UA 側の同名要素 morph に委譲し、
/// 共有レイアウト遷移（本モジュール）は起動しない（実装計画「VT 委譲
/// 判定」）。
#[wasm_bindgen_test]
async fn view_transition_supported_delegates_and_writes_no_transform() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "shared-layout-vt-supported-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    let _stub = ViewTransitionStub::install(&document);

    let runtime = Runtime::mount("shared-layout-vt-supported-root", IndicatorState::new())
        .expect("mount must succeed");

    runtime.component().position.set(Position::After);
    runtime.apply_with_view_transition();

    // `style` 属性の**リテラル文字列**ではなく CSSOM 経由の
    // `get_property_value("left")` で判定する: `assign_transition_names`
    // （`shared_flip == false` 経路、UA 委譲時に本テストが起動する）が
    // 同じ要素へ `style().set_property("view-transition-name", ...)` を
    // 書き込むと、ブラウザは `style` 属性全体を CSSOM の正規シリアライズ
    // （コロン後に半角スペースを挿入）で書き戻すため、`el()` 構築時の
    // リテラル `"left:150px"`（スペースなし）との部分文字列一致が壊れる
    // （実装の不具合ではなく本アサーション側の脆さ）。
    assert!(
        wait_until(
            || {
                placeholder
                    .query_selector("[data-testid='indicator']")
                    .ok()
                    .flatten()
                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                    .is_some_and(|el| {
                        el.style().get_property_value("left").ok().as_deref() == Some("150px")
                    })
            },
            60,
        )
        .await,
        "update コールバック実行後、after の位置へ再構築されていること"
    );
    assert!(
        indicator_transform(&placeholder).is_empty(),
        "VT 対応時は共有レイアウト遷移を起動しないため transform は書き \
         込まれないはず"
    );
}

/// View Transitions が非対応（機能検出失敗、同期フォールバック）の場合は
/// 共有レイアウト遷移を起動する。
#[wasm_bindgen_test]
fn view_transition_unsupported_falls_back_to_shared_flip() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "shared-layout-vt-unsupported-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    let _shadow = NonFunctionViewTransitionShadow::install(&document);

    let runtime = Runtime::mount("shared-layout-vt-unsupported-root", IndicatorState::new())
        .expect("mount must succeed");

    runtime.component().position.set(Position::After);
    runtime.apply_with_view_transition();

    assert!(
        !indicator_transform(&placeholder).is_empty(),
        "非対応ブラウザ相当の同期フォールバック経路では共有レイアウト \
         遷移が起動し、呼び出し直後に補正 transform が書き込まれている \
         はず"
    );
}

/// [`LAYOUT_ID_ATTR`] を持たない要素は共有レイアウト遷移の対象外であり、
/// `rerender` を経ても transform は一切書き込まれない。
struct PlainState {
    dirty: Vec<&'static str>,
}

impl Component for PlainState {
    type Action = ();

    fn update(&mut self, _action: Self::Action) {}

    fn view(&self) -> Node {
        el(
            "div",
            vec![("data-testid", "plain-root")],
            vec![el(
                "span",
                vec![("data-testid", "plain"), ("style", "position:absolute")],
                vec![],
            )],
        )
    }

    fn decode_action(_name: &str, _payload: &str) -> Option<Self::Action> {
        None
    }
}

impl DirtyTracked for PlainState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for PlainState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

#[wasm_bindgen_test]
fn element_without_attr_is_untouched() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "shared-layout-plain-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("shared-layout-plain-root", PlainState { dirty: Vec::new() })
        .expect("mount must succeed");
    runtime.rerender();

    let transform = placeholder
        .query_selector("[data-testid='plain']")
        .expect("query_selector must not fail")
        .expect("plain must exist")
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("plain must be an HtmlElement")
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert!(
        transform.is_empty(),
        "data-fandhe-layout-id を持たない要素には共有レイアウト遷移が \
         適用されないはず"
    );
}
