//! TASK-1.3b（#92、親 #90、REQ-1）: WASM 経路（クライアントのイベント処理・
//! DOM 更新）における既定エスケープ保証の実ブラウザ回帰テスト。
//!
//! `wasm-full/tests/dom_update.rs`（TASK-11.2c・#76）は DOM/`wasm-bindgen` に
//! 依存しない純粋関数 [`rws_wasm_full::render_component_html`] までを native
//! `cargo test` で検証済みである。本ファイルはその先、**実ブラウザ
//! （headless Chromium、`wasm-pack test --headless --chrome`）上で
//! `set_inner_html` による実 DOM 反映まで含めた製品経路**を検証する点に
//! 付加価値がある（`docs/spec/04-requirements.md` REQ-1 受け入れ基準
//! 「クライアント WASM のイベント処理・DOM 更新を経由した出力にも同一の
//! エスケープ保証が及ぶこと」）。
//!
//! # #91（TASK-1.3a）との関係
//!
//! 本イシュー着手時点で #91（テスト設計サブタスク）は OPEN・未マージのため、
//! 実装計画（#92）第 2 節のフォールバック方針に従い、`docs/spec/05-tasks.md`
//! TASK-1.3 の記述と `docs/wasm-full-architecture.md` 第 3 節の公開面を根拠に
//! 本ファイルのテストケースを設計した。#91 の設計ドキュメントがマージされた
//! 場合は、その設計との整合を別途レビューする（実装計画 §2 参照）。
//!
//! # 配置クレートについて
//!
//! 仕様上の成果物パスは `wasm-client/tests/xss_escape_wasm.rs` だが、本コミット
//! 時点で `wasm-client/`（TASK-6.2b・#48）は未作成である。REQ-1 が検証対象と
//! する「イベント処理・DOM 更新」の実装実体は本クレート（`rws-wasm-full`、
//! `events.rs`/`dom.rs`）であるため、存在しないクレートを本イシューの範囲外
//! で新設することを避け、安全側の判断として本クレートに配置する
//! （実装計画 §2・§9、`out-of-scope-tracking.md` に基づき #48 マージ後の
//! 移設要否をユーザーに提案する）。
//!
//! # 検証する経路と不変条件
//!
//! - `rws_wasm_full::render_component_html`（[`dom` モジュール](../src/dom.rs)、
//!   `rws_core::render` の呼び出しのみ）が返す文字列だけが `set_inner_html` に
//!   渡ること。本ファイルは `format!` 等による HTML 文字列直接組み立てを
//!   一切行わず、`raw_html()` も呼ばない（不変条件、`.claude/rules/coding-rust.md`）。
//! - テキストノード経由（`items` へ確定した値）・属性値経由（`draft` の
//!   `value` 属性）の双方でエスケープが効くこと。
//! - `wire_events`（`events.rs`）によるイベント委譲配線を経由しても、
//!   直接 `dispatch` した場合と同一のエスケープ保証が成立すること
//!   （REQ-1 の本旨: イベント処理経路でも保証が弱まらないこと）。
//! - SSR 相当の直接呼び出し（`rws_core::render(&state.view())`）と、WASM 経路
//!   （`render_component_html` → `set_inner_html` → 実 DOM 読み戻し）とで、
//!   エスケープ保証の観点に矛盾がないこと（経路間一貫性）。
//!
//! ペイロード集合は `core/tests/xss_escape.rs`（TASK-1.2・#8）と対応させる
//! （`<script>` タグ注入・属性注入・`<img onerror>` の代表例）。
//!
//! # ハイドレーション経路の XSS 回帰（TASK-11.4c・#84、`docs/hydration-state-format.md`
//! 第 6 節「XSS 回帰」）
//!
//! `render_for_hydration`（`interactive/src/lib.rs`）が付与する `data-hydrate-*`
//! 属性も `rws_core::render` の既定エスケープ経路を通るため、ハイドレーション
//! フォーマット層の追加が既存のエスケープ保証を弱めていないことを回帰確認する。
//! `Runtime::hydrate`（`wasm-full/src/lib.rs`）で状態を復元 → 再描画した実 DOM
//! に対し、上記の「テキストノード経由」「属性値経由」と同じ観点（生タグが
//! 要素として生成されない・属性境界が破られない）を検証する。

#![cfg(target_arch = "wasm32")]

use rws_interactive::{dispatch, render_for_hydration, AppState, Component};
use rws_wasm_full::events::{wire_events, ActionRef};
use rws_wasm_full::render_component_html;
use rws_wasm_full::Runtime;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する。
///
/// `wasm-full/tests/perf_browser.rs` の `create_scenario_container` と同じ
/// 意図: 一意な id を振ることで、同一テストバイナリ内の複数テストケースが
/// 要素を奪い合わないようにする（wasm-bindgen-test はテスト間で DOM を
/// リセットしない）。
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

/// draft へペイロードを設定し `add_item` で `items` へ確定させた [`AppState`] を作る。
///
/// `items[i]` はテキストノード（`interactive/src/lib.rs` の
/// `render_with_root_attrs`）として描画されるため、テキストノード経由の
/// エスケープ検証に使う。
fn state_with_item(payload: &str) -> AppState {
    let mut state = AppState::new();
    assert!(dispatch(&mut state, "set_draft", payload));
    assert!(dispatch(&mut state, "add_item", ""));
    state
}

/// 合成 `click` イベントを生成する（`bubbles: true`）。
///
/// `wire_events`（events.rs）はリスナーを root 要素へ登録するため、子要素上で
/// 発火したイベントがバブリングで root まで届く必要がある。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

/// テスト末尾でコンテナを document から確実に除去する RAII ガード。
///
/// `AppState::view`（`render_with_root_attrs`）が返すルート要素は固定 id
/// `"interactive-root"` を持つため、後始末を怠ると同一ページ上で後から実行
/// される別テスト（特に本ファイル末尾のハイドレーション経路テスト）の
/// `document.get_element_by_id("interactive-root")`（`Runtime::hydrate` 内部）
/// が本テストの残留要素を誤って拾う（`wasm-full/tests/runtime_browser.rs::RemoveOnDrop`
/// と同じ理由、CI issue #73 の再発防止パターン）。本ファイルの全テストが
/// `render_component_html`/`render_for_hydration` いずれかを経由して同じ
/// 固定 id を出力するため、全テストで一貫してこのガードを使う。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// REQ-1 回帰（テキストノード経路・実 DOM）: `<script>` ペイロードを `items` へ
/// 確定させ `set_inner_html` で実 DOM へ反映しても、
///
/// - ルート配下に `script` 要素が 1 つも生成されない
/// - 該当テキストノードの `text_content()` がペイロード原文と一致する
///   （ブラウザがエンティティを復元しても二重エスケープが起きていない確認）
/// - `inner_html()` に生の `<script>` は含まれず `&lt;script&gt;` が含まれる
///
/// ことを確認する。
#[wasm_bindgen_test]
fn script_tag_payload_in_item_text_is_escaped_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "xss-text-node-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "<script>alert(1)</script>";
    let state = state_with_item(payload);

    let html = render_component_html(&state);
    container.set_inner_html(&html);

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "生の <script> 要素が実 DOM に生成されてはならない"
    );

    // `data-testid="item-list"`（`ul`）は `interactive/src/lib.rs` の
    // `render_with_root_attrs` が付与する固定 testid。
    let items_root = container
        .query_selector("[data-testid='item-list']")
        .expect("query_selector must not fail")
        .expect("item-list container must exist in rendered output");
    assert!(
        items_root
            .text_content()
            .unwrap_or_default()
            .contains(payload),
        "item-list 配下のテキストにペイロード原文が（エスケープ解除された形で）含まれること"
    );

    let inner = container.inner_html();
    assert!(
        !inner.contains("<script>"),
        "inner_html に生の <script> タグが含まれてはならない: {inner}"
    );
    assert!(
        inner.contains("&lt;script&gt;"),
        "inner_html にエスケープ済みペイロードが含まれること: {inner}"
    );
}

/// REQ-1 回帰（属性値経路・実 DOM）: 属性境界を破壊しようとするペイロードが
/// `draft` の `value` 属性へ渡っても、
///
/// - ルート配下のどの要素にも `onmouseover` 属性が実 DOM 上に存在しない
///   （属性注入が成立していない）
/// - `value` 属性の読み戻しがペイロード原文と一致する
///   （二重エスケープなしの確認）
///
/// ことを確認する。
#[wasm_bindgen_test]
fn attribute_injection_payload_in_draft_value_is_escaped_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "xss-attr-injection-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\" onmouseover=\"alert(1)\" data-x=\"'&";
    let mut state = AppState::new();
    assert!(dispatch(&mut state, "set_draft", payload));

    let html = render_component_html(&state);
    container.set_inner_html(&html);

    let input = container
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist in rendered output");

    assert!(
        input.get_attribute("onmouseover").is_none(),
        "属性注入によって onmouseover 属性が実 DOM に生成されてはならない"
    );
    assert_eq!(
        input.get_attribute("value").as_deref(),
        Some(payload),
        "value 属性の読み戻しがペイロード原文と一致すること（二重エスケープなし）"
    );

    let inner = container.inner_html();
    assert!(
        inner.contains("&quot;"),
        "inner_html に二重引用符のエスケープ済み表現が含まれること: {inner}"
    );
}

/// REQ-1 回帰（イベント処理経路・実 DOM）: `wire_events` によるイベント委譲配線
/// を経由した場合でも、直接 `dispatch` した場合と同一のエスケープ保証が
/// 成立することを確認する（TASK-1.3 の本旨）。
///
/// 合成 `click` イベントを `data-action="set_draft"` / `data-payload=<ペイロード>`
/// を持つ要素（`root` の子孫）に対して発火させ、`wire_events` の判定
/// （[`rws_wasm_full::events::action_from_click`]）→ コールバック内での
/// `dispatch` → 再描画（`render_component_html` → `set_inner_html`）という
/// 製品経路を通しで検証する。
#[wasm_bindgen_test]
fn set_draft_via_wired_click_event_preserves_escape_guarantee() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "xss-event-wired-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let state = Rc::new(RefCell::new(AppState::new()));

    // 初期描画（マウント相当）。
    container.set_inner_html(&render_component_html(&*state.borrow()));
    let root = container
        .first_element_child()
        .expect("render_component_html output must contain a root element");

    // マウント時に 1 回だけイベント配線する契約（events.rs 冒頭コメント）。
    // コールバックは dispatch → 再描画のみを行い、HTML 文字列を独自に
    // 組み立てない（唯一の生成経路は render_component_html = rws_core::render）。
    {
        let state = Rc::clone(&state);
        let container = container.clone();
        wire_events(root.clone(), move |action_ref: ActionRef| {
            let dispatched = {
                let mut state = state.borrow_mut();
                dispatch(&mut *state, &action_ref.action, &action_ref.payload)
            };
            if dispatched && action_ref.should_repaint {
                let html = render_component_html(&*state.borrow());
                container.set_inner_html(&html);
            }
        })
        .expect("wire_events must not fail");
    }

    // root の子孫として、悪性ペイロードを data-payload に載せた合成クリック
    // ターゲットを追加する（events.rs の delegation は root 直下のみでなく
    // closest("[data-action]") による祖先探索のため、子孫要素で成立する）。
    let payload = "\" onmouseover=\"alert(1)\" data-x=\"'&";
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", "set_draft")
        .expect("set_attribute must not fail");
    trigger
        .set_attribute("data-payload", payload)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");

    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");

    // set_draft は should_repaint=true（action_from_click の契約）のため、
    // コールバック内で再描画済み。container 配下（新しい root）から
    // draft-input を再取得して検証する。
    let input = container
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist after repaint");

    assert!(
        input.get_attribute("onmouseover").is_none(),
        "イベント配線経由でも属性注入が成立してはならない"
    );
    assert_eq!(
        input.get_attribute("value").as_deref(),
        Some(payload),
        "イベント配線経由でも value 属性の読み戻しがペイロード原文と一致すること"
    );
    assert!(
        container
            .query_selector("[onmouseover]")
            .expect("query_selector must not fail")
            .is_none(),
        "イベント配線経由でも onmouseover 属性を持つ要素が実 DOM に存在してはならない"
    );
}

/// REQ-1 回帰（経路間一貫性）: SSR 相当の直接呼び出し
/// （`rws_core::render(&state.view())`）と WASM 経路
/// （`render_component_html` → `set_inner_html` → 実 DOM 読み戻し）とで、
/// エスケープ保証の観点に矛盾がないことを確認する。
///
/// `render_component_html`（`dom.rs`）は `rws_core::render` を呼ぶだけの薄い
/// 層のため、両者の出力文字列は本来完全に一致する契約である
/// （`wasm-full/src/dom.rs` 冒頭コメント）。本テストはその契約に加え、
/// ブラウザの HTML パーサを一度経由した実 DOM 上でも同じ保証
/// （`<img onerror>` ペイロードが要素として生成されない）が崩れないことまで
/// 検証する。
#[wasm_bindgen_test]
fn ssr_equivalent_render_and_wasm_dom_path_agree_on_escape_guarantee() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "xss-cross-path-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "<img src=x onerror=alert(1)>";
    let state = state_with_item(payload);

    // SSR 相当: rws_core::render を直接呼ぶ（rws-wasm-full を経由しない）。
    let ssr_html = rws_core::render(&state.view());
    // WASM 経路: render_component_html（dom.rs）経由。
    let wasm_html = render_component_html(&state);

    assert_eq!(
        ssr_html, wasm_html,
        "render_component_html は rws_core::render の薄いラッパーであり、\
         SSR 相当の出力と完全に一致する契約であること"
    );

    container.set_inner_html(&wasm_html);

    assert!(
        container
            .query_selector("img")
            .expect("query_selector must not fail")
            .is_none(),
        "img 要素として実 DOM に生成されてはならない（onerror が発火し得る形で解釈されない）"
    );
    assert!(
        !container.inner_html().contains("<img"),
        "inner_html に生の <img> タグが含まれてはならない: {}",
        container.inner_html()
    );
}

/// REQ-1 回帰（ハイドレーション経路・実 DOM、TASK-11.4c・#84）: `render_for_hydration`
/// が付与する `data-hydrate-*` 属性値に XSS ペイロードを含む状態を SSR 出力
/// → 実 DOM 展開 → `Runtime::hydrate` で復元しても、
///
/// - 復元直後の既存 DOM（SSR 出力そのもの）に生の `<script>` 要素が存在しない
/// - `draft-input` の `value` 属性の読み戻しがペイロード原文と一致する
///   （ハイドレーション属性からの復元でも二重エスケープが起きない）
///
/// ことを確認する。ハイドレーション属性フォーマット層
/// （`wasm-full/src/hydration.rs`）の追加が `render_for_hydration`
/// （`rws_core::render` の既定エスケープ経路）を迂回していないことの回帰。
#[wasm_bindgen_test]
fn hydrate_restores_xss_payload_in_draft_without_escape_regression() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let payload = "\" onmouseover=\"alert(1)\" data-x=\"'&";
    let mut seed_state = AppState::new();
    assert!(dispatch(&mut seed_state, "set_draft", payload));

    // `render_for_hydration` は rws_core::render の既定エスケープ経路のみを
    // 通す唯一の生成経路（`.claude/rules/coding-rust.md`: HTML 文字列直接
    // 組み立て・raw_html() 禁止）。ルート要素（id="interactive-root"）へ
    // hydration_attrs を後付けした Node を rws_core::render でそのまま文字列化する。
    let ssr_html = rws_core::render(&render_for_hydration(&seed_state));

    let container = create_container(&document, "xss-hydrate-draft-root");
    let _cleanup = RemoveOnDrop(container.clone());
    container.set_inner_html(&ssr_html);

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "SSR 出力の実 DOM 展開時点で生の <script> 要素が生成されてはならない"
    );

    let runtime = Runtime::hydrate("interactive-root", AppState::new())
        .expect("hydrate must succeed for well-formed SSR output containing an XSS payload");
    assert_eq!(
        runtime.component().draft,
        payload,
        "hydrate がハイドレーション属性からペイロード原文を復元すること（二重エスケープなし）"
    );

    let input = container
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist in the hydrated DOM");
    assert!(
        input.get_attribute("onmouseover").is_none(),
        "ハイドレーション経由でも属性注入によって onmouseover 属性が実 DOM に生成されてはならない"
    );
    assert_eq!(
        input.get_attribute("value").as_deref(),
        Some(payload),
        "value 属性の読み戻しがペイロード原文と一致すること（二重エスケープなし）"
    );
}

/// REQ-1 回帰（ハイドレーション経路・再描画後の実 DOM、TASK-11.4c・#84）:
/// `Runtime::hydrate` で XSS ペイロードを含む `draft` を復元した後、
/// `add_item` により `items` へ確定 → イベント配線経由で再描画された DOM でも
/// エスケープ保証が保たれること（`docs/hydration-state-format.md` 第 6 節
/// 「XSS 回帰」が要求する「再描画した DOM」での検証）。
#[wasm_bindgen_test]
fn hydrate_then_repaint_via_wired_event_preserves_escape_guarantee_for_xss_payload() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let payload = "<script>alert(1)</script>";
    let mut seed_state = AppState::new();
    assert!(dispatch(&mut seed_state, "set_draft", payload));

    let ssr_html = rws_core::render(&render_for_hydration(&seed_state));

    let container = create_container(&document, "xss-hydrate-repaint-root");
    let _cleanup = RemoveOnDrop(container.clone());
    container.set_inner_html(&ssr_html);

    let runtime = Runtime::hydrate("interactive-root", AppState::new())
        .expect("hydrate must succeed for well-formed SSR output containing an XSS payload");
    assert_eq!(runtime.component().draft, payload);

    // `add-btn`（data-action="add_item"）をクリックし、draft を items へ確定
    // させて再描画をトリガーする（events.rs: click は should_repaint=true）。
    let add_button = container
        .query_selector("[data-testid='add-btn']")
        .expect("query_selector must not fail")
        .expect("add-btn must exist in the hydrated DOM");
    add_button
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");

    assert!(
        runtime.component().items.iter().any(|item| item == payload),
        "add_item により draft のペイロードが items へ確定していること: {:?}",
        runtime.component().items
    );

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "再描画後も生の <script> 要素が実 DOM に生成されてはならない"
    );

    let items_root = container
        .query_selector("[data-testid='item-list']")
        .expect("query_selector must not fail")
        .expect("item-list container must exist after repaint");
    assert!(
        items_root
            .text_content()
            .unwrap_or_default()
            .contains(payload),
        "item-list 配下のテキストにペイロード原文が（エスケープ解除された形で）含まれること"
    );

    // `container.inner_html()` は `dom::paint` が上書きしないルート要素自身の
    // `data-hydrate-draft` 属性（SSR 時点でエスケープ済みのペイロードを保持
    // したまま残存する）を含むため、そこを見ると repaint が実際にエスケープ
    // したかどうかに関わらず判定が真になってしまう（Bugbot 指摘、repaint
    // パスの検証が空洞化する）。repaint で書き換えられる `item-list` 配下
    // （`items_root`）の `inner_html()` のみを見ることで、再描画されたテキスト
    // ノードのエスケープを実際に検証する。
    let items_inner = items_root.inner_html();
    assert!(
        !items_inner.contains("<script>alert(1)</script>"),
        "再描画後の item-list inner_html に生の <script> タグが含まれてはならない: {items_inner}"
    );
    assert!(
        items_inner.contains("&lt;script&gt;"),
        "再描画後の item-list inner_html にエスケープ済みペイロードが含まれること: {items_inner}"
    );
}
