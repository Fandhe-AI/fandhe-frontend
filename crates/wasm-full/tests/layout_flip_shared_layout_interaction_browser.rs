//! `layout_flip`（keyed list の並べ替え FLIP）と `shared_layout`（別要素
//! への引き継ぎ FLIP）が同一更新内で共存する際の相互作用を検証する実
//! ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! Bugbot 指摘（イシュー #2578「FLIP lists skip nested shared layout」）:
//! `Runtime::apply_update_for_dirty` は `layout_flip::play_after` が
//! transform を適用した行要素を `shared_layout::play_after_excluding`
//! から除外するが、旧実装は除外対象を keyed list コンテナ（`<ul
//! data-fandhe-flip-auto>` 自体）にしていたため、同一 keyed list 内に
//! 新規挿入された行の中の `data-fandhe-layout-id` 要素までコンテナ丸ごと
//! 除外に巻き込まれ、共有レイアウト遷移を一度も受け取れなかった。本
//! テストは、新規挿入行の中へ移動してくる `data-fandhe-layout-id` 要素
//! （detail view から一覧へ戻るカードの再現）が、同じ更新内で
//! `FLIP_AUTO_ATTR` リストの構造変化が起きていても正しく共有レイアウト
//! 遷移を受け取ることを確認する。

#![cfg(target_arch = "wasm32")]
// `layout_flip`/`shared_layout` はいずれも `layout-animation` feature
// （既定 on）配下（`animate_browser.rs` と同型のゲート方針）。
#![cfg(feature = "layout-animation")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::layout_flip::FLIP_AUTO_ATTR;
use fandhe_frontend_wasm_full::shared_layout::LAYOUT_ID_ATTR;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// `layout_flip_browser.rs::create_placeholder` と同じ意図。
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

/// `layout_flip_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `layout_flip_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// `layout_flip_browser.rs::dispatch_action` と同じ手法。
fn dispatch_action(document: &Document, root: &Element, action: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", action)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

fn badge_transform(root: &Element) -> String {
    root.query_selector("[data-testid='badge']")
        .expect("query_selector must not fail")
        .expect("badge must exist")
        .dyn_into::<HtmlElement>()
        .expect("badge must be an HtmlElement")
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail")
}

/// `flip-list`（[`FLIP_AUTO_ATTR`] 付き keyed list）の外側に配置された
/// `data-fandhe-layout-id="badge"` の span が、更新後は新規挿入された
/// 行（keyed list の末尾行）の中へ「別要素への引き継ぎ」で移動する最小
/// component（detail view から一覧カードへ戻るケースの再現、Bugbot 指摘
/// の再現条件）。
#[derive(Debug, Clone)]
struct BadgeListState {
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
}

impl BadgeListState {
    const FIELD_ITEMS: &'static str = "items";
    const FIELD_BADGES: &'static str = "badges";

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

enum BadgeListAction {
    /// バッジをリスト外の待避位置から、新規挿入行の中へ移す。
    MoveBadgeIntoNewRow,
}

impl Component for BadgeListState {
    type Action = BadgeListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            BadgeListAction::MoveBadgeIntoNewRow => {
                self.items.push((3, "c".to_string()));
                self.dirty.push(Self::FIELD_ITEMS);
                // 待避位置の旧バッジは `badges` keyed list（空になる）経由で
                // 除去する。`Runtime::apply_update_for_dirty` は dirty field
                // に対応する束縛点・keyed list しか更新しないため、`items`
                // だけを dirty にすると旧バッジが DOM に接続されたまま残り、
                // `shared_layout::pair_by_id` の「旧要素がまだ接続中なら
                // 曖昧として対象外」の fail-safe で遷移が起きない。
                self.dirty.push(Self::FIELD_BADGES);
            }
        }
    }

    fn view(&self) -> Node {
        let badge_in_list = self.items.len() >= 3;
        let last_index = self.items.len().saturating_sub(1);
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, (id, content))| {
                let style = format!("position:absolute;top:{}px;left:0px;", index * 50);
                let mut children = vec![text(content)];
                if badge_in_list && index == last_index {
                    // 新規挿入行の中へバッジを引き継がせる（`before` 側
                    // では別ロケーション、`after` 側では本行の子として
                    // 同じ `LAYOUT_ID_ATTR` を持つ新規ノードになる）。
                    children.push(el(
                        "span",
                        vec![
                            ("data-testid", "badge"),
                            (LAYOUT_ID_ATTR, "badge"),
                            (
                                "style",
                                "position:absolute;left:0px;top:0px;width:20px;height:20px;",
                            ),
                        ],
                        vec![],
                    ));
                }
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "flip-item"), ("style", style.as_str())],
                        children,
                    ),
                )
            })
            .collect();
        let list = keyed_list(
            "ul",
            vec![("id", "flip-list"), (FLIP_AUTO_ATTR, "")],
            "items",
            items,
        )
        .expect("test fixture keyed items must be valid");
        // 待避位置のバッジは独立した keyed list（`FLIP_AUTO_ATTR` なし）
        // に置き、更新時は空リストへの差分として除去する（`update` の
        // コメント参照）。
        let parked: Vec<(String, Node)> = if badge_in_list {
            Vec::new()
        } else {
            vec![(
                "badge".to_string(),
                el(
                    "span",
                    vec![
                        ("data-testid", "badge"),
                        (LAYOUT_ID_ATTR, "badge"),
                        (
                            "style",
                            "position:absolute;left:300px;top:300px;width:20px;height:20px;",
                        ),
                    ],
                    vec![],
                ),
            )]
        };
        let badges = keyed_list("div", vec![("id", "badge-slot")], "badges", parked)
            .expect("test fixture keyed badges must be valid");
        el("div", vec![("id", "flip-root")], vec![list, badges])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "move_badge_into_new_row" => Some(BadgeListAction::MoveBadgeIntoNewRow),
            _ => None,
        }
    }
}

impl DirtyTracked for BadgeListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for BadgeListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// 受け入れ条件（Bugbot 指摘是正、イシュー #2578「FLIP lists skip
/// nested shared layout」）: `FLIP_AUTO_ATTR` リストへ新規行が挿入される
/// 更新（keyed list の構造変化。同時に `layout_flip::play_after` が
/// 呼ばれる）の中で、その新規行へ引き継がれる `data-fandhe-layout-id`
/// 要素は、共有レイアウト遷移から取りこぼされず補正 `transform` を
/// 受け取る。
#[wasm_bindgen_test]
fn newly_inserted_row_carries_shared_layout_element_despite_list_flip() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-shared-layout-root");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = BadgeListState::new(&[(1, "a"), (2, "b")]);
    let runtime = Runtime::mount("flip-shared-layout-root", state).expect("mount must succeed");
    let root = runtime.root();

    assert!(
        badge_transform(root).is_empty(),
        "初期表示のバッジには補正 transform がないはず"
    );

    dispatch_action(&document, root, "move_badge_into_new_row");

    assert!(
        !badge_transform(root).is_empty(),
        "新規挿入行へ引き継がれたバッジは、同一更新内で keyed list の \
         構造変化（layout FLIP）が起きていても共有レイアウト遷移の \
         補正 transform を受け取っているはず（旧実装は keyed list \
         コンテナ全体を除外対象にしていたため、この transform が \
         一切書き込まれなかった）"
    );
}
