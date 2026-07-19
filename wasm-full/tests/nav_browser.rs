//! クライアント側ルーティングの実ブラウザ統合テスト（イシュー #374、
//! `wasm-pack test --headless --chrome`）。View Transitions 連携
//! （イシュー #404）の統合検証も本ファイルに含める。
//!
//! # 責務境界
//!
//! `wasm-full/tests/nav_native.rs` がルート解決・loader 接続の純粋層を
//! native で固定済み。本ファイルは配線層（`rws_wasm_full::nav::start_router`、
//! `#[cfg(target_arch = "wasm32")]`）を実 DOM 上で検証する
//! （`wasm-full/tests/three_mode_browser.rs` と同じ「SSR 相当を
//! `rws_app::page_shell` + `assemble_*_page` の直接合成で再現する」方式で
//! フィクスチャを組み立てる。`rws-server` への dev-dependency 追加はしない
//! ため）。
//!
//! # 検証内容（#374 実装計画 §6・#404 実装計画 §6 に対応）
//!
//! 1. クリック遷移: `a[data-nav]` への合成クリックで `pushState` + 描画
//! 2. 戻る/進む（popstate 契約）: 合成 `PopStateEvent` で再描画（`history.back()`
//!    は非同期でヘッドレス環境で不安定なため、決定的な合成イベントで
//!    ハンドラの契約を固定する）
//! 3. 直接アクセス・リロード相当: `start_router` 呼び出し自体は DOM を
//!    変更しない（初期表示で loader を再実行しない凍結事項の直接証明）
//! 4. XSS 回帰: クライアント遷移後もペイロードがテキストのまま
//! 5. 非インターセプト: `data-nav` なし・Ctrl+クリック・未登録パスでは
//!    `prevent_default` されない
//! 6. 連続遷移: 一覧 → 詳細 → 一覧の往復後もクリック配線が生きている
//!    （`document` レベル委譲のため `root` のサブツリー差し替え後も
//!    リスナーが失われないことの直接証明）
//! 7. （#404）View Transitions スタブ検証: nav 遷移 1 回につき
//!    `document.startViewTransition` が 1 回だけ呼ばれ、非同期の update
//!    コールバック実行で DOM/タイトルが確定すること
//! 8. （#404）非対応ブラウザ相当（`startViewTransition` を非関数値で
//!    shadow）では同期的に DOM が差し替わること（graceful degradation）
//! 9. （#404）連続遷移が loader 解決との競合なく最後のルートへ収束し、
//!    SSR 相当出力とバイト一致すること
//! 11. 遷移後の `data-hydrate` 再配線（イシュー #403）: 遷移 → いいねボタン
//!     操作 → `class="liked"` 付与・トグル、往復遷移後の再配線、初期表示
//!     ページ非配線の契約、XSS 複合ケース
//!
//! # 不変条件: ネイティブ `startViewTransition` を必ずスタブする
//!
//! **遷移を発生させる（`with_view_transition` を経由する）テストは、
//! ネイティブ `document.startViewTransition` を必ずスタブする**
//! （[`ViewTransitionStub`] または検証 8 専用の
//! [`NonFunctionViewTransitionShadow`] のいずれか。遷移に到達しないテスト
//! （検証 3・5 等、`resolve_path` が `None` を返す・`start_router` 呼び出し
//! のみで完結する等）は対象外であり、各テストの doc comment に非対象である
//! 理由を明記する）。self-hosted CI の
//! headless Chrome はネイティブ実装を持つため、ネイティブ経路を素通しする
//! テストは `wasm-bindgen-test-runner` の完了検出に失敗し、ジョブが
//! `timeout-minutes` に達するまでハングする事象が実証された
//! （CI run 29695628266 / job 88215777129、`navigating_to_xss_payload_item_
//! keeps_payload_as_text_not_element` で発生）。以前は検証 4 のみがこの理由で
//! スタブ化されていたが、検証 1 等が同じネイティブ経路を素通しし続けて
//! いたためハングリスクが残っていた（レビュー指摘・#404 フォローアップ）。
//! ネイティブ実装の実ブラウザ挙動検証（旧検証 10）は決定性を欠くため本ファイル
//! からは削除し、実ブラウザでの手動確認事項とする。
//!
//! # 非同期化の方針（#404）
//!
//! `document.startViewTransition()` の update コールバックは実ブラウザでは
//! 非同期実行されうるため（[`ViewTransitionStub`] もこれを模してマイクロ
//! タスクで非同期実行する）、遷移後の DOM/タイトル断定はすべて
//! [`wait_until`]（`requestAnimationFrame` ベースのポーリング）を介して行う。
//! 同期実行環境（検証 8 の非対応ブラウザ相当フォールバック経路）では 1 回目の
//! チェックで即座に条件が満たされるため、待機コストは実質ゼロで従来どおりの
//! 決定的な検証を維持する。
//!
//! # CI ハング再発の根本原因と修正（PR #420 フォローアップ）
//!
//! `wasm-bindgen-test` はテストを登録順とは逆順（`Vec::pop` による LIFO）で
//! 実行するため、本ファイルでは実行順が宣言順とは一致しない
//! （`v` 始まり → `s` → `p` → `n` → `l` → `c` 始まりの逆アルファベット順で
//! 実行される、CI run 29696865573 実測）。この実行順で
//! [`non_matching_clicks_are_not_intercepted`] の直後に実行されるのが
//! [`navigating_to_xss_payload_item_keeps_payload_as_text_not_element`]
//! であり、後者が原因不明のままハングし chromedriver が `SIGKILL` される
//! 事象が確認された（CI run 29695628266・29696865573）。
//!
//! 根本原因は [`non_matching_clicks_are_not_intercepted`] の Ctrl+クリック
//! ケースにあった: 検証対象（nav モジュールが `prevent_default` を呼ばない
//! こと）を確定させた後も、実 `href="/items/1"` を持つ実 DOM `<a>` 要素への
//! 合成クリックのブラウザ既定動作（新規タブで開く等）を止めていなかった。
//! `dispatchEvent` は `isTrusted: false` だが、`<a href>` の既定動作は
//! `preventDefault` されない限り実行され得るため、headless Chrome の
//! テスト実行ページ自身が新規タブ生成・フォーカス遷移に巻き込まれうる。
//! 巻き込まれると `document.visibilityState` がバックグラウンド化し、
//! 以後のテストが依存する `requestAnimationFrame`（バックグラウンドタブでは
//! 抑制されうる）ベースの [`wait_until`] が無期限に停止し得る。これは
//! 「実行順で直後に来る、最初に `wait_until` を使うテスト」で症状が
//! 現れることと整合する（[`non_function_shadow_falls_back_to_synchronous_
//! render`] は同期のみのため症状が出ず、その次の
//! [`navigating_to_xss_payload_item_keeps_payload_as_text_not_element`]
//! で初めて `wait_until` の `await` が観測される）。
//!
//! 修正は 2 点:
//!
//! 1. [`non_matching_clicks_are_not_intercepted`] で、検証用アサーション
//!    確定後に `ctrl_event.prevent_default()` をテスト側から明示的に呼び、
//!    実ブラウザ既定動作（新規タブ生成等）そのものを発生させない
//!    （「nav モジュールが `prevent_default` を呼ばないこと」の検証は
//!    アサーションが既定動作の前に完了しているため弱まらない）。
//! 2. [`next_animation_frame`]（[`wait_until`] の内部実装）に壁時計ベースの
//!    `setTimeout` フォールバック（[`FRAME_FALLBACK_TIMEOUT_MS`]）を追加する。
//!    上記 1. で根本原因を解消した後も、rAF が発火しない未知の環境要因が
//!    残る場合に「原因不明の無限ハング」ではなく「診断可能な `assert!`
//!    失敗」へ確実に変換するための多層防御。

#![cfg(target_arch = "wasm32")]

use js_sys::{Function, Reflect};
use rws_app::{assemble_list_page, demo_items, DemoItemsLoader};
use rws_wasm_full::nav::encode_scroll_state;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, MouseEvent, MouseEventInit, PopStateEvent, PopStateEventInit,
    ScrollRestoration,
};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダを document body へ生成する。`page_shell` の
/// 実際の DOM 構造（`<body>` 直下に `<div id="app-root">` を単独で置く）を
/// 再現するため、`initial_html`（`rws_app::layout` の render 出力＝
/// `<div id="app-root">...</div>` そのもの）を**一意な test スコープ id**
/// を持つ外側コンテナへ挿入する（`page_shell` の `<body>` 相当）。返り値は
/// 実際に `id="app-root"` を持つ内側要素（`start_router("app-root")` が
/// `get_element_by_id` で解決する対象）であり、`container.set_id("app-root")`
/// のような二重付与は行わない（`id` はドキュメント全体で一意という前提を
/// 崩さない。テスト間の干渉は `RemoveOnDrop` の確実な除去で防ぐ）。
///
/// 戻り値は `(container, root)`。`container`（一意な test id を持つ外側
/// 要素、`RemoveOnDrop` の対象）を除去すればサブツリーごと消える。`root`
/// はその内側の `#app-root` 要素で、`start_router` の `root_id` 解決対象。
fn create_app_root(
    document: &Document,
    container_test_id: &str,
    initial_html: &str,
) -> (Element, Element) {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(container_test_id);
    container.set_inner_html(initial_html);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    let root = container
        .query_selector("#app-root")
        .expect("query_selector must not fail")
        .expect("initial_html must contain a single #app-root element (rws_app::layout output)");
    (container, root)
}

/// スクロール復元テスト用に、`container`（`create_app_root` が返す外側
/// コンテナ）へヘッドレスビューポートより十分に高い（3000px）スペーサーを
/// 追加する（イシュー #406）。テストは `RemoveOnDrop` により `container`
/// ごと除去されるため、後続テストの `window.scroll_y()` に影響を残さない。
fn append_tall_spacer(document: &Document, container: &Element) {
    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height: 3000px;")
        .expect("set_attribute must not fail");
    container
        .append_child(&spacer)
        .expect("append_child must not fail");
}

/// スクロール復元テストの前提を揃える（他テストとの `window.scroll_y()`
/// 干渉を避けるため、各テスト冒頭で先頭へ揃える）。
fn reset_scroll_to_top() {
    web_sys::window()
        .expect("window must exist")
        .scroll_to_with_x_and_y(0.0, 0.0);
}

/// SSR 相当の詳細ページ（`item_id`）を独立したプレースホルダへ実 DOM 展開し、
/// `#app-root` のシリアライズ結果を返す（`three_mode_browser.rs::
/// paint_and_extract_app_root` と同じ「双方を同一のパース・シリアライズ
/// 経路に通す」方式。受け入れ条件 4「三モード整合」を、クライアント遷移後の
/// DOM とのバイト比較として直接固定する）。
fn ssr_equivalent_detail_outer_html(document: &Document, item_id: &str) -> String {
    let ssr_body =
        rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &item_id.to_string())
            .expect("infallible loader");
    let placeholder = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    placeholder.set_inner_html(&rws_core::render(&ssr_body));
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&placeholder)
        .expect("append_child must not fail for a detached div");
    let outer_html = placeholder
        .query_selector("#app-root")
        .expect("query_selector must not fail")
        .expect("ssr_body must contain a single #app-root element")
        .outer_html();
    placeholder.remove();
    outer_html
}

struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// SSR 相当の一覧ページ本文（`<div id="app-root">` の innerHTML 相当）を
/// 組み立てる（`three_mode_browser.rs` と同じ「rws_app 直接合成」方式）。
fn ssr_equivalent_list_inner_html() -> String {
    let body = assemble_list_page(&DemoItemsLoader, &()).expect("infallible loader");
    // `page_shell` は `<div id="app-root">` を含む完全文書を返すため、
    // その内側（layout() が組み立てる h1/main の子ノード）のみを
    // 抽出する。`layout()` 自体は非公開のため `page_shell` の出力から
    // `render` した内容を直接使う代わりに、`body` ノードを直接 render する
    // （`body` は `layout()` の戻り値そのもの、`assemble_list_page` の
    // 内部で `list_page` が呼ばれた結果）。
    rws_core::render(&body)
}

/// history state（URL）を `path` へ揃える（テスト前提条件のセットアップ、
/// 実際のナビゲーション経路とは別に `replace_state` で直接書き換える）。
fn set_location_path(path: &str) {
    let window = web_sys::window().expect("window must exist");
    let history = window.history().expect("history must exist");
    history
        .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
        .expect("replace_state must not fail in test environment");
}

/// `bubbles: true` の合成 `MouseEvent`（click）を組み立てる（`events.rs`
/// の `xss_escape_wasm.rs` と同じ方式。リスナーは `document` へ登録される
/// ため、`target` から `document` までバブリングする前提）。
fn synthetic_click_event() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_button(0);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// Ctrl+クリック相当の合成 `MouseEvent`（検証 5、非インターセプト対象）。
fn synthetic_ctrl_click_event() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_button(0);
    init.set_ctrl_key(true);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// 合成 `PopStateEvent` を組み立てる（検証 2、`history.back()` の非同期性を
/// 避けて決定的にハンドラを起動する）。
fn synthetic_popstate_event() -> PopStateEvent {
    let init = PopStateEventInit::new();
    PopStateEvent::new_with_event_init_dict("popstate", &init)
        .expect("PopStateEvent construction must not fail")
}

/// `document` を `Reflect` API へ渡せる `JsValue` へ変換する
/// （`web_sys::Document` は `wasm_bindgen` の newtype ラッパーであり、
/// `Reflect::set`/`Reflect::get` はいずれも `&JsValue` を要求するため、
/// テスト内のスタブ設置・取得処理で共通に使う）。
fn document_as_value(document: &Document) -> JsValue {
    JsValue::from(document.clone())
}

/// `document` を `Reflect::delete_property` 用の `&js_sys::Object` へ変換する
/// （`delete_property` のみ `&Object<T>` を要求するため、`Reflect::set`/
/// `Reflect::get` の `document_as_value` とは異なるヘルパーとして分離する）。
fn document_as_object(document: &Document) -> js_sys::Object {
    document.clone().unchecked_into::<js_sys::Object>()
}

/// 1 回分の待機ステップを `requestAnimationFrame` または
/// [`FRAME_FALLBACK_TIMEOUT_MS`] のいずれか早い方まで `Promise` 化して
/// await する（[`wait_until`] の内部実装）。
///
/// # fail-fast 化の経緯（CI ハング再発、PR #420 レビュー指摘）
///
/// 従来は `requestAnimationFrame` のみを待っていたため、headless Chrome の
/// 実行環境で rAF コールバックが発火しない（描画不可視・compositor 停止等）
/// 場合、この `await` が無期限に返らず `wait_until` の `max_frames` による
/// 回数上限が実質無意味化する（ループ回数は有限でも、各回の `await` 自体が
/// 無限に停止し得るため）。これにより「テスト失敗」ではなく
/// `wasm-bindgen-test-runner` 側の完了検出タイムアウト・chromedriver の
/// 強制終了（`SIGKILL`）に至るまで診断不能なハングとなっていた
/// （CI run 29695628266 等、`docs/ci/ci-runner-requirements.md` §6）。
///
/// 本関数は `requestAnimationFrame` と `setTimeout`（[`FRAME_FALLBACK_TIMEOUT_MS`]
/// ミリ秒後）の双方を仕掛け、`js_sys::Promise` の `resolve` を **どちらが
/// 先に呼んでも構わない**形で共有する（2 回目の呼び出しは仕様上 no-op）。
/// これにより 1 ステップあたりの待機時間が壁時計時間で必ず上限を持ち、
/// rAF が発火しない環境でも `wait_until` の総待機時間が
/// `max_frames * FRAME_FALLBACK_TIMEOUT_MS` を超えて停止することはない
/// （無限ハングを、原因が診断可能な決定的な `assert!` 失敗へ変換する）。
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

/// [`next_animation_frame`] の壁時計フォールバック締切（ミリ秒）。
/// 本ファイル中で最大の `max_frames`（[`consecutive_navigations_with_stub_
/// converge_to_last_route_and_match_ssr`] の 120）との組み合わせでも、
/// 1 回の `wait_until` 呼び出し全体の最大待機時間は高々 `120 * 100ms = 12` 秒
/// （`wasm-bindgen-test` の既定タイムアウト `WASM_BINDGEN_TEST_TIMEOUT`
/// （20 秒）に対し十分な余裕を残す値。rAF が正常発火する通常経路では毎回
/// rAF が先に解決するため実待機コストへの影響はない）に収まる。
const FRAME_FALLBACK_TIMEOUT_MS: i32 = 100;

/// `condition` が `true` になるまで最大 `max_frames` フレーム（実際には
/// [`next_animation_frame`] の壁時計フォールバック込みで有限時間）待つ
/// （`nav.rs` の `with_view_transition` の update コールバックが実ブラウザ
/// では非同期実行されうるため、遷移後の DOM/タイトル断定は本ヘルパーを介して
/// 行う。同期実行環境（フォールバック経路）では 1 回目のチェックで即座に
/// `true` となるため待機コストは実質ゼロ。上限に達しても条件を満たさない
/// 場合は最後の判定結果をそのまま返し、呼び出し側の `assert!` でテスト失敗
/// として明示させる）。
async fn wait_until<F: Fn() -> bool>(condition: F, max_frames: u32) -> bool {
    for _ in 0..max_frames {
        if condition() {
            return true;
        }
        next_animation_frame().await;
    }
    condition()
}

/// `document.startViewTransition` を記録用スタブへインスタンスプロパティ
/// として差し替える（イシュー #404、判断 D「スタブによる決定的検証」）。
///
/// - `calls()`: スタブが呼ばれた回数（`with_view_transition` が
///   `document.startViewTransition` を関数として検出し呼び出した回数と一致）
/// - update コールバックは `wasm_bindgen_futures::spawn_local` によるマイクロ
///   タスクで**非同期**実行する（実ブラウザの非同期性を模す。同期実行では
///   ないことを構造的に保証し、`wait_until` を介した断定が必須であることを
///   テスト自身が体現する）
/// - `Drop` でプロパティを削除し、他テストへ影響を残さない（`RemoveOnDrop`
///   と同じ「テスト間の状態非共有」方針）
struct ViewTransitionStub {
    call_count: Rc<Cell<u32>>,
    // `Closure` は JS 側へ渡した関数の生存期間を保持するためだけに保持する
    // （`as_ref().unchecked_ref()` で参照を渡した後も、この構造体が生きている
    // 間はクロージャの実体が解放されない）。
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

/// `document.startViewTransition` を非関数値で shadow する（検証 8、
/// 非対応ブラウザ相当の機能検出フォールバック直接証明）。`Drop` で復元する。
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

/// 検証 1・6: `/` → `/items/1` → `/` の連続クリック遷移で URL・DOM・
/// `document.title` が追従し、往復後もクリック配線が生きていること。
///
/// `startViewTransition` は [`ViewTransitionStub`] で決定的にスタブする
/// （ファイル冒頭の不変条件参照。以前はスタブせずネイティブ経路を素通しして
/// いたが、headless Chrome での完了検出ハングの原因となったため #404
/// フォローアップでスタブへ切り替えた）。[`wait_until`] はスタブの非同期
/// update コールバックとフォールバック経路の同期実行の両方を同一コードで
/// 許容する。
#[wasm_bindgen_test]
async fn click_navigation_updates_url_dom_and_title_across_round_trip() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-round-trip",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // 1 回目: 一覧 → 詳細（/items/1）。
    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    // `pushState` は apply 段より前（同期）で実行されるため URL は即座に反映
    // される（イシュー #404 実装計画・判断 B）。
    assert_eq!(window.location().pathname().unwrap(), "/items/1");
    // 真因修正（CI 再発、PR #420 フォローアップ）: `wait_until` の待機条件は
    // `document.title()` のような document 全体で共有され、テスト間で
    // リセットされないグローバル状態を単独の判定基準にしない。`document.title`
    // は直前に実行された別テストの最終状態を引き継ぐため、たまたま
    // 遷移前から目的の文字列と一致していると `wait_until` が「本テスト自身の
    // apply」が実際に走るより前の 1 回目のチェックで早期に `true` を返して
    // しまい、直後の DOM 断定が「まだ差し替わっていない」状態を検出して
    // 失敗する（apply 自体は非同期のまま宙に浮き、後続テストの
    // `#app-root` を後から巻き込んで壊す事象にもつながる）。判定は本テスト
    // 自身にスコープされた `root`（`create_app_root` が返す本テスト専用の
    // 要素）の DOM 内容を主条件とし、`document.title` の検証は DOM 確定後の
    // 同期断定へ切り出す。
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "遷移後、詳細 DOM が確定すること"
    );
    assert_eq!(
        document.title(),
        "記事詳細",
        "遷移後に document.title が「記事詳細」へ確定すること"
    );
    // 受け入れ条件 4（三モード整合）: クライアント遷移後の `#app-root` の
    // 実 DOM シリアライズが SSR 相当出力とバイト一致すること
    // （`three_mode_browser.rs` と同じ「独立プレースホルダへの実 DOM 展開 +
    // シリアライズ比較」方式。`nav::resolve_route_view_with` が `csr::
    // resolve_detail_node` を呼ぶ経路自体は `three_mode_browser.rs` が
    // 別途 SSR ≡ CSR で固定済みだが、本テストは遷移機構込みの実 DOM 結果を
    // 直接比較する）。
    assert_eq!(
        root.outer_html(),
        ssr_equivalent_detail_outer_html(&document, "1"),
        "クライアント遷移後の #app-root は SSR 相当出力とバイト一致すること"
    );

    // 2 回目: 詳細 → 一覧（往復、委譲リスナーが差し替え後も生きていることの
    // 直接証明。`document` レベル登録のため `root` のサブツリー差し替えの
    // 影響を受けない）。
    let back_link = document
        .query_selector("a[data-nav=\"/\"]")
        .expect("query_selector must not fail")
        .expect("detail page must contain a data-nav link back to /");
    back_link
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert_eq!(window.location().pathname().unwrap(), "/");
    // 上記 1 回目と同じ理由（`document.title` を単独の待機条件にしない）で
    // `root` スコープの DOM を主条件とする。
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-list\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "遷移後、一覧 DOM が確定すること"
    );
    assert_eq!(
        document.title(),
        "記事一覧",
        "遷移後に document.title が「記事一覧」へ確定すること"
    );

    // 3 回目: 一覧 → 詳細（2 往復目）。委譲リスナーが継続して機能すること。
    let link_again = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page (round 2) must contain a data-nav link to /items/1");
    link_again
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    assert_eq!(window.location().pathname().unwrap(), "/items/1");
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "2 往復目の遷移後も詳細 DOM が確定すること"
    );
}

/// 検証 2: `history.replace_state` で `/` へ揃えたのち合成 `popstate` を
/// dispatch すると、一覧 DOM・タイトルへ復帰する（`history.back()` の
/// 非同期性を避けた決定的な契約固定）。`startViewTransition` は
/// [`ViewTransitionStub`] でスタブする（ファイル冒頭の不変条件参照）。
#[wasm_bindgen_test]
async fn popstate_event_re_resolves_and_renders_without_pushing_history() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    // 開始状態は詳細ページ（/items/1）とする。
    let body = rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &"1".to_string())
        .expect("infallible loader");
    set_location_path("/items/1");
    let (container, root) =
        create_app_root(&document, "nav-test-popstate", &rws_core::render(&body));
    let _cleanup = RemoveOnDrop(container);
    document.set_title("記事詳細");
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // ブラウザの「戻る」操作を模した状態遷移: URL を `/` へ書き換えてから
    // popstate を発火する（実際の `back()` は非同期でヘッドレス環境が
    // 不安定なため、決定的な合成イベントでハンドラ契約を固定する）。
    set_location_path("/");
    window
        .dispatch_event(&synthetic_popstate_event())
        .expect("dispatch_event must not fail");

    // 真因修正（CI 再発、PR #420 フォローアップ）: 待機条件は `document.title`
    // のようなテスト間でリセットされないグローバル状態ではなく、本テスト
    // 専用の `root` の DOM 内容を主条件とする
    // （`click_navigation_updates_url_dom_and_title_across_round_trip` と
    // 同じ理由。詳細は同テストのコメント参照）。
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-list\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "popstate 後、一覧 DOM が確定すること"
    );
    assert_eq!(
        document.title(),
        "記事一覧",
        "popstate 後に document.title が「記事一覧」へ確定すること"
    );
}

/// 検証 3: SSR 済み DOM の上で `start_router` を呼んでも DOM は不変
/// （初期表示で loader を再実行・再描画しない凍結事項の直接証明）。
/// View Transitions 連携は遷移（クリック/popstate）発生時のみ関与するため
/// 本テストは同期のままでよい。
#[wasm_bindgen_test]
fn start_router_does_not_repaint_on_initial_call() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let body = rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &"1".to_string())
        .expect("infallible loader");
    set_location_path("/items/1");
    let (container, root) =
        create_app_root(&document, "nav-test-no-repaint", &rws_core::render(&body));
    let _cleanup = RemoveOnDrop(container);
    // `root`（実際の `#app-root` 要素）の innerHTML をここで固定する。
    // `create_app_root` の `initial_html` は外側コンテナへ渡した文字列
    // （`<div id="app-root">...</div>` を含む）であり `root.inner_html()`
    // （`#app-root` タグ自身を含まない中身のみ）とは形が異なるため、比較は
    // 実際の DOM から読み取った値同士で行う。
    let inner_html_before_start_router = root.inner_html();

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    assert_eq!(
        root.inner_html(),
        inner_html_before_start_router,
        "start_router 呼び出し直後は SSR 済み DOM を一切変更しないこと"
    );
}

/// 検証 4: XSS ペイロード item（id="2"）へのクライアント遷移後もペイロードが
/// 実 DOM 上でテキストのまま（要素化されない）こと。View Transitions 経由
/// （非同期になりうる）でも描画内容自体はエスケープ済みのまま変化しない
/// ことを固定する（async 版、イシュー #404）。
///
/// 本テストの目的はエスケープ保証（既定エスケープの非弱体化、REQ-1）の
/// 回帰検証であり、`document.startViewTransition` 自体の実ブラウザ挙動
/// 検証は検証 7・9・10 が別途担う。そのため [`ViewTransitionStub`]
/// （検証 7 と同じ決定的スタブ）で `startViewTransition` を差し替える
/// （CI 実測: headless Chrome 実行環境でネイティブ実装を素通しした場合、
/// 本テストが `wasm-bindgen-test-runner` のテスト完了検出に失敗して
/// ジョブ全体が `timeout-minutes` に達するまでハングする事象が発生した
/// ため、イシュー #404 フォローアップとしてスタブへ切り替え、決定性を
/// 優先する）。
#[wasm_bindgen_test]
async fn navigating_to_xss_payload_item_keeps_payload_as_text_not_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-xss-payload",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let xss_item_id = demo_items()
        .into_iter()
        .find(|it| it.title.contains("<script>"))
        .map(|it| it.id)
        .expect("demo_items() must contain the XSS payload fixture item");
    let selector = format!("a[data-nav=\"/items/{xss_item_id}\"]");
    let link = document
        .query_selector(&selector)
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to the XSS payload item");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "XSS ペイロード item への遷移後、詳細 DOM が確定すること"
    );
    assert!(
        root.query_selector("script").unwrap().is_none(),
        "XSS ペイロードが実 DOM 上で <script> 要素として生成されてはならない"
    );
    assert!(
        root.inner_html()
            .contains("&lt;script&gt;alert('xss')&lt;/script&gt;"),
        "XSS ペイロードはエスケープ済みテキストとして DOM に保持されること: {}",
        root.inner_html()
    );
}

/// 検証 5: `data-nav` を持たない要素のクリック・Ctrl+クリック・未登録パス
/// への `data-nav` ではいずれも `prevent_default` されない
/// （ブラウザ既定動作を壊さない安全側フォールバック）。いずれのケースも
/// `resolve_path` が `None` を返す時点で `render_route`（View Transitions
/// 連携の起点）自体に到達しないため、本テストは同期のままでよい。
#[wasm_bindgen_test]
fn non_matching_clicks_are_not_intercepted() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-non-intercepted",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // data-nav なし要素。
    let plain_span = document
        .create_element("span")
        .expect("create_element must not fail");
    plain_span.set_text_content(Some("plain"));
    root.append_child(&plain_span)
        .expect("append_child must not fail");
    let event = synthetic_click_event();
    plain_span
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
    assert!(
        !event.default_prevented(),
        "data-nav を持たない要素のクリックは prevent_default されないこと"
    );

    // Ctrl+クリック（新規タブで開く操作の既定動作を壊さない）。
    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    let ctrl_event = synthetic_ctrl_click_event();
    link.dispatch_event(&ctrl_event)
        .expect("dispatch_event must not fail");
    assert!(
        !ctrl_event.default_prevented(),
        "Ctrl+クリックは prevent_default されないこと（新規タブで開く操作を維持）"
    );
    // 検証対象（nav モジュールの click ハンドラが `prevent_default` を
    // 呼ばなかったこと）は上記アサーションで既に確定している。ここから先は
    // `link`（実 `href="/items/1"` を持つ実 DOM 要素）へのクリックのブラウザ
    // 既定動作そのものを止める（イシュー #404 フォローアップ、CI ハング
    // 再発調査）。`dispatch_event` は `isTrusted: false` の合成イベントだが、
    // `<a href>` の既定動作（新規タブで開く/遷移）は cancelable な click
    // イベントが `preventDefault` されない限り実行され得る（Web 標準の
    // activation behavior は `isTrusted` を前提としない）。テスト実行用
    // ページ自身（≒
    // `wasm-bindgen-test-runner` のハーネスページ）が新規タブ生成やタブの
    // フォーカス遷移に巻き込まれると、以後のテストが依存する
    // `requestAnimationFrame` ベースの `wait_until`（バックグラウンドタブでは
    // rAF が抑制されうる）が無期限に停止しかねない（CI run 29695628266・
    // 29696865573 で `navigating_to_xss_payload_item_keeps_payload_as_text_
    // not_element`（本テストの直後に実行される、`wasm-bindgen-test` の
    // `Vec::pop` による LIFO 実行順）が原因不明のままハングし chromedriver が
    // `SIGKILL` された事象と符合）。既定動作を止めても「nav モジュールが
    // `prevent_default` を呼ばないこと」の検証には影響しない（アサーション
    // 自体は既定動作が走る前に完了している）ため、既存の検証意図を弱めない。
    ctrl_event.prevent_default();
    assert_eq!(
        window.location().pathname().unwrap(),
        "/",
        "Ctrl+クリックでは URL が変化しないこと"
    );

    // 未登録パスへの data-nav（ルート表に一致しない安全な相対パス）。
    let unregistered_link = document
        .create_element("a")
        .expect("create_element must not fail");
    unregistered_link
        .set_attribute("data-nav", "/no-such-route")
        .expect("set_attribute must not fail");
    root.append_child(&unregistered_link)
        .expect("append_child must not fail");
    let unregistered_event = synthetic_click_event();
    unregistered_link
        .dispatch_event(&unregistered_event)
        .expect("dispatch_event must not fail");
    assert!(
        !unregistered_event.default_prevented(),
        "ルート表に一致しない data-nav 値は prevent_default されないこと"
    );
    assert_eq!(
        window.location().pathname().unwrap(),
        "/",
        "未登録パスへのクリックでは URL が変化しないこと"
    );
}

/// 検証 7（イシュー #404）: `document.startViewTransition` をスタブすると、
/// nav 遷移 1 回につき 1 回だけ呼ばれ、非同期の update コールバック実行で
/// DOM/タイトルが確定すること。
#[wasm_bindgen_test]
async fn view_transition_stub_is_called_once_and_dom_updates_after_async_callback() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-vt-stub-single-call",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    // `with_view_transition` は update コールバックを渡す前提として
    // `document.startViewTransition` を同期的に 1 回呼ぶ（呼び出し自体は
    // 遷移の成否・update の実行タイミングに関わらず同期）。
    assert_eq!(
        stub.calls(),
        1,
        "nav 遷移 1 回につき startViewTransition が 1 回だけ呼ばれること"
    );
    // スタブの update コールバックはマイクロタスクで非同期実行するため、
    // dispatch_event 直後の時点では DOM は未確定でありうる
    // （実装が真に非同期実行を経由していることの間接証明）。
    //
    // 真因修正（CI 再発、PR #420 フォローアップ）: 待機条件は `document.title`
    // のようなテスト間でリセットされないグローバル状態ではなく、本テスト
    // 専用の `root` の DOM 内容を主条件とする（`click_navigation_updates_
    // url_dom_and_title_across_round_trip` と同じ理由。詳細は同テストの
    // コメント参照）。
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "非同期 update コールバック実行後に詳細 DOM が確定すること"
    );
    assert_eq!(
        document.title(),
        "記事詳細",
        "非同期 update コールバック実行後に document.title が確定すること"
    );
}

/// 検証 8（イシュー #404）: `document.startViewTransition` を非関数値で
/// shadow した場合、機能検出により同期的に DOM が差し替わること
/// （graceful degradation の直接証明）。
#[wasm_bindgen_test]
fn non_function_shadow_falls_back_to_synchronous_render() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-vt-fallback",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _shadow = NonFunctionViewTransitionShadow::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    // `wait_until` を使わず、dispatch_event 直後の同期的な状態を直接断定する
    // （非対応ブラウザ相当の経路は `apply` を即座に実行するはず、という
    // 契約そのものを検証する）。
    assert_eq!(
        document.title(),
        "記事詳細",
        "非関数値で shadow された場合、遷移は同期的に完了すること"
    );
    assert!(root
        .query_selector("[data-testid=\"item-detail\"]")
        .unwrap()
        .is_some());
}

/// 検証 9（イシュー #404）: `document.startViewTransition` をスタブした状態
/// での連続遷移（詳細 → 一覧 → 詳細、各 update コールバックの解決を待たず
/// 連続クリック）が、loader 解決との競合なく最後のルートへ収束し、最終 DOM
/// が SSR 相当出力とバイト一致すること。
///
/// クリック対象は `root`（`render_route` により都度サブツリー差し替えされる
/// 範囲）の**外側**、`container` 直下に固定で用意した 2 本のリンクとする
/// （`root` の子要素から `data-nav` リンクを探す方式だと、直前のクリックの
/// apply 未完了時点で DOM がどちらの状態かに依存してテストが不安定になる
/// ため、クリック対象自体を DOM 差し替えの影響を受けない位置に固定する）。
#[wasm_bindgen_test]
async fn consecutive_navigations_with_stub_converge_to_last_route_and_match_ssr() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-vt-consecutive",
        &ssr_equivalent_list_inner_html(),
    );
    let container_for_links = container.clone();
    let _cleanup = RemoveOnDrop(container);
    let stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let to_detail = document
        .create_element("a")
        .expect("create_element must not fail");
    to_detail
        .set_attribute("data-nav", "/items/1")
        .expect("set_attribute must not fail");
    container_for_links
        .append_child(&to_detail)
        .expect("append_child must not fail");

    let to_list = document
        .create_element("a")
        .expect("create_element must not fail");
    to_list
        .set_attribute("data-nav", "/")
        .expect("set_attribute must not fail");
    container_for_links
        .append_child(&to_list)
        .expect("append_child must not fail");

    // 詳細 → 一覧 → 詳細。各クリックは前段の update コールバック解決を待たず
    // 連続で発火する（`startViewTransition` の標準挙動では先行遷移が
    // スキップされうるが、全 update コールバックは順序どおり実行される仕様の
    // ため、最終状態は最後にクリックしたルート＝詳細へ収束するはず）。
    to_detail
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    to_list
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    to_detail
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            120
        )
        .await,
        "連続遷移後、最終的に詳細 DOM へ収束すること"
    );
    assert_eq!(window.location().pathname().unwrap(), "/items/1");
    assert_eq!(
        root.outer_html(),
        ssr_equivalent_detail_outer_html(&document, "1"),
        "連続遷移後の最終 #app-root は SSR 相当出力とバイト一致すること"
    );
    assert_eq!(
        stub.calls(),
        3,
        "連続 3 回のクリックそれぞれで startViewTransition が呼ばれていること"
    );
}

/// `#like-btn`（`rws_app::LIKE_BUTTON_ID`、`data-hydrate="like"`）へ合成
/// クリックを dispatch する（`nav::wiring::render_route` が登録する
/// `click` リスナーは要素へ直接付く後付けのため、`document` 委譲リスナー
/// （クリック遷移用）とは異なりバブリング前提は不要だが、他テストと同じ
/// 構築関数を再利用する）。
fn dispatch_like_click(like_button: &Element) {
    like_button
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
}

/// 検証 11: `class` 属性値に `"liked"` トークンが含まれるかを判定する
/// （`DomTokenList` feature を wasm-full の dev-dependencies へ追加しない
/// 方針のため `get_attribute("class")` の文字列検査で代替する）。
fn has_liked_class(element: &Element) -> bool {
    element
        .get_attribute("class")
        .map(|value| value.split_whitespace().any(|token| token == "liked"))
        .unwrap_or(false)
}

/// 検証 11（中核）: 一覧 → 詳細（`/items/1`）へクリック遷移した後、遷移で
/// 新規構築された `#like-btn` へのクリックが `class="liked"` の付与・解除を
/// トグルすること（イシュー #403 の受け入れ条件 1・3 の直接証明）。
///
/// イシュー #404 で DOM 差し替え + `wire_hydrate_targets` 呼び出しが
/// `startViewTransition` の update コールバック内（非同期になりうる）へ
/// 移動したため、クリック直後に `#like-btn` を断定せず [`wait_until`] で
/// apply 段の完了を待つ（Cursor Bugbot 指摘 `d68cf969`）。`startViewTransition`
/// 自体は他検証と同じ [`ViewTransitionStub`] で決定的にスタブする
/// （検証 4 と同じ理由: 実ブラウザの View Transitions 挙動検証は検証
/// 7・9・10 が別途担う）。
#[wasm_bindgen_test]
async fn like_button_toggles_after_client_side_navigation() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-rewire",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(|| root.query_selector("#like-btn").unwrap().is_some(), 60).await,
        "遷移後、apply 段の完了により #like-btn が確定すること"
    );
    let like_button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("detail page must contain the like button (data-hydrate=\"like\")");
    assert!(
        !has_liked_class(&like_button),
        "遷移直後の like ボタンは liked class を持たないこと"
    );

    dispatch_like_click(&like_button);
    assert!(
        has_liked_class(&like_button),
        "遷移後に新規生成された like ボタンへのクリックが再配線され、\
         class=\"liked\" が付与されること（イシュー #403 の中核）"
    );

    dispatch_like_click(&like_button);
    assert!(
        !has_liked_class(&like_button),
        "再クリックで liked class がトグルオフされること"
    );
}

/// 検証 11（往復後の再配線）: 詳細 → 一覧 → 詳細と往復遷移した後も like
/// ボタンが機能すること。`render_route` の都度 `wire_hydrate_targets` が
/// 旧ハンドルを解除して新規登録することの直接証明
/// （`rws-wasm-client::registry::replace_handles` の反復成立）。
///
/// 検証 11（中核）と同じ理由（イシュー #404、Cursor Bugbot 指摘 `d68cf969`）
/// で `startViewTransition` を [`ViewTransitionStub`] でスタブし、各遷移後の
/// DOM 断定は [`wait_until`] で apply 段の完了を待つ。
#[wasm_bindgen_test]
async fn like_button_works_after_round_trip_navigation() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-rewire-round-trip",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // 1 回目の詳細遷移。
    document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "1 回目の詳細遷移後、apply 段の完了により詳細 DOM が確定すること"
    );

    // 一覧へ戻る。
    document
        .query_selector("a[data-nav=\"/\"]")
        .expect("query_selector must not fail")
        .expect("detail page must contain a data-nav link back to /")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        wait_until(
            || document
                .query_selector("a[data-nav=\"/items/1\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "一覧への遷移後、apply 段の完了により一覧 DOM が確定すること"
    );

    // 2 回目の詳細遷移。
    document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page (round 2) must contain a data-nav link to /items/1")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        wait_until(|| root.query_selector("#like-btn").unwrap().is_some(), 60).await,
        "2 回目の詳細遷移後、apply 段の完了により #like-btn が確定すること"
    );

    let like_button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("detail page (round 2) must contain the like button");
    dispatch_like_click(&like_button);
    assert!(
        has_liked_class(&like_button),
        "往復遷移後も like ボタンの再配線が成立すること"
    );
}

/// 検証 11（初期ページ非配線の契約）: `start_router` 直後（クライアント遷移
/// 前）の SSR 済み `#like-btn` へのクリックは `class` を変化させない。
/// 初期表示ページの配線は `rws-wasm-client::wiring::hydrate`（REQ-6 デモ）の
/// 管轄のままであり、`nav::render_route` は遷移で新規構築されたサブツリー
/// のみを対象とする（二重配線回避の凍結事項、`docs/design/
/// wasm-full-architecture.md` §10 参照）。
#[wasm_bindgen_test]
fn like_button_on_initial_page_is_not_wired_by_nav_module() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let body = rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &"1".to_string())
        .expect("infallible loader");
    set_location_path("/items/1");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-initial-not-wired",
        &rws_core::render(&body),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let like_button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("initial detail page must contain the like button");
    dispatch_like_click(&like_button);
    assert!(
        !has_liked_class(&like_button),
        "start_router 呼び出し直後（クライアント遷移前）の like ボタンは \
         nav モジュールの再配線対象外であり class は変化しないこと"
    );
}

/// 検証 11（XSS × 再配線の複合）: XSS ペイロード item（id="2"）へ遷移した後も
/// ペイロードはエスケープ済みテキストのままであり（検証 4 の既存契約を
/// 変更しない）、かつ like ボタンが機能すること。
///
/// 検証 11（中核）と同じ理由（イシュー #404、Cursor Bugbot 指摘 `d68cf969`）
/// で `startViewTransition` を [`ViewTransitionStub`] でスタブし、遷移後の
/// DOM 断定は [`wait_until`] で apply 段の完了を待つ。
#[wasm_bindgen_test]
async fn like_button_works_after_navigating_to_xss_payload_item() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-xss",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let xss_item_id = demo_items()
        .into_iter()
        .find(|it| it.title.contains("<script>"))
        .map(|it| it.id)
        .expect("demo_items() must contain the XSS payload fixture item");
    let selector = format!("a[data-nav=\"/items/{xss_item_id}\"]");
    document
        .query_selector(&selector)
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to the XSS payload item")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "XSS ペイロード item への遷移後、apply 段の完了により詳細 DOM が確定すること"
    );
    assert!(
        root.query_selector("script").unwrap().is_none(),
        "XSS ペイロードが実 DOM 上で <script> 要素として生成されてはならない（既存契約の非弱体化）"
    );
    assert!(
        root.inner_html()
            .contains("&lt;script&gt;alert('xss')&lt;/script&gt;"),
        "XSS ペイロードはエスケープ済みテキストとして DOM に保持されること: {}",
        root.inner_html()
    );

    let like_button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("XSS payload item's detail page must still contain the like button");
    dispatch_like_click(&like_button);
    assert!(
        has_liked_class(&like_button),
        "XSS ペイロード item への遷移後も like ボタンの再配線が成立すること"
    );
}

// -----------------------------------------------------------------------
// スクロール位置復元制御（イシュー #406）
// -----------------------------------------------------------------------

/// 検証 1: `start_router` 呼び出し後、`history.scrollRestoration` が
/// `"manual"` へ設定されていること（§2 の方針決定の直接証明）。
#[wasm_bindgen_test]
fn start_router_sets_scroll_restoration_to_manual() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, _root) = create_app_root(
        &document,
        "nav-test-scroll-restoration-manual",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let history = window.history().expect("history must exist");
    assert_eq!(
        history
            .scroll_restoration()
            .expect("scroll_restoration must not fail"),
        ScrollRestoration::Manual,
        "start_router は history.scrollRestoration を \"manual\" に設定すること"
    );
}

/// 検証 2: 新規遷移（クリック遷移）は常にページ先頭（`scroll_y == 0`）から
/// 表示すること（§2.2）。
///
/// イシュー #404 との統合により先頭スクロールは `apply_render_with_post` の
/// `post_apply`（`with_view_transition` の update コールバック内、実ブラウザ
/// では非同期になりうる）として実行されるため、[`wait_until`] で断定する。
/// `startViewTransition` は [`ViewTransitionStub`] でスタブする（ファイル
/// 冒頭の不変条件参照。以前はスタブせずネイティブ経路を素通ししていたが、
/// headless Chrome での完了検出ハングの原因となったため #404 フォローアップ
/// でスタブへ切り替えた）。
#[wasm_bindgen_test]
async fn click_navigation_scrolls_to_top_on_new_entry() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    reset_scroll_to_top();
    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-scroll-new-entry",
        &ssr_equivalent_list_inner_html(),
    );
    append_tall_spacer(&document, &container);
    let _cleanup = RemoveOnDrop(container);
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    window.scroll_to_with_x_and_y(0.0, 800.0);
    assert!(
        window.scroll_y().unwrap() > 0.0,
        "テスト前提条件: 遷移前にスクロール済みであること"
    );

    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-detail\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "テスト前提条件: 遷移先の DOM が確定していること"
    );
    assert!(
        wait_until(|| window.scroll_y().unwrap() == 0.0, 60).await,
        "新規遷移後はページ先頭（scroll_y == 0）から表示すること"
    );
}

/// 検証 3: `history.state` に保存済みのスクロールレコードを持つ合成
/// `popstate` を dispatch すると、その位置へ復元すること（戻る/進む操作の
/// 決定的な契約固定、`history.back()` はヘッドレスで不安定なため使わない）。
///
/// イシュー #404 との統合によりスクロール復元は
/// `navigate_render_with_post` の `post_apply`（`with_view_transition` の
/// update コールバック内、実ブラウザでは非同期になりうる）として実行される
/// ため、[`wait_until`] で断定する。`startViewTransition` は
/// [`ViewTransitionStub`] でスタブする（ファイル冒頭の不変条件参照。以前は
/// スタブせずネイティブ経路を素通ししていたが、headless Chrome での完了
/// 検出ハングの原因となったため #404 フォローアップでスタブへ切り替えた）。
#[wasm_bindgen_test]
async fn popstate_with_encoded_state_restores_saved_scroll_position() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    reset_scroll_to_top();
    set_location_path("/items/1");
    let body = rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &"1".to_string())
        .expect("infallible loader");
    let (container, root) = create_app_root(
        &document,
        "nav-test-scroll-popstate-restore",
        &rws_core::render(&body),
    );
    append_tall_spacer(&document, &container);
    let _cleanup = RemoveOnDrop(container);
    document.set_title("記事詳細");
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // 「戻る」操作を模した状態遷移: URL を書き換え、離脱元エントリの
    // スクロール位置を積んだ state 付きの合成 popstate を dispatch する
    // （`push_and_render` が実運用で `replace_state` に積む値と同じ形式）。
    set_location_path("/");
    let init = PopStateEventInit::new();
    init.set_state(&wasm_bindgen::JsValue::from_str(&encode_scroll_state(
        0.0, 500.0,
    )));
    let event = PopStateEvent::new_with_event_init_dict("popstate", &init)
        .expect("PopStateEvent construction must not fail");
    window
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-list\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "テスト前提条件: popstate によりルートが再解決・再描画されていること"
    );
    assert!(
        wait_until(|| (window.scroll_y().unwrap() - 500.0).abs() < 1.0, 60).await,
        "state に積まれたスクロール位置（500）へサブピクセル許容で復元すること: {}",
        window.scroll_y().unwrap()
    );
}

/// 検証 4: state が `NULL`（`push_state` が積む通常値）または不正な形式の
/// 合成 `popstate` では、スクロール済みの状態からも先頭 `(0, 0)` へ
/// フォールバックすること（fail-closed、§2.2）。
///
/// イシュー #404 との統合によりスクロール復元は
/// `navigate_render_with_post` の `post_apply`（`with_view_transition` の
/// update コールバック内、実ブラウザでは非同期になりうる）として実行される
/// ため、[`wait_until`] で断定する。`startViewTransition` は
/// [`ViewTransitionStub`] でスタブする（ファイル冒頭の不変条件参照。以前は
/// スタブせずネイティブ経路を素通ししていたが、headless Chrome での完了
/// 検出ハングの原因となったため #404 フォローアップでスタブへ切り替えた）。
#[wasm_bindgen_test]
async fn popstate_with_null_or_invalid_state_falls_back_to_top() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    reset_scroll_to_top();
    set_location_path("/items/1");
    let body = rws_app::assemble_detail_page(&rws_app::DemoItemDetailLoader, &"1".to_string())
        .expect("infallible loader");
    let (container, root) = create_app_root(
        &document,
        "nav-test-scroll-popstate-fallback",
        &rws_core::render(&body),
    );
    append_tall_spacer(&document, &container);
    let _cleanup = RemoveOnDrop(container);
    document.set_title("記事詳細");
    let _stub = ViewTransitionStub::install(&document);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    window.scroll_to_with_x_and_y(0.0, 900.0);
    assert!(
        window.scroll_y().unwrap() > 0.0,
        "テスト前提条件: popstate 前にスクロール済みであること"
    );

    // NULL state（`push_state` が積む通常値と同じ）。
    set_location_path("/");
    window
        .dispatch_event(&synthetic_popstate_event())
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(
            || root
                .query_selector("[data-testid=\"item-list\"]")
                .unwrap()
                .is_some(),
            60
        )
        .await,
        "テスト前提条件: popstate によりルートが再解決・再描画されていること"
    );
    assert!(
        wait_until(|| window.scroll_y().unwrap() == 0.0, 60).await,
        "NULL state の popstate では先頭 (0, 0) へフォールバックすること"
    );

    // 不正な形式の state。
    window.scroll_to_with_x_and_y(0.0, 900.0);
    set_location_path("/items/1");
    let init = PopStateEventInit::new();
    init.set_state(&wasm_bindgen::JsValue::from_str("not-a-scroll-record"));
    let invalid_event = PopStateEvent::new_with_event_init_dict("popstate", &init)
        .expect("PopStateEvent construction must not fail");
    window
        .dispatch_event(&invalid_event)
        .expect("dispatch_event must not fail");

    assert!(
        wait_until(|| window.scroll_y().unwrap() == 0.0, 60).await,
        "不正な形式の state を持つ popstate でも先頭 (0, 0) へフォールバックすること（fail-closed）"
    );
}

/// 検証 5: ルート表に一致しないパスへの合成 `popstate` はスクロール位置を
/// 変更しないこと（従来どおり完全 no-op、§2.2）。
#[wasm_bindgen_test]
fn popstate_for_unresolved_path_does_not_change_scroll_position() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    reset_scroll_to_top();
    set_location_path("/");
    let (container, _root) = create_app_root(
        &document,
        "nav-test-scroll-popstate-noop",
        &ssr_equivalent_list_inner_html(),
    );
    append_tall_spacer(&document, &container);
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    window.scroll_to_with_x_and_y(0.0, 700.0);
    assert!(
        window.scroll_y().unwrap() > 0.0,
        "テスト前提条件: popstate 前にスクロール済みであること"
    );

    set_location_path("/no-such-route");
    window
        .dispatch_event(&synthetic_popstate_event())
        .expect("dispatch_event must not fail");

    assert_eq!(
        window.scroll_y().unwrap(),
        700.0,
        "ルート表に一致しないパスへの popstate はスクロール位置を変更しない（完全 no-op）こと"
    );
}
