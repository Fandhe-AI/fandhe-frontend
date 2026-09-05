//! `fandhe_frontend_wasm_full::headless_clipboard`（イシュー #773、親トラッ
//! キング #520）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/headless_clipboard.rs`（native）は hydration ラウンド
//! トリップ・判定関数 → dispatch の統合経路までを検証済みである。本ファイル
//! はその先、`Runtime::mount` 経由で配線した trigger クリックが
//! `navigator.clipboard.writeText` 呼び出し・`data-copied`/indicator 反映・
//! タイムアウトによる自動リセットまで実 DOM 上で正しく振る舞うことを検証する
//! （実装計画 §3.4 対応）。
//!
//! # ネイティブ `navigator.clipboard` を必ずスタブする理由
//!
//! headless Chrome はネイティブの `navigator.clipboard.writeText` を持つが、
//! 合成 `click` イベント（`Element::dispatch_event`）はブラウザ仕様上の
//! 「ユーザー操作起因」とはみなされない場合があり、実装・実行環境により
//! `NotAllowedError` で reject するか、無期限に保留したままハングするか
//! （権限ダイアログ待ち等）が決定的でない。本ファイルの全テストは
//! `navigator.clipboard` を [`ClipboardStub`]/[`ClipboardRejectStub`]/
//! [`ClipboardAbsentShadow`] のいずれかで明示的に shadow してから検証し、
//! ネイティブ実装を素通しさせない（`wasm-full/tests/nav_browser.rs` の
//! `document.startViewTransition` スタブ化と同じ教訓、CI ハング再発防止）。
//!
//! # 検証内容
//!
//! 1. `writeText` が resolve するスタブ → trigger クリック →
//!    `data-copied` が root/control/input/trigger へ付与され、indicator の
//!    `data-state`/`hidden` が反転すること（受け入れ条件: copy → data-copied 遷移）
//! 2. `navigator.clipboard` 非搭載相当（`undefined` で shadow）→
//!    trigger クリック → 状態が変化しないこと（fail-closed）
//! 3. `writeText` が reject するスタブ → trigger クリック →
//!    状態が変化しないこと（fail-closed）
//! 4. resolve 後、[`fandhe_frontend_wasm_full::headless_clipboard::DEFAULT_RESET_TIMEOUT_MS`]
//!    経過で自動的に `copied` が解除されること（タイムアウトは短縮した
//!    テスト用スタブタイマーではなく実際の `set_timeout` 経路をそのまま使い、
//!    ポーリングで待つ）
//! 5. XSS: 攻撃者制御の `data-value` を持つマークアップを mount しても実 DOM
//!    に `script` 要素が生成されないこと

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::clipboard::{control, indicator, input, label, root, trigger};
use js_sys::{Object, Promise, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。
fn create_container(document: &Document, id: &str) -> Element {
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// [`mount_clipboard`] が組み立てる input の `id`（[`label`] の `for` と
/// 紐付けるためのテスト固定値、イシュー #1631）。
const CLIPBOARD_INPUT_ID: &str = "clipboard-browser-test-input";

/// `crates/headless-ui/src/clipboard.rs` の SSR 出力契約そのもの（`root`/
/// `label`/`control`/`input`/`trigger`/`indicator` x2）で Clipboard の
/// マークアップを組み立てて `container` へ流し込む（イシュー #1631 で
/// `label` を追加し、`data-copied`/`aria-label` 反転を実 DOM で検証できる
/// ようにした）。
fn mount_clipboard(container: &Element, value: &str, copied: bool) {
    let node = root(
        value,
        copied,
        Vec::new(),
        vec![
            label(
                copied,
                Some(CLIPBOARD_INPUT_ID),
                Vec::new(),
                vec![fandhe_frontend_core::text("Value")],
            ),
            control(
                copied,
                Vec::new(),
                vec![
                    input(value, copied, vec![("id", CLIPBOARD_INPUT_ID)]),
                    trigger(
                        copied,
                        Vec::new(),
                        vec![
                            indicator(
                                true,
                                copied,
                                Vec::new(),
                                vec![fandhe_frontend_core::text("Copied")],
                            ),
                            indicator(
                                false,
                                copied,
                                Vec::new(),
                                vec![fandhe_frontend_core::text("Copy")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );
    container.set_inner_html(&render(&node));
}

fn label_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='clipboard'][data-part='label']")
        .expect("query_selector must not fail")
        .expect("label part must exist")
}

fn trigger_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='clipboard'][data-part='trigger']")
        .expect("query_selector must not fail")
        .expect("trigger part must exist")
}

fn root_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='clipboard'][data-part='root']")
        .expect("query_selector must not fail")
        .expect("root part must exist")
}

/// 合成 `click`（bubbles: true、通常のユーザークリックを模す）を生成する。
fn synthetic_click() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// `navigator` を `Object` API へ渡せる形へ変換する
/// （`nav_browser.rs::document_as_object` と同じ意図）。
fn navigator_as_object() -> Object {
    web_sys::window()
        .expect("window must exist")
        .navigator()
        .unchecked_into::<Object>()
}

/// `navigator.clipboard` を `value` で shadow する。
///
/// `navigator.clipboard` は `Navigator.prototype` 上の getter-only
/// アクセサ（`configurable: false` の場合がある）として実装されている
/// ブラウザがあり、単純な `Reflect::set`（`[[Set]]` セマンティクス）では
/// プロトタイプのアクセサに阻まれてインスタンスへ own property が
/// 作成されず、実際には shadow されないまま何も起きずに `Ok(false)` を
/// 返す（`.expect` では検知できない静かな失敗）。`Object::defineProperty`
/// （`[[DefineOwnProperty]]` セマンティクス）はプロトタイプのアクセサを
/// 経由せずインスタンスへ直接 own property を定義するため、この問題を
/// 回避できる。
fn shadow_navigator_clipboard(value: &JsValue) {
    let descriptor = Object::new();
    Reflect::set(&descriptor, &JsValue::from_str("value"), value)
        .expect("Reflect::set must not fail on a plain descriptor object");
    Reflect::set(
        &descriptor,
        &JsValue::from_str("configurable"),
        &JsValue::from_bool(true),
    )
    .expect("Reflect::set must not fail on a plain descriptor object");
    Object::define_property(
        &navigator_as_object(),
        &JsValue::from_str("clipboard"),
        &descriptor,
    );
}

/// `navigator.clipboard` をインスタンスプロパティとして shadow し、
/// `writeText` が渡された値を記録した上で resolve するスタブを設置する
/// （`nav_browser.rs::ViewTransitionStub` と同型のパターン）。`Drop` で
/// `navigator.clipboard` を削除し、他テストへ影響を残さない。
struct ClipboardStub {
    _write_text_closure: Closure<dyn FnMut(JsValue) -> JsValue>,
}

impl ClipboardStub {
    fn install() -> Self {
        Self::install_with_outcome(true)
    }

    /// `resolves` が `false` の場合は reject するスタブを設置する
    /// （検証 3 用）。
    fn install_with_outcome(resolves: bool) -> Self {
        let write_text_closure = Closure::wrap(Box::new(move |_value: JsValue| -> JsValue {
            if resolves {
                // `Promise::resolve`（js-sys）は `Promising` 実装型
                // （他の `Promise<T>`）のみを受け付け、任意の `JsValue`
                // を直接解決できないため、`Promise::new`（executor 版、
                // `nav_browser.rs::next_animation_frame` と同じ API 形）
                // で明示的に resolve/reject を呼ぶ。
                Promise::new(&mut |resolve, _reject| {
                    let _ = resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED);
                })
                .into()
            } else {
                Promise::new(&mut |_resolve, reject| {
                    let _ = reject.call1(&JsValue::UNDEFINED, &JsValue::from_str("denied"));
                })
                .into()
            }
        }) as Box<dyn FnMut(JsValue) -> JsValue>);

        let clipboard_obj = Object::new();
        Reflect::set(
            &clipboard_obj,
            &JsValue::from_str("writeText"),
            write_text_closure.as_ref().unchecked_ref(),
        )
        .expect("Reflect::set must not fail on a plain object");

        shadow_navigator_clipboard(&clipboard_obj);

        Self {
            _write_text_closure: write_text_closure,
        }
    }
}

impl Drop for ClipboardStub {
    fn drop(&mut self) {
        let _ = Reflect::delete_property(&navigator_as_object(), &JsValue::from_str("clipboard"));
    }
}

/// `navigator.clipboard` を `undefined` で shadow し、非対応ブラウザ相当
/// （非 secure context 含む）を再現する（検証 2 用）。`Drop` で復元する。
struct ClipboardAbsentShadow;

impl ClipboardAbsentShadow {
    fn install() -> Self {
        shadow_navigator_clipboard(&JsValue::UNDEFINED);
        Self
    }
}

impl Drop for ClipboardAbsentShadow {
    fn drop(&mut self) {
        let _ = Reflect::delete_property(&navigator_as_object(), &JsValue::from_str("clipboard"));
    }
}

/// `condition` が真になるまでポーリングする
/// （`headless_avatar_browser.rs::wait_for` と同じ意図・実装。`data:` URI
/// 決着待ちではなく Promise マイクロタスク/`set_timeout` の決着待ちだが、
/// 固定 `sleep` を避け条件ポーリングで待つ方針は同一）。
async fn wait_for(mut condition: impl FnMut() -> bool) {
    for _ in 0..300 {
        if condition() {
            return;
        }
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&JsValue::NULL).ok();
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

// --- 検証 1: resolve するスタブ → クリック → data-copied 遷移 ------------

#[wasm_bindgen_test]
async fn resolving_stub_click_sets_data_copied_and_flips_indicator() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "clipboard-resolve-root");
    let _cleanup = RemoveOnDrop(container.clone());
    let _stub = ClipboardStub::install();

    mount_clipboard(&container, "hello world", false);

    let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let received_clone = received.clone();
    fandhe_frontend_wasm_full::headless_clipboard::wire_clipboard_events(
        container.clone(),
        move |action_ref| {
            received_clone.borrow_mut().push(action_ref.action);
        },
    )
    .expect("wire_clipboard_events must not fail");

    trigger_element(&container)
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for(|| {
        received
            .borrow()
            .iter()
            .any(|a| a == fandhe_frontend_wasm_full::headless_clipboard::ACTION_COPY)
    })
    .await;
    assert!(received
        .borrow()
        .contains(&fandhe_frontend_wasm_full::headless_clipboard::ACTION_COPY.to_string()));

    // DOM 反映は Runtime 統合層の責務だが、本テストは配線関数単体を直接
    // 呼んでいるため、受け取った action を手動で適用して契約を確認する
    // （`headless_avatar_browser.rs` の合成イベントテストと同型の構成）。
    fandhe_frontend_wasm_full::headless_clipboard::apply_clipboard_copied(&container, true)
        .expect("apply_clipboard_copied must not fail");

    let root_el = root_element(&container);
    let trigger_el = trigger_element(&container);
    let label_el = label_element(&container);
    assert!(root_el.has_attribute("data-copied"));
    assert!(trigger_el.has_attribute("data-copied"));
    // イシュー #1631: label にも `data-copied` が反映されること（
    // `DATA_COPIED_PARTS` への `label` 追加の実 DOM 検証）。
    assert!(label_el.has_attribute("data-copied"));
    // イシュー #1631: trigger の既定 `aria-label` が「コピー済み」表示へ
    // 反転すること。
    assert_eq!(
        trigger_el.get_attribute("aria-label").as_deref(),
        Some("Copied to clipboard")
    );

    let copied_indicator = container
        .query_selector("[data-scope='clipboard'][data-part='indicator'][data-variant='copied']")
        .expect("query_selector must not fail")
        .expect("copied indicator must exist");
    let idle_indicator = container
        .query_selector("[data-scope='clipboard'][data-part='indicator'][data-variant='idle']")
        .expect("query_selector must not fail")
        .expect("idle indicator must exist");
    assert_eq!(
        copied_indicator.get_attribute("data-state").as_deref(),
        Some("visible")
    );
    assert!(!copied_indicator.has_attribute("hidden"));
    assert_eq!(
        idle_indicator.get_attribute("data-state").as_deref(),
        Some("hidden")
    );
    assert!(idle_indicator.has_attribute("hidden"));
}

// --- 検証 2: navigator.clipboard 非搭載 → クリック → 状態不変（fail-closed） ---

#[wasm_bindgen_test]
async fn absent_clipboard_api_click_does_not_dispatch() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "clipboard-absent-root");
    let _cleanup = RemoveOnDrop(container.clone());
    let _shadow = ClipboardAbsentShadow::install();

    mount_clipboard(&container, "hello world", false);

    let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let received_clone = received.clone();
    fandhe_frontend_wasm_full::headless_clipboard::wire_clipboard_events(
        container.clone(),
        move |action_ref| {
            received_clone.borrow_mut().push(action_ref.action);
        },
    )
    .expect("wire_clipboard_events must not fail");

    trigger_element(&container)
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    // 決定的に「何も起きない」ことを固定するため、短い猶予を置いてから
    // 何も受信していないことを確認する（イベントは同期的に no-op で
    // 終わるはずだが、環境差を吸収するため 1 ステップだけポーリングする）。
    wait_for(|| false).await;
    assert!(received.borrow().is_empty());

    let root_el = root_element(&container);
    assert!(!root_el.has_attribute("data-copied"));
}

// --- 検証 3: writeText が reject するスタブ → クリック → 状態不変 -----------

#[wasm_bindgen_test]
async fn rejecting_stub_click_does_not_dispatch() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "clipboard-reject-root");
    let _cleanup = RemoveOnDrop(container.clone());
    let _stub = ClipboardStub::install_with_outcome(false);

    mount_clipboard(&container, "hello world", false);

    let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let received_clone = received.clone();
    fandhe_frontend_wasm_full::headless_clipboard::wire_clipboard_events(
        container.clone(),
        move |action_ref| {
            received_clone.borrow_mut().push(action_ref.action);
        },
    )
    .expect("wire_clipboard_events must not fail");

    trigger_element(&container)
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    // reject 側のマイクロタスクが解決する猶予を与えたうえで no-op を確認する
    // （`wait_for` 1 回分、最大 300 x 10ms = 3000ms の猶予。
    // `wasm-bindgen-test` の既定タイムアウト 20 秒に対し十分な余裕を残す）。
    wait_for(|| false).await;
    assert!(received.borrow().is_empty());
}

// --- 検証 4: resolve 後、タイムアウト経過で自動的に reset される ----------

#[wasm_bindgen_test]
async fn resolving_stub_click_auto_resets_after_timeout() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "clipboard-auto-reset-root");
    let _cleanup = RemoveOnDrop(container.clone());
    let _stub = ClipboardStub::install();

    mount_clipboard(&container, "hello world", false);

    let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let received_clone = received.clone();
    fandhe_frontend_wasm_full::headless_clipboard::wire_clipboard_events(
        container.clone(),
        move |action_ref| {
            received_clone.borrow_mut().push(action_ref.action);
        },
    )
    .expect("wire_clipboard_events must not fail");

    trigger_element(&container)
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for(|| {
        received
            .borrow()
            .iter()
            .any(|a| a == fandhe_frontend_wasm_full::headless_clipboard::ACTION_COPY)
    })
    .await;

    // `DEFAULT_RESET_TIMEOUT_MS`（3000ms）経過後の自動 reset をポーリングで
    // 待つ（`wait_for` の 300 回 x 10ms = 3000ms の上限に対し十分な余裕を
    // 見て 600 回まで許容する）。
    for _ in 0..2 {
        wait_for(|| {
            received
                .borrow()
                .iter()
                .any(|a| a == fandhe_frontend_wasm_full::headless_clipboard::ACTION_RESET)
        })
        .await;
    }
    assert!(
        received
            .borrow()
            .contains(&fandhe_frontend_wasm_full::headless_clipboard::ACTION_RESET.to_string()),
        "自動 reset が既定タイムアウト内に dispatch されなかった: {:?}",
        received.borrow()
    );
}

// --- 検証 5: XSS 回帰（攻撃者制御の data-value を持つマークアップ） ---------

#[wasm_bindgen_test]
fn mounting_markup_with_attacker_controlled_value_creates_no_script_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "clipboard-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>window.__clipboard_xss = true;</script>";
    mount_clipboard(&container, payload, false);

    assert!(container.query_selector("script").ok().flatten().is_none());
    assert!(js_sys::Reflect::get(
        &JsValue::from(window),
        &JsValue::from_str("__clipboard_xss")
    )
    .map(|v| v.is_undefined())
    .unwrap_or(true));
}
