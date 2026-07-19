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
//! - history state には固定形式のスクロール座標レコード（[`encode_scroll_state`]
//!   が生成する `"rws-scroll:{x},{y}"` 文字列）のみを格納する（イシュー #406、
//!   従来の「何も格納しない」不変条件を限定緩和）。`push_state` に渡す state は
//!   従来どおり `JsValue::NULL` のまま（新規履歴エントリは URL のみを状態の
//!   正とする）で、離脱元エントリへ `replace_state` でスクロール位置を書き戻す
//!   点、および `pagehide`（ドキュメント破棄直前、リロード・外部遷移・タブ
//!   クローズを含む）で**現在エントリ**へも同様に `replace_state` で書き戻す
//!   点が変更点（後者はリロード時にスクロール位置が復元できない不具合の
//!   修正、イシュー #406 追加分）。読み取りは [`decode_scroll_state`] による厳格検証
//!   （fail-closed: 形式不一致・非数・非有限・負値はすべて `None`）を経てから
//!   `Window::scroll_to_with_x_and_y`（数値専用 API）にのみ渡し、DOM・URL・
//!   HTML へは一切流さない（改ざんされても表示位置がずれるだけで注入面を
//!   持たない）。
//! - リスナー登録は起動時の定数回（click 1 + popstate 1 + pagehide 1、
//!   最後者はイシュー #406 のリロード時スクロール消失修正で追加）の
//!   `Closure::forget` に限定する（`events.rs` と同方針、無制限リークの
//!   構造的回避）。[`wiring::start_router`] は同一 `root_id` で複数回呼ばれても
//!   [`wiring::REGISTERED_ROOT_IDS`] により 2 回目以降を no-op とするため、
//!   呼び出し側が誤って複数回呼んでも「`root_id` あたり定数回（1 組）」の
//!   不変条件が壊れない（多重マウント・再初期化・複数の統合テストが同一
//!   `document` を共有するテスト環境でのリスナー積み上がり対策、イシュー
//!   #404 フォローアップ）。加えて
//!   イシュー #404 で導入した遷移ごとの View Transitions update コールバック
//!   は `Closure::once_into_js`（1 回呼び出し後に JS 側が所有権ごと解放）で
//!   生成するため `forget` の対象には含めない。`startViewTransition` の
//!   update コールバックは遷移がスキップされる場合でも仕様上必ず一度呼ばれる
//!   ため、無制限リークにはならない（`docs/design/wasm-full-architecture.md`
//!   第 4 節・判断 10）。
//! - 遷移後の `data-hydrate` 要素へのイベント再配線（イシュー #403）は
//!   [`rws_wasm_client::wire_hydrate_targets`] の呼び出しに限定する。同関数は
//!   `add_event_listener_with_callback` の後付けのみを行い `set_inner_html`
//!   等の再構築系 API を呼ばない（`rws-wasm-client` 側の不変条件を継承）。
//!   クロージャの寿命は `rws-wasm-client::registry` が root 要素の `id` 単位
//!   で管理し、再配線のたびに旧ハンドルを解除してから差し替えるため、上記
//!   「`forget` は起動時定数回」の不変条件（`click`/`popstate` の 2 回）とは
//!   独立に、遷移ごとの再配線が無制限リークを生まない（`registry::replace_handles`
//!   による寿命管理、`forget()` を使わない）。この再配線呼び出しは
//!   [`wiring::apply_render_with_post`] の `startViewTransition` update
//!   コールバック内、DOM 差し替え + タイトル更新の直後に実行する（イシュー
//!   #404 との統合）。
//!
//! # View Transitions 連携（イシュー #404）
//!
//! [`wiring::render_route_with_post`] は「loader 解決 + 新 DOM 構築（`prepare` 段、
//! 遷移の外）」と「`root` への差し替え + タイトル更新（`apply` 段、
//! `document.startViewTransition()` の update コールバック内）」の 2 段に
//! 分割されている。loader 解決を遷移の外に置くことで、データ取得の遅延が
//! 遷移アニメーションの開始を妨げない（旧ビューは新ビューの準備が整うまで
//! 表示され続ける、View Transitions の推奨パターン）。`document` が
//! `startViewTransition` を持たない（非対応ブラウザ）場合は
//! [`wiring::with_view_transition`] が機能検出で判定し、apply 段を同期
//! 実行する（graceful degradation、失敗時も描画は必ず完了する）。

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

/// history state に格納するスクロール座標レコードの固定プレフィックス
/// （[`encode_scroll_state`]/[`decode_scroll_state`] が共有する契約）。
const SCROLL_STATE_PREFIX: &str = "rws-scroll:";

/// スクロール座標 `(x, y)` を history state 用の固定形式文字列へ変換する
/// （イシュー #406）。DOM 非依存の純粋関数（native テスト可能）。
///
/// 形式: `"rws-scroll:{x},{y}"`。読み取り側は [`decode_scroll_state`] で
/// 厳格検証してから使うため、ここでは値の妥当性チェックは行わない
/// （呼び出し元の [`mod wiring`] が `window.scroll_x()`/`scroll_y()` の
/// 実測値のみを渡す前提）。
pub fn encode_scroll_state(x: f64, y: f64) -> String {
    format!("{SCROLL_STATE_PREFIX}{x},{y}")
}

/// [`encode_scroll_state`] が生成した文字列を `(x, y)` へ復号する
/// （イシュー #406）。history state は同一オリジンから改ざん可能な前提の
/// ため fail-closed で検証する: プレフィックス不一致・カンマ区切りでない・
/// 非数・非有限（`NaN`/`Inf`）・負値のいずれかであれば `None` を返す。
///
/// 呼び出し元（[`mod wiring`]）は `None` を「先頭 `(0, 0)` へフォール
/// バック」として扱う。この関数の戻り値は `Window::
/// scroll_to_with_x_and_y`（数値専用 API）にのみ渡され、DOM・URL・HTML へ
/// 流入する経路を持たない。
pub fn decode_scroll_state(value: &str) -> Option<(f64, f64)> {
    let rest = value.strip_prefix(SCROLL_STATE_PREFIX)?;
    let (x_raw, y_raw) = rest.split_once(',')?;
    let x: f64 = x_raw.parse().ok()?;
    let y: f64 = y_raw.parse().ok()?;
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    Some((x, y))
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`hydration.rs`/`dom.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        decode_scroll_state, encode_scroll_state, resolve_path, resolve_route_view_with,
        ClientRoute,
    };
    use rws_app::{DemoItemDetailLoader, DemoItemsLoader};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, MouseEvent, PopStateEvent, ScrollRestoration};

    thread_local! {
        /// [`start_router`] を同一 `root_id` で複数回呼んでも `click`/`popstate`
        /// リスナーを重複登録しないための既登録 `root_id` 集合（イシュー #404
        /// フォローアップ）。
        ///
        /// [`start_router`] は `document` レベルのリスナーを
        /// `Closure::forget()`（意図的リーク）で登録するため、本来「起動時の
        /// 定数回（click 1 + popstate 1）」の呼び出しを前提とする
        /// （本ファイル冒頭のセキュリティ不変条件参照）。しかし呼び出し側が
        /// 同一 `root_id` に対して誤って複数回呼んだ場合（多重マウント・
        /// 再初期化・`wasm-full/tests/nav_browser.rs` のように 1 ページ内で
        /// 複数の統合テストが同一 `"app-root"` を使い回す場合等）、本集合が
        /// なければ呼ぶたびに新しい `click`/`popstate` リスナーが `document`
        /// へ積み上がり、以後の 1 クリックが登録済みリスナーの数だけ
        /// `push_and_render`/View Transitions を重複実行してしまう
        /// （無制限リークの構造的回避という不変条件そのものが崩れる）。
        /// `root_id` ごとに 1 度目の呼び出しのみリスナーを登録し、2 度目以降は
        /// 早期 `Ok(())` で no-op とすることで、`start_router` を同一 `root_id`
        /// で何度呼んでも登録済みリスナー数が高々 1 組であることを構造的に
        /// 保証する。
        static REGISTERED_ROOT_IDS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    }

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
    /// （**apply 段**、[`apply_render_with_post`]）のみを [`with_view_transition`] の
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
    ///
    /// `post_apply` は DOM 差し替え・タイトル更新・`data-hydrate` 再配線が
    /// 完了した**直後**（`with_view_transition` の update コールバック内、
    /// View Transitions 対応ブラウザでは非同期）に実行される（イシュー #406
    /// との統合: `push_and_render` の「新規遷移は先頭へスクロール」・
    /// popstate 側の「保存位置へスクロール復元」のいずれも、新 DOM が実際に
    /// 差し替わった後でなければ「差し替え前の旧ページの高さ」を基準に
    /// スクロールしてしまい得るため、この関数の呼び出し直後に同期実行する
    /// ことはできない）。
    fn apply_render_with_post<P>(
        document: &Document,
        root_id: &str,
        title: &'static str,
        new_dom_node: web_sys::Node,
        post_apply: P,
    ) where
        P: FnOnce() + 'static,
    {
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

            // イシュー #403: 差し替えた子要素は build_dom_node による新規生成
            // ノードであり、イベントリスナーが一切付いていない
            // （`rws-wasm-client::wiring::hydrate` が担う初期表示ページの配線とは
            // 別経路）。registry キーは root 要素の `id`（実運用 `app-root`）とし、
            // wasm-client デモ側（別 wasm インスタンス・別 registry、キー `app`）
            // とは衝突しない。対象 0 件（`detail_page(None)`/`loader_error_view`
            // 等）のページへの遷移では空集合で差し替わる（旧リスナー解除のみ）。
            if let Err(_err) = rws_wasm_client::wire_hydrate_targets(&root.id(), &root) {
                // fail-safe: 再配線に失敗しても遷移自体（DOM 差し替え・URL・
                // タイトル更新）は既に成立させているため、ここでは継続する
                // （内部状態を含まない固定英語文言、不変条件 6 の継承）。
                web_sys::console::warn_1(
                    &"rws-wasm-full: nav render_route failed to wire data-hydrate targets".into(),
                );
            }
            post_apply();
        });
    }

    /// `route` を解決して `root_id` 配下を差し替える。`post_apply` の実行
    /// タイミングは [`apply_render_with_post`] を参照（描画が no-op の場合は
    /// 呼ばれない）。呼び出し元がいずれも `post_apply` を必要とする構成
    /// （`push_and_render` は先頭スクロール、`navigate_render_with_post`
    /// 経由の popstate はスクロール復元）のため、`post_apply` なしの単純形は
    /// 提供しない。
    fn render_route_with_post<P>(
        document: &Document,
        root_id: &str,
        route: &ClientRoute,
        post_apply: P,
    ) where
        P: FnOnce() + 'static,
    {
        let Some((title, new_dom_node)) = prepare_render(document, route) else {
            // 現在の DOM を維持したまま no-op（panic しない）。
            web_sys::console::warn_1(
                &"rws-wasm-full: nav render_route failed to build replacement DOM node".into(),
            );
            return;
        };
        apply_render_with_post(document, root_id, title, new_dom_node, post_apply);
    }

    /// `path`（`location.pathname` + `location.search` 相当）を再解決して
    /// 描画する。一致しないパスは no-op（現在の DOM を維持、安全側
    /// フォールバック）。戻り値は描画を実行したか（ルートが解決できたか）を
    /// 表す。
    ///
    /// `post_apply` は [`render_route_with_post`] と同じタイミングで実行
    /// される（描画が no-op の場合は呼ばれない）。呼び出し元（popstate
    /// クロージャ）はスクロール復元をこの `post_apply` に委ねており、戻り値
    /// 自体は使わない（描画が no-op なら `post_apply` が呼ばれないことで
    /// 「未解決パスはスクロールも含めて完全 no-op」の不変条件が保たれるため、
    /// 戻り値を見た事前分岐は不要、イシュー #406）。
    fn navigate_render_with_post<P>(
        document: &Document,
        root_id: &str,
        path: &str,
        post_apply: P,
    ) -> bool
    where
        P: FnOnce() + 'static,
    {
        let Some(route) = resolve_path(path) else {
            return false;
        };
        render_route_with_post(document, root_id, &route, post_apply);
        true
    }

    /// `state`（`PopStateEvent::state()`）をデコードして復元し、失敗時は
    /// 先頭 `(0, 0)` へフォールバックする（イシュー #406、§2.2 の popstate
    /// 挙動）。`state` が `JsValue::NULL`（本モジュールの `push_state` が
    /// 積む値）の場合も [`decode_scroll_state`] は `None` を返す
    /// （`JsValue::as_string` が `NULL` に対して `None` を返すため）。
    fn restore_scroll_from_popstate_state(window: &web_sys::Window, state: &JsValue) {
        let target = state
            .as_string()
            .and_then(|encoded| decode_scroll_state(&encoded))
            .unwrap_or((0.0, 0.0));
        window.scroll_to_with_x_and_y(target.0, target.1);
    }

    /// 現在の history エントリへ `(x, y)` を `replace_state` で書き戻す
    /// （イシュー #406、レビュー指摘 #423 対応: [`save_current_scroll_state`]
    /// と [`push_and_render`] の離脱元保存が同一のエンコード＋`replace_state`
    /// 手順を別々に実装していた重複を解消する共通ヘルパー）。
    ///
    /// 第 3 引数 `None` で現在の URL を維持したまま state のみを差し替える
    /// （呼び出し元の URL を書き換えない）。`replace_state_with_url` の失敗は
    /// best-effort で無視する（呼び出し元がいずれも失敗時にリトライする
    /// 機会を持たない・遷移や離脱自体を妨げてはならない箇所のため）。
    fn write_scroll_state_to_history(history: &web_sys::History, x: f64, y: f64) {
        let encoded = JsValue::from_str(&encode_scroll_state(x, y));
        let _ = history.replace_state_with_url(&encoded, "", None);
    }

    /// 現在の history エントリへ最新スクロール位置を書き戻す（イシュー #406
    /// 追加分、リロード時にスクロール位置が失われる不具合の修正）。
    ///
    /// [`push_and_render`] は**離脱元**エントリへのみ保存するため、
    /// `push_state` で新規に作られたエントリ自身の `state` は、そのページ上で
    /// ユーザーがスクロールしても `JsValue::NULL` のまま更新されない
    /// （そのままリロードすると復元先の記録が存在せず先頭へ戻ってしまう）。
    /// 本関数を `pagehide`（ドキュメント破棄直前。リロード・外部サイトへの
    /// 遷移・タブクローズ等、`popstate` を伴わない離脱を広く捕捉する）から
    /// 呼び出し、破棄直前の時点の現在エントリへスクロール位置を書き戻すことで、
    /// 次回ロード時に [`start_router`] が読み取れる状態にする。
    ///
    /// `window`/`history` の取得失敗・`scroll_x`/`scroll_y` の取得失敗は
    /// best-effort で無視する（`pagehide` はドキュメント破棄直前のため、
    /// 失敗時にリトライする機会がなく、遷移自体を妨げてはならない）。
    fn save_current_scroll_state() {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(history) = window.history() else {
            return;
        };
        if let (Ok(x), Ok(y)) = (window.scroll_x(), window.scroll_y()) {
            write_scroll_state_to_history(&history, x, y);
        }
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
    /// する。apply 段の `root` 再解決失敗（要素消失）はこの関数自身は検知
    /// できないため、呼び出し元の `start_router` クリックハンドラ側で
    /// `root_id` 要素の存在確認を `prevent_default`/本関数呼び出しの前提
    /// 条件としている（イシュー #404 レビュー指摘。要素消失時に
    /// `prevent_default` だけ呼んでブラウザ既定のフォールバック遷移まで
    /// 止めてしまう「詰み」状態を防ぐ）。
    ///
    /// イシュー #406: 遷移前に現在のスクロール位置を**離脱元エントリ**へ
    /// `replace_state` で保存してから `push_state` する。新規エントリの
    /// state は従来どおり `JsValue::NULL`（URL のみを状態の正とする不変
    /// 条件を維持）。先頭 `(0, 0)` へのスクロールは [`apply_render_with_post`]
    /// の `post_apply`（DOM 差し替えが実際に成立した後、View Transitions
    /// 対応ブラウザでは非同期）として実行し、`prepare_render` 失敗時は
    /// 早期 `return` により `post_apply` 自体が登録されないため、URL だけ
    /// 進んで旧ページがトップへスクロールされる不整合は生じない（Bugbot
    /// 指摘、PR #423 の意図をイシュー #404 の 2 段構成へ統合）。
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
                // 離脱元エントリのスクロール位置を保存する。`scroll_x`/
                // `scroll_y` 取得または [`write_scroll_state_to_history`]
                // の失敗は best-effort で無視し、遷移自体は継続する
                // （機能劣化のみで安全側）。
                if let (Ok(x), Ok(y)) = (window.scroll_x(), window.scroll_y()) {
                    write_scroll_state_to_history(&history, x, y);
                }
                // 新規エントリの state は従来どおり `JsValue::NULL`
                // （URL のみを状態の正とする、改ざん面を持たない設計判断）。
                // `pushState` は View Transitions の対象外（apply 段より前、
                // prepare 段と同じく同期実行のまま）。
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
            }
        }
        // 新規遷移は常にページ先頭から表示する（§2.2）。DOM 差し替えが
        // 実際に成立した後（`post_apply`）にのみスクロールすることで、
        // 差し替え前の旧ページの高さを基準にスクロールしてしまう不整合を
        // 避ける（Bugbot 指摘、PR #423）。
        apply_render_with_post(document, root_id, title, new_dom_node, || {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
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
    ///   history の往復のみに追従する）。ルート解決成功時のみ
    ///   `PopStateEvent::state()` をデコードしてスクロール位置を復元する
    ///   （成功なら保存位置へ・失敗/NULL なら先頭 `(0, 0)` へ、イシュー
    ///   #406）。ルート未解決パスはスクロールも含めて完全 no-op のまま。
    /// - `window` へ `pagehide` を 1 回だけ登録し、[`save_current_scroll_state`]
    ///   を呼ぶ（イシュー #406 追加分）。リロード・外部遷移・タブクローズ等、
    ///   `popstate` を伴わないドキュメント破棄の直前に現在エントリへスクロール
    ///   位置を書き戻すことで、リロード後も本関数の起動時処理が復元できる
    ///   ようにする（従来は `push_state` 直後のエントリの `state` が
    ///   `JsValue::NULL` のままのため、そのページ上でリロードするとスクロール
    ///   位置が失われていた）。
    /// - **起動時（本関数呼び出し時点）は描画を一切行わない**（SSR 済み
    ///   DOM をそのまま維持する。初期表示で loader を再実行しない凍結事項
    ///   の遵守、`docs/design/loader-trait-design.md` §4・§7.3）。
    ///   `history.scrollRestoration` を `"manual"` へ設定し（失敗は
    ///   best-effort で無視、機能劣化のみ）、現エントリの `history.state`
    ///   が有効なスクロールレコードであればその位置へ復元する
    ///   （リロード・クロスドキュメント traversal 後の復元。DOM 自体は
    ///   変更しない上記凍結事項と両立する）。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、または
    /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
    ///
    /// 同一 `root_id` で 2 回目以降に呼ばれた場合はリスナーを再登録せず
    /// `Ok(())` を返す（[`REGISTERED_ROOT_IDS`] 参照、イシュー #404
    /// フォローアップ）。この場合 `root_id` 要素の存在確認も行わない
    /// （初回呼び出し時点で既に確認済みのため）。
    pub fn start_router(root_id: &str) -> Result<(), JsValue> {
        let already_registered =
            REGISTERED_ROOT_IDS.with(|registered| registered.borrow().contains(root_id));
        if already_registered {
            return Ok(());
        }

        let doc = document()?;
        // `root_id` 要素の存在確認のみに使う（クロージャへは捕捉しない）。
        // 遷移のたびに `document.get_element_by_id` で再取得する方針
        // （下記クロージャ内コメント参照）のため、ここでの取得は起動時の
        // 前提チェック（要素が存在しない設定ミスを早期に `Err` で検出する）
        // に限定する。
        let _root = root_element(root_id)?;

        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                // ブラウザ既定の自動スクロール復元を止め、本モジュールが
                // 決定的に制御する（§2 の方針決定。失敗は best-effort で
                // 無視し、ブラウザ既定へフォールバックさせる）。
                let _ = history.set_scroll_restoration(ScrollRestoration::Manual);
                // リロード・クロスドキュメント traversal 直後は現エントリの
                // state が既にスクロールレコードを持ち得る（前回セッションの
                // `replace_state`/`push_state` が積んだ値）。デコードに成功
                // した場合のみ復元し、失敗/NULL（通常の初回ロード）では
                // 何もしない（SSR 済み DOM のみを表示する凍結事項の維持、
                // 起動時に先頭 `(0, 0)` を強制しない）。
                if let Some(encoded) = history.state().ok().and_then(|s| s.as_string()) {
                    if let Some((x, y)) = decode_scroll_state(&encoded) {
                        window.scroll_to_with_x_and_y(x, y);
                    }
                }
            }
        }

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
            // `root_id` 要素が現に存在する場合のみ `prevent_default` する
            // （イシュー #404 レビュー指摘。`push_and_render` は
            // `prepare_render` 成功後に `pushState` するため通常は URL/DOM
            // 不整合を防げるが、`root_id` 要素自体が消失している場合は
            // `prepare_render` は成功しうる一方 `apply_render` 側の
            // `get_element_by_id` が `None` になり no-op で終わる。この状態で
            // `prevent_default` を呼ぶと、ブラウザ既定のフルページ遷移
            // フォールバックも止めた上で `pushState` により URL のみが進み
            // DOM・タイトルが更新されない「詰み」状態になる。ここで存在確認
            // した上で `prevent_default`/`push_and_render` の実行有無を
            // 決めることで、要素消失時はブラウザ既定の遷移に委ねる
            // （fail-closed ではなく安全側フォールバック）。
            if click_document.get_element_by_id(&root_id_owned).is_none() {
                return;
            }
            mouse_event.prevent_default();
            push_and_render(&click_document, &root_id_owned, &value);
        });
        doc.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let popstate_document = doc.clone();
        let popstate_root_id = root_id.to_string();
        // ルート解決が成功した（＝実際に再描画した）場合のみスクロールを
        // 触る。未解決パスは従来どおり完全 no-op（イシュー #406、§2.2）。
        // スクロール復元は `navigate_render_with_post` の `post_apply` として
        // 渡し、DOM 差し替えが実際に成立した後（View Transitions 対応
        // ブラウザでは非同期）に実行する（イシュー #404 との統合、差し替え前
        // の旧ページ高さを基準に復元してしまう不整合の回避）。`post_apply` は
        // 描画が no-op の場合（ルート未解決・`root_id` 要素消失）は呼ばれない
        // ため、明示的な事前存在確認は不要。
        let popstate_closure =
            Closure::<dyn FnMut(PopStateEvent)>::new(move |event: PopStateEvent| {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let Ok(location_pathname) = window.location().pathname() else {
                    return;
                };
                let search = window.location().search().unwrap_or_default();
                let path = format!("{location_pathname}{search}");
                let state = event.state();
                let restore_window = window.clone();
                navigate_render_with_post(
                    &popstate_document,
                    &popstate_root_id,
                    &path,
                    move || {
                        restore_scroll_from_popstate_state(&restore_window, &state);
                    },
                );
            });
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .add_event_listener_with_callback(
                "popstate",
                popstate_closure.as_ref().unchecked_ref(),
            )?;
        popstate_closure.forget();

        // イシュー #406 追加分: リロード・外部遷移・タブクローズ等の
        // ドキュメント破棄直前に現在エントリへスクロール位置を書き戻す
        // （`popstate` を伴わない離脱では `push_and_render` の離脱元保存が
        // 発火しないため、そのままでは復元先の記録が残らない）。
        let pagehide_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            save_current_scroll_state();
        });
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .add_event_listener_with_callback(
                "pagehide",
                pagehide_closure.as_ref().unchecked_ref(),
            )?;
        pagehide_closure.forget();

        // click/popstate/pagehide リスナーの登録がすべて成功した場合にのみ
        // `root_id` を既登録として記録する（部分失敗時に「登録済み」と
        // 誤記録しない）。
        REGISTERED_ROOT_IDS.with(|registered| {
            registered.borrow_mut().insert(root_id.to_string());
        });

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

    // -------------------------------------------------------------
    // スクロール座標コーデック（イシュー #406）
    // -------------------------------------------------------------

    #[test]
    fn scroll_state_round_trips_through_encode_and_decode() {
        assert_eq!(
            decode_scroll_state(&encode_scroll_state(0.0, 500.0)),
            Some((0.0, 500.0))
        );
        assert_eq!(
            decode_scroll_state(&encode_scroll_state(120.5, 3000.25)),
            Some((120.5, 3000.25))
        );
    }

    #[test]
    fn decode_scroll_state_rejects_prefix_mismatch() {
        assert_eq!(decode_scroll_state("0,500"), None);
        assert_eq!(decode_scroll_state("rws-scroll-x:0,500"), None);
    }

    #[test]
    fn decode_scroll_state_rejects_non_numeric_values() {
        assert_eq!(decode_scroll_state("rws-scroll:abc,500"), None);
        assert_eq!(decode_scroll_state("rws-scroll:0,xyz"), None);
    }

    #[test]
    fn decode_scroll_state_rejects_non_finite_values() {
        assert_eq!(decode_scroll_state("rws-scroll:NaN,500"), None);
        assert_eq!(decode_scroll_state("rws-scroll:0,inf"), None);
        assert_eq!(decode_scroll_state("rws-scroll:0,infinity"), None);
    }

    #[test]
    fn decode_scroll_state_rejects_negative_values() {
        assert_eq!(decode_scroll_state("rws-scroll:-1,500"), None);
        assert_eq!(decode_scroll_state("rws-scroll:0,-1"), None);
    }

    #[test]
    fn decode_scroll_state_rejects_injection_like_strings() {
        // history state は改ざん可能な前提のため、HTML/script 風の文字列が
        // 紛れ込んでも復号は必ず失敗し、DOM/HTML へ流入する経路を持たない
        // ことを直接確認する。
        assert_eq!(
            decode_scroll_state("rws-scroll:<script>alert(1)</script>,0"),
            None
        );
        assert_eq!(decode_scroll_state(""), None);
        assert_eq!(decode_scroll_state("rws-scroll:"), None);
        assert_eq!(decode_scroll_state("rws-scroll:0"), None);
    }
}
