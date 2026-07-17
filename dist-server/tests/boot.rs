//! `rws-dist-server` バイナリの実プロセス起動検証（TASK-9.1c、イシュー #97）。
//!
//! `tests/routes.rs` はハンドラレベル（`routes::route_request` 呼び出し）に
//! 留め、実際の TCP bind・hyper 接続処理は検証しない設計だった
//! （`tests/routes.rs` 冒頭 doc 参照）。本ファイルはそのギャップを埋め、
//! `env!("CARGO_BIN_EXE_dist-server")` で実バイナリを子プロセスとして起動し、
//! 素の `TcpStream` で HTTP/1.1 リクエストを送ることで、`main.rs`（トランス
//! ポート層）と `routes::route_request`（ルーティングコア）の結合が実際の
//! プロセス境界を越えて機能することを固定する。
//!
//! 外部 dev-dependency（reqwest 等）は追加しない（`Cargo.toml` の
//! `[dev-dependencies]` は空のまま — REQ-3 の趣旨、`dist-server/Cargo.toml`
//! 冒頭コメント参照）。プロセス起動・HTTP 通信はすべて `std` のみで行う。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 子プロセスを確実に終了させる Drop ガード。
///
/// テストがアサート失敗・panic で早期リターンしても、`dist-server` の
/// 子プロセスがゾンビ化・ポート占有したまま CI 上に残らないようにする
/// （実装計画のセキュリティ考慮「プロセス管理」参照）。
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // 終了操作の失敗（既に終了済み等）はテスト結果に影響しないため無視する。
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// `RWS_BIND_ADDR` にポート 0（OS 割当）を指定して `dist-server` を起動し、
/// stderr の `listening on` 行から実際に割り当てられたポートを読み取る。
///
/// タイムアウト（5 秒）以内に当該行が出力されない場合は panic する
/// （テストコードでの panic は `coding-rust.md` の対象外 — ライブラリコード
/// のみ禁止）。
fn spawn_and_wait_for_port() -> (ChildGuard, u16) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_dist-server"))
        .env("RWS_BIND_ADDR", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dist-server binary must spawn");

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    let reader = BufReader::new(stderr);

    let port = read_listening_port(reader);
    (ChildGuard(child), port)
}

/// stderr から `"rws-dist-server: listening on 127.0.0.1:<port>"` 行を探し、
/// `<port>` を返す。5 秒待っても見つからなければ panic する。
///
/// `BufReader::read_line` は子プロセスの `ChildStderr` パイプに対する
/// ブロッキング呼び出しで、読み取り自体にタイムアウトを設定する手段が
/// 標準ライブラリにはない（`TcpStream::set_read_timeout` 相当が存在しない）。
/// そのため実際の読み取りは別スレッドへ切り出し、本スレッドは
/// `mpsc::Receiver::recv_timeout` でデッドラインまで待つことでタイムアウトを
/// 実効化する（レビュー指摘: パイプ読み取りが無期限にブロックし CI がハング
/// しうる問題への対応）。
fn read_listening_port(reader: BufReader<std::process::ChildStderr>) -> u16 {
    let (tx, rx) = mpsc::channel::<String>();

    // 読み取りスレッドは検出後も本体側から join しない（detach する）。
    // 本関数はポートが見つかり次第 return するため、join すると子プロセスの
    // 後続出力を待ち続けてしまい、タイムアウト対策そのものが無意味になる。
    // 受信側が既に return してチャネルが閉じている場合は素直に終了する。
    std::thread::spawn(move || {
        let mut reader = reader;
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

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if let Some(port) = parse_listening_port(line.trim_end()) {
                    return port;
                }
            }
            // タイムアウト・読み取りスレッド終了（EOF/エラー）はいずれも
            // 「該当行が見つからなかった」扱いとして panic へフォールスルーする。
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    panic!("dist-server did not print a \"listening on\" line with a port within timeout");
}

/// `"rws-dist-server: listening on 127.0.0.1:PORT"` 形式の 1 行から
/// ポート番号を抽出する（`main.rs` の起動ログ契約に対応）。
fn parse_listening_port(line: &str) -> Option<u16> {
    let addr = line.strip_prefix("rws-dist-server: listening on ")?;
    let port_str = addr.rsplit(':').next()?;
    port_str.parse::<u16>().ok()
}

/// `127.0.0.1:port` へ TCP 接続し、素の HTTP/1.1 リクエストを送って
/// レスポンス全体（ヘッダ + ボディ）を文字列で返す。
fn send_http_request(port: u16, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to dist-server");
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

/// レスポンス文字列の先頭ステータス行（`HTTP/1.1 200 OK` 等）からステータス
/// コードを取り出す。
fn status_code(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("response must start with a valid HTTP status line")
}

#[test]
fn get_root_returns_200_with_escaped_xss_payload() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = send_http_request(port, "GET", "/");

    assert_eq!(status_code(&response), 200);
    assert!(
        response.contains("&lt;script&gt;"),
        "list page must escape the XSS payload by default (REQ-1): {response}"
    );
}

#[test]
fn get_static_asset_returns_200() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = send_http_request(port, "GET", "/static/view-transitions.js");

    assert_eq!(status_code(&response), 200);
}

#[test]
fn path_traversal_against_static_assets_returns_404() {
    let (_guard, port) = spawn_and_wait_for_port();

    let response = send_http_request(port, "GET", "/static/../Cargo.toml");

    assert_eq!(status_code(&response), 404);
}

#[test]
fn bind_conflict_exits_non_zero_with_fixed_stderr_message() {
    // 先に素の `TcpListener` でポートを 1 つ占有し、`dist-server` に同じ
    // アドレスを明示指定させて bind 失敗を再現する。
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").expect("must bind a probe port");
    let occupied_addr = occupied.local_addr().expect("must read local_addr");

    let mut child = Command::new(env!("CARGO_BIN_EXE_dist-server"))
        .env("RWS_BIND_ADDR", occupied_addr.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dist-server binary must spawn");

    let status = child.wait().expect("dist-server process must exit");
    assert!(
        !status.success(),
        "dist-server must exit non-zero when the bind address is already in use"
    );

    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child")
        .read_to_string(&mut stderr)
        .expect("stderr must be readable as UTF-8");

    // 機微情報（内部パス・スタックトレース等）を含まない固定文言のみで
    // あることを確認する（`security.md` A09、`main.rs` の doc 参照）。
    assert!(
        stderr.contains("rws-dist-server: failed to bind"),
        "stderr must contain the fixed bind-failure message: {stderr}"
    );
    assert!(
        !stderr.to_lowercase().contains("panic"),
        "bind failure must not panic: {stderr}"
    );

    drop(occupied);
}
