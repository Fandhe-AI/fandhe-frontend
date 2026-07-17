//! `rws-dist-server` の起動エントリ。
//!
//! `RWS_BIND_ADDR`（既定 `127.0.0.1:3100`）で TCP リッスンし、1 接続ごとに
//! `hyper`（HTTP/1.1）で処理する。実際のルーティング・レスポンス生成は
//! `rws_dist_server::routes::route_request`（HTTP に依存しない純粋関数）に
//! 委譲し、本ファイルは「hyper の接続を受けてバイト列に変換する」薄い
//! トランスポート層のみを担う。
//!
//! # セキュリティ設定（`security.md` A05 セキュリティ設定ミス）
//!
//! 既定 bind アドレスはループバック（`127.0.0.1`）とし、外部公開は
//! `RWS_BIND_ADDR` の明示的なオプトインを要求する。bind 失敗時は `panic!`
//! せず、アドレスと OS エラーのみを stderr に出力して非 0 終了する
//! （内部パス・スタックトレース等の機微情報は出力しない）。

// `lib.rs` の `#![forbid(unsafe_code)]`（REQ-2）はクレートルートを跨いで継承
// されないため、バイナリクレートルートである本ファイルにも明示的に付与し、
// 宣言の一貫性を保つ（実装上も unsafe は使用していない）。
#![forbid(unsafe_code)]

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use rws_dist_server::assets::{active_mode, AssetMode};
use rws_dist_server::routes::route_request;
use std::convert::Infallible;
use std::process::ExitCode;
use tokio::net::TcpListener;

/// 既定の bind アドレス。`RWS_BIND_ADDR` 未設定時に使う
/// （ループバック限定。`security.md` 参照）。
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3100";

fn main() -> ExitCode {
    // `#[tokio::main]`（tokio-macros、依存グラフ深さ増の一因）を使わず、
    // `Builder` を直接呼ぶ（`Cargo.toml` の REQ-3 実測コメント参照）。
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("rws-dist-server: failed to start tokio runtime: {err}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(run())
}

/// 非同期本体。bind 成功後は無限に接続を受け付け続ける
/// （通常運用では戻らない。bind 失敗時のみ `ExitCode::FAILURE` を返す）。
async fn run() -> ExitCode {
    let bind_addr =
        std::env::var("RWS_BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            // bind 失敗はアドレスと OS エラーのみを出力する（機微情報を含めない）。
            eprintln!("rws-dist-server: failed to bind {bind_addr}: {err}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("rws-dist-server: listening on {bind_addr}");
    // アセット配信モードを起動時に 1 行だけ出力する（TASK-10.1a、イシュー #106）。
    // 内部絶対パス（`static/` の実パス等）は含めない固定文言のみとし、
    // 機微情報を露出しない（`security.md`）。
    eprintln!(
        "rws-dist-server: assets={}",
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
                eprintln!("rws-dist-server: accept error: {err}");
                continue;
            }
        };
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("rws-dist-server: connection error: {err}");
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
    use super::{response_for, Method};

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
}
