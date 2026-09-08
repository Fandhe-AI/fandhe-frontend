//! `fandhe_frontend_wasm_full::sidebar`（イシュー #2074）の実ブラウザ回帰
//! テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/sidebar.rs` の native `#[cfg(test)] mod tests` は
//! 純粋ロジック層（[`fandhe_frontend_wasm_full::sidebar::is_toggle_shortcut`]
//! 等）とヘッドレス出力のドリフト検知までを検証済みである。本ファイルは
//! その先、配線層（`wiring`、`#[cfg(target_arch = "wasm32")]`）が実 DOM
//! （headless Chromium）上で
//!
//! 1. trigger/rail クリック → [`fandhe_frontend_wasm_full::sidebar::
//!    wire_sidebar_dispatch`]（`crate::headless::wire_headless_component`
//!    の薄いラッパー、イシュー #2074 codex-review P1 是正で新設）経由の
//!    `"toggle"` dispatch（`crate::headless::MAPPING_TABLE` への
//!    `(sidebar, trigger)`/`(sidebar, rail)` 追加分の実 DOM 確認、
//!    disabled trigger は no-op）
//! 2. Cmd/Ctrl+B ショートカット（document keydown）が trigger/rail への
//!    click 合成で同じ dispatch 経路を通ること、Shift 併用・Alt 併用・
//!    `preventDefault()` 済みイベント・trigger/rail 双方 disabled では
//!    発火しないこと
//! 3. `window.matchMedia` 常時 true/false クエリでの `data-mobile` の
//!    付け外し、モバイル進入エッジでの expanded → collapsed 寄せ
//! 4. モバイル drawer の Escape・外側 pointerdown 閉鎖（root/trigger/rail
//!    内側 pointerdown は無視、デスクトップでは no-op）
//! 5. `collapsible=icon` かつ collapsed の `menu-button` tooltip
//!    hover/focus 表示（`aria-describedby` 解決・`hidden`/`data-state`
//!    切替）、expanded/offcanvas/mobile では非表示のまま
//! 6. sidebar 非搭載 root への配線が no-op（Ctrl+B が `preventDefault()`
//!    されない）
//!
//! を検証する。`on_update` コールバックが行う `data-state` の DOM 反映は
//! 本ファイルのテストハーネス自身が模擬する（本クレートの
//! `Runtime`（`crates/wasm-full/src/lib.rs`）が本番で担う束縛点/構造
//! フォールバック再描画の代わり。`headless_wiring_browser.rs` は
//! `on_update` を no-op にして状態オブジェクトの変化のみを見るが、本
//! モジュールの配線対象（`data-mobile`/Escape/pointerdown/tooltip）は
//! いずれも DOM 属性を読むため、DOM 反映を模擬しないと検証にならない）。
//!
//! `mql` の `change` イベント実発火は headless Chrome で viewport を
//! 変更できないため未検証（[`fandhe_frontend_wasm_full::sidebar::
//! wire_sidebar_events_with_query`] の `query` 引数によるテスト用注入
//! （`"(min-width: 1px)"`/`"(max-width: 0px)"`）で代替する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::sidebar::{
    menu_button, provider, rail, root as sidebar_root, trigger, Sidebar, SidebarCollapsible,
    SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::tooltip;
use fandhe_frontend_wasm_full::sidebar::{
    wire_sidebar_dispatch, wire_sidebar_events, wire_sidebar_events_with_query,
};

/// テスト用の常時非一致メディアクエリ（デスクトップ扱いを強制する）。
///
/// [`fandhe_frontend_wasm_full::sidebar::wire_sidebar_events`]（既定の
/// `DEFAULT_MOBILE_MEDIA_QUERY = "(max-width: 767px)"`）は headless
/// Chrome の実ビューポート幅（環境によって 767px 以下になり得る）に
/// 左右されるため、モバイル判定そのものを検証する対象（§3・§4）以外の
/// テストは本定数を経由する [`wire_sidebar_events_with_query`] で
/// デスクトップ固定にし、実ビューポート依存の flaky を避ける。
const DESKTOP_QUERY: &str = "(max-width: 0px)";
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する（一意な
/// id で同一バイナリ内の複数テストの要素・リスナーが奪い合わないように
/// する、`headless_wiring_browser.rs::create_container` と同型）。
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
/// テスト末尾でコンテナを document から確実に除去する RAII ガード。
///
/// [`wire_sidebar_events`]/[`wire_sidebar_events_with_query`] が登録する
/// document 委譲リスナー（keydown/pointerdown）は本クレートの他
/// `wire_*` と同じくアプリ生存期間ぶん `Closure::forget` する設計
/// （モジュール冒頭 doc「セキュリティ不変条件」参照）であり、明示的な
/// 解除手段を持たない。同一 `wasm-pack test` バイナリ内の複数
/// `#[wasm_bindgen_test]` は同一 `document` を共有するため、後続テストが
/// 合成 keydown/pointerdown を document へ dispatch すると、**既に完了した
/// テストの登録済みリスナーも同じイベントを受け取る**（リスナー登録順に
/// 同期的に実行される DOM イベントの仕様どおり）。そのリスナーが自分の
/// （既に完了しテスト対象ではなくなった）trigger/rail をまだ DOM 上に
/// 見つけられると、無関係のクリック合成・`preventDefault()` を後続テスト
/// の同一イベントへ副作用として持ち込んでしまう。
///
/// 本ガードは `remove()`（document からの分離）の前に `set_inner_html("")`
/// でコンテナの子要素を空にすることで、以後同じ document に残り続ける
/// 旧リスナーが `find_first`/`find_first_enabled`（`root.query_selector_all`
/// はコンテナが分離済みでも子要素に対しては動作する）で trigger/rail/
/// provider を一切解決できなくする（早期 return、no-op）。これにより
/// テスト間の意図しない相互汚染を構造的に断つ（本フレームワーク自体の
/// 修正ではなく、複数テストが同一 document を共有する
/// `wasm-bindgen-test` 実行モデル固有のテストハーネス側の対策）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.set_inner_html("");
        self.0.remove();
    }
}

/// `bubbles: true` の合成 `click` を生成する
/// （`headless_wiring_browser.rs::bubbling_click_event` と同型）。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

fn dispatch_click(target: &Element) {
    target
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
}

/// `bubbles: true, cancelable: true` の合成 `keydown` を組み立てる
/// （`splitter_browser.rs::keydown_event` と同型に修飾子を追加）。
fn keydown_event(key: &str, ctrl: bool, meta: bool, alt: bool) -> KeyboardEvent {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(ctrl);
    init.set_meta_key(meta);
    init.set_alt_key(alt);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
}

/// document へ合成 keydown を発火する。
fn dispatch_document_keydown(
    document: &Document,
    key: &str,
    ctrl: bool,
    meta: bool,
    alt: bool,
) -> bool {
    let event = keydown_event(key, ctrl, meta, alt);
    document
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail")
}

/// `bubbles: true` の合成 `pointerdown`/`pointerover`/`pointerout`/
/// `focusin`/`focusout` を生成する（`overlay_close_browser.rs::
/// pointerdown_event` と同型。専用 Init 型を持つ実 PointerEvent/FocusEvent
/// である必要はなく、配線層は `dyn_ref` で opportunistic に扱う）。
fn bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

fn dispatch_event_on(target: &Element, kind: &str) {
    target
        .dispatch_event(&bubbling_event(kind))
        .expect("dispatch_event must not fail");
}

fn dispatch_event_on_document(document: &Document, kind: &str) {
    document
        .dispatch_event(&bubbling_event(kind))
        .expect("dispatch_event must not fail");
}

const PROVIDER_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"provider\"]";
const ROOT_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"root\"]";
const TRIGGER_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"trigger\"]";
const RAIL_SELECTOR: &str = "[data-scope=\"sidebar\"][data-part=\"rail\"]";

fn query(container: &Element, selector: &str) -> Option<Element> {
    container.query_selector(selector).ok().flatten()
}

/// `container` へ provider/root/trigger/rail からなる最小 Sidebar
/// マークアップを流し込み、`(component, provider, root, trigger, rail)` を
/// 返す。`trigger_disabled`/`rail_disabled` で `data-disabled` を付与
/// できる。
///
/// [`wire_sidebar_dispatch`]（イシュー #2074 codex-review P1 是正で新設
/// した公開オプトイン API、`crate::headless::wire_headless_component` の
/// 薄いラッパー）で `container` 自体を root として配線し、`on_update` で
/// `data-state` を provider/root へ反映する（モジュール冒頭 doc「本ファイル
/// のテストハーネス自身が模擬する」参照）。本番アプリがこの API を経由
/// せずに `Runtime::mount`/`Runtime::hydrate` のみに頼った場合、trigger/
/// rail クリックは dispatch へ到達しない（`wire_sidebar_events` 自体は
/// dispatch チャネルを持たない、モジュール冒頭 doc §1 参照）。
fn build_sidebar_markup(
    container: &Element,
    initial_state: SidebarState,
    trigger_disabled: bool,
    rail_disabled: bool,
) -> (Sidebar, Element, Element, Element, Element) {
    let sidebar = Sidebar::new(initial_state);
    let props = SidebarProps::default();
    let trigger_attrs: Vec<(&str, &str)> = if trigger_disabled {
        vec![("data-disabled", "")]
    } else {
        Vec::new()
    };
    let rail_attrs: Vec<(&str, &str)> = if rail_disabled {
        vec![("data-disabled", "")]
    } else {
        Vec::new()
    };
    let node = provider(
        &sidebar,
        &props,
        Vec::new(),
        vec![
            trigger(
                &sidebar,
                "Toggle Sidebar",
                None,
                trigger_attrs,
                vec![text("Toggle")],
            ),
            sidebar_root(
                &sidebar,
                &props,
                "Sidebar",
                None,
                Vec::new(),
                vec![rail(&sidebar, "Resize Sidebar", rail_attrs, Vec::new())],
            ),
        ],
    );
    container.set_inner_html(&render(&node));

    let provider_el = query(container, PROVIDER_SELECTOR).expect("provider must exist");
    let root_el = query(container, ROOT_SELECTOR).expect("root must exist");
    let trigger_el = query(container, TRIGGER_SELECTOR).expect("trigger must exist");
    let rail_el = query(container, RAIL_SELECTOR).expect("rail must exist");
    (sidebar, provider_el, root_el, trigger_el, rail_el)
}

/// `container` へ `data-state` を反映する `on_update` を組み立てて
/// [`wire_sidebar_dispatch`] を配線する（`mount_sidebar`/新規追加の
/// 順序回帰テストの双方から共有する）。
fn wire_dispatch_reflecting_data_state(container: &Element, component: Rc<RefCell<Sidebar>>) {
    let update_container = container.clone();
    wire_sidebar_dispatch(container.clone(), component, move |state, _root| {
        let data_state = state.data_state();
        if let Some(el) = query(&update_container, PROVIDER_SELECTOR) {
            let _ = el.set_attribute("data-state", data_state);
        }
        if let Some(el) = query(&update_container, ROOT_SELECTOR) {
            let _ = el.set_attribute("data-state", data_state);
        }
    })
    .expect("wire_sidebar_dispatch must not fail");
}

fn mount_sidebar(
    document: &Document,
    container: &Element,
    initial_state: SidebarState,
    trigger_disabled: bool,
    rail_disabled: bool,
) -> (Rc<RefCell<Sidebar>>, Element, Element, Element, Element) {
    let (sidebar, provider_el, root_el, trigger_el, rail_el) =
        build_sidebar_markup(container, initial_state, trigger_disabled, rail_disabled);
    let component = Rc::new(RefCell::new(sidebar));
    wire_dispatch_reflecting_data_state(container, component.clone());

    let _ = document;
    (component, provider_el, root_el, trigger_el, rail_el)
}

// --- 1. trigger/rail クリックの実 DOM dispatch ---

#[wasm_bindgen_test]
fn trigger_click_toggles_expanded_collapsed_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-trigger-click-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
    assert_eq!(
        provider_el.get_attribute("data-state").as_deref(),
        Some("collapsed")
    );

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn rail_click_toggles_expanded_collapsed_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-rail-click-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, _trigger_el, rail_el) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);

    dispatch_click(&rail_el);
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn disabled_trigger_click_is_noop_in_real_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-disabled-trigger-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Expanded, true, false);

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

// --- 2. Cmd/Ctrl+B ショートカット ---

#[wasm_bindgen_test]
fn ctrl_b_shortcut_toggles_via_trigger_click_synthesis() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-ctrl-b-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, _trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    let not_prevented = dispatch_document_keydown(&document, "b", true, false, false);
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
    assert!(
        !not_prevented,
        "ショートカット発火時は preventDefault() されること"
    );
}

#[wasm_bindgen_test]
fn detached_root_document_listener_does_not_block_new_sidebar_shortcut() {
    // イシュー #2074 codex-review P1 是正の回帰テスト。
    //
    // `wire_keydown`/`wire_pointerdown` が登録する document リスナーは
    // `root` を `move` で捕捉したまま解除手段を持たないため、`root` を
    // 含むコンテナを DOM から取り外して（例: 別画面への差し替え）別の
    // Sidebar を新たにマウントしても、旧リスナーは document に residual
    // として残り続ける。旧 `root`（detached だが子要素の `trigger` 自体は
    // 参照可能）がまだ enabled な trigger を解決できてしまうと、旧
    // リスナーが Cmd/Ctrl+B を `prevent_default()` し、同一 keydown
    // イベントを処理する新 Sidebar 側のリスナーが `default_prevented()`
    // を見て早期 return してしまい、新 Sidebar のショートカットが機能
    // しなくなる（`sidebar.rs::handle_document_keydown` の
    // `is_toggle_shortcut` 判定参照）。document リスナーは登録順に実行
    // されるため、先に登録した「旧」インスタンスを detach しても
    // `root.is_connected()` を確認していなければこの干渉が起きる。
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    // 旧インスタンス: マウント・配線した後、`RemoveOnDrop` と異なり
    // 子要素は空にせず `remove()` のみで DOM から切り離す（detached でも
    // trigger 自体は解決可能な状態を保つ、上記シナリオの再現）。
    let container_old = create_container(&document, "sidebar-detached-old-root");
    let (_component_old, ..) = mount_sidebar(
        &document,
        &container_old,
        SidebarState::Expanded,
        false,
        false,
    );
    wire_sidebar_events_with_query(container_old.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");
    container_old.remove();
    assert!(
        !container_old.is_connected(),
        "旧コンテナは DOM から切り離されていること"
    );

    // 新インスタンス: 通常どおり document へ接続した状態でマウント・配線
    // する。
    let container_new = create_container(&document, "sidebar-detached-new-root");
    let _cleanup = RemoveOnDrop(container_new.clone());
    let (component_new, ..) = mount_sidebar(
        &document,
        &container_new,
        SidebarState::Expanded,
        false,
        false,
    );
    wire_sidebar_events_with_query(container_new.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    // 旧リスナーが先に登録されているため、修正前はここで旧リスナーが
    // `prevent_default()` してしまい、新インスタンスの `is_toggle_shortcut`
    // が `default_prevented()` を見て no-op になる。
    let not_prevented = dispatch_document_keydown(&document, "b", true, false, false);
    assert_eq!(
        component_new.borrow().state(),
        SidebarState::Collapsed,
        "detach 済みの旧 root が新 Sidebar のショートカットを妨げないこと"
    );
    assert!(
        !not_prevented,
        "新インスタンス自身が preventDefault() すること"
    );

    // 後始末: 旧コンテナは既に DOM から切り離し済みだが、後続テストとの
    // 相互汚染防止のため子要素も明示的に空にしておく
    // （`RemoveOnDrop::drop` と同じ配慮）。
    container_old.set_inner_html("");
}

#[wasm_bindgen_test]
fn meta_b_shortcut_toggles_via_trigger_click_synthesis() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-meta-b-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_document_keydown(&document, "b", false, true, false);
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn shift_b_uppercase_does_not_trigger_shortcut() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-shift-b-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_document_keydown(&document, "B", true, false, false);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn alt_modifier_does_not_trigger_shortcut() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-alt-b-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_document_keydown(&document, "b", true, false, true);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn already_prevented_keydown_does_not_trigger_shortcut() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-prevented-b-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    let event = keydown_event("b", true, false, false);
    event.prevent_default();
    document
        .dispatch_event(event.as_ref())
        .expect("dispatch_event must not fail");
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn disabled_trigger_and_rail_shortcut_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-disabled-shortcut-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, ..) = mount_sidebar(&document, &container, SidebarState::Expanded, true, true);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    let not_prevented = dispatch_document_keydown(&document, "b", true, false, false);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
    assert!(
        not_prevented,
        "disabled な trigger/rail しかない場合は preventDefault() されないこと"
    );
}

// --- 3. モバイル判定（`window.matchMedia` 常時 true/false クエリ）---

#[wasm_bindgen_test]
fn always_matching_query_sets_data_mobile_and_collapses_expanded_on_enter() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-mobile-enter-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, provider_el, root_el, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    assert!(provider_el.has_attribute("data-mobile"));
    assert!(root_el.has_attribute("data-mobile"));
    // モバイル進入エッジで expanded だったため collapsed へ寄せる
    // （`sidebar.rs` モジュール doc「モバイル進入時に expanded を
    // collapsed へ寄せる意図的差分」参照）。
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn dispatch_registered_after_mobile_wiring_still_collapses_on_mount() {
    // イシュー #2074 codex-review P1 是正の回帰テスト。
    //
    // 本番経路（`Runtime::mount`/`Runtime::hydrate`）は
    // `wire_sidebar_events`（`wire_mobile` の初回 `apply_mobile_state`
    // 呼び出しを含む）を自動実行し、アプリはその**後**に個別で
    // `wire_sidebar_dispatch` を呼ぶ（`sidebar.rs` モジュール doc・
    // `Runtime::wire_sidebar` rustdoc 参照）。他のテスト
    // （`always_matching_query_sets_data_mobile_and_collapses_expanded_on_enter`
    // 等）は `mount_sidebar` が dispatch を先に配線するため、この
    // 本番の呼び出し順（モバイル判定 → dispatch 登録）を再現しない。
    // 本テストは `build_sidebar_markup` で markup のみ組み立て、
    // `wire_sidebar_events_with_query`（モバイル判定含む）→
    // `wire_sidebar_dispatch` の順で明示的に配線し、モバイル進入時の
    // 折りたたみ合成 click が dispatch 未登録で失われても、
    // `wire_sidebar_dispatch` 側の再確認（reconcile）で最終的に
    // `Collapsed` へ収束することを検証する。
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-mobile-dispatch-after-wiring-root");
    let _cleanup = RemoveOnDrop(container.clone());
    let _ = &document;

    let (sidebar, provider_el, root_el, ..) =
        build_sidebar_markup(&container, SidebarState::Expanded, false, false);

    // 本番の呼び出し順を再現: モバイル配線（初回 `apply_mobile_state` の
    // 合成 click は、まだ dispatch が登録されていないため失われる）を
    // 先に行う。
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    // この時点では DOM 上の `data-mobile` は付くが、dispatch が無いため
    // headless-ui 側の状態機械は `Expanded` のまま取り残されている
    // （合成 click 自体は誰にも処理されない）。
    assert!(provider_el.has_attribute("data-mobile"));

    let component = Rc::new(RefCell::new(sidebar));
    wire_dispatch_reflecting_data_state(&container, component.clone());

    // `wire_sidebar_dispatch` 登録直後の reconcile により、取りこぼされて
    // いた初期モバイル折りたたみへ追いついて `Collapsed` へ収束する。
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
    assert_eq!(
        provider_el.get_attribute("data-state").as_deref(),
        Some("collapsed")
    );
    assert_eq!(
        root_el.get_attribute("data-state").as_deref(),
        Some("collapsed")
    );
}

#[wasm_bindgen_test]
fn always_matching_query_does_not_force_collapse_when_already_collapsed() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-mobile-already-collapsed-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, provider_el, ..) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    assert!(provider_el.has_attribute("data-mobile"));
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn never_matching_query_does_not_set_data_mobile() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-mobile-never-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, provider_el, root_el, ..) =
        mount_sidebar(&document, &container, SidebarState::Expanded, false, false);
    wire_sidebar_events_with_query(container.clone(), "(max-width: 0px)")
        .expect("wire_sidebar_events_with_query must not fail");

    assert!(!provider_el.has_attribute("data-mobile"));
    assert!(!root_el.has_attribute("data-mobile"));
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

// --- 4. モバイル drawer の Escape・外側 pointerdown 閉鎖 ---

#[wasm_bindgen_test]
fn escape_dismisses_mobile_drawer_when_expanded() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-escape-dismiss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // Collapsed で開始し、モバイル進入時の強制 collapse を経由させない。
    let (component, _provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);

    // 利用者がドロワーを開く。
    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    dispatch_document_keydown(&document, "Escape", false, false, false);
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn outside_pointerdown_dismisses_mobile_drawer_when_expanded() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-outside-pointerdown-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    // container 自体は sidebar root ではないため「外側」扱いになる。
    dispatch_event_on_document(&document, "pointerdown");
    assert_eq!(component.borrow().state(), SidebarState::Collapsed);
}

#[wasm_bindgen_test]
fn pointerdown_inside_root_does_not_dismiss_mobile_drawer() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-inside-root-pointerdown-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    dispatch_event_on(&root_el, "pointerdown");
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn pointerdown_on_trigger_does_not_dismiss_mobile_drawer() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-trigger-pointerdown-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    dispatch_event_on(&trigger_el, "pointerdown");
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

#[wasm_bindgen_test]
fn escape_and_outside_pointerdown_are_noop_on_desktop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-desktop-noop-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (component, _provider_el, _root_el, trigger_el, _rail_el) =
        mount_sidebar(&document, &container, SidebarState::Collapsed, false, false);
    // 常時非一致クエリ = デスクトップ扱い（`data-mobile` 無し）。
    wire_sidebar_events_with_query(container.clone(), "(max-width: 0px)")
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_click(&trigger_el);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    dispatch_document_keydown(&document, "Escape", false, false, false);
    assert_eq!(component.borrow().state(), SidebarState::Expanded);

    dispatch_event_on_document(&document, "pointerdown");
    assert_eq!(component.borrow().state(), SidebarState::Expanded);
}

// --- 5. `collapsible=icon` の `menu-button` tooltip hover/focus ---

/// `container` へ collapsed + `collapsible=icon` の sidebar
/// （provider/root/menu-button）と、`aria-describedby` で関連付けた
/// tooltip（root/positioner/content）を組み立てる。`(menu_button, content)`
/// を返す。
fn mount_sidebar_with_tooltip(container: &Element, mobile: bool) -> (Element, Element) {
    let sidebar = Sidebar::new(SidebarState::Collapsed);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        mobile,
        ..Default::default()
    };
    let menu_button_props = SidebarMenuButtonProps {
        describedby: Some("sidebar-tooltip-content"),
        ..Default::default()
    };
    let node = provider(
        &sidebar,
        &props,
        Vec::new(),
        vec![
            sidebar_root(
                &sidebar,
                &props,
                "Sidebar",
                None,
                Vec::new(),
                vec![menu_button(
                    &menu_button_props,
                    Vec::new(),
                    vec![text("Home")],
                )],
            ),
            tooltip::root(
                OpenState::Closed,
                Vec::new(),
                vec![tooltip::positioner(
                    OpenState::Closed,
                    Vec::new(),
                    vec![tooltip::content(
                        OpenState::Closed,
                        Some("sidebar-tooltip-content"),
                        Vec::new(),
                        vec![text("Home")],
                    )],
                )],
            ),
        ],
    );
    container.set_inner_html(&render(&node));

    let menu_button_el = container
        .query_selector("[data-scope=\"sidebar\"][data-part=\"menu-button\"]")
        .expect("query_selector must not fail")
        .expect("menu-button must exist");
    let content_el = container
        .query_selector("#sidebar-tooltip-content")
        .expect("query_selector must not fail")
        .expect("tooltip content must exist");
    (menu_button_el, content_el)
}

#[wasm_bindgen_test]
fn pointerover_shows_tooltip_when_collapsed_icon_desktop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-tooltip-pointerover-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (menu_button_el, content_el) = mount_sidebar_with_tooltip(&container, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    assert!(content_el.has_attribute("hidden"));

    dispatch_event_on(&menu_button_el, "pointerover");
    assert!(!content_el.has_attribute("hidden"));
    assert_eq!(
        content_el.get_attribute("data-state").as_deref(),
        Some("open")
    );

    dispatch_event_on(&menu_button_el, "pointerout");
    assert!(content_el.has_attribute("hidden"));
    assert_eq!(
        content_el.get_attribute("data-state").as_deref(),
        Some("closed")
    );
}

#[wasm_bindgen_test]
fn focusin_shows_and_focusout_hides_tooltip() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-tooltip-focus-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (menu_button_el, content_el) = mount_sidebar_with_tooltip(&container, false);
    wire_sidebar_events_with_query(container.clone(), DESKTOP_QUERY)
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_event_on(&menu_button_el, "focusin");
    assert!(!content_el.has_attribute("hidden"));

    dispatch_event_on(&menu_button_el, "focusout");
    assert!(content_el.has_attribute("hidden"));
}

#[wasm_bindgen_test]
fn tooltip_stays_hidden_when_mobile() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-tooltip-mobile-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let (menu_button_el, content_el) = mount_sidebar_with_tooltip(&container, true);
    // 常時一致クエリでモバイル判定を確定させる（`DESKTOP_QUERY` の逆、
    // 実ビューポート依存を避けるための決定的注入）。
    wire_sidebar_events_with_query(container.clone(), "(min-width: 1px)")
        .expect("wire_sidebar_events_with_query must not fail");

    dispatch_event_on(&menu_button_el, "pointerover");
    assert!(content_el.has_attribute("hidden"));
}

// --- 6. sidebar 非搭載アプリへの副作用なし ---

#[wasm_bindgen_test]
fn wiring_is_noop_when_sidebar_not_present() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "sidebar-absent-root");
    let _cleanup = RemoveOnDrop(container.clone());

    container.set_inner_html("<button type=\"button\">plain button</button>");
    wire_sidebar_events(container.clone())
        .expect("wire_sidebar_events must not fail on absent sidebar");

    let not_prevented = dispatch_document_keydown(&document, "b", true, false, false);
    assert!(
        not_prevented,
        "sidebar 非搭載アプリでは Ctrl+B が preventDefault() されないこと"
    );
}
