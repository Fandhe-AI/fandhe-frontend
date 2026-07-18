//! `dist-server` の実プロセス起動検証（`tests/boot.rs`・`tests/isolated_run.rs`・
//! `tests/xss_via_embedded_binary.rs`）が共有するヘルパ群
//! （TASK-9.2a / TASK-9.4、イシュー #99 / #104）。
//!
//! 元々は `tests/boot.rs`（TASK-9.1c、イシュー #97）に単体で実装されていたが、
//! `isolated_run.rs`（隔離ディレクトリへコピーしたバイナリを起動する変種）・
//! `xss_via_embedded_binary.rs`（単一バイナリ経由の XSS エスケープ維持検証）が
//! 同じプロセス管理・HTTP 送受信の仕組みを必要としたため、共通部分をこの
//! `tests/support/mod.rs` へ抽出した。各テストファイルからは `mod support;`
//! （`#[path = "support/mod.rs"] mod support;` 不要 — ディレクトリ名がモジュール
//! 名と一致するため通常の `mod` 宣言で解決される）で利用する。
//!
//! `tests/bind_addr.rs`（イシュー #162）は `RWS_BIND_ADDR` によるバインド
//! アドレス・ポートの切り替え自体を検証対象とするため、`127.0.0.1:0`（OS
//! 割当ポート）固定だった従来のヘルパでは検証できない（「指定した値が実際に
//! 反映されたか」を確かめるには任意のアドレスを明示指定できる必要がある）。
//! そのため [`spawn_and_wait_for_port`] は任意アドレスを受け付ける
//! [`spawn_with_bind_addr`] への薄いラッパへ、[`parse_listening_port`] は
//! 完全なアドレス文字列を返す [`parse_listening_addr`] への薄いラッパへ、
//! [`send_http_request`] は接続先ホストを可変化した [`send_http_request_to`]
//! への薄いラッパへ、それぞれ委譲する形にリファクタリングした。既存の公開
//! 関数のシグネチャ・戻り値・呼び出し側からの見え方は変えていない。
//!
//! `spawn_with_bind_addr`・`parse_listening_addr`・`send_http_request_to` は
//! それぞれ既存の `spawn_and_wait_for_port`・`send_http_request` から内部的に
//! 呼ばれるため、`boot.rs` / `isolated_run.rs` のテストバイナリでも間接的に
//! 使われており `dead_code` 警告の対象にならない。一方 `parse_listening_port`
//! は `read_listening_addr` が `parse_listening_addr` を直接使う形に変わった
//! ことで内部呼び出しが無くなったため、`#[allow(dead_code)]` を付けている
//! （本モジュール既存の慣行を踏襲）。
//!
//! 外部 dev-dependency（reqwest 等）は追加しない（`dist-server/Cargo.toml` の
//! `[dev-dependencies]` は空のまま — REQ-3 の趣旨）。プロセス起動・HTTP 通信は
//! すべて `std` のみで行う。
//!
//! # integration test ハーネスの制約について
//!
//! `tests/` 配下の各 `.rs` ファイルは cargo によって独立したテストバイナリへ
//! コンパイルされる。共通モジュールを `mod support;` で複数のテストバイナリ
//! （`boot.rs`・`isolated_run.rs`・`xss_via_embedded_binary.rs`）から読み込むと、
//! 本ファイル中の未使用関数がテストバイナリごとに `dead_code` 警告の対象に
//! なり得る（各バイナリが使う関数の組み合わせが異なるため）。利用側の
//! `mod support;` 宣言に本モジュール全体の未使用を許容する属性は付与せず、
//! 個々の未使用関数へ `#[allow(dead_code)]` を付ける方針とする
//! （`parse_listening_port` 等、上記参照）。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
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

/// `binary` を起動し、`RWS_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）で bind
/// させたうえで stderr の `listening on` 行から実際のポートを読み取る。
///
/// `cwd` を指定すると子プロセスの作業ディレクトリを固定する
/// （`isolated_run.rs` がソースツリーと無関係な隔離ディレクトリを
/// カレントディレクトリとして起動する用途）。`None` の場合は親プロセスの
/// カレントディレクトリを継承する（`boot.rs` の従来動作と同一）。
///
/// タイムアウト（5 秒）以内に当該行が出力されない場合は panic する
/// （テストコードでの panic は `coding-rust.md` の対象外 — ライブラリコード
/// のみ禁止）。
///
/// `boot.rs` / `isolated_run.rs` の従来動作（`127.0.0.1:0` 固定・ポートのみ
/// 返す）を変えないための薄いラッパ。任意のバインドアドレスを指定したい
/// 場合は [`spawn_with_bind_addr`] を直接使う（`tests/bind_addr.rs`）。
///
/// `bind_addr.rs` のテストバイナリでは未使用（同ファイルは任意アドレスを
/// 指定できる [`spawn_with_bind_addr`] を直接使う）ため `#[allow(dead_code)]`
/// で抑止する。
#[allow(dead_code)]
pub fn spawn_and_wait_for_port(binary: &Path, cwd: Option<&Path>) -> (ChildGuard, u16) {
    let (guard, addr) = spawn_with_bind_addr(binary, cwd, "127.0.0.1:0");
    let port = addr
        .rsplit(':')
        .next()
        .and_then(|port_str| port_str.parse::<u16>().ok())
        .expect("listening address must end with a valid port");
    (guard, port)
}

/// `binary` を `RWS_BIND_ADDR=<bind_addr>` で起動し、stderr の
/// `listening on` 行から実際のバインドアドレス（`127.0.0.2:34567` 等の
/// 完全な文字列）を読み取る。
///
/// `tests/bind_addr.rs`（イシュー #162）が `RWS_BIND_ADDR` の値そのものを
/// 明示指定してポート・アドレスの切り替えを検証するために使う。
/// `cwd` は [`spawn_and_wait_for_port`] と同様の意味を持つ。
///
/// [`spawn_and_wait_for_port`] がこの関数を呼ぶため、`boot.rs` /
/// `isolated_run.rs` のテストバイナリでも間接的に使われており
/// `dead_code` 警告の対象にはならない。
pub fn spawn_with_bind_addr(
    binary: &Path,
    cwd: Option<&Path>,
    bind_addr: &str,
) -> (ChildGuard, String) {
    let mut command = Command::new(binary);
    command
        .env("RWS_BIND_ADDR", bind_addr)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let mut child = spawn_with_etxtbsy_retry(&mut command);

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    let reader = BufReader::new(stderr);

    // `read_listening_addr` の呼び出し（タイムアウト・panic し得る）より前に
    // `ChildGuard` でラップする。ラップを後回しにすると、その間に panic や
    // 早期リターンが起きた場合、生の `Child` が guard で保護されないまま
    // drop されてしまう（`std::process::Child` は drop 時に自動終了しない
    // ため、子プロセスが起動したままゾンビ化・ポート占有し得る）。
    let guard = ChildGuard(child);

    let addr = read_listening_addr(reader);
    (guard, addr)
}

/// `Command::spawn` を、`ETXTBSY`（"Text file busy"）に限り短い待機を挟んで
/// リトライする。
///
/// `isolated_run.rs` は「`fs::copy` でバイナリをコピー → 直後に spawn」を
/// 各テストが並列（`cargo test` 既定のマルチスレッド実行）に行う。実測で、
/// コピー先パスは呼び出しごとに一意（衝突なし）であるにもかかわらず、
/// 別スレッドが同時に別のバイナリを fork+exec している最中だと、Linux の
/// カーネルが無関係のパスに対して一時的に `ETXTBSY` を返すことがある
/// （fork 直後・exec 前の書き込み可能な fd 継承に起因する既知の過渡的事象。
/// Go の `exec_test.go` 等でも同種のリトライ対策が取られている）。
/// 対象ファイル自体の破損やロックではなく一過性のカーネル挙動のため、
/// 短い待機を挟んだ再試行で解消する。`ETXTBSY` 以外のエラーは即座に
/// panic する（真の起動失敗を握り潰さないため）。
fn spawn_with_etxtbsy_retry(command: &mut Command) -> Child {
    const MAX_ATTEMPTS: u32 = 20;
    const RETRY_DELAY: Duration = Duration::from_millis(25);

    for attempt in 1..=MAX_ATTEMPTS {
        match command.spawn() {
            Ok(child) => return child,
            Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                if attempt == MAX_ATTEMPTS {
                    panic!(
                        "dist-server binary must spawn (still ETXTBSY after {MAX_ATTEMPTS} attempts): {err}"
                    );
                }
                std::thread::sleep(RETRY_DELAY);
            }
            Err(err) => panic!("dist-server binary must spawn: {err}"),
        }
    }

    unreachable!("loop above always returns or panics");
}

/// stderr から `"rws-dist-server: listening on <addr>"` 行を探し、
/// `<addr>`（`127.0.0.1:34567` 等の完全なアドレス文字列）を返す。
/// 5 秒待っても見つからなければ panic する。
///
/// `BufReader::read_line` は子プロセスの `ChildStderr` パイプに対する
/// ブロッキング呼び出しで、読み取り自体にタイムアウトを設定する手段が
/// 標準ライブラリにはない（`TcpStream::set_read_timeout` 相当が存在しない）。
/// そのため実際の読み取りは別スレッドへ切り出し、本スレッドは
/// `mpsc::Receiver::recv_timeout` でデッドラインまで待つことでタイムアウトを
/// 実効化する（パイプ読み取りが無期限にブロックし CI がハングしうる問題への
/// 対応）。
fn read_listening_addr(reader: BufReader<std::process::ChildStderr>) -> String {
    let (tx, rx) = mpsc::channel::<String>();

    // 読み取りスレッドは検出後も本体側から join しない（detach する）。
    // 本関数はアドレスが見つかり次第 return するため、join すると子プロセスの
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
                if let Some(addr) = parse_listening_addr(line.trim_end()) {
                    return addr.to_string();
                }
            }
            // タイムアウト・読み取りスレッド終了(EOF/エラー)はいずれも
            // 「該当行が見つからなかった」扱いとして panic へフォールスルーする。
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    panic!("dist-server did not print a \"listening on\" line with an address within timeout");
}

/// `child.wait()` を無期限にブロックさせず、タイムアウト（5 秒）付きで
/// 子プロセスの終了を待つ。
///
/// bind 失敗を検証するテスト用。bind 失敗のパスでは子プロセスは即座に
/// 終了するはずだが、想定外に起動が停滞した場合でも `wait()` を無期限に
/// 呼ぶとテストごと CI をハングさせてしまう。`try_wait()` によるポーリング
/// でデッドラインを実効化し、タイムアウト時は panic する（呼び出し元は
/// `ChildGuard` でラップ済みであることが前提 — panic 後も `Drop` で子
/// プロセスの kill/wait が保証される）。
///
/// `isolated_run.rs`・`xss_via_embedded_binary.rs` のテストバイナリでは
/// 未使用（bind 失敗検証は `boot.rs` の
/// `bind_conflict_exits_non_zero_with_fixed_stderr_message` 専用のシナリオ）
/// のため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn wait_with_timeout(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .expect("try_wait on dist-server child must not error")
        {
            return status;
        }
        if Instant::now() >= deadline {
            panic!("dist-server did not exit within timeout after a bind conflict");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `"rws-dist-server: listening on <addr>"` 形式の 1 行から完全なバインド
/// アドレス文字列（`127.0.0.1:34567` 等）を抽出する（`main.rs` の起動ログ
/// 契約に対応）。
///
/// `tests/bind_addr.rs` はこの戻り値のホスト部・ポート部の両方を検証する
/// （`RWS_BIND_ADDR` の値がそのまま反映されたことの直接証明のため）。
pub fn parse_listening_addr(line: &str) -> Option<&str> {
    line.strip_prefix("rws-dist-server: listening on ")
}

/// [`parse_listening_addr`] からポート番号のみを取り出す薄いラッパ。
/// 既存の呼び出し形（戻り値 `Option<u16>`）を変えないために残している。
///
/// `read_listening_addr` が `parse_listening_addr` を直接使う形に変わった
/// ため、現状どのテストバイナリからも呼ばれない（上記モジュール doc 参照）。
#[allow(dead_code)]
pub fn parse_listening_port(line: &str) -> Option<u16> {
    let addr = parse_listening_addr(line)?;
    let port_str = addr.rsplit(':').next()?;
    port_str.parse::<u16>().ok()
}

/// `127.0.0.1:port` へ TCP 接続し、素の HTTP/1.1 リクエストを送って
/// レスポンス全体（ヘッダ + ボディ）を文字列で返す。
///
/// 接続先ホストを固定しない汎用版は [`send_http_request_to`]（`bind_addr.rs`
/// が `127.0.0.2` 等の非既定ホストへ接続する際に使う）。
pub fn send_http_request(port: u16, method: &str, path: &str) -> String {
    send_http_request_to("127.0.0.1", port, method, path)
}

/// [`send_http_request`] の接続先ホストを可変化した版。`tests/bind_addr.rs`
/// が `RWS_BIND_ADDR=127.0.0.2:0` 等、ループバック /8 内の非既定アドレスへの
/// 到達性を検証するために使う。
pub fn send_http_request_to(host: &str, port: u16, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect((host, port)).expect("must connect to dist-server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");

    let request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
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
/// `boot.rs`・`isolated_run.rs` は本関数を使わないため、そちらのテスト
/// バイナリでは未使用警告が出る。`tests/` 配下の各 `.rs` は独立のテスト
/// バイナリとしてコンパイルされ、`mod support;` で取り込んだ関数のうち
/// 使わないものが出るのは共有モジュール抽出の構造上避けられないため、
/// `#[allow(dead_code)]` を明示する。
#[allow(dead_code)]
pub fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response must contain a header/body separator")
}

/// [`send_http_request`] のバイト列版。
///
/// WASM バイナリ（`.wasm`）の応答本文は有効な UTF-8 とは限らない
/// （`\0asm` マジックナンバーを含む）ため、`send_http_request` の
/// `read_to_string`（UTF-8 前提）では読み取りに失敗する。
/// `isolated_run.rs` の WASM アセット検証はこちらを使う。
///
/// `tests/` 配下は各ファイルが独立したテストバイナリへコンパイルされる
/// （`mod support;` を宣言するファイルごとに本モジュールが再コンパイル
/// される）。`boot.rs`・`xss_via_embedded_binary.rs` はこの関数を呼ばない
/// ため、それらのテストバイナリでは常に未使用になり `dead_code` 警告の
/// 対象となる。実際には `isolated_run.rs`（`wasm_assets_embedded` cfg 有効
/// 時）から使われるため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn send_http_request_bytes(port: u16, method: &str, path: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to dist-server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");

    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request must be written");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response must be readable");
    response
}

/// バイト列レスポンスの先頭ステータス行からステータスコードを取り出す
/// （[`status_code`] のバイト列版）。ステータス行自体は常に ASCII のため
/// `str::from_utf8` で先頭行だけを取り出せば十分。
///
/// `boot.rs`・`xss_via_embedded_binary.rs` のテストバイナリでは未使用
/// （`send_http_request_bytes` と同じ理由、上記 doc 参照）のため
/// `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn status_code_bytes(response: &[u8]) -> u16 {
    let header_end = find_header_end(response).unwrap_or(response.len());
    let header_bytes = &response[..header_end];
    let header_str =
        std::str::from_utf8(header_bytes).expect("HTTP header section must be ASCII/UTF-8");
    header_str
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("response must start with a valid HTTP status line")
}

/// バイト列レスポンスからヘッダ部分を除いた本文（ボディ）を取り出す。
/// `Content-Length` は解釈せず、`Connection: close` 前提（本テスト群の
/// リクエストは常に付与している）で接続断までを読み切った
/// [`send_http_request_bytes`] の出力をヘッダ区切り（`\r\n\r\n`）で
/// 単純分割するのみで十分。
///
/// `boot.rs`・`xss_via_embedded_binary.rs` のテストバイナリでは未使用
/// （`send_http_request_bytes` と同じ理由、上記 doc 参照）のため
/// `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn response_body_bytes(response: &[u8]) -> &[u8] {
    match find_header_end(response) {
        Some(header_end) => &response[header_end + 4..],
        None => &[],
    }
}

/// `\r\n\r\n`（ヘッダ / ボディ区切り）の開始インデックスを探す。
#[allow(dead_code)]
fn find_header_end(response: &[u8]) -> Option<usize> {
    response.windows(4).position(|window| window == b"\r\n\r\n")
}
