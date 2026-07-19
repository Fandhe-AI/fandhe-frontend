//! `hydrate()` / `mount_csr()` の実ブラウザ正式実証テスト（TASK-6.3、#64〜#68）。
//!
//! `hydrate_smoke.rs`（TASK-6.2c、#49）は CI の `browser-test` ジョブを green に
//! 保つための最小限のスモークテストであり、REQ-6 受け入れ基準 4（実ブラウザでの
//! クリックイベント発火・状態復元）の**正式な実証**は本ファイルが引き継ぐ
//! （`docs/spec/05-tasks.md` TASK-6.3、`docs/guides/browser-testing.md` 第 7 節）。
//!
//! 本ファイルが実ブラウザで実証する 5 観点（`docs/api/hydration-api.md` 第 3 節・
//! 第 6 節の不変条件に対応）:
//!
//! 1. SSR/SSG 出力との整合（CSR の DOM 反映） — [`mount_csr_reflects_same_render_output_as_ssr`]
//! 2. サーバー出力済み DOM を再構築しないハイドレーション — [`hydrate_does_not_rebuild_server_rendered_dom`]
//! 3. クリックイベント発火 — [`hydrate_toggles_liked_class_on_click_and_untoggles_on_second_click`]
//! 4. 状態復元（既存状態の保持・再ハイドレーション後の状態保持） — [`hydrate_preserves_pre_existing_liked_state`] /
//!    [`re_hydrate_preserves_click_state_and_fires_exactly_once`]
//! 5. 実ブラウザでの既定エスケープ証跡（REQ-1 連動） — [`xss_payload_item_does_not_produce_script_element_in_real_dom`]
//!
//! 状態注入フォーマット（`data-hydrate-*` エンコード）による状態復元の製品化は
//! TASK-11.4 系（#81〜#84）のスコープであり、本ファイルの検証範囲は「サーバー
//! 出力済み DOM 上の状態（テキスト・属性・class）が `hydrate()` で破壊されず
//! 引き継がれること」に限定する（`docs/api/hydration-api.md` 第 5 節スコープ外表）。

#![cfg(target_arch = "wasm32")]

use rws_wasm_client::{hydrate, mount_csr, render_detail_page_html, render_list_page_html};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナを document body へ 1 個生成し、一意な id を振る。
/// `wasm-bindgen-test` はテスト間で DOM をリセットしないため、テストごとに
/// 個別の id を振って干渉を避ける（`hydrate_smoke.rs::create_container` と同じ意図）。
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

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

/// 観点 1: `mount_csr()` は [`render_list_page_html`]（SSR/SSG と同一関数の出力）を
/// root の `inner_html` へ反映する。ブラウザの innerHTML 正規化差異（属性順序・
/// 空白等）による偽陰性を避けるため、生文字列との直接比較ではなく、同じ
/// `render_list_page_html()` の出力を参照ノードへ `set_inner_html` した結果と
/// `inner_html` 同士で比較する（同一注入経路同士の比較。SSR/SSG 間の文字列完全
/// 一致自体は `server/tests/ssr_ssg_parity.rs` が native で担保済み）。
#[wasm_bindgen_test]
fn mount_csr_reflects_same_render_output_as_ssr() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-mount-csr-root");
    mount_csr("browser-mount-csr-root").expect("mount_csr must succeed when root exists");

    let reference = create_container(&document, "browser-mount-csr-reference");
    reference.set_inner_html(&render_list_page_html());

    assert_eq!(
        root.inner_html(),
        reference.inner_html(),
        "mount_csr() は render_list_page_html()（SSR/SSG と同一関数）の出力を \
         そのまま inner_html へ反映すること"
    );
}

/// 観点 2: `hydrate()` はサーバー出力済み DOM（`render_detail_page_html` の
/// 出力を模した既存 DOM）を再構築しない（`set_inner_html` 等を呼ばない、
/// `docs/api/hydration-api.md` 不変条件 2）。`hydrate()` 前後で root の `inner_html`
/// がバイト一致することを実ブラウザ上で確認する。
#[wasm_bindgen_test]
fn hydrate_does_not_rebuild_server_rendered_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-no-rebuild-root");
    // サーバー出力済み DOM を模す: render_detail_page_html() の出力を
    // 直接 inner_html へセットする（これは「サーバーが返した HTML をブラウザが
    // パースした結果」を模する準備段階であり、hydrate() 自体の呼び出しではない）。
    root.set_inner_html(&render_detail_page_html("1"));
    let inner_html_before = root.inner_html();

    hydrate("browser-no-rebuild-root").expect("hydrate must succeed when root and target exist");

    assert_eq!(
        root.inner_html(),
        inner_html_before,
        "hydrate() 実行前後で inner_html がバイト一致すること（再構築しない不変条件の実ブラウザ証跡）"
    );
}

/// 観点 3: `data-hydrate="like"` 要素（`rws_app::detail_page` 実出力の
/// `#like-btn`）への合成 click イベント（`bubbles: true`）で `liked` クラスが
/// トグルされ、2 回目のクリックで解除されること（ハンドラが 1 回だけ登録されて
/// いる証跡）。
#[wasm_bindgen_test]
fn hydrate_toggles_liked_class_on_click_and_untoggles_on_second_click() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-click-toggle-root");
    root.set_inner_html(&render_detail_page_html("1"));
    let button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("render_detail_page_html must emit #like-btn");

    hydrate("browser-click-toggle-root").expect("hydrate must succeed when root and target exist");

    assert!(
        !button.class_list().contains("liked"),
        "hydrate() 直後（クリック前）は liked クラスが付与されていないこと"
    );

    button
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        button.class_list().contains("liked"),
        "1 回目のクリックで liked クラスが付与されること"
    );

    button
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        !button.class_list().contains("liked"),
        "2 回目のクリックで liked クラスが解除されること（ハンドラが 1 回だけ登録されている証跡）"
    );
}

/// 観点 4a: サーバー出力済み DOM に事前付与された状態（`liked` クラス等）が
/// `hydrate()` 後も保持されること。状態注入フォーマット（TASK-11.4 スコープ）
/// ではなく、素朴な DOM 属性（`class`）の保持を確認する。
#[wasm_bindgen_test]
fn hydrate_preserves_pre_existing_liked_state() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-preserve-state-root");
    root.set_inner_html(&render_detail_page_html("1"));
    let button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("render_detail_page_html must emit #like-btn");
    // サーバーが「既にいいね済み」の状態で DOM を出力した状況を模す
    // （状態注入フォーマットの製品化を待たず、DOM 属性の素朴な事前付与で代替）。
    button
        .class_list()
        .add_1("liked")
        .expect("class_list().add_1 must not fail");

    hydrate("browser-preserve-state-root")
        .expect("hydrate must succeed when root and target exist");

    // hydrate() が set_inner_html 等で DOM を再構築した場合、hydrate() 前に
    // 取得した button 参照は detached ノードとなり、クラス状態を読んでも
    // 生きた DOM の状態を反映しない（偽陽性の温床）。再構築の有無に関わらず
    // 「今の生きた DOM」を検証するため、hydrate() 後に #like-btn を再クエリする
    // （Bugbot 指摘: stale DOM 参照によるアサーション回避の防止）。
    let button_after = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("#like-btn must still exist in the live DOM after hydrate()");

    assert!(
        button_after.class_list().contains("liked"),
        "hydrate() はサーバー出力済み DOM の既存 class を破壊しないこと"
    );
}

/// 観点 4b（PR #236 Bugbot 指摘の回帰確認を兼ねる）: クリックで変化した状態が
/// 同一 root への再 `hydrate()` 後も保持され、再クリックでリスナーが二重発火
/// しない（1 クリック＝1 トグル）こと。
#[wasm_bindgen_test]
fn re_hydrate_preserves_click_state_and_fires_exactly_once() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-rehydrate-state-root");
    root.set_inner_html(&render_detail_page_html("1"));
    // hydrate() 前に #like-btn の存在のみ確認する（サニティチェック）。この
    // 参照はハイドレーション後の検証には使わない（後続の再クエリ参照）。
    root.query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("render_detail_page_html must emit #like-btn");

    hydrate("browser-rehydrate-state-root")
        .expect("first hydrate must succeed when root and target exist");

    // hydrate() が DOM を再構築していれば、1 回目の hydrate() 前に取得した
    // button 参照は detached になる。再構築有無を問わず「生きた DOM」に対して
    // クリック・アサーションを行うため、各 hydrate() 呼び出し後に #like-btn を
    // 再クエリする（Bugbot 指摘: stale DOM 参照による偽陽性の防止。以降の
    // dispatch_event / class_list 参照はすべて button_after_first を使う）。
    let button_after_first = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("#like-btn must still exist in the live DOM after the first hydrate()");

    button_after_first
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    assert!(
        button_after_first.class_list().contains("liked"),
        "1 回目のクリックで liked クラスが付与されること"
    );

    // 同一 root_id への再ハイドレーション（registry::replace_handles が
    // 旧リスナーを remove_event_listener_with_callback で解除する契約）。
    hydrate("browser-rehydrate-state-root")
        .expect("second hydrate on the same root_id must succeed (re-hydration)");

    // 2 回目の hydrate() 後も同様に再クエリし、以降の検証・クリックは
    // 生きた DOM の #like-btn（button_after_second）に対して行う。
    let button_after_second = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail for a valid selector")
        .expect("#like-btn must still exist in the live DOM after the second hydrate()");

    assert!(
        button_after_second.class_list().contains("liked"),
        "再 hydrate() 後もクリックで変化した状態（liked クラス）が保持されること"
    );

    // 孤立した旧リスナーが残っていれば、ここで 1 クリックにつき 2 回
    // トグルが走り liked が再び外れてしまう（二重発火の検出）。
    button_after_second
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail after re-hydrate");
    assert!(
        !button_after_second.class_list().contains("liked"),
        "再 hydrate() 後も 1 クリック＝1 トグルであること（リスナー二重発火なし）"
    );
}

/// 観点 5（REQ-1 連動）: `demo_items()[1]` の XSS ペイロードを
/// `render_detail_page_html("2")` で描画し、実 DOM 上で `script` 要素が
/// 生成されない（`query_selector("script")` が `None`）・タイトルがテキスト
/// として表示されることを確認する。DOM への HTML 挿入は
/// `render_detail_page_html`（`rws_core::render` 出力、既定エスケープ済み）
/// のみを経由し、`format!` による HTML 組み立て・`raw_html()` は使わない
/// （`docs/api/hydration-api.md` 第 6 節不変条件 1・4）。
#[wasm_bindgen_test]
fn xss_payload_item_does_not_produce_script_element_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "browser-xss-root");
    root.set_inner_html(&render_detail_page_html("2"));

    let script_element = root
        .query_selector("script")
        .expect("query_selector must not fail for a valid selector");
    assert!(
        script_element.is_none(),
        "XSS ペイロードを含むタイトルでも実 DOM 上に script 要素が生成されないこと"
    );

    let title = root
        .query_selector("[data-testid=\"item-title\"]")
        .expect("query_selector must not fail for a valid selector")
        .expect("render_detail_page_html must emit the item-title element");
    assert!(
        title
            .text_content()
            .expect("text_content must be present")
            .contains("<script>alert('xss')</script>"),
        "エスケープされたペイロードがタグとして解釈されず、テキストとして表示されること"
    );
}
