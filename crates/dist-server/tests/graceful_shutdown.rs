//! `main.rs` の graceful shutdown（イシュー #3337）を実プロセス起動で検証する。
//!
//! `SIGTERM`/`SIGINT` は Unix 固有のシグナルであり、`Child::kill()`（SIGKILL
//! 相当）では代替できないため、本ファイル全体を `#[cfg(unix)]` でゲートする
//! （非 Unix ランナーでは無条件でコンパイル対象から外れる。`unsafe` を要する
//! `libc::kill` は使わず、`std::process::Command::new("kill")` で送る）。
#![cfg(unix)]

mod support;

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
