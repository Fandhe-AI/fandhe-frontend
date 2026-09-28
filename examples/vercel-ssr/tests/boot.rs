//! `examples/vercel-ssr` バイナリの実プロセス起動検証（イシュー #3289）。
//!
//! `examples/dist-server-docker/tests/boot.rs` を雛形にした、本サンプル単体で
//! 完結する最小版。本サンプルは root workspace から独立した単独パッケージ
//! （`Cargo.toml` の `[workspace] members = ["."]` 参照）のため共有ヘルパは
//! 参照できず、std のみで自己完結する実装とする（外部 dev-dependency は
//! 追加しない、`Cargo.toml` の `[dev-dependencies]` は空のまま）。
//!
//! `GET /` の 200・`GET /no-such-page` の 404 に加え、`PORT` の不正値が
//! fail-closed に拒否され、その際 stderr へ実際の値を漏らさないこと
//! （`security.md` A09）も固定する。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 子プロセスを確実に kill・wait する RAII ガード。アサート失敗時の
/// panic でも子プロセスがゾンビ化しないようにする。
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// バイナリを `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）で起動し、
/// `PORT` は明示的に未設定へ倒す（両方設定されていると `FANDHE_FRONTEND_BIND_ADDR`
/// が優先されるため意味は変わらないが、`env_remove` で優先順位の前提を
/// 明示する）。stderr の `listening on` 行から実際に割り当てられたポート
/// 番号を読み取る。タイムアウト（5 秒）を超えても起動ログが得られない場合は
/// テストを失敗させる。
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

    // 読み取りスレッドは検出後も本体側から join しない（detach する）。
    // 本関数はポートが見つかり次第 return するため、join すると子プロセスの
    // 後続出力を待ち続けてしまい、タイムアウト対策そのものが無意味になる。
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: プロセスが起動前に終了した
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
            Err(_) => break, // タイムアウトまたは送信側が終了（EOF）
        }
    }

    let port = port.unwrap_or_else(|| {
        let _ = child.kill();
        panic!("failed to read listening port from stderr within timeout: {collected}");
    });

    (ChildGuard(child), port)
}

/// `vercel-ssr-example: listening on 127.0.0.1:PORT` 行からポート番号を
/// 抽出する（`src/main.rs` の固定ログ書式に依存する）。
fn extract_port(line: &str) -> Option<u16> {
    let addr = line.trim().rsplit(' ').next()?;
    let (_, port_str) = addr.rsplit_once(':')?;
    port_str.trim().parse().ok()
}

/// 素の `TcpStream` で `GET <path> HTTP/1.1` を送り、応答全体を文字列として
/// 返す（外部 HTTP クライアント依存を追加しない、上記モジュール doc 参照）。
fn get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("must set read timeout");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )
    .expect("must write request");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("must read response as UTF-8");
    response
}

/// `child` の終了をタイムアウト付きで待つ（無期限ブロックによる CI ハング
/// を避ける）。
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

#[test]
fn get_root_returns_200() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = get(port, "/");

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "GET / must return 200: {response}"
    );
}

#[test]
fn get_unknown_path_returns_404() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = get(port, "/no-such-page");

    assert!(
        response.starts_with("HTTP/1.1 404"),
        "GET /no-such-page must return 404: {response}"
    );
}

/// `PORT` に不正値（非数値）を与えて起動すると fail-closed に非 0 終了し、
/// かつ stderr に実際の不正値（`abc`）を含めないことを確認する
/// （`security.md` A09、`FANDHE_FRONTEND_BIND_ADDR` は未設定のため優先順位 2
/// が成立する経路）。
#[test]
fn invalid_port_env_fails_closed_without_leaking_the_value() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssr"))
        .env_remove("FANDHE_FRONTEND_BIND_ADDR")
        .env("PORT", "abc")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vercel-ssr example binary must spawn");

    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr must be piped")
        .read_to_string(&mut stderr)
        .expect("stderr must be readable as UTF-8");

    let status = wait_with_timeout(&mut child, Duration::from_secs(5));

    assert!(
        !status.success(),
        "invalid PORT must cause a non-zero exit, got {status:?}"
    );
    assert!(
        !stderr.contains("abc"),
        "stderr must not leak the invalid PORT value: {stderr}"
    );
    assert!(
        stderr.contains("PORT"),
        "stderr must mention PORT as the failure category: {stderr}"
    );
}
