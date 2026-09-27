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
//! # graceful shutdown（イシュー #3337）
//!
//! Vercel Container Images はスケールイン時に `SIGTERM` を送り、30 秒の猶予後に
//! 強制終了する契約を持つ。accept ループは新規接続の受付と [`ShutdownSignals`]
//! のポーリングを [`std::future::poll_fn`] で手動 race させ、`SIGTERM`（Unix）・
//! `SIGINT`（Unix、Ctrl-C 相当）・非 Unix の Ctrl-C のいずれかを受信すると
//! ループを抜けて `listener` を明示的に drop する（以後の新規接続は OS レベルで
//! 即座に拒否され、backlog に滞留しない）。続けて `hyper_util::server::graceful::
//! GracefulShutdown` が処理中の接続の完了を [`DRAIN_TIMEOUT_SECS`]（Vercel の
//! 30 秒猶予より短い既定値）まで待ち、間に合わなければ待たずに終了する
//! （悪意ある・低速なクライアントがプロセス終了を無期限に妨げる事態を防ぐ、
//! `security.md` A05 参照）。`tokio::select!`/`tokio::join!` は `macros` feature
//! （`tokio-macros → syn → quote → proc-macro2` の proc-macro 連鎖、本クレートが
//! 依存グラフ抑制のため意図的に避けている構成）を要求するため使わず、
//! `std::future::poll_fn` + `std::pin::pin!`（標準ライブラリのみ、追加依存なし）
//! で手動 race を組む（`drain_within` 参照）。
//!
//! # セキュリティ設定（`security.md` A05 セキュリティ設定ミス）
//!
//! 既定 bind アドレスはループバック（`127.0.0.1`）とし、外部公開は
//! `FANDHE_FRONTEND_BIND_ADDR` または `PORT` の明示的なオプトインを要求する。bind 失敗時は
//! `panic!` せず、アドレスと OS エラーのみを stderr に出力して非 0 終了する
//! （内部パス・スタックトレース等の機微情報は出力しない）。`PORT` の値が不正な場合も
//! 黙って既定値へフォールバックせず起動を失敗させ（fail-closed）、エラーメッセージには
//! `PORT` の実際の値を含めない（カテゴリのみの固定英語文言、機微情報を露出しない）。
//! シグナルハンドラの登録に失敗した場合（通常起こらない）も `panic!` せず、固定の
//! 英語メッセージのみを stderr に出して起動を継続する（この場合 graceful shutdown は
//! 機能しなくなるが、OS 既定の signal disposition〔即時終了〕は引き続き有効なため
//! 無応答のまま残り続ける危険な状態には陥らない、安全側フォールバック）。

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
use hyper_util::server::graceful::GracefulShutdown;
use std::convert::Infallible;
use std::future::Future;
use std::process::ExitCode;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::net::TcpListener;

/// 既定の bind アドレス。`FANDHE_FRONTEND_BIND_ADDR` も `PORT` も未設定のときに使う
/// （ループバック限定。`security.md` 参照）。
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3100";

/// シグナル受信後、処理中の接続の完了を待つ猶予秒数。
///
/// Vercel Container Images の `SIGTERM` 後 30 秒強制終了猶予より短い値とし、
/// 悪意ある・低速なクライアントが接続を握り続けてプロセス終了を無期限に
/// 妨げる事態を防ぐ（`security.md` A05、モジュール冒頭 doc 参照）。
const DRAIN_TIMEOUT_SECS: u64 = 25;

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

/// `SIGTERM`/`SIGINT`（Unix）または Ctrl-C（非 Unix）の受信を待ち受ける状態。
///
/// `run()` の accept ループが [`std::future::poll_fn`] で毎回 [`Self::poll`] を
/// 呼び出し、新規接続の到着と競合させる（`tokio::select!` を使わない理由は
/// モジュール冒頭 doc 参照）。Unix では `tokio::signal::unix::Signal` を
/// `poll_recv` で繰り返しポーリングできるため `Pin` は不要、非 Unix では
/// 単発の `tokio::signal::ctrl_c()` をヒープに `Pin` して繰り返しポーリング
/// する。
struct ShutdownSignals {
    #[cfg(unix)]
    sigterm: Option<tokio::signal::unix::Signal>,
    #[cfg(unix)]
    sigint: Option<tokio::signal::unix::Signal>,
    #[cfg(not(unix))]
    ctrl_c: std::pin::Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>>,
    /// 非 Unix 経路で `ctrl_c` の poll がエラーを返した後 `true` にする。
    ///
    /// `Future::poll` は一度 `Poll::Ready` を返した後の再 poll を保証しない
    /// ため、ハンドラのポーリング失敗（Ctrl-C 受信ではない）を検知したら
    /// 固定メッセージを stderr へ出したうえでこのフラグを立て、以後は
    /// `ctrl_c` を再 poll せず常に [`Poll::Pending`] を返す（Unix の
    /// インストール失敗時と同じ「安全側フォールバック」— OS 既定の
    /// シグナル処理は引き続き有効なため無応答のまま残り続ける危険はない）。
    #[cfg(not(unix))]
    ctrl_c_failed: bool,
    /// [`Self::install`] 内の早期 poll（後述）が既に `Poll::Ready(Ok(()))`
    /// を観測していた場合に `true` にする。
    ///
    /// `tokio::signal::ctrl_c()` は `async fn` であるため、呼び出し自体
    /// ではハンドラ登録が走らず、最初に `poll` された時点で初めて内部の
    /// 登録処理が実行される（Unix の `signal(SignalKind::interrupt())` が
    /// 呼び出し時点で同期的に登録するのとは対照的）。「bind より先に
    /// ハンドラを登録し、その間のシグナルを取りこぼさない」契約を非 Unix
    /// でも満たすため、`install()` は生成直後に 1 度 poll してハンドラ
    /// 登録を強制する。この早期 poll が（理論上ほぼ起こり得ないが）既に
    /// `Ready` を返した場合、`Future::poll` の「一度 Ready を返した後の
    /// 再 poll を保証しない」契約により `ctrl_c` を再 poll せず、以後の
    /// [`Self::poll`] 呼び出しではこのフラグのみで即座に `Ready` を返す。
    #[cfg(not(unix))]
    ctrl_c_ready: bool,
}

impl ShutdownSignals {
    /// シグナルハンドラを登録する。
    ///
    /// 登録自体の失敗（OS リソース枯渇等、通常起こらない）は `panic!` せず、
    /// 固定の英語メッセージを stderr に出したうえで当該シグナルを「待たない」
    /// 扱いにする（`coding-rust.md` のエラー処理規約。SIGKILL 相当の外部終了は
    /// 引き続き OS 既定動作が効くため安全側に倒れる）。
    fn install() -> Self {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let sigterm = signal(SignalKind::terminate())
                .inspect_err(|err| {
                    eprintln!(
                        "fandhe-frontend-dist-server: failed to install SIGTERM handler: {err}"
                    );
                })
                .ok();
            let sigint = signal(SignalKind::interrupt())
                .inspect_err(|err| {
                    eprintln!(
                        "fandhe-frontend-dist-server: failed to install SIGINT handler: {err}"
                    );
                })
                .ok();
            Self { sigterm, sigint }
        }
        #[cfg(not(unix))]
        {
            let mut ctrl_c: std::pin::Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>> =
                Box::pin(tokio::signal::ctrl_c());
            // `ctrl_c()` は `async fn` のため、呼び出し時点ではハンドラ登録
            // が走らない（構造体フィールドの doc 参照）。bind より先に
            // 登録を完了させるため、ここで即座に一度 poll して登録処理を
            // 強制する。`Waker::noop()` は何もしない Waker で、この poll
            // で `Pending` が返っても誰も再起床させないが、実際の受信検知
            // は accept ループ内の `poll_fn` が改めて `poll` するため問題
            // ない（ここでの唯一の目的はハンドラの早期登録）。
            let waker = std::task::Waker::noop();
            let mut cx = Context::from_waker(waker);
            let ctrl_c_ready = match ctrl_c.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(())) => true,
                Poll::Ready(Err(err)) => {
                    eprintln!("fandhe-frontend-dist-server: failed to poll Ctrl-C handler: {err}");
                    return Self {
                        ctrl_c,
                        ctrl_c_failed: true,
                        ctrl_c_ready: false,
                    };
                }
                Poll::Pending => false,
            };
            Self {
                ctrl_c,
                ctrl_c_failed: false,
                ctrl_c_ready,
            }
        }
    }

    /// 登録済みのいずれかのシグナルを受信していれば固定名
    /// （`"SIGTERM"`/`"SIGINT"`/`"Ctrl-C"`）を返す。ログにのみ使う固定文字列
    /// であり機微情報を含まない。
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<&'static str> {
        #[cfg(unix)]
        {
            if let Some(sig) = self.sigterm.as_mut() {
                if sig.poll_recv(cx).is_ready() {
                    return Poll::Ready("SIGTERM");
                }
            }
            if let Some(sig) = self.sigint.as_mut() {
                if sig.poll_recv(cx).is_ready() {
                    return Poll::Ready("SIGINT");
                }
            }
            Poll::Pending
        }
        #[cfg(not(unix))]
        {
            // `install()` の早期 poll で既に受信済みだった場合（構造体
            // フィールドの doc 参照）。`Future::poll` の「Ready 後は再 poll
            // しない」契約を守るため、`ctrl_c` を再度 poll せずここで確定
            // する。
            if self.ctrl_c_ready {
                return Poll::Ready("Ctrl-C");
            }
            if self.ctrl_c_failed {
                return Poll::Pending;
            }
            match self.ctrl_c.as_mut().poll(cx) {
                Poll::Ready(Ok(())) => {
                    self.ctrl_c_ready = true;
                    Poll::Ready("Ctrl-C")
                }
                Poll::Ready(Err(err)) => {
                    eprintln!("fandhe-frontend-dist-server: failed to poll Ctrl-C handler: {err}");
                    self.ctrl_c_failed = true;
                    Poll::Pending
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

/// [`GracefulShutdown::shutdown`] の完了を `timeout` まで待つ。
///
/// 間に合えば `true`、打ち切ったら `false` を返す。`tokio::select!` を使わず
/// `poll_fn` で手動 race を組む（モジュール冒頭 doc 参照）。「同期コア／
/// 非同期シェル」分離方針（`routes::route_request` 等と同様）に沿い、
/// 分岐そのものは `#[cfg(test)]` のユニットテストで確定的に検証できる。
async fn drain_within(graceful: GracefulShutdown, timeout: Duration) -> bool {
    let mut shutdown_fut = std::pin::pin!(graceful.shutdown());
    let mut sleep_fut = std::pin::pin!(tokio::time::sleep(timeout));
    std::future::poll_fn(|cx| {
        if shutdown_fut.as_mut().poll(cx).is_ready() {
            return Poll::Ready(true);
        }
        if sleep_fut.as_mut().poll(cx).is_ready() {
            return Poll::Ready(false);
        }
        Poll::Pending
    })
    .await
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

/// 非同期本体。bind 成功後は `SIGTERM`/`SIGINT`（非 Unix は Ctrl-C）を受信する
/// まで接続を受け付け続け、受信後は graceful shutdown（モジュール冒頭 doc
/// 参照）を経て `ExitCode::SUCCESS` を返す（bind 失敗時のみ `ExitCode::FAILURE`
/// を返す）。
async fn run() -> ExitCode {
    // bind に時間がかかるケースでもその間に届いた SIGTERM を取りこぼさない
    // よう、シグナルハンドラは bind より先にできるだけ早く登録する。
    let mut signals = ShutdownSignals::install();

    // `FANDHE_FRONTEND_BIND_ADDR` は非 UTF-8 値を「未設定」扱いへ畳み込む（既存動作を維持）。
    let bind_addr_env = std::env::var("FANDHE_FRONTEND_BIND_ADDR").ok();
    // `PORT` の UTF-8 デコードは `bind_addr_env` が未設定（＝優先順位 1 が不成立）の
    // ときのみ行う。`resolve_bind_addr` の契約（`bind_addr` が設定されていれば
    // `port` の値を一切検証しない）を `run()` 側の非 UTF-8 判定にも及ぼすための
    // 分岐で、`FANDHE_FRONTEND_BIND_ADDR` 設定時に非 UTF-8 な `PORT` があっても
    // 起動失敗させないためのもの（レビュー指摘、イシュー #3336）。
    let port_env = if non_empty(bind_addr_env.as_deref()).is_some() {
        None
    } else {
        // `PORT` は非 UTF-8 値を「不正な値」としてエラー経路へ倒す（黙ってフォールバック
        // しない、という受け入れ条件の趣旨に合わせる。`var_os` + `to_str()` で判定する）。
        match std::env::var_os("PORT") {
            Some(value) => match value.to_str() {
                Some(value) => Some(value.to_string()),
                None => {
                    eprintln!(
                        "fandhe-frontend-dist-server: {}",
                        BindAddrError::InvalidPort
                    );
                    return ExitCode::FAILURE;
                }
            },
            None => None,
        }
    };

    let bind_addr = match resolve_bind_addr(bind_addr_env.as_deref(), port_env.as_deref()) {
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

    let graceful = GracefulShutdown::new();

    // シグナル受信と新規接続の到着を手動 race させる（`tokio::select!` を
    // 使わない理由はモジュール冒頭 doc 参照）。`Err` 側にシグナル名を載せて
    // ループを抜ける契機として使う（実際の accept エラーではない）。
    let shutdown_reason: &'static str = loop {
        let event = std::future::poll_fn(|cx| {
            if let Poll::Ready(reason) = signals.poll(cx) {
                return Poll::Ready(Err(reason));
            }
            listener.poll_accept(cx).map(Ok)
        })
        .await;

        let (stream, _peer_addr) = match event {
            Ok(Ok(accepted)) => accepted,
            Ok(Err(err)) => {
                // 個別接続の accept 失敗でプロセス全体を落とさない
                // （エラー処理規約 `coding-rust.md`: panic! を避ける）。
                eprintln!("fandhe-frontend-dist-server: accept error: {err}");
                continue;
            }
            Err(reason) => break reason,
        };
        let io = TokioIo::new(stream);

        // `graceful.watch(..)` は呼び出した時点で同期的に watcher を登録する。
        // `tokio::spawn` に渡す「前」にここで呼ぶ必要がある（spawn 後に watch
        // すると、spawn されたタスクが初めて poll されるまでの間 shutdown の
        // 対象から漏れる可能性がある）。
        let conn = graceful.watch(http1::Builder::new().serve_connection(io, service_fn(handle)));
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("fandhe-frontend-dist-server: connection error: {err}");
            }
        });
    };

    eprintln!(
        "fandhe-frontend-dist-server: {shutdown_reason} received, no longer accepting new connections"
    );
    // listener を明示的に drop し、以後の新規接続を OS レベルで即座に拒否
    // させる（backlog に滞留させない。クライアントからは connection refused
    // が返る。`security.md` A01 参照）。
    drop(listener);

    eprintln!(
        "fandhe-frontend-dist-server: draining in-flight connections (up to {DRAIN_TIMEOUT_SECS}s)"
    );
    if drain_within(graceful, Duration::from_secs(DRAIN_TIMEOUT_SECS)).await {
        eprintln!("fandhe-frontend-dist-server: graceful shutdown complete");
    } else {
        eprintln!("fandhe-frontend-dist-server: drain timeout exceeded, forcing shutdown");
    }

    ExitCode::SUCCESS
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
    use super::{
        drain_within, resolve_bind_addr, response_for, BindAddrError, Method, DEFAULT_BIND_ADDR,
    };
    use hyper_util::server::graceful::GracefulShutdown;
    use std::time::Duration;

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

    /// `#[tokio::test]` は `macros` feature を要求するため使わず、`main()` と
    /// 同様に手書きの `current_thread` ランタイムで `block_on` する
    /// （テストコードでの `.expect()` は `coding-rust.md` の対象外）。
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime must build")
            .block_on(future)
    }

    #[test]
    fn drain_within_returns_true_when_no_connections_are_in_flight() {
        block_on(async {
            let graceful = GracefulShutdown::new();
            // watcher を作らず（＝処理中コネクションなしを模す）即座に
            // `shutdown()` が完了することを確認する。
            assert!(drain_within(graceful, Duration::from_millis(50)).await);
        });
    }

    #[test]
    fn drain_within_returns_false_when_a_connection_outlives_the_timeout() {
        block_on(async {
            let graceful = GracefulShutdown::new();
            // 実コネクションを経由せず「処理中コネクションが 1 件ある」状態を
            // 安価に模す（`GracefulShutdown::watcher()` の直接利用）。
            let watcher = graceful.watcher();
            assert!(!drain_within(graceful, Duration::from_millis(50)).await);
            drop(watcher);
        });
    }
}
