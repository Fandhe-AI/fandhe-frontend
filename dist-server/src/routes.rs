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
//! 1. `/static/` プレフィックスは [`assets::lookup`] へ委譲（開発 / 本番
//!    モードに応じてファイルシステム読み込み・コンパイル時埋め込みテーブル
//!    検索を切り替える。いずれもパストラバーサル不能、`assets.rs` 参照
//!    — TASK-10.1a、イシュー #106）。開発モード（`DevFilesystem`）の 200
//!    応答には [`RouteResponse::cache_control`] に `Some("no-store")` を
//!    設定し、ブラウザキャッシュがディスクの即時反映（REQ-10）を体感上
//!    無効化しないようにする（TASK-10.1b、イシュー #107）。
//! 2. それ以外は [`rws_server::ssr::respond`]（TASK-6.1c の SSR コア）へ
//!    委譲する。ページ解決・rws-app 呼び出しの実体は `rws-server` 側の
//!    単一実装であり、本モジュールは HTTP レスポンス表現（[`RouteResponse`]）
//!    への詰め替えのみを行う。`rws_server::ssr` 内部の `Router` は
//!    ワイルドカード（`*path`）に対応しないため、1 の `/static/` 分岐で
//!    文字列プレフィックス判定を手動補完している（`server/src/router.rs`
//!    のスコープ外事項、PR に記録）。
//! 3. いずれにも一致しなければ 404（本文は固定文言のみ。内部パス等を含めない
//!    ＝機微情報露出の回避、`security.md`）。
//!
//! # 既定エスケープの引き継ぎ（REQ-1）
//!
//! ページ本文の生成は [`rws_server::ssr::respond`] を経由して
//! [`rws_app::list_page`] / [`rws_app::detail_page`] / [`rws_app::page_shell`]
//! （いずれも `rws_core::text` 経由で既定エスケープ済み）のみを呼ぶ。
//! 本モジュールが独自に `format!` で HTML を組み立てることはない
//! （`coding-rust.md`「HTML 文字列の直接組み立て禁止」）。

use crate::assets;
use crate::mime::content_type_for_path;
use rws_server::ssr::respond;

/// [`route_request`] の返り値。`main.rs` がこれを HTTP レスポンスへ変換する。
pub struct RouteResponse {
    /// HTTP ステータスコード。
    pub status: u16,
    /// 固定文言の `Content-Type`（リクエスト由来文字列を含まない）。
    pub content_type: &'static str,
    /// レスポンスボディ（HTML は既定エスケープ済み UTF-8、アセットは埋め込み済みバイト列）。
    pub body: Vec<u8>,
    /// `Some` のとき `main.rs` が `Cache-Control` ヘッダとして付与する固定文言。
    ///
    /// [`assets::AssetMode::DevFilesystem`] の静的アセット応答のみ
    /// `Some("no-store")` を返し、ブラウザキャッシュにより「毎リクエストで
    /// ディスクの最新内容を読む」（REQ-10 即時反映）が体感上無効化されるのを
    /// 防ぐ（TASK-10.1b、イシュー #107）。ページ応答・404・
    /// [`assets::AssetMode::Embedded`] の場合は `None` とし、本番のキャッシュ
    /// 挙動を変更しない。値は必ず固定文言の `&'static str` のみとし、
    /// リクエスト由来文字列をヘッダへ流さない（ヘッダインジェクション対策、
    /// `security.md`）。
    pub cache_control: Option<&'static str>,
}

/// リクエストパス（クエリ文字列を含んでよい。`rws_server::ssr::respond` 内部
/// の `Router::resolve` が `?` 以降を切り落とす）を解決し、[`RouteResponse`]
/// を返す。`main.rs` の hyper サービス関数から 1 リクエストにつき 1 回呼ばれる。
pub fn route_request(path: &str) -> RouteResponse {
    if let Some(asset_path) = path.split('?').next().filter(|p| p.starts_with("/static/")) {
        // `assets::lookup` は `Cow<'static, [u8]>` を返す（埋め込みモードは
        // 借用、開発モードは所有バイト列。`assets.rs` の doc 参照）。
        // `RouteResponse::body` は `Vec<u8>` 固定のため `into_owned()` で
        // 両モードを同一に扱う。
        return match assets::lookup(asset_path) {
            Some(bytes) => RouteResponse {
                status: 200,
                content_type: content_type_for_path(asset_path),
                body: bytes.into_owned(),
                // 開発モード（DevFilesystem）の応答のみ no-store を付与し、
                // ブラウザキャッシュ越しに古いアセットが表示され続けるのを防ぐ
                // （`RouteResponse::cache_control` の doc 参照）。本番
                // （Embedded）は既存どおりキャッシュ制御ヘッダを付けない。
                cache_control: matches!(assets::active_mode(), assets::AssetMode::DevFilesystem)
                    .then_some("no-store"),
            },
            None => not_found(),
        };
    }

    // ページ解決の実体は `rws_server::ssr::respond`（TASK-6.1c の SSR コア）
    // に一本化されている。本モジュールは HTTP レスポンス表現への詰め替えのみ
    // 行い、`rws-app` のページ関数を直接呼ばない（重複実装の回避、REQ-6）。
    match respond(path) {
        Some(ssr_response) => RouteResponse {
            status: ssr_response.status,
            content_type: ssr_response.content_type,
            body: ssr_response.body.into_bytes(),
            cache_control: None,
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
        cache_control: None,
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
    fn page_routes_with_query_string_resolve_like_bare_paths() {
        // `route_request` 自体はクエリを剥がさず `rws_server::router::Router::resolve`
        // に生パスを渡す（本ファイル冒頭のドキュメンテーションコメント参照）。
        // `Router::resolve` 内部の `path.split_once('?')` によるクエリ除去
        // （`server/src/router.rs`）に暗黙依存しているため、その挙動をこちらの
        // 層でも固定する回帰テスト（Review 指摘: テストギャップの解消）。
        let list_with_query = route_request("/?utm=1");
        assert_eq!(list_with_query.status, 200);
        assert_eq!(list_with_query.body, route_request("/").body);

        let detail_with_query = route_request("/items/1?utm=1&ref=x");
        assert_eq!(detail_with_query.status, 200);
        assert_eq!(detail_with_query.body, route_request("/items/1").body);
    }

    #[test]
    fn unknown_path_returns_404_without_leaking_internal_details() {
        let response = route_request("/no-such-page");
        assert_eq!(response.status, 404);
        let body = body_as_string(&response.body);
        assert!(!body.contains("Cargo"));
        assert!(!body.contains("/home/"));
    }

    #[test]
    fn page_and_404_responses_never_set_cache_control() {
        // ページ応答・404 応答は開発 / 本番モードによらず常に `None`
        // （`Cache-Control` を付与するのは開発モードの静的アセット応答のみ、
        // `RouteResponse::cache_control` の doc 参照）。
        assert_eq!(route_request("/").cache_control, None);
        assert_eq!(route_request("/items/1").cache_control, None);
        assert_eq!(route_request("/items/999").cache_control, None);
        assert_eq!(route_request("/no-such-page").cache_control, None);
    }

    // 静的アセット応答の `cache_control` はビルド構成（開発 / 本番モード）に
    // よって固定値が変わるため、`assets.rs` の `active_mode_is_*` テストと
    // 同じ cfg ゲートでモードごとに固定する（TASK-10.1b、イシュー #107）。
    #[cfg(all(debug_assertions, not(feature = "force-embed")))]
    #[test]
    fn static_asset_response_sets_no_store_cache_control_in_dev_filesystem_mode() {
        let response = route_request("/static/view-transitions.js");
        assert_eq!(response.status, 200);
        assert_eq!(response.cache_control, Some("no-store"));
    }

    #[cfg(not(all(debug_assertions, not(feature = "force-embed"))))]
    #[test]
    fn static_asset_response_has_no_cache_control_in_embedded_mode() {
        let response = route_request("/static/view-transitions.js");
        assert_eq!(response.status, 200);
        assert_eq!(response.cache_control, None);
    }
}
