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
use fandhe_frontend_interactive::{dispatch, AppState, Hydrate, HYDRATE_ATTR_PREFIX};
use fandhe_frontend_wasm_full::headless_avatar::{apply_avatar_visibility, wire_avatar_events};
use fandhe_frontend_wasm_full::Runtime;
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

/// `container` 内の Avatar image 要素の `src` プロパティを DOM 直接操作で
/// 差し替える（`HtmlImageElement::set_src`、`render()`/SSR 経路を経由しない）。
///
/// `data:` スキームは `fandhe_frontend_core::is_safe_url`（REQ-1、
/// `crates/core/src/url.rs`）が SSR 出力からは意図的に拒否するため、
/// `mount_avatar` を経由した `render()` 出力には `data:` URI の `src` が
/// そもそも乗らない（属性ごと出力されない）。本ヘルパはクライアント側で
/// 動的に `img.src` を差し替える正規シナリオ（例: `createObjectURL`／署名
/// 付き URL の遅延差し込み）を模した、実ブラウザの `load`/`error` イベント
/// を実際に発火させるためのテスト専用の迂回であり、SSR エスケープ・URL
/// 検証を弱める変更ではない（本モジュールの受け入れ条件はあくまで
/// `wire_avatar_events`/`apply_avatar_visibility` が実 DOM 状態に対して
/// 正しく振る舞うことであり、`src` がどう設定されたかは無関係）。
fn set_image_src(container: &Element, src: &str) {
    let img = image_element(container)
        .dyn_into::<web_sys::HtmlImageElement>()
        .expect("image part must be an HtmlImageElement");
    img.set_src(src);
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

    // `mount_avatar` は `render()`（SSR 経路）を経由するため、`data:` URI は
    // `is_safe_url`（REQ-1）に拒否されて `src` 属性ごと出力されない
    // （`crates/core/src/url.rs` 参照）。まず安全な相対 URL で mount した上で、
    // `set_image_src` で `img.src` を直接（`render()` を経由せず）差し替えて
    // 実ブラウザの `load` イベントを発火させる。`wire_avatar_events` より
    // 前に `src` を差し替えることで、配線時点の settle 検査
    // （`avatar_action_for_settled_image`）がプレースホルダの空 `src`
    // （`complete()==true`／`natural_width()==0`）を拾って誤った合成
    // dispatch を行わないようにする。
    mount_avatar(
        &container,
        ImageStatus::Loading,
        "/placeholder.png",
        "avatar",
    );
    set_image_src(&container, ONE_PX_GIF);

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

    // (b) と同じ理由（`is_safe_url` が `data:` を SSR 出力から拒否する、
    // `crates/core/src/url.rs` 参照）で、安全なプレースホルダで mount した
    // 後に `img.src` を直接差し替えて実ブラウザの `error` イベントを
    // 発火させる。
    mount_avatar(
        &container,
        ImageStatus::Loading,
        "/placeholder.png",
        "avatar",
    );
    set_image_src(&container, INVALID_DATA_URI);

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
    // (b) と同じ理由（`is_safe_url` が `data:` を SSR 出力から拒否する、
    // `crates/core/src/url.rs` 参照）で、安全なプレースホルダで mount した
    // 後に `img.src` を直接差し替える。
    mount_avatar(
        &container,
        ImageStatus::Loading,
        "/placeholder.png",
        "avatar",
    );
    set_image_src(&container, ONE_PX_GIF);

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

// --- (g)〜(i) Runtime::mount/hydrate への統合（イシュー #711） -------------
//
// `wire_avatar_events`/`apply_avatar_visibility` を直接呼ぶ (a)〜(f) は
// #591 時点の配線層単体の契約を検証する。#711 は `Runtime::mount`/
// `Runtime::hydrate`（`crate::lib::Runtime::wire_avatar`）が本配線を
// アプリ側の手動呼び出しなしに自動で行うことが要求であるため、(g)〜(i) は
// `Runtime<C>` 経由でのみ検証し、`wire_avatar_events` を直接呼ばない。
//
// `fandhe_frontend_headless_ui::Avatar` 自体は「束縛点更新 + keyed list」
// （`fandhe_frontend_wasm_client::BindingSource`）・dirty tracking
// （`fandhe_frontend_interactive::DirtyTracked`）を使わない設計（本モジュール
// 冒頭 doc「`data-state` 語彙について」参照。属性反映は
// `apply_avatar_visibility` の直接 `set_attribute` に閉じる）であるため、
// `Runtime<C>`（`C: Component + DirtyTracked + BindingSource`）の型制約を
// 満たさない。これは `crates/headless-ui` 側の契約を変更する話ではなく
// （G2 は doc コメントのみに限定、実装計画 §2 参照）、`Runtime<C>` を使う
// **アプリ側**の `Component` 実装が両トレイトを備えていれば足りるという
// 設計上の分離である。本テストではその最小形として `TestAvatarHost`
// （`Avatar` へ `update`/`view`/`decode_action` を委譲し、
// `DirtyTracked`/`BindingSource` は「未使用」を表す空実装を返す）を用いる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestAvatarHost(Avatar);

impl fandhe_frontend_interactive::Component for TestAvatarHost {
    type Action = fandhe_frontend_headless_ui::avatar::AvatarAction;

    fn update(&mut self, action: Self::Action) {
        fandhe_frontend_interactive::Component::update(&mut self.0, action);
    }

    fn view(&self) -> fandhe_frontend_core::Node {
        fandhe_frontend_interactive::Component::view(&self.0)
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        Avatar::decode_action(name, payload)
    }
}

impl fandhe_frontend_interactive::DirtyTracked for TestAvatarHost {
    fn dirty_fields(&self) -> &[&'static str] {
        // `Runtime::wire` の束縛点更新（`data-bind-text`/`data-bind-attr`）を
        // 使わないため常に空（`apply_avatar_visibility` 経由の直接属性反映の
        // みで完結する、`Runtime::wire_avatar` 参照）。
        &[]
    }
}

impl fandhe_frontend_wasm_client::BindingSource for TestAvatarHost {
    fn bound_value(&self, _field: &str) -> Option<fandhe_frontend_wasm_client::BoundValue> {
        // 上記と同じ理由で束縛点を持たない。
        None
    }
}

/// `Runtime::mount` 経由で配線した Avatar の実 `load` イベントで
/// `component().status()` が `Loaded` になり、`data-state` も追随すること
/// （受け入れ条件、Runtime 標準経路）。
#[wasm_bindgen_test]
async fn runtime_mount_real_load_event_updates_image_status_to_loaded() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-runtime-mount-load-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    let runtime = Runtime::<TestAvatarHost>::mount(
        "avatar-runtime-mount-load-root",
        TestAvatarHost(Avatar::new(ImageStatus::Loading)),
    )
    .expect("Runtime::mount must not fail");

    // `Avatar::view` は `src`/`alt` を空文字列で描画する（`crates/headless-ui/
    // src/avatar.rs::Component for Avatar::view` 参照）。settle 検査が空
    // `src` を決着済みと誤判定しないよう、配線（`Runtime::mount` 内で完了
    // 済み）の後に `data:` URI を差し替えて実 `load` を発火させる。
    set_image_src(&container, ONE_PX_GIF);

    wait_for(|| runtime.component().0.status() == ImageStatus::Loaded).await;

    assert_eq!(runtime.component().0.status(), ImageStatus::Loaded);
    let img = image_element(&container);
    let fallback_el = fallback_element(&container);
    assert_eq!(img.get_attribute("data-state").as_deref(), Some("visible"));
    assert!(!img.has_attribute("hidden"));
    assert_eq!(
        fallback_el.get_attribute("data-state").as_deref(),
        Some("hidden")
    );
    assert!(fallback_el.has_attribute("hidden"));
}

/// `Runtime::mount` 経由で配線した Avatar の実 `error` イベントで
/// `component().status()` が `Error` になり、fallback が visible へ切り替わる
/// こと（受け入れ条件、Runtime 標準経路）。
#[wasm_bindgen_test]
async fn runtime_mount_real_error_event_updates_image_status_to_error() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-runtime-mount-error-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const INVALID_DATA_URI: &str = "data:image/gif;base64,not-a-valid-gif";

    let runtime = Runtime::<TestAvatarHost>::mount(
        "avatar-runtime-mount-error-root",
        TestAvatarHost(Avatar::new(ImageStatus::Loading)),
    )
    .expect("Runtime::mount must not fail");

    set_image_src(&container, INVALID_DATA_URI);

    wait_for(|| runtime.component().0.status() == ImageStatus::Error).await;

    assert_eq!(runtime.component().0.status(), ImageStatus::Error);
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

/// fail-closed 回帰: Avatar ではない `Component`（`AppState`）を
/// `Runtime::mount` しても、`root` 配下にたまたま Avatar と同じ
/// `data-scope`/`data-part` を持つ要素が紛れ込んだ場合でも、
/// `AppState::decode_action` が `"loaded"`/`"error"` を認識しない
/// （`fandhe_frontend_interactive::dispatch` が `false` を返す）ため状態が
/// 変化しないこと。`Runtime::wire_avatar`（`crate::lib` 参照）の
/// fail-closed 不変条件（Avatar 非搭載アプリへの副作用なし）を Runtime
/// 標準経路で固定する。
#[wasm_bindgen_test]
async fn runtime_mount_non_avatar_component_ignores_avatar_shaped_image_events() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-runtime-mount-non-avatar-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let runtime =
        Runtime::<AppState>::mount("avatar-runtime-mount-non-avatar-root", AppState::new())
            .expect("Runtime::mount must not fail");
    let before = runtime.component().clone();

    // AppState の描画には Avatar パーツは存在しないため、fail-closed 経路
    // （`collect_avatar_images` が空集合を返す settle 検査・
    // `avatar_action_for_image_event` の data-scope/data-part 完全一致
    // ガード）を実際に運動させるため、Avatar と同じ属性を持つ `img` 要素を
    // `root` 配下へ手動で追加する（改ざん・偶発混入シナリオの模擬）。
    let img = document
        .create_element("img")
        .expect("create_element must not fail for img");
    img.set_attribute("data-scope", "avatar")
        .expect("set_attribute must not fail");
    img.set_attribute("data-part", "image")
        .expect("set_attribute must not fail");
    container
        .append_child(&img)
        .expect("append_child must not fail");
    let img = img
        .dyn_into::<web_sys::HtmlImageElement>()
        .expect("img element must be an HtmlImageElement");

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    img.set_src(ONE_PX_GIF);
    wait_for(|| img.complete()).await;

    assert_eq!(
        *runtime.component(),
        before,
        "AppState は \"loaded\" アクションを認識しないため、Avatar 形状の \
         img への load イベントで状態が変化してはならない"
    );
}

// --- (j)〜(m) src 差し替えの MutationObserver 検知（イシュー #731） --------
//
// `MutationObserver` コールバックはマイクロタスクチェックポイントで実行され、
// `img` の `load`/`error` イベントはタスク（マクロタスク）で dispatch される
// （`docs/spec` の実装計画 §2 参照）。よって「src 差し替え → マイクロタスク
// 1 回 await → reset（Loading）を assert」はブラウザ仕様上レースなく成立し、
// 固定 `sleep` に頼らない（[`microtask_tick`] 参照）。

/// 別バイトの 1x1 透過 GIF（`ONE_PX_GIF` と区別するための 2 個目の
/// 有効な `data:` URI。同一 URI への再設定でも `MutationObserver` は
/// 属性変異を記録する仕様だが、テストの意図を明確にするため値を変える）。
const OTHER_PX_GIF: &str = "data:image/gif;base64,R0lGODlhAQABAIAAAAD/AP///ywAAAAAAQABAAACAUwAOw==";

/// Promise マイクロタスクを 1 回消化するまで待つ。`set_image_src` の
/// `setAttribute` 相当操作は同期的に `MutationRecord` をマイクロタスク
/// キューへ積むため（DOM 仕様）、本関数が作る `Promise::resolve` の
/// `then` コールバックはそれより後のマイクロタスクとして実行される。
/// よって本関数から復帰した時点で observer コールバックの実行は完了して
/// いることが保証される（固定 `sleep` に頼らない決定的な待機）。
async fn microtask_tick() {
    let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("microtask promise must resolve");
}

/// (j) `wire_avatar_events` 直接経路: 配線済み Avatar image の `src` を
/// 差し替えると、次の real `load` イベントより先に `"reset"` が記録される
/// こと（受け入れ条件 1）。
#[wasm_bindgen_test]
async fn wire_avatar_events_src_mutation_dispatches_reset_before_next_load() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-wire-src-mutation-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    // (b) と同じ理由で、安全なプレースホルダで mount した後に `img.src` を
    // 直接差し替えてから配線する（settle 検査がプレースホルダを拾わない
    // ようにするため）。
    mount_avatar(
        &container,
        ImageStatus::Loading,
        "/placeholder.png",
        "avatar",
    );
    set_image_src(&container, ONE_PX_GIF);

    let received = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let received_clone = received.clone();
    wire_avatar_events(container.clone(), move |action_ref| {
        received_clone.borrow_mut().push(action_ref.action);
    })
    .expect("wire_avatar_events must not fail");

    wait_for(|| !received.borrow().is_empty()).await;
    assert_eq!(
        received.borrow().last().map(String::as_str),
        Some("loaded"),
        "配線直後は最初の real load イベントで \"loaded\" が記録されるはず"
    );

    // src を差し替えて MutationObserver を発火させる。
    set_image_src(&container, OTHER_PX_GIF);
    microtask_tick().await;

    assert_eq!(
        received.borrow().get(1).map(String::as_str),
        Some("reset"),
        "src 差し替え直後、次の real load イベントより先に \"reset\" が記録されているはず"
    );

    // 差し替え後の画像も最終的に決着し、\"loaded\" が続くこと。
    wait_for(|| received.borrow().len() >= 3).await;
    assert_eq!(received.borrow().last().map(String::as_str), Some("loaded"));
}

/// (k) `Runtime::mount` 経路（受け入れ条件 1 の中核）: 一度 `Loaded` に
/// 決着した Avatar の `src` を再度差し替えると `ImageStatus::Loading` へ
/// reset され、`data-state` も追随すること。その後、差し替え後の画像も
/// 最終的に決着（Loaded/Error）へ回復すること。
#[wasm_bindgen_test]
async fn runtime_mount_src_mutation_resets_to_loading_then_settles() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-runtime-src-mutation-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    let runtime = Runtime::<TestAvatarHost>::mount(
        "avatar-runtime-src-mutation-root",
        TestAvatarHost(Avatar::new(ImageStatus::Loading)),
    )
    .expect("Runtime::mount must not fail");

    set_image_src(&container, ONE_PX_GIF);
    wait_for(|| runtime.component().0.status() == ImageStatus::Loaded).await;
    assert_eq!(runtime.component().0.status(), ImageStatus::Loaded);

    // 決着済みの画像の src を差し替える。
    set_image_src(&container, OTHER_PX_GIF);
    microtask_tick().await;

    assert_eq!(
        runtime.component().0.status(),
        ImageStatus::Loading,
        "src 差し替えで ImageStatus::Loading へ reset されるはず（イシュー #731）"
    );
    let img = image_element(&container);
    let fallback_el = fallback_element(&container);
    assert_eq!(img.get_attribute("data-state").as_deref(), Some("hidden"));
    assert!(img.has_attribute("hidden"));
    assert_eq!(
        fallback_el.get_attribute("data-state").as_deref(),
        Some("visible")
    );
    assert!(!fallback_el.has_attribute("hidden"));

    // 差し替え後の画像も最終的に決着すること（読み込み回復の確認）。
    wait_for(|| runtime.component().0.status() != ImageStatus::Loading).await;
    assert_eq!(runtime.component().0.status(), ImageStatus::Loaded);
}

/// (l) fail-closed 回帰（受け入れ条件 2）: Avatar ではない `Component`
/// （`AppState`）の root 配下に Avatar 形状の `img`（`data-scope="avatar"`
/// `data-part="image"`）が紛れ込んでいても、その `src` 差し替えで状態が
/// 変化しないこと（`AppState::decode_action` が `"reset"` を認識せず
/// `dispatch` が `false` を返すため no-op）。
#[wasm_bindgen_test]
async fn runtime_mount_non_avatar_component_ignores_src_mutation_on_avatar_shaped_image() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-runtime-non-avatar-src-mutation-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let runtime = Runtime::<AppState>::mount(
        "avatar-runtime-non-avatar-src-mutation-root",
        AppState::new(),
    )
    .expect("Runtime::mount must not fail");

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    let img = document
        .create_element("img")
        .expect("create_element must not fail for img");
    img.set_attribute("data-scope", "avatar")
        .expect("set_attribute must not fail");
    img.set_attribute("data-part", "image")
        .expect("set_attribute must not fail");
    container
        .append_child(&img)
        .expect("append_child must not fail");
    let img = img
        .dyn_into::<web_sys::HtmlImageElement>()
        .expect("img element must be an HtmlImageElement");
    img.set_src(ONE_PX_GIF);
    wait_for(|| img.complete()).await;

    let before = runtime.component().clone();

    img.set_src(OTHER_PX_GIF);
    microtask_tick().await;

    assert_eq!(
        *runtime.component(),
        before,
        "AppState は \"reset\" アクションを認識しないため、Avatar 形状の \
         img への src 差し替えで状態が変化してはならない"
    );
}

/// (m) 属性ガード回帰: `data-scope`/`data-part` を持たない素の `img`
/// （Avatar のマークアップではない）の `src` 差し替えでは `"reset"` が
/// dispatch されず、既に決着済みの Avatar 状態（`Loaded`）が変化しない
/// こと（`avatar_action_for_src_mutation` の scope/part 完全一致ガード）。
#[wasm_bindgen_test]
async fn runtime_mount_plain_img_without_avatar_attrs_does_not_trigger_reset() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "avatar-plain-img-src-mutation-root");
    let _cleanup = RemoveOnDrop(container.clone());

    const ONE_PX_GIF: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

    let runtime = Runtime::<TestAvatarHost>::mount(
        "avatar-plain-img-src-mutation-root",
        TestAvatarHost(Avatar::new(ImageStatus::Loading)),
    )
    .expect("Runtime::mount must not fail");

    set_image_src(&container, ONE_PX_GIF);
    wait_for(|| runtime.component().0.status() == ImageStatus::Loaded).await;
    assert_eq!(runtime.component().0.status(), ImageStatus::Loaded);

    // Avatar のマークアップとは無関係な素の img を root 配下へ追加する。
    let plain_img = document
        .create_element("img")
        .expect("create_element must not fail for img");
    container
        .append_child(&plain_img)
        .expect("append_child must not fail");
    let plain_img = plain_img
        .dyn_into::<web_sys::HtmlImageElement>()
        .expect("img element must be an HtmlImageElement");
    plain_img.set_src(OTHER_PX_GIF);
    microtask_tick().await;

    assert_eq!(
        runtime.component().0.status(),
        ImageStatus::Loaded,
        "data-scope/data-part を持たない素の img の src 差し替えは \
         Avatar の状態に影響してはならない"
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
