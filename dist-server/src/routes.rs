//! HTTP に依存しないルート解決層。
//!
//! `main.rs`（hyper 接続処理）から呼ばれる純粋関数 [`route_request`] を提供する。
//! 本モジュールは `hyper` / `tokio` を一切知らず、`&str` のパスを受け取り
//! [`RouteResponse`]（ステータス・Content-Type・ボディ）を返すだけの
//! 同期関数として実装する。HTTP 層（ソケット・ヘッダ解析等）と分離することで
//! `tests/routes.rs`・本ファイルのユニットテストがサーバー起動なしに
//! ルーティング・エスケープ回帰を検証できる（TASK-9.1c の起動検証テストとは
//! 別レイヤー）。
//!
//! # ルーティング方針
//!
//! 1. `/static/` プレフィックスは [`assets::lookup`] へ委譲（コンパイル時
//!    埋め込みテーブルの完全一致検索。パストラバーサル不能、`assets.rs` 参照）。
//! 2. それ以外は `rws_server::router::Router<PageRoute>`（REQ-7 共通コア）で
//!    解決する。v1 の `Router` はワイルドカード（`*path`）に対応しないため、
//!    1 の `/static/` 分岐で文字列プレフィックス判定を手動補完している
//!    （`server/src/router.rs` のスコープ外事項、PR に記録）。
//! 3. いずれにも一致しなければ 404（本文は固定文言のみ。内部パス等を含めない
//!    ＝機微情報露出の回避、`security.md`）。
//!
//! # 既定エスケープの引き継ぎ（REQ-1）
//!
//! ページ本文の生成は [`rws_app::list_page`] / [`rws_app::detail_page`] /
//! [`rws_app::page_shell`]（いずれも `rws_core::text` 経由で既定エスケープ済み）
//! のみを呼ぶ。本モジュールが独自に `format!` で HTML を組み立てることはない
//! （`coding-rust.md`「HTML 文字列の直接組み立て禁止」）。

use crate::assets;
use crate::mime::content_type_for_path;
use rws_app::{demo_items, detail_page, list_page, page_shell, Item};
use rws_server::router::Router;

/// `rws_server::router::Router` に登録するページ種別。
///
/// ルーターはハンドラ型 `H` を不透明値として扱う契約（`server/src/router.rs`）
/// のため、実際の描画処理は [`route_request`] 側が `match` で行う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageRoute {
    /// `/` — 一覧画面。
    List,
    /// `/items/:id` — 詳細画面。
    Detail,
}

/// [`route_request`] の返り値。`main.rs` がこれを HTTP レスポンスへ変換する。
pub struct RouteResponse {
    /// HTTP ステータスコード。
    pub status: u16,
    /// 固定文言の `Content-Type`（リクエスト由来文字列を含まない）。
    pub content_type: &'static str,
    /// レスポンスボディ（HTML は既定エスケープ済み UTF-8、アセットは埋め込み済みバイト列）。
    pub body: Vec<u8>,
}

/// `PageRoute` 用ルーターを構築する。
///
/// パターンは `docs/spec/04-requirements.md` REQ-9 受け入れ基準の
/// `/`・`/items/:id` の 2 ルートのみ（`/search` は本タスクのスコープ外、
/// PoC-3 の 3 ルート構成のうち未接続分は TASK-6.1c 以降で扱う）。
/// パターン文字列はここでハードコードした開発者入力であり `unwrap` して良い
/// （`coding-rust.md` のエラー処理規約はエンドユーザー入力由来の失敗を panic
/// させないことを求めるものであり、コンパイル時定数の妥当性はこの限りでない）。
fn build_page_router() -> Router<PageRoute> {
    Router::new()
        .route("/", PageRoute::List)
        .expect("static pattern \"/\" is valid")
        .route("/items/:id", PageRoute::Detail)
        .expect("static pattern \"/items/:id\" is valid")
}

/// リクエストパス（クエリ文字列を含んでよい。`Router::resolve` が `?` 以降を
/// 切り落とす）を解決し、[`RouteResponse`] を返す。`main.rs` の hyper
/// サービス関数から 1 リクエストにつき 1 回呼ばれる。
pub fn route_request(path: &str) -> RouteResponse {
    if let Some(asset_path) = path.split('?').next().filter(|p| p.starts_with("/static/")) {
        return match assets::lookup(asset_path) {
            Some(bytes) => RouteResponse {
                status: 200,
                content_type: content_type_for_path(asset_path),
                body: bytes.to_vec(),
            },
            None => not_found(),
        };
    }

    let router = build_page_router();
    match router.resolve(path) {
        Some(route_match) => match route_match.handler {
            PageRoute::List => {
                let items = demo_items();
                let html = page_shell("記事一覧", list_page(&items));
                RouteResponse {
                    status: 200,
                    content_type: "text/html; charset=utf-8",
                    body: html.into_bytes(),
                }
            }
            PageRoute::Detail => {
                let items = demo_items();
                // `Params::get` はルーター（`rws-server`）の契約どおり生文字列を
                // 返す。ここでは `Item::id` との文字列一致に使うのみで HTML へは
                // 出力しないため、既定エスケープの対象外（数値変換もしない、
                // `id` は元々 `String` フィールド）。
                let id = route_match.params.get("id");
                let item = id.and_then(|id| items.iter().find(|it: &&Item| it.id == id));
                match item {
                    Some(item) => {
                        let html = page_shell("記事詳細", detail_page(Some(item)));
                        RouteResponse {
                            status: 200,
                            content_type: "text/html; charset=utf-8",
                            body: html.into_bytes(),
                        }
                    }
                    None => {
                        // 未知の id: `detail_page(None)` が返す 404 相当のノードを
                        // そのまま描画し、HTML ボディとステータス 404 を一致させる。
                        let html = page_shell("記事詳細", detail_page(None));
                        RouteResponse {
                            status: 404,
                            content_type: "text/html; charset=utf-8",
                            body: html.into_bytes(),
                        }
                    }
                }
            }
        },
        None => not_found(),
    }
}

/// 未一致パス共通の 404 応答。内部パス・スタックトレース等の機微情報を
/// 含まない固定文言のみ返す（`security.md`「機微情報の露出」）。
fn not_found() -> RouteResponse {
    RouteResponse {
        status: 404,
        content_type: "text/plain; charset=utf-8",
        body: b"404 Not Found".to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::route_request;

    fn body_as_string(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).expect("HTML body is UTF-8")
    }

    #[test]
    fn list_page_route_returns_200_and_escapes_xss_payload_title() {
        let response = route_request("/");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "text/html; charset=utf-8");
        let html = body_as_string(&response.body);
        // demo_items()[1] の title は `<script>...` の XSS ペイロード。
        // 既定エスケープ（REQ-1）により `&lt;script&gt;` として出力される
        // ことを固定する回帰テスト（網羅的な統合テストは TASK-9.4 のスコープ）。
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn detail_page_route_returns_200_for_known_id_and_404_for_unknown_id() {
        let found = route_request("/items/1");
        assert_eq!(found.status, 200);
        assert!(body_as_string(&found.body).contains("Rust 製フロントエンド基盤の構想"));

        let missing = route_request("/items/999");
        assert_eq!(missing.status, 404);
    }

    #[test]
    fn static_asset_route_serves_embedded_view_transitions_js() {
        let response = route_request("/static/view-transitions.js");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "text/javascript; charset=utf-8");
        assert!(body_as_string(&response.body).contains("withViewTransition"));
    }

    #[test]
    fn static_asset_traversal_attempts_return_404() {
        // assets::lookup の完全一致検索により、`../` を含むパス・URL エンコード
        // 済みパスのいずれもテーブルに一致せず 404 になることを固定する
        // （OWASP A01 パストラバーサル回帰テスト）。
        assert_eq!(route_request("/static/../Cargo.toml").status, 404);
        assert_eq!(route_request("/static/..%2FCargo.toml").status, 404);
    }

    #[test]
    fn unknown_path_returns_404_without_leaking_internal_details() {
        let response = route_request("/no-such-page");
        assert_eq!(response.status, 404);
        let body = body_as_string(&response.body);
        assert!(!body.contains("Cargo"));
        assert!(!body.contains("/home/"));
    }
}
