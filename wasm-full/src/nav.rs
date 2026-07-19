//! クライアント側ルーティング（history API 連携・URL 同期・遷移時 loader 配線、
//! イシュー #374）。
//!
//! # 位置付け・呼び出し文脈
//!
//! `rws-wasm-full` は `rws-server`（`server/src/ssr.rs`）へ依存できない
//! （`structure.toml` の `server.allowed_dependents = ["dist-server"]`）ため、
//! クライアント側のルート解決（パス → `PageRoute` 相当の判定）は本モジュールに
//! `rws_server::router::Router` を使わず独自実装する。`docs/api/router-path-matching.md`
//! v1 仕様（クエリ切り落とし・セグメント厳格一致・末尾スラッシュ非正規化・
//! `:id` 捕捉）のうち、本アプリの 2 ルート（`/`・`/items/:id`）に必要な範囲のみを
//! 実装する。`server/src/ssr.rs::build_page_router()` のルートパターン
//! リテラル・ページタイトルリテラルとの一致は `wasm-full/tests/route_sync_static.rs`
//! （静的ソース走査、`core/tests/no_branching_across_modes.rs` と同方式）が
//! ドリフト検知として強制する。
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
//!   に限定する（`events.rs` と同方針、無制限リークの構造的回避）。加えて
//!   イシュー #404 で導入した遷移ごとの View Transitions update コールバック
//!   は `Closure::once_into_js`（1 回呼び出し後に JS 側が所有権ごと解放）で
//!   生成するため `forget` の対象には含めない。`startViewTransition` の
//!   update コールバックは遷移がスキップされる場合でも仕様上必ず一度呼ばれる
//!   ため、無制限リークにはならない（`docs/design/wasm-full-architecture.md`
//!   第 4 節・判断 10）。
//!
//! # View Transitions 連携（イシュー #404）
//!
//! [`wiring::render_route`] は「loader 解決 + 新 DOM 構築（`prepare` 段、
//! 遷移の外）」と「`root` への差し替え + タイトル更新（`apply` 段、
//! `document.startViewTransition()` の update コールバック内）」の 2 段に
//! 分割されている。loader 解決を遷移の外に置くことで、データ取得の遅延が
//! 遷移アニメーションの開始を妨げない（旧ビューは新ビューの準備が整うまで
//! 表示され続ける、View Transitions の推奨パターン）。`document` が
//! `startViewTransition` を持たない（非対応ブラウザ）場合は
//! [`wiring::with_view_transition`] が機能検出で判定し、apply 段を同期
//! 実行する（graceful degradation、失敗時も描画は必ず完了する）。

use crate::csr::{resolve_detail_node, resolve_list_node};
use rws_app::{Item, Loader};
use rws_core::Node;

/// クライアント側で解決したルート（`server/src/ssr.rs::PageRoute` に対応する
/// クライアント側の等価表現。`rws-server` への依存を避けるため独自定義する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientRoute {
    /// `/` — 一覧画面。
    List,
    /// `/items/:id` — 詳細画面（捕捉した `id`）。
    Detail(String),
}

/// パスをルートへ解決する（DOM 非依存の純粋関数）。
///
/// `docs/api/router-path-matching.md` v1 仕様のうち、本アプリの 2 ルートに
/// 必要な範囲を実装する:
///
/// - クエリ文字列（`?` 以降）は照合前に切り落とす
/// - 末尾スラッシュは正規化しない厳格一致（`/items/1/` は一致しない）
/// - 連続スラッシュ（空セグメント）は一致しない
/// - `id` は空でない 1 セグメントのみを捕捉する（`/items/` は一致しない）
///
/// 一致しないパスは `None`（呼び出し側はブラウザ既定遷移に委ねる、
/// 安全側フォールバック）。
pub fn resolve_path(path: &str) -> Option<ClientRoute> {
    let path_only = path.split('?').next().unwrap_or(path);
    if !path_only.starts_with('/') {
        return None;
    }
    if path_only == "/" {
        return Some(ClientRoute::List);
    }
    let segments: Vec<&str> = path_only[1..].split('/').collect();
    if segments.iter().any(|s| s.is_empty()) {
        // 連続スラッシュ・末尾スラッシュはいずれかのセグメントが空文字列に
        // なるため、ここで一括して非一致とする（v1 仕様の「空セグメント」
        // 「末尾スラッシュ非正規化」の双方を満たす）。
        return None;
    }
    match segments.as_slice() {
        ["items", id] => Some(ClientRoute::Detail((*id).to_string())),
        _ => None,
    }
}

/// ルートを解決済み loader で「タイトル + 描画済み Node」へ変換する。
///
/// `server/src/ssr.rs::respond_with` と同じ分岐構造を踏襲する:
///
/// - `List`: `list_loader` を解決し `resolve_list_node`（内部で `Err` を
///   [`crate::csr::loader_error_view`] へ変換、fail-closed）を呼ぶ。タイトルは常に
///   `"記事一覧"`（ssr.rs と同一リテラル、`route_sync_static.rs` が固定）。
/// - `Detail`: `detail_loader` を解決し `resolve_detail_node` を呼ぶ。未知の
///   `id`（`Ok(None)`）も `Err` もいずれもタイトルは `"記事詳細"` のまま
///   （`page_shell` へ渡すタイトルは `Ok`/`Err`/`None` のいずれでも
///   `respond_with` が "記事詳細" を使う契約と一致させる。ページ内の見出し
///   文言 "見つかりません" とは独立した `<title>` 相当の値である）。
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
        ClientRoute::List => ("記事一覧", resolve_list_node(list_loader)),
        ClientRoute::Detail(id) => ("記事詳細", resolve_detail_node(detail_loader, id)),
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

    /// `document.startViewTransition` の機能検出・呼び出し専用の duck-typing
    /// extern バインディング（イシュー #404）。
    ///
    /// web-sys 0.3 系の `Document::start_view_transition` は
    /// `#[cfg(web_sys_unstable_apis)]` ゲート付きであり、有効化には
    /// `RUSTFLAGS='--cfg web_sys_unstable_apis'` をワークスペース全体へ適用
    /// する必要がある（共有 `CARGO_TARGET_DIR` 運用・他クレートのビルド
    /// フラグ汚染を招くため不採用、`docs/design/wasm-full-architecture.md`
    /// 第 4 節・判断 10）。本 extern ブロックは安定版 wasm-bindgen のみで
    /// 完結し、ソーステキスト上に `unsafe` トークンを含まない（マクロ展開後の
    /// グルーコードのみが `unsafe` を含む、`docs/policy/unsafe-boundary.md`
    /// 第 2 節の許容境界 2 点目と同区分）。新規外部パッケージの追加はゼロ
    /// （既存の `wasm-bindgen`/`web-sys` のみで完結）。
    #[wasm_bindgen::prelude::wasm_bindgen]
    extern "C" {
        type DocumentViewTransitions;

        /// 機能検出用: `document.startViewTransition` プロパティの取得。
        /// 非対応ブラウザでは `undefined`（`JsValue::is_function()` が
        /// `false` を返す）になる。
        #[wasm_bindgen::prelude::wasm_bindgen(method, getter, js_name = startViewTransition)]
        fn start_view_transition_prop(this: &DocumentViewTransitions) -> JsValue;

        /// `document.startViewTransition(updateCallback)` の呼び出し。
        /// `updateCallback` は同期または Promise を返す関数（本モジュールでは
        /// 常に同期の update コールバックを渡す）。呼び出しが throw した場合は
        /// `catch` 属性により `Err` として返る。
        #[wasm_bindgen::prelude::wasm_bindgen(method, catch, js_name = startViewTransition)]
        fn start_view_transition(
            this: &DocumentViewTransitions,
            update: &JsValue,
        ) -> Result<JsValue, JsValue>;
    }

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

    /// `apply`（DOM 差し替え + タイトル更新等の副作用のみを行うクロージャ）を
    /// `document.startViewTransition()` でラップして呼び出す（イシュー #404）。
    ///
    /// `document` が `startViewTransition` を関数として持たない場合
    /// （非対応ブラウザ、機能検出）、または呼び出し自体が throw した場合は
    /// `apply` を同期的に直接実行する（graceful degradation。遷移が
    /// 失敗しても描画は必ず完了させる、fail-closed にしない）。
    ///
    /// `apply` を `Rc<RefCell<Option<F>>>` で包み、update コールバック
    /// （`Closure::once_into_js` で JS 側へ所有権を移し、呼び出し後に自己
    /// 解放する。`forget` 不使用）と throw 時フォールバックの双方から
    /// 「`take()` できた側のみが 1 回だけ実行する」形にすることで、
    /// 呼び出しが throw した場合でも `apply` を確実に一度だけ実行する
    /// （update コールバックが呼ばれずに throw するケースへの対処。
    /// `startViewTransition` の update コールバックは遷移がスキップされても
    /// 仕様上必ず一度呼ばれるため、通常経路では throw 側の `take()` は
    /// 常に空になり二重実行は起きない）。
    fn with_view_transition<F>(document: &Document, apply: F)
    where
        F: FnOnce() + 'static,
    {
        let doc_vt = document.clone().unchecked_into::<DocumentViewTransitions>();
        if !doc_vt.start_view_transition_prop().is_function() {
            // 非対応ブラウザ: 同期フォールバック。
            apply();
            return;
        }

        let slot = std::rc::Rc::new(std::cell::RefCell::new(Some(apply)));
        let update_slot = slot.clone();
        let update = Closure::once_into_js(move || {
            if let Some(apply) = update_slot.borrow_mut().take() {
                apply();
            }
        });
        if let Err(err) = doc_vt.start_view_transition(&update) {
            // 呼び出し自体が throw し、update コールバックが未実行のまま
            // 終わった場合の同期フォールバック（警告ログのみ、内部状態は
            // 含めない不変条件 6）。
            web_sys::console::warn_1(
                &"rws-wasm-full: document.startViewTransition threw, view transition skipped"
                    .into(),
            );
            let _ = err;
            if let Some(apply) = slot.borrow_mut().take() {
                apply();
            }
        }
    }

    /// `value` が `pushState`/描画へ渡してよい相対パスかを判定する。
    ///
    /// `/` 始まりかつ `//` 非始まり（プロトコル相対 URL・外部オリジンへの
    /// 迂回を構造的に排除、オープンリダイレクト対策）の値のみを許可する。
    fn is_safe_relative_path(value: &str) -> bool {
        value.starts_with('/') && !value.starts_with("//")
    }

    /// `route` を解決し、`root_id`（`rws_app::layout` が組み立てる
    /// `<div id="app-root" data-rws="root">` 相当の要素の id。呼び出し元は
    /// `root_id = "app-root"` を渡す想定）の子要素を loader 解決済みノードで
    /// 差し替えて `document.title` を更新する（受け入れ条件 2: 束縛点更新/
    /// keyed list ではなくサブツリー差し替えだが、`set_inner_html` は使わない）。
    ///
    /// # prepare/apply 2 段構成（イシュー #404）
    ///
    /// loader 解決 + `build_dom_node` による新 DOM 構築（**prepare 段**、
    /// [`prepare_render`]）は `document.startViewTransition()` の呼び出しより
    /// 前、同期に行う。`root` への差し替え + `document.set_title`
    /// （**apply 段**、[`apply_render`]）のみを [`with_view_transition`] の
    /// update コールバック内で実行する。これにより「遷移中に loader 解決が
    /// 走らない」（旧ビューはデータ準備完了まで表示され続ける、View
    /// Transitions の推奨パターン）ことが構造的に保証される。`root` は
    /// コールバック実行時に `document.get_element_by_id` で再解決する
    /// （`root` 自身は `replace_child`/`replaceWith` で入れ替えないため通常は
    /// 起動時の要素のままでも有効だが、prepare と apply の間に時間差が生まれる
    /// 非同期構成のため、将来 `root_id` 要素自体が差し替えられる変更が入った
    /// 場合に備え明示的に再解決する。要素が消えていた場合は no-op、panic
    /// しない）。
    ///
    /// 描画は [`rws_wasm_client::build_dom_node`]（`createElement`/
    /// `createTextNode`/`set_attribute` のみ）経由で `resolve_route_view_with`
    /// が返す `Node`（`layout()` の出力＝`<div id="app-root">...</div>`
    /// 相当）を丸ごと新規構築し、その**子要素のみ**を `root` へ移し替える
    /// （`root` の属性（`id`/`data-rws`）はナビゲーション間で不変のため
    /// コピー不要）。既定 loader（`DemoItemsLoader`/`DemoItemDetailLoader`、
    /// `server/src/ssr.rs::respond` と同じ既定）を使う。
    ///
    /// prepare 段のみを独立関数 [`prepare_render`] に切り出しているのは、
    /// `push_and_render` が `history.pushState` より**前**に構築結果を
    /// 必要とするため（イシュー #404 フォローアップ、Cursor Bugbot 指摘
    /// `27cc68fd`）。構築に失敗した場合（`RawHtml` 混入等の構造的にあり得ない
    /// ケース、fail-closed）は `None` を返し、呼び出し側は「URL・DOM の
    /// いずれも変更しない」no-op として扱う。
    fn prepare_render(
        document: &Document,
        route: &ClientRoute,
    ) -> Option<(&'static str, web_sys::Node)> {
        let (title, node) = resolve_route_view_with(&DemoItemsLoader, &DemoItemDetailLoader, route);
        let new_dom_node = rws_wasm_client::build_dom_node(document, &node)?;
        Some((title, new_dom_node))
    }

    /// apply 段（`root` への差し替え + タイトル更新）を
    /// `document.startViewTransition()` でラップして実行する。
    /// `prepare_render` が返した `title`/`new_dom_node` を受け取る側で、
    /// この関数自体は失敗しない（`root` 消失時は `with_view_transition`
    /// 経由の no-op、既存の fail-closed 方針を維持）。
    fn apply_render(
        document: &Document,
        root_id: &str,
        title: &'static str,
        new_dom_node: web_sys::Node,
    ) {
        let apply_document = document.clone();
        let root_id_owned = root_id.to_string();
        with_view_transition(document, move || {
            let Some(root) = apply_document.get_element_by_id(&root_id_owned) else {
                return;
            };
            while let Some(child) = root.first_child() {
                let _ = root.remove_child(&child);
            }
            while let Some(new_child) = new_dom_node.first_child() {
                let _ = root.append_child(&new_child);
            }
            apply_document.set_title(title);
        });
    }

    fn render_route(document: &Document, root_id: &str, route: &ClientRoute) {
        let Some((title, new_dom_node)) = prepare_render(document, route) else {
            // 現在の DOM を維持したまま no-op（panic しない）。
            web_sys::console::warn_1(
                &"rws-wasm-full: nav render_route failed to build replacement DOM node".into(),
            );
            return;
        };
        apply_render(document, root_id, title, new_dom_node);
    }

    /// `path`（`location.pathname` + `location.search` 相当）を再解決して
    /// 描画する。一致しないパスは no-op（現在の DOM を維持、安全側
    /// フォールバック）。
    fn navigate_render(document: &Document, root_id: &str, path: &str) {
        let Some(route) = resolve_path(path) else {
            return;
        };
        render_route(document, root_id, &route);
    }

    /// `path` が [`resolve_path`] で解決可能な場合のみ `history.pushState`
    /// で URL を進め、描画する（`popstate` からは呼ばない。history へ
    /// エントリを追加するのはユーザー操作起点のクリック遷移のみ）。
    ///
    /// `prepare_render`（loader 解決 + DOM 構築）を `pushState` より**前**に
    /// 実行し、成功した場合のみ `pushState` する（イシュー #404 フォロー
    /// アップ、Cursor Bugbot 指摘 `27cc68fd`）。構築失敗時に `pushState` が
    /// 先行して実行されると、URL だけが進みアドレスバーと表示中の DOM が
    /// 食い違う状態になり得るため、prepare 成功を `pushState` の前提条件と
    /// する（apply 段の `root` 再解決失敗（要素消失）はこの時点では検知
    /// できないため対象外。通常運用でこのアプリの `root` 自身は
    /// `render_route`/`apply_render` から差し替えられないため消失しない）。
    fn push_and_render(document: &Document, root_id: &str, path: &str) {
        let Some(route) = resolve_path(path) else {
            return;
        };
        let Some((title, new_dom_node)) = prepare_render(document, &route) else {
            web_sys::console::warn_1(
                &"rws-wasm-full: nav push_and_render failed to build replacement DOM node, URL not updated"
                    .into(),
            );
            return;
        };
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                // history state には何も格納しない（URL のみを状態の正とする、
                // 改ざん面を持たない設計判断）。`pushState` は View Transitions
                // の対象外（apply 段より前、prepare 段と同じく同期実行のまま）。
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
            }
        }
        apply_render(document, root_id, title, new_dom_node);
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
            // `render_route`（`push_and_render` 経由）の apply 段が
            // `root_id` 要素を `get_element_by_id` で再解決するため、ここでの
            // 存在確認は不要（イシュー #404 で apply 段へ移動）。
            mouse_event.prevent_default();
            push_and_render(&click_document, &root_id_owned, &value);
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
            navigate_render(&popstate_document, &popstate_root_id, &path);
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
