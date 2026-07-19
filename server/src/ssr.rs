//! SSR エントリ（TASK-6.1c・#348）: リクエストパスを fandhe-frontend-app の `Loader` 経由
//! データ解決 + ページ関数へ分岐なくつなぎ、「ステータス・Content-Type・HTML
//! 文字列」に文字列化する純関数を提供する。
//!
//! # 呼び出し文脈・契約
//!
//! - `docs/api/app-api.md` 第 4 節の設計判断 5（REQ-6/REQ-7 受け入れ基準）に従い、
//!   [`respond`] / [`respond_with`] は [`fandhe_frontend_app::list_page`] /
//!   [`fandhe_frontend_app::detail_page`] / [`fandhe_frontend_app::page_shell`] を SSR・SSG・（将来の）
//!   CSR のいずれのモードからも**同一関数として分岐なく**呼ぶ。
//!   [`crate::ssg::generate_with`] は本関数が返す [`SsrResponse::body`] を
//!   そのままファイルへ書き出すことで、SSR/SSG の出力完全一致（REQ-6）を
//!   構造的に保証する（SSG が独自に loader を呼ぶ描画経路は新設しない。
//!   `docs/design/loader-trait-design.md` §4）。
//! - `server/src/main.rs`（SSR エントリの CLI 版）から呼ばれる想定。HTTP
//!   ソケット層は本関数の責務ではなく、`fandhe-frontend-dist-server`
//!   （`dist-server/src/routes.rs`）が本関数を呼んで HTTP レスポンスへ変換する
//!   （`docs/api/app-api.md` 追記: axum 不採用の実測根拠により、HTTP 配信は
//!   `fandhe-frontend-dist-server` の hyper 構成に委譲し、本クレートは外部依存ゼロを保つ）。
//! - ルーティングは [`fandhe_frontend_app::routes`]（イシュー #407: server / client 単一
//!   ルート定義の共有機構、`fandhe-frontend-app` に集約したエンジン
//!   [`fandhe_frontend_app::router::Router`] を経由）を使う。ルート定義（パターン +
//!   ページタイトル）は `fandhe-frontend-app` 側の単一定義であり、本ファイルではパターン
//!   リテラル・タイトルリテラルを再定義しない（`wasm-full/src/nav.rs` も
//!   同じ `fandhe_frontend_app::routes` を参照する。`wasm-full/tests/route_shared_static.rs`
//!   が静的走査で再定義がないことを固定する）。ルーター自体はエスケープを
//!   行わない契約であり、HTML 化は必ず fandhe-frontend-app 経由（既定エスケープ済み）で
//!   行う。`format!` によるタグ文字列の直接組み立ては行わない
//!   （`coding-rust.md`）。
//! - データ取得は [`fandhe_frontend_app::Loader`]（#347・イシュー #346 設計確定書）経由に
//!   統一する。[`respond`] は既定 loader（[`fandhe_frontend_app::DemoItemsLoader`] /
//!   [`fandhe_frontend_app::DemoItemDetailLoader`]）で [`respond_with`] を呼ぶ薄い
//!   互換エントリであり、公開シグネチャは #347 以前から非破壊（`main.rs`・
//!   `dist-server` は無修正のまま利用継続できる）。
//!
//! # セキュリティ不変条件
//!
//! - [`SsrResponse::content_type`] は固定 `&'static str` のみを返し、
//!   リクエスト由来の文字列をヘッダ相当の値へ流さない（ヘッダインジェクション
//!   対策）。
//! - 未知の `id`（`/items/:id`）は 404 ステータス + `detail_page(None)` の
//!   固定文言 HTML を返す（`Loader::Error` ではなく `Output = None` として
//!   表現される契約。設計書 §3.3）。内部パス・スタックトレース等は含めない
//!   （`security.md`「機微情報の露出」）。
//! - loader が [`fandhe_frontend_app::Loader::Error`] を返した場合は
//!   [`loader_error_response`] が組み立てる 500 固定文言応答を返す
//!   （fail-closed、設計書 §5）。**`L::Error` / `D::Error` の値自体は一切
//!   参照しない**（`Display`/`Debug` を呼ばない）ため、loader 実装が内部
//!   パス・接続情報等を `Error` に含めていても応答へ混入する経路が構造的に
//!   存在しない（`security.md`「機微情報の露出」・設計書 §9-5）。
//! - 未一致パスは `None` を返すのみで `panic!` しない（呼び出し側が 404 応答を
//!   組み立てる）。

use fandhe_frontend_app::routes::{resolve as resolve_route, title as route_title, AppRoute};
use fandhe_frontend_app::{
    detail_page, list_page, page_shell, DemoItemDetailLoader, DemoItemsLoader, Item, Loader,
};

/// [`respond`] の返り値。「HTTP レスポンス文字列化」（TASK-6.1 の SSR 定義）
/// の最小表現。HTTP ソケット層への変換は呼び出し元（`fandhe-frontend-dist-server` 等）の
/// 責務とする。
pub struct SsrResponse {
    /// HTTP ステータスコード相当。
    pub status: u16,
    /// 固定文言の Content-Type（リクエスト由来文字列を含まない）。
    pub content_type: &'static str,
    /// 既定エスケープ済み HTML 文字列（`fandhe_frontend_app::page_shell` の出力）。
    pub body: String,
}

/// リクエストパスを解決し、既定 loader（[`DemoItemsLoader`] /
/// [`DemoItemDetailLoader`]）で [`SsrResponse`] を返す。
///
/// `fandhe-frontend-dist-server`（HTTP 配信）・`server/src/main.rs`（SSR エントリ CLI）・
/// [`crate::ssg::generate`]（SSG）のいずれからも同一実装を共有する
/// （REQ-6/REQ-7 受け入れ基準）。loader を差し替えたい呼び出し元（テスト等）は
/// [`respond_with`] を直接使う。公開シグネチャは #347 以前から非破壊
/// （`main.rs`・`dist-server` は無修正のまま利用継続できる）。
pub fn respond(path: &str) -> Option<SsrResponse> {
    respond_with(&DemoItemsLoader, &DemoItemDetailLoader, path)
}

/// loader を差し替え可能なジェネリック版。`where` 束縛で `Loader::Output` を
/// ページ関数（[`fandhe_frontend_app::list_page`] / [`fandhe_frontend_app::detail_page`]）の引数型へ
/// 型接続する（設計書 §3.4「型で保証する範囲」を server 側にも適用）。
///
/// 一覧・詳細のいずれも `fandhe_frontend_app::assemble_list_page` /
/// `assemble_detail_page`（`Node` のみを返すヘルパー）は使わず、
/// `loader.load()` → 判定 → ページ関数の順で自前に組み立てる。理由は
/// ルートごとに異なる:
///
/// - 一覧（`/`）: `list_loader.load(&())` を解決し `list_page(&items)` を
///   呼ぶ。`Err(_)` は [`loader_error_response`] の 500 固定文言応答に変換
///   する（`Ok`/`Err` の 2 値のみで `Vec<Item>` の中身自体はステータス判定に
///   関与しない）。`assemble_list_page` ではなく `list_page` を直接呼ぶのは、
///   `core/tests/no_branching_across_modes.rs` の REQ-7 静的検証
///   （`both_call_sites_reference_shared_app_functions_without_redefining`）
///   が本ファイルで `fandhe_frontend_app::list_page` への直接参照（`use` import 経由を
///   含む）を要求するため。`assemble_list_page` 経由では内部で `list_page`
///   を呼んでいても本ファイル上に識別子が現れず検証に通らない。機能的には
///   `assemble_list_page(list_loader, &())` と等価であり、`where` 束縛
///   （`L: Loader<Output = Vec<Item>>`）による型接続の保証も同様に働く。
/// - 詳細（`/items/:id`）: `detail_loader.load(&id)` → `Option` 判定 →
///   [`detail_page`] の順で組み立てる（後述）。`Result` の中身
///   （`Option<Item>` が `Some`/`None` のどちらか）が HTTP ステータス
///   （200/404）の判定材料であり、`Node` のみを返す `assemble_detail_page`
///   ではこの情報が失われるため直接組み立てる。
pub fn respond_with<L, D>(list_loader: &L, detail_loader: &D, path: &str) -> Option<SsrResponse>
where
    L: Loader<Input = (), Output = Vec<Item>>,
    D: Loader<Input = String, Output = Option<Item>>,
{
    // ルート解決は `fandhe_frontend_app::routes::resolve`（イシュー #407 の単一定義）へ
    // 委譲する。パターンリテラル・意味論（クエリ除去・末尾スラッシュ厳格
    // 一致等）は本ファイルで再定義しない。
    let resolved = resolve_route(path)?;
    let response = match resolved.route {
        AppRoute::List => match list_loader.load(&()) {
            Ok(items) => SsrResponse {
                status: 200,
                content_type: "text/html; charset=utf-8",
                body: page_shell(route_title(AppRoute::List), list_page(&items)),
            },
            Err(_) => loader_error_response(),
        },
        AppRoute::Detail => {
            // `resolved.id` は `fandhe_frontend_app::routes::resolve` の契約どおり生文字列
            // を返す。loader への入力（`String`）としてのみ使い、HTML へは
            // 出力しない（既定エスケープ対象外。`Item::id` はもともと
            // `String` フィールド）。
            let id = match resolved.id {
                Some(id) => id,
                // ルートパターン `/items/:id` は `id` を必ずキャプチャするため
                // 通常到達しないが、`fandhe_frontend_app::routes::resolve` の内部実装変更
                // に対する防御として 404 応答（機微情報を含まない）に
                // フォールバックする。
                None => {
                    let html = page_shell(route_title(AppRoute::Detail), detail_page(None));
                    return Some(SsrResponse {
                        status: 404,
                        content_type: "text/html; charset=utf-8",
                        body: html,
                    });
                }
            };
            match detail_loader.load(&id) {
                Ok(Some(item)) => {
                    let html = page_shell(route_title(AppRoute::Detail), detail_page(Some(&item)));
                    SsrResponse {
                        status: 200,
                        content_type: "text/html; charset=utf-8",
                        body: html,
                    }
                }
                Ok(None) => {
                    // 未知の id: `detail_page(None)` が返す 404 相当のノードを
                    // そのまま描画し、ステータス 404 と HTML ボディを一致させる
                    // （見つからない、は loader の `Error` ではなく `Output` の
                    // 一部。設計書 §3.3）。
                    let html = page_shell(route_title(AppRoute::Detail), detail_page(None));
                    SsrResponse {
                        status: 404,
                        content_type: "text/html; charset=utf-8",
                        body: html,
                    }
                }
                Err(_) => loader_error_response(),
            }
        }
    };
    Some(response)
}

/// loader 解決失敗時の fail-closed 応答（設計書 §5）。
///
/// **呼び出し元は `Loader::Error` の値をこの関数へ渡さない**（意図的にシグ
/// ネチャに含めない）。`Display`/`Debug` を一切経由しない構造にすることで、
/// loader 実装が `Error` に内部パス・接続情報等を含めていても応答本文へ
/// 混入する経路が型レベルで存在しない（`security.md`「機微情報の露出」・
/// 設計書 §9-5）。body はノード木 API（[`page_shell`]）経由で組み立て、
/// `format!` によるタグ文字列の直接組み立ては行わない（REQ-1）。
fn loader_error_response() -> SsrResponse {
    let body = page_shell(
        "Internal Server Error",
        fandhe_frontend_core::p(
            vec![],
            vec![fandhe_frontend_core::text(
                "An internal error occurred while loading data.",
            )],
        ),
    );
    SsrResponse {
        status: 500,
        content_type: "text/html; charset=utf-8",
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::{respond, respond_with};
    use fandhe_frontend_app::{DemoItemDetailLoader, DemoItemsLoader, Item, Loader};

    /// 受け入れ条件 2 の直接証明用フィクスチャ: 一覧 loader が必ず失敗する。
    /// `Error` にダミーの機微情報風文字列を含め、[`respond_with`] の 500
    /// 応答本文へ一切混入しないことを検証する（内部パス・接続情報の非露出）。
    struct FailingListLoader;

    impl Loader for FailingListLoader {
        type Input = ();
        type Output = Vec<Item>;
        type Error = String;

        fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
            Err("db_password=dummy-secret /internal/path".to_string())
        }
    }

    /// 詳細 loader 版の失敗フィクスチャ（[`FailingListLoader`] と同様）。
    struct FailingDetailLoader;

    impl Loader for FailingDetailLoader {
        type Input = String;
        type Output = Option<Item>;
        type Error = String;

        fn load(&self, _input: &String) -> Result<Option<Item>, String> {
            Err("db_password=dummy-secret /internal/path".to_string())
        }
    }

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

    /// 受け入れ条件 2: 一覧 loader が失敗した場合、500 固定文言応答になり、
    /// `Loader::Error` に含めたダミー機微文字列が body・content_type の
    /// いずれにも一切含まれないこと（fail-closed の直接証明）。
    #[test]
    fn list_route_returns_500_fixed_message_when_loader_fails_and_leaks_nothing() {
        let response = respond_with(&FailingListLoader, &DemoItemDetailLoader, "/")
            .expect("\"/\" should still match the route pattern");
        assert_eq!(response.status, 500);
        assert_eq!(response.content_type, "text/html; charset=utf-8");
        assert!(response.body.contains("Internal Server Error"));
        assert!(!response.body.contains("db_password"));
        assert!(!response.body.contains("dummy-secret"));
        assert!(!response.body.contains("/internal/path"));
    }

    /// 詳細 loader 版の fail-closed 回帰。既知 id でも loader が失敗すれば
    /// 500 になり、`Ok(None)`（未知 id）は従来どおり 404 のままであること
    /// （`Error` と `Output` の一部としての「見つからない」の区別を固定）。
    #[test]
    fn detail_route_returns_500_fixed_message_when_loader_fails_and_leaks_nothing() {
        let response = respond_with(&DemoItemsLoader, &FailingDetailLoader, "/items/1")
            .expect("\"/items/1\" should still match the route pattern");
        assert_eq!(response.status, 500);
        assert!(response.body.contains("Internal Server Error"));
        assert!(!response.body.contains("db_password"));
        assert!(!response.body.contains("dummy-secret"));
        assert!(!response.body.contains("/internal/path"));

        // 対照: 成功する既定 loader での未知 id は従来どおり 404（Error ではない）。
        let missing = respond_with(&DemoItemsLoader, &DemoItemDetailLoader, "/items/999")
            .expect("\"/items/999\" should match the pattern");
        assert_eq!(missing.status, 404);
    }

    /// `respond()` は既定 loader で `respond_with` を呼ぶ薄いラッパーであり、
    /// 既存 4 テストが証明する互換性がジェネリック版導入後も破れないことを
    /// 直接固定する。
    #[test]
    fn respond_matches_respond_with_default_loaders() {
        for path in ["/", "/items/1", "/items/999"] {
            let via_respond = respond(path).expect("known path should match");
            let via_respond_with = respond_with(&DemoItemsLoader, &DemoItemDetailLoader, path)
                .expect("known path should match");
            assert_eq!(via_respond.status, via_respond_with.status);
            assert_eq!(via_respond.body, via_respond_with.body);
        }
    }
}
