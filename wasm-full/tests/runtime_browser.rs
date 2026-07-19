//! `fandhe_frontend_wasm_full::Runtime` の実ブラウザ統合テスト（TASK-11.2d・#77、
//! `wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/runtime_headless.rs`（native）は
//! [`fandhe_frontend_wasm_full::dispatch_and_render_headless`]（DOM 非依存）までを
//! 検証済みである。本ファイルはその先、`Runtime::mount`/`Runtime::hydrate`
//! が実 DOM（headless Chromium）上で以下を満たすことを検証する
//! （`docs/design/wasm-full-architecture.md` 第 3.2 節・第 5 節、
//! イシュー #345 で束縛点更新 + keyed list 更新へ置き換え後の契約）。
//!
//! 1. `Runtime::mount` → クリック（`increment`/`add_item`）→ DOM 更新反映
//! 2. イベント後更新は束縛点更新（`set_text_content`/`set_attribute`）に
//!    限定され、無関係ノードの参照（`is_same_node`）・フォーカス・
//!    入力途中の値・スクロール位置が保持されること（#345 の実装動機。
//!    旧 `should_repaint`（`set_inner_html` 全置換回避のための再描画抑止）
//!    は不要になり撤去済み、`events.rs` doc 参照）
//! 3. `fandhe_frontend_interactive::render_for_hydration` 出力相当の DOM への
//!    `Runtime::hydrate` → 状態復元・イベント配線
//! 4. 改ざんされた `data-hydrate-*` 属性 → panic せず初期状態 CSR フォール
//!    バック（`docs/design/wasm-full-architecture.md` 第 4 節・判断 5）
//! 5. XSS ペイロードを持つ状態での `Runtime::mount`・keyed list への項目
//!    追加 → 実 DOM に `script` 要素が生成されないこと（REQ-1、
//!    `wasm-full/tests/xss_escape_wasm.rs` が検証する
//!    `render_component_html` 単体の保証を `Runtime::mount`/イベント後更新
//!    という製品経路まで通しで確認する）
//! 6. keyed list（`items`）への追加・削除で該当ノードのみが増減し、
//!    無関係ノードの参照が保持されること（#345 受け入れ条件、
//!    `fandhe_frontend_wasm_client::keyed_dom` の DOM 適用契約の統合確認）
//!
//! `AppState::view`（`interactive/src/lib.rs`）はルート要素へ固定 id
//! `interactive-root` を付与するため、本ファイルの `root_id` は一貫して
//! `"interactive-root"` を使う（`Runtime::hydrate` が読み取る
//! `data-hydrate-*` 属性は `render_for_hydration` によりこのルート要素自身へ
//! 付与される契約、`wasm-full/src/hydration.rs` 冒頭コメント参照）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_interactive::{AppState, Hydrate};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素（`Runtime::mount`/`hydrate` が対象とする
/// `root_id` 要素）を document body へ 1 個生成する。id を一意にすることで、
/// 同一テストバイナリ内の複数テストケースが要素を奪い合わないようにする
/// （`wasm-full/tests/xss_escape_wasm.rs::create_container` と同じ意図）。
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

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （CI issue #73 で顕在化したテスト間 DOM 汚染の再発防止）。
///
/// `wasm-pack test --headless --chrome` は本ファイル内の全 `#[wasm_bindgen_test]`
/// 関数を同一ページ（同一 `Document`）上で順に実行し、テスト間でページ
/// リロードやイフレーム分離を行わない。一方 `AppState::view()`
/// （`interactive/src/lib.rs`）が返すルート要素は常に固定 id
/// `"interactive-root"` を持つため、`Runtime::mount` で描画した内容を
/// 片付けずに残すと、後続テストの `document.get_element_by_id("interactive-root")`
/// （`Runtime::hydrate` 内部、`wasm-full/src/lib.rs::Runtime::get_root`）が
/// 意図せず過去のテストの残留要素にヒットし得る
/// （`hydrate_restores_state_from_existing_dom_and_wires_events` が
/// `mount_then_click_updates_state_and_dom`/`input_event_updates_state_without_repainting_dom`
/// の残留要素を拾って誤って CSR フォールバックする実際の不具合として観測
/// 済み。`Runtime::hydrate` 自体のロジックは単体では正しく、本ガードで
/// 再現条件を断つのが正しい修正）。
///
/// 各テストの `placeholder`（トップレベルコンテナ）自身を除去すれば、
/// その子孫として存在し得る `id="interactive-root"` の描画済み内容も
/// まとめて document から取り除かれる。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click`/`input` イベントを生成する（`bubbles: true`）。
///
/// `Runtime::mount`/`hydrate` はリスナーをルート要素へ登録するため、子要素上で
/// 発火したイベントがバブリングでルートまで届く必要がある
/// （`xss_escape_wasm.rs::bubbling_click_event` と同じ意図。`input` にも流用する）。
fn bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

/// `Runtime::mount` → クリックによる状態遷移・DOM 反映（観点 1）。
#[wasm_bindgen_test]
fn mount_then_click_updates_state_and_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-mount-click-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime =
        Runtime::mount("runtime-mount-click-root", AppState::new()).expect("mount must succeed");

    assert_eq!(runtime.component().counter, 0);
    assert!(
        placeholder
            .inner_html()
            .contains(r#"data-bind-text="counter">0</span>"#),
        "mount 直後に初期状態が DOM へ反映されていること: {}",
        placeholder.inner_html()
    );

    // `data-action="increment"` ボタン（`AppState::view` が
    // `data-testid="inc-btn"` として出力する）へ合成クリックを発火する。
    let button = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist after mount");
    button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().counter,
        1,
        "click 後に Runtime が保持する状態が更新されていること"
    );
    assert!(
        placeholder
            .inner_html()
            .contains(r#"data-bind-text="counter">1</span>"#),
        "click 後に DOM が再描画され最新の状態を反映していること: {}",
        placeholder.inner_html()
    );

    // add_item も同じ mount 経路で dispatch できることを確認する（複数
    // アクション種別が同一配線から扱えること）。
    let draft_input = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist");
    draft_input
        .dyn_ref::<HtmlInputElement>()
        .expect("draft-input must be an HtmlInputElement")
        .set_value("新しい項目");
    draft_input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");
    assert_eq!(runtime.component().draft, "新しい項目");

    let add_button = placeholder
        .query_selector("[data-testid='add-btn']")
        .expect("query_selector must not fail")
        .expect("add button must exist");
    add_button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert!(
        runtime
            .component()
            .items
            .iter()
            .any(|item| item == "新しい項目"),
        "add_item 後に items へ確定していること: {:?}",
        runtime.component().items
    );
    assert!(
        placeholder.inner_html().contains("新しい項目"),
        "add_item 後の DOM に新規項目が反映されていること: {}",
        placeholder.inner_html()
    );
}

/// `input` イベント後も要素参照（フォーカス・入力途中の値の保持の土台）が
/// 保たれること（観点 2、イシュー #345）。
///
/// 旧実装（`should_repaint: false`）は `set_inner_html` 全置換による
/// フォーカス破壊を避けるため、input イベント後の再描画そのものをスキップ
/// していた。#345 以降はイベント後更新が束縛点更新
/// （`set_text_content`/`set_attribute`、`draft-input` は
/// `HtmlInputElement::set_value` の等値ガード付き同期）に一本化され、
/// `set_inner_html` を一切呼ばない。本テストは
/// 「`draft-input` 要素自身が再生成されず（`is_same_node`）、無関係な兄弟
/// ノード（`inc-btn` 等）も同一参照のまま残ること」を確認し、
/// フォーカス・キャレット位置が破壊されないことの DOM 上の根拠とする。
#[wasm_bindgen_test]
fn input_event_preserves_element_identity_and_updates_state() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-mount-input-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime =
        Runtime::mount("runtime-mount-input-root", AppState::new()).expect("mount must succeed");

    let draft_input = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist");
    let inc_button_before = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist");

    draft_input
        .dyn_ref::<HtmlInputElement>()
        .expect("draft-input must be an HtmlInputElement")
        .set_value("入力中のテキスト");
    draft_input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().draft,
        "入力中のテキスト",
        "input イベントで内部状態（draft）が更新されること"
    );

    let draft_input_after = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must still exist");
    assert!(
        draft_input.is_same_node(Some(&draft_input_after)),
        "draft-input は再生成されず同一ノード参照が保持されること \
         （set_inner_html 全置換を経由しないことの DOM 上の根拠）"
    );
    assert_eq!(
        draft_input_after
            .dyn_ref::<HtmlInputElement>()
            .expect("must be HtmlInputElement")
            .value(),
        "入力中のテキスト",
        "現在値と一致する再同期は no-op だが、ブラウザ自身の入力反映により \
         表示値は入力内容のまま保たれること"
    );

    let inc_button_after = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist");
    assert!(
        inc_button_before.is_same_node(Some(&inc_button_after)),
        "draft の更新は無関係な inc-btn ノードへ触れないこと \
         （束縛点更新の対象限定、受け入れ条件: 無関係ノードの DOM 変異なし）"
    );
}

/// `add_item` 後、`draft-input` の live value プロパティが同期されクリア
/// されること（`set_attribute` だけでは live value が更新されない DOM
/// 仕様上の非対称性への対応、イシュー #345・
/// `docs/design/dom-binding-update-design.md` #345 実装確定節）。
#[wasm_bindgen_test]
fn add_item_clears_draft_input_live_value_via_property_sync() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-add-item-clears-value-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("runtime-add-item-clears-value-root", AppState::new())
        .expect("mount must succeed");

    let draft_input = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist");
    let input_el = draft_input
        .dyn_ref::<HtmlInputElement>()
        .expect("draft-input must be an HtmlInputElement");
    input_el.set_value("新規タスク");
    draft_input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");
    assert_eq!(input_el.value(), "新規タスク");

    let add_button = placeholder
        .query_selector("[data-testid='add-btn']")
        .expect("query_selector must not fail")
        .expect("add button must exist");
    add_button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(runtime.component().draft, "");
    assert_eq!(
        input_el.value(),
        "",
        "add_item 後は live value プロパティも同期されクリアされること \
         （set_attribute のみでは live value に反映されない DOM 仕様の対応）"
    );
}

/// `Runtime::hydrate`: SSR 済み DOM 相当（`render_for_hydration` が
/// ルート要素へ付与する `data-hydrate-*` 属性を持つ既存 DOM）から状態を
/// 復元し、イベント配線も成立すること（観点 3）。
#[wasm_bindgen_test]
fn hydrate_restores_state_from_existing_dom_and_wires_events() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    // SSR 相当の初期状態・DOM を用意する: AppState を直接 render し、その
    // ルート要素（id="interactive-root"、AppState::view 参照）へ
    // hydration_attrs を付与した上で、プレースホルダの子として配置する
    // （`render_for_hydration` が行う「view() の root へ hydration_attrs を
    // 後付けする」処理を、実 DOM 属性として直接再現する）。
    let seed_state = AppState::new();
    let mut seed_state = seed_state;
    seed_state.counter = 5;
    seed_state.items.push("SSR済み項目".to_string());

    let placeholder = create_placeholder(&document, "interactive-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());
    placeholder.set_inner_html(&fandhe_frontend_wasm_full::render_component_html(
        &seed_state,
    ));
    // render_component_html の出力は AppState::view() のルート div
    // （id="interactive-root"）そのものであり、`interactive-root` という id を
    // 持つプレースホルダの子孫として重複生成される。hydrate() が読み取る
    // `root`（get_element_by_id("interactive-root") が最初にヒットする要素）は
    // このケースではプレースホルダ自身になるため、hydration 属性は
    // プレースホルダに直接付与する。
    for (name, value) in seed_state.hydration_attrs() {
        placeholder
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let runtime = Runtime::hydrate("interactive-root", AppState::new())
        .expect("hydrate must succeed for well-formed attrs");

    assert_eq!(
        runtime.component().counter,
        5,
        "hydrate が data-hydrate-counter から状態を復元すること"
    );
    assert!(
        runtime
            .component()
            .items
            .iter()
            .any(|item| item == "SSR済み項目"),
        "hydrate が data-hydrate-items から状態を復元すること: {:?}",
        runtime.component().items
    );

    // イベント配線が成立していること: increment 相当のボタンをクリックして
    // 状態が復元値から遷移すること（`data-testid='inc-btn'` は
    // set_inner_html されたペイロード内、または hydrate 成功時は既存 DOM の
    // 子孫としてそのまま存在する）。
    let button = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist in hydrated DOM");
    button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().counter,
        6,
        "hydrate 後もイベント配線経由で dispatch が行われること"
    );
}

/// 改ざんされた `data-hydrate-*` 属性（数値パース不能な counter）は panic
/// せず、初期状態での CSR 再描画へフォールバックすること（観点 4、
/// `docs/design/wasm-full-architecture.md` 第 4 節・判断 5）。
#[wasm_bindgen_test]
fn hydrate_falls_back_to_initial_state_csr_on_corrupted_attrs() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-hydrate-corrupted-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    // 改ざん想定: counter に数値パース不能な値を設定する。
    placeholder
        .set_attribute("data-hydrate-counter", "not-a-number")
        .expect("set_attribute must not fail");
    placeholder
        .set_attribute("data-hydrate-draft", "")
        .expect("set_attribute must not fail");
    placeholder
        .set_attribute("data-hydrate-items", "")
        .expect("set_attribute must not fail");

    let runtime = Runtime::hydrate("runtime-hydrate-corrupted-root", AppState::new())
        .expect("hydrate must not error (panic-free fallback, returns Ok with initial state)");

    assert_eq!(
        runtime.component().counter,
        0,
        "改ざん属性は復元せず初期状態（counter=0）へフォールバックすること"
    );
    assert!(
        placeholder
            .inner_html()
            .contains(r#"data-bind-text="counter">0</span>"#),
        "フォールバック時は初期状態で CSR 再描画されること: {}",
        placeholder.inner_html()
    );

    // フォールバック後もイベント配線が成立していること。
    let button = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist after fallback repaint");
    button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");
    assert_eq!(runtime.component().counter, 1);
}

/// REQ-1 回帰: XSS ペイロードを持つ状態で `Runtime::mount` しても、実 DOM に
/// `script` 要素が生成されないこと（観点 5。`xss_escape_wasm.rs` は
/// `render_component_html` を手動で `set_inner_html` する経路を検証済みだが、
/// 本テストは `Runtime::mount` という製品 API そのものを経由する点が異なる）。
#[wasm_bindgen_test]
fn mount_with_xss_payload_state_produces_no_script_element_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-mount-xss-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let mut state = AppState::new();
    state.draft = "<script>alert(1)</script>".to_string();
    state.items.push("<script>alert(2)</script>".to_string());
    // `items` への直接代入は `item_ids`（keyed list の安定キー、イシュー
    // #345）を追随させないため、ここで揃える（`interactive` クレートの
    // `AppState::item_ids` 型ドキュメント参照。揃えないと `view()` の
    // `zip` により 2 件目の項目が描画から欠落し、本来検証したい
    // 「keyed list 経由の挿入項目の XSS」を確認できなくなる）。
    state.item_ids = (0..state.items.len() as u64).collect();

    let _runtime = Runtime::mount("runtime-mount-xss-root", state).expect("mount must succeed");

    assert!(
        placeholder
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "生の <script> 要素が Runtime::mount 経由でも実 DOM に生成されてはならない"
    );
    let inner = placeholder.inner_html();
    assert!(
        !inner.contains("<script>"),
        "inner_html に生の <script> タグが含まれてはならない: {inner}"
    );
    assert!(
        inner.contains("&lt;script&gt;"),
        "inner_html にエスケープ済みペイロードが含まれること: {inner}"
    );
}

/// 観点 6: `add_item`（keyed list への挿入）で既存項目ノードの参照が保持され
/// つつ、新規項目のみが末尾へ追加されること（設計書 §5.3「既存ノード参照を
/// 保持したまま」の統合確認）。
#[wasm_bindgen_test]
fn add_item_appends_new_node_without_recreating_existing_list_items() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-keyed-insert-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime =
        Runtime::mount("runtime-keyed-insert-root", AppState::new()).expect("mount must succeed");

    let list = placeholder
        .query_selector("[data-testid='item-list']")
        .expect("query_selector must not fail")
        .expect("item list must exist");
    assert_eq!(list.children().length(), 1, "初期状態は項目 1 件");
    let existing_first_item = list.children().item(0).expect("first item must exist");

    let draft_input = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist");
    draft_input
        .dyn_ref::<HtmlInputElement>()
        .expect("must be HtmlInputElement")
        .set_value("2 件目の項目");
    draft_input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");
    placeholder
        .query_selector("[data-testid='add-btn']")
        .expect("query_selector must not fail")
        .expect("add button must exist")
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(runtime.component().items.len(), 2);
    assert_eq!(
        list.children().length(),
        2,
        "keyed list への挿入で子要素が 1 件増えること"
    );
    let first_item_after = list.children().item(0).expect("first item must exist");
    assert!(
        existing_first_item.is_same_node(Some(&first_item_after)),
        "既存の 1 件目ノードは再生成されず同一参照が保持されること \
         （keyed list 挿入が set_inner_html 全置換を経由しないことの根拠）"
    );
    let second_item = list.children().item(1).expect("second item must exist");
    assert!(
        second_item
            .text_content()
            .unwrap_or_default()
            .contains("2 件目の項目"),
        "新規項目が末尾へ追加されること"
    );
}

/// 観点 6: `remove_item`（keyed list からの削除）で対象項目のみが除去され、
/// 無関係な項目ノードの参照が保持されること。
#[wasm_bindgen_test]
fn remove_item_removes_target_node_preserving_sibling_identity() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-keyed-remove-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let mut state = AppState::new();
    state.items.push("2 件目".to_string());
    state.items.push("3 件目".to_string());
    state.item_ids = (0..state.items.len() as u64).collect();
    state.next_item_id = state.item_ids.len() as u64;

    let runtime = Runtime::mount("runtime-keyed-remove-root", state).expect("mount must succeed");

    let list = placeholder
        .query_selector("[data-testid='item-list']")
        .expect("query_selector must not fail")
        .expect("item list must exist");
    assert_eq!(list.children().length(), 3);
    let third_item_before = list.children().item(2).expect("third item must exist");

    // 中間（2 件目、id=1）の削除ボタンをクリックする。
    let remove_buttons = list
        .query_selector_all("[data-testid='remove-btn']")
        .expect("query_selector_all must not fail");
    let middle_remove_button = remove_buttons
        .get(1)
        .expect("middle remove button must exist");
    middle_remove_button
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().items,
        vec!["最初の項目".to_string(), "3 件目".to_string()],
        "id=1（2 件目）のみが削除されること"
    );
    assert_eq!(list.children().length(), 2);
    let third_item_after = list.children().item(1).expect("remaining second item");
    assert!(
        third_item_before.is_same_node(Some(&third_item_after)),
        "削除対象以外のノード（3 件目）は再生成されず同一参照が保持されること"
    );
}

/// 観点 5・6 の複合: keyed list への挿入経路（`add_item`）で script 相当の
/// 文字列を追加しても `<script>` 要素が生成されず、`innerHTML` を経由しない
/// プログラム的構築（`create_element`/`set_text_content`）でテキストとして
/// 安全に挿入されること（`wasm-client::keyed_dom` の不変条件の統合確認）。
#[wasm_bindgen_test]
fn add_item_with_script_payload_inserts_as_text_not_script_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "runtime-keyed-insert-xss-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("runtime-keyed-insert-xss-root", AppState::new())
        .expect("mount must succeed");

    let malicious = "<script>alert(1)</script>";
    let draft_input = placeholder
        .query_selector("#draft-input")
        .expect("query_selector must not fail")
        .expect("draft-input must exist");
    draft_input
        .dyn_ref::<HtmlInputElement>()
        .expect("must be HtmlInputElement")
        .set_value(malicious);
    draft_input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");
    placeholder
        .query_selector("[data-testid='add-btn']")
        .expect("query_selector must not fail")
        .expect("add button must exist")
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert!(runtime
        .component()
        .items
        .iter()
        .any(|item| item == malicious));
    assert!(
        placeholder
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "keyed list への挿入経路でも <script> 要素が実 DOM に生成されないこと"
    );
    let list = placeholder
        .query_selector("[data-testid='item-list']")
        .expect("query_selector must not fail")
        .expect("item list must exist");
    assert!(
        list.text_content().unwrap_or_default().contains(malicious),
        "script 文字列はテキストとして安全に挿入されること"
    );
}
