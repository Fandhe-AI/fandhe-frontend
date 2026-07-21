//! `examples/dist-server-docker` の起動エントリ（イシュー #502）。
//!
//! `fandhe-frontend-dist-server`（crates.io v0.1.0）を通常の外部依存として
//! 利用し、単一バイナリ + Docker（`FROM scratch`）配布の最小構成を実演する
//! 正本サンプル。`crates/dist-server/src/main.rs`（フレームワーク本体の
//! 配布サーバー実装）を雛形にした薄い hyper トランスポート層であり、実際の
//! ルーティング・ページ生成は `fandhe_frontend_dist_server::routes::route_request`
//! （HTTP に依存しない純粋関数）に委譲する。
//!
//! # `static/` アセットが外部依存経由では配信されない制約（実測、イシュー #502）
//!
//! `fandhe-frontend-dist-server` の `assets::lookup`（開発モード）・
//! `assets::embedded_lookup`（本番モード）はいずれも、ライブラリ自身の
//! `crates/dist-server/` を基準にした固定パス（`CARGO_MANIFEST_DIR` の 2 段上
//! と `static` の結合、ライブラリ側 `build.rs`／`assets.rs::dev_fs::static_root`
//! 参照）から `static/` を解決する。crates.io からの通常の外部依存として
//! 利用する本サンプルの `static/`（本ディレクトリ直下）はこの基準から
//! 外れるため、`lib` 経由では開発ビルド・release ビルドのいずれでも配信
//! されない（scratchpad での実測で確認済み。開発ビルドは
//! `active_mode() == DevFilesystem` だが `lookup("/static/style.css")` は
//! `None`、release ビルドの `EMBEDDED_ASSETS` テーブルも空になる。詳細は
//! README「学べること」参照）。
//!
//! この制約への対処として、本サンプル自身が `/static/style.css` への
//! アクセスのみを [`route_request`] より手前で完全一致判定し、
//! `include_bytes!` でコンパイル時にバイナリへ埋め込んだ CSS を固定
//! `Content-Type` で返す（下記 [`static_style_css`] 参照）。ユーザー入力から
//! ファイルパスを組み立てることは一切なく、パストラバーサル面はゼロ
//! （`security.md` A01）。`fandhe-frontend-dist-server` ライブラリ側の
//! 「外部依存時にもプロジェクト直下 `static/` を配信できるようにする」改善
//! は本サンプルのスコープ外（後続 Issue で追跡）。
//!
//! # セキュリティ設定（`security.md` A05 セキュリティ設定ミス）
//!
//! 既定 bind アドレスはループバック（`127.0.0.1`）とし、外部公開は
//! `FANDHE_FRONTEND_BIND_ADDR` の明示的なオプトインを要求する（`crates/dist-server/src/main.rs`
//! と同じ契約）。bind 失敗時は `panic!` せず、アドレスと OS エラーのみを
//! stderr に出力して非 0 終了する。

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

/// 既定の bind アドレス。`FANDHE_FRONTEND_BIND_ADDR` 未設定時に使う
/// （ループバック限定。`security.md` 参照）。
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3100";

/// このサンプル自身が `include_bytes!` でコンパイル時に埋め込む唯一の静的
/// アセット。`fandhe-frontend-dist-server` ライブラリの `assets::lookup` は
/// 外部依存として使う本サンプルの `static/` を解決できないため
/// （モジュール doc 参照）、実演用に 1 ファイルだけ自前配信する。
const STATIC_STYLE_CSS: &[u8] = include_bytes!("../static/style.css");

/// [`STATIC_STYLE_CSS`] を配信する固定 URL パス。完全一致判定のみに使い、
/// リクエストパスからファイルシステムパスを組み立てることはない
/// （パストラバーサル面ゼロ、`security.md` A01）。
const STATIC_STYLE_CSS_PATH: &str = "/static/style.css";

fn main() -> ExitCode {
    // `#[tokio::main]`（tokio-macros、依存グラフ深さ増の一因）を使わず、
    // `Builder` を直接呼ぶ（`crates/dist-server/src/main.rs` と同じ方針）。
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("dist-server-docker-example: failed to start tokio runtime: {err}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(run())
}

/// 非同期本体。bind 成功後は無限に接続を受け付け続ける
/// （通常運用では戻らない。bind 失敗時のみ `ExitCode::FAILURE` を返す）。
async fn run() -> ExitCode {
    let bind_addr = std::env::var("FANDHE_FRONTEND_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            // bind 失敗はアドレスと OS エラーのみを出力する（機微情報を含めない）。
            eprintln!("dist-server-docker-example: failed to bind {bind_addr}: {err}");
            return ExitCode::FAILURE;
        }
    };
    // `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（ポート 0 = OS 割当）で起動した
    // 場合、設定文字列 `bind_addr` をそのままログに出すと実際に割り当てられた
    // ポート番号が分からない。`listener.local_addr()` を使うことで
    // `tests/boot.rs` が stderr の当該行から実ポートを取得できるようにする
    // （`crates/dist-server/src/main.rs` と同じ理由）。
    let listening_addr = listener
        .local_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| bind_addr.clone());
    eprintln!("dist-server-docker-example: listening on {listening_addr}");
    // アセット配信モードを起動時に 1 行だけ出力する（`crates/dist-server/src/main.rs`
    // と同じ運用）。`DevFilesystem`/`Embedded` いずれの場合も本サンプル自身の
    // `static/style.css` は配信されず、上記 [`STATIC_STYLE_CSS`] 経由でのみ
    // 配信される（モジュール doc 参照）。
    eprintln!(
        "dist-server-docker-example: assets={}",
        match active_mode() {
            AssetMode::Embedded => "embedded",
            AssetMode::DevFilesystem => "dev-filesystem",
        }
    );

    loop {
        let (stream, _peer_addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                // 個別接続の accept 失敗でプロセス全体を落とさない。
                eprintln!("dist-server-docker-example: accept error: {err}");
                continue;
            }
        };
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("dist-server-docker-example: connection error: {err}");
            }
        });
    }
}

/// hyper の 1 リクエストを [`response_for`] へ委譲する薄い非同期アダプタ
/// （`crates/dist-server/src/main.rs::handle` と同じ「同期コア／非同期シェル」
/// の分離方針。`response_for` を非同期テスト基盤なしで直接ユニットテスト
/// できるようにするための分離）。
async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
    Ok(response_for(req.method(), path))
}

/// `handle` の同期コア。メソッドとパスから `Response<Full<Bytes>>` を組み立てる。
///
/// GET・HEAD 以外は 405 で弾く（`route_request` は GET 専用の SSR/静的配信を
/// 前提とした設計のため、`crates/dist-server/src/main.rs::response_for` と
/// 同じ方針）。[`STATIC_STYLE_CSS_PATH`] への完全一致は `route_request` より
/// 手前で判定する（モジュール doc 参照）。
fn response_for(method: &Method, path: &str) -> Response<Full<Bytes>> {
    if method != Method::GET && method != Method::HEAD {
        return Response::builder()
            .status(405)
            .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(hyper::header::ALLOW, "GET, HEAD")
            .body(Full::new(Bytes::from_static(b"405 Method Not Allowed")))
            .unwrap_or_else(|_| fallback_500());
    }

    // クエリ文字列を除いた完全一致のみを対象にする（`route_request` の
    // `/static/` プレフィックス分岐と同じくクエリ許容の慣習に合わせる）。
    let path_without_query = path.split('?').next().unwrap_or(path);
    if path_without_query == STATIC_STYLE_CSS_PATH {
        return static_style_css_response();
    }

    let route_response = route_request(path);

    let mut builder = Response::builder().status(route_response.status);
    if let Some(headers) = builder.headers_mut() {
        headers.insert(
            hyper::header::CONTENT_TYPE,
            // `content_type` は固定表由来の `&'static str`（ライブラリ側
            // `mime.rs` 参照）であり、リクエスト由来の文字列をヘッダへ
            // 反映することはない（ヘッダインジェクション対策、`security.md`）。
            hyper::header::HeaderValue::from_static(route_response.content_type),
        );
        if let Some(cache_control) = route_response.cache_control {
            headers.insert(
                hyper::header::CACHE_CONTROL,
                hyper::header::HeaderValue::from_static(cache_control),
            );
        }
    }

    builder
        .body(Full::new(Bytes::from(route_response.body)))
        .unwrap_or_else(|_| fallback_500())
}

/// [`STATIC_STYLE_CSS_PATH`] への応答を組み立てる。バイト列はコンパイル時に
/// `include_bytes!` で埋め込み済みであり、実行時のファイルシステムアクセス
/// は発生しない（パストラバーサル面ゼロ、モジュール doc 参照）。
fn static_style_css_response() -> Response<Full<Bytes>> {
    Response::builder()
        .status(200)
        .header(hyper::header::CONTENT_TYPE, "text/css; charset=utf-8")
        .body(Full::new(Bytes::from_static(STATIC_STYLE_CSS)))
        .unwrap_or_else(|_| fallback_500())
}

/// レスポンス組み立てが失敗した場合の固定フォールバック（ステータス・本文
/// とも固定値のため `Response::builder()` は失敗しない契約。`unwrap_or_else`
/// の唯一の分岐先として `panic!` を避ける、`coding-rust.md` のエラー処理規約）。
fn fallback_500() -> Response<Full<Bytes>> {
    Response::builder()
        .status(500)
        .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
        .expect("fallback response with fixed, valid status/body must build")
}

#[cfg(test)]
mod tests {
    use super::{response_for, Method};

    #[test]
    fn get_root_is_routed_to_route_request() {
        assert_eq!(response_for(&Method::GET, "/").status(), 200);
    }

    #[test]
    fn head_is_routed_like_get() {
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
    fn static_style_css_is_served_with_fixed_content_type() {
        let response = response_for(&Method::GET, "/static/style.css");
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get(hyper::header::CONTENT_TYPE).unwrap(),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn unknown_path_returns_404() {
        assert_eq!(response_for(&Method::GET, "/no-such-page").status(), 404);
    }
}
