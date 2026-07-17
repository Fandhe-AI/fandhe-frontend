//! SSR エントリ（TASK-6.1c）: リクエストパスを rws-app のページ関数へ分岐なく
//! つなぎ、「ステータス・Content-Type・HTML 文字列」に文字列化する純関数を
//! 提供する。
//!
//! # 呼び出し文脈・契約
//!
//! - `docs/app-api.md` 第 4 節の設計判断 5（REQ-6/REQ-7 受け入れ基準）に従い、
//!   [`respond`] は [`rws_app::list_page`] / [`rws_app::detail_page`] /
//!   [`rws_app::page_shell`] を SSR・SSG・（将来の）CSR のいずれのモードからも
//!   **同一関数として分岐なく**呼ぶ。[`crate::ssg::generate`] は本関数が返す
//!   [`SsrResponse::body`] をそのままファイルへ書き出すことで、SSR/SSG の
//!   出力完全一致（REQ-6）を構造的に保証する。
//! - `server/src/main.rs`（SSR エントリの CLI 版）から呼ばれる想定。HTTP
//!   ソケット層は本関数の責務ではなく、`rws-dist-server`
//!   （`dist-server/src/routes.rs`）が本関数を呼んで HTTP レスポンスへ変換する
//!   （`docs/app-api.md` 追記: axum 不採用の実測根拠により、HTTP 配信は
//!   `rws-dist-server` の hyper 構成に委譲し、本クレートは外部依存ゼロを保つ）。
//! - ルーティングは [`crate::router::Router`]（外部依存ゼロ・パニックしない
//!   パスマッチング）を使う。ルーター自体はエスケープを行わない契約
//!   （`router.rs` 参照）であり、HTML 化は必ず rws-app 経由（既定エスケープ
//!   済み）で行う。`format!` によるタグ文字列の直接組み立ては行わない
//!   （`coding-rust.md`）。
//!
//! # セキュリティ不変条件
//!
//! - [`SsrResponse::content_type`] は固定 `&'static str` のみを返し、
//!   リクエスト由来の文字列をヘッダ相当の値へ流さない（ヘッダインジェクション
//!   対策）。
//! - 未知の `id`（`/items/:id`）は 404 ステータス + `detail_page(None)` の
//!   固定文言 HTML を返す。内部パス・スタックトレース等は含めない
//!   （`security.md`「機微情報の露出」）。
//! - 未一致パスは `None` を返すのみで `panic!` しない（呼び出し側が 404 応答を
//!   組み立てる）。

use crate::router::Router;
use rws_app::{demo_items, detail_page, list_page, page_shell, Item};
use std::sync::OnceLock;

/// [`Router`] に登録するページ種別。ハンドラ型は本モジュール外に公開しない
/// （`respond()` の内部実装詳細であり、呼び出し元は [`SsrResponse`] のみを
/// 契約として扱う）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageRoute {
    /// `/` — 一覧画面。
    List,
    /// `/items/:id` — 詳細画面。
    Detail,
}

/// [`respond`] の返り値。「HTTP レスポンス文字列化」（TASK-6.1 の SSR 定義）
/// の最小表現。HTTP ソケット層への変換は呼び出し元（`rws-dist-server` 等）の
/// 責務とする。
pub struct SsrResponse {
    /// HTTP ステータスコード相当。
    pub status: u16,
    /// 固定文言の Content-Type（リクエスト由来文字列を含まない）。
    pub content_type: &'static str,
    /// 既定エスケープ済み HTML 文字列（`rws_app::page_shell` の出力）。
    pub body: String,
}

/// `/`・`/items/:id` を登録した [`Router`] を構築する。
///
/// パターン文字列は開発者がハードコードした定数であり、エンドユーザー入力
/// ではないため `expect` してよい（`coding-rust.md` のエラー処理規約はエンド
/// ユーザー入力由来の失敗を panic させないことを求めるものであり、コンパイル
/// 時定数の妥当性はこの限りでない）。`/search` は rws-app の凍結 API に
/// search ページが存在しないため本 v1 では接続しない（スコープ外。
/// `docs/app-api.md` 追記・PR 本文に記録）。
fn build_page_router() -> Router<PageRoute> {
    Router::new()
        .route("/", PageRoute::List)
        .expect("static pattern \"/\" is valid")
        .route("/items/:id", PageRoute::Detail)
        .expect("static pattern \"/items/:id\" is valid")
}

/// `build_page_router()` の結果をプロセス生存期間中 1 回だけ構築してキャッシュ
/// する。ルート定義は固定（開発者がハードコードしたパターンのみ）であり実行時
/// に変化しないため、`OnceLock`（`std` のみ・追加依存なし）で使い回す
/// （`dist-server/src/routes.rs` の従前実装を踏襲）。
fn page_router() -> &'static Router<PageRoute> {
    static ROUTER: OnceLock<Router<PageRoute>> = OnceLock::new();
    ROUTER.get_or_init(build_page_router)
}

/// リクエストパスを解決し、[`SsrResponse`] を返す。
///
/// `rws-dist-server`（HTTP 配信）・`server/src/main.rs`（SSR エントリ CLI）・
/// [`crate::ssg::generate`]（SSG）のいずれからも同一実装を共有する
/// （REQ-6/REQ-7 受け入れ基準）。未一致パスは `None`（呼び出し側が 404 応答を
/// 組み立てる。本関数自身は固定 404 文言を持たない＝呼び出し文脈ごとの
/// 404 表現差異を許容する）。
pub fn respond(path: &str) -> Option<SsrResponse> {
    let route_match = page_router().resolve(path)?;
    let response = match route_match.handler {
        PageRoute::List => {
            let items = demo_items();
            let html = page_shell("記事一覧", list_page(&items));
            SsrResponse {
                status: 200,
                content_type: "text/html; charset=utf-8",
                body: html,
            }
        }
        PageRoute::Detail => {
            let items = demo_items();
            // `Params::get` は router（rws-server）の契約どおり生文字列を返す。
            // ここでは `Item::id` との文字列一致にのみ使い、HTML へは出力しない
            // （既定エスケープ対象外。`id` はもともと `String` フィールド）。
            let id = route_match.params.get("id");
            let item = id.and_then(|id| items.iter().find(|it: &&Item| it.id == id));
            match item {
                Some(item) => {
                    let html = page_shell("記事詳細", detail_page(Some(item)));
                    SsrResponse {
                        status: 200,
                        content_type: "text/html; charset=utf-8",
                        body: html,
                    }
                }
                None => {
                    // 未知の id: `detail_page(None)` が返す 404 相当のノードを
                    // そのまま描画し、ステータス 404 と HTML ボディを一致させる。
                    let html = page_shell("記事詳細", detail_page(None));
                    SsrResponse {
                        status: 404,
                        content_type: "text/html; charset=utf-8",
                        body: html,
                    }
                }
            }
        }
    };
    Some(response)
}

#[cfg(test)]
mod tests {
    use super::respond;

    #[test]
    fn list_page_returns_200_and_escapes_xss_payload_title() {
        let response = respond("/").expect("\"/\" should match");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "text/html; charset=utf-8");
        // demo_items()[1] の title は `<script>...` の XSS ペイロード。
        // 既定エスケープ（REQ-1）により `&lt;script&gt;` として出力されることを
        // 固定する回帰テスト。
        assert!(response.body.contains("&lt;script&gt;"));
        assert!(!response.body.contains("<script>alert"));
        assert!(response.body.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn detail_page_returns_200_for_known_id_and_404_for_unknown_id() {
        let found = respond("/items/1").expect("\"/items/1\" should match");
        assert_eq!(found.status, 200);
        assert!(found.body.contains("Rust 製フロントエンド基盤の構想"));

        let missing = respond("/items/999").expect("\"/items/999\" should match the pattern");
        assert_eq!(missing.status, 404);
        assert!(missing.body.contains("見つかりません"));
    }

    #[test]
    fn unmatched_path_returns_none() {
        assert!(respond("/no-such-page").is_none());
    }

    #[test]
    fn query_string_is_stripped_before_matching() {
        let list_with_query = respond("/?utm=1").expect("should match ignoring query string");
        assert_eq!(list_with_query.body, respond("/").unwrap().body);
    }
}
