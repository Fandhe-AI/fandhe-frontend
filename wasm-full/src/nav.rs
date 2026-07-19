//! クライアント側ルーティング（history API 連携・URL 同期・遷移時 loader 配線、
//! イシュー #374）。
//!
//! # 位置付け・呼び出し文脈
//!
//! `rws-wasm-full` は `rws-server`（`server/src/ssr.rs`）へ依存できない
//! （`structure.toml` の `server.allowed_dependents = ["dist-server"]`）ため、
//! クライアント側のルート解決は `rws-server` を経由しない。イシュー #407 で
//! `rws-app`（`server`・`wasm-full` 双方から依存可能な唯一の層）へルート定義
//! （パターン・マッチングエンジン・ページタイトル）を単一化したため、本
//! モジュールは [`rws_app::routes::resolve`] / [`rws_app::routes::title`] を
//! そのまま呼ぶ（`server/src/ssr.rs` と同じ関数を共有し、パターンリテラル・
//! タイトルリテラル・マッチング意味論のいずれも本ファイルで再定義しない）。
//! `wasm-full/tests/route_shared_static.rs`（静的ソース走査、
//! `core/tests/no_branching_across_modes.rs` と同方式）が単一定義の強制を
//! 継続検証する。
//!
//! [`crate::csr`]（イシュー #349）の `resolve_list_node`/`resolve_detail_node`
//! を loader ジェネリックなまま呼び出し、遷移時のデータ解決を行う
//! （初期表示・ハイドレーションでは呼ばない、という #349/#345 由来の凍結事項は
//! 本モジュールでは「初期ロード時に [`crate::entry::start_router`] が描画を
//! 行わない」という形で維持する）。
//!
//! # 2 層構成（`events.rs`/`csr.rs` と同じ方針）
//!
//! - **純粋層**（本ファイル直下）: [`ClientRoute`]・[`resolve_path`]・
//!   [`resolve_route_view_with`]。DOM（`web-sys`）に一切依存せず、native の
//!   `cargo test --workspace` から直接呼べる（`wasm-full/tests/nav_native.rs`）。
//! - **配線層**（[`mod wiring`]、`#[cfg(target_arch = "wasm32")]`）: history API
//!   （`pushState`/`popstate`）連携、`data-nav` クリック委譲、DOM 差し替え。
//!
//! # セキュリティ不変条件
//!
//! - 遷移描画は [`rws_wasm_client::build_dom_node`]（`createElement`/
//!   `createTextNode`/`set_attribute` のみ）で行い、`set_inner_html` を
//!   一切呼ばない（受け入れ条件 2、#345 の不変条件の継承）。
//! - インターセプト対象は「`/` 始まりかつ `//` 非始まり・ルート表に一致する」
//!   相対パスのみに構造的に限定する。一致しない値・修飾キー付きクリック・
//!   左クリック以外はブラウザ既定動作に委ねる（オープンリダイレクト対策）。
//! - history state には何も格納しない（URL のみを状態の正とする）。
//! - リスナー登録は起動時の定数回（click 1 + popstate 1）の `Closure::forget`
//!   に限定する（`events.rs` と同方針、無制限リークの構造的回避）。
//! - 遷移後の `data-hydrate` 要素へのイベント再配線（イシュー #403）は
//!   [`rws_wasm_client::wire_hydrate_targets`] の呼び出しに限定する。同関数は
//!   `add_event_listener_with_callback` の後付けのみを行い `set_inner_html`
//!   等の再構築系 API を呼ばない（`rws-wasm-client` 側の不変条件を継承）。
//!   クロージャの寿命は `rws-wasm-client::registry` が root 要素の `id` 単位
//!   で管理し、再配線のたびに旧ハンドルを解除してから差し替えるため、上記
//!   「`forget` は起動時定数回」の不変条件（`click`/`popstate` の 2 回）とは
//!   独立に、遷移ごとの再配線が無制限リークを生まない（`registry::replace_handles`
//!   による寿命管理、`forget()` を使わない）。

use crate::csr::{resolve_detail_node, resolve_list_node};
use rws_app::routes::{resolve as resolve_route, title as route_title, AppRoute};
use rws_app::{Item, Loader};
use rws_core::Node;

/// クライアント側で解決したルート。`rws_app::routes::ResolvedRoute`
/// （イシュー #407 の単一定義）を [`resolve_path`] がクライアント側の呼び出し
/// 形へ変換した表現であり、パターン・意味論は再定義しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientRoute {
    /// `/` — 一覧画面。
    List,
    /// `/items/:id` — 詳細画面（捕捉した `id`）。
    Detail(String),
}

/// パスをルートへ解決する（DOM 非依存の純粋関数）。
///
/// マッチング本体は [`rws_app::routes::resolve`]（`rws_app::router::Router`
/// 経由、`docs/api/router-path-matching.md` v1 仕様準拠）に委譲する。
/// 一致しないパスは `None`（呼び出し側はブラウザ既定遷移に委ねる、
/// 安全側フォールバック）。
pub fn resolve_path(path: &str) -> Option<ClientRoute> {
    let resolved = resolve_route(path)?;
    match resolved.route {
        AppRoute::List => Some(ClientRoute::List),
        // `AppRoute::Detail` は `rws_app::routes::resolve` の契約上、常に
        // `id` を捕捉して返す（`resolved.id` が `None` になるのは `List`
        // 側のみ）。万一 `None` の場合でも `unwrap_or_default` で空文字列に
        // フォールバックし、panic しない（ライブラリコードの規約継承）。
        AppRoute::Detail => Some(ClientRoute::Detail(resolved.id.unwrap_or_default())),
    }
}

/// ルートを解決済み loader で「タイトル + 描画済み Node」へ変換する。
///
/// `server/src/ssr.rs::respond_with` と同じ分岐構造を踏襲する:
///
/// - `List`: `list_loader` を解決し `resolve_list_node`（内部で `Err` を
///   [`crate::csr::loader_error_view`] へ変換、fail-closed）を呼ぶ。タイトルは
///   [`rws_app::routes::title`]（単一定義、イシュー #407）を使う。
/// - `Detail`: `detail_loader` を解決し `resolve_detail_node` を呼ぶ。未知の
///   `id`（`Ok(None)`）も `Err` もいずれもタイトルは変わらない
///   （`page_shell` へ渡すタイトルは `Ok`/`Err`/`None` のいずれでも
///   `respond_with` と同じ [`rws_app::routes::title`] の値と一致させる。
///   ページ内の見出し文言 "見つかりません" とは独立した `<title>` 相当の値
///   である）。
pub fn resolve_route_view_with<L, D>(
    list_loader: &L,
    detail_loader: &D,
    route: &ClientRoute,
) -> (&'static str, Node)
where
    L: Loader<Input = (), Output = Vec<Item>>,
    D: Loader<Input = String, Output = Option<Item>>,
{
    match route {
        ClientRoute::List => (route_title(AppRoute::List), resolve_list_node(list_loader)),
        ClientRoute::Detail(id) => (
            route_title(AppRoute::Detail),
            resolve_detail_node(detail_loader, id),
        ),
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`hydration.rs`/`dom.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{resolve_path, resolve_route_view_with, ClientRoute};
    use rws_app::{DemoItemDetailLoader, DemoItemsLoader};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, MouseEvent};

    /// `window().document()` を解決する。`events.rs`/`wasm-client::wiring` と
    /// 同じ固定文言方針（内部状態を含めない、不変条件 6）。
    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))
    }

    fn root_element(root_id: &str) -> Result<Element, JsValue> {
        document()?
            .get_element_by_id(root_id)
            .ok_or_else(|| JsValue::from_str("root element not found"))
    }

    /// `value` が `pushState`/描画へ渡してよい相対パスかを判定する。
    ///
    /// `/` 始まりかつ `//` 非始まり（プロトコル相対 URL・外部オリジンへの
    /// 迂回を構造的に排除、オープンリダイレクト対策）の値のみを許可する。
    fn is_safe_relative_path(value: &str) -> bool {
        value.starts_with('/') && !value.starts_with("//")
    }

    /// `route` を解決し、`root`（`rws_app::layout` が組み立てる
    /// `<div id="app-root" data-rws="root">` 相当の要素そのもの。
    /// `page_shell` の出力は `<body>` 直下にこの要素を単独で置くため、呼び出し
    /// 元は `root_id = "app-root"` を渡す想定）の子要素を loader 解決済み
    /// ノードで差し替えて `document.title` を更新する（受け入れ条件 2:
    /// 束縛点更新/keyed list ではなくサブツリー差し替えだが、`set_inner_html`
    /// は使わない）。
    ///
    /// 描画は [`rws_wasm_client::build_dom_node`]（`createElement`/
    /// `createTextNode`/`set_attribute` のみ）経由で `resolve_route_view_with`
    /// が返す `Node`（`layout()` の出力＝`<div id="app-root">...</div>`
    /// 相当）を丸ごと新規構築し、その**子要素のみ**を `root` へ移し替える
    /// （`root` 自身は `replace_child`/`replaceWith` で入れ替えない。
    /// `root_id` で解決した要素の同一性を維持したまま中身だけ差し替える設計。
    /// `root` の属性（`id`/`data-rws`）はナビゲーション間で不変のため
    /// コピー不要）。既定 loader（`DemoItemsLoader`/`DemoItemDetailLoader`、
    /// `server/src/ssr.rs::respond` と同じ既定）を使う。
    fn render_route(document: &Document, root: &Element, route: &ClientRoute) {
        let (title, node) = resolve_route_view_with(&DemoItemsLoader, &DemoItemDetailLoader, route);

        let Some(new_dom_node) = rws_wasm_client::build_dom_node(document, &node) else {
            // RawHtml 混入等の構造的にあり得ないケース（fail-closed）。
            // 現在の DOM を維持したまま no-op（panic しない）。
            web_sys::console::warn_1(
                &"rws-wasm-full: nav render_route failed to build replacement DOM node".into(),
            );
            return;
        };

        while let Some(child) = root.first_child() {
            let _ = root.remove_child(&child);
        }
        while let Some(new_child) = new_dom_node.first_child() {
            let _ = root.append_child(&new_child);
        }

        document.set_title(title);

        // イシュー #403: 差し替えた子要素は build_dom_node による新規生成
        // ノードであり、イベントリスナーが一切付いていない
        // （`rws-wasm-client::wiring::hydrate` が担う初期表示ページの配線とは
        // 別経路）。registry キーは root 要素の `id`（実運用 `app-root`）とし、
        // wasm-client デモ側（別 wasm インスタンス・別 registry、キー `app`）
        // とは衝突しない。対象 0 件（`detail_page(None)`/`loader_error_view`
        // 等）のページへの遷移では空集合で差し替わる（旧リスナー解除のみ）。
        if let Err(_err) = rws_wasm_client::wire_hydrate_targets(&root.id(), root) {
            // fail-safe: 再配線に失敗しても遷移自体（DOM 差し替え・URL・
            // タイトル更新）は既に成立させているため、ここでは継続する
            // （内部状態を含まない固定英語文言、不変条件 6 の継承）。
            web_sys::console::warn_1(
                &"rws-wasm-full: nav render_route failed to wire data-hydrate targets".into(),
            );
        }
    }

    /// `path`（`location.pathname` + `location.search` 相当）を再解決して
    /// 描画する。一致しないパスは no-op（現在の DOM を維持、安全側
    /// フォールバック）。
    fn navigate_render(document: &Document, root: &Element, path: &str) {
        let Some(route) = resolve_path(path) else {
            return;
        };
        render_route(document, root, &route);
    }

    /// `path` が [`resolve_path`] で解決可能な場合のみ `history.pushState`
    /// で URL を進め、描画する（`popstate` からは呼ばない。history へ
    /// エントリを追加するのはユーザー操作起点のクリック遷移のみ）。
    fn push_and_render(document: &Document, root: &Element, path: &str) {
        let Some(route) = resolve_path(path) else {
            return;
        };
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                // history state には何も格納しない（URL のみを状態の正とする、
                // 改ざん面を持たない設計判断）。
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
            }
        }
        render_route(document, root, &route);
    }

    /// クリックが左クリック・無修飾キーかを判定する（新規タブで開く等の
    /// ブラウザ既定動作を壊さないための判定、`events.rs` にはない
    /// `nav` 固有の追加ガード）。
    fn is_plain_left_click(event: &MouseEvent) -> bool {
        event.button() == 0
            && !event.ctrl_key()
            && !event.meta_key()
            && !event.shift_key()
            && !event.alt_key()
    }

    /// `root_id` 要素へのクライアント側ルーティングを起動する（`nav.rs` の
    /// 配線エントリ、[`crate::entry::start_router`] から呼ばれる）。
    ///
    /// - `document` レベルで `click` を委譲登録し、`closest("[data-nav]")`
    ///   で祖先方向に `data-nav` 属性を持つ要素を探す。`render_route` は
    ///   `root` 自身ではなく `root` の**子要素のみ**を差し替えるため `root`
    ///   へ登録しても理論上は生存するが、将来の描画方式変更（`root` 自体の
    ///   再生成を伴う変更）でリスナーが失われないよう、**リスナーは
    ///   `document` へ登録**する（`events.rs` の `root` 委譲とは異なる判断。
    ///   `docs/design/wasm-full-architecture.md` 第 4 節・判断 8）。
    /// - `window` へ `popstate` を 1 回だけ登録し、`location.pathname` +
    ///   `location.search` から再解決・再描画する（`pushState` は呼ばない、
    ///   history の往復のみに追従する）。
    /// - **起動時（本関数呼び出し時点）は描画を一切行わない**（SSR 済み
    ///   DOM をそのまま維持する。初期表示で loader を再実行しない凍結事項
    ///   の遵守、`docs/design/loader-trait-design.md` §4・§7.3）。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、または
    /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
    pub fn start_router(root_id: &str) -> Result<(), JsValue> {
        let doc = document()?;
        // `root_id` 要素の存在確認のみに使う（クロージャへは捕捉しない）。
        // 遷移のたびに `document.get_element_by_id` で再取得する方針
        // （下記クロージャ内コメント参照）のため、ここでの取得は起動時の
        // 前提チェック（要素が存在しない設定ミスを早期に `Err` で検出する）
        // に限定する。
        let _root = root_element(root_id)?;

        let click_document = doc.clone();
        let root_id_owned = root_id.to_string();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() else {
                return;
            };
            if !is_plain_left_click(&mouse_event) {
                return;
            }
            let Some(target) = event.target() else {
                return;
            };
            let target_element: Element = match target.dyn_ref::<Element>() {
                Some(element) => element.clone(),
                None => {
                    let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                        return;
                    };
                    let Some(parent) = node.parent_element() else {
                        return;
                    };
                    parent
                }
            };
            let Ok(Some(matched)) = target_element.closest("[data-nav]") else {
                return;
            };
            let Some(value) = matched.get_attribute("data-nav") else {
                return;
            };
            if !is_safe_relative_path(&value) {
                return;
            }
            if resolve_path(&value).is_none() {
                // ルート表に一致しない値はサーバー権威の通常遷移に委ねる
                // （安全側フォールバック、`docs/api/router-path-matching.md`
                // に定めのない値をクライアント側で誤判定しない）。
                return;
            }
            // `root_id` 要素は差し替えの都度 `get_element_by_id` で再取得する
            // （`render_route` は `root` の子要素のみを差し替え、`root` 自身
            // （`#app-root` 相当の要素）は再生成しないため実際には起動時の
            // 要素のままでも有効だが、将来 `root_id` 要素自体が差し替えられる
            // 変更が入った場合に備え明示的に再解決する）。
            let Some(current_root) = click_document.get_element_by_id(&root_id_owned) else {
                return;
            };
            mouse_event.prevent_default();
            push_and_render(&click_document, &current_root, &value);
        });
        doc.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let popstate_document = doc.clone();
        let popstate_root_id = root_id.to_string();
        let popstate_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let Ok(location_pathname) = window.location().pathname() else {
                return;
            };
            let search = window.location().search().unwrap_or_default();
            let path = format!("{location_pathname}{search}");
            let Some(current_root) = popstate_document.get_element_by_id(&popstate_root_id) else {
                return;
            };
            navigate_render(&popstate_document, &current_root, &path);
        });
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .add_event_listener_with_callback(
                "popstate",
                popstate_closure.as_ref().unchecked_ref(),
            )?;
        popstate_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::start_router;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_matches_list_route() {
        assert_eq!(resolve_path("/"), Some(ClientRoute::List));
    }

    #[test]
    fn resolve_path_matches_detail_route_and_strips_query() {
        assert_eq!(
            resolve_path("/items/2?ref=top"),
            Some(ClientRoute::Detail("2".to_string()))
        );
    }

    #[test]
    fn resolve_path_rejects_trailing_slash() {
        assert_eq!(resolve_path("/items/1/"), None);
    }

    #[test]
    fn resolve_path_rejects_unregistered_path() {
        assert_eq!(resolve_path("/nope"), None);
    }

    #[test]
    fn resolve_path_rejects_missing_leading_slash() {
        assert_eq!(resolve_path("items/1"), None);
    }

    #[test]
    fn resolve_path_rejects_empty_id_segment() {
        assert_eq!(resolve_path("/items/"), None);
    }
}
