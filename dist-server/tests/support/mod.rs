//! `dist-server/tests/` 配下の統合テストが共有するプロセス起動・HTTP 通信
//! ヘルパー群（TASK-9.4、イシュー #104 でのリファクタで `boot.rs` から抽出）。
//!
//! `tests/` 直下に置いた `.rs` ファイルはそれぞれ独立したテストバイナリとして
//! コンパイルされるため、`boot.rs` と `xss_via_embedded_binary.rs` の双方が
//! 実プロセス起動（`env!("CARGO_BIN_EXE_dist-server")`）・素の `TcpStream`
//! による HTTP 送受信を必要とする。Cargo の慣例（`tests/<name>/mod.rs` は
//! テストターゲットとして扱われず、`mod support;` で明示的に取り込んだ場合
//! のみコンパイルされる）に従い、本モジュールへヘルパーを一本化する。
//!
//! 外部 dev-dependency（reqwest 等）は依然として追加しない
//! （`dist-server/Cargo.toml` の `[dev-dependencies]` は空のまま — REQ-3 の
//! 趣旨）。プロセス起動・HTTP 通信はすべて `std` のみで行う。
//!
//! `wait_with_timeout`（bind 競合検証専用）は `boot.rs` 側にのみ必要なため
//! 本モジュールには含めない。

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
pub struct ChildGuard(pub Child);

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
pub fn spawn_and_wait_for_port() -> (ChildGuard, u16) {
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

    // `read_listening_port` の呼び出し（タイムアウト・panic し得る）より前に
    // `ChildGuard` でラップする。ラップを後回しにすると、その間に panic や
    // 早期リターンが起きた場合、生の `Child` が guard で保護されないまま
    // drop されてしまう（`std::process::Child` は drop 時に自動終了しない
    // ため、子プロセスが起動したままゾンビ化・ポート占有し得る。レビュー指摘対応）。
    let guard = ChildGuard(child);

    let port = read_listening_port(reader);
    (guard, port)
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
pub fn read_listening_port(reader: BufReader<std::process::ChildStderr>) -> u16 {
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
pub fn parse_listening_port(line: &str) -> Option<u16> {
    let addr = line.strip_prefix("rws-dist-server: listening on ")?;
    let port_str = addr.rsplit(':').next()?;
    port_str.parse::<u16>().ok()
}

/// `127.0.0.1:port` へ TCP 接続し、素の HTTP/1.1 リクエストを送って
/// レスポンス全体（ヘッダ + ボディ）を文字列で返す。
pub fn send_http_request(port: u16, method: &str, path: &str) -> String {
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
pub fn status_code(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("response must start with a valid HTTP status line")
}

/// レスポンス文字列からヘッダ部を取り除き、ボディのみを返す。
///
/// ヘッダとボディは空行（`\r\n\r\n`）で区切られる（RFC 9112）。`dist-server`
/// は `Connection: close` の HTTP/1.1 応答を固定長ボディ（`Full<Bytes>`、
/// `main.rs` 参照）で返すため chunked transfer-encoding は使わず、単純な
/// 最初の `\r\n\r\n` 分割でボディ全体を取り出せる
/// （`xss_via_embedded_binary.rs` のバイト列完全一致検証で使用）。
///
/// `boot.rs` は本関数を使わないため、そちらのテストバイナリでは未使用警告
/// が出る。`tests/` 配下の各 `.rs` は独立のテストバイナリとしてコンパイル
/// され、`mod support;` で取り込んだ関数のうち使わないものが出るのは
/// 共有モジュール抽出の構造上避けられないため、`#[allow(dead_code)]` を
/// 明示する（実装計画 3.2 参照）。
#[allow(dead_code)]
pub fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response must contain a header/body separator")
}
