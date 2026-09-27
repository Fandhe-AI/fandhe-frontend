//! `main.rs` の graceful shutdown（イシュー #3337）を実プロセス起動で検証する。
//!
//! `SIGTERM`/`SIGINT` は Unix 固有のシグナルであり、`Child::kill()`（SIGKILL
//! 相当）では代替できないため、本ファイル全体を `#[cfg(unix)]` でゲートする
//! （非 Unix ランナーでは無条件でコンパイル対象から外れる。`unsafe` を要する
//! `libc::kill` は使わず、`std::process::Command::new("kill")` で送る）。
#![cfg(unix)]

mod support;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use support::{send_http_request, spawn_and_wait_for_port, wait_with_timeout};

/// テスト対象バイナリのパスを解決する（`tests/boot.rs` 等と同じ慣行）。
fn dist_server_binary() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_dist-server"))
}

/// `pid` へ `SIGTERM`（`-s TERM`）を送る。`Child::kill()` が送る SIGKILL とは
/// 異なり、`main.rs` の graceful shutdown 経路（`ShutdownSignals`）を起動する。
/// `unsafe`（`libc::kill`）を使わず外部コマンド経由で送ることで、
/// `#![forbid(unsafe_code)]`（REQ-2）をテストコードでも維持する。
fn send_sigterm(pid: u32) {
    let status = std::process::Command::new("kill")
        .args(["-s", "TERM", &pid.to_string()])
        .status()
        .expect("kill command must be spawnable");
    assert!(status.success(), "kill -s TERM must succeed");
}

/// SIGTERM 受信後、プロセスが猶予時間内に正常終了（exit code 0）することを
/// 確認する（`DRAIN_TIMEOUT_SECS`〔25 秒〕より十分短いタイムアウトで待つ。
/// 処理中の接続が無い状態なので即座に drain が完了するはず）。
#[test]
fn sigterm_triggers_graceful_exit_with_success_status() {
    let (mut guard, port) = spawn_and_wait_for_port(&dist_server_binary(), None);
    let pid = guard.0.id();

    send_sigterm(pid);

    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(
        status.success(),
        "dist-server must exit successfully after SIGTERM, got {status:?}"
    );

    // プロセス終了後は listener が閉じているため、当初のポートへの新規接続は
    // 失敗するはず（`connection_refused_shortly_after_sigterm` がこの経路を
    // 厳密に検証する。ここでは正常終了そのものを確認する）。
    let _ = port;
}

/// SIGTERM 受信後、新規接続が速やかに（backlog に滞留せず）拒否されることを
/// 確認する。「シグナル受信後に listener を drop し、新規接続を OS レベルで
/// 即座に拒否する」ことの直接証拠（実装計画のテスト B）。
#[test]
fn connection_refused_shortly_after_sigterm() {
    let (mut guard, port) = spawn_and_wait_for_port(&dist_server_binary(), None);
    let pid = guard.0.id();

    // シグナル送信前は通常どおり応答することを確認する（後続の拒否が
    // 「そもそも繋がっていなかった」ことによる偽陽性でないことの根拠）。
    let response = send_http_request(port, "GET", "/");
    assert!(response.starts_with("HTTP/1.1 200"));

    send_sigterm(pid);

    // 最大 2 秒、20ms 間隔でポーリングし、新規 `TcpStream::connect` が
    // 拒否される（`ConnectionRefused` 相当）ことを確認する。listener の
    // drop と OS のソケットクローズには若干のタイミング差があり得るため、
    // 即時 1 回の判定ではなく短いポーリングにする。
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut last_err: Option<std::io::Error> = None;
    let mut refused = false;
    while std::time::Instant::now() < deadline {
        match std::net::TcpStream::connect(("127.0.0.1", port)) {
            Ok(_stream) => {
                // まだ listener が生きている可能性がある。少し待って再試行する。
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                refused = true;
                last_err = Some(err);
                break;
            }
            Err(err) => {
                // `ConnectionRefused` 以外（一時的な OS 側エラー等）は
                // listener を閉じたことの証拠にならないため、これだけで
                // 成功扱いにはしない。次のポーリングへ回す。
                last_err = Some(err);
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }

    assert!(
        refused,
        "new connections must be refused shortly after SIGTERM, last_err={last_err:?}"
    );

    // プロセス自体も猶予時間内に正常終了することを併せて確認する
    // （テスト A の重複ではなく、同一シグナル配送での終了保証を再確認する）。
    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(status.success());
}

/// SIGTERM 受信時に処理中（応答未完了）の接続が、`graceful.watch(..)` により
/// 応答完了まで維持されることを検証する（レビュー指摘、PR #3358 スレッド
/// `PRRT_kwDOTarxgc6mdwt5`）。
///
/// 上記 2 テストはいずれも「処理中のリクエストが無い状態」で SIGTERM を
/// 送っており、`drain_within` の単体テスト（`main.rs` の `#[cfg(test)]`）も
/// `GracefulShutdown::watcher()` を直接呼ぶだけで実 HTTP 接続を経由しない。
/// このテストは実際に TCP 接続を確立し、HTTP リクエストを**意図的に 2 回の
/// 書き込みへ分割**して「まだ受信完了していない」状態を作ってから SIGTERM を
/// 送ることで、`graceful.watch(..)` が当該接続を「処理中の接続」として
/// 認識し、シグナル受信後も応答完了まで hyper が接続を切断しないこと
/// の直接証拠とする。
#[test]
fn in_flight_request_completes_after_sigterm() {
    let (mut guard, port) = spawn_and_wait_for_port(&dist_server_binary(), None);
    let pid = guard.0.id();

    // 1) まず TCP 接続を確立し、リクエストの前半のみを送る（`\r\n\r\n` を
    //    送らないため、hyper 側はヘッダ受信完了と判断できず「処理中」の
    //    ままになる）。`accept` → `graceful.watch(..)` は接続確立時点で
    //    同期的に走るため、この時点で当該接続は既に watch 対象になっている
    //    （`main.rs` の accept ループ実装参照）。
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to dist-server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n")
        .expect("partial request headers must be written");

    // 2) 接続が実際に accept され、`graceful.watch(..)` の登録が完了するのを
    //    確実にするため、少し待ってから SIGTERM を送る。
    std::thread::sleep(Duration::from_millis(100));
    send_sigterm(pid);

    // 3) シグナル受信・listener drop（新規接続拒否への切り替わり）が確実に
    //    先に処理されるよう、少し待ってからリクエストの残り（ヘッダ終端 +
    //    `Connection: close`）を送信する。「SIGTERM 受信後に受信を完了させた
    //    リクエスト」であることを保証するための順序である。
    std::thread::sleep(Duration::from_millis(100));
    stream
        .write_all(b"Connection: close\r\n\r\n")
        .expect("remaining request bytes must be written after SIGTERM");

    // 4) 応答を最後まで読み切れること（＝接続が完了まで維持されたこと）を
    //    確認する。`read_to_string` はピア側が `Connection: close` を守って
    //    ソケットを閉じるまでブロックするため、これが完了する時点で
    //    「shutdown 経路が接続を中断せず最後まで処理した」ことの直接証拠になる。
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("in-flight response must be readable to completion after SIGTERM");
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "in-flight request must still receive a normal response, got: {response:?}"
    );

    // 5) プロセス自体も猶予時間内（`DRAIN_TIMEOUT_SECS` 未満）に正常終了する
    //    ことを確認する。
    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(
        status.success(),
        "dist-server must exit successfully once the in-flight connection drains, got {status:?}"
    );
}
