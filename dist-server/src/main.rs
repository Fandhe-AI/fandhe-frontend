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
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
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
async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let route_response = route_request(req.uri().path_and_query().map_or("/", |pq| pq.as_str()));

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
    let response = builder
        .body(Full::new(Bytes::from(route_response.body)))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
                .expect("fallback response with fixed, valid status/body must build")
        });

    Ok(response)
}
