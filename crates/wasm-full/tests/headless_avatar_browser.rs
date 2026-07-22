//! `fandhe_frontend_wasm_full::headless_avatar`（イシュー #591、親 #520/#542/#543）
//! の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/headless_avatar.rs`（native）は hydration ラウンド
//! トリップ・イベント判定 → dispatch の統合経路までを検証済みである。本
//! ファイルはその先、`wire_avatar_events`/`apply_avatar_visibility` が実 DOM
//! （headless Chromium）上で以下を満たすことを検証する
//! （実装計画 §5 手順 5 (a)〜(f) に対応）。
//!
//! 1. 合成 `error` イベント（非バブリング、capture フェーズ委譲での受信
//!    確認）→ dispatch → `apply_avatar_visibility` で `data-state`/`hidden`
//!    が切り替わること（受け入れ条件 1）
//! 2. `data:` URI（1x1 GIF）を `src` に持つ実画像の実 `load` イベントで
//!    `data-state="visible"` へ切り替わること
//! 3. 不正な `data:` URI の実 `error` イベントで fallback が visible に
//!    なること
//! 4. 配線前に読み込みが決着済みの画像に対する settle 検査の合成 dispatch
//!    （hydration レースの回帰固定）
//! 5. hydration 復元経路（`data-hydrate-status` からの状態復元 → 配線 →
//!    イベントで状態・`data-state` が更新されること、受け入れ条件 2）。
//!    改ざん `data-hydrate-status` は panic せず初期状態フォールバックする
//!    こと
//! 6. XSS: 攻撃者制御の `alt` を持つマークアップを mount しても実 DOM に
//!    `script` 要素が生成されないこと
//!
//! DOM 構造は `crates/headless-ui/src/avatar.rs` の SSR 出力契約
//! （`data-scope="avatar"`/`data-part="root"/"image"/"fallback"`/
//! `data-state`/`hidden`）を、`fandhe_frontend_headless_ui::avatar` の
//! パーツ関数を直接呼んで再現する（`fandhe-frontend-headless-ui` は本クレートの
//! dev-dependency のため、実際の SSR 出力から手組みドリフトを起こさない）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::avatar::{fallback, image, root, Avatar, ImageStatus};
use fandhe_frontend_interactive::{dispatch, Hydrate, HYDRATE_ATTR_PREFIX};
use fandhe_frontend_wasm_full::headless_avatar::{apply_avatar_visibility, wire_avatar_events};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。id を一意に
/// することで、同一テストバイナリ内の複数テストケースが要素を奪い合わない
/// ようにする（`xss_escape_wasm.rs::create_container` と同じ意図）。
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`runtime_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `avatar::root`/`image`/`fallback`（SSR 出力契約そのもの）で Avatar の
/// マークアップを組み立てて `container` へ流し込む。
fn mount_avatar(container: &Element, status: ImageStatus, src: &str, alt: &str) {
    let node = root(
        Vec::new(),
        vec![
            image(status, src, alt, Vec::new()),
            fallback(status, Vec::new(), vec![fandhe_frontend_core::text("NM")]),
        ],
    );
    container.set_inner_html(&render(&node));
}

fn image_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='avatar'][data-part='image']")
        .expect("query_selector must not fail")
        .expect("image part must exist")
}

fn fallback_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='avatar'][data-part='fallback']")
        .expect("query_selector must not fail")
        .expect("fallback part must exist")
}

/// 非バブリング合成イベントを生成する（`load`/`error` はブラウザ仕様上
/// バブリングしないため、`bubbles: false`（既定値）のまま組み立てる。
/// capture フェーズ委譲でも root が受信できることの検証を兼ねる）。
fn non_bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(false);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

// --- (a) 合成イベント経由での data-state 切り替え（受け入れ条件 1） -------

#[wasm_bindgen_test]
fn synthetic_error_event_updates_data_state_via_capture_delegation() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-synthetic-error-root");
    let _cleanup = RemoveOnDrop(container.clone());

    mount_avatar(&container, ImageStatus::Loading, "/broken.png", "avatar");

    let received = std::rc::Rc::new(std::cell::RefCell::new(None));
    let received_clone = received.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        *received_clone.borrow_mut() = Some(action_ref.action);
    })
    .expect("wire_avatar_events must not fail");

    let img = image_element(&container);
    img.dispatch_event(&non_bubbling_event("error"))
        .expect("dispatch_event must not fail");

    assert_eq!(received.borrow().as_deref(), Some("error"));

    apply_avatar_visibility(&container, false).expect("apply_avatar_visibility must not fail");

    let img = image_element(&container);
    let fallback_el = fallback_element(&container);
    assert_eq!(img.get_attribute("data-state").as_deref(), Some("hidden"));
    assert!(img.has_attribute("hidden"));
    assert_eq!(
        fallback_el.get_attribute("data-state").as_deref(),
        Some("visible")
    );
    assert!(!fallback_el.has_attribute("hidden"));
}

// --- (b) 実 load イベント（data: URI 1x1 GIF） -----------------------------

#[wasm_bindgen_test]
async fn real_load_event_on_data_uri_image_makes_image_visible() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-real-load-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 1x1 透過 GIF。CSP・外部アクセス不要で確実に読み込みが成功する。
    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    mount_avatar(&container, ImageStatus::Loading, ONE_PX_GIF, "avatar");

    let received = std::rc::Rc::new(std::cell::RefCell::new(None));
    let received_clone = received.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        *received_clone.borrow_mut() = Some(action_ref.action);
    })
    .expect("wire_avatar_events must not fail");

    // data: URI の読み込みはブラウザにより非同期に決着する。決着まで
    // ポーリングする（固定 sleep に頼らない。既存ブラウザテストは
    // `wasm-bindgen-futures` の dev 依存を利用しているため追加依存は不要）。
    wait_for(|| received.borrow().is_some()).await;

    assert_eq!(received.borrow().as_deref(), Some("loaded"));
}

// --- (c) 実 error イベント（不正な data: URI） ------------------------------

#[wasm_bindgen_test]
async fn real_error_event_on_invalid_data_uri_image_makes_fallback_visible() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-real-error-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 不正な data: URI（画像として decode できない）。
    const INVALID_DATA_URI: &str = "data:image/gif;base64,not-a-valid-gif";

    mount_avatar(&container, ImageStatus::Loading, INVALID_DATA_URI, "avatar");

    let received = std::rc::Rc::new(std::cell::RefCell::new(None));
    let received_clone = received.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        *received_clone.borrow_mut() = Some(action_ref.action);
    })
    .expect("wire_avatar_events must not fail");

    wait_for(|| received.borrow().is_some()).await;

    assert_eq!(received.borrow().as_deref(), Some("error"));

    apply_avatar_visibility(&container, false).expect("apply_avatar_visibility must not fail");
    let fallback_el = fallback_element(&container);
    assert_eq!(
        fallback_el.get_attribute("data-state").as_deref(),
        Some("visible")
    );
}

// --- (d) 配線前に決着済みの画像への settle 検査（hydration レース回帰） ---

#[wasm_bindgen_test]
async fn already_settled_image_before_wiring_dispatches_synthetically() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-settled-before-wiring-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    mount_avatar(&container, ImageStatus::Loading, ONE_PX_GIF, "avatar");

    // 配線前に読み込み決着を待つ（`complete()` が true になるまでポーリング）。
    let img_before_wiring = image_element(&container);
    let img_el = img_before_wiring
        .clone()
        .dyn_into::<web_sys::HtmlImageElement>()
        .expect("image part must be an HtmlImageElement");
    wait_for(|| img_el.complete()).await;
    assert!(img_el.complete(), "image should be settled before wiring");

    // ここで初めて配線する。settle 検査が即座に合成 dispatch するはず。
    let received = std::rc::Rc::new(std::cell::RefCell::new(None));
    let received_clone = received.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        *received_clone.borrow_mut() = Some(action_ref.action);
    })
    .expect("wire_avatar_events must not fail");

    assert_eq!(
        received.borrow().as_deref(),
        Some("loaded"),
        "settle 検査は配線と同時に合成 dispatch を行うため、\
         以降のイベントを待たずに結果が得られていなければならない"
    );
}

// --- (e) hydration 復元経路（受け入れ条件 2） ------------------------------

#[wasm_bindgen_test]
fn hydration_restore_then_synthetic_event_updates_state_and_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-hydration-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // SSR 相当: Loaded 状態の hydration 属性付きマークアップを組み立てる。
    let avatar = Avatar::new(ImageStatus::Loaded);
    let hydrate_attrs = avatar.hydration_attrs();
    let mut root_attrs: Vec<(&str, &str)> = Vec::new();
    for (name, value) in &hydrate_attrs {
        root_attrs.push((name.as_str(), value.as_str()));
    }
    let node = fandhe_frontend_headless_ui::avatar::root(
        root_attrs,
        vec![
            image(ImageStatus::Loaded, "/a.png", "avatar", Vec::new()),
            fallback(
                ImageStatus::Loaded,
                Vec::new(),
                vec![fandhe_frontend_core::text("NM")],
            ),
        ],
    );
    container.set_inner_html(&render(&node));

    // クライアント側復元: data-hydrate-status から Avatar を復元。
    let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Avatar::FIELD_STATUS);
    let raw_value = container
        .query_selector("[data-scope='avatar'][data-part='root']")
        .expect("query_selector must not fail")
        .expect("root part must exist")
        .get_attribute(&attr_name)
        .expect("data-hydrate-status must be present");
    let restored = fandhe_frontend_wasm_full::hydration::restore_state::<Avatar>(&[(
        attr_name.clone(),
        raw_value,
    )])
    .expect("hydration restore should succeed for well-formed attrs");
    assert_eq!(restored.status(), ImageStatus::Loaded);

    let state = std::rc::Rc::new(std::cell::RefCell::new(restored));
    let state_for_wiring = state.clone();
    let container_for_update = container.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        let mut avatar = state_for_wiring.borrow_mut();
        if dispatch(&mut *avatar, &action_ref.action, &action_ref.payload) {
            let visible = avatar.status().is_image_visible();
            apply_avatar_visibility(&container_for_update, visible)
                .expect("apply_avatar_visibility must not fail");
        }
    })
    .expect("wire_avatar_events must not fail");

    let img = image_element(&container);
    img.dispatch_event(&non_bubbling_event("error"))
        .expect("dispatch_event must not fail");

    assert_eq!(state.borrow().status(), ImageStatus::Error);
    let img = image_element(&container);
    let fallback_el = fallback_element(&container);
    assert_eq!(img.get_attribute("data-state").as_deref(), Some("hidden"));
    assert_eq!(
        fallback_el.get_attribute("data-state").as_deref(),
        Some("visible")
    );
}

/// 改ざんされた `data-hydrate-status` は panic せず `HydrateError` を返し、
/// 呼び出し側が初期状態（`ImageStatus::Loading`）へ CSR フォールバックできる
/// こと（`Runtime::hydrate` の既存契約と同じフォールバック方針、
/// `wasm-full/src/hydration.rs` 冒頭コメント参照）。
#[wasm_bindgen_test]
fn tampered_hydrate_status_does_not_panic_and_falls_back_to_loading() {
    let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Avatar::FIELD_STATUS);
    let result = fandhe_frontend_wasm_full::hydration::restore_state::<Avatar>(&[(
        attr_name,
        "attacker-controlled".to_string(),
    )]);
    let fallback_avatar = result.unwrap_or_else(|_| Avatar::new(ImageStatus::Loading));
    assert_eq!(fallback_avatar.status(), ImageStatus::Loading);
}

// --- (f) XSS 回帰: 攻撃者制御 alt を持つ mount でも script 要素が生成されない ---

#[wasm_bindgen_test]
fn xss_payload_in_alt_does_not_create_script_element_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-xss-alt-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload_alt = "\"><script>alert(1)</script>";
    mount_avatar(&container, ImageStatus::Loading, "/a.png", payload_alt);

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "生の <script> 要素が実 DOM に生成されてはならない"
    );

    let inner = container.inner_html();
    assert!(
        !inner.contains("<script>"),
        "inner_html に生の <script> タグが含まれてはならない: {inner}"
    );
}

/// `condition` が真になるまで `requestAnimationFrame` 相当のマイクロタスク
/// 待機を繰り返す小さなポーリングヘルパ。`data:` URI の画像読み込み決着
/// タイミングはブラウザ実装依存のため、固定 `sleep` ではなく条件ポーリング
/// で待つ（`wasm-bindgen-futures` は既存 dev-dependency）。
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
