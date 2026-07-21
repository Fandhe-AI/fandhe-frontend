//! `examples/dist-server-docker` バイナリの実プロセス起動検証（イシュー #502）。
//!
//! `crates/dist-server/tests/boot.rs`（フレームワーク本体の起動検証、共有
//! `tests/support/mod.rs` ヘルパー基盤あり）を雛形にした、本サンプル単体で
//! 完結する最小版。本サンプルは root workspace から独立した単独パッケージ
//! （`Cargo.toml` の `[workspace] members = ["."]` 参照）のため、上記の共有
//! ヘルパーは参照できず、std のみで自己完結する実装とする（外部
//! dev-dependency は追加しない、`Cargo.toml` の `[dev-dependencies]` は空の
//! まま）。
//!
//! 受け入れ条件 1（実装計画）: 生成した単一バイナリを起動し `/` への GET が
//! 200 を返すことを機械的に固定する。`fw gate --project examples/dist-server-docker`
//! の `test` チェック経由でも本ファイルは実行される。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 子プロセスを確実に kill・wait する RAII ガード。アサート失敗時の
/// panic でも子プロセスがゾンビ化しないようにする
/// （`crates/dist-server/tests/support/mod.rs::ChildGuard` と同じ意図）。
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// バイナリを `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）で
/// 起動し、stderr の `listening on` 行から実際に割り当てられたポート番号を
/// 読み取る。タイムアウト（5 秒）を超えても起動ログが得られない場合は
/// テストを失敗させる（無限ハングを避ける、`ci.md` の非対話実行前提）。
///
/// `BufReader::read_line` は子プロセスの `ChildStderr` パイプに対する
/// ブロッキング呼び出しで、読み取り自体にタイムアウトを設定する手段が
/// 標準ライブラリにはない（`TcpStream::set_read_timeout` 相当が存在しない）。
/// 読み取りをそのまま本スレッドで行うと `Instant::now() < deadline` の
/// チェックは呼び出しの「間」でしか働かず、`read()` 自体が返らない限り
/// タイムアウトが実効化しない（`crates/dist-server/tests/support/mod.rs::
/// read_listening_addr` と同じ既知の問題）。そのため読み取りを別スレッドへ
/// 切り出し、本スレッドは `mpsc::Receiver::recv_timeout` でデッドラインまで
/// 待つことでタイムアウトを実効化する。
fn spawn_and_wait_for_port() -> (ChildGuard, u16) {
    let mut child = Command::new(env!(
        "CARGO_BIN_EXE_fandhe-frontend-example-dist-server-docker"
    ))
    .env("FANDHE_FRONTEND_BIND_ADDR", "127.0.0.1:0")
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .spawn()
    .expect("dist-server-docker example binary must spawn");

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");

    let (tx, rx) = mpsc::channel::<String>();

    // 読み取りスレッドは検出後も本体側から join しない（detach する）。
    // 本関数はポートが見つかり次第 return するため、join すると子プロセスの
    // 後続出力を待ち続けてしまい、タイムアウト対策そのものが無意味になる。
    // 受信側が既に return してチャネルが閉じている場合は素直に終了する。
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: プロセスが起動前に終了した
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        break; // 受信側は既に return 済み
                    }
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

/// `dist-server-docker-example: listening on 127.0.0.1:PORT` 行から
/// ポート番号を抽出する（`src/main.rs` の固定ログ書式に依存する、
/// フォーマットが変わった場合はこの関数だけを追随させればよい）。
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
fn get_static_style_css_returns_200_with_css_content_type() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = get(port, "/static/style.css");

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "GET /static/style.css must return 200: {response}"
    );
    assert!(
        response.to_lowercase().contains("content-type: text/css"),
        "GET /static/style.css must set a text/css Content-Type: {response}"
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
