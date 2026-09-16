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
use fandhe_frontend_core::{bind_attr_tokens, el, text, Node};
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

/// 入れ子 keyed list（外側・内側とも [`FLIP_AUTO_ATTR`]）の外側行の中へ
/// `data-fandhe-layout-id="badge"` の span が待避位置から引き継がれ、その
/// 後に**内側リストだけ**が dirty になる更新が続く component（Cursor
/// Bugbot 指摘「Ancestor FLIP lists skip stop scope」の再現条件）。
#[derive(Debug, Clone)]
struct NestedBadgeState {
    outer: Vec<u64>,
    inner: Vec<u64>,
    badge_in_outer: bool,
    dirty: Vec<&'static str>,
}

impl NestedBadgeState {
    const FIELD_OUTER: &'static str = "outer";
    const FIELD_INNER: &'static str = "inner";
    const FIELD_BADGES: &'static str = "badges";

    fn new() -> Self {
        Self {
            outer: vec![1, 2],
            inner: vec![10, 20],
            badge_in_outer: false,
            dirty: Vec::new(),
        }
    }
}

enum NestedBadgeAction {
    /// バッジを待避位置から外側行（`id == 2`、内側リストを含まない
    /// 行）の中へ移す（dirty: `outer` + `badges`）。
    MoveBadgeIntoOuterRow,
    /// 内側リストのみ並べ替える（dirty: `inner` のみ。外側リストは dirty
    /// ではないが、内側の祖先として `flip_lists_containing` で捕捉され
    /// list FLIP の対象になる）。
    SwapInnerOnly,
}

impl Component for NestedBadgeState {
    type Action = NestedBadgeAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            NestedBadgeAction::MoveBadgeIntoOuterRow => {
                self.badge_in_outer = true;
                self.dirty.push(Self::FIELD_OUTER);
                self.dirty.push(Self::FIELD_BADGES);
            }
            NestedBadgeAction::SwapInnerOnly => {
                let len = self.inner.len();
                if len >= 2 {
                    self.inner.swap(0, len - 1);
                }
                self.dirty.push(Self::FIELD_INNER);
            }
        }
    }

    fn view(&self) -> Node {
        let badge = |style: &str| {
            el(
                "span",
                vec![
                    ("data-testid", "badge"),
                    (LAYOUT_ID_ATTR, "badge"),
                    ("style", style),
                ],
                vec![],
            )
        };
        let outer_items: Vec<(String, Node)> = self
            .outer
            .iter()
            .map(|id| {
                let node = if *id == 1 {
                    let inner_items: Vec<(String, Node)> = self
                        .inner
                        .iter()
                        .map(|inner_id| {
                            (
                                inner_id.to_string(),
                                el(
                                    "li",
                                    vec![("data-testid", "inner-item")],
                                    vec![text("inner row")],
                                ),
                            )
                        })
                        .collect();
                    let inner_list = keyed_list(
                        "ul",
                        vec![("data-testid", "inner-list"), (FLIP_AUTO_ATTR, "")],
                        Self::FIELD_INNER,
                        inner_items,
                    )
                    .expect("test fixture keyed items must be valid");
                    el("li", vec![("data-testid", "outer-item")], vec![inner_list])
                } else {
                    let mut children = vec![text("plain outer row")];
                    if self.badge_in_outer {
                        children.push(badge(
                            "position:absolute;left:0px;top:0px;width:20px;height:20px;",
                        ));
                    }
                    el(
                        "li",
                        vec![
                            ("data-testid", "outer-item"),
                            ("style", "position:relative;"),
                        ],
                        children,
                    )
                };
                (id.to_string(), node)
            })
            .collect();
        let outer_list = keyed_list(
            "ul",
            vec![("id", "outer-list"), (FLIP_AUTO_ATTR, "")],
            Self::FIELD_OUTER,
            outer_items,
        )
        .expect("test fixture keyed items must be valid");
        let parked: Vec<(String, Node)> = if self.badge_in_outer {
            Vec::new()
        } else {
            vec![(
                "badge".to_string(),
                badge("position:absolute;left:300px;top:300px;width:20px;height:20px;"),
            )]
        };
        let badges = keyed_list(
            "div",
            vec![("id", "badge-slot")],
            Self::FIELD_BADGES,
            parked,
        )
        .expect("test fixture keyed badges must be valid");
        el("div", vec![("id", "flip-root")], vec![outer_list, badges])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "move_badge_into_outer_row" => Some(NestedBadgeAction::MoveBadgeIntoOuterRow),
            "swap_inner_only" => Some(NestedBadgeAction::SwapInnerOnly),
            _ => None,
        }
    }
}

impl DirtyTracked for NestedBadgeState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for NestedBadgeState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// 受け入れ条件（Cursor Bugbot 指摘是正、イシュー #2578「Ancestor FLIP
/// lists skip stop scope」）: 外側行の中で共有レイアウト遷移が進行中の
/// とき、内側リストだけが dirty になる更新でも、その祖先である外側
/// リスト（list FLIP が transform を書き込み得る範囲）配下の進行中
/// ハンドルは DOM 更新前に停止・復元される（旧実装は dirty field 自身の
/// keyed list と束縛先しか scope にせず、外側行のハンドルが残った）。
#[wasm_bindgen_test]
fn inner_only_update_stops_in_progress_shared_layout_in_ancestor_flip_row() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "nested-flip-shared-layout-root");
    let _guard = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("nested-flip-shared-layout-root", NestedBadgeState::new())
        .expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "move_badge_into_outer_row");
    assert!(
        !badge_transform(root).is_empty(),
        "外側行へ引き継がれたバッジは共有レイアウト遷移の補正 transform を \
         受け取り進行中のはず（前提条件）"
    );

    dispatch_action(&document, root, "swap_inner_only");
    assert!(
        badge_transform(root).is_empty(),
        "内側リストのみの更新でも、祖先の FLIP リスト配下にある進行中の \
         共有レイアウト遷移は DOM 更新前に停止・復元されているはず \
         （旧実装は外側行のハンドルを停止せず transform が残った）"
    );
}

/// フラット（入れ子でない）な [`FLIP_AUTO_ATTR`] keyed list の行の中に、
/// 待避位置から引き継がれる `data-fandhe-layout-id="badge"` の span と、
/// 別 field `label` へ束縛された `data-bind-attr` の span が同居する
/// component（Cursor Bugbot 指摘「Stop scope includes non-playing lists」
/// の再現条件: `label` のみの更新は走査 (ii) でこのリストを捕捉するが、
/// `chain.len() == 1` のため list FLIP は再生しない）。
#[derive(Debug, Clone)]
struct FlatSideState {
    badge_in_row: bool,
    label: String,
    dirty: Vec<&'static str>,
}

impl FlatSideState {
    const FIELD_SIDE: &'static str = "side";
    const FIELD_BADGES: &'static str = "badges";
    const FIELD_LABEL: &'static str = "label";

    fn new() -> Self {
        Self {
            badge_in_row: false,
            label: "before".to_string(),
            dirty: Vec::new(),
        }
    }
}

enum FlatSideAction {
    /// バッジを待避位置からフラットリストの行の中へ移す（dirty: `side` +
    /// `badges`）。
    MoveBadgeIntoRow,
    /// 束縛値 `label` だけを更新する（dirty: `label` のみ。keyed list の
    /// 構造変化なし）。
    BumpLabel,
}

impl Component for FlatSideState {
    type Action = FlatSideAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            FlatSideAction::MoveBadgeIntoRow => {
                self.badge_in_row = true;
                self.dirty.push(Self::FIELD_SIDE);
                self.dirty.push(Self::FIELD_BADGES);
            }
            FlatSideAction::BumpLabel => {
                self.label = "after".to_string();
                self.dirty.push(Self::FIELD_LABEL);
            }
        }
    }

    fn view(&self) -> Node {
        let badge = |style: &str| {
            el(
                "span",
                vec![
                    ("data-testid", "badge"),
                    (LAYOUT_ID_ATTR, "badge"),
                    ("style", style),
                ],
                vec![],
            )
        };
        let bind_attr = bind_attr_tokens(&[("title", Self::FIELD_LABEL)]);
        let mut row_children = vec![el(
            "span",
            vec![
                ("data-testid", "label"),
                ("data-bind-attr", bind_attr.as_str()),
            ],
            vec![text("label")],
        )];
        if self.badge_in_row {
            row_children.push(badge(
                "position:absolute;left:0px;top:0px;width:20px;height:20px;",
            ));
        }
        let side = keyed_list(
            "ul",
            vec![("id", "side-list"), (FLIP_AUTO_ATTR, "")],
            Self::FIELD_SIDE,
            vec![
                (
                    "1".to_string(),
                    el(
                        "li",
                        vec![
                            ("data-testid", "side-item"),
                            ("style", "position:relative;"),
                        ],
                        row_children,
                    ),
                ),
                (
                    "2".to_string(),
                    el("li", vec![("data-testid", "side-item")], vec![text("b")]),
                ),
            ],
        )
        .expect("test fixture keyed items must be valid");
        let parked: Vec<(String, Node)> = if self.badge_in_row {
            Vec::new()
        } else {
            vec![(
                "badge".to_string(),
                badge("position:absolute;left:300px;top:300px;width:20px;height:20px;"),
            )]
        };
        let badges = keyed_list(
            "div",
            vec![("id", "badge-slot")],
            Self::FIELD_BADGES,
            parked,
        )
        .expect("test fixture keyed badges must be valid");
        el("div", vec![("id", "flip-root")], vec![side, badges])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "move_badge_into_row" => Some(FlatSideAction::MoveBadgeIntoRow),
            "bump_label" => Some(FlatSideAction::BumpLabel),
            _ => None,
        }
    }
}

impl DirtyTracked for FlatSideState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for FlatSideState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            Self::FIELD_LABEL => Some(BoundValue::Text(self.label.clone())),
            _ => None,
        }
    }
}

/// 受け入れ条件（Cursor Bugbot 指摘是正、イシュー #2578「Stop scope
/// includes non-playing lists」）: フラットな `FLIP_AUTO_ATTR` リストの
/// 行で共有レイアウト遷移が進行中のとき、同じ行内の別要素へ束縛された
/// field だけを更新しても（走査 (ii) はこのリストを捕捉するが list FLIP
/// は再生しない）、進行中ハンドルは停止されず補正 transform が残る。
/// 束縛値の反映自体は行われる。
#[wasm_bindgen_test]
fn binding_only_update_keeps_in_progress_shared_layout_in_non_playing_flat_list() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flat-side-shared-layout-root");
    let _guard = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("flat-side-shared-layout-root", FlatSideState::new())
        .expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "move_badge_into_row");
    assert!(
        !badge_transform(root).is_empty(),
        "行へ引き継がれたバッジは共有レイアウト遷移の補正 transform を \
         受け取り進行中のはず（前提条件）"
    );

    dispatch_action(&document, root, "bump_label");
    let label = root
        .query_selector("[data-testid='label']")
        .expect("query_selector must not fail")
        .expect("label must exist");
    assert_eq!(
        label.get_attribute("title").as_deref(),
        Some("after"),
        "束縛値の更新自体は反映されているはず"
    );
    assert!(
        !badge_transform(root).is_empty(),
        "list FLIP を再生しないフラットリスト内の進行中の共有レイアウト \
         遷移は、束縛のみの更新では停止されず補正 transform が残るはず \
         （旧実装は flip_captured 全体を scope にしていたため中断された）"
    );
}
