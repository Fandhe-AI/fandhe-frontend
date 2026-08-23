//! `KeyedListDom::sync_attrs` の同値スキップ（イシュー #1382）に対する
//! codex-review P0 回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `keyed_update_browser.rs` が同一キー内容変更（`KeyedOp::Update`）の
//! テキスト反映を検証済みだが、`sync_attrs` の属性同値スキップが
//! **URL 属性（`href` 等）を除外していること**は未検証だった。本ファイルは
//! その回帰を固定する: 外部コード（テストコード自身が模倣）がライブ DOM の
//! `href` を直接 `javascript:` スキームへ書き換えても、次の `Update` op
//! （`old_attrs`（キャッシュ）・`new_attrs`（新 view）の `href` 値は変わらず
//! 安全なまま）が同じ tick 内で安全値を書き戻し、危険なライブ値を残存
//! させないことを確認する。
//!
//! `crates/wasm-client/src/keyed_dom.rs::sync_attrs`（イベントハンドラ / URL /
//! `srcset` の 3 カテゴリを同値スキップの対象外とする実装）の doc「同値
//! スキップ」節・`crates/wasm-client/src/keyed_apply.rs`（トレイト doc
//! 「同値属性のスキップ許容」節）と対をなす。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// `keyed_update_browser.rs::create_placeholder` と同じ意図。
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

/// `keyed_update_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `keyed_update_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// keyed list 1 件のみを持つ最小 component。各アイテムは `text`（keyed
/// diff を確実に発火させる可変フィールド）と `href`（`URL_ATTRS` 対象の
/// 属性。テストシナリオでは値そのものは変わらず安全なまま据え置き、同値
/// スキップの適用可否のみを検証する）を持つ。
#[derive(Debug, Clone)]
struct ListState {
    /// `(安定キー, テキスト内容, href 属性値)` の順序付きリスト。
    items: Vec<(u64, String, String)>,
    dirty: Vec<&'static str>,
}

impl ListState {
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str, &str)]) -> Self {
        Self {
            items: initial
                .iter()
                .map(|(id, text, href)| (*id, text.to_string(), href.to_string()))
                .collect(),
            dirty: Vec::new(),
        }
    }
}

/// `ListState::decode_action` が復号する型付きアクション。
enum ListAction {
    /// `id` の項目テキストを `content` へ置き換える（`href` は変更しない
    /// ため、`sync_attrs` の set ループでは `href` が `old_attrs` と
    /// バイト等価のまま `KeyedOp::Update` が発火する）。
    Retext { id: u64, content: String },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::Retext { id, content } => {
                if let Some(item) = self
                    .items
                    .iter_mut()
                    .find(|(existing, _, _)| *existing == id)
                {
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
            .map(|(id, content, href)| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![
                            ("data-testid", "keyed-attr-drift-item"),
                            ("href", href.as_str()),
                        ],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let list = fandhe_frontend_core::keyed::keyed_list("ul", vec![], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "keyed-attr-drift-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            // payload 形式: "<id>:<content>"。
            "retext" => {
                let (id_str, content) = payload.split_once(':')?;
                let id = id_str.parse::<u64>().ok()?;
                Some(ListAction::Retext {
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

/// `keyed_update_browser.rs::ListState` と同じ意図（`href` は
/// `data-bind-*` 束縛点を使わず keyed diff 経由でのみ反映されるため常に
/// `None`）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `keyed_update_browser.rs::dispatch_rename` と同じ手法。
fn dispatch_retext(document: &Document, root: &Element, id: u64, content: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", "retext")
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

/// codex-review P0（イシュー #1382）の再現手順そのもの:
/// 1. `href="/safe"` の項目をマウントする（`keyed_list_cache` の achieved
///    Node に `href="/safe"` がキャッシュされる）。
/// 2. テストコードが外部スクリプトを模倣し、ライブ DOM の `href` を直接
///    `javascript:alert(1)` へ書き換える（`old_attrs` キャッシュ・次
///    tick の `new_attrs` はいずれも変化しない、ライブ値だけのドリフト）。
/// 3. `retext` でテキストのみを変更し `KeyedOp::Update` を発火させる
///    （`href` は新 view でも `/safe` のまま = `old_attrs` と同値）。
///
/// 修正前（`href` も同値スキップ対象だった実装）では、手順 3 の
/// `sync_attrs` が `attr_value_unchanged(old_attrs, "href", "/safe")` を
/// `true` と判定して `set_attribute` を省略し、手順 2 で書き換えられた
/// `javascript:alert(1)` がライブ DOM に残存したまま Update 後も直らな
/// かった。修正後は URL 属性が同値スキップの対象外のため、手順 3 で
/// `href` の検証・書き込みが必ず実行され、安全値 `/safe` へ同一 tick で
/// 復元される。
#[wasm_bindgen_test]
fn external_href_drift_to_javascript_scheme_is_healed_within_same_update_tick() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "keyed-attr-drift-root-container");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "old-label", "/safe")]);
    let runtime =
        Runtime::mount("keyed-attr-drift-root-container", state).expect("mount must succeed");
    let root = runtime.root();

    let item = root
        .query_selector("[data-testid='keyed-attr-drift-item']")
        .expect("query_selector must not fail")
        .expect("item must exist after mount");
    assert_eq!(
        item.get_attribute("href").as_deref(),
        Some("/safe"),
        "mount 直後は安全な href がそのまま反映されているはず"
    );

    // 手順 2: 外部スクリプトによるライブ DOM 直接書き換えを模倣する
    // （`old_attrs` キャッシュ・次 tick の `new_attrs` は一切変化しない）。
    item.set_attribute("href", "javascript:alert(1)")
        .expect("set_attribute must not fail for the drift simulation");
    assert_eq!(
        item.get_attribute("href").as_deref(),
        Some("javascript:alert(1)"),
        "ドリフト直後は危険なライブ値が一時的に存在する（テスト前提の確認）"
    );

    // 手順 3: href は変えず（新 view でも "/safe"）、テキストのみを変更して
    // KeyedOp::Update を発火させる。
    dispatch_retext(&document, root, 1, "new-label");

    let item_after_update = root
        .query_selector("[data-testid='keyed-attr-drift-item']")
        .expect("query_selector must not fail")
        .expect("item must still exist after update");
    assert_eq!(
        item_after_update.text_content().as_deref(),
        Some("new-label"),
        "Update op によりテキストは新しい内容へ反映されているはず"
    );
    assert_eq!(
        item_after_update.get_attribute("href").as_deref(),
        Some("/safe"),
        "URL 属性は同値スキップの対象外のため、外部ドリフトした危険な \
         href は同一 tick で安全値へ復元されているはず（codex-review P0 \
         回帰、イシュー #1382）"
    );
}
