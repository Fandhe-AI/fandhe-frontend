//! `fandhe-frontend-dist-server` の起動エントリ。
//!
//! bind 先アドレスは次の優先順位で決定する（イシュー #3336、#3335 で確定した
//! Vercel Container Images 向け仕様）:
//!
//! 1. `FANDHE_FRONTEND_BIND_ADDR` が設定されていればその値をそのまま使う（従来どおり）
//! 2. 未設定で `PORT` が設定されていれば `0.0.0.0:$PORT`（Vercel Container Images が
//!    既定で要求する「`PORT` 環境変数でポートを受け取り全インターフェースで listen する」
//!    契約に応える経路）
//! 3. どちらも未設定なら既定 `127.0.0.1:3100`
//!
//! 空文字（`FANDHE_FRONTEND_BIND_ADDR=` / `PORT=`）はコンテナ実行時に `-e` で環境変数を
//! 打ち消せるよう「未設定」として扱う。決定ロジックは [`resolve_bind_addr`]（プロセス
//! 環境変数を読まない純粋関数）に切り出してあり、`run()` のみがプロセス環境を読む。
//!
//! 1 接続ごとに `hyper`（HTTP/1.1）で処理する。実際のルーティング・レスポンス生成は
//! `fandhe_frontend_dist_server::routes::route_request`（HTTP に依存しない純粋関数）に
//! 委譲し、本ファイルは「hyper の接続を受けてバイト列に変換する」薄い
//! トランスポート層のみを担う。
//!
//! # セキュリティ設定（`security.md` A05 セキュリティ設定ミス）
//!
//! 既定 bind アドレスはループバック（`127.0.0.1`）とし、外部公開は
//! `FANDHE_FRONTEND_BIND_ADDR` または `PORT` の明示的なオプトインを要求する。bind 失敗時は
//! `panic!` せず、アドレスと OS エラーのみを stderr に出力して非 0 終了する
//! （内部パス・スタックトレース等の機微情報は出力しない）。`PORT` の値が不正な場合も
//! 黙って既定値へフォールバックせず起動を失敗させ（fail-closed）、エラーメッセージには
//! `PORT` の実際の値を含めない（カテゴリのみの固定英語文言、機微情報を露出しない）。

// `lib.rs` の `#![forbid(unsafe_code)]`（REQ-2）はクレートルートを跨いで継承
// されないため、バイナリクレートルートである本ファイルにも明示的に付与し、
// 宣言の一貫性を保つ（実装上も unsafe は使用していない）。
#![forbid(unsafe_code)]

use fandhe_frontend_dist_server::assets::{active_mode, AssetMode};
use fandhe_frontend_dist_server::routes::route_request;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::process::ExitCode;
use tokio::net::TcpListener;

/// 既定の bind アドレス。`FANDHE_FRONTEND_BIND_ADDR` も `PORT` も未設定のときに使う
/// （ループバック限定。`security.md` 参照）。
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3100";

/// [`resolve_bind_addr`] が返しうる失敗の種類。
///
/// `Display` は固定の英語文言のみを返し、実際の環境変数値は含めない
/// （`PORT` の不正値をそのままエラーメッセージへ埋め込むと、ログ経由で
/// 内部設定情報が露出しうるため。`security.md` A09 参照）。このバイナリ
/// クレート内に閉じた私有型であり、`lib.rs` の公開 API には影響しない。
#[derive(Debug, PartialEq, Eq)]
enum BindAddrError {
    /// `PORT` が UTF-8 として読めない、数値としてパースできない、または
    /// `1..=65535` の範囲外（`0` を含む）だった。
    InvalidPort,
}

impl std::fmt::Display for BindAddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindAddrError::InvalidPort => {
                write!(f, "PORT must be a valid port number between 1 and 65535")
            }
        }
    }
}

/// 空文字を「未設定」に正規化する。
///
/// コンテナ実行時に `-e FANDHE_FRONTEND_BIND_ADDR=` / `-e PORT=` で環境変数を空文字に
/// 打ち消せるようにするための正規化（イシュー #3336 で確定した仕様）。
fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

/// bind 先アドレスを決定する純粋関数。
///
/// プロセス環境変数を直接読まず、呼び出し側（`run()`）が読み取った値を
/// 引数として受け取ることで、ユニットテスト側がプロセス環境を書き換え
/// ずに全分岐を検証できるようにする（`tests/bind_addr.rs` と役割分担）。
///
/// 優先順位はモジュール冒頭の `//!` ドキュメントを正とする:
/// `bind_addr`（`FANDHE_FRONTEND_BIND_ADDR` の値）が設定されていれば
/// **`port` の値を一切検証せずに**そのまま採用する。`bind_addr` が未設定
/// のときのみ `port`（`PORT` の値）を検証し、有効なら `0.0.0.0:$port`、
/// 無効なら [`BindAddrError::InvalidPort`] を返す。両方未設定なら
/// [`DEFAULT_BIND_ADDR`]。
fn resolve_bind_addr(bind_addr: Option<&str>, port: Option<&str>) -> Result<String, BindAddrError> {
    if let Some(bind_addr) = non_empty(bind_addr) {
        return Ok(bind_addr.to_string());
    }

    match non_empty(port) {
        Some(port) => match port.parse::<u16>() {
            Ok(0) | Err(_) => Err(BindAddrError::InvalidPort),
            Ok(port) => Ok(format!("0.0.0.0:{port}")),
        },
        None => Ok(DEFAULT_BIND_ADDR.to_string()),
    }
}

fn main() -> ExitCode {
    // `#[tokio::main]`（tokio-macros、依存グラフ深さ増の一因）を使わず、
    // `Builder` を直接呼ぶ（`Cargo.toml` の REQ-3 実測コメント参照）。
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("fandhe-frontend-dist-server: failed to start tokio runtime: {err}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(run())
}

/// 非同期本体。bind 成功後は無限に接続を受け付け続ける
/// （通常運用では戻らない。bind 失敗時のみ `ExitCode::FAILURE` を返す）。
async fn run() -> ExitCode {
    // `FANDHE_FRONTEND_BIND_ADDR` は非 UTF-8 値を「未設定」扱いへ畳み込む（既存動作を維持）。
    let bind_addr_env = std::env::var("FANDHE_FRONTEND_BIND_ADDR").ok();
    // `PORT` は非 UTF-8 値を「不正な値」としてエラー経路へ倒す（黙ってフォールバック
    // しない、という受け入れ条件の趣旨に合わせる。`var_os` + `to_str()` で判定する）。
    let port_env_os = std::env::var_os("PORT");
    let port_env = match &port_env_os {
        Some(value) => match value.to_str() {
            Some(value) => Some(value),
            None => {
                eprintln!(
                    "fandhe-frontend-dist-server: {}",
                    BindAddrError::InvalidPort
                );
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };

    let bind_addr = match resolve_bind_addr(bind_addr_env.as_deref(), port_env) {
        Ok(bind_addr) => bind_addr,
        Err(err) => {
            eprintln!("fandhe-frontend-dist-server: {err}");
            return ExitCode::FAILURE;
        }
    };

    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            // bind 失敗はアドレスと OS エラーのみを出力する（機微情報を含めない）。
            eprintln!("fandhe-frontend-dist-server: failed to bind {bind_addr}: {err}");
            return ExitCode::FAILURE;
        }
    };
    // `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（ポート 0 = OS 割当）で起動した場合、
    // 設定文字列 `bind_addr` をそのままログに出すと実際に割り当てられた
    // ポート番号が外部から分からない。`listener.local_addr()`（bind 済み
    // ソケットの実アドレス）を使うことで、TASK-9.1c（イシュー #97）の
    // 実プロセス起動検証テスト（`tests/boot.rs`）が stderr の当該行から
    // 実ポートを取得できるようにする。`local_addr()` は bind 直後の
    // ソケットに対しては OS 側の失敗要因がなく、実運用上失敗しないが、
    // 万一失敗しても `panic!` はせず設定文字列にフォールバックする
    // （`coding-rust.md` のエラー処理規約）。
    let listening_addr = listener
        .local_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| bind_addr.clone());
    eprintln!("fandhe-frontend-dist-server: listening on {listening_addr}");
    // アセット配信モードを起動時に 1 行だけ出力する（TASK-10.1a、イシュー #106）。
    // 内部絶対パス（`static/` の実パス等）は含めない固定文言のみとし、
    // 機微情報を露出しない（`security.md`）。
    eprintln!(
        "fandhe-frontend-dist-server: assets={}",
        match active_mode() {
            AssetMode::Embedded => "embedded",
            AssetMode::DevFilesystem => "dev-filesystem",
        }
    );

    loop {
        let (stream, _peer_addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                // 個別接続の accept 失敗でプロセス全体を落とさない
                // （エラー処理規約 `coding-rust.md`: panic! を避ける）。
                eprintln!("fandhe-frontend-dist-server: accept error: {err}");
                continue;
            }
        };
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("fandhe-frontend-dist-server: connection error: {err}");
            }
        });
    }
}

/// hyper の 1 リクエストを [`route_request`] へ委譲し、結果を
/// `Response<Full<Bytes>>` へ変換する。`Infallible` はこの関数自身が失敗
/// しないことを型で保証する（`route_request` は `RouteResponse` を必ず返す
/// 契約であり、パニックしない設計）。
///
/// 実際のレスポンス組み立ては [`response_for`]（`hyper::Incoming` 等の
/// 非同期・実 I/O 型に依存しない同期関数）に委譲する。本関数はその結果を
/// `Ok` で包むだけの薄い非同期アダプタであり、`tokio::test` 等の非同期テスト
/// 基盤（新規依存追加を要する）を導入しなくても `response_for` を直接
/// ユニットテストできるようにするための分離である
/// （`routes::route_request` と同じ「同期コア／非同期シェル」の分離方針）。
async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
    Ok(response_for(req.method(), path))
}

/// `handle` の同期コア。メソッドとパスから `Response<Full<Bytes>>` を組み立てる。
///
/// `route_request` は GET 専用の SSR/静的配信を前提とした設計（`routes.rs`）
/// のため、GET・HEAD 以外のメソッド（POST/PUT/DELETE 等）はページ本文を
/// 組み立てず先に 405 で弾く（Review 指摘: メソッド無検証で全メソッドに
/// 200 を返していたギャップの解消）。HEAD は GET と同じ本文を返してよい
/// （hyper 側で HEAD のボディ送出有無は扱わないため、ここでは GET と同列に許可する）。
fn response_for(method: &Method, path: &str) -> Response<Full<Bytes>> {
    if method != Method::GET && method != Method::HEAD {
        return Response::builder()
            .status(405)
            .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(hyper::header::ALLOW, "GET, HEAD")
            .body(Full::new(Bytes::from_static(b"405 Method Not Allowed")))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
                    .expect("fallback response with fixed, valid status/body must build")
            });
    }

    let route_response = route_request(path);

    let mut builder = Response::builder().status(route_response.status);
    if let Some(headers) = builder.headers_mut() {
        headers.insert(
            hyper::header::CONTENT_TYPE,
            // `content_type` は固定表由来の `&'static str`（`mime.rs` 参照）で
            // あり、リクエスト由来の文字列をヘッダへ反映することはない
            // （ヘッダインジェクション対策、`security.md`）。
            hyper::header::HeaderValue::from_static(route_response.content_type),
        );
        // 開発モード（DevFilesystem）の静的アセット応答のみ `cache_control`
        // が `Some` になる（`routes::RouteResponse::cache_control` の doc・
        // TASK-10.1b・イシュー #107 参照）。ブラウザキャッシュにより
        // ディスクの即時反映が体感上無効化されるのを防ぐ。値は
        // `route_request` 側で固定文言のみを設定する契約のため、ここでも
        // リクエスト由来文字列をヘッダへ流し込むことはない。
        if let Some(cache_control) = route_response.cache_control {
            headers.insert(
                hyper::header::CACHE_CONTROL,
                hyper::header::HeaderValue::from_static(cache_control),
            );
        }
    }

    // `Response::builder()` はステータスコードが不正な場合のみ失敗するが、
    // `route_response.status` は本クレート内部で 200/404 のみを組み立てる
    // 定数的な値であるため `unwrap_or_else` でフォールバックし `panic!` は
    // しない（`coding-rust.md` のエラー処理規約）。
    builder
        .body(Full::new(Bytes::from(route_response.body)))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
                .expect("fallback response with fixed, valid status/body must build")
        })
}

#[cfg(test)]
mod tests {
    use super::{resolve_bind_addr, response_for, BindAddrError, Method, DEFAULT_BIND_ADDR};

    // `resolve_bind_addr` はプロセス環境変数を読まない純粋関数のため、以下は
    // 環境変数を書き換えずに全分岐を検証できる（イシュー #3336）。

    #[test]
    fn both_unset_falls_back_to_default() {
        assert_eq!(
            resolve_bind_addr(None, None),
            Ok(DEFAULT_BIND_ADDR.to_string())
        );
    }

    #[test]
    fn both_empty_string_falls_back_to_default() {
        assert_eq!(
            resolve_bind_addr(Some(""), Some("")),
            Ok(DEFAULT_BIND_ADDR.to_string())
        );
    }

    #[test]
    fn bind_addr_only_uses_its_value() {
        assert_eq!(
            resolve_bind_addr(Some("127.0.0.1:8080"), None),
            Ok("127.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn empty_bind_addr_with_valid_port_uses_port() {
        assert_eq!(
            resolve_bind_addr(Some(""), Some("8080")),
            Ok("0.0.0.0:8080".to_string())
        );
    }

    #[test]
    fn port_only_uses_wildcard_host() {
        assert_eq!(
            resolve_bind_addr(None, Some("8080")),
            Ok("0.0.0.0:8080".to_string())
        );
    }

    #[test]
    fn bind_addr_set_wins_even_with_invalid_port() {
        // 優先順位 1（`FANDHE_FRONTEND_BIND_ADDR`）が成立する限り、`PORT` の値は
        // 一切検証されない（見落としやすい分岐、`tests/bind_addr.rs` 側のコメント
        // と同じ仕様）。
        for invalid_port in ["0", "65536", "abc", "-1"] {
            assert_eq!(
                resolve_bind_addr(Some("127.0.0.1:9000"), Some(invalid_port)),
                Ok("127.0.0.1:9000".to_string()),
                "invalid_port={invalid_port}"
            );
        }
    }

    #[test]
    fn invalid_port_values_are_rejected() {
        for invalid_port in ["0", "65536", "abc", "-1", " 8080", "8080 "] {
            assert_eq!(
                resolve_bind_addr(None, Some(invalid_port)),
                Err(BindAddrError::InvalidPort),
                "invalid_port={invalid_port}"
            );
        }
    }

    #[test]
    fn bind_addr_error_display_does_not_leak_the_port_value() {
        let message = BindAddrError::InvalidPort.to_string();
        assert!(!message.contains("65536"));
        assert!(message.contains("PORT"));
    }

    #[test]
    fn get_and_head_are_routed_to_route_request() {
        // GET: `/` は一覧ページ（200）を返す（`routes::route_request` の契約）。
        assert_eq!(response_for(&Method::GET, "/").status(), 200);
        // HEAD も GET と同列に許可され、`route_request` へ委譲される。
        assert_eq!(response_for(&Method::HEAD, "/").status(), 200);
    }

    #[test]
    fn non_get_head_methods_return_405_with_allow_header() {
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            let response = response_for(&method, "/");
            assert_eq!(response.status(), 405, "method={method}");
            assert_eq!(
                response.headers().get(hyper::header::ALLOW).unwrap(),
                "GET, HEAD"
            );
        }
    }

    #[test]
    fn page_response_never_sets_cache_control_header() {
        // ページ応答は開発 / 本番モードによらず `Cache-Control` を付与しない
        // （`routes::RouteResponse::cache_control` の doc 参照）。
        let response = response_for(&Method::GET, "/");
        assert!(response
            .headers()
            .get(hyper::header::CACHE_CONTROL)
            .is_none());
    }

    // 静的アセット応答への `Cache-Control: no-store` 付与はビルド構成
    // （開発 / 本番モード）によって固定値が変わるため、`assets.rs` /
    // `routes.rs` の `active_mode` 系テストと同じ cfg ゲートで固定する
    // （TASK-10.1b、イシュー #107）。
    #[cfg(all(debug_assertions, not(feature = "force-embed")))]
    #[test]
    fn static_asset_response_sets_no_store_cache_control_header_in_dev_filesystem_mode() {
        let response = response_for(&Method::GET, "/static/view-transitions.js");
        assert_eq!(response.status(), 200);
        assert_eq!(
            response
                .headers()
                .get(hyper::header::CACHE_CONTROL)
                .unwrap(),
            "no-store"
        );
    }

    #[cfg(not(all(debug_assertions, not(feature = "force-embed"))))]
    #[test]
    fn static_asset_response_has_no_cache_control_header_in_embedded_mode() {
        let response = response_for(&Method::GET, "/static/view-transitions.js");
        assert_eq!(response.status(), 200);
        assert!(response
            .headers()
            .get(hyper::header::CACHE_CONTROL)
            .is_none());
    }
}
