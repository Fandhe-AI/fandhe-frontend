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
//!   後者はイシュー #406 のリロード時スクロール消失修正で追加）の
//!   `Closure::forget` に限定する（`events.rs` と同方針、無制限リークの
//!   構造的回避）。
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
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Document, Element, Event, MouseEvent, PopStateEvent, ScrollRestoration};

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
    /// フォールバック）。戻り値は描画を実行したか（popstate クロージャが
    /// スクロール復元を行ってよいかの判定に使う、イシュー #406）。
    fn navigate_render(document: &Document, root: &Element, path: &str) -> bool {
        let Some(route) = resolve_path(path) else {
            return false;
        };
        render_route(document, root, &route);
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

    /// 現在の history エントリへ最新スクロール位置を `replace_state` で
    /// 書き戻す（イシュー #406 追加分、リロード時にスクロール位置が失われる
    /// 不具合の修正）。
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
            let encoded = JsValue::from_str(&encode_scroll_state(x, y));
            // 第 3 引数 `None` で現在の URL を維持したまま state のみを
            // 差し替える（`push_and_render` の離脱元保存と同じ方針）。
            let _ = history.replace_state_with_url(&encoded, "", None);
        }
    }

    /// `path` が [`resolve_path`] で解決可能な場合のみ `history.pushState`
    /// で URL を進め、描画する（`popstate` からは呼ばない。history へ
    /// エントリを追加するのはユーザー操作起点のクリック遷移のみ）。
    ///
    /// イシュー #406: 遷移前に現在のスクロール位置を**離脱元エントリ**へ
    /// `replace_state` で保存してから `push_state` する。新規エントリの
    /// state は従来どおり `JsValue::NULL`（URL のみを状態の正とする不変
    /// 条件を維持）。描画後は先頭 `(0, 0)` へスクロールする（新規遷移は
    /// 常にページ先頭から表示する仕様、§2.2）。
    fn push_and_render(document: &Document, root: &Element, path: &str) {
        let Some(route) = resolve_path(path) else {
            return;
        };
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                // 離脱元エントリのスクロール位置を保存する。`scroll_x`/
                // `scroll_y` 取得または `replace_state` が失敗しても遷移
                // 自体は継続する（best-effort、機能劣化のみで安全側）。
                // `replace_state_with_url` の第 3 引数に `None` を渡すことで
                // 現在の URL を維持したまま state のみを差し替える
                // （離脱元の URL を書き換えない）。
                if let (Ok(x), Ok(y)) = (window.scroll_x(), window.scroll_y()) {
                    let encoded = JsValue::from_str(&encode_scroll_state(x, y));
                    let _ = history.replace_state_with_url(&encoded, "", None);
                }
                // 新規エントリの state は従来どおり `JsValue::NULL`
                // （URL のみを状態の正とする、改ざん面を持たない設計判断）。
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
            }
            // 新規遷移は常にページ先頭から表示する（§2.2）。失敗は無視
            // （best-effort、DOM 差し替え・URL 更新は既に成立済みのため
            // 遷移自体は継続する）。
            window.scroll_to_with_x_and_y(0.0, 0.0);
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
    pub fn start_router(root_id: &str) -> Result<(), JsValue> {
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
                let Some(current_root) = popstate_document.get_element_by_id(&popstate_root_id)
                else {
                    return;
                };
                // ルート解決が成功した（＝実際に再描画した）場合のみスクロールを
                // 触る。未解決パスは従来どおり完全 no-op（イシュー #406、§2.2）。
                if navigate_render(&popstate_document, &current_root, &path) {
                    restore_scroll_from_popstate_state(&window, &event.state());
                }
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
