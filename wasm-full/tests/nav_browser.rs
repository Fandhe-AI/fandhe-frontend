//! クライアント側ルーティングの実ブラウザ統合テスト（イシュー #374、
//! `wasm-pack test --headless --chrome`）。
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
//! # 検証内容（#374 実装計画 §6 に対応）
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
//! 7. 遷移後の `data-hydrate` 再配線（イシュー #403）: 遷移 → いいねボタン
//!    操作 → `class="liked"` 付与・トグル、往復遷移後の再配線、初期表示
//!    ページ非配線の契約、XSS 複合ケース

#![cfg(target_arch = "wasm32")]

use rws_app::{assemble_list_page, demo_items, DemoItemsLoader};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit, PopStateEvent, PopStateEventInit};

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

/// 検証 1・6: `/` → `/items/1` → `/` の連続クリック遷移で URL・DOM・
/// `document.title` が追従し、往復後もクリック配線が生きていること。
#[wasm_bindgen_test]
fn click_navigation_updates_url_dom_and_title_across_round_trip() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-round-trip",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // 1 回目: 一覧 → 詳細（/items/1）。
    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    assert_eq!(window.location().pathname().unwrap(), "/items/1");
    assert_eq!(document.title(), "記事詳細");
    assert!(root
        .query_selector("[data-testid=\"item-detail\"]")
        .unwrap()
        .is_some());
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
    assert_eq!(document.title(), "記事一覧");
    assert!(root
        .query_selector("[data-testid=\"item-list\"]")
        .unwrap()
        .is_some());

    // 3 回目: 一覧 → 詳細（2 往復目）。委譲リスナーが継続して機能すること。
    let link_again = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page (round 2) must contain a data-nav link to /items/1");
    link_again
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");
    assert_eq!(window.location().pathname().unwrap(), "/items/1");
}

/// 検証 2: `history.replace_state` で `/` へ揃えたのち合成 `popstate` を
/// dispatch すると、一覧 DOM・タイトルへ復帰する（`history.back()` の
/// 非同期性を避けた決定的な契約固定）。
#[wasm_bindgen_test]
fn popstate_event_re_resolves_and_renders_without_pushing_history() {
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

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // ブラウザの「戻る」操作を模した状態遷移: URL を `/` へ書き換えてから
    // popstate を発火する（実際の `back()` は非同期でヘッドレス環境が
    // 不安定なため、決定的な合成イベントでハンドラ契約を固定する）。
    set_location_path("/");
    window
        .dispatch_event(&synthetic_popstate_event())
        .expect("dispatch_event must not fail");

    assert_eq!(document.title(), "記事一覧");
    assert!(root
        .query_selector("[data-testid=\"item-list\"]")
        .unwrap()
        .is_some());
}

/// 検証 3: SSR 済み DOM の上で `start_router` を呼んでも DOM は不変
/// （初期表示で loader を再実行・再描画しない凍結事項の直接証明）。
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
/// 実 DOM 上でテキストのまま（要素化されない）こと。
#[wasm_bindgen_test]
fn navigating_to_xss_payload_item_keeps_payload_as_text_not_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-xss-payload",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

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
/// （ブラウザ既定動作を壊さない安全側フォールバック）。
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

/// 検証 7: `class` 属性値に `"liked"` トークンが含まれるかを判定する
/// （`DomTokenList` feature を wasm-full の dev-dependencies へ追加しない
/// 方針のため `get_attribute("class")` の文字列検査で代替する）。
fn has_liked_class(element: &Element) -> bool {
    element
        .get_attribute("class")
        .map(|value| value.split_whitespace().any(|token| token == "liked"))
        .unwrap_or(false)
}

/// 検証 7（中核）: 一覧 → 詳細（`/items/1`）へクリック遷移した後、遷移で
/// 新規構築された `#like-btn` へのクリックが `class="liked"` の付与・解除を
/// トグルすること（イシュー #403 の受け入れ条件 1・3 の直接証明）。
#[wasm_bindgen_test]
fn like_button_toggles_after_client_side_navigation() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-rewire",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    let link = document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1");
    link.dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

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

/// 検証 7（往復後の再配線）: 詳細 → 一覧 → 詳細と往復遷移した後も like
/// ボタンが機能すること。`render_route` の都度 `wire_hydrate_targets` が
/// 旧ハンドルを解除して新規登録することの直接証明
/// （`rws-wasm-client::registry::replace_handles` の反復成立）。
#[wasm_bindgen_test]
fn like_button_works_after_round_trip_navigation() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-rewire-round-trip",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

    rws_wasm_full::nav::start_router("app-root").expect("start_router must succeed");

    // 1 回目の詳細遷移。
    document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page must contain a data-nav link to /items/1")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    // 一覧へ戻る。
    document
        .query_selector("a[data-nav=\"/\"]")
        .expect("query_selector must not fail")
        .expect("detail page must contain a data-nav link back to /")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

    // 2 回目の詳細遷移。
    document
        .query_selector("a[data-nav=\"/items/1\"]")
        .expect("query_selector must not fail")
        .expect("list page (round 2) must contain a data-nav link to /items/1")
        .dispatch_event(&synthetic_click_event())
        .expect("dispatch_event must not fail");

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

/// 検証 7（初期ページ非配線の契約）: `start_router` 直後（クライアント遷移
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

/// 検証 7（XSS × 再配線の複合）: XSS ペイロード item（id="2"）へ遷移した後も
/// ペイロードはエスケープ済みテキストのままであり（検証 4 の既存契約を
/// 変更しない）、かつ like ボタンが機能すること。
#[wasm_bindgen_test]
fn like_button_works_after_navigating_to_xss_payload_item() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    set_location_path("/");
    let (container, root) = create_app_root(
        &document,
        "nav-test-hydrate-xss",
        &ssr_equivalent_list_inner_html(),
    );
    let _cleanup = RemoveOnDrop(container);

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
