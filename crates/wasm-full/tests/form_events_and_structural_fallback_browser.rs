//! `Runtime` の構造フォールバック（全再描画）・フォーム属性契約
//! （`data-action-input`/`data-action-change`）の実ブラウザ統合テスト
//! （イシュー #1120、`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/runtime_browser.rs` は `interactive::AppState`
//! （束縛点・keyed list のみで完結するカウンター/フォーム/動的リスト）を
//! 対象に既存の更新経路を検証済みである。本ファイルは #1120 が追加した
//! 2 つの新経路をそれぞれ実 DOM 上で検証する:
//!
//! 1. **構造フォールバック**（`docs/design/wasm-full-architecture.md` §21.1）:
//!    束縛点にも keyed list にも対応しない dirty field（画面遷移相当）が
//!    `root` サブツリー全体の差し替えを誘発すること、差し替え後もイベント
//!    委譲（`root` への 1 回だけの登録）が再配線なしに機能し続けること、
//!    `Runtime::rerender` の能動呼び出し、`build_dom_node` 失敗時
//!    （`Node::RawHtml` 混入）の fail-closed（既存 DOM 維持）。
//! 2. **フォーム属性契約**（`docs/design/wasm-full-architecture.md` §21.2）:
//!    `<select>` の `change` イベントが `data-action-change` 経由で
//!    dispatch されること、checkbox の `checked` が `"true"`/`"false"`
//!    文字列 payload として渡ること。
//!
//! 本ファイル専用のテスト component（[`ScreenState`]）を用意する理由:
//! `interactive::AppState` の `view()` は `<select>`/checkbox を持たず、
//! 全フィールドが束縛点で処理可能なため、構造フォールバックを自然に
//! 誘発する dirty field を持たない（`screen` フィールドはどの
//! `data-bind-*` にも対応させないことで、意図的に「束縛点にも keyed list
//! にも解決できない dirty field」を作る）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{bind_attr_token, el, raw_html, text, Node, BIND_ATTR_ATTR};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlInputElement, HtmlSelectElement};

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

/// `runtime_browser.rs::bubbling_event` と同じ意図（`change` にも流用）。
fn bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

/// 画面（構造フォールバックの誘発源）。`Broken` は `build_dom_node` の
/// fail-closed（`Node::RawHtml` 混入）を検証するための意図的な壊れた画面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    List,
    Detail,
    Broken,
}

/// `ScreenState::decode_action` が復号する型付きアクション。
enum ScreenAction {
    GoDetail,
    GoList,
    GoBroken,
    SetStatus(String),
    ToggleActive(bool),
}

/// 「属性フォーム → 一覧 → 詳細」相当のマルチ画面 SPA を模した最小 component
/// （イシュー #1120 フィードバック 1・2 の再現用）。
///
/// - `screen`: どの `data-bind-*` にも対応させない dirty field。画面遷移で
///   `dirty_fields()` に積まれるが、束縛点にも keyed list にも解決できない
///   ため `Runtime::apply_update_for_dirty` の構造フォールバックを誘発する。
/// - `status`/`active`: 通常の束縛点（テキスト/属性）で処理される dirty
///   field。`<select>`/checkbox の `change` イベント配線（`data-action-change`）
///   の検証対象。
#[derive(Debug, Clone)]
struct ScreenState {
    screen: Screen,
    status: String,
    active: bool,
    dirty: Vec<&'static str>,
}

impl ScreenState {
    fn new() -> Self {
        Self {
            screen: Screen::List,
            status: "pending".to_string(),
            active: false,
            dirty: Vec::new(),
        }
    }
}

impl Component for ScreenState {
    type Action = ScreenAction;

    fn update(&mut self, action: Self::Action) {
        // `DirtyTracked` 契約: 呼び出し冒頭で前回分をクリアし、今回の
        // update で実際に変更したフィールドのみを積む
        // (`interactive::DirtyTracked` doc 参照)。
        self.dirty.clear();
        match action {
            ScreenAction::GoDetail => {
                if self.screen != Screen::Detail {
                    self.screen = Screen::Detail;
                    self.dirty.push("screen");
                }
            }
            ScreenAction::GoList => {
                if self.screen != Screen::List {
                    self.screen = Screen::List;
                    self.dirty.push("screen");
                }
            }
            ScreenAction::GoBroken => {
                if self.screen != Screen::Broken {
                    self.screen = Screen::Broken;
                    self.dirty.push("screen");
                }
            }
            ScreenAction::SetStatus(value) => {
                if self.status != value {
                    self.status = value;
                    self.dirty.push("status");
                }
            }
            ScreenAction::ToggleActive(value) => {
                if self.active != value {
                    self.active = value;
                    self.dirty.push("active");
                }
            }
        }
    }

    fn view(&self) -> Node {
        // `Broken` はあえて `raw_html` をトップレベルへ返す（唯一のノードが
        // `Node::RawHtml` であるケース）。`fandhe_frontend_wasm_client::build_dom_node`
        // は `RawHtml` を fail-closed に `None` として拒否する契約
        // （`keyed_dom.rs` doc 参照）ため、構造フォールバックが「差し替え
        // 失敗 → 既存 DOM 維持」を選ぶことの検証に使う。本経路は
        // `fandhe_frontend_core::render()` を通さないため既定エスケープの
        // 保証対象外だが、`Runtime::mount` の初期描画（`dom::mount_initial`）
        // は必ず `render()` を経由するため、`Broken` へ遷移する前段は
        // 通常の（エスケープ済み）画面である前提（本テストでも初期状態は
        // `List` から開始する）。
        if self.screen == Screen::Broken {
            // テスト専用の意図的な壊れたノード（`build_dom_node` の
            // fail-closed 契約を検証するための固定文字列。外部入力・
            // 利用者制御値を一切含まない、ハードコードされたテスト
            // フィクスチャであるため ESCAPE-REVIEWED とする）。
            #[expect(
                clippy::disallowed_methods,
                reason = "ESCAPE-REVIEWED: fixed test fixture literal, no external input"
            )]
            return raw_html("<div data-testid=\"broken-should-not-render\"></div>");
        }

        let content = match self.screen {
            Screen::List => el(
                "div",
                vec![("data-testid", "list-view")],
                vec![text("LIST VIEW")],
            ),
            Screen::Detail => el(
                "div",
                vec![("data-testid", "detail-view")],
                vec![text("DETAIL VIEW")],
            ),
            Screen::Broken => unreachable!("Broken は上の早期 return で処理済み"),
        };

        let active_bind_attr = bind_attr_token("data-active", "active");
        el(
            "div",
            vec![("data-testid", "screen-app")],
            vec![
                el(
                    "button",
                    vec![
                        ("data-action", "go_detail"),
                        ("data-testid", "go-detail-btn"),
                    ],
                    vec![text("Detail")],
                ),
                el(
                    "button",
                    vec![("data-action", "go_list"), ("data-testid", "go-list-btn")],
                    vec![text("List")],
                ),
                el(
                    "button",
                    vec![
                        ("data-action", "go_broken"),
                        ("data-testid", "go-broken-btn"),
                    ],
                    vec![text("Broken")],
                ),
                el(
                    "select",
                    vec![
                        ("data-action-change", "set_status"),
                        ("data-testid", "status-select"),
                    ],
                    vec![
                        el("option", vec![("value", "pending")], vec![text("Pending")]),
                        el("option", vec![("value", "shipped")], vec![text("Shipped")]),
                    ],
                ),
                el(
                    "input",
                    vec![
                        ("type", "checkbox"),
                        ("data-action-change", "toggle_active"),
                        ("data-testid", "active-checkbox"),
                        (BIND_ATTR_ATTR, &active_bind_attr),
                    ],
                    vec![],
                ),
                fandhe_frontend_core::bind_text(
                    "span",
                    vec![("data-testid", "status-value")],
                    "status",
                    self.status.clone(),
                ),
                content,
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            "go_detail" => Some(ScreenAction::GoDetail),
            "go_list" => Some(ScreenAction::GoList),
            "go_broken" => Some(ScreenAction::GoBroken),
            "set_status" => Some(ScreenAction::SetStatus(payload.to_string())),
            "toggle_active" => Some(ScreenAction::ToggleActive(payload == "true")),
            _ => None,
        }
    }
}

impl DirtyTracked for ScreenState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for ScreenState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "status" => Some(BoundValue::Text(self.status.clone())),
            "active" => Some(BoundValue::Flag(self.active)),
            // `screen` は意図的にどの束縛点にも対応させない
            // （構造フォールバックの誘発源、本ファイル冒頭 doc 参照）。
            _ => None,
        }
    }
}

/// 構造フォールバック: 束縛点にも keyed list にも対応しない dirty field
/// （`screen`）が root サブツリー全体の差し替えを誘発すること。差し替え後も
/// イベント委譲（`root` への 1 回だけの登録）が再配線なしに機能し続ける
/// こと（`go-list-btn` が新しく構築されたノードであっても click に反応する）
/// も合わせて確認する。
#[wasm_bindgen_test]
fn structural_fallback_replaces_subtree_and_keeps_event_delegation_working() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "screen-structural-fallback-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("screen-structural-fallback-root", ScreenState::new())
        .expect("mount must succeed");

    assert!(
        placeholder
            .query_selector("[data-testid='list-view']")
            .expect("query_selector must not fail")
            .is_some(),
        "初期状態は List 画面であること"
    );

    let go_detail = placeholder
        .query_selector("[data-testid='go-detail-btn']")
        .expect("query_selector must not fail")
        .expect("go-detail-btn must exist");
    go_detail
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(runtime.component().screen, Screen::Detail);
    assert!(
        placeholder
            .query_selector("[data-testid='detail-view']")
            .expect("query_selector must not fail")
            .is_some(),
        "画面遷移（束縛点にも keyed list にも対応しない dirty field）で \
         root サブツリーが detail-view を含む新しい内容へ差し替わること"
    );
    assert!(
        placeholder
            .query_selector("[data-testid='list-view']")
            .expect("query_selector must not fail")
            .is_none(),
        "差し替え後は旧 list-view が DOM から除去されていること"
    );

    // 差し替え後の go-list-btn は新規構築されたノードだが、イベント委譲は
    // root への 1 回だけの登録（`closest`/`contains` ベース）のため
    // 再配線なしにクリックへ反応するはずである。
    let go_list_after_replace = placeholder
        .query_selector("[data-testid='go-list-btn']")
        .expect("query_selector must not fail")
        .expect("go-list-btn must exist after subtree replacement");
    go_list_after_replace
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().screen,
        Screen::List,
        "差し替え後の新規ノードに対する click も、再配線なしに \
         イベント委譲経由で dispatch されること（root 委譲の再配線不要性の証跡）"
    );
    assert!(
        placeholder
            .query_selector("[data-testid='list-view']")
            .expect("query_selector must not fail")
            .is_some(),
        "2 回目の構造フォールバックでも list-view へ戻ること"
    );
}

/// `Runtime::rerender` の能動的な明示呼び出しが `Self::rerender_subtree` と
/// 同じ全再描画を行うこと（自動発動＝dirty field 検知経由と同じ実装を
/// アプリ側から能動的にトリガーできることの検証）。
#[wasm_bindgen_test]
fn rerender_explicitly_rebuilds_subtree_from_current_state() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "screen-rerender-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let mut state = ScreenState::new();
    state.screen = Screen::Detail;
    let runtime = Runtime::mount("screen-rerender-root", state).expect("mount must succeed");

    // mount 直後は Detail 画面として初期描画されている（構造フォールバックを
    // 経由せず `dom::mount_initial` が直接反映する経路）。
    assert!(placeholder
        .query_selector("[data-testid='detail-view']")
        .expect("query_selector must not fail")
        .is_some());

    let list_view_before_rerender = placeholder
        .query_selector("[data-testid='detail-view']")
        .expect("query_selector must not fail")
        .expect("detail-view must exist before rerender");

    runtime.rerender();

    let detail_view_after_rerender = placeholder
        .query_selector("[data-testid='detail-view']")
        .expect("query_selector must not fail")
        .expect("detail-view must exist after rerender (same screen, rebuilt subtree)");
    assert!(
        !list_view_before_rerender.is_same_node(Some(&detail_view_after_rerender)),
        "rerender() は root の子ノードを丸ごと新規構築し直すため、同じ内容の \
         画面でもノード参照は再生成されること（全再描画であることの根拠）"
    );

    // rerender 後もイベント委譲が機能すること。
    let go_list = placeholder
        .query_selector("[data-testid='go-list-btn']")
        .expect("query_selector must not fail")
        .expect("go-list-btn must exist after rerender");
    go_list
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");
    assert_eq!(runtime.component().screen, Screen::List);
}

/// `build_dom_node` が `None`（`Node::RawHtml` の混入、fail-closed）を返す
/// 場合、構造フォールバックは既存 DOM を維持し何も差し替えないこと
/// （`Runtime::rerender_subtree` doc の fail-closed 契約）。
#[wasm_bindgen_test]
fn structural_fallback_keeps_existing_dom_when_build_dom_node_fails() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "screen-broken-fallback-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("screen-broken-fallback-root", ScreenState::new())
        .expect("mount must succeed");

    let go_broken = placeholder
        .query_selector("[data-testid='go-broken-btn']")
        .expect("query_selector must not fail")
        .expect("go-broken-btn must exist");
    go_broken
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    // Rust 側の状態自体は dispatch により更新される（構造フォールバックの
    // DOM 反映失敗は状態のロールバックを意味しない、`apply_update_for_dirty`
    // は状態更新と DOM 反映を別工程として扱う設計）。
    assert_eq!(runtime.component().screen, Screen::Broken);

    // DOM 側は `build_dom_node` が `None` を返すため差し替えが行われず、
    // 直前（List 画面）の内容が維持されているはずである。
    assert!(
        placeholder
            .query_selector("[data-testid='list-view']")
            .expect("query_selector must not fail")
            .is_some(),
        "build_dom_node の失敗時は既存 DOM（List 画面）が維持されること"
    );
    assert!(
        placeholder
            .query_selector("[data-testid='broken-should-not-render']")
            .expect("query_selector must not fail")
            .is_none(),
        "RawHtml 由来のノードは fail-closed により DOM へ一切反映されないこと"
    );
}

/// フォーム属性契約: `<select>` の `change` イベントが `data-action-change`
/// 経由で dispatch され、選択値が payload として反映されること
/// （イシュー #1120 フィードバック 2）。
#[wasm_bindgen_test]
fn select_change_event_dispatches_via_data_action_change_attribute() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "screen-select-change-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("screen-select-change-root", ScreenState::new())
        .expect("mount must succeed");

    let select = placeholder
        .query_selector("[data-testid='status-select']")
        .expect("query_selector must not fail")
        .expect("status-select must exist");
    select
        .dyn_ref::<HtmlSelectElement>()
        .expect("status-select must be an HtmlSelectElement")
        .set_value("shipped");
    select
        .dispatch_event(&bubbling_event("change"))
        .expect("dispatch_event must not fail");

    assert_eq!(
        runtime.component().status,
        "shipped",
        "select の change イベントが data-action-change 経由で dispatch され \
         選択値が payload として反映されること"
    );
    assert!(
        placeholder
            .inner_html()
            .contains(r#"data-bind-text="status">shipped<"#),
        "status は通常の束縛点更新（テキスト）で DOM へ反映されること: {}",
        placeholder.inner_html()
    );
}

/// フォーム属性契約: checkbox の `change` イベントが `checked` を
/// `"true"`/`"false"` 文字列 payload として `data-action-change` 経由で
/// dispatch すること（イシュー #1120 フィードバック 2）。
#[wasm_bindgen_test]
fn checkbox_change_event_dispatches_checked_state_as_string_payload() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "screen-checkbox-change-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let runtime = Runtime::mount("screen-checkbox-change-root", ScreenState::new())
        .expect("mount must succeed");

    let checkbox = placeholder
        .query_selector("[data-testid='active-checkbox']")
        .expect("query_selector must not fail")
        .expect("active-checkbox must exist");
    checkbox
        .dyn_ref::<HtmlInputElement>()
        .expect("active-checkbox must be an HtmlInputElement")
        .set_checked(true);
    checkbox
        .dispatch_event(&bubbling_event("change"))
        .expect("dispatch_event must not fail");

    assert!(
        runtime.component().active,
        "checkbox の checked=true が \"true\" 文字列 payload として \
         dispatch され、状態へ反映されること"
    );
}

// レガシー経路の非退行: `data-action-input`/`data-action-change` 属性が
// 付いていない要素（`interactive::AppState` の `draft-input` 相当、
// `id="draft-input"` ハードコード経路）は本ファイルの `ScreenState` には
// 存在しないため、`runtime_browser.rs::input_event_preserves_element_identity_and_updates_state`
// が `interactive::AppState` を対象に既に検証済みである（本ファイルでの
// 重複追加はしない）。
