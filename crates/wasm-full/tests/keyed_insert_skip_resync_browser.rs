//! `Runtime::apply_update_for_dirty` の「保持キャッシュが無い field」経路
//! （`keyed_list_cache` の `None` 分岐、`apply_keyed_list` 呼び出し）が
//! 挿入スキップ（`Node::RawHtml` 混入等の構築失敗）を正しくキャッシュへ
//! 反映することの実ブラウザ統合テスト（Bugbot 指摘、PR #1340、イシュー
//! #1340）。
//!
//! `wasm-full/tests/keyed_update_browser.rs` は「保持キャッシュが**有る**
//! field」（`apply_keyed_list_with_previous` 経由の `Update` 適用）を検証
//! 済みだが、初回の keyed list 構造変化（`None` 分岐、`apply_keyed_list`
//! 経由の構造フォールバック）で挿入スキップが起きるケースは未検証だった。
//!
//! `None` 分岐は `Runtime::mount` が初期 view から
//! `collect_keyed_list_nodes` で全 keyed list field を事前にキャッシュへ
//! 種付けするため、**初期 view に存在しない keyed list field**（`screen`
//! トグルで後から出現する）でのみ自然に再現できる（本ファイルの
//! [`ListState`] が screen A/B トグルを持つ理由）。`screen` トグル自体は
//! 束縛点にも keyed list にも解決できない dirty field として構造フォール
//! バック（[`Runtime::rerender_subtree`]、`keyed_list_cache` を丸ごと
//! `clear()` するのみで再種付けはしない）を誘発し、以降 `items` field は
//! `None` 分岐（`keyed_list_cache` にエントリが無い）から開始する。
//!
//! Bugbot 指摘の再現手順（`crates/wasm-full/src/lib.rs`
//! `apply_update_for_dirty` の `None` 分岐）: 修正前は `apply_keyed_list`
//! の戻り値（達成可否）を見ずに常に「望ましい view」（`list_node.clone()`）
//! を `keyed_list_cache` へ確定させていた。挿入が 1 件でもスキップされる
//! と、実 DOM には存在しないアイテムが「直前に DOM へ反映した内容」として
//! キャッシュへ紛れ込み、次回そのアイテムの内容だけを正しい値へ修正しても
//! （`KeyedOp::Update` 経路、`apply_keyed_list_with_previous`）実 DOM へは
//! 一切反映されない（`Update` 対象は「既存アイテムの内容更新」であって
//! 「新規挿入」ではなく、`find_by_key` が実 DOM 上に存在しないキーを
//! 見つけられずスキップするのみで、`resync_required`〔#1340 P1 対応〕は
//! 次回の再同期を要求するだけで**その回の**実 DOM を修復しない）。
//! 本ファイルはこの手順どおり、
//!
//! 1. `screen` トグルで `items` keyed list を初出現させる（`None` 分岐が
//!    種付けなしで開始する状態を作る）
//! 2. 2 件目のアイテムを `Node::RawHtml` 混入つきで挿入（構築失敗、実 DOM
//!    には反映されない）
//! 3. 次の tick でそのアイテムの内容を正当な値へ修正する
//!
//! を行い、手順 3 の dispatch 後に実 DOM が正しく（2 件とも）収束すること
//! を確認する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{el, raw_html, text, Node};
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

/// 内容がこのマーカーのアイテムは、[`ListState::view`] が `Node::RawHtml`
/// 混入つきで構築するため `build_dom_node` が `None` を返し、
/// `KeyedListDom::create_item` の構築失敗（`Insert` の fail-closed skip）を
/// 誘発する（`crate::keyed_dom` モジュール doc 不変条件 4 参照）。
const POISON_MARKER: &str = "__poison__";

/// `screen` トグル（`form_events_and_structural_fallback_browser.rs::ScreenState`
/// と同じ意図: 束縛点にも keyed list にも解決できない dirty field を作り、
/// 構造フォールバックを誘発する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    /// `items` keyed list を含まない初期画面。
    Empty,
    /// `items` keyed list を含む画面。
    List,
}

/// screen トグル + 動的リストを持つ最小 component。
/// `keyed_update_browser.rs::ListState` と異なり、`items` keyed list が
/// **初期 view には存在しない**（`screen == Empty` の間は `view()` が
/// `<ul data-bind-list="items">` を一切出力しない）ことが本ファイルの
/// 目的（`Runtime::mount` の事前キャッシュ種付けを回避し、`None` 分岐を
/// 種付けなしで発生させる、モジュール doc 参照）。
#[derive(Debug, Clone)]
struct ListState {
    screen: Screen,
    /// `(安定キー, 現在の内容)` の順序付きリスト。
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
}

impl ListState {
    const FIELD_SCREEN: &'static str = "screen";
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str)]) -> Self {
        Self {
            screen: Screen::Empty,
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
    /// `screen` を `List` へ切り替える（構造フォールバックを誘発、
    /// `items` keyed list を初出現させる）。
    ShowList,
    /// `id` を内容 `POISON_MARKER` で新規追加する（構築失敗を誘発する
    /// `Insert`）。
    InsertPoisoned { id: u64 },
    /// `id` の項目内容を `content` へ置き換える（`InsertPoisoned` で失敗
    /// した挿入を、正当な内容で再修正するために使う）。
    Fix { id: u64, content: String },
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::ShowList => {
                if self.screen != Screen::List {
                    self.screen = Screen::List;
                    self.dirty.push(Self::FIELD_SCREEN);
                }
            }
            ListAction::InsertPoisoned { id } => {
                self.items.push((id, POISON_MARKER.to_string()));
                self.dirty.push(Self::FIELD_ITEMS);
            }
            ListAction::Fix { id, content } => {
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
        if self.screen == Screen::Empty {
            return el(
                "div",
                vec![
                    ("id", "keyed-insert-skip-root"),
                    ("data-testid", "screen-empty"),
                ],
                vec![text("EMPTY")],
            );
        }

        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|(id, content)| {
                let node = if content == POISON_MARKER {
                    // テスト専用の意図的な壊れたノード（`build_dom_node` の
                    // fail-closed 契約を検証するための固定文字列。外部
                    // 入力・利用者制御値を一切含まない、ハードコードされた
                    // テストフィクスチャであるため ESCAPE-REVIEWED とする、
                    // `form_events_and_structural_fallback_browser.rs` と
                    // 同じ方針）。
                    #[expect(
                        clippy::disallowed_methods,
                        reason = "ESCAPE-REVIEWED: fixed test fixture literal, no external input"
                    )]
                    let poisoned_child = raw_html("<script>alert(1)</script>");
                    el(
                        "li",
                        vec![("data-testid", "keyed-insert-skip-item")],
                        vec![poisoned_child],
                    )
                } else {
                    el(
                        "li",
                        vec![("data-testid", "keyed-insert-skip-item")],
                        vec![text(content)],
                    )
                };
                (id.to_string(), node)
            })
            .collect();
        let list = keyed_list("ul", vec![], "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "keyed-insert-skip-root")], vec![list])
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        match name {
            "show_list" => Some(ListAction::ShowList),
            // payload 形式: "<id>"。
            "insert_poisoned" => Some(ListAction::InsertPoisoned {
                id: payload.parse::<u64>().ok()?,
            }),
            // payload 形式: "<id>:<content>"。
            "fix" => {
                let (id_str, content) = payload.split_once(':')?;
                Some(ListAction::Fix {
                    id: id_str.parse::<u64>().ok()?,
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

/// `screen`/`items` のいずれも `data-bind-*` 束縛点を使わないため、
/// `bound_value` は常に `None`。`screen` は意図的にどの束縛点にも
/// 解決させないことで構造フォールバックを誘発する
/// （`ListState::view` doc 参照）。
impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `keyed_update_browser.rs::dispatch_rename` と同じ手法（合成クリック
/// ターゲットの動的追加 → dispatch → 除去）で、任意の action 名・payload を
/// dispatch する。
fn dispatch(document: &Document, root: &Element, action: &str, payload: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", action)
        .expect("set_attribute must not fail");
    trigger
        .set_attribute("data-payload", payload)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// Bugbot 指摘の再現手順そのもの（PR #1340、イシュー #1340）:
/// 1. `show_list` で `items` keyed list を初出現させる（構造フォール
///    バック、`keyed_list_cache` は種付けされないまま）。
/// 2. `insert_poisoned` で 2 件目のアイテムを `Node::RawHtml` 混入つきで
///    挿入しようとし、構築失敗で実 DOM には反映されない（`None` 分岐、
///    `apply_keyed_list`）。
/// 3. `fix` で同じキーの内容を正当な値へ修正する。
///
/// 修正前（`apply_keyed_list` の戻り値を見ずに常時 `list_node` をキャッシュ
/// へ確定させる実装）では、手順 2 の時点で実際には存在しない 2 件目が
/// 「直前に DOM へ反映した内容」として `keyed_list_cache` へ紛れ込み、
/// 手順 3 の `fix` dispatch は `KeyedOp::Update`（既存アイテムの内容更新）
/// として処理されるため、実 DOM には一切反映されなかった（`Update` は
/// 新規挿入を行わないため）。修正後は手順 2 で `apply_keyed_list` が
/// `false` を返した field はキャッシュへ確定させないため、手順 3 の
/// dispatch も引き続き `None` 分岐（`apply_keyed_list` による実 DOM 読み
/// 出しベースの構造フォールバック）を通り、正しく `Insert` として適用
/// され 2 件とも実 DOM 上に収束する。
#[wasm_bindgen_test]
fn insert_skip_then_fixed_content_converges_on_next_dispatch() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "keyed-insert-skip-root-container");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a")]);
    let runtime =
        Runtime::mount("keyed-insert-skip-root-container", state).expect("mount must succeed");
    let root = runtime.root();

    // 手順 1: items keyed list を初出現させる（構造フォールバック経由、
    // `keyed_list_cache` は種付けされないまま「items」field の `None`
    // 分岐が以降の起点になる）。
    dispatch(&document, root, "show_list", "");
    let items_after_show = root
        .query_selector_all("[data-testid='keyed-insert-skip-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(
        items_after_show.length(),
        1,
        "手順 1 の時点で 1 件目（毒性なし）は正しく表示されているはず"
    );

    // 手順 2: 2 件目（キー 2）を構築失敗させる内容で挿入しようとする。
    dispatch(&document, root, "insert_poisoned", "2");

    let items_after_poisoned = root
        .query_selector_all("[data-testid='keyed-insert-skip-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(
        items_after_poisoned.length(),
        1,
        "構築失敗した 2 件目は実 DOM へ反映されず、1 件目のみが存在する \
         はず（fail-closed skip）"
    );

    // 手順 3: キー 2 の内容を正当な値へ修正する。
    dispatch(&document, root, "fix", "2:b-fixed");

    let items_after_fix = root
        .query_selector_all("[data-testid='keyed-insert-skip-item']")
        .expect("query_selector_all must not fail");
    assert_eq!(
        items_after_fix.length(),
        2,
        "内容修正後の dispatch で 2 件目が正しく実 DOM へ挿入され、\
         2 件とも存在するはず（キャッシュが未達成状態を確定させていた \
         場合、この Insert が Update として誤処理され反映されないまま \
         残る）"
    );

    let texts: Vec<Option<String>> = (0..items_after_fix.length())
        .map(|i| {
            items_after_fix
                .item(i)
                .map(|node| node.text_content())
                .unwrap_or(None)
        })
        .collect();
    assert_eq!(
        texts,
        vec![Some("a".to_string()), Some("b-fixed".to_string())],
        "1 件目・2 件目とも最終的に正しい内容へ収束しているはず"
    );

    assert_eq!(
        runtime.component().items,
        vec![(1u64, "a".to_string()), (2u64, "b-fixed".to_string())],
        "状態側も最終値と一致していること"
    );
}
