//! `fandhe_frontend_wasm_full::questionnaire`（イシュー #2118、親 #2117）の
//! 実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/questionnaire.rs` の native `#[cfg(test)] mod tests`
//! は純粋ロジック層とヘッドレス出力のドリフト検知までを検証済みである。
//! 本ファイルはその先、配線層（`wiring`、`#[cfg(target_arch = "wasm32")]`）
//! が実 DOM（headless Chromium）上で
//!
//! 1. next クリック → `data-step`/question 各要素の `data-state`・`hidden`・
//!    progress の `aria-valuenow`/`aria-valuetext`・back の `disabled` 除去・
//!    `"questionnaire:next"` 通知
//! 2. prev クリックで戻り、back の `disabled` が再付与されること
//! 3. skip クリック → `"questionnaire:skip"` 通知
//! 4. 完了到達（`data-complete`・next/skip の `disabled` 付与）と、そこから
//!    back で完了解除（`disabled` 除去）
//! 5. 完了状態での no-op next（before == after）で DOM 不変・通知なし
//! 6. `data-disabled`/ネイティブ `disabled` 祖先を持つ click の no-op
//! 7. button 内テキストノードを target にした click が trigger へ解決
//!    されること
//! 8. 改ざん入力（非数値 `data-step`・範囲外 `data-step`・非数値
//!    `data-index`）に対する fail-closed 挙動
//! 9. 同一 container 内の 2 インスタンスの独立性
//! 10. `Runtime::hydrate` 統合（`headless_timer_browser.rs`
//!     `runtime_dirty_rerender` と同型）: `questionnaire:next` 通知を
//!     `C` が受理して束縛点を再描画すること
//!
//! を検証する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::questionnaire::{QuestionProps, Questionnaire};
use fandhe_frontend_wasm_full::events::ActionRef;
use fandhe_frontend_wasm_full::questionnaire::wire_questionnaire_events;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`headless_timer_browser.rs::create_container` と同型）。
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

/// `crates/headless-ui/src/questionnaire.rs` の SSR 出力契約そのもの
/// （root > progress + question(0..count) + actions(back/next/skip)）で
/// Questionnaire のマークアップを組み立て、`html` へ追記する
/// （複数インスタンス共存テストのため文字列を返す設計）。
fn questionnaire_markup(q: &Questionnaire, id_prefix: &str) -> String {
    let mut questions = Vec::new();
    for index in 0..q.count() {
        questions.push(q.question(index, QuestionProps::default(), Vec::new(), Vec::new()));
    }
    let node = q.root(
        vec![("id", id_prefix)],
        vec![
            q.progress("", Vec::new(), Vec::new()),
            q.actions(
                Vec::new(),
                vec![
                    q.back(false, Vec::new(), Vec::new()),
                    q.next(false, Vec::new(), Vec::new()),
                    q.skip(false, Vec::new(), Vec::new()),
                ],
            ),
        ]
        .into_iter()
        .chain(questions)
        .collect(),
    );
    render(&node)
}

fn root_element(container: &Element, id_prefix: &str) -> Element {
    container
        .query_selector(&format!("#{id_prefix}"))
        .expect("query_selector must not fail")
        .expect("questionnaire root must exist")
}

fn part_element(root: &Element, part: &str) -> Element {
    root.query_selector(&format!(
        r#"[data-scope="questionnaire"][data-part="{part}"]"#
    ))
    .expect("query_selector must not fail")
    .unwrap_or_else(|| panic!("{part} part must exist"))
}

fn question_element(root: &Element, index: usize) -> Element {
    root.query_selector(&format!(
        r#"[data-scope="questionnaire"][data-part="question"][data-index="{index}"]"#
    ))
    .expect("query_selector must not fail")
    .unwrap_or_else(|| panic!("question {index} must exist"))
}

/// 合成 `click`（bubbles: true、通常のユーザークリックを模す）を生成する
/// （`headless_timer_browser.rs::synthetic_click` と同型）。
fn synthetic_click() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// `condition` が真になるまでポーリングする
/// （`headless_timer_browser.rs::wait_for` と同型）。
async fn wait_for(desc: &str, mut condition: impl FnMut() -> bool) {
    for _ in 0..500 {
        if condition() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
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
    panic!("wait_for timed out after 500 polls (~5s): {desc}");
}

/// `setTimeout(0)` を 1 回発行して microtask/次 tick をまたぐ、条件
/// ポーリングを伴わない単純な猶予待機（no-op click の DOM 不変確認用）。
async fn settle() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                0,
            )
            .expect("setTimeout must not fail");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");
}

// --- 検証: next クリック ---------------------------------------------

#[wasm_bindgen_test]
async fn next_click_advances_step_and_updates_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-next-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-next"));
    let root = root_element(&container, "qn-next");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-step to become 1 after next click", || {
        root.get_attribute("data-step").as_deref() == Some("1")
    })
    .await;

    assert_eq!(
        question_element(&root, 0)
            .get_attribute("data-state")
            .as_deref(),
        Some("completed")
    );
    assert!(question_element(&root, 0).has_attribute("hidden"));
    assert_eq!(
        question_element(&root, 1)
            .get_attribute("data-state")
            .as_deref(),
        Some("active")
    );
    assert!(!question_element(&root, 1).has_attribute("hidden"));
    assert_eq!(
        question_element(&root, 2)
            .get_attribute("data-state")
            .as_deref(),
        Some("upcoming")
    );
    assert!(question_element(&root, 2).has_attribute("hidden"));

    let progress = part_element(&root, "progress");
    assert_eq!(
        progress.get_attribute("aria-valuenow").as_deref(),
        Some("33")
    );
    assert_eq!(
        progress.get_attribute("aria-valuetext").as_deref(),
        Some("33% complete")
    );

    let back = part_element(&root, "back");
    assert!(!back.has_attribute("disabled"));
    assert!(!back.has_attribute("data-disabled"));

    assert_eq!(actions.borrow().len(), 1);
    assert_eq!(actions.borrow()[0].action, "questionnaire:next");
    // payload は "{遷移前の step}|{instance root の id}"（イシュー #2118
    // PR #2286 codex-review P1 指摘: 複数インスタンス識別）。
    assert_eq!(actions.borrow()[0].payload, "0|qn-next");
}

// --- 検証: prev クリックで back が再度 disabled になること ------------

#[wasm_bindgen_test]
async fn prev_click_retreats_step_and_redisables_back() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-prev-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 1, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-prev"));
    let root = root_element(&container, "qn-prev");

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    part_element(&root, "back")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-step to become 0 after prev click", || {
        root.get_attribute("data-step").as_deref() == Some("0")
    })
    .await;

    assert_eq!(
        question_element(&root, 0)
            .get_attribute("data-state")
            .as_deref(),
        Some("active")
    );
    let back = part_element(&root, "back");
    assert!(back.has_attribute("disabled"));
    assert!(back.has_attribute("data-disabled"));
}

// --- 検証: skip クリック ------------------------------------------------

#[wasm_bindgen_test]
async fn skip_click_advances_step_and_notifies_skip_action() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-skip-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 1, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-skip"));
    let root = root_element(&container, "qn-skip");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    part_element(&root, "skip")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-step to become 2 after skip click", || {
        root.get_attribute("data-step").as_deref() == Some("2")
    })
    .await;

    assert_eq!(actions.borrow().len(), 1);
    assert_eq!(actions.borrow()[0].action, "questionnaire:skip");
    assert_eq!(actions.borrow()[0].payload, "1|qn-skip");
}

// --- 検証: 完了到達と、そこからの back による完了解除 -------------------

#[wasm_bindgen_test]
async fn completion_marks_data_complete_and_back_reverts_it() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-complete-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 2, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-complete"));
    let root = root_element(&container, "qn-complete");

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-step to become 3 after final next click", || {
        root.get_attribute("data-step").as_deref() == Some("3")
    })
    .await;

    assert!(root.has_attribute("data-complete"));
    let progress = part_element(&root, "progress");
    assert!(progress.has_attribute("data-complete"));
    assert_eq!(
        progress.get_attribute("aria-valuenow").as_deref(),
        Some("100")
    );

    let next = part_element(&root, "next");
    let skip = part_element(&root, "skip");
    assert!(next.has_attribute("disabled"));
    assert!(next.has_attribute("data-disabled"));
    assert!(skip.has_attribute("disabled"));
    assert!(skip.has_attribute("data-disabled"));

    for index in 0..3 {
        assert!(question_element(&root, index).has_attribute("hidden"));
    }

    // back で完了解除。
    part_element(&root, "back")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-complete to be removed after back click", || {
        !root.has_attribute("data-complete")
    })
    .await;

    assert!(!part_element(&root, "next").has_attribute("disabled"));
    assert!(!part_element(&root, "skip").has_attribute("disabled"));
    assert_eq!(
        question_element(&root, 2)
            .get_attribute("data-state")
            .as_deref(),
        Some("active")
    );
}

// --- 検証: 完了状態での no-op next（境界での no-op） -------------------

#[wasm_bindgen_test]
async fn next_click_at_completion_is_noop_and_emits_no_notification() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-noop-completion-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(2, 2, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-noop-completion"));
    let root = root_element(&container, "qn-noop-completion");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    // 完了状態の next は SSR 時点で disabled/data-disabled 済みだが、合成
    // click がブラウザの disabled 抑止に依存せず本モジュール側で no-op に
    // なることを確認するため、明示的に外してから click する。
    let next = part_element(&root, "next");
    next.remove_attribute("disabled")
        .expect("remove_attribute must not fail");
    next.remove_attribute("data-disabled")
        .expect("remove_attribute must not fail");

    next.dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(root.get_attribute("data-step").as_deref(), Some("2"));
    assert!(root.has_attribute("data-complete"));
    assert_eq!(actions.borrow().len(), 0);
}

// --- 検証: disabled 祖先を持つ click は no-op ---------------------------

#[wasm_bindgen_test]
async fn click_with_data_disabled_ancestor_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-disabled-ancestor-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-disabled-ancestor"));
    let root = root_element(&container, "qn-disabled-ancestor");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    root.set_attribute("data-disabled", "")
        .expect("set_attribute must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(root.get_attribute("data-step").as_deref(), Some("0"));
    assert_eq!(actions.borrow().len(), 0);
}

#[wasm_bindgen_test]
async fn click_on_natively_disabled_trigger_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-native-disabled-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // step 0 の back は SSR 時点でネイティブ `disabled` を持つ。
    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-native-disabled"));
    let root = root_element(&container, "qn-native-disabled");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    part_element(&root, "back")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(root.get_attribute("data-step").as_deref(), Some("0"));
    assert_eq!(actions.borrow().len(), 0);
}

// --- 検証: button 内テキストノードを target にした click ---------------

#[wasm_bindgen_test]
async fn click_on_text_node_inside_trigger_resolves_to_trigger() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-text-node-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(2, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-text-node"));
    let root = root_element(&container, "qn-text-node");

    let next = part_element(&root, "next");
    next.set_text_content(Some("Next"));

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    let text_node = next
        .first_child()
        .expect("next trigger must have a text node child");
    text_node
        .dyn_ref::<web_sys::EventTarget>()
        .expect("text node must be a valid EventTarget")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("data-step to become 1 via text node click", || {
        root.get_attribute("data-step").as_deref() == Some("1")
    })
    .await;
}

// --- 検証: 改ざん入力に対する fail-closed 挙動 --------------------------

#[wasm_bindgen_test]
async fn tampered_non_numeric_step_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-tampered-step-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-tampered-step"));
    let root = root_element(&container, "qn-tampered-step");

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    root.set_attribute("data-step", "abc<script>")
        .expect("set_attribute must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(
        root.get_attribute("data-step").as_deref(),
        Some("abc<script>")
    );
    assert!(container.query_selector("script").unwrap().is_none());
}

#[wasm_bindgen_test]
async fn tampered_step_greater_than_count_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-tampered-overflow-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-tampered-overflow"));
    let root = root_element(&container, "qn-tampered-overflow");

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    root.set_attribute("data-step", "9")
        .expect("set_attribute must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    settle().await;

    assert_eq!(root.get_attribute("data-step").as_deref(), Some("9"));
}

#[wasm_bindgen_test]
async fn tampered_question_index_is_skipped_but_others_update() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-tampered-index-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q = Questionnaire::new(3, 0, Orientation::Horizontal);
    container.set_inner_html(&questionnaire_markup(&q, "qn-tampered-index"));
    let root = root_element(&container, "qn-tampered-index");

    // 1 つの question の data-index を改ざんする（XSS ペイロードを含む
    // 非数値値。パースに失敗しその要素だけスキップされることを確認する）。
    question_element(&root, 1)
        .set_attribute("data-index", "x<script>")
        .expect("set_attribute must not fail");

    wire_questionnaire_events(container.clone(), |_action_ref| {})
        .expect("wire_questionnaire_events must not fail");

    part_element(&root, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for(
        "data-step to become 1 despite one tampered question",
        || root.get_attribute("data-step").as_deref() == Some("1"),
    )
    .await;

    // index 0（正常な question）は更新される。
    assert_eq!(
        question_element(&root, 0)
            .get_attribute("data-state")
            .as_deref(),
        Some("completed")
    );
    // 改ざんされた要素は data-index 経由で見つからなくなるため、素の
    // querySelector で個別に確認する（更新されず放置されているはず）。
    let tampered = root
        .query_selector(
            r#"[data-scope="questionnaire"][data-part="question"][data-index="x<script>"]"#,
        )
        .expect("query_selector must not fail")
        .expect("tampered question element must still exist untouched");
    assert!(tampered.has_attribute("hidden"));
    assert!(container.query_selector("script").unwrap().is_none());
}

// --- 検証: 同一 container 内の 2 インスタンスの独立性 -------------------

#[wasm_bindgen_test]
async fn two_instances_in_same_container_are_independent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "questionnaire-two-instances-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let q_a = Questionnaire::new(2, 0, Orientation::Horizontal);
    let q_b = Questionnaire::new(2, 0, Orientation::Horizontal);
    let html = format!(
        "{}{}",
        questionnaire_markup(&q_a, "qn-instance-a"),
        questionnaire_markup(&q_b, "qn-instance-b")
    );
    container.set_inner_html(&html);
    let root_a = root_element(&container, "qn-instance-a");
    let root_b = root_element(&container, "qn-instance-b");

    let actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ActionRef>::new()));
    let actions_clone = actions.clone();
    wire_questionnaire_events(container.clone(), move |action_ref| {
        actions_clone.borrow_mut().push(action_ref);
    })
    .expect("wire_questionnaire_events must not fail");

    part_element(&root_a, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("instance A data-step to become 1", || {
        root_a.get_attribute("data-step").as_deref() == Some("1")
    })
    .await;

    assert_eq!(root_b.get_attribute("data-step").as_deref(), Some("0"));

    part_element(&root_b, "next")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");

    wait_for("instance B data-step to become 1", || {
        root_b.get_attribute("data-step").as_deref() == Some("1")
    })
    .await;

    // 通知 payload のインスタンス識別子（`instance root` の `id`）が
    // クリックしたインスタンスごとに異なることを確認する（イシュー #2118
    // PR #2286 codex-review P1 指摘: 通知だけではどのインスタンスの
    // 遷移か判別できなかった不具合の回帰防止）。
    assert_eq!(actions.borrow().len(), 2);
    assert_eq!(actions.borrow()[0].payload, "0|qn-instance-a");
    assert_eq!(actions.borrow()[1].payload, "0|qn-instance-b");
}

// --- 検証: `Runtime::hydrate` 統合（`questionnaire:next` 通知後の束縛点
// 再描画、`headless_timer_browser.rs::runtime_dirty_rerender` と同型） ---

mod runtime_dirty_rerender {
    use super::{synthetic_click, wait_for, RemoveOnDrop};
    use fandhe_frontend_core::{bind_text, render, Node};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;
    use fandhe_frontend_headless_ui::questionnaire::{
        QuestionProps, Questionnaire, QuestionnaireAction,
    };
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen_test::*;
    use web_sys::{Document, Element};

    /// `Runtime::wire_questionnaire` の dispatch 後再描画接続を実 DOM で
    /// 固定するための最小ホスト。`Questionnaire` をラップし、アプリ側の
    /// 派生フィールド（step ラベル）を dirty へ積む
    /// （`headless_timer_browser.rs::runtime_dirty_rerender::TimerHost`
    /// と同じ設計）。
    struct QuestionnaireHost {
        questionnaire: Questionnaire,
        step_label: String,
        dirty: Vec<&'static str>,
        root_id: String,
    }

    impl QuestionnaireHost {
        fn new(questionnaire: Questionnaire) -> Self {
            let step_label = questionnaire.step().to_string();
            Self {
                questionnaire,
                step_label,
                dirty: Vec::new(),
                root_id: String::new(),
            }
        }

        fn with_root_id(mut self, root_id: &str) -> Self {
            self.root_id = root_id.to_string();
            self
        }
    }

    impl Component for QuestionnaireHost {
        type Action = QuestionnaireAction;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            self.questionnaire.update(action);
            let new_step_label = self.questionnaire.step().to_string();
            if new_step_label != self.step_label {
                self.step_label = new_step_label;
                self.dirty.push("step_label");
            }
        }

        fn view(&self) -> Node {
            let mut questions = Vec::new();
            for index in 0..self.questionnaire.count() {
                questions.push(self.questionnaire.question(
                    index,
                    QuestionProps::default(),
                    Vec::new(),
                    Vec::new(),
                ));
            }
            self.questionnaire.root(
                vec![("id", self.root_id.as_str())],
                vec![
                    self.questionnaire.progress("", Vec::new(), Vec::new()),
                    self.questionnaire.actions(
                        Vec::new(),
                        vec![
                            self.questionnaire.back(false, Vec::new(), Vec::new()),
                            self.questionnaire.next(false, Vec::new(), Vec::new()),
                            self.questionnaire.skip(false, Vec::new(), Vec::new()),
                        ],
                    ),
                    bind_text(
                        "span",
                        vec![("data-testid", "step-label")],
                        "step_label",
                        self.step_label.clone(),
                    ),
                ]
                .into_iter()
                .chain(questions)
                .collect(),
            )
        }

        fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
            // `questionnaire::wiring::notify_action` は `C` 側の名前空間衝突
            // 回避のため `"questionnaire:"` で修飾したアクション名
            // （`ACTION_PREV`/`ACTION_NEXT`/`ACTION_SKIP`）を通知する
            // （questionnaire.rs モジュール冒頭「`headless::MAPPING_TABLE` へ
            // 登録しない理由」節参照）。一方 `Questionnaire::decode_action`
            // 自身は他コンポーネントからも同じ語彙で再利用可能であることを
            // 意図し、修飾なしの `"prev"`/`"next"`/`"skip"` のみを受理する
            // （wasm-full 内部の DOM 追跡用エフェメラル `Questionnaire` の
            // dispatch も同じ修飾なし語彙を使う、questionnaire.rs
            // `handle_click` 参照）。このためアプリ側 `Component` が
            // `Questionnaire::decode_action` へそのまま委譲する構成では、
            // 通知された修飾済みアクション名を委譲前に剥がす必要がある
            // （Bugbot/cursor 指摘、イシュー #2118 PR #2286 レビュー）。
            let name = name.strip_prefix("questionnaire:").unwrap_or(name);
            Questionnaire::decode_action(name, payload)
        }
    }

    impl DirtyTracked for QuestionnaireHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    impl BindingSource for QuestionnaireHost {
        fn bound_value(&self, field: &str) -> Option<BoundValue> {
            match field {
                "step_label" => Some(BoundValue::Text(self.step_label.clone())),
                _ => None,
            }
        }
    }

    impl Hydrate for QuestionnaireHost {
        fn hydration_attrs(&self) -> Vec<(String, String)> {
            self.questionnaire.hydration_attrs()
        }

        fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
            Questionnaire::from_hydration_attrs(attrs).map(Self::new)
        }
    }

    fn mount_host_and_hydrate(
        document: &Document,
        root_id: &str,
        questionnaire: Questionnaire,
    ) -> (Element, Runtime<QuestionnaireHost>) {
        let host = QuestionnaireHost::new(questionnaire).with_root_id(root_id);
        let html = render(&host.view());
        document
            .body()
            .expect("document body must exist in browser test environment")
            .insert_adjacent_html("beforeend", &html)
            .expect("insert_adjacent_html must not fail");
        let root_el: Element = document
            .get_element_by_id(root_id)
            .expect("rendered Questionnaire root must have the expected id");

        for (name, value) in host.hydration_attrs() {
            root_el
                .set_attribute(&name, &value)
                .expect("set_attribute must not fail");
        }

        let runtime = Runtime::hydrate(root_id, QuestionnaireHost::new(questionnaire))
            .expect("hydrate must succeed for well-formed attrs");
        assert_eq!(
            runtime.root().id(),
            root_id,
            "hydrate は root_id 要素自身を Questionnaire root として復元すること"
        );

        (root_el, runtime)
    }

    #[wasm_bindgen_test]
    async fn runtime_hydrate_click_rerenders_host_binding_and_writes_data_step_once() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");

        let (root_el, _runtime) = mount_host_and_hydrate(
            &document,
            "questionnaire-host-runtime-dirty-rerender-root",
            Questionnaire::new(3, 0, Orientation::Horizontal),
        );
        let _cleanup = RemoveOnDrop(root_el.clone());

        let step_label = || {
            root_el
                .query_selector("[data-bind-text='step_label']")
                .expect("query_selector must not fail")
                .expect("step_label binding point must exist")
                .text_content()
                .unwrap_or_default()
        };
        assert_eq!(step_label(), "0");

        let next_button = root_el
            .query_selector(r#"[data-scope="questionnaire"][data-part="next"]"#)
            .expect("query_selector must not fail")
            .expect("next trigger must exist");
        next_button
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");

        wait_for("data-step becomes 1 after next click", || {
            root_el.get_attribute("data-step").as_deref() == Some("1")
        })
        .await;

        wait_for(
            "step_label binding reflects the new step after dispatch",
            || step_label() == "1",
        )
        .await;
    }
}
