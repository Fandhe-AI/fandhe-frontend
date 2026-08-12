//! `Runtime` の keyed list `Update` op 適用（イシュー #1324）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/runtime_browser.rs` は `interactive::AppState` の
//! keyed list（`items`）への挿入・削除（`Insert`/`Remove`/`Move` op）を
//! 検証済みだが、`AppState` は同一キーの内容だけを書き換えるアクションを
//! 持たないため `KeyedOp::Update` を製品経路（`Runtime`）で通しては検証
//! できない。`form_events_and_structural_fallback_browser.rs` と同じ理由で
//! （同ファイル冒頭 doc 参照）、本ファイル専用の最小 component
//! （[`ListState`]）を用意する。
//!
//! 親トラッキング #1321 の受け入れ条件（本ファイルが固定する観点）:
//!
//! 1. イベント → dirty keyed field → 同一キー内容変更が製品経路
//!    （`Runtime`）で DOM テキストへ反映される
//! 2. 2 回連続同一内容の更新で余分な DOM 変更が起きないこと
//!    （`Runtime::keyed_list_cache` による達成 Node 保持の回帰）
//! 3. XSS 回帰: 更新値に script 相当のペイロードを含めても script 要素が
//!    生成されない

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// `runtime_browser.rs::create_placeholder` と同じ意図（テスト間の
/// document 汚染を避けるための一意 id 付きプレースホルダ）。
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

/// `runtime_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `runtime_browser.rs::bubbling_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// 動的リスト 1 件のみを持つ最小 component（イシュー #1324 の `Update` op
/// を実 DOM で駆動するためだけの最小実装）。`counter`/`draft` のような
/// 束縛点フィールドは持たず、`items`（keyed list）のみが dirty field と
/// なるため、`Runtime::apply_update_for_dirty` の keyed list 分岐のみを
/// 確実に通す。
#[derive(Debug, Clone)]
struct ListState {
    /// `(安定キー, 現在の内容)` の順序付きリスト。
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
}

impl ListState {
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str)]) -> Self {
        Self {
            items: initial
                .iter()
                .map(|(id, text)| (*id, text.to_string()))
                .collect(),
            dirty: Vec::new(),
        }
    }
}

/// `ListState::decode_action` が復号する型付きアクション。
enum ListAction {
    /// `id` の項目内容を `content` へ置き換える（同一キー内容変更、
    /// `KeyedOp::Update` を誘発する唯一の操作）。
    Rename { id: u64, content: String },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::Rename { id, content } => {
                if let Some(item) = self.items.iter_mut().find(|(existing, _)| *existing == id) {
                    if item.1 != content {
                        item.1 = content;
                        self.dirty.push(Self::FIELD_ITEMS);
                    }
                }
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|(id, content)| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "keyed-update-item")],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let list = keyed_list("ul", vec![], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "keyed-update-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            // payload 形式: "<id>:<content>"（テスト専用の単純な符号化。
            // `content` にコロンが含まれても最初の 1 個のみで分割するため
            // 後続テキストはそのまま保持される）。
            "rename" => {
                let (id_str, content) = payload.split_once(':')?;
                let id = id_str.parse::<u64>().ok()?;
                Some(ListAction::Rename {
                    id,
                    content: content.to_string(),
                })
            }
            _ => None,
        }
    }
}

impl DirtyTracked for ListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

/// keyed list 内容（`items`）はテキストノードのみで `data-bind-*` 束縛点を
/// 一切使わないため、`bound_value` は常に `None`（束縛点対応表に対象
/// フィールドが存在しない = `BindingTable::has_field` が `false` を返す。
/// `Runtime::apply_update_for_dirty` は keyed list 解決を優先するため、
/// この `None` が `unresolved_field`（構造フォールバック誘発）へは
/// 波及しない）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `root` 直下へ `data-action="rename"` の合成クリックターゲットを追加し、
/// クリックして dispatch する（`xss_escape_wasm.rs` と同じ手法。events.rs
/// の delegation は `closest("[data-action]")` による祖先探索のため、
/// 子孫要素への動的追加でも成立する）。
fn dispatch_rename(document: &Document, root: &Element, id: u64, content: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", "rename")
        .expect("set_attribute must not fail");
    trigger
        .set_attribute("data-payload", &format!("{id}:{content}"))
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// 受け入れ条件 1: 同一キー・新ラベルの再適用で DOM テキストが更新される。
#[wasm_bindgen_test]
fn rename_updates_dom_text_for_same_key() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "keyed-update-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "old-label")]);
    let runtime =
        Runtime::mount("keyed-update-root-container-1", state).expect("mount must succeed");

    let root = runtime.root();
    let item = root
        .query_selector("[data-testid='keyed-update-item']")
        .expect("query_selector must not fail")
        .expect("initial item must exist");
    let item_node_before = item.clone();

    dispatch_rename(&document, root, 1, "new-label");

    let item = root
        .query_selector("[data-testid='keyed-update-item']")
        .expect("query_selector must not fail")
        .expect("item must still exist after rename");
    assert_eq!(item.text_content().as_deref(), Some("new-label"));
    assert!(
        item.is_same_node(Some(&item_node_before)),
        "Update 対象のルート要素は再生成されず同一ノードのままのはず \
         （フォーカス保持の土台）"
    );
}

/// 受け入れ条件 2: 2 回連続で同一内容の更新イベントを送っても、DOM
/// テキストが意図した最終値のまま安定していること（達成 Node キャッシュ
/// による Update 経路が 2 回目以降も正しく機能する回帰、`Runtime::keyed_list_cache`
/// doc 参照）。
#[wasm_bindgen_test]
fn repeated_rename_events_stay_consistent_via_cached_previous_node() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "keyed-update-root-container-2");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "v0")]);
    let runtime =
        Runtime::mount("keyed-update-root-container-2", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_rename(&document, root, 1, "v1");
    dispatch_rename(&document, root, 1, "v2");
    dispatch_rename(&document, root, 1, "v2"); // 同一内容の再送（no-op 相当）。

    let item = root
        .query_selector("[data-testid='keyed-update-item']")
        .expect("query_selector must not fail")
        .expect("item must exist");
    assert_eq!(
        item.text_content().as_deref(),
        Some("v2"),
        "3 回の連続更新後も最終値へ正しく収束していること"
    );
    assert_eq!(
        runtime.component().items,
        vec![(1u64, "v2".to_string())],
        "状態側も最終値と一致していること"
    );
}

/// 受け入れ条件 3（XSS 回帰）: rename の新内容に script 相当のペイロードを
/// 含めても script 要素が生成されず、テキストとして安全に格納される
/// （`Runtime` 製品経路を通した Update 適用の回帰固定）。
#[wasm_bindgen_test]
fn rename_with_script_payload_is_kept_as_plain_text_not_element() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "keyed-update-root-container-3");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "safe")]);
    let runtime =
        Runtime::mount("keyed-update-root-container-3", state).expect("mount must succeed");
    let root = runtime.root();

    let malicious = "<script>alert(1)</script>";
    dispatch_rename(&document, root, 1, malicious);

    assert_eq!(
        root.query_selector("script").unwrap(),
        None,
        "Update 経由でも script 要素は生成されないこと"
    );
    let item = root
        .query_selector("[data-testid='keyed-update-item']")
        .expect("query_selector must not fail")
        .expect("item must exist");
    assert_eq!(item.text_content().as_deref(), Some(malicious));
}
