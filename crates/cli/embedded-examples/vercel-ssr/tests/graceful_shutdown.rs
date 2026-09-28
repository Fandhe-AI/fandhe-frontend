//! `src/main.rs` の graceful shutdown（#3337 の移植）を実プロセス起動で
//! 検証する。`crates/dist-server/tests/graceful_shutdown.rs` を雛形にした、
//! 本サンプル単体で完結する最小版（`Cargo.toml` の `[workspace] members =
//! ["."]` により root workspace のテストヘルパは参照できない）。
//!
//! `SIGTERM`/`SIGINT` は Unix 固有のシグナルであり、`Child::kill()`（SIGKILL
//! 相当）では代替できないため、本ファイル全体を `#[cfg(unix)]` でゲートする
//! （非 Unix ランナーでは無条件でコンパイル対象から外れる。`unsafe` を要する
//! `libc::kill` は使わず、`std::process::Command::new("kill")` で送る）。
#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 子プロセスを確実に kill・wait する RAII ガード。
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// バイナリを `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）で起動し、
/// stderr の `listening on` 行から実際に割り当てられたポート番号を読み取る
/// （`tests/boot.rs::spawn_and_wait_for_port` と同型）。
fn spawn_and_wait_for_port() -> (ChildGuard, u16) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssr"))
        .env("FANDHE_FRONTEND_BIND_ADDR", "127.0.0.1:0")
        .env_remove("PORT")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vercel-ssr example binary must spawn");

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");

    let (tx, rx) = mpsc::channel::<String>();

    // 読み取りスレッドは送信失敗（受信側が既に return 済み）でも break しない
    // （`crates/dist-server/tests/support/mod.rs::read_listening_addr` が
    // 踏んだ EPIPE 障害と同じ回避策。graceful shutdown はシグナル受信後に
    // 複数行を追加出力するため、早期に reader を drop すると子プロセスの
    // `eprintln!` が Broken pipe で panic しうる）。
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: 子プロセスが終了し書き込み側が閉じた
                Ok(_) => {
                    let _ = tx.send(line.clone());
                }
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut collected = String::new();
    let mut port: Option<u16> = None;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                collected.push_str(&line);
                if let Some(found) = extract_port(&line) {
                    port = Some(found);
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let port = port.unwrap_or_else(|| {
        let _ = child.kill();
        panic!("failed to read listening port from stderr within timeout: {collected}");
    });

    (ChildGuard(child), port)
}

fn extract_port(line: &str) -> Option<u16> {
    let addr = line.trim().rsplit(' ').next()?;
    let (_, port_str) = addr.rsplit_once(':')?;
    port_str.trim().parse().ok()
}

fn send_http_request(port: u16, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");
    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request must be written");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("response must be readable as UTF-8");
    response
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .expect("try_wait on vercel-ssr example child must not error")
        {
            return status;
        }
        if Instant::now() >= deadline {
            panic!("vercel-ssr example did not exit within timeout");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `pid` へ `SIGTERM`（`-s TERM`）を送る。`Child::kill()` が送る SIGKILL とは
/// 異なり、`main.rs` の graceful shutdown 経路（`ShutdownSignals`）を起動する。
/// `unsafe`（`libc::kill`）を使わず外部コマンド経由で送ることで、
/// `#![forbid(unsafe_code)]`（REQ-2）をテストコードでも維持する。
fn send_sigterm(pid: u32) {
    let status = Command::new("kill")
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
    let (mut guard, port) = spawn_and_wait_for_port();
    let pid = guard.0.id();

    send_sigterm(pid);

    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(
        status.success(),
        "vercel-ssr example must exit successfully after SIGTERM, got {status:?}"
    );

    let _ = port;
}

/// SIGTERM 受信後、新規接続が速やかに（backlog に滞留せず）拒否されることを
/// 確認する。
#[test]
fn connection_refused_shortly_after_sigterm() {
    let (mut guard, port) = spawn_and_wait_for_port();
    let pid = guard.0.id();

    let response = send_http_request(port, "GET", "/");
    assert!(response.starts_with("HTTP/1.1 200"));

    send_sigterm(pid);

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut last_err: Option<std::io::Error> = None;
    let mut refused = false;
    while Instant::now() < deadline {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(_stream) => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                refused = true;
                last_err = Some(err);
                break;
            }
            Err(err) => {
                last_err = Some(err);
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }

    assert!(
        refused,
        "new connections must be refused shortly after SIGTERM, last_err={last_err:?}"
    );

    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(status.success());
}
