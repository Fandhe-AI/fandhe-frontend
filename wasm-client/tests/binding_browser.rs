//! 束縛点ベースの最小更新（イシュー #343）の実ブラウザ受け入れテスト。
//!
//! `wasm-client/src/binding.rs`（DOM 非依存の純粋ロジック層）は
//! `wasm-client/tests/binding_logic.rs`（native）で検証済み。本ファイルは
//! `wasm-client/src/binding_dom.rs`（`BindingTable`）が実 DOM 上で正しく
//! 動作することを、イシュー #343 の受け入れ条件に対応付けて検証する。
//!
//! - 受け入れ条件 1: 変更フィールドに対応する束縛点のみが更新される
//!   （無関係ノードの DOM 変異がない） →
//!   [`only_dirty_field_binding_points_are_updated`]
//! - 受け入れ条件 2: テキスト更新が `set_text_content` 経由であること
//!   （innerHTML 不使用） →
//!   [`text_update_does_not_parse_payload_as_html`]
//!
//! 加えて、属性束縛・class 束縛・リスナー保持（DOM 再構築なしの実ブラウザ
//! 証跡）・fail-closed（未束縛 field・型不一致・改ざん相当のマーカー）を
//! `docs/design/dom-binding-update-design.md` §4・§9 の不変条件に対応付けて
//! 検証する。

#![cfg(target_arch = "wasm32")]

use rws_core::{bind_attr_token, bind_class_token, bind_text, el, render, text};
use rws_interactive::{dispatch, Component, DirtyTracked};
use rws_wasm_client::{BindingSource, BindingTable, BoundValue};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------
// テストフィクスチャ: counter（テキスト束縛）・draft（テキスト束縛）・
// liked（属性束縛 + class 束縛）を持つ最小コンポーネント。`rws_interactive`
// の `Component`/`DirtyTracked` と `rws_wasm_client` の `BindingSource` を
// すべて本テストクレート内のローカル型へ実装することで、orphan rule
// （外部 trait × 外部 type の impl 禁止）を回避しつつ、#343 が消費側
// （#345 の wasm-full 想定）に課す 3 trait 実装契約を実地で確認する。
// ---------------------------------------------------------------------

/// [`TestState::update`] が受理する型付きアクション。
enum TestAction {
    Increment,
    SetDraft(String),
    ToggleLiked,
}

/// counter（テキスト束縛）/draft（テキスト束縛）/liked（属性 + class 束縛）の
/// 3 フィールドを持つ、本テスト専用の最小状態コンポーネント。
struct TestState {
    counter: i64,
    draft: String,
    liked: bool,
    dirty: Vec<&'static str>,
}

impl TestState {
    const FIELD_COUNTER: &'static str = "counter";
    const FIELD_DRAFT: &'static str = "draft";
    const FIELD_LIKED: &'static str = "liked";

    fn new() -> Self {
        Self {
            counter: 0,
            draft: String::new(),
            liked: false,
            dirty: Vec::new(),
        }
    }
}

impl Component for TestState {
    type Action = TestAction;

    fn update(&mut self, action: Self::Action) {
        // DirtyTracked の契約: 呼び出し冒頭で前回分の記録をクリアする。
        self.dirty.clear();
        match action {
            TestAction::Increment => {
                self.counter += 1;
                self.dirty.push(Self::FIELD_COUNTER);
            }
            TestAction::SetDraft(value) => {
                self.draft = value;
                self.dirty.push(Self::FIELD_DRAFT);
            }
            TestAction::ToggleLiked => {
                self.liked = !self.liked;
                self.dirty.push(Self::FIELD_LIKED);
            }
        }
    }

    fn view(&self) -> rws_core::Node {
        // 本テストでは view() 自体は使わない（BindingTable は SSR 出力済み
        // DOM を対象に走査・適用するため）。Component の型契約を満たす
        // ための最小実装。
        text("unused")
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            "increment" => Some(TestAction::Increment),
            "set_draft" => Some(TestAction::SetDraft(payload.to_string())),
            "toggle_liked" => Some(TestAction::ToggleLiked),
            _ => None,
        }
    }
}

impl DirtyTracked for TestState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for TestState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "counter" => Some(BoundValue::Text(self.counter.to_string())),
            "draft" => Some(BoundValue::Text(self.draft.clone())),
            "liked" => Some(BoundValue::Flag(self.liked)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------
// DOM フィクスチャ構築ヘルパー
// ---------------------------------------------------------------------

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

/// counter/draft のテキスト束縛・liked の属性 + class 束縛・非束縛ノードを
/// 持つ SSR 出力相当のノード木を組み立てる（`rws_core::bind` ヘルパーの
/// 実利用、core が定める SSR 出力形式そのもの）。
fn fixture_tree() -> rws_core::Node {
    el(
        "div",
        vec![],
        vec![
            bind_text("span", vec![("id", "counter-node")], "counter", "0"),
            bind_text("span", vec![("id", "draft-node")], "draft", ""),
            el(
                "button",
                vec![
                    ("id", "like-btn"),
                    ("aria-pressed", "false"),
                    ("data-bind-attr", &bind_attr_token("aria-pressed", "liked")),
                    ("data-bind-class", &bind_class_token("liked", "liked")),
                ],
                vec![text("Like")],
            ),
            el(
                "span",
                vec![("id", "unrelated-node")],
                vec![text("unrelated")],
            ),
            el(
                "a",
                vec![
                    ("id", "draft-link"),
                    ("href", "/safe"),
                    ("data-bind-attr", &bind_attr_token("href", "draft")),
                ],
                vec![text("link")],
            ),
        ],
    )
}

fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

// ---------------------------------------------------------------------
// 受け入れ条件 1: 無関係ノードの DOM 変異ゼロ
// ---------------------------------------------------------------------

/// counter フィールドのみを dirty にして `apply_update` を適用したとき、
/// counter 束縛点のみが更新され、draft 束縛点・非束縛ノードの `outer_html`
/// がバイト一致で不変であること。
#[wasm_bindgen_test]
fn only_dirty_field_binding_points_are_updated() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-dirty-scope-root");
    root.set_inner_html(&render(&fixture_tree()));

    let draft_node = root
        .query_selector("#draft-node")
        .expect("query_selector must not fail")
        .expect("fixture must contain #draft-node");
    let unrelated_node = root
        .query_selector("#unrelated-node")
        .expect("query_selector must not fail")
        .expect("fixture must contain #unrelated-node");
    let draft_before = draft_node.outer_html();
    let unrelated_before = unrelated_node.outer_html();

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    let mut state = TestState::new();
    dispatch(&mut state, "increment", "");
    table.apply_update(&state);

    let counter_node = root
        .query_selector("#counter-node")
        .expect("query_selector must not fail")
        .expect("fixture must contain #counter-node");
    assert_eq!(
        counter_node.text_content().as_deref(),
        Some("1"),
        "dirty な counter 束縛点は更新されること"
    );

    assert_eq!(
        draft_node.outer_html(),
        draft_before,
        "counter のみが dirty のとき、無関係な draft 束縛点は変異しないこと（受け入れ条件 1）"
    );
    assert_eq!(
        unrelated_node.outer_html(),
        unrelated_before,
        "counter のみが dirty のとき、非束縛ノードは変異しないこと（受け入れ条件 1）"
    );
}

// ---------------------------------------------------------------------
// 受け入れ条件 2: set_text_content 経由（innerHTML 不使用）の回帰固定
// ---------------------------------------------------------------------

/// draft へ XSS ペイロードを設定して適用しても、束縛先要素に子要素
/// （`script`/`img` 等）が生成されず、ペイロードはテキストとしてのみ
/// 反映されること（`set_text_content` は DOM 仕様上 HTML として解釈しない）。
#[wasm_bindgen_test]
fn text_update_does_not_parse_payload_as_html() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-xss-root");
    root.set_inner_html(&render(&fixture_tree()));

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    let payload = "<script>alert('xss')</script><img src=x onerror=alert(1)>";
    let mut state = TestState::new();
    dispatch(&mut state, "set_draft", payload);
    table.apply_update(&state);

    let draft_node = root
        .query_selector("#draft-node")
        .expect("query_selector must not fail")
        .expect("fixture must contain #draft-node");

    assert_eq!(
        draft_node.text_content().as_deref(),
        Some(payload),
        "set_text_content で反映されたペイロードはテキストとして一致すること"
    );
    assert_eq!(
        draft_node.children().length(),
        0,
        "set_text_content 経由の更新では子要素（script/img 等）が生成されないこと（受け入れ条件 2）"
    );
    assert!(
        root.query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "root 配下に script 要素が生成されないこと"
    );
    assert!(
        root.query_selector("img")
            .expect("query_selector must not fail")
            .is_none(),
        "root 配下に img 要素が生成されないこと"
    );
}

// ---------------------------------------------------------------------
// 属性束縛・class 束縛
// ---------------------------------------------------------------------

/// liked フィールドの dirty 化で aria-pressed 属性が "true"/"false" へ
/// 反映され、liked class が状態値と一致してトグルされること（2 回適用しても
/// 冪等であること、`toggle_with_force` の性質）。
#[wasm_bindgen_test]
fn attr_and_class_bindings_reflect_state_and_are_idempotent() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-attr-class-root");
    root.set_inner_html(&render(&fixture_tree()));

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    let mut state = TestState::new();
    dispatch(&mut state, "toggle_liked", "");
    table.apply_update(&state);
    // 同一 dirty を 2 回適用しても副作用が変わらない（冪等性）ことを確認する。
    table.apply_update(&state);

    let button = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("fixture must contain #like-btn");

    assert_eq!(
        button.get_attribute("aria-pressed").as_deref(),
        Some("true"),
        "liked=true のとき aria-pressed 属性が \"true\" に更新されること"
    );
    assert!(
        button.class_list().contains("liked"),
        "liked=true のとき liked class が付与されること"
    );

    dispatch(&mut state, "toggle_liked", "");
    table.apply_update(&state);

    assert_eq!(
        button.get_attribute("aria-pressed").as_deref(),
        Some("false"),
        "liked=false のとき aria-pressed 属性が \"false\" に戻ること"
    );
    assert!(
        !button.class_list().contains("liked"),
        "liked=false のとき liked class が解除されること"
    );
}

// ---------------------------------------------------------------------
// リスナー保持（DOM 再構築なしの実ブラウザ証跡）
// ---------------------------------------------------------------------

/// `apply_update` の前後で束縛先要素の参照が同一ノードを指し続け
/// （`is_same_node`）、適用前に登録したクリックリスナーが適用後も発火する
/// こと（`BindingTable` が DOM 再構築系 API を呼ばない不変条件の実ブラウザ
/// 証跡、設計書 §9 不変条件 4）。
#[wasm_bindgen_test]
fn binding_apply_preserves_element_identity_and_existing_listeners() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-listener-preserve-root");
    root.set_inner_html(&render(&fixture_tree()));

    let button_before = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("fixture must contain #like-btn");

    let click_count = std::rc::Rc::new(std::cell::Cell::new(0));
    let click_count_for_closure = click_count.clone();
    let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        click_count_for_closure.set(click_count_for_closure.get() + 1);
    });
    button_before
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .expect("add_event_listener_with_callback must not fail");

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");
    let mut state = TestState::new();
    dispatch(&mut state, "toggle_liked", "");
    table.apply_update(&state);

    let button_after = root
        .query_selector("#like-btn")
        .expect("query_selector must not fail")
        .expect("#like-btn must still exist in the live DOM after apply_update");

    assert!(
        button_before.is_same_node(Some(&button_after)),
        "apply_update() は要素を再生成せず、適用前後で同一ノードを指すこと（DOM 再構築なし）"
    );

    button_after
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    assert_eq!(
        click_count.get(),
        1,
        "apply_update() の前に登録したリスナーが適用後も発火すること（リスナー保持）"
    );

    // Closure は本テストのローカル変数スコープが尽きるまで生存させる
    // （drop すると "closure invoked recursively or after being dropped" で
    // 異常終了するため）。
    closure.forget();
}

// ---------------------------------------------------------------------
// fail-closed: 未束縛 field・型不一致・改ざん相当のマーカー
// ---------------------------------------------------------------------

/// dirty に含まれない・`bound_value` が `None` を返す field は no-op であり
/// panic しないこと。
#[wasm_bindgen_test]
fn unbound_and_unknown_dirty_fields_are_noop_without_panic() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-unbound-field-root");
    root.set_inner_html(&render(&fixture_tree()));

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    /// `bound_value` が常に `None` を返す状態（未知 field 相当）。
    struct AlwaysNoneSource;
    impl BindingSource for AlwaysNoneSource {
        fn bound_value(&self, _field: &str) -> Option<BoundValue> {
            None
        }
    }

    // panic しないことを確認する（アサーションの対象は「クラッシュしない」こと自体）。
    table.apply_dirty(
        &["counter", "draft", "liked", "unknown-field"],
        &AlwaysNoneSource,
    );

    let counter_node = root
        .query_selector("#counter-node")
        .expect("query_selector must not fail")
        .expect("fixture must contain #counter-node");
    assert_eq!(
        counter_node.text_content().as_deref(),
        Some("0"),
        "bound_value が None を返す field は適用されず、初期値のまま残ること"
    );
}

/// DOM 改ざん相当のマーカー（`data-bind-attr="onclick:draft"`）を含む DOM に
/// 対して `scan`/`apply_update` を行っても、`onclick` 属性が生成されないこと
/// （`parse_binding_tokens` の fail-closed 検証、設計書 §9 不変条件 2 の
/// 消費側契約の実ブラウザ証跡）。
#[wasm_bindgen_test]
fn tampered_on_prefixed_attr_marker_is_rejected_and_produces_no_onclick_attribute() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-tampered-marker-root");
    let tampered = el(
        "div",
        vec![],
        vec![el(
            "button",
            vec![("id", "tampered-btn"), ("data-bind-attr", "onclick:draft")],
            vec![text("Tampered")],
        )],
    );
    root.set_inner_html(&render(&tampered));

    let table = BindingTable::scan(&root).expect("scan must succeed even for a tampered marker");

    let mut state = TestState::new();
    dispatch(&mut state, "set_draft", "alert(1)");
    table.apply_update(&state);

    let tampered_btn = root
        .query_selector("#tampered-btn")
        .expect("query_selector must not fail")
        .expect("fixture must contain #tampered-btn");
    assert!(
        tampered_btn.get_attribute("onclick").is_none(),
        "onclick 接頭辞の data-bind-attr トークンは拒否され、onclick 属性が生成されないこと（fail-closed）"
    );
}

// ---------------------------------------------------------------------
// URL スキーム検証（イシュー #373）: 実 DOM 属性更新経路の fail-closed
// ---------------------------------------------------------------------

/// `href` 属性へ束縛された field を危険スキーム（`javascript:`）に更新した
/// とき、`set_attribute` を素通りさせず、既存の安全な `href` 属性値を
/// `remove_attribute` で除去すること（`binding_dom.rs` の `apply_one` が
/// `rws_core::is_safe_url` を経由する契約の実ブラウザ証跡。
/// `docs/policy/attribute-output-policy.md` 参照）。
#[wasm_bindgen_test]
fn dangerous_url_scheme_bound_to_href_removes_the_attribute() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-url-scheme-root");
    root.set_inner_html(&render(&fixture_tree()));

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    let mut state = TestState::new();
    dispatch(&mut state, "set_draft", "javascript:alert(1)");
    table.apply_update(&state);

    let link = root
        .query_selector("#draft-link")
        .expect("query_selector must not fail")
        .expect("fixture must contain #draft-link");

    assert!(
        link.get_attribute("href").is_none(),
        "危険スキームの href 束縛値は set_attribute されず、既存値も remove_attribute で除去されること（fail-closed）"
    );
}

/// 安全な URL（相対 URL）へ更新した場合は従来どおり `href` が反映される
/// こと（過剰ブロックでないことの確認）。
#[wasm_bindgen_test]
fn safe_url_bound_to_href_is_applied_normally() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    let root = create_container(&document, "binding-url-safe-root");
    root.set_inner_html(&render(&fixture_tree()));

    let table = BindingTable::scan(&root).expect("scan must succeed for a well-formed fixture");

    let mut state = TestState::new();
    dispatch(&mut state, "set_draft", "/items/42");
    table.apply_update(&state);

    let link = root
        .query_selector("#draft-link")
        .expect("query_selector must not fail")
        .expect("fixture must contain #draft-link");

    assert_eq!(
        link.get_attribute("href").as_deref(),
        Some("/items/42"),
        "安全な相対 URL は href 束縛として正常に反映されること"
    );
}
